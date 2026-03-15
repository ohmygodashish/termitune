# termitune

A fast, lightweight, keyboard-driven terminal music player in Rust for unix systems.

## Features

- **File Browser** - Navigate directories and browse audio files
- **Audio Metadata** - Displays title, artist, and duration
- **Queue System** - Add tracks to queue and manage playback
- **Playback Controls** - Play/pause, next/previous track
- **Volume Control** - Adjust volume with +/- keys
- **Progress Bar** - Track progress with time display
- **Automatic Queue Progression** - Automatically advances to next track when current finishes
- **Persistent Volume** - Volume settings persist across track changes

## Installation

```bash
cargo install termitune
```
**NOTE:** If cargo is missing, then refer: [Building From Source](#building-from-source)

## Building From Source

### 1. Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 2. Install System Dependencies

#### Debian/Ubuntu

```bash
sudo apt-get update
sudo apt-get install build-essential pkg-config libasound2-dev
```

#### Fedora

```bash
sudo dnf update
sudo dnf install gcc pkgconfig alsa-lib-devel
```

#### Arch Linux

```bash
sudo pacman -Sy
sudo pacman -S base-devel alsa-lib
```

#### macOS (untested)

```bash
brew install pkg-config
# Note: Additional audio library setup may be required on macOS
```

### 3. Build & Run

```bash
# Clone the repository
git clone https://github.com/ohmygodashish/termitune.git
cd termitune

# Build release version
cargo build --release

# Run
cargo run --release
```

## Keyboard Shortcuts

| Action | Key |
|--------|-----|
| Move up/down | Up/Down arrows |
| Enter directory / Play track | Enter or Right arrow |
| Go back | Backspace or Left arrow |
| Add to queue | Space |
| Play/Pause | p |
| Next track | n |
| Previous track | b |
| Volume up | + |
| Volume down | - |
| Quit | q |

## Supported Audio Formats

- MP3
- FLAC
- WAV
- OGG
- M4A
- AAC
- WMA

## License

[MIT License](LICENSE)
