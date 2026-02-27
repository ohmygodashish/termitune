use crate::audio::metadata::{is_audio_file, TrackMetadata};
use crate::audio::player::AudioPlayer;
use crate::input::handler::InputHandler;
use crate::queue::manager::QueueManager;
use std::path::PathBuf;
use walkdir::WalkDir;

#[allow(dead_code)]
pub struct App {
    pub current_dir: PathBuf,
    pub entries: Vec<DirEntry>,
    pub selected_index: usize,
    pub queue: QueueManager,
    pub player: AudioPlayer,
    pub input_handler: InputHandler,
    pub queue_scroll: usize,
    pub search_query: String,
    pub is_searching: bool,
    pub browser_scroll: usize,
}

#[derive(Debug, Clone)]
pub enum DirEntry {
    Directory(PathBuf),
    File(PathBuf, Option<TrackMetadata>),
}

impl App {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let current_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));

        let mut app = Self {
            current_dir: current_dir.clone(),
            entries: Vec::new(),
            selected_index: 0,
            queue: QueueManager::new(),
            player: AudioPlayer::new()?,
            input_handler: InputHandler::new(),
            queue_scroll: 0,
            search_query: String::new(),
            is_searching: false,
            browser_scroll: 0,
        };

        app.load_directory(&current_dir)?;
        Ok(app)
    }

    pub fn load_directory(&mut self, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        self.entries.clear();
        self.selected_index = 0;
        self.browser_scroll = 0;
        self.current_dir = path.clone();

        let mut dirs: Vec<DirEntry> = Vec::new();
        let mut files: Vec<DirEntry> = Vec::new();

        for entry in WalkDir::new(path).max_depth(1).min_depth(1) {
            let entry = entry?;
            let path = entry.path().to_path_buf();

            if path.is_dir() {
                dirs.push(DirEntry::Directory(path));
            } else if is_audio_file(&path) {
                let metadata = TrackMetadata::from_path(&path);
                files.push(DirEntry::File(path, metadata));
            }
        }

        dirs.sort_by(|a, b| {
            let name_a = match a {
                DirEntry::Directory(p) => p.file_name().unwrap_or_default().to_string_lossy(),
                DirEntry::File(p, _) => p.file_name().unwrap_or_default().to_string_lossy(),
            };
            let name_b = match b {
                DirEntry::Directory(p) => p.file_name().unwrap_or_default().to_string_lossy(),
                DirEntry::File(p, _) => p.file_name().unwrap_or_default().to_string_lossy(),
            };
            name_a.to_lowercase().cmp(&name_b.to_lowercase())
        });

        files.sort_by(|a, b| {
            let name_a = match a {
                DirEntry::Directory(_) => String::new(),
                DirEntry::File(p, _) => p
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
            };
            let name_b = match b {
                DirEntry::Directory(_) => String::new(),
                DirEntry::File(p, _) => p
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
            };
            name_a.to_lowercase().cmp(&name_b.to_lowercase())
        });

        self.entries = dirs;
        self.entries.extend(files);

        Ok(())
    }

    pub fn navigate_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            if self.selected_index < self.browser_scroll {
                self.browser_scroll = self.selected_index;
            }
        }
    }

    pub fn navigate_down(&mut self) {
        if self.selected_index < self.entries.len().saturating_sub(1) {
            self.selected_index += 1;
            if self.selected_index >= self.browser_scroll + 20 {
                self.browser_scroll = self.selected_index - 19;
            }
        }
    }

    #[allow(dead_code)]
    pub fn enter_directory(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let entry = self.entries.get(self.selected_index).cloned();
        if let Some(entry) = entry {
            if let DirEntry::Directory(path) = entry {
                self.load_directory(&path)?;
            }
        }
        Ok(())
    }

    pub fn handle_enter_key(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let entry = self.entries.get(self.selected_index).cloned();
        if let Some(entry) = entry {
            match entry {
                DirEntry::Directory(path) => {
                    self.load_directory(&path)?;
                }
                DirEntry::File(path, _) => {
                    if let Some(metadata) = TrackMetadata::from_path(&path) {
                        self.player.stop();
                        self.queue.clear();
                        self.queue.add(metadata);
                        if let Some(track) = self.queue.current() {
                            let _ = self.player.play(&track.path, Some(track.duration));
                        }
                    }
                }
            }
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub fn play_selected_track(&mut self) {
        let entry = self.entries.get(self.selected_index).cloned();
        if let Some(entry) = entry {
            if let DirEntry::File(path, _) = entry {
                if let Some(metadata) = TrackMetadata::from_path(&path) {
                    self.queue.add(metadata);
                    if let Some(track) = self.queue.current() {
                        let _ = self.player.play(&track.path, Some(track.duration));
                    }
                }
            }
        }
    }

    pub fn go_back(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(parent) = self.current_dir.parent() {
            self.load_directory(&parent.to_path_buf())?;
        }
        Ok(())
    }

    pub fn add_to_queue(&mut self) {
        if let Some(entry) = self.entries.get(self.selected_index) {
            match entry {
                DirEntry::File(path, _) => {
                    if let Some(metadata) = TrackMetadata::from_path(path) {
                        self.queue.add(metadata);
                    }
                }
                _ => {}
            }
        }
    }

    pub fn toggle_playback(&mut self) {
        if self.player.is_playing() {
            self.player.pause();
        } else if self.player.is_paused() {
            self.player.resume();
        } else if !self.queue.is_empty() {
            if let Some(track) = self.queue.current() {
                let _ = self.player.play(&track.path, Some(track.duration));
            }
        }
    }

    #[allow(dead_code)]
    pub fn play_track(&mut self) {
        if let Some(track) = self.queue.current() {
            let _ = self.player.play(&track.path, Some(track.duration));
        }
    }

    pub fn next_track(&mut self) {
        if let Some(track) = self.queue.next() {
            let _ = self.player.play(&track.path, Some(track.duration));
        }
    }

    pub fn check_and_play_next(&mut self) {
        if self.player.is_finished() {
            self.next_track();
        }
    }

    pub fn previous_track(&mut self) {
        if let Some(track) = self.queue.previous() {
            let _ = self.player.play(&track.path, Some(track.duration));
        }
    }

    pub fn volume_up(&mut self) {
        let new_volume = (self.player.volume() + 0.1).min(1.0);
        self.player.set_volume(new_volume);
    }

    pub fn volume_down(&mut self) {
        let new_volume = (self.player.volume() - 0.1).max(0.0);
        self.player.set_volume(new_volume);
    }

    #[allow(dead_code)]
    pub fn queue_up(&mut self) {
        if let Some(current) = self.queue.current_index() {
            if current > 0 {
                self.queue.move_up(current);
            }
        }
    }

    #[allow(dead_code)]
    pub fn queue_down(&mut self) {
        if let Some(current) = self.queue.current_index() {
            self.queue.move_down(current);
        }
    }

    #[allow(dead_code)]
    pub fn remove_from_queue(&mut self, index: usize) {
        self.queue.remove(index);
    }

    #[allow(dead_code)]
    pub fn clear_queue(&mut self) {
        self.queue.clear();
        self.player.stop();
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new().expect("Failed to create app")
    }
}
