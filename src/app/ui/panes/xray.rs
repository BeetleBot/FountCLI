use crate::app::App;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, BorderType, Clear, Paragraph, List, ListItem, Sparkline, Table, Row, Cell},
};

pub fn draw_xray(f: &mut Frame, app: &mut App) {
    let area = f.area();
    let theme = app.theme.clone();

    // Dim the background
    let buf = f.buffer_mut();
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            if let Some(cell) = buf.cell_mut((x, y)) {
                let st = cell.style();
                if !theme.is_light() {
                    cell.set_style(st.add_modifier(Modifier::DIM));
                }
            }
        }
    }

    let accent = Color::from(theme.ui.tree_mode_bg.clone());
    let dim = Color::from(theme.ui.dim.clone());
    let normal_fg = theme.primary_fg();
    let normal_bg = theme.primary_bg();

    let modal_w = 100u16.min(area.width.saturating_sub(4));
    let modal_h = 36u16.min(area.height.saturating_sub(2));
    let x = area.x + (area.width.saturating_sub(modal_w)) / 2;
    let y = area.y + (area.height.saturating_sub(modal_h)) / 2;
    let modal_area = Rect::new(x, y, modal_w, modal_h);

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
            Style::default().fg(theme.ui.selection_fg.clone().into()).bg(accent).add_modifier(Modifier::BOLD)
        } else {
            theme.secondary_style()
        }),
        Span::styled(" 2: Pacing ", if app.xray_tab == 1 {
            Style::default().fg(theme.ui.selection_fg.clone().into()).bg(accent).add_modifier(Modifier::BOLD)
        } else {
            theme.secondary_style()
        }),
        Span::styled(" 3: Scenes ", if app.xray_tab == 2 {
            Style::default().fg(theme.ui.selection_fg.clone().into()).bg(accent).add_modifier(Modifier::BOLD)
        } else {
            theme.secondary_style()
        }),
        Span::styled(" 4: Breakdown ", if app.xray_tab == 3 {
            Style::default().fg(theme.ui.selection_fg.clone().into()).bg(accent).add_modifier(Modifier::BOLD)
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
            dim,
            theme: &theme,
            use_nerd_fonts: app.config.use_nerd_fonts,
        };

        match app.xray_tab {
            0 => draw_dialogue_tab(f, content_area, data, app.xray_scroll, &ctx),
            1 => draw_pacing_tab(f, content_area, data, app.xray_scroll, &ctx),
            2 => draw_scenes_tab(f, content_area, data, app.xray_scroll, &ctx),
            3 => draw_breakdown_tab(f, content_area, app),
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
            Span::styled("  ^/v ", Style::default().fg(accent).add_modifier(Modifier::BOLD)),
            Span::styled("Scroll", theme.secondary_style()),
            Span::styled("  Esc ", Style::default().fg(accent).add_modifier(Modifier::BOLD)),
            Span::styled("Close", theme.secondary_style()),
        ])),
        tab_layout[3],
    );
}

struct XrayRenderContext<'a> {
    pub accent: Color,
    pub dim: Color,
    pub theme: &'a crate::theme::Theme,
    pub use_nerd_fonts: bool,
}

fn draw_dialogue_tab(
    f: &mut Frame,
    area: Rect,
    data: &crate::app::XRayData,
    _scroll: usize, // Table handles its own layout usually, but we'll use area constraints
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
        Cell::from("Frequency").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Lines").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Words").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Balance").style(Style::default().add_modifier(Modifier::BOLD)),
    ])
    .style(Style::default().fg(accent))
    .bottom_margin(1);

    let mut rows = Vec::new();
    let bar_w: usize = 20;

    for ch in &data.characters {
        let filled = ((ch.percentage / 100.0) * bar_w as f32).round() as usize;
        let empty = bar_w.saturating_sub(filled);
        let bar = format!("{}{}", "█".repeat(filled), " ".repeat(empty));

        rows.push(Row::new(vec![
            Cell::from(ch.name.clone()).style(Style::default().add_modifier(Modifier::BOLD)),
            Cell::from(format!("{:.1}%", ch.percentage)),
            Cell::from(ch.dialogue_lines.to_string()),
            Cell::from(ch.word_count.to_string()),
            Cell::from(bar).style(Style::default().fg(accent)),
        ])
        .style(theme.secondary_style())
        .bottom_margin(0));
    }

    let table = Table::new(rows, [
        Constraint::Percentage(25),
        Constraint::Percentage(15),
        Constraint::Percentage(15),
        Constraint::Percentage(15),
        Constraint::Min(25),
    ])
    .header(header)
    .block(Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(dim))
        .border_type(BorderType::Plain))
    .column_spacing(1);

    f.render_widget(table, chunks[1]);
}

fn draw_pacing_tab(
    f: &mut Frame,
    area: Rect,
    data: &crate::app::XRayData,
    _scroll: usize,
    ctx: &XrayRenderContext,
) {
    let accent = ctx.accent;
    let theme = ctx.theme;
    let dim = ctx.dim;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Title
            Constraint::Length(8), // Action Sparkline
            Constraint::Length(8), // Dialogue Sparkline
            Constraint::Min(0),   // Legend/Details
        ])
        .split(area);

    f.render_widget(
        Paragraph::new(vec![
            Line::from(Span::styled("Pacing Heatmap", Style::default().fg(accent).add_modifier(Modifier::BOLD))),
            Line::from(Span::styled("Action Energy vs. Dialogue Frequency across pages", theme.secondary_style())),
        ]),
        chunks[0],
    );

    if data.pacing_map.is_empty() {
        f.render_widget(
            Paragraph::new("\n  No page data available.").alignment(Alignment::Center),
            chunks[1],
        );
        return;
    }

    let action_data: Vec<u64> = data.pacing_map.iter().map(|b| b.action_lines as u64).collect();
    let dialogue_data: Vec<u64> = data.pacing_map.iter().map(|b| b.dialogue_lines as u64).collect();

    let action_spark = Sparkline::default()
        .block(Block::default()
            .title(Span::styled(" Action Energy ", Style::default().fg(accent)))
            .borders(Borders::LEFT | Borders::BOTTOM)
            .border_style(Style::default().fg(dim)))
        .data(&action_data)
        .style(Style::default().fg(accent));

    let dialogue_spark = Sparkline::default()
        .block(Block::default()
            .title(Span::styled(" Dialogue Frequency ", Style::default().fg(Color::from(theme.ui.search_highlight_bg.clone()))))
            .borders(Borders::LEFT | Borders::BOTTOM)
            .border_style(Style::default().fg(dim)))
        .data(&dialogue_data)
        .style(Style::default().fg(Color::from(theme.ui.search_highlight_bg.clone())));

    f.render_widget(action_spark, chunks[1]);
    f.render_widget(dialogue_spark, chunks[2]);

    // Summary box
    let avg_action = if !action_data.is_empty() { action_data.iter().sum::<u64>() / action_data.len() as u64 } else { 0 };
    let avg_dialogue = if !dialogue_data.is_empty() { dialogue_data.iter().sum::<u64>() / dialogue_data.len() as u64 } else { 0 };

    let summary = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Analysis: ", theme.secondary_style().add_modifier(Modifier::BOLD)),
            Span::styled(format!("Avg {} action lines/page, {} dialogue lines/page.", avg_action, avg_dialogue), theme.secondary_style()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Tip: ", theme.secondary_style().add_modifier(Modifier::BOLD)),
            Span::styled("Higher action energy indicates faster pacing. Dense dialogue slows it down.", theme.secondary_style().add_modifier(Modifier::ITALIC)),
        ]),
    ]).block(Block::default().borders(Borders::TOP).border_style(Style::default().fg(dim)));

    f.render_widget(summary, chunks[3]);
}

fn draw_scenes_tab(
    f: &mut Frame,
    area: Rect,
    data: &crate::app::XRayData,
    _scroll: usize,
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
        .style(theme.secondary_style())
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
    .column_spacing(2);

    f.render_widget(table, chunks[1]);
}

fn draw_breakdown_tab(
    f: &mut Frame,
    area: Rect,
    app: &mut App,
) {
    let data = match &app.xray_data {
        Some(d) => d,
        None => return,
    };
    
    let accent = Color::from(app.theme.ui.tree_mode_bg.clone());
    let dim = Color::from(app.theme.ui.dim.clone());
    let theme = &app.theme;

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
            Constraint::Percentage(30),
            Constraint::Percentage(70),
        ])
        .split(area);

    // Left: Scene List
    let mut scene_items = Vec::new();
    for (i, s) in data.scene_breakdown.iter().enumerate() {
        let prefix = s.scene_num.as_deref().unwrap_or("-");
        let label = format!(" {:>3} {}", prefix, s.label);
        let style = if i == app.xray_breakdown_idx {
            Style::default().fg(theme.ui.selection_fg.clone().into()).bg(accent).add_modifier(Modifier::BOLD)
        } else {
            theme.secondary_style()
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
            Span::styled("  Tags for: ", theme.secondary_style()),
            Span::styled(&selected_scene.label, Style::default().fg(accent).add_modifier(Modifier::BOLD)),
        ]));
        tag_lines.push(Line::from(Span::styled(format!("  {}", "─".repeat(chunks[1].width.saturating_sub(4) as usize)), theme.secondary_style())));
        tag_lines.push(Line::from(""));

        if selected_scene.breakdown.is_empty() {
            tag_lines.push(Line::from(Span::styled(
                "    No production tags identified in this scene.",
                theme.secondary_style().add_modifier(Modifier::ITALIC),
            )));
        } else {
            for (key, values) in &selected_scene.breakdown {
                tag_lines.push(Line::from(vec![
                    Span::styled(format!("    {} ", key.to_uppercase()), Style::default().fg(accent).add_modifier(Modifier::BOLD)),
                ]));
                for v in values {
                    tag_lines.push(Line::from(vec![
                        Span::styled("      → ", theme.secondary_style()),
                        Span::styled(v, theme.secondary_style()),
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
