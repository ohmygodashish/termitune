use crate::audio::metadata::TrackMetadata;
use log::info;
use std::collections::VecDeque;

#[derive(Clone)]
pub struct QueuedTrack {
    pub track: TrackMetadata,
    pub manually_added: bool,
}

pub struct QueueManager {
    queue: VecDeque<QueuedTrack>,
    current_index: Option<usize>,
    last_manual_insert_index: Option<usize>,
}

impl QueueManager {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            current_index: None,
            last_manual_insert_index: None,
        }
    }

    pub fn add(&mut self, track: TrackMetadata, manually_added: bool) {
        info!(
            "Adding to queue (manual={}): {}",
            manually_added, track.title
        );
        self.queue.push_back(QueuedTrack {
            track,
            manually_added,
        });

        if self.current_index.is_none() {
            self.current_index = Some(0);
        }
    }

    pub fn add_multiple(&mut self, tracks: Vec<TrackMetadata>, manually_added: bool) {
        for track in tracks {
            self.add(track, manually_added);
        }
    }

    pub fn insert_after_current(&mut self, track: TrackMetadata) {
        let insert_pos = if let Some(last_manual) = self.last_manual_insert_index {
            if let Some(current) = self.current_index {
                if last_manual >= current {
                    last_manual + 1
                } else {
                    current + 1
                }
            } else {
                last_manual + 1
            }
        } else {
            self.current_index.map(|i| i + 1).unwrap_or(0)
        };

        self.queue.insert(
            insert_pos,
            QueuedTrack {
                track,
                manually_added: true,
            },
        );

        self.last_manual_insert_index = Some(insert_pos);

        if let Some(current) = self.current_index {
            if insert_pos <= current {
                self.current_index = Some(current + 1);
            }
        }
    }

    pub fn remove(&mut self, index: usize) -> Option<QueuedTrack> {
        if index < self.queue.len() {
            let track = self.queue.remove(index)?;

            if let Some(current) = self.current_index {
                if index < current {
                    self.current_index = Some(current - 1);
                } else if index == current && current >= self.queue.len() {
                    self.current_index = Some(current.saturating_sub(1));
                }
            }

            Some(track)
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        self.queue.clear();
        self.current_index = None;
        self.last_manual_insert_index = None;
    }

    pub fn next_track(&mut self) -> Option<&QueuedTrack> {
        if let Some(current) = self.current_index {
            let next_index = current + 1;
            if next_index < self.queue.len() {
                self.current_index = Some(next_index);

                if let Some(last_manual) = self.last_manual_insert_index {
                    if next_index > last_manual {
                        self.last_manual_insert_index = Some(next_index);
                    }
                }

                return self.queue.get(next_index);
            }
        }
        None
    }

    pub fn previous(&mut self) -> Option<&QueuedTrack> {
        if let Some(current) = self.current_index {
            if current > 0 {
                self.current_index = Some(current - 1);
                return self.queue.get(current - 1);
            }
        }
        None
    }

    pub fn current(&self) -> Option<&QueuedTrack> {
        if let Some(index) = self.current_index {
            self.queue.get(index)
        } else {
            None
        }
    }

    pub fn current_index(&self) -> Option<usize> {
        self.current_index
    }

    pub fn get_track(&self, index: usize) -> Option<&QueuedTrack> {
        self.queue.get(index)
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    pub fn move_up(&mut self, index: usize) -> bool {
        if index > 0 && index < self.queue.len() {
            self.queue.swap(index, index - 1);

            if let Some(current) = self.current_index {
                if current == index {
                    self.current_index = Some(index - 1);
                } else if current == index - 1 {
                    self.current_index = Some(index);
                }
            }

            true
        } else {
            false
        }
    }

    pub fn move_down(&mut self, index: usize) -> bool {
        if index + 1 < self.queue.len() {
            self.queue.swap(index, index + 1);

            if let Some(current) = self.current_index {
                if current == index {
                    self.current_index = Some(index + 1);
                } else if current == index + 1 {
                    self.current_index = Some(index);
                }
            }

            true
        } else {
            false
        }
    }
}

impl Default for QueueManager {
    fn default() -> Self {
        Self::new()
    }
}
