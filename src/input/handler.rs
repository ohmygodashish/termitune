use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone)]
pub enum KeyAction {
    Quit,
    Up,
    Down,
    Left,
    Right,
    Enter,
    Backspace,
    Space,
    PlayPause,
    Next,
    Previous,
    VolumeUp,
    VolumeDown,
    SeekForward,
    SeekBackward,
    Search,
    None,
}

pub struct InputHandler;

impl InputHandler {
    pub fn new() -> Self {
        Self
    }

    pub fn handle_key_event(&self, event: KeyEvent) -> KeyAction {
        let KeyEvent {
            code, modifiers, ..
        } = event;

        if modifiers.contains(KeyModifiers::CONTROL) {
            return match code {
                KeyCode::Char('f') | KeyCode::Char('F') => KeyAction::Search,
                _ => KeyAction::None,
            };
        }

        match code {
            KeyCode::Char('q') => KeyAction::Quit,
            KeyCode::Up => KeyAction::Up,
            KeyCode::Down => KeyAction::Down,
            KeyCode::Left => KeyAction::Left,
            KeyCode::Right => KeyAction::Right,
            KeyCode::Enter => KeyAction::Enter,
            KeyCode::Backspace => KeyAction::Backspace,
            KeyCode::Char(' ') => KeyAction::Space,
            KeyCode::Char('p') => KeyAction::PlayPause,
            KeyCode::Char('n') => KeyAction::Next,
            KeyCode::Char('b') => KeyAction::Previous,
            KeyCode::Char('+') | KeyCode::Char('=') => KeyAction::VolumeUp,
            KeyCode::Char('-') => KeyAction::VolumeDown,
            KeyCode::Char('>') => KeyAction::SeekForward,
            KeyCode::Char('<') => KeyAction::SeekBackward,
            KeyCode::Char('/') | KeyCode::Char('?') => KeyAction::Search,
            _ => KeyAction::None,
        }
    }
}

impl Default for InputHandler {
    fn default() -> Self {
        Self::new()
    }
}
