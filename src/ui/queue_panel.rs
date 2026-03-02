use crate::ui::app::App;
use ratatui::prelude::*;
use ratatui::widgets::{Block, List, ListItem, ListState};

pub fn render_queue(app: &App, frame: &mut Frame, area: Rect) {
    let block = Block::bordered()
        .title("Queue")
        .border_type(ratatui::widgets::BorderType::Rounded);

    let items: Vec<ListItem> = (0..app.queue.len())
        .enumerate()
        .map(|(i, _)| {
            if let Some(queued) = app.queue.get_track(i) {
                let current_marker = if Some(i) == app.queue.current_index() {
                    "▶ "
                } else {
                    "  "
                };

                let content = format!(
                    "{}{} - {}",
                    current_marker, queued.track.artist, queued.track.title
                );

                let is_current = Some(i) == app.queue.current_index();
                let is_manual = queued.manually_added;

                if is_current {
                    ListItem::new(content)
                        .style(Style::default().bg(Color::LightBlue).fg(Color::Black))
                } else if is_manual {
                    ListItem::new(content).style(Style::default().fg(Color::Yellow))
                } else {
                    ListItem::new(content)
                }
            } else {
                ListItem::new("")
            }
        })
        .collect();

    let list = List::new(items).block(block);

    let mut state = ListState::default();
    state.select(Some(app.queue_scroll));

    frame.render_stateful_widget(list, area, &mut state);
}
