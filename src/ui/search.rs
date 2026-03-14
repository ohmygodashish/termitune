use crate::ui::app::{App, SearchScope};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph};

pub fn render_search(app: &App, frame: &mut Frame) {
    if !app.search.active {
        return;
    }

    let area = centered_rect(frame.area(), 70, 60);

    let result_count = app.search.results.len();
    let title = match app.search.scope() {
        SearchScope::Local => format!("Search (Music Root, {result_count} results)"),
        SearchScope::Global => format!("Search (Global, {result_count} results)"),
    };

    frame.render_widget(Clear, area);

    let outer_block = Block::default().borders(Borders::ALL).title(title);
    let inner = outer_block.inner(area);
    frame.render_widget(outer_block, area);

    let sections = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(5),
        Constraint::Length(1),
    ])
    .split(inner);

    let prompt = Paragraph::new(format!("> {}", app.search.query))
        .block(Block::default().borders(Borders::BOTTOM).title("Query"));
    frame.render_widget(prompt, sections[0]);

    let mut items: Vec<ListItem> = if app.search.results.is_empty() {
        vec![ListItem::new("No matches").style(Style::default().fg(Color::DarkGray))]
    } else {
        app.search
            .results
            .iter()
            .skip(app.search.scroll)
            .take(sections[1].height.saturating_sub(2) as usize)
            .enumerate()
            .map(|(visible_index, result)| {
                let line = highlighted_result_line(
                    result.display(),
                    &app.search.normalized_query(),
                    app.search.selected_index == app.search.scroll + visible_index,
                );
                ListItem::new(line)
            })
            .collect()
    };

    if items.is_empty() {
        items.push(ListItem::new("No matches").style(Style::default().fg(Color::DarkGray)));
    }

    let list = List::new(items).block(Block::default());
    let mut state = ListState::default();
    if !app.search.results.is_empty() {
        state.select(Some(
            app.search.selected_index.saturating_sub(app.search.scroll),
        ));
    }

    frame.render_stateful_widget(
        list.highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        ),
        sections[1],
        &mut state,
    );

    let help = Paragraph::new(format!(
        "{} shown  Enter queue  Space type  * toggle scope  Esc close",
        result_count,
    ))
    .alignment(Alignment::Center)
    .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(help, sections[2]);
}

fn highlighted_result_line(text: &str, query: &str, selected: bool) -> Line<'static> {
    let mut spans = Vec::new();
    let highlight_mask = build_highlight_mask(text, query);
    let chars: Vec<char> = text.chars().collect();

    if chars.is_empty() {
        return Line::from(spans);
    }

    let mut buffer = String::new();
    let mut current_highlight = highlight_mask.first().copied().unwrap_or(false);

    for (index, ch) in chars.iter().enumerate() {
        let is_highlighted = highlight_mask.get(index).copied().unwrap_or(false);
        if index > 0 && is_highlighted != current_highlight {
            spans.push(styled_span(&buffer, current_highlight, selected));
            buffer.clear();
            current_highlight = is_highlighted;
        }
        buffer.push(*ch);
    }

    if !buffer.is_empty() {
        spans.push(styled_span(&buffer, current_highlight, selected));
    }

    Line::from(spans)
}

fn styled_span(text: &str, highlighted: bool, selected: bool) -> Span<'static> {
    if highlighted {
        Span::styled(
            text.to_string(),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )
    } else if selected {
        Span::styled(text.to_string(), Style::default().fg(Color::White))
    } else {
        Span::raw(text.to_string())
    }
}

fn build_highlight_mask(text: &str, query: &str) -> Vec<bool> {
    let chars: Vec<char> = text.chars().collect();
    let mut mask = vec![false; chars.len()];
    if chars.is_empty() {
        return mask;
    }

    let tokens: Vec<&str> = query
        .split_whitespace()
        .filter(|token| !token.is_empty())
        .collect();

    if tokens.is_empty() {
        return mask;
    }

    let lower_chars: Vec<char> = text.to_lowercase().chars().collect();

    for token in tokens {
        if let Some((start, len)) = contiguous_match_range(&lower_chars, token) {
            for index in start..start + len {
                if let Some(slot) = mask.get_mut(index) {
                    *slot = true;
                }
            }
            continue;
        }

        for index in subsequence_match_indices(&lower_chars, token) {
            if let Some(slot) = mask.get_mut(index) {
                *slot = true;
            }
        }
    }

    mask
}

fn contiguous_match_range(chars: &[char], token: &str) -> Option<(usize, usize)> {
    let token_chars: Vec<char> = token.chars().collect();
    if token_chars.is_empty() || token_chars.len() > chars.len() {
        return None;
    }

    chars
        .windows(token_chars.len())
        .position(|window| window == token_chars.as_slice())
        .map(|start| (start, token_chars.len()))
}

fn subsequence_match_indices(chars: &[char], token: &str) -> Vec<usize> {
    let mut indices = Vec::new();
    let mut search_start = 0;

    for needle in token.chars() {
        let Some(relative_index) = chars[search_start..].iter().position(|ch| *ch == needle) else {
            return Vec::new();
        };

        let absolute_index = search_start + relative_index;
        indices.push(absolute_index);
        search_start = absolute_index + 1;
    }

    indices
}

fn centered_rect(area: Rect, width_percent: u16, height_percent: u16) -> Rect {
    let vertical = Layout::vertical([
        Constraint::Percentage((100 - height_percent) / 2),
        Constraint::Percentage(height_percent),
        Constraint::Percentage((100 - height_percent) / 2),
    ])
    .split(area);

    Layout::horizontal([
        Constraint::Percentage((100 - width_percent) / 2),
        Constraint::Percentage(width_percent),
        Constraint::Percentage((100 - width_percent) / 2),
    ])
    .split(vertical[1])[1]
}
