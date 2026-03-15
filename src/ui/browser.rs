use crate::ui::app::App;
use ratatui::prelude::*;
use ratatui::widgets::{Block, List, ListItem, ListState};

pub fn render_browser(app: &App, frame: &mut Frame, area: Rect) {
    let block = Block::bordered()
        .title("File Browser")
        .border_type(ratatui::widgets::BorderType::Rounded);

    let items: Vec<ListItem> = app
        .entries
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let content = entry.display_text();

            if i == app.selected_index {
                ListItem::new(content).style(Style::default().bg(Color::LightBlue).fg(Color::Black))
            } else {
                ListItem::new(content)
            }
        })
        .collect();

    let list = List::new(items).block(block);

    let mut state = ListState::default();
    state.select(Some(app.selected_index));

    frame.render_stateful_widget(list, area, &mut state);
}
