use crate::app::App;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, BorderType, Clear, Paragraph, List, ListItem, Table, Row, Cell, TableState,
    },
};

pub fn draw_xray(f: &mut Frame, app: &mut App) {
    let area = f.area();
    let theme = app.theme.clone();

    let accent = Color::from(theme.ui.tree_mode_bg.clone());
    let selection_bg = Color::from(theme.ui.selection_bg.clone());
    let selection_fg = Color::from(theme.ui.selection_fg.clone());
    let dim = Color::from(theme.ui.dim.clone());
    let normal_fg = theme.primary_fg();
    let normal_bg = theme.primary_bg();

    let modal_w = 110u16.min(area.width.saturating_sub(4));
    let modal_h = (area.height * 90 / 100).max(36).min(area.height.saturating_sub(2));
    let x = area.x + (area.width.saturating_sub(modal_w)) / 2;
    let y = area.y + (area.height.saturating_sub(modal_h)) / 2;
    let modal_area = Rect::new(x, y, modal_w, modal_h);

    // Dim the background ONLY outside the modal area
    let buf = f.buffer_mut();
    for row in area.top()..area.bottom() {
        for col in area.left()..area.right() {
            if !modal_area.contains(ratatui::layout::Position { x: col, y: row })
                && let Some(cell) = buf.cell_mut((col, row))
            {
                let st = cell.style();
                if !theme.is_light() {
                    cell.set_style(st.add_modifier(Modifier::DIM));
                }
            }
        }
    }

    f.render_widget(Clear, modal_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(dim))
        .style(Style::default().bg(normal_bg).fg(normal_fg))
        .title(Span::styled(
            " [ X-Ray Analysis ] ",
            Style::default().fg(accent).add_modifier(Modifier::BOLD),
        ));

    f.render_widget(block, modal_area);

    let inner = modal_area.inner(ratatui::layout::Margin {
        horizontal: 2,
        vertical: 1,
    });

    // Tab bar
    let tab_titles = vec![
        Span::styled(" 1: Dialogue ", if app.xray_tab == 0 {
            Style::default().fg(selection_fg).bg(selection_bg).add_modifier(Modifier::BOLD)
        } else {
            theme.secondary_style()
        }),
        Span::styled(" 2: Pacing ", if app.xray_tab == 1 {
            Style::default().fg(selection_fg).bg(selection_bg).add_modifier(Modifier::BOLD)
        } else {
            theme.secondary_style()
        }),
        Span::styled(" 3: Scenes ", if app.xray_tab == 2 {
            Style::default().fg(selection_fg).bg(selection_bg).add_modifier(Modifier::BOLD)
        } else {
            theme.secondary_style()
        }),
        Span::styled(" 4: Breakdown ", if app.xray_tab == 3 {
            Style::default().fg(selection_fg).bg(selection_bg).add_modifier(Modifier::BOLD)
        } else {
            theme.secondary_style()
        }),
    ];

    let tab_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // tabs
            Constraint::Length(1), // separator
            Constraint::Min(0),   // content
            Constraint::Length(1), // footer hint
        ])
        .split(inner);

    // Render tab bar
    let tab_line = Line::from(tab_titles);
    f.render_widget(Paragraph::new(tab_line), tab_layout[0]);

    // Separator
    let sep_w = tab_layout[1].width as usize;
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "─".repeat(sep_w),
            theme.secondary_style(),
        ))),
        tab_layout[1],
    );

    let content_area = tab_layout[2];

    if let Some(ref data) = app.xray_data {
        let ctx = XrayRenderContext {
            accent,
            selection_bg,
            selection_fg,
            dim,
            theme: &theme,
            use_nerd_fonts: app.config.use_nerd_fonts,
        };

        match app.xray_tab {
            0 => draw_dialogue_tab(f, content_area, data, &mut app.xray_dialogue_state, &ctx),
            1 => draw_pacing_tab(f, content_area, data, app.xray_scroll, &ctx),
            2 => draw_scenes_tab(f, content_area, data, &mut app.xray_scene_state, &ctx),
            3 => draw_breakdown_tab(f, content_area, app, &ctx),
            _ => {}
        }
    } else {
        f.render_widget(
            Paragraph::new("No data. Run /xray on a script.").alignment(Alignment::Center),
            content_area,
        );
    }

    // Footer
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" <-/-> ", Style::default().fg(accent).add_modifier(Modifier::BOLD)),
            Span::styled("Switch Tab", theme.secondary_style()),
            Span::styled("  ↑/↓ ", Style::default().fg(accent).add_modifier(Modifier::BOLD)),
            Span::styled("Scroll", theme.secondary_style()),
            Span::styled("  Esc ", Style::default().fg(accent).add_modifier(Modifier::BOLD)),
            Span::styled("Close", theme.secondary_style()),
        ])),
        tab_layout[3],
    );
}

struct XrayRenderContext<'a> {
    pub accent: Color,
    pub selection_bg: Color,
    pub selection_fg: Color,
    pub dim: Color,
    pub theme: &'a crate::theme::Theme,
    pub use_nerd_fonts: bool,
}

fn draw_dialogue_tab(
    f: &mut Frame,
    area: Rect,
    data: &crate::app::XRayData,
    state: &mut TableState,
    ctx: &XrayRenderContext,
) {
    let accent = ctx.accent;
    let theme = ctx.theme;
    let dim = ctx.dim;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Title
            Constraint::Min(0),   // Table
        ])
        .split(area);

    f.render_widget(
        Paragraph::new(vec![
            Line::from(Span::styled("Dialogue Balance", Style::default().fg(accent).add_modifier(Modifier::BOLD))),
            Line::from(Span::styled(format!("Total dialogue words: {}", data.total_dialogue_words), theme.secondary_style())),
        ]),
        chunks[0],
    );

    if data.characters.is_empty() {
        f.render_widget(
            Paragraph::new("\n  No dialogue found in script.").alignment(Alignment::Center),
            chunks[1],
        );
        return;
    }

    let header = Row::new(vec![
        Cell::from("Character").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("% of Script").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Total Lines").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Word Count").style(Style::default().add_modifier(Modifier::BOLD)),
    ])
    .style(Style::default().fg(accent))
    .bottom_margin(1);

    let mut rows = Vec::new();

    for ch in &data.characters {
        rows.push(Row::new(vec![
            Cell::from(ch.name.clone()).style(Style::default().add_modifier(Modifier::BOLD)),
            Cell::from(format!("{:.1}%", ch.percentage)),
            Cell::from(ch.dialogue_lines.to_string()),
            Cell::from(ch.word_count.to_string()),
        ])
        .style(Style::default().fg(ctx.theme.primary_fg()))
        .bottom_margin(1));
    }

    let table = Table::new(rows, [
        Constraint::Percentage(40),
        Constraint::Percentage(20),
        Constraint::Percentage(20),
        Constraint::Percentage(20),
    ])
    .header(header)
    .block(Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(dim))
        .border_type(BorderType::Plain))
    .column_spacing(1)
    .row_highlight_style(Style::default().bg(ctx.selection_bg).fg(ctx.selection_fg).add_modifier(Modifier::BOLD))
    .highlight_symbol("> ");

    f.render_stateful_widget(table, chunks[1], state);
}

fn draw_pacing_tab(
    f: &mut Frame,
    area: Rect,
    data: &crate::app::XRayData,
    scroll: usize,
    ctx: &XrayRenderContext,
) {
    let accent = ctx.accent;
    let theme = ctx.theme;
    let dim = ctx.dim;

    if data.scenes.is_empty() || data.pacing_map.is_empty() {
        f.render_widget(
            Paragraph::new("\n  No scene pacing data available.").alignment(Alignment::Center),
            area,
        );
        return;
    }

    let main_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(0),      // Pulse Axis
            Constraint::Length(28),  // Sidebar
        ])
        .split(area);

    let pulse_area = main_layout[0];
    let sidebar_area = main_layout[1];

    // --- 1. Aggregating Pacing per Scene ---
    let mut scene_pacing = Vec::new();
    let mut current_page_f64: f64 = 0.0;
    
    for scene in &data.scenes {
        let start_page = current_page_f64.floor() as usize;
        let duration = scene.page_count as f64;
        let end_page = (current_page_f64 + duration).ceil() as usize;
        
        // Aggregate action/dialogue from pacing_map
        let range = start_page..end_page.min(data.pacing_map.len());
        let (avg_action, avg_dialogue) = if range.is_empty() {
            (0.0, 0.0)
        } else {
            let sum_action: usize = data.pacing_map[range.clone()].iter().map(|b| b.action_lines).sum();
            let sum_dialogue: usize = data.pacing_map[range.clone()].iter().map(|b| b.dialogue_lines).sum();
            let count = range.len() as f32;
            (sum_action as f32 / count, sum_dialogue as f32 / count)
        };

        scene_pacing.push((
            scene.scene_num.as_deref().unwrap_or("-").to_string(),
            (current_page_f64 + 1.0).floor() as usize,
            scene.label.clone(),
            avg_action,
            avg_dialogue,
        ));
        
        current_page_f64 += duration;
    }

    // --- 2. Render Pulse Axis ---
    let max_action_global = scene_pacing.iter().map(|p| p.3).fold(0.0, f32::max).max(1.0);
    let peak_scene_val = max_action_global;

    let action_color = Color::from(theme.ui.info.clone());
    let dialogue_color = Color::from(theme.ui.search_highlight_bg.clone());
    let bar_max_width = (pulse_area.width.saturating_sub(15) / 3) as usize;

    let rows: Vec<Row> = scene_pacing.iter().skip(scroll).map(|(num, page, heading, action, dialogue)| {
        let action_ratio = (action / 25.0).min(1.0);
        let dialogue_ratio = (dialogue / 25.0).min(1.0);
        
        let action_len = (action_ratio * bar_max_width as f32) as usize;
        let dialogue_len = (dialogue_ratio * bar_max_width as f32) as usize;

        // Gradient logic for action (growing left)
        let mut action_str = String::new();
        if action_len > 0 {
            action_str.push('░');
            if action_len > 1 { action_str.push('▒'); }
            if action_len > 2 { action_str.push('▓'); }
            if action_len > 3 {
                action_str.push_str(&"█".repeat(action_len - 3));
            }
        }
        let action_display = format!("{:>width$}", action_str, width = bar_max_width);

        // Gradient logic for dialogue (growing right)
        let mut dialogue_str = String::new();
        if dialogue_len > 0 {
            dialogue_str.push_str(&"█".repeat(dialogue_len.saturating_sub(3)));
            if dialogue_len > 2 { dialogue_str.push('▓'); }
            if dialogue_len > 1 { dialogue_str.push('▒'); }
            dialogue_str.push('░');
        }
        let dialogue_display = format!("{:<width$}", dialogue_str, width = bar_max_width);

        let is_peak = *action == peak_scene_val;
        let spine_style = if is_peak { Style::default().fg(theme.warning_style().fg.unwrap()).add_modifier(Modifier::BOLD) } else { Style::default().fg(dim) };
        let spine_label = if is_peak { format!("󱐋 {:>2}/{} ", num, page) } else { format!("  {:>2}/{} ", num, page) };

        Row::new(vec![
            Cell::from(action_display).style(Style::default().fg(action_color)),
            Cell::from(spine_label).style(spine_style),
            Cell::from(dialogue_display).style(Style::default().fg(dialogue_color)),
            Cell::from(heading.clone()).style(theme.secondary_style()),
        ])
        .bottom_margin(1)
    }).collect();

    let table = Table::new(rows, [
        Constraint::Length(bar_max_width as u16),
        Constraint::Length(12),
        Constraint::Length(bar_max_width as u16),
        Constraint::Min(0),
    ])
    .header(Row::new(vec![
        Cell::from(Line::from("ACTION").alignment(Alignment::Right)),
        Cell::from(Line::from("SCENE/PG").alignment(Alignment::Center)),
        Cell::from(Line::from("DIALOGUE").alignment(Alignment::Left)),
        Cell::from("CONTEXT"),
    ]).style(Style::default().fg(accent).add_modifier(Modifier::BOLD)).bottom_margin(1))
    .block(Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(dim))
        .title(Span::styled(" [ Pulse Axis Analysis ] ", Style::default().fg(accent).add_modifier(Modifier::BOLD))))
    .column_spacing(1);

    f.render_widget(table, pulse_area);

    // --- 3. Sidebar Diagnostic ---
    let side_block = Block::default()
        .borders(Borders::LEFT)
        .border_style(Style::default().fg(dim))
        .padding(ratatui::widgets::Padding::new(1, 1, 1, 0));
    
    let side_inner = side_block.inner(sidebar_area);
    f.render_widget(side_block, sidebar_area);

    let total_action: f32 = scene_pacing.iter().map(|p| p.3).sum();
    let total_dialogue: f32 = scene_pacing.iter().map(|p| p.4).sum();
    let avg_action = total_action / scene_pacing.len() as f32;
    let avg_dialogue = total_dialogue / scene_pacing.len() as f32;
    
    let peak_scene_num = scene_pacing.iter().find(|p| p.3 == peak_scene_val).map(|p| p.0.clone()).unwrap_or_else(|| "-".to_string());

    let mut side_lines = Vec::new();
    side_lines.push(Line::from(Span::styled("DIAGNOSTICS", Style::default().fg(accent).add_modifier(Modifier::BOLD))));
    side_lines.push(Line::from(vec![
        Span::styled("Energy Avg:  ", theme.secondary_style()),
        Span::styled(format!("{:.1}", avg_action), Style::default().fg(action_color).add_modifier(Modifier::BOLD)),
    ]));
    side_lines.push(Line::from(vec![
        Span::styled("Verbal Avg:  ", theme.secondary_style()),
        Span::styled(format!("{:.1}", avg_dialogue), Style::default().fg(dialogue_color).add_modifier(Modifier::BOLD)),
    ]));
    side_lines.push(Line::from(""));
    side_lines.push(Line::from(Span::styled("PEAK RHYTHM", Style::default().fg(accent).add_modifier(Modifier::BOLD))));
    side_lines.push(Line::from(vec![
        Span::styled("Climax:      ", theme.secondary_style()),
        Span::styled(format!("Scene #{}", peak_scene_num), Style::default().fg(theme.warning_style().fg.unwrap()).add_modifier(Modifier::BOLD)),
    ]));
    side_lines.push(Line::from(""));
    side_lines.push(Line::from(""));
    side_lines.push(Line::from(Span::styled("LEGEND", Style::default().fg(accent).add_modifier(Modifier::BOLD))));
    side_lines.push(Line::from(vec![
        Span::styled(" █ ", Style::default().fg(action_color)),
        Span::styled("Action", theme.secondary_style()),
    ]));
    side_lines.push(Line::from(vec![
        Span::styled(" █ ", Style::default().fg(dialogue_color)),
        Span::styled("Dialogue", theme.secondary_style()),
    ]));
    side_lines.push(Line::from(vec![
        Span::styled(" 󱐋 ", Style::default().fg(theme.warning_style().fg.unwrap())),
        Span::styled("Climax Peak", theme.secondary_style()),
    ]));

    f.render_widget(
        Paragraph::new(side_lines).wrap(ratatui::widgets::Wrap { trim: true }),
        side_inner,
    );
}

fn draw_scenes_tab(
    f: &mut Frame,
    area: Rect,
    data: &crate::app::XRayData,
    state: &mut TableState,
    ctx: &XrayRenderContext,
) {
    let accent = ctx.accent;
    let dim = ctx.dim;
    let theme = ctx.theme;
    let use_nerd_fonts = ctx.use_nerd_fonts;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Title
            Constraint::Min(0),   // Table
        ])
        .split(area);

    let over_count = data.scenes.iter().filter(|s| s.is_over_limit).count();
    f.render_widget(
        Paragraph::new(vec![
            Line::from(Span::styled("Scene Length Analysis", Style::default().fg(accent).add_modifier(Modifier::BOLD))),
            Line::from(vec![
                Span::styled(format!("Total: {}  ·  ", data.scenes.len()), theme.secondary_style()),
                if over_count > 0 {
                    Span::styled(format!("{} scene(s) exceed 3 pages", over_count), theme.warning_style().add_modifier(Modifier::BOLD))
                } else {
                    Span::styled("All scenes within optimal length", theme.success_style())
                },
            ]),
        ]),
        chunks[0],
    );

    if data.scenes.is_empty() {
        f.render_widget(
            Paragraph::new("\n  No scenes found in script.").alignment(Alignment::Center),
            chunks[1],
        );
        return;
    }

    let header = Row::new(vec![
        Cell::from("№").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Scene Description").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Pages").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Status").style(Style::default().add_modifier(Modifier::BOLD)),
    ])
    .style(Style::default().fg(accent))
    .bottom_margin(1);

    let mut rows = Vec::new();
    for scene in &data.scenes {
        let (status, status_style) = if scene.is_over_limit {
            (if use_nerd_fonts { " TOO LONG" } else { "(!) LONG" }, theme.warning_style())
        } else {
            (if use_nerd_fonts { "  OK" } else { "(X) OK" }, theme.success_style())
        };

        rows.push(Row::new(vec![
            Cell::from(scene.scene_num.as_deref().unwrap_or("-").to_string()),
            Cell::from(scene.label.clone()),
            Cell::from(format!("{:.1}", scene.page_count)),
            Cell::from(status).style(status_style),
        ])
        .style(Style::default().fg(theme.primary_fg()))
        .bottom_margin(0));
    }

    let table = Table::new(rows, [
        Constraint::Length(6),
        Constraint::Percentage(60),
        Constraint::Length(10),
        Constraint::Min(15),
    ])
    .header(header)
    .block(Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(dim)))
    .column_spacing(2)
    .row_highlight_style(Style::default().bg(ctx.selection_bg).fg(ctx.selection_fg).add_modifier(Modifier::BOLD))
    .highlight_symbol("> ");

    f.render_stateful_widget(table, chunks[1], state);
}

fn draw_breakdown_tab(
    f: &mut Frame,
    area: Rect,
    app: &mut App,
    ctx: &XrayRenderContext,
) {
    let data = match &app.xray_data {
        Some(d) => d,
        None => return,
    };
    
    let accent = ctx.accent;
    let dim = ctx.dim;
    let theme = ctx.theme;

    if data.scene_breakdown.is_empty() {
        f.render_widget(
            Paragraph::new("  No production tags found in script.").alignment(Alignment::Center),
            area,
        );
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(area);

    // Left: Scene List
    let mut scene_items = Vec::new();
    for (i, s) in data.scene_breakdown.iter().enumerate() {
        let prefix = s.scene_num.as_deref().unwrap_or("-");
        let label = if i == app.xray_breakdown_idx {
            format!(" > {:>3} {}", prefix, s.label)
        } else {
            format!("   {:>3} {}", prefix, s.label)
        };
        
        let style = if i == app.xray_breakdown_idx {
            Style::default().fg(ctx.selection_fg).bg(ctx.selection_bg).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.primary_fg())
        };
        
        // Add breathing space by using multiple lines for the ListItem
        scene_items.push(ListItem::new(vec![
            Line::from(""), // Spacer above
            Line::from(Span::styled(label, style)),
        ]));
    }

    let scene_list = List::new(scene_items)
        .block(Block::default()
            .borders(Borders::RIGHT)
            .border_style(Style::default().fg(dim))
            .title(Span::styled(" [ Scenes ] ", Style::default().fg(accent).add_modifier(Modifier::BOLD))));

    f.render_stateful_widget(scene_list, chunks[0], &mut app.xray_breakdown_state);

    // Right: Tag Breakdown
    let mut tag_lines = Vec::new();
    if let Some(selected_scene) = data.scene_breakdown.get(app.xray_breakdown_idx) {
        tag_lines.push(Line::from(""));
        tag_lines.push(Line::from(vec![
            Span::styled("  Tags for: ", Style::default().fg(theme.primary_fg())),
            Span::styled(&selected_scene.label, Style::default().fg(accent).add_modifier(Modifier::BOLD)),
        ]));
        tag_lines.push(Line::from(Span::styled(format!("  {}", "─".repeat(chunks[1].width.saturating_sub(4) as usize)), theme.secondary_style())));
        tag_lines.push(Line::from(""));

        if selected_scene.breakdown.is_empty() {
            tag_lines.push(Line::from(Span::styled(
                "    No production tags identified in this scene.",
                Style::default().fg(theme.primary_fg()).add_modifier(Modifier::ITALIC),
            )));
        } else {
            for (key, values) in &selected_scene.breakdown {
                tag_lines.push(Line::from(vec![
                    Span::styled(format!("    {} ", key.to_uppercase()), Style::default().fg(accent).add_modifier(Modifier::BOLD)),
                ]));
                for v in values {
                    tag_lines.push(Line::from(vec![
                        Span::styled("      → ", Style::default().fg(theme.primary_fg())),
                        Span::styled(v, Style::default().fg(theme.primary_fg())),
                    ]));
                }
                tag_lines.push(Line::from(""));
            }
        }
    }

    f.render_widget(
        Paragraph::new(tag_lines)
            .block(Block::default()
                .borders(Borders::LEFT)
                .border_style(Style::default().fg(dim))
                .title(Span::styled(" [ Details ] ", Style::default().fg(accent).add_modifier(Modifier::BOLD)))),
        chunks[1],
    );
}
