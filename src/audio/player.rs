use log::info;
use rodio::{Decoder, OutputStream, OutputStreamHandle, PlayError, Sink, StreamError};
use std::fs::File;
use std::io::{self, BufReader};
use std::path::Path;
use std::time::{Duration, Instant};

#[derive(Debug)]
pub enum PlayerError {
    OutputStream(StreamError),
    Sink(PlayError),
    Io(io::Error),
    Decoder(rodio::decoder::DecoderError),
}

impl From<io::Error> for PlayerError {
    fn from(err: io::Error) -> Self {
        PlayerError::Io(err)
    }
}

impl From<rodio::decoder::DecoderError> for PlayerError {
    fn from(err: rodio::decoder::DecoderError) -> Self {
        PlayerError::Decoder(err)
    }
}

impl From<StreamError> for PlayerError {
    fn from(err: StreamError) -> Self {
        PlayerError::OutputStream(err)
    }
}

impl From<PlayError> for PlayerError {
    fn from(err: PlayError) -> Self {
        PlayerError::Sink(err)
    }
}

impl std::fmt::Display for PlayerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlayerError::OutputStream(e) => write!(f, "Output stream error: {}", e),
            PlayerError::Sink(e) => write!(f, "Sink error: {}", e),
            PlayerError::Io(e) => write!(f, "IO error: {}", e),
            PlayerError::Decoder(e) => write!(f, "Decoder error: {}", e),
        }
    }
}

impl std::error::Error for PlayerError {}

pub struct AudioPlayer {
    _stream: OutputStream,
    _stream_handle: OutputStreamHandle,
    sink: Sink,
    current_track: Option<std::path::PathBuf>,
    current_duration: Option<Duration>,
    playback_start: Option<Instant>,
    pause_start_time: Option<Instant>,
    total_paused_duration: Duration,
}

impl AudioPlayer {
    pub fn new() -> Result<Self, PlayerError> {
        let (stream, stream_handle) = OutputStream::try_default()?;
        let sink = Sink::try_new(&stream_handle)?;

        Ok(Self {
            _stream: stream,
            _stream_handle: stream_handle,
            sink,
            current_track: None,
            current_duration: None,
            playback_start: None,
            pause_start_time: None,
            total_paused_duration: Duration::ZERO,
        })
    }

    pub fn play(&mut self, path: &Path, duration: Option<Duration>) -> Result<(), PlayerError> {
        info!("Playing: {:?}", path);

        self.stop();

        let file = File::open(path)?;
        let source = Decoder::new(BufReader::new(file))?;

        self.sink.append(source);
        self.current_track = Some(path.to_path_buf());
        self.current_duration = duration;
        self.playback_start = Some(Instant::now());
        self.pause_start_time = None;
        self.total_paused_duration = Duration::ZERO;

        Ok(())
    }

    pub fn pause(&mut self) {
        if !self.sink.is_paused() {
            self.pause_start_time = Some(Instant::now());
        }
        self.sink.pause();
    }

    pub fn resume(&mut self) {
        if let Some(pause_start) = self.pause_start_time {
            let paused_duration = pause_start.elapsed();
            self.total_paused_duration += paused_duration;
            self.pause_start_time = None;
        }
        self.sink.play();
    }

    pub fn stop(&mut self) {
        self.sink.stop();
        self.playback_start = None;
        self.pause_start_time = None;
        self.total_paused_duration = Duration::ZERO;
        self.current_track = None;
        self.current_duration = None;
    }

    pub fn is_playing(&self) -> bool {
        !self.sink.is_paused() && !self.sink.empty()
    }

    pub fn is_paused(&self) -> bool {
        self.sink.is_paused()
    }

    pub fn is_finished(&self) -> bool {
        self.sink.empty() && !self.is_playing() && self.current_track.is_none()
    }

    #[allow(dead_code)]
    pub fn current_track(&self) -> Option<&std::path::PathBuf> {
        self.current_track.as_ref()
    }

    pub fn set_volume(&self, volume: f32) {
        self.sink.set_volume(volume.clamp(0.0, 1.0));
    }

    pub fn volume(&self) -> f32 {
        self.sink.volume()
    }

    pub fn elapsed_time(&self) -> Option<Duration> {
        if let Some(start) = self.playback_start {
            let mut elapsed = start.elapsed();
            elapsed = elapsed.saturating_sub(self.total_paused_duration);

            if let Some(pause_start) = self.pause_start_time {
                elapsed = elapsed.saturating_sub(pause_start.elapsed());
            }

            Some(elapsed)
        } else {
            None
        }
    }

    pub fn duration(&self) -> Option<Duration> {
        self.current_duration
    }

    #[allow(dead_code)]
    pub fn empty(&self) -> bool {
        self.sink.empty()
    }
}

impl Default for AudioPlayer {
    fn default() -> Self {
        Self::new().expect("Failed to create audio player")
    }
}
