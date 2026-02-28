use crate::audio::metadata::TrackMetadata;
use log::info;
use std::collections::VecDeque;

pub struct QueueManager {
    queue: VecDeque<TrackMetadata>,
    current_index: Option<usize>,
}

impl QueueManager {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            current_index: None,
        }
    }

    pub fn add(&mut self, track: TrackMetadata) {
        info!("Adding to queue: {}", track.title);
        self.queue.push_back(track);

        if self.current_index.is_none() {
            self.current_index = Some(0);
        }
    }

    #[allow(dead_code)]
    pub fn add_multiple(&mut self, tracks: Vec<TrackMetadata>) {
        for track in tracks {
            self.add(track);
        }
    }

    pub fn remove(&mut self, index: usize) -> Option<TrackMetadata> {
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

    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.queue.clear();
        self.current_index = None;
    }

    pub fn next_track(&mut self) -> Option<&TrackMetadata> {
        if let Some(current) = self.current_index {
            let next_index = current + 1;
            if next_index < self.queue.len() {
                self.current_index = Some(next_index);
                return self.queue.get(next_index);
            }
        }
        None
    }

    pub fn previous(&mut self) -> Option<&TrackMetadata> {
        if let Some(current) = self.current_index {
            if current > 0 {
                self.current_index = Some(current - 1);
                return self.queue.get(current - 1);
            }
        }
        None
    }

    pub fn current(&self) -> Option<&TrackMetadata> {
        if let Some(index) = self.current_index {
            self.queue.get(index)
        } else {
            None
        }
    }

    pub fn current_index(&self) -> Option<usize> {
        self.current_index
    }

    pub fn get_track(&self, index: usize) -> Option<&TrackMetadata> {
        self.queue.get(index)
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    #[allow(dead_code)]
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

    #[allow(dead_code)]
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
