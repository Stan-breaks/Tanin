use crate::app::App;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render_downloads_view(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Length(3), // Name
            Constraint::Length(3), // Category
            Constraint::Length(3), // Icon
            Constraint::Length(3), // URL
            Constraint::Length(1), // Status
            Constraint::Min(0),    // Queue
        ])
        .margin(1)
        .split(area);

    f.render_widget(
        Paragraph::new("Add Sound via yt-dlp")
            .style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Cyan),
            )
            .alignment(Alignment::Center),
        chunks[0],
    );

    let inputs = [
        ("Name", &app.add_sound_name),
        ("Category", &app.add_sound_category),
        ("Icon", &app.add_sound_icon),
        ("URL", &app.add_sound_url),
    ];

    for (i, (label, value)) in inputs.iter().enumerate() {
        let is_focused = app.add_sound_focus_index == i;
        let style = if is_focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::White)
        };

        // Handle Suggestion rendering for Category (i == 1)
        let block = Block::default()
            .borders(Borders::ALL)
            .title(*label)
            .border_style(style);

        if i == 1 && is_focused {
            if let Some(suggestion) = &app.add_sound_suggestion {
                if suggestion.to_lowercase().starts_with(&value.to_lowercase()) {
                    let typed_len = value.len();
                    let suggested_part = &suggestion[typed_len..];

                    let spans = vec![
                        Span::raw(value.as_str()),
                        Span::styled(suggested_part, Style::default().fg(Color::DarkGray)),
                    ];
                    f.render_widget(
                        Paragraph::new(Line::from(spans)).block(block),
                        chunks[i + 1],
                    );
                } else {
                    f.render_widget(Paragraph::new(value.as_str()).block(block), chunks[i + 1]);
                }
            } else {
                f.render_widget(Paragraph::new(value.as_str()).block(block), chunks[i + 1]);
            }
        } else {
            f.render_widget(Paragraph::new(value.as_str()).block(block), chunks[i + 1]);
        }
    }

    // Status
    let status_color = if app.add_sound_status.starts_with("Error") {
        Color::Red
    } else {
        Color::Green
    };
    f.render_widget(
        Paragraph::new(app.add_sound_status.as_str()).style(Style::default().fg(status_color)),
        chunks[5],
    );

    // Queue Rendering
    // We use a List or Table. Let's use List for simplicity.

    let queue_block = Block::default()
        .borders(Borders::ALL)
        .title(" Download Queue ");
    f.render_widget(queue_block, chunks[6]);

    let queue_area = chunks[6].inner(ratatui::layout::Margin {
        horizontal: 1,
        vertical: 1,
    });

    let mut items = Vec::new();
    if app.download_queue.is_empty() {
        items.push(Line::from(Span::raw("Queue is empty.")));
    } else {
        for task in &app.download_queue {
            let status_span = match &task.status {
                crate::app::DownloadStatus::Pending => {
                    Span::styled("Pending", Style::default().fg(Color::DarkGray))
                }
                crate::app::DownloadStatus::Downloading(p) => Span::styled(
                    format!("Downloading {:.1}%", p),
                    Style::default().fg(Color::Yellow),
                ),
                crate::app::DownloadStatus::Done => {
                    Span::styled("Done", Style::default().fg(Color::Green))
                }
                crate::app::DownloadStatus::Error(e) => {
                    Span::styled(format!("Error: {}", e), Style::default().fg(Color::Red))
                }
            };

            items.push(Line::from(vec![
                Span::styled(
                    format!("{:<20} ", task.name),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                status_span,
            ]));
        }
    }

    f.render_widget(ratatui::widgets::Paragraph::new(items), queue_area);
}
