use crate::app::{App, CurrentView};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render_footer(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(Color::DarkGray));
    f.render_widget(block, area);

    let inner_area = Rect::new(area.x, area.y + 1, area.width, 1);

    let mute_status = if app.muted {
        Span::styled(
            "🔇 MUTED",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )
    } else {
        Span::raw("🔊")
    };

    let vol_slider_width = 12;
    let knob_pos = if vol_slider_width > 0 {
        (app.session.global_volume * (vol_slider_width - 1) as f32).round() as usize
    } else {
        0
    };
    let mut slider = String::new();
    for i in 0..vol_slider_width {
        if i == knob_pos {
            slider.push('●');
        } else if i < knob_pos {
            slider.push('━');
        } else {
            slider.push('─');
        }
    }

    let master_vol_spans = vec![
        mute_status,
        Span::raw("  Master "),
        Span::styled(slider, Style::default().fg(Color::Blue)),
        Span::raw(format!(
            " {:>3}%",
            (app.session.global_volume * 100.0) as u32
        )),
    ];

    let mut left_content = vec![Span::raw(" ")]; // Padding
    left_content.extend(master_vol_spans);

    if let Some(name) = &app.active_preset {
        left_content.push(Span::raw("  │  "));
        left_content.push(Span::styled(
            format!("Preset: {}", name),
            Style::default().fg(Color::Cyan),
        ));
    }

    let master_vol = Line::from(left_content);

    // Dynamic help text based on view
    let help_text = match app.view {
        CurrentView::Presets => {
            if app.preset_input_mode {
                "Enter: Confirm  Esc: Cancel"
            } else {
                "n: New  r: Rename  u: Update  d: Delete  Enter: Load  Tab: Sounds  q: Quit"
            }
        }
        CurrentView::Downloads => "Enter: Queue Download  Tab: Switch View  q: Quit",
        _ => "Tab: Presets  SPACE: Toggle  m: Mute  ?: Help  q: Quit",
    };

    let p_left = Paragraph::new(master_vol).alignment(Alignment::Left);
    let p_right = Paragraph::new(Span::styled(
        help_text,
        Style::default().fg(Color::DarkGray),
    ))
    .alignment(Alignment::Right);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(inner_area);

    f.render_widget(p_left, chunks[0]);
    f.render_widget(p_right, chunks[1]);
}
