use crate::app::App;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, BorderType, Clear, Paragraph, List, ListItem},
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

    let accent = Color::from(theme.ui.navigator_mode_bg.clone());
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
        match app.xray_tab {
            0 => draw_dialogue_tab(f, content_area, data, app.xray_scroll, accent, dim, normal_fg, &theme),
            1 => draw_pacing_tab(f, content_area, data, app.xray_scroll, accent, dim, normal_fg, &theme),
            2 => draw_scenes_tab(f, content_area, data, app.xray_scroll, accent, dim, normal_fg, &theme, app.config.use_nerd_fonts),
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

fn draw_dialogue_tab(
    f: &mut Frame,
    area: Rect,
    data: &crate::app::XRayData,
    scroll: usize,
    accent: Color,
    _dim: Color,
    normal_fg: Color,
    theme: &crate::theme::Theme,
) {
    let mut lines = Vec::new();

    lines.push(Line::from(Span::styled(
        "Dialogue Balance",
        Style::default().fg(accent).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    if data.characters.is_empty() {
        lines.push(Line::from(Span::styled(
            "  No dialogue found in script.",
            theme.secondary_style().add_modifier(Modifier::ITALIC),
        )));
    } else {
        lines.push(Line::from(vec![
            Span::styled(
                format!("  Total dialogue words: {}", data.total_dialogue_words),
                theme.secondary_style(),
            ),
        ]));
        lines.push(Line::from(""));

        let bar_max_w = area.width.saturating_sub(45) as usize;
        let max_name_len = data.characters.iter().map(|c| c.name.len()).max().unwrap_or(8).min(18);

        for ch in &data.characters {
            let name = if ch.name.len() > max_name_len {
                format!("{:.width$}", ch.name, width = max_name_len)
            } else {
                format!("{:width$}", ch.name, width = max_name_len)
            };

            let filled = ((ch.percentage / 100.0) * bar_max_w as f32).round() as usize;
            let empty = bar_max_w.saturating_sub(filled);
            let bar = format!("{}{}", "█".repeat(filled), "░".repeat(empty));

            let pct_str = format!("{:5.1}%", ch.percentage);
            let line_str = format!("{:>4}L", ch.dialogue_lines);
            let word_str = format!("{:>5}w", ch.word_count);

            lines.push(Line::from(vec![
                Span::styled(format!("  {} ", name), Style::default().fg(normal_fg).add_modifier(Modifier::BOLD)),
                Span::styled(bar, Style::default().fg(accent)),
                Span::styled(format!(" {} {} {}", pct_str, line_str, word_str), theme.secondary_style()),
            ]));
        }
    }

    let content_h = area.height as usize;
    let max_scroll = lines.len().saturating_sub(content_h);
    let scroll = scroll.min(max_scroll);
    let visible: Vec<Line> = lines.into_iter().skip(scroll).take(content_h).collect();
    f.render_widget(Paragraph::new(visible), area);
}

fn draw_pacing_tab(
    f: &mut Frame,
    area: Rect,
    data: &crate::app::XRayData,
    scroll: usize,
    accent: Color,
    _dim: Color,
    _normal_fg: Color,
    theme: &crate::theme::Theme,
) {
    let mut lines = Vec::new();

    lines.push(Line::from(Span::styled(
        "Pacing Heatmap — Action vs Dialogue",
        Style::default().fg(accent).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    if data.pacing_map.is_empty() {
        lines.push(Line::from(Span::styled(
            "  No page data available.",
            theme.secondary_style().add_modifier(Modifier::ITALIC),
        )));
    } else {
        // Legend
        lines.push(Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled("█", Style::default().fg(accent)),
            Span::styled(" = Action   ", theme.secondary_style()),
            Span::styled("░", Style::default().fg(Color::from(theme.ui.search_highlight_bg.clone()))),
            Span::styled(" = Dialogue", theme.secondary_style()),
        ]));
        lines.push(Line::from(""));

        let bar_w = area.width.saturating_sub(14) as usize;

        for block in &data.pacing_map {
            let total = block.action_lines + block.dialogue_lines;
            if total == 0 {
                lines.push(Line::from(vec![
                    Span::styled(format!("  pg{:<3} ", block.page), theme.secondary_style()),
                    Span::styled("─".repeat(bar_w), theme.secondary_style()),
                ]));
                continue;
            }

            let action_ratio = block.action_lines as f32 / total as f32;
            let action_cells = (action_ratio * bar_w as f32).round() as usize;
            let dialogue_cells = bar_w.saturating_sub(action_cells);

            let pct_str = format!("{:3.0}%A", action_ratio * 100.0);

            lines.push(Line::from(vec![
                Span::styled(format!("  pg{:<3} ", block.page), theme.secondary_style()),
                Span::styled("█".repeat(action_cells), Style::default().fg(accent)),
                Span::styled("░".repeat(dialogue_cells), Style::default().fg(Color::from(theme.ui.search_highlight_bg.clone()))),
                Span::styled(format!(" {}", pct_str), theme.secondary_style()),
            ]));
        }
    }

    let content_h = area.height as usize;
    let max_scroll = lines.len().saturating_sub(content_h);
    let scroll = scroll.min(max_scroll);
    let visible: Vec<Line> = lines.into_iter().skip(scroll).take(content_h).collect();
    f.render_widget(Paragraph::new(visible), area);
}

fn draw_scenes_tab(
    f: &mut Frame,
    area: Rect,
    data: &crate::app::XRayData,
    scroll: usize,
    accent: Color,
    dim: Color,
    normal_fg: Color,
    theme: &crate::theme::Theme,
    use_nerd_fonts: bool,
) {
    let mut lines = Vec::new();

    lines.push(Line::from(Span::styled(
        "Scene Length Analysis",
        Style::default().fg(accent).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    if data.scenes.is_empty() {
        lines.push(Line::from(Span::styled(
            "  No scenes found in script.",
            theme.secondary_style().add_modifier(Modifier::ITALIC),
        )));
    } else {
        let over_count = data.scenes.iter().filter(|s| s.is_over_limit).count();
        lines.push(Line::from(vec![
            Span::styled(
                format!("  {} scenes total", data.scenes.len()),
                Style::default().fg(dim),
            ),
            if over_count > 0 {
                Span::styled(
                    format!("  ·  {} over 3 pages", over_count),
                    theme.warning_style().add_modifier(Modifier::BOLD),
                )
            } else {
                Span::styled(
                    format!("  {}  All scenes within limit", if use_nerd_fonts { " " } else { "* [X]" }),
                    theme.success_style(),
                )
            },
        ]));
        lines.push(Line::from(""));

        // Header
        let max_label_w = area.width.saturating_sub(28) as usize;
        lines.push(Line::from(vec![
            Span::styled(
                format!("  {:>4}  {:<width$}  {:>6}  Status", "№", "Scene", "Pages", width = max_label_w),
                theme.secondary_style().add_modifier(Modifier::BOLD),
            ),
        ]));
        lines.push(Line::from(Span::styled(
            format!("  {}", "─".repeat(area.width.saturating_sub(4) as usize)),
            theme.secondary_style(),
        )));

        for scene in &data.scenes {
            let num_str = scene.scene_num.as_deref().unwrap_or("-").to_string();
            let label = if scene.label.len() > max_label_w {
                format!("{:.width$}...", &scene.label[..max_label_w.saturating_sub(3)], width = max_label_w - 3)
            } else {
                format!("{:<width$}", scene.label, width = max_label_w)
            };

            let pages_str = format!("{:.1}", scene.page_count);

            let (status, status_style) = if scene.is_over_limit {
                (
                    if use_nerd_fonts { " TOO LONG" } else { "[!] TOO LONG" },
                    theme.warning_style().add_modifier(Modifier::BOLD)
                )
            } else {
                (
                    if use_nerd_fonts { " " } else { "[X]" },
                    theme.success_style()
                )
            };

            let line_style = if scene.is_over_limit {
                Style::default().fg(normal_fg)
            } else {
                theme.secondary_style()
            };

            lines.push(Line::from(vec![
                Span::styled(format!("  {:>4}  ", num_str), line_style),
                Span::styled(format!("{}  ", label), line_style),
                Span::styled(format!("{:>5}  ", pages_str), line_style),
                Span::styled(status, status_style),
            ]));
        }
    }

    let content_h = area.height as usize;
    let max_scroll = lines.len().saturating_sub(content_h);
    let scroll = scroll.min(max_scroll);
    let visible: Vec<Line> = lines.into_iter().skip(scroll).take(content_h).collect();
    f.render_widget(Paragraph::new(visible), area);
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
    
    let accent = Color::from(app.theme.ui.navigator_mode_bg.clone());
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
        tag_lines.push(Line::from(vec![
            Span::styled("  Tags for: ", theme.secondary_style()),
            Span::styled(&selected_scene.label, Style::default().fg(accent).add_modifier(Modifier::BOLD)),
        ]));
        tag_lines.push(Line::from(""));

        if selected_scene.breakdown.is_empty() {
            tag_lines.push(Line::from(Span::styled(
                "    No tags in this scene.",
                theme.secondary_style().add_modifier(Modifier::ITALIC),
            )));
        } else {
            for (key, values) in &selected_scene.breakdown {
                tag_lines.push(Line::from(vec![
                    Span::styled(format!("    {}: ", key), Style::default().fg(accent).add_modifier(Modifier::BOLD)),
                ]));
                for v in values {
                    tag_lines.push(Line::from(vec![
                        Span::styled("      · ", theme.secondary_style()),
                        Span::styled(v, theme.secondary_style()),
                    ]));
                }
                tag_lines.push(Line::from(""));
            }
        }
    }

    f.render_widget(Paragraph::new(tag_lines).block(Block::default().title(Span::styled(" [ Details ] ", Style::default().fg(accent).add_modifier(Modifier::BOLD)))), chunks[1]);
}
