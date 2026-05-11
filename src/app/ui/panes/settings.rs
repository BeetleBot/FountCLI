use crate::app::App;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};
use super::centered_rect;

pub fn draw_settings_modal(f: &mut Frame, app: &mut App, area: Rect) {
    let theme = &app.theme;
    let mode_bg = Color::from(theme.ui.normal_mode_bg.clone());

    let modal_area = centered_rect(40, 50, area);
    f.render_widget(Clear, modal_area);

    let block = Block::default()
        .title(" [ Settings ] ")
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(theme.secondary_style())
        .style(theme.normal_style());
    f.render_widget(block, modal_area);

    let inner_area = modal_area.inner(ratatui::layout::Margin {
        horizontal: 2,
        vertical: 1,
    });

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),    // Options
            Constraint::Length(1), // Footer hint
        ])
        .split(inner_area);

    let mut options = Vec::new();

    let render_option = |label: &str, value: &str, is_selected: bool| -> ListItem {
        let style = if is_selected {
            Style::default()
                .fg(Color::from(theme.ui.selection_fg.clone()))
                .bg(Color::from(theme.ui.selection_bg.clone()))
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        let val_style = if is_selected {
            style
        } else {
            Style::default().fg(mode_bg).add_modifier(Modifier::BOLD)
        };

        ListItem::new(Line::from(vec![
            Span::styled(if is_selected { " > " } else { "   " }, style),
            Span::styled(format!("{:<20}", label), style),
            Span::styled(value.to_string(), val_style),
        ]))
    };

    // List of settings options (sync with execute_command /set)
    options.push(render_option("Focus Mode", if app.config.focus_mode { "[ON]" } else { "[OFF]" }, app.selected_setting == 0));
    options.push(render_option("Line Numbers", if app.config.show_line_numbers { "[ON]" } else { "[OFF]" }, app.selected_setting == 1));
    options.push(render_option("Typewriter Mode", if app.config.typewriter_mode { "[ON]" } else { "[OFF]" }, app.selected_setting == 2));
    options.push(render_option("Show Markup", if !app.config.hide_markup { "[ON]" } else { "[OFF]" }, app.selected_setting == 3));
    options.push(render_option("Highlight Block", if app.config.highlight_active_action { "[ON]" } else { "[OFF]" }, app.selected_setting == 4));
    options.push(render_option("Page Numbers", if app.config.show_page_numbers { "[ON]" } else { "[OFF]" }, app.selected_setting == 5));
    options.push(render_option("Scene Numbers", if app.config.show_scene_numbers { "[ON]" } else { "[OFF]" }, app.selected_setting == 6));
    options.push(render_option("Auto (CONT'D)", if app.config.auto_contd { "[ON]" } else { "[OFF]" }, app.selected_setting == 7));

    let auto_save_val = if !app.config.auto_save {
        "[OFF]".to_string()
    } else {
        match app.config.auto_save_interval {
            60 => "[1 min]".to_string(),
            180 => "[3 min]".to_string(),
            300 => "[5 min]".to_string(),
            600 => "[10 min]".to_string(),
            v => format!("[{}s]", v),
        }
    };
    options.push(render_option("Auto-Save", &auto_save_val, app.selected_setting == 8));

    options.push(render_option("Autocomplete", if app.config.autocomplete { "[ON]" } else { "[OFF]" }, app.selected_setting == 9));
    options.push(render_option("Smart Breaks", if app.config.auto_paragraph_breaks { "[ON]" } else { "[OFF]" }, app.selected_setting == 10));

    f.render_widget(List::new(options), layout[0]);

    // Footer
    let footer_text = Line::from(vec![
        Span::styled(" [^/v] ", Style::default().fg(mode_bg).add_modifier(Modifier::BOLD)),
        Span::styled("Select  ", theme.secondary_style()),
        Span::styled(" [Enter/Space] ", Style::default().fg(mode_bg).add_modifier(Modifier::BOLD)),
        Span::styled("Toggle Option", theme.secondary_style()),
    ]);
    f.render_widget(Paragraph::new(footer_text).alignment(ratatui::layout::Alignment::Center), layout[1]);
}
