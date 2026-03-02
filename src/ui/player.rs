use crate::ui::app::App;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Gauge, Paragraph};
use std::time::Duration;

pub fn render_player(app: &App, frame: &mut Frame, area: Rect) {
    let block = Block::bordered()
        .title("Now Playing")
        .border_type(ratatui::widgets::BorderType::Rounded);

    let current_track = app.queue.current();

    let title = if let Some(queued) = current_track {
        format!("{} - {}", queued.track.artist, queued.track.title)
    } else {
        "No track playing".to_string()
    };

    let status = if app.player.is_playing() {
        "Playing"
    } else if app.player.is_paused() {
        "Paused"
    } else {
        "Stopped"
    };

    let volume = app.player.volume();
    let volume_percent = (volume * 100.0) as u16;

    let elapsed = app.player.elapsed_time();
    let duration = app.player.duration();

    let (progress_bar, progress_percent) = calculate_progress(elapsed, duration);

    let paragraph = Paragraph::new(vec![
        Line::from(title),
        Line::from(format!("Status: {}", status)),
        Line::from(""),
    ])
    .block(block)
    .alignment(ratatui::layout::Alignment::Center);

    frame.render_widget(paragraph, area);

    let progress_area = Rect::new(area.x + 2, area.y + 4, area.width - 4, 1);
    let progress_text = Paragraph::new(progress_bar).alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(progress_text, progress_area);

    let gauge_area = Rect::new(area.x + 2, area.y + 5, area.width - 4, 1);
    let gauge = Gauge::default()
        .gauge_style(Style::default().fg(Color::Cyan))
        .percent(progress_percent);
    frame.render_widget(gauge, gauge_area);

    let volume_area = Rect::new(area.x + 2, area.y + 7, 15, 1);
    let volume_text = Paragraph::new(format!("Vol: {}%", volume_percent))
        .alignment(ratatui::layout::Alignment::Left);
    frame.render_widget(volume_text, volume_area);

    let quit_padding = 2;
    let quit_hint_width = 19;
    let quit_hint_area = Rect::new(
        area.x + area.width.saturating_sub(quit_hint_width + quit_padding),
        area.y + 7,
        quit_hint_width,
        1,
    );
    let quit_hint =
        Paragraph::new("Press 'q' to quit").alignment(ratatui::layout::Alignment::Right);
    frame.render_widget(quit_hint, quit_hint_area);
}

fn calculate_progress(elapsed: Option<Duration>, duration: Option<Duration>) -> (String, u16) {
    match (elapsed, duration) {
        (Some(elapsed), Some(total)) if total.as_secs() > 0 => {
            let elapsed_capped = if elapsed > total { total } else { elapsed };
            let percent = ((elapsed_capped.as_secs() as f64 / total.as_secs() as f64) * 100.0)
                .min(100.0) as u16;
            let progress_bar =
                format!("[{} / {}]", format_time(elapsed_capped), format_time(total));
            (progress_bar, percent)
        }
        (Some(_), Some(_)) => ("[00:00 / 00:00]".to_string(), 0),
        _ => ("[--:-- / --:--]".to_string(), 0),
    }
}

fn format_time(duration: Duration) -> String {
    let secs = duration.as_secs();
    let minutes = secs / 60;
    let seconds = secs % 60;
    format!("{:02}:{:02}", minutes, seconds)
}
