pub mod app;
pub mod browser;
pub mod player;
pub mod queue_panel;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::Rect;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::time::Duration;

use crate::input::handler::KeyAction;
use crate::ui::app::{App, AppError};
use crate::ui::browser::render_browser;
use crate::ui::player::render_player;
use crate::ui::queue_panel::render_queue;

pub fn run() -> Result<(), AppError> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new()?;

    let res = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {}", err);
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<(), AppError> {
    loop {
        terminal.draw(|f| {
            let size = f.area();

            let browser_width = (size.width as f32 * 0.5) as u16;
            let player_height = 10;
            let queue_height = size.height.saturating_sub(player_height);

            let browser_area = Rect::new(0, 0, browser_width, queue_height);
            let queue_area = Rect::new(browser_width, 0, size.width, queue_height);
            let player_area = Rect::new(0, queue_height, size.width, player_height);

            render_browser(app, f, browser_area);
            render_queue(app, f, queue_area);
            render_player(app, f, player_area);
        })?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    let action = app.input_handler.handle_key_event(key);

                    match action {
                        KeyAction::Quit => break,
                        KeyAction::Up => app.navigate_up(),
                        KeyAction::Down => app.navigate_down(),
                        KeyAction::Left => {
                            if let Err(e) = app.go_back() {
                                log::warn!("Failed to go back: {}", e);
                            }
                        }
                        KeyAction::Right => {
                            if let Err(e) = app.handle_enter_key() {
                                log::warn!("Failed to handle enter: {}", e);
                            }
                        }
                        KeyAction::Space => app.add_to_queue(),
                        KeyAction::Enter => {
                            if let Err(e) = app.handle_enter_key() {
                                log::warn!("Failed to handle enter: {}", e);
                            }
                        }
                        KeyAction::Backspace => {
                            if let Err(e) = app.go_back() {
                                log::warn!("Failed to go back: {}", e);
                            }
                        }
                        KeyAction::PlayPause => app.toggle_playback(),
                        KeyAction::Next => app.next_track(),
                        KeyAction::Previous => app.previous_track(),
                        KeyAction::VolumeUp => app.volume_up(),
                        KeyAction::VolumeDown => app.volume_down(),
                        KeyAction::SeekForward => {}
                        KeyAction::SeekBackward => {}
                        KeyAction::Search => {}
                        KeyAction::None => {}
                    }
                }
            }
        }

        app.check_and_play_next();
    }

    Ok(())
}
