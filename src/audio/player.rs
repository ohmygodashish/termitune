use log::info;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

pub struct AudioPlayer {
    _stream: OutputStream,
    _stream_handle: OutputStreamHandle,
    sink: Sink,
    current_track: Option<PathBuf>,
    current_duration: Option<Duration>,
    playback_start: Option<Instant>,
    pause_start_time: Option<Instant>,
    total_paused_duration: Duration,
    is_paused: Arc<AtomicBool>,
    has_played: bool,
}

impl AudioPlayer {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let (stream, stream_handle) = OutputStream::try_default()
            .map_err(|e| format!("Failed to get audio output: {}", e))?;
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
            is_paused: Arc::new(AtomicBool::new(false)),
            has_played: false,
        })
    }

    pub fn play(
        &mut self,
        path: &PathBuf,
        duration: Option<Duration>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!("Playing: {:?}", path);

        self.stop();

        let file = File::open(path)?;
        let source = Decoder::new(BufReader::new(file))?;

        self.sink.append(source);
        self.current_track = Some(path.clone());
        self.current_duration = duration;
        self.playback_start = Some(Instant::now());
        self.pause_start_time = None;
        self.total_paused_duration = Duration::ZERO;
        self.has_played = true;

        Ok(())
    }

    pub fn pause(&mut self) {
        if !self.sink.is_paused() {
            self.pause_start_time = Some(Instant::now());
        }
        self.sink.pause();
        self.is_paused.store(true, Ordering::SeqCst);
    }

    pub fn resume(&mut self) {
        if let Some(pause_start) = self.pause_start_time {
            let paused_duration = pause_start.elapsed();
            self.total_paused_duration += paused_duration;
            self.pause_start_time = None;
        }
        self.sink.play();
        self.is_paused.store(false, Ordering::SeqCst);
    }

    pub fn stop(&self) {
        self.sink.stop();
    }

    pub fn is_playing(&self) -> bool {
        !self.sink.is_paused() && !self.sink.empty()
    }

    pub fn is_paused(&self) -> bool {
        self.sink.is_paused()
    }

    pub fn is_finished(&self) -> bool {
        self.has_played && self.sink.empty() && !self.is_playing()
    }

    #[allow(dead_code)]
    pub fn current_track(&self) -> Option<&PathBuf> {
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
    pub fn seek_forward(&mut self, _duration: Duration) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    #[allow(dead_code)]
    pub fn seek_backward(&mut self, _duration: Duration) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
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
