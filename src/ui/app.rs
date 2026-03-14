use crate::audio::metadata::{is_audio_file, TrackMetadata};
use crate::audio::player::{AudioPlayer, PlayerError};
use crate::input::handler::InputHandler;
use crate::queue::manager::QueueManager;
use std::fs::{self, ReadDir};
use std::io;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const TITLE_WEIGHT: i64 = 140;
const ARTIST_WEIGHT: i64 = 110;
const FILE_NAME_WEIGHT: i64 = 120;
const ALBUM_WEIGHT: i64 = 80;
const PATH_WEIGHT: i64 = 60;
const DIR_NAME_WEIGHT: i64 = 120;

#[derive(Debug)]
pub enum AppError {
    Io(io::Error),
    NoHomeDirectory,
    AudioPlayer(PlayerError),
}

impl From<io::Error> for AppError {
    fn from(err: io::Error) -> Self {
        AppError::Io(err)
    }
}

impl From<PlayerError> for AppError {
    fn from(err: PlayerError) -> Self {
        AppError::AudioPlayer(err)
    }
}

impl std::error::Error for AppError {}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Io(e) => write!(f, "IO error: {}", e),
            AppError::NoHomeDirectory => write!(f, "Could not determine home directory"),
            AppError::AudioPlayer(e) => write!(f, "Audio player error: {}", e),
        }
    }
}

#[derive(Debug, Clone)]
pub enum DirEntry {
    Directory(PathBuf),
    File(PathBuf, Option<TrackMetadata>),
}

impl DirEntry {
    pub fn display_text(&self) -> String {
        match self {
            DirEntry::Directory(path) => {
                format!(
                    "📁 {}",
                    path.file_name().unwrap_or_default().to_string_lossy()
                )
            }
            DirEntry::File(path, metadata) => {
                if let Some(meta) = metadata {
                    format!(
                        "🎵 {} - {} [{}]",
                        meta.title,
                        meta.artist,
                        meta.format_duration()
                    )
                } else {
                    format!(
                        "🎵 {}",
                        path.file_name().unwrap_or_default().to_string_lossy()
                    )
                }
            }
        }
    }

    fn to_local_search_result(&self) -> SearchResult {
        match self {
            DirEntry::Directory(path) => SearchResult::Directory {
                path: path.clone(),
                display: self.display_text(),
            },
            DirEntry::File(path, metadata) => SearchResult::File {
                path: path.clone(),
                metadata: metadata.clone(),
                display: self.display_text(),
            },
        }
    }

    fn local_search_score(&self, query: &str) -> Option<i64> {
        if query.is_empty() {
            return Some(0);
        }

        match self {
            DirEntry::Directory(path) => score_query(
                query,
                &[(
                    path.file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .as_ref(),
                    DIR_NAME_WEIGHT,
                )],
            ),
            DirEntry::File(path, metadata) => {
                let file_name = path.file_name().unwrap_or_default().to_string_lossy();
                let title = metadata.as_ref().map(|m| m.title.as_str()).unwrap_or("");
                let artist = metadata.as_ref().map(|m| m.artist.as_str()).unwrap_or("");
                let album = metadata.as_ref().map(|m| m.album.as_str()).unwrap_or("");

                score_query(
                    query,
                    &[
                        (title, TITLE_WEIGHT),
                        (artist, ARTIST_WEIGHT),
                        (file_name.as_ref(), FILE_NAME_WEIGHT),
                        (album, ALBUM_WEIGHT),
                    ],
                )
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchScope {
    Local,
    Global,
}

#[derive(Debug, Clone)]
pub enum SearchResult {
    Directory {
        path: PathBuf,
        display: String,
    },
    File {
        path: PathBuf,
        metadata: Option<TrackMetadata>,
        display: String,
    },
}

impl SearchResult {
    pub fn display(&self) -> &str {
        match self {
            SearchResult::Directory { display, .. } => display,
            SearchResult::File { display, .. } => display,
        }
    }

    fn metadata(&self) -> Option<TrackMetadata> {
        match self {
            SearchResult::File { path, metadata, .. } => {
                metadata.clone().or_else(|| TrackMetadata::from_path(path))
            }
            SearchResult::Directory { .. } => None,
        }
    }
}

#[derive(Debug, Clone)]
struct SearchCandidate {
    result: SearchResult,
    score_fields: Vec<(String, i64)>,
}

impl SearchCandidate {
    fn score(&self, query: &str) -> Option<i64> {
        score_query(
            query,
            &self
                .score_fields
                .iter()
                .map(|(field, weight)| (field.as_str(), *weight))
                .collect::<Vec<_>>(),
        )
    }
}

#[derive(Debug, Clone)]
struct GlobalSearchCache {
    root: PathBuf,
    candidates: Vec<SearchCandidate>,
}

#[derive(Debug, Clone)]
pub struct SearchState {
    pub active: bool,
    pub scope: SearchScope,
    pub query: String,
    pub results: Vec<SearchResult>,
    pub selected_index: usize,
    pub scroll: usize,
}

impl Default for SearchState {
    fn default() -> Self {
        Self {
            active: false,
            scope: SearchScope::Local,
            query: String::new(),
            results: Vec::new(),
            selected_index: 0,
            scroll: 0,
        }
    }
}

impl SearchState {
    pub fn scope(&self) -> SearchScope {
        self.scope
    }

    pub fn normalized_query(&self) -> String {
        self.query.trim().to_lowercase()
    }
}

pub struct App {
    pub music_dir: PathBuf,
    pub current_dir: PathBuf,
    pub entries: Vec<DirEntry>,
    pub selected_index: usize,
    pub queue: QueueManager,
    pub player: AudioPlayer,
    pub input_handler: InputHandler,
    pub queue_scroll: usize,
    pub search: SearchState,
    pub browser_scroll: usize,
    global_search_cache: Option<GlobalSearchCache>,
}

impl App {
    pub fn new() -> Result<Self, AppError> {
        let home_dir = dirs::home_dir().ok_or(AppError::NoHomeDirectory)?;
        let music_dir = dirs::audio_dir().unwrap_or_else(|| home_dir.join("Music"));
        fs::create_dir_all(&music_dir)?;

        let mut app = Self {
            music_dir: music_dir.clone(),
            current_dir: music_dir.clone(),
            entries: Vec::new(),
            selected_index: 0,
            queue: QueueManager::new(),
            player: AudioPlayer::new()?,
            input_handler: InputHandler::new(),
            queue_scroll: 0,
            search: SearchState::default(),
            browser_scroll: 0,
            global_search_cache: None,
        };

        app.load_directory(&music_dir)?;
        Ok(app)
    }

    pub fn load_directory(&mut self, path: &Path) -> Result<(), AppError> {
        let target_path = if path.starts_with(&self.music_dir) {
            path.to_path_buf()
        } else {
            self.music_dir.clone()
        };

        self.entries.clear();
        self.selected_index = 0;
        self.browser_scroll = 0;
        self.current_dir = target_path.clone();
        self.close_search();
        self.global_search_cache = None;

        self.entries = Self::read_directory_entries(&target_path)?;

        Ok(())
    }

    fn read_directory_entries(path: &Path) -> Result<Vec<DirEntry>, AppError> {
        let mut dirs: Vec<DirEntry> = Vec::new();
        let mut files: Vec<DirEntry> = Vec::new();

        let dir_entries: ReadDir = fs::read_dir(path)?;

        for entry in dir_entries {
            let entry = entry?;
            let file_path = entry.path();

            if file_path.is_dir() {
                dirs.push(DirEntry::Directory(file_path));
            } else if is_audio_file(&file_path) {
                let metadata = TrackMetadata::from_path(&file_path);
                files.push(DirEntry::File(file_path, metadata));
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

        let mut entries = dirs;
        entries.extend(files);

        Ok(entries)
    }

    pub fn open_search(&mut self) {
        self.search.active = true;
        self.search.query.clear();
        self.search.selected_index = 0;
        self.search.scroll = 0;
        self.refresh_search_results();
    }

    pub fn close_search(&mut self) {
        let scope = self.search.scope;
        self.search = SearchState {
            scope,
            ..SearchState::default()
        };
    }

    pub fn toggle_search_scope(&mut self) {
        self.search.scope = match self.search.scope {
            SearchScope::Local => SearchScope::Global,
            SearchScope::Global => SearchScope::Local,
        };
        self.search.selected_index = 0;
        self.search.scroll = 0;
        self.refresh_search_results();
    }

    pub fn refresh_search_results(&mut self) {
        if !self.search.active {
            return;
        }

        let query = self.search.normalized_query();

        self.search.results = match self.search.scope() {
            SearchScope::Local => self.build_local_search_results(&query),
            SearchScope::Global => self.build_global_search_results(&query),
        };

        self.clamp_search_selection();
    }

    pub fn append_search_char(&mut self, ch: char) {
        self.search.query.push(ch);
        self.refresh_search_results();
    }

    pub fn pop_search_char(&mut self) {
        self.search.query.pop();
        self.refresh_search_results();
    }

    pub fn search_up(&mut self) {
        if self.search.selected_index > 0 {
            self.search.selected_index -= 1;
            if self.search.selected_index < self.search.scroll {
                self.search.scroll = self.search.selected_index;
            }
        }
    }

    pub fn search_down(&mut self) {
        if self.search.selected_index < self.search.results.len().saturating_sub(1) {
            self.search.selected_index += 1;
            if self.search.selected_index >= self.search.scroll + 10 {
                self.search.scroll = self.search.selected_index - 9;
            }
        }
    }

    pub fn activate_selected_search_result(&mut self) -> Result<(), AppError> {
        let Some(result) = self.search.results.get(self.search.selected_index).cloned() else {
            return Ok(());
        };

        match result {
            SearchResult::Directory { path, .. } => {
                self.load_directory(&path)?;
            }
            SearchResult::File { .. } => {
                if let Some(metadata) = result.metadata() {
                    self.queue.insert_after_current(metadata);
                }

                self.close_search();
            }
        }

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
    pub fn enter_directory(&mut self) -> Result<(), AppError> {
        if let Some(DirEntry::Directory(path)) = self.entries.get(self.selected_index).cloned() {
            self.load_directory(&path)?;
        }
        Ok(())
    }

    pub fn handle_enter_key(&mut self) -> Result<(), AppError> {
        if let Some(entry) = self.entries.get(self.selected_index).cloned() {
            match entry {
                DirEntry::Directory(path) => {
                    self.load_directory(&path)?;
                }
                DirEntry::File(path, _) => {
                    if let Some(metadata) = TrackMetadata::from_path(&path) {
                        self.player.stop();
                        self.queue.clear();

                        let selected_idx = self.selected_index;

                        let mut remaining_songs: Vec<TrackMetadata> = Vec::new();
                        for i in (selected_idx + 1)..self.entries.len() {
                            if let Some(DirEntry::File(p, _)) = self.entries.get(i) {
                                if let Some(m) = TrackMetadata::from_path(p) {
                                    remaining_songs.push(m);
                                }
                            }
                        }

                        self.queue.add(metadata.clone(), false);
                        self.queue.add_multiple(remaining_songs, false);

                        if let Some(queued) = self.queue.current() {
                            let _ = self
                                .player
                                .play(&queued.track.path, Some(queued.track.duration));
                        }
                    }
                }
            }
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub fn play_selected_track(&mut self) {
        if let Some(DirEntry::File(path, _)) = self.entries.get(self.selected_index).cloned() {
            if let Some(metadata) = TrackMetadata::from_path(&path) {
                self.queue.add(metadata, true);
                if let Some(queued) = self.queue.current() {
                    let _ = self
                        .player
                        .play(&queued.track.path, Some(queued.track.duration));
                }
            }
        }
    }

    pub fn go_back(&mut self) -> Result<(), AppError> {
        if let Some(parent) = self.current_dir.parent().map(|p| p.to_path_buf()) {
            if parent.starts_with(&self.music_dir) {
                self.load_directory(&parent)?;
            }
        }
        Ok(())
    }

    pub fn add_to_queue(&mut self) {
        if let Some(DirEntry::File(path, _)) = self.entries.get(self.selected_index) {
            if let Some(metadata) = TrackMetadata::from_path(path) {
                self.queue.insert_after_current(metadata);
            }
        }
    }

    pub fn toggle_playback(&mut self) {
        if self.player.is_playing() {
            self.player.pause();
        } else if self.player.is_paused() {
            self.player.resume();
        } else if !self.queue.is_empty() {
            if let Some(queued) = self.queue.current() {
                let _ = self
                    .player
                    .play(&queued.track.path, Some(queued.track.duration));
            }
        }
    }

    #[allow(dead_code)]
    pub fn play_track(&mut self) {
        if let Some(queued) = self.queue.current() {
            let _ = self
                .player
                .play(&queued.track.path, Some(queued.track.duration));
        }
    }

    pub fn next_track(&mut self) {
        if !self.player.is_playing() && !self.player.is_paused() {
            if let Some(queued) = self.queue.current() {
                let _ = self
                    .player
                    .play(&queued.track.path, Some(queued.track.duration));
                return;
            }
        }

        if let Some(queued) = self.queue.next_track() {
            let _ = self
                .player
                .play(&queued.track.path, Some(queued.track.duration));
        }
    }

    pub fn check_and_play_next(&mut self) {
        if self.player.is_finished() {
            self.next_track();
        }
    }

    pub fn previous_track(&mut self) {
        if let Some(queued) = self.queue.previous() {
            let _ = self
                .player
                .play(&queued.track.path, Some(queued.track.duration));
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

    fn clamp_search_selection(&mut self) {
        if self.search.results.is_empty() {
            self.search.selected_index = 0;
            self.search.scroll = 0;
            return;
        }

        self.search.selected_index = self
            .search
            .selected_index
            .min(self.search.results.len().saturating_sub(1));
        self.search.scroll = self.search.scroll.min(self.search.selected_index);
    }

    fn build_global_search_results(&mut self, query: &str) -> Vec<SearchResult> {
        let mut scored: Vec<(usize, i64, SearchResult)> = self
            .global_search_candidates()
            .iter()
            .enumerate()
            .filter_map(|(index, candidate)| {
                let score = if query.is_empty() {
                    Some(0)
                } else {
                    candidate.score(query)
                }?;

                Some((index, score, candidate.result.clone()))
            })
            .collect();

        if !query.is_empty() {
            scored.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        }

        scored.into_iter().map(|(_, _, result)| result).collect()
    }

    fn relative_path_display(&self, path: &Path) -> String {
        path.strip_prefix(&self.music_dir)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string()
    }

    fn build_local_search_results(&self, query: &str) -> Vec<SearchResult> {
        let Ok(entries) = Self::read_directory_entries(&self.music_dir) else {
            return Vec::new();
        };

        let mut scored: Vec<(usize, i64, SearchResult)> = entries
            .iter()
            .enumerate()
            .filter_map(|(index, entry)| {
                let score = entry.local_search_score(query)?;
                Some((index, score, entry.to_local_search_result()))
            })
            .collect();

        if !query.is_empty() {
            scored.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        }

        scored.into_iter().map(|(_, _, result)| result).collect()
    }

    fn global_search_candidates(&mut self) -> Vec<SearchCandidate> {
        if let Some(cache) = &self.global_search_cache {
            if cache.root == self.music_dir {
                return cache.candidates.clone();
            }
        }

        let mut candidates = Vec::new();

        for entry in WalkDir::new(&self.music_dir)
            .min_depth(1)
            .into_iter()
            .filter_map(Result::ok)
        {
            let path = entry.path().to_path_buf();
            let relative_path = self.relative_path_display(&path);

            if entry.file_type().is_dir() {
                let dir_name = path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                candidates.push(SearchCandidate {
                    result: SearchResult::Directory {
                        path,
                        display: format!("📁 {}", relative_path.clone()),
                    },
                    score_fields: vec![(dir_name, DIR_NAME_WEIGHT), (relative_path, PATH_WEIGHT)],
                });
                continue;
            }

            if !is_audio_file(&path) {
                continue;
            }

            let metadata = TrackMetadata::from_path(&path);
            let file_name = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            let mut score_fields = vec![
                (file_name, FILE_NAME_WEIGHT),
                (relative_path.clone(), PATH_WEIGHT),
            ];
            if let Some(metadata) = &metadata {
                score_fields.push((metadata.title.clone(), TITLE_WEIGHT));
                score_fields.push((metadata.artist.clone(), ARTIST_WEIGHT));
                score_fields.push((metadata.album.clone(), ALBUM_WEIGHT));
            }

            let display = format_global_file_display(&relative_path, metadata.as_ref());

            candidates.push(SearchCandidate {
                result: SearchResult::File {
                    path,
                    metadata,
                    display,
                },
                score_fields,
            });
        }

        self.global_search_cache = Some(GlobalSearchCache {
            root: self.music_dir.clone(),
            candidates,
        });

        self.global_search_cache
            .as_ref()
            .expect("global search cache initialized")
            .candidates
            .clone()
    }
}

fn score_query(query: &str, fields: &[(&str, i64)]) -> Option<i64> {
    let normalized_query = query.trim().to_lowercase();
    if normalized_query.is_empty() {
        return Some(0);
    }

    let tokens: Vec<&str> = normalized_query.split_whitespace().collect();
    if tokens.is_empty() {
        return Some(0);
    }

    let mut total_score = 0;

    for token in tokens {
        let token_score = fields
            .iter()
            .filter_map(|(field, weight)| {
                fuzzy_field_score(field, token).map(|score| score * *weight)
            })
            .max();

        match token_score {
            Some(score) => total_score += score,
            None => return None,
        }
    }

    Some(total_score)
}

fn fuzzy_field_score(field: &str, token: &str) -> Option<i64> {
    let field = field.trim().to_lowercase();
    if token.is_empty() {
        return Some(0);
    }
    if field.is_empty() {
        return None;
    }

    if field == token {
        return Some(1_000);
    }

    if field.starts_with(token) {
        return Some(800 - field.len() as i64);
    }

    if let Some(index) = field.find(token) {
        return Some(650 - index as i64 * 5 - (field.len() as i64 - token.len() as i64));
    }

    let mut last_match = None;
    let mut first_match = None;
    let mut gaps = 0;
    let mut chars = field.char_indices();

    for needle in token.chars() {
        let mut matched = None;

        for (index, hay) in chars.by_ref() {
            if hay == needle {
                matched = Some(index);
                break;
            }
        }

        let index = matched?;
        if let Some(previous) = last_match {
            gaps += index.saturating_sub(previous + 1);
        } else {
            first_match = Some(index);
        }
        last_match = Some(index);
    }

    let start = first_match.unwrap_or(0) as i64;
    Some(400 - start * 3 - gaps as i64 * 4 - (field.len() as i64 - token.len() as i64))
}

fn format_global_file_display(relative_path: &str, metadata: Option<&TrackMetadata>) -> String {
    if let Some(metadata) = metadata {
        format!(
            "🎵 {} - {} [{}] ({})",
            metadata.title,
            metadata.artist,
            metadata.format_duration(),
            relative_path
        )
    } else {
        format!("🎵 {}", relative_path)
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new().expect("Failed to create app")
    }
}
