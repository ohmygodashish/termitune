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
use std::error::Error;
use std::time::Duration;

use crate::input::handler::KeyAction;
use crate::ui::app::App;
use crate::ui::browser::render_browser;
use crate::ui::player::render_player;
use crate::ui::queue_panel::render_queue;

pub fn run() -> Result<(), Box<dyn Error>> {
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
) -> Result<(), Box<dyn Error>> {
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
                            let _ = app.go_back();
                        }
                        KeyAction::Right => {
                            let _ = app.handle_enter_key();
                        }
                        KeyAction::Space => app.add_to_queue(),
                        KeyAction::Enter => {
                            let _ = app.handle_enter_key();
                        }
                        KeyAction::Backspace => {
                            let _ = app.go_back();
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
