pub mod home;
pub mod xray;
pub mod index_cards;
pub mod quick_help;
pub mod structure_picker;

use unicode_width::UnicodeWidthStr;
use crate::app::App;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table, List, ListItem},
};

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}

pub fn draw_sprint_stats(f: &mut Frame, app: &mut App) {
    let area = f.area();
    let theme = &app.theme;
    let mode_bg = Color::from(theme.ui.normal_mode_bg.clone());

    let modal_area = centered_rect(80, 60, area);
    f.render_widget(Clear, modal_area);

    let history_block = Block::default()
        .title(" [ Sprint History | Press E to Export ] ")
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(theme.secondary_style())
        .style(theme.normal_style());

    let inner_area = modal_area.inner(ratatui::layout::Margin {
        horizontal: 1,
        vertical: 1,
    });

    let header = Row::new(vec![
        Cell::from("Project"),
        Cell::from("Date"),
        Cell::from("Time"),
        Cell::from("Words"),
        Cell::from("Lines"),
    ])
    .style(
        Style::default()
            .bg(mode_bg)
            .fg(theme.ui.selection_fg.clone().into())
            .add_modifier(Modifier::BOLD),
    );

    let rows: Vec<Row> = app
        .sprint_history
        .iter()
        .map(|s| {
            Row::new(vec![
                Cell::from(s.project_name.clone()),
                Cell::from(s.timestamp.format("%Y-%m-%d").to_string()),
                Cell::from(format!("{}m", s.duration_mins)),
                Cell::from(s.word_count.to_string()),
                Cell::from(s.line_count.to_string()),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(30),
            Constraint::Percentage(20),
            Constraint::Percentage(15),
            Constraint::Percentage(15),
            Constraint::Percentage(20),
        ],
    )
    .header(header)
    .block(history_block)
    .row_highlight_style(Style::default().bg(Color::from(theme.ui.selection_bg.clone())).fg(Color::from(theme.ui.selection_fg.clone())));

    f.render_stateful_widget(table, inner_area, &mut app.sprint_stats_state);
}

pub fn draw_file_picker(f: &mut Frame, app: &mut App, area: Rect) {
    let state = if let Some(ref mut s) = app.file_picker {
        s
    } else {
        return;
    };

    let block_w = 70u16.min(area.width);
    let block_h = 24u16.min(area.height);
    let x = area.x + (area.width - block_w) / 2;
    let y = area.y + (area.height - block_h) / 2;
    let block_area = Rect::new(x, y, block_w, block_h);

    f.render_widget(Clear, block_area);

    let title = match state.action {
        crate::app::FilePickerAction::Open => " [ Open File ] ",
        crate::app::FilePickerAction::Save => " [ Save As ] ",
        crate::app::FilePickerAction::ExportReport => " [ Export Report ] ",
        crate::app::FilePickerAction::ExportScript => " [ Export Script ] ",
        crate::app::FilePickerAction::ExportSprints => " [ Export Sprints ] ",
    };

    let mode_bg = Color::from(app.theme.ui.normal_mode_bg.clone());
    let block = Block::default()
        .title(Span::styled(
            title,
            Style::default()
                .fg(mode_bg)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(app.theme.secondary_style())
        .style(app.theme.normal_style());
 f.render_widget(block, block_area);

    let inner_area = block_area.inner(ratatui::layout::Margin {
        horizontal: 2,
        vertical: 1,
    });

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Current Dir
            Constraint::Min(0),    // List
            Constraint::Length(1), // Input Label
            Constraint::Length(1), // Filename Input
        ])
        .split(inner_area);

    // 1. Current Dir
    let dir_str = format!(" Dir: {}", state.current_dir.display());
    let dir_style = if state.naming_mode {
        app.theme.warning_style().add_modifier(Modifier::BOLD)
    } else {
        app.theme.secondary_style().add_modifier(Modifier::ITALIC)
    };
    
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(dir_str, dir_style),
            if state.naming_mode {
                Span::styled(" [LOCKED]", app.theme.error_style().add_modifier(Modifier::BOLD))
            } else {
                Span::raw("")
            }
        ])),
        layout[0],
    );

    // 2. List of items
    let items_len = state.items.len();
    let selected_idx = state.list_state.selected().unwrap_or(0);
    let mut display_items: Vec<ListItem> = state
        .items
        .iter()
        .enumerate()
        .map(|(i, path)| {
            let is_selected = i == selected_idx;
            let is_dir = path.is_dir();

            let name = if let Some(parent) = state.current_dir.parent() {
                if path == parent {
                    ".. (Parent Directory)".to_string()
                } else {
                    path.file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_else(|| "/".to_string())
                }
            } else {
                path.file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| "/".to_string())
            };

            let (icon, color) = if is_dir {
                ("[D] ", app.theme.ui.navigator_mode_bg.clone().into())
            } else {
                ("    ", app.theme.primary_fg())
            };

            let style = if is_selected {
                Style::default().bg(Color::from(app.theme.ui.selection_bg.clone())).fg(Color::from(app.theme.ui.selection_fg.clone()))
            } else {
                Style::default().fg(color)
            };

            ListItem::new(Line::from(vec![
                Span::styled(if is_selected { " > " } else { "   " }, style),
                Span::styled(icon, style),
                Span::styled(name, style),
            ]))
        })
        .collect();

    // Add virtual item for custom filename if in Save mode
    if state.action != crate::app::FilePickerAction::Open && !state.filename_input.is_empty() {
        let is_selected = selected_idx == items_len;
        let style = if is_selected {
            Style::default().bg(Color::from(app.theme.ui.selection_bg.clone())).fg(Color::from(app.theme.ui.selection_fg.clone()))
        } else {
            Style::default().fg(Color::from(app.theme.ui.normal_mode_bg.clone()))
        };
        display_items.push(ListItem::new(Line::from(vec![
            Span::styled(if is_selected { " > " } else { "   " }, style),
            Span::styled("[!] ", style),
            Span::styled(
                format!("Confirm: {}", state.filename_input),
                style.add_modifier(Modifier::BOLD),
            ),
        ])));
    }

    let list = List::new(display_items).highlight_style(Style::default());
    f.render_stateful_widget(list, layout[1], &mut state.list_state);

    // 3. Input Label & Hints
    if state.action != crate::app::FilePickerAction::Open {
        let hints = if state.naming_mode {
            Line::from(vec![
                Span::styled(" Filename: ", Style::default().fg(Color::from(app.theme.ui.normal_mode_bg.clone())).add_modifier(Modifier::BOLD)),
                Span::styled(" [Enter] ", app.theme.success_style().add_modifier(Modifier::BOLD)),
                Span::styled("to SAVE to locked folder", app.theme.secondary_style()),
            ])
        } else {
            Line::from(vec![
                Span::styled(" Filename: ", app.theme.secondary_style()),
                Span::styled(" [Tab] ", Style::default().fg(Color::from(app.theme.ui.normal_mode_bg.clone())).add_modifier(Modifier::BOLD)),
                Span::styled("to LOCK folder & type name", app.theme.secondary_style()),
            ])
        };
        f.render_widget(Paragraph::new(hints), layout[2]);

        // 4. Filename Input
        let input_style = Style::default().fg(Color::from(app.theme.ui.selection_fg.clone())).bg(Color::from(app.theme.ui.selection_bg.clone()));
        f.render_widget(
            Paragraph::new(Line::from(vec![Span::styled(
                format!("  {}", state.filename_input),
                input_style,
            )]))
            .block(Block::default().borders(Borders::NONE)),
            layout[3],
        );

        // Cursor for input
        let cursor_pos =
            layout[3].x + 2 + UnicodeWidthStr::width(state.filename_input.as_str()) as u16;
        f.set_cursor_position((cursor_pos, layout[3].y));
    }

    // Overwrite Confirmation Overlay
    if state.show_overwrite_confirm {
        let confirm_area = centered_rect(60, 30, area);
        f.render_widget(Clear, confirm_area);
        let confirm_block = Block::default()
            .title(Span::styled(" [ Confirm Overwrite ] ", app.theme.error_style().add_modifier(Modifier::BOLD)))
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Double)
            .border_style(app.theme.error_style())
            .style(app.theme.normal_style());
        
        let file_name = state.target_path.as_ref().and_then(|p| p.file_name()).map(|n| n.to_string_lossy()).unwrap_or_default();
        
        let confirm_text = vec![
            Line::from(""),
            Line::from(vec![
                Span::raw(" File "),
                Span::styled(file_name, app.theme.warning_style().add_modifier(Modifier::BOLD)),
                Span::raw(" already exists!"),
            ]),
            Line::from(""),
            Line::from(" Would you like to overwrite it?"),
            Line::from(""),
            Line::from(vec![
                if state.overwrite_confirmed {
                    Span::styled("  > YES  ", Style::default().bg(Color::from(app.theme.ui.success.clone())).fg(Color::from(app.theme.ui.selection_fg.clone())).add_modifier(Modifier::BOLD))
                } else {
                    Span::styled("    Yes  ", app.theme.success_style())
                },
                Span::raw("      "),
                if !state.overwrite_confirmed {
                    Span::styled("  > NO   ", Style::default().bg(Color::from(app.theme.ui.error.clone())).fg(Color::from(app.theme.ui.selection_fg.clone())).add_modifier(Modifier::BOLD))
                } else {
                    Span::styled("    No   ", app.theme.error_style())
                },
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(" [<-/->] ", app.theme.secondary_style()),
                Span::raw("Switch  "),
                Span::styled(" [Enter] ", app.theme.secondary_style()),
                Span::raw("Confirm"),
            ]),
        ];
        
        f.render_widget(Paragraph::new(confirm_text).block(confirm_block).alignment(ratatui::layout::Alignment::Center), confirm_area);
    }
}

pub fn draw_snapshots(f: &mut Frame, app: &mut App) {
    let area = f.area();
    let theme = &app.theme;
    let mode_bg = Color::from(theme.ui.normal_mode_bg.clone());

    let modal_area = centered_rect(70, 60, area);
    f.render_widget(Clear, modal_area);

    let block = Block::default()
        .title(" [ Snapshots | Enter: Replace | O: Open in New ] ")
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(theme.secondary_style())
        .style(theme.normal_style());

    let header = Row::new(vec![
        Cell::from("File Name"),
        Cell::from("Date"),
        Cell::from("Time"),
    ])
    .style(
        Style::default()
            .bg(mode_bg)
            .fg(theme.ui.selection_fg.clone().into())
            .add_modifier(Modifier::BOLD),
    );

    let rows: Vec<Row> = app
        .snapshots
        .iter()
        .map(|s| {
            Row::new(vec![
                Cell::from(s.display_stem()),
                Cell::from(s.display_date()),
                Cell::from(s.display_time_only()),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(50),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ],
    )
    .header(header)
    .block(block)
    .row_highlight_style(
        Style::default()
            .bg(Color::from(theme.ui.selection_bg.clone()))
            .fg(Color::from(theme.ui.selection_fg.clone()))
            .add_modifier(Modifier::BOLD),
    );

    f.render_stateful_widget(table, modal_area, &mut app.snapshot_list_state);
}

pub fn draw_export_modal(f: &mut Frame, app: &App) {
    let area = f.area();
    let theme = &app.theme;
    let mode_bg = Color::from(theme.ui.normal_mode_bg.clone());
    let _dim_color = Color::from(theme.ui.dim.clone());

    let modal_area = centered_rect(60, 60, area);
    f.render_widget(Clear, modal_area);

    let block = Block::default()
        .title(" [ Export ] ")
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
            Constraint::Length(3), // Tabs
            Constraint::Min(0),    // Options
            Constraint::Length(1), // Footer hint
        ])
        .split(inner_area);

    // 1. Tabs
    let tab_titles = vec![" 1. Screenplay ", " 2. Reports "];
    let tabs_spans: Vec<Span> = tab_titles
        .iter()
        .enumerate()
        .map(|(i, t)| {
            if i == app.export_tab {
                Span::styled(
                    t.to_string(),
                    Style::default()
                        .fg(Color::from(theme.ui.selection_fg.clone()))
                        .bg(Color::from(theme.ui.selection_bg.clone()))
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                Span::styled(t.to_string(), theme.secondary_style())
            }
        })
        .collect();

    let mut tab_line = Vec::new();
    for (i, span) in tabs_spans.into_iter().enumerate() {
        tab_line.push(span);
        if i < 1 {
            tab_line.push(Span::styled("  ", Style::default()));
        }
    }
    f.render_widget(Paragraph::new(Line::from(tab_line)).alignment(ratatui::layout::Alignment::Center).block(Block::default().borders(Borders::BOTTOM).border_style(theme.secondary_style())), layout[0]);

    // 2. Options
    let mut options = Vec::new();

    let render_option = |_idx: usize, label: &str, value: &str, is_selected: bool| -> ListItem {
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
            Span::styled(format!("{:<18}", label), style),
            Span::styled(value.to_string(), val_style),
        ]))
    };

    if app.export_tab == 0 {
        // Screenplay Tab
        let format_label = match app.config.export_format.as_str() {
            "pdf" => "PDF",
            "fountain" => "Fountain",
            _ => "PDF",
        };
        let font_label = match app.config.export_font.as_str() {
            "courier_prime" => "Courier Prime",
            "courier_prime_sans" => "Courier Prime Sans",
            _ => "Courier Prime",
        };

        options.push(render_option(0, "Format", format_label, app.selected_export_option == 0));
        options.push(render_option(1, "Paper Size", &app.config.paper_size.to_uppercase(), app.selected_export_option == 1));
        options.push(render_option(2, "Font", font_label, app.selected_export_option == 2));
        options.push(render_option(3, "Bold Headings", if app.config.export_bold_scene_headings { "[X]" } else { "[ ]" }, app.selected_export_option == 3));
        options.push(render_option(4, "Scene Numbers", if app.config.mirror_scene_numbers != crate::config::MirrorOption::Off { "[X]" } else { "[ ]" }, app.selected_export_option == 4));
        options.push(render_option(5, "Include Sections", if app.config.export_sections { "[X]" } else { "[ ]" }, app.selected_export_option == 5));
        options.push(render_option(6, "Include Synopses", if app.config.export_synopses { "[X]" } else { "[ ]" }, app.selected_export_option == 6));
        options.push(render_option(7, "Title Page", if app.config.include_title_page { "[X]" } else { "[ ]" }, app.selected_export_option == 7));
        
        options.push(ListItem::new(Line::raw("")));
        let export_style = if app.selected_export_option == 8 {
            Style::default().bg(theme.ui.success.clone().into()).fg(theme.ui.selection_fg.clone().into()).add_modifier(Modifier::BOLD)
        } else {
            theme.success_style()
        };
        options.push(ListItem::new(Line::from(vec![
            Span::styled(if app.selected_export_option == 8 { " > " } else { "   " }, export_style),
            Span::styled(" [ EXPORT SCREENPLAY ] ", export_style),
        ])));
    } else {
        // Reports Tab
        let report_label = match app.config.report_format.as_str() {
            "csv_scene" => "Scene List (CSV)",
            "csv_char" => "Character Report (CSV)",
            "csv_location" => "Location Report (CSV)",
            "csv_notes" => "Notes & Markers (CSV)",
            "csv_breakdown" => "Script Breakdown (CSV)",
            "txt_dialogue" => "Dialogue Only (TXT)",
            _ => "Scene List (CSV)",
        };

        options.push(render_option(0, "Report Type", report_label, app.selected_export_option == 0));
        
        options.push(ListItem::new(Line::raw("")));
        let export_style = if app.selected_export_option == 1 {
            Style::default().bg(theme.ui.success.clone().into()).fg(theme.ui.selection_fg.clone().into()).add_modifier(Modifier::BOLD)
        } else {
            theme.success_style()
        };
        options.push(ListItem::new(Line::from(vec![
            Span::styled(if app.selected_export_option == 1 { " > " } else { "   " }, export_style),
            Span::styled(" [ EXPORT REPORT ] ", export_style),
        ])));
    }

    f.render_widget(List::new(options), layout[1]);

    // 3. Footer
    let footer_text = Line::from(vec![
        Span::styled(" [<-/->] ", Style::default().fg(mode_bg).add_modifier(Modifier::BOLD)),
        Span::styled("Switch Tabs  ", theme.secondary_style()),
        Span::styled(" [^/v] ", Style::default().fg(mode_bg).add_modifier(Modifier::BOLD)),
        Span::styled("Select  ", theme.secondary_style()),
        Span::styled(" [Tab/Enter] ", Style::default().fg(mode_bg).add_modifier(Modifier::BOLD)),
        Span::styled("Toggle/Export", theme.secondary_style()),
    ]);
    f.render_widget(Paragraph::new(footer_text).alignment(ratatui::layout::Alignment::Center), layout[2]);
}
