use crate::app::App;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render_presets(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    let input_style = if app.preset_input_mode {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let (input_title, input_text) = if app.preset_input_mode {
        let title = if app.preset_rename_target.is_some() {
            "Rename Preset"
        } else {
            "Create Preset"
        };
        (title, format!("Name: {}_", app.preset_input_buffer))
    } else {
        (
            "Manage Presets",
            "Press 'n' to create new preset from current sounds".to_string(),
        )
    };

    let p_input = Paragraph::new(input_text)
        .block(Block::default().borders(Borders::ALL).title(input_title))
        .style(input_style);
    f.render_widget(p_input, chunks[0]);

    let presets = &app.presets_config.presets;

    if presets.is_empty() {
        let p_empty = Paragraph::new("No presets saved yet.")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(p_empty, chunks[1]);
        return;
    }

    let list_height = chunks[1].height.saturating_sub(2) as usize; // Subtract borders
    let offset = if app.preset_cursor_pos >= list_height {
        app.preset_cursor_pos - list_height + 1
    } else {
        0
    };

    let mut list_items = Vec::new();
    for (i, preset) in presets.iter().enumerate().skip(offset).take(list_height) {
        let is_selected = i == app.preset_cursor_pos;
        let style = if is_selected {
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        let prefix = if is_selected { "> " } else { "  " };
        let active_sounds_count = preset.sounds.len();

        let line = Line::from(vec![
            Span::styled(prefix, style),
            Span::styled(format!("{} ", preset.name), style),
            Span::styled(
                format!("({} sounds)", active_sounds_count),
                Style::default().fg(Color::DarkGray),
            ),
        ]);
        list_items.push(line);
    }

    let p_list = Paragraph::new(list_items).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Saved Presets"),
    );
    f.render_widget(p_list, chunks[1]);
}
