pub mod panes;
use self::panes::{
    draw_file_picker, draw_snapshots, draw_sprint_stats, home::draw_home,
    index_cards::draw_index_cards, xray::draw_xray, draw_export_modal,
    quick_help::draw_quick_help,
};

use crate::{
    app::{App, AppMode, EnsembleItem, GoalType, shortcuts},
    formatting::{RenderConfig, StringCaseExt, render_inline},
    layout::{find_visual_cursor, strip_sigils},
    types::{LineType, PAGE_WIDTH, base_style},
};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph, Table, Row, Cell},
};
use std::collections::HashSet;
use unicode_width::UnicodeWidthStr;

pub fn draw(f: &mut Frame, app: &mut App) {
    let area = f.area();
    let theme = &app.theme;
    let dim_color = Color::from(theme.ui.dim.clone());

    let mut base_ui_style = Style::default();
    if let Some(bg) = &theme.ui.background {
        base_ui_style = base_ui_style.bg(Color::from(bg.clone()));
    }
    if let Some(fg) = &theme.ui.foreground {
        base_ui_style = base_ui_style.fg(Color::from(fg.clone()));
    }
    f.render_widget(Block::default().style(base_ui_style), area);

    let (mode_str, mode_bg) = match app.mode {
        AppMode::Normal => (" Normal ", Color::from(theme.ui.normal_mode_bg.clone())),
        AppMode::Command => (" Command ", Color::from(theme.ui.command_mode_bg.clone())),
        AppMode::SceneNavigator => (
            " Navigator ",
            Color::from(theme.ui.navigator_mode_bg.clone()),
        ),
        AppMode::SettingsPane => (" Settings ", Color::from(theme.ui.settings_mode_bg.clone())),
        AppMode::ExportPane => (" Export ", Color::from(theme.ui.normal_mode_bg.clone())),
        AppMode::Shortcuts => (" Legend ", Color::from(theme.ui.normal_mode_bg.clone())),
        AppMode::Search => (" Search ", Color::from(theme.ui.search_mode_bg.clone())),
        AppMode::Home => (" Home ", Color::from(theme.ui.normal_mode_bg.clone())),
        AppMode::FilePicker => (" File ", Color::from(theme.ui.normal_mode_bg.clone())),
        AppMode::Snapshots => (
            " Snapshots ",
            Color::from(theme.ui.navigator_mode_bg.clone()),
        ),
        AppMode::SprintStat => (" Sprints ", Color::from(theme.ui.normal_mode_bg.clone())),
        AppMode::XRay => (" X-Ray ", Color::from(theme.ui.navigator_mode_bg.clone())),
        AppMode::IndexCards => (
            " Index Cards ",
            Color::from(theme.ui.navigator_mode_bg.clone()),
        ),
        AppMode::ReplaceOne | AppMode::ReplaceAll => {
            (" Replace ", Color::from(theme.ui.command_mode_bg.clone()))
        }
        AppMode::StructurePicker => (" Structure ", Color::from(theme.ui.normal_mode_bg.clone())),
        _ => (" Prompt ", Color::from(theme.ui.command_mode_bg.clone())),
    };

    let is_prompt = app.mode != AppMode::Normal;
    let has_status = app.status_msg.is_some();

    let show_bottom = !app.config.focus_mode || is_prompt || has_status;

    let _in_command_mode = app.mode == AppMode::Command;
    let footer_height = if show_bottom { 1 } else { 0 };
    let show_header = !app.config.focus_mode || is_prompt || has_status;
    let header_height: u16 = if show_header { 1 } else { 0 };
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(header_height),
            Constraint::Min(0),
            Constraint::Length(footer_height),
        ])
        .split(area);

    let (header_area, mut text_area, footer_area) = (chunks[0], chunks[1], chunks[2]);

    // ── Header rendering (Zen Style) ──────────────────────────────────────
    {
        let dim_style = theme.secondary_style();
        let sep = " | ";
        let sep_style = dim_style;

        let mut left_spans = Vec::new();

        // Opening bracket
        left_spans.push(Span::styled("[ ", sep_style));

        // Buffer tabs (Always shown on the left)
        for i in 0..app.buffers.len() {
            let (file, dirty) = if i == app.current_buf_idx {
                (&app.file, app.dirty)
            } else {
                (&app.buffers[i].file, app.buffers[i].dirty)
            };

            let name = file
                .as_ref()
                .and_then(|p| p.file_name())
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| "New Script".to_string());

            let dirty_mark = if dirty { "*" } else { "" };
            let label = format!("{}{}", name, dirty_mark);

            if i == app.current_buf_idx {
                left_spans.push(Span::styled(
                    label,
                    Style::default()
                        .fg(Color::from(theme.ui.selection_fg.clone()))
                        .bg(Color::from(theme.ui.selection_bg.clone()))
                        .add_modifier(Modifier::BOLD),
                ));
            } else {
                left_spans.push(Span::styled(label, dim_style));
            }

            if i + 1 < app.buffers.len() {
                left_spans.push(Span::styled(sep, sep_style));
            }
        }

        // Right side: EDITOR | FOUNT VERSION ]
        let mut right_spans = Vec::new();

        // Mode label
        let active_context_mode = if app.mode == AppMode::Command || app.mode == AppMode::Search {
            app.previous_mode
        } else {
            app.mode
        };

        let mode_label = match active_context_mode {
            AppMode::IndexCards => " INDEX CARDS ",
            _ => " EDITOR ",
        };
        right_spans.push(Span::styled(
            mode_label,
            Style::default().fg(mode_bg).add_modifier(Modifier::BOLD),
        ));
        right_spans.push(Span::styled(sep, sep_style));

        // Fount version
        right_spans.push(Span::styled(
            format!("Fount v{}", env!("CARGO_PKG_VERSION")),
            Style::default().fg(mode_bg).add_modifier(Modifier::BOLD),
        ));

        // Closing bracket
        right_spans.push(Span::styled(" ]", sep_style));

        let left_width: usize = left_spans
            .iter()
            .map(|s| UnicodeWidthStr::width(s.content.as_ref()))
            .sum();
        let right_width: usize = right_spans
            .iter()
            .map(|s| UnicodeWidthStr::width(s.content.as_ref()))
            .sum();
        let total_width = header_area.width as usize;

        if total_width > left_width + right_width {
            let pad_len = total_width - left_width - right_width;
            left_spans.push(Span::raw(" ".repeat(pad_len)));
        }

        left_spans.extend(right_spans);
        f.render_widget(Paragraph::new(Line::from(left_spans)), header_area);
    }

    app.sidebar_area = Rect::default();
    if app.mode == AppMode::SceneNavigator || app.mode == AppMode::CharacterNavigator {
        let side_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(41),
                Constraint::Length(1),
                Constraint::Min(0),
            ])
            .split(text_area);
        app.sidebar_area = side_chunks[0];
        let shadow_area = side_chunks[1];
        text_area = side_chunks[2];

        // Draw clean separator
        let sep_col = "│".repeat(shadow_area.height as usize);
        let sep_lines: Vec<Line> = sep_col
            .chars()
            .map(|_| Line::from(Span::styled("│", theme.secondary_style())))
            .collect();
        f.render_widget(Paragraph::new(sep_lines), shadow_area);
    }

    app.settings_area = Rect::default();

    let height = text_area.height as usize;
    app.visible_height = height;
    let page_w = PAGE_WIDTH.min(text_area.width);
    let global_pad = text_area.width.saturating_sub(page_w) / 2;

    let mut pad_top = 0;

    if app.mode != AppMode::Home {
        let (vis_row, _vis_x) = find_visual_cursor(&app.layout, app.cursor_y, app.cursor_x);

        if app.config.typewriter_mode {
            let center_offset = (height / 2) as usize;
            if vis_row < center_offset {
                pad_top = center_offset - vis_row;
            }
            app.scroll = vis_row.saturating_sub(center_offset);
        } else {
            if vis_row < app.scroll {
                app.scroll = vis_row;
            }
            if vis_row >= app.scroll + height {
                app.scroll = vis_row + 1 - height;
            }
        }
    }

    let dark_gray_style = theme.secondary_style();

    let mut sug_style = theme.secondary_style();
    if !app.config.no_formatting {
        sug_style = sug_style.add_modifier(Modifier::BOLD);
    }

    let page_num_style = theme.secondary_style();

    let current_view_mode = if app.mode == AppMode::Command || app.mode == AppMode::Search {
        app.previous_mode
    } else {
        app.mode
    };

    if current_view_mode != AppMode::IndexCards {
        let mut visible: Vec<Line> = Vec::new();
        for _ in 0..pad_top {
            visible.push(Line::raw(""));
        }

        let mirror_scenes = app.config.mirror_scene_numbers == crate::config::MirrorOption::Always;

        visible.extend(
            app.layout
                .iter()
                .skip(app.scroll)
                .take(height.saturating_sub(pad_top))
                .map(|row| {
                    let mut spans = Vec::new();
                    let gap_size = 6u16;
                    let gutter_w = 6u16;
                    let is_current_logical = row.line_idx == app.cursor_y;
                    let is_first_visual = row.char_start == 0;
                    let mut current_pad = global_pad;

                    // 1. Line Number Gutter
                    let show_linenums = app.config.show_line_numbers && !app.config.focus_mode;
                    if global_pad >= gutter_w && show_linenums {
                        if is_first_visual {
                            let num_str = format!("{:>4}  ", row.line_idx + 1);
                            let num_style = if is_current_logical {
                                Style::default().fg(mode_bg).add_modifier(Modifier::BOLD)
                            } else {
                                theme.secondary_style().add_modifier(Modifier::DIM)
                            };
                            spans.push(Span::styled(num_str, num_style));
                        } else {
                            spans.push(Span::raw(" ".repeat(gutter_w as usize)));
                        }
                        current_pad = current_pad.saturating_sub(gutter_w);
                    }

                    // 2. Scene Number or Padding
                    if let Some(ref snum) = row.scene_num {
                        let s_str = snum.to_string();
                        let s_len = UnicodeWidthStr::width(s_str.as_str()) as u16;
                        let gap_size = 6u16;

                        if current_pad >= s_len + gap_size {
                            let pad = current_pad - s_len - gap_size;
                            spans.push(Span::raw(" ".repeat(pad as usize)));
                            spans.push(Span::styled(s_str, dark_gray_style));
                            spans.push(Span::raw(" ".repeat(gap_size as usize)));
                        } else {
                            spans.push(Span::styled(s_str, dark_gray_style));
                            spans.push(Span::raw(" "));
                        }
                    } else {
                        spans.push(Span::raw(" ".repeat(current_pad as usize)));
                    }

                    spans.push(Span::raw(" ".repeat(row.indent as usize)));

                    let mut bst = base_style(row.line_type, &app.config, &app.theme);
                    if let Some(c) = row.override_color
                        && !app.config.no_color
                    {
                        bst.fg = Some(c);
                    }

                    if app.config.highlight_active_action && !row.is_active {
                        bst = bst.add_modifier(Modifier::DIM);
                    }

                    let mut display = if row.is_active || !app.config.hide_markup {
                        row.raw_text.clone()
                    } else {
                        strip_sigils(&row.raw_text, row.line_type).to_string()
                    };

                    let reveal_markup = !app.config.hide_markup || row.is_active;
                    let skip_md = row.line_type == LineType::Boneyard;

                    if matches!(
                        row.line_type,
                        LineType::SceneHeading | LineType::Transition | LineType::Shot
                    ) {
                        display = display.to_uppercase_1to1();
                    } else if matches!(
                        row.line_type,
                        LineType::Character | LineType::DualDialogueCharacter
                    ) {
                        if let Some(idx) = display.find('(') {
                            let name = display[..idx].to_uppercase_1to1();
                            let ext = &display[idx..];
                            display = format!("{}{}", name, ext);
                        } else {
                            display = display.to_uppercase_1to1();
                        }
                    }

                    let empty_logical_line = String::new();
                    let full_logical_line =
                        app.lines.get(row.line_idx).unwrap_or(&empty_logical_line);

                    let is_last_visual_row = row.char_end == full_logical_line.chars().count();
                    let mut meta_key_end = 0;

                    if (row.line_type == LineType::MetadataKey
                        || (row.line_type == LineType::MetadataTitle && row.is_active))
                        && let Some(idx) = full_logical_line.find(':')
                    {
                        meta_key_end = full_logical_line[..=idx].chars().count() + 1;
                    }

                    let mut row_highlights = HashSet::new();
                    if app.show_search_highlight
                        && let Some(re) = &app.compiled_search_regex
                    {
                        for mat in re.find_iter(full_logical_line) {
                            let start_byte = mat.start();
                            let end_byte = mat.end();

                            let char_start = full_logical_line[..start_byte].chars().count();
                            let char_len = full_logical_line[start_byte..end_byte].chars().count();

                            for idx in char_start..(char_start + char_len) {
                                row_highlights.insert(idx);
                            }
                        }
                    }

                    // Selection highlight (overrides search)
                    let mut sel_highlights = HashSet::new();
                    if let Some(((sel_sl, sel_sc), (sel_el, sel_ec))) = app.selection_range() {
                        let li = row.line_idx;
                        if li >= sel_sl && li <= sel_el {
                            let line_len = full_logical_line.chars().count();
                            let from = if li == sel_sl { sel_sc } else { 0 };
                            let to = if li == sel_el {
                                sel_ec.min(line_len)
                            } else {
                                line_len
                            };
                            for idx in from..to {
                                sel_highlights.insert(idx);
                            }
                        }
                    }

                    // Merge: sel_highlights takes priority — remove those from row_highlights
                    // so render_inline gets the clean search set, and we'll override selected below.
                    for idx in &sel_highlights {
                        row_highlights.remove(idx);
                    }

                    spans.extend(render_inline(
                        &display,
                        bst,
                        &row.fmt,
                        RenderConfig {
                            reveal_markup,
                            skip_markdown: skip_md,
                            exclude_comments: false,
                            char_offset: row.char_start,
                            meta_key_end,
                            no_color: app.config.no_color,
                            no_formatting: app.config.no_formatting,
                        },
                        &row_highlights,
                        &sel_highlights,
                    ));

                    if row.is_active
                        && row.line_idx == app.cursor_y
                        && is_last_visual_row
                        && let Some(sug) = &app.suggestion
                    {
                        spans.push(Span::styled(sug.clone(), sug_style));
                    }

                    let right_text = if mirror_scenes {
                        row.scene_num.clone()
                    } else {
                        row.page_num.map(|pnum| format!("{}.", pnum))
                    };

                    if let Some(r_str) = right_text {
                        let current_line_width: usize = spans
                            .iter()
                            .map(|s| UnicodeWidthStr::width(s.content.as_ref()))
                            .sum();

                        let target_pos = global_pad as usize + page_w as usize + gap_size as usize;
                        if target_pos > current_line_width {
                            spans.push(Span::raw(" ".repeat(target_pos - current_line_width)));
                            spans.push(Span::styled(r_str, page_num_style));
                        }
                    }

                    // Revision Mark (*) rendering
                    if app.revised_lines.get(row.line_idx).cloned().unwrap_or(false) {
                        let current_line_width: usize = spans
                            .iter()
                            .map(|s| UnicodeWidthStr::width(s.content.as_ref()))
                            .sum();

                        let rev_pos = global_pad as usize + page_w as usize + 2;
                        if rev_pos > current_line_width {
                            spans.push(Span::raw(" ".repeat(rev_pos - current_line_width)));
                        }

                        let rev_color = if theme.is_dark() {
                            Color::Yellow
                        } else {
                            Color::Red
                        };

                        spans.push(Span::styled(
                            " *",
                            Style::default().fg(rev_color).add_modifier(Modifier::BOLD),
                        ));
                    }

                    Line::from(spans)
                }),
        );

        f.render_widget(Paragraph::new(visible), text_area);
    }

    if app.mode == AppMode::SceneNavigator {
        let selected_bg = Color::from(theme.ui.selection_bg.clone());
        let selected_fg = Color::from(theme.ui.selection_fg.clone());
        let header_color = theme
            .sidebar
            .section_header
            .clone()
            .map(Color::from)
            .unwrap_or(mode_bg);

        let items: Vec<ListItem> = app
            .scenes
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let is_selected = i == app.selected_scene;
                let mut lines = Vec::new();

                let line_style = theme.secondary_style();

                if item.is_section {
                    let style = if is_selected {
                        Style::default()
                            .fg(selected_fg)
                            .bg(selected_bg)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                            .fg(header_color)
                            .add_modifier(Modifier::BOLD)
                    };

                    let prefix = if is_selected {
                        if app.config.use_nerd_fonts {
                            " 󰁔 "
                        } else {
                            " ▸ "
                        }
                    } else {
                        "   "
                    };
                    let max_section_w = 32;
                    let mut current_line = String::new();
                    let mut first_line = true;

                    for word in item.label.to_uppercase().split_whitespace() {
                        if current_line.len() + word.len() + 1 > max_section_w {
                            if first_line {
                                lines.push(Line::from(vec![
                                    Span::styled(prefix, style),
                                    Span::styled(
                                        if app.config.use_nerd_fonts {
                                            "󰉋 "
                                        } else {
                                            "◆ "
                                        },
                                        style,
                                    ),
                                    Span::styled(current_line.clone(), style),
                                ]));
                                first_line = false;
                            } else {
                                lines.push(Line::from(vec![
                                    Span::styled("   ", Style::default()),
                                    Span::styled("│  ", line_style),
                                    Span::styled(current_line.clone(), style),
                                ]));
                            }
                            current_line = word.to_string();
                        } else {
                            if !current_line.is_empty() {
                                current_line.push(' ');
                            }
                            current_line.push_str(word);
                        }
                    }
                    if !current_line.is_empty() {
                        if first_line {
                            lines.push(Line::from(vec![
                                Span::styled(prefix, style),
                                Span::styled(
                                    if app.config.use_nerd_fonts {
                                        "󰉋 "
                                    } else {
                                        "◆ "
                                    },
                                    style,
                                ),
                                Span::styled(current_line, style),
                            ]));
                        } else {
                            lines.push(Line::from(vec![
                                Span::styled("   ", Style::default()),
                                Span::styled("│  ", line_style),
                                Span::styled(current_line, style),
                            ]));
                        }
                    }

                    if !item.synopses.is_empty() {
                        let dim_style = if is_selected {
                            style
                        } else {
                            theme.secondary_style().add_modifier(Modifier::ITALIC)
                        };

                        for syn in &item.synopses {
                            let mut current_line = String::new();
                            let max_syn_w = 32;

                            for word in syn.split_whitespace() {
                                if current_line.len() + word.len() + 1 > max_syn_w {
                                    lines.push(Line::from(vec![
                                        Span::styled("   ", Style::default()),
                                        Span::styled("│  ", line_style),
                                        Span::styled(current_line.clone(), dim_style),
                                    ]));
                                    current_line = word.to_string();
                                } else {
                                    if !current_line.is_empty() {
                                        current_line.push(' ');
                                    }
                                    current_line.push_str(word);
                                }
                            }
                            if !current_line.is_empty() {
                                lines.push(Line::from(vec![
                                    Span::styled("   ", Style::default()),
                                    Span::styled("│  ", line_style),
                                    Span::styled(current_line, dim_style),
                                ]));
                            }
                        }
                    }

                    if i + 1 < app.scenes.len() && !app.scenes[i + 1].is_section {
                        lines.push(Line::from(vec![
                            Span::styled("   ", Style::default()),
                            Span::styled("│", line_style),
                        ]));
                    } else {
                        lines.push(Line::from(""));
                    }
                } else {
                    let mut base_style = if is_selected {
                        Style::default()
                            .bg(selected_bg)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().add_modifier(Modifier::BOLD)
                    };

                    if let Some(c) = item.color {
                        base_style = base_style.fg(c);
                    } else if is_selected {
                        base_style = base_style.fg(selected_fg);
                    } else if let Some(c) = &theme.ui.foreground {
                        base_style = base_style.fg(Color::from(c.clone()));
                    }

                    let prefix = if is_selected { " ▸ " } else { "   " };

                    let is_last_in_section =
                        i + 1 == app.scenes.len() || app.scenes[i + 1].is_section;

                    let has_parent_section = app.scenes[..i].iter().rev().any(|s| s.is_section);

                    let connector = if !has_parent_section {
                        "   "
                    } else if is_last_in_section {
                        "└─ "
                    } else {
                        "├─ "
                    };

                    let s_tag = if let Some(ref s) = item.scene_num {
                        format!("{}. ", s)
                    } else {
                        String::new()
                    };

                    let max_heading_w = 33;
                    let heading_indent = 5;
                    let mut current_heading_len = 0;

                    let mut current_spans = vec![
                        Span::styled(prefix, base_style),
                        Span::styled(connector, line_style),
                    ];

                    if !s_tag.is_empty() {
                        current_spans.push(Span::styled(s_tag.clone(), base_style));
                        current_heading_len += s_tag.len();
                    }

                    for word in item.label.split_whitespace() {
                        if current_heading_len + word.len() + heading_indent + 1 > max_heading_w {
                            lines.push(Line::from(current_spans));
                            let cont_char = if !has_parent_section || is_last_in_section {
                                "   "
                            } else {
                                "│  "
                            };
                            current_spans = vec![
                                Span::styled("   ", Style::default()),
                                Span::styled(cont_char, line_style),
                            ];
                            current_heading_len = 0;
                        }

                        if current_heading_len > 0 {
                            current_spans.push(Span::styled(" ", base_style));
                            current_heading_len += 1;
                        }

                        current_spans.push(Span::styled(word.to_string(), base_style));
                        current_heading_len += word.len();
                    }

                    lines.push(Line::from(current_spans));

                    if !item.synopses.is_empty() {
                        let cont_char = if !has_parent_section || is_last_in_section {
                            "   "
                        } else {
                            "│  "
                        };
                        let dim_style = if is_selected {
                            base_style
                        } else {
                            theme.secondary_style().add_modifier(Modifier::ITALIC)
                        };

                        for syn in &item.synopses {
                            let mut current_line = String::new();
                            let max_syn_w = 32;

                            for word in syn.split_whitespace() {
                                if current_line.len() + word.len() + 1 > max_syn_w {
                                    lines.push(Line::from(vec![
                                        Span::styled("   ", Style::default()),
                                        Span::styled(cont_char, line_style),
                                        Span::styled(current_line.clone(), dim_style),
                                    ]));
                                    current_line = word.to_string();
                                } else {
                                    if !current_line.is_empty() {
                                        current_line.push(' ');
                                    }
                                    current_line.push_str(word);
                                }
                            }
                            if !current_line.is_empty() {
                                lines.push(Line::from(vec![
                                    Span::styled("   ", Style::default()),
                                    Span::styled(cont_char, line_style),
                                    Span::styled(current_line, dim_style),
                                ]));
                            }
                        }
                    }

                    if !is_last_in_section && has_parent_section {
                        lines.push(Line::from(vec![
                            Span::styled("   ", Style::default()),
                            Span::styled("│", line_style),
                        ]));
                    } else {
                        lines.push(Line::from(""));
                    }
                }

                ListItem::new(lines)
            })
            .collect();

        let list = List::new(items).highlight_style(Style::default());
        f.render_stateful_widget(
            list,
            app.sidebar_area.inner(ratatui::layout::Margin {
                horizontal: 0,
                vertical: 1,
            }),
            &mut app.navigator_state,
        );
    }

    if app.mode == AppMode::CharacterNavigator {
        let _border_color = theme
            .sidebar
            .border
            .clone()
            .map(Color::from)
            .unwrap_or_else(|| theme.ui.dim.clone().into());
        let selected_bg = theme
            .sidebar
            .item_selected_bg
            .clone()
            .map(Color::from)
            .unwrap_or(mode_bg);
        let selected_fg = theme
            .sidebar
            .item_selected_fg
            .clone()
            .map(Color::from)
            .unwrap_or_else(|| theme.ui.selection_fg.clone().into());
        let dim_color = theme
            .sidebar
            .item_dimmed
            .clone()
            .map(Color::from)
            .unwrap_or_else(|| theme.ui.dim.clone().into());
        let header_color = theme
            .sidebar
            .section_header
            .clone()
            .map(Color::from)
            .unwrap_or(mode_bg);

        let items: Vec<ListItem> = app
            .ensemble_items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let is_selected = i == app.selected_ensemble_idx;
                let mut lines = Vec::new();

                let line_style = theme.secondary_style();
                let base_style = if is_selected {
                    Style::default()
                        .fg(selected_fg)
                        .bg(selected_bg)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                let dim_style = if is_selected {
                    base_style
                } else {
                    Style::default().fg(dim_color).add_modifier(Modifier::DIM)
                };

                let prefix = if is_selected {
                    if app.config.use_nerd_fonts {
                        " 󰁔 "
                    } else {
                        " > "
                    }
                } else {
                    "   "
                };

                match item {
                    EnsembleItem::CharacterHeader(char_idx) => {
                        let char_item = &app.character_stats[*char_idx];
                        lines.push(Line::from(vec![
                            Span::styled(prefix, base_style),
                            Span::styled(
                                char_item.name.clone(),
                                Style::default()
                                    .fg(header_color)
                                    .add_modifier(Modifier::BOLD),
                            ),
                        ]));
                    }
                    EnsembleItem::Stat(text, hint, is_last) => {
                        let connector = if text.is_empty() {
                            "│"
                        } else if *is_last {
                            "└─ "
                        } else {
                            "├─ "
                        };

                        let mut spans = vec![
                            Span::styled("   ", Style::default()),
                            Span::styled(connector, line_style),
                            Span::styled(text.clone(), dim_style.add_modifier(Modifier::ITALIC)),
                        ];

                        if let Some(h) = hint {
                            spans.push(Span::styled(
                                format!(" {}", h),
                                line_style.add_modifier(Modifier::ITALIC),
                            ));
                        }

                        lines.push(Line::from(spans));
                    }
                    EnsembleItem::SceneLink(name, _, _) => {
                        lines.push(Line::from(vec![
                            Span::styled(prefix, base_style),
                            Span::styled("│  └─ ", line_style),
                            Span::styled(name.clone(), dim_style.add_modifier(Modifier::ITALIC)),
                        ]));
                    }
                    EnsembleItem::Separator => {
                        lines.push(Line::from(""));
                    }
                }

                ListItem::new(lines)
            })
            .collect();

        let title = format!(" [ Ensemble ({}) ]", app.character_stats.len());
        let list = List::new(items)
            .block(Block::default().title(Span::styled(
                title,
                Style::default().fg(Color::from(theme.ui.dim.clone())),
            )))
            .highlight_style(Style::default());

        f.render_stateful_widget(
            list,
            app.sidebar_area.inner(ratatui::layout::Margin {
                horizontal: 0,
                vertical: 1,
            }),
            &mut app.ensemble_state,
        );
    }

    if app.mode == AppMode::SettingsPane {
        let settings = vec![
            ("Typewriter Mode", &app.config.typewriter_mode),
            ("Auto-Save", &app.config.auto_save),
            ("Autocomplete", &app.config.autocomplete),
            ("Auto-Breaks", &app.config.auto_paragraph_breaks),
            ("Focus Mode", &app.config.focus_mode),
            ("Theme", &false), // Not a toggle
        ];

        let theme_name = &app.config.theme;
        let items: Vec<ListItem> = settings
            .into_iter()
            .enumerate()
            .map(|(i, (label, value))| {
                let is_selected = i == app.selected_setting;
                let style = if is_selected {
                    Style::default()
                        .fg(theme.ui.selection_fg.clone().into())
                        .bg(mode_bg)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                let (icon, icon_style) = if label == "Theme" {
                    (
                        "[T] ",
                        Style::default().fg(Color::from(theme.ui.normal_mode_bg.clone())),
                    )
                } else if *value {
                    ("[X] ", theme.success_style())
                } else {
                    ("[ ] ", theme.secondary_style())
                };

                let line = if label == "Theme" {
                    Line::from(vec![
                        Span::styled(if is_selected { " > " } else { "   " }, style),
                        Span::styled(icon, if is_selected { style } else { icon_style }),
                        Span::styled(format!("{}: {}", label, theme_name), style),
                    ])
                } else {
                    Line::from(vec![
                        Span::styled(if is_selected { " > " } else { "   " }, style),
                        Span::styled(icon, if is_selected { style } else { icon_style }),
                        Span::styled(label, style),
                    ])
                };

                ListItem::new(line)
            })
            .collect();

        let list = List::new(items);
        f.render_widget(
            list,
            app.settings_area.inner(ratatui::layout::Margin {
                horizontal: 0,
                vertical: 1,
            }),
        );
    }

    if app.mode == AppMode::ExportPane {
        draw_export_modal(f, app);
    }

    if app.mode == AppMode::Shortcuts {
        let modal_area = panes::centered_rect(55, 70, area);
        f.render_widget(ratatui::widgets::Clear, modal_area);

        let bg = theme.primary_bg();
        let fg = theme.primary_fg();

        let all_shortcuts = shortcuts::get_all_shortcuts();
        let categories = shortcuts::get_categories(&all_shortcuts);
        let query = app.shortcuts_query.to_lowercase();
        let is_searching = !query.is_empty() || app.is_shortcuts_searching;

        let visible_shortcuts: Vec<&shortcuts::Shortcut> = if !query.is_empty() {
            shortcuts::filter_shortcuts(&all_shortcuts, &query)
        } else if !categories.is_empty() {
            let cat = &categories[app.shortcuts_selected_tab.min(categories.len().saturating_sub(1))];
            shortcuts::shortcuts_in_category(&all_shortcuts, cat)
        } else {
            all_shortcuts.iter().collect()
        };

        let outer_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(dim_color))
            .style(Style::default().bg(bg).fg(fg))
            .title(Span::styled(
                " [ Cheat Sheet ] ",
                Style::default().fg(mode_bg).add_modifier(Modifier::BOLD),
            ));

        let inner_area = outer_block.inner(modal_area);
        f.render_widget(outer_block, modal_area);

        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(1),
                Constraint::Min(0),
                Constraint::Length(1),
            ])
            .split(inner_area);

        let tab_area = main_chunks[0];
        let content_area = main_chunks[2];
        let footer_hint_area = main_chunks[3];

        if is_searching {
            let search_style = if app.is_shortcuts_searching {
                Style::default().fg(mode_bg).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(dim_color)
            };

            let search_text = if app.shortcuts_query.is_empty() && app.is_shortcuts_searching {
                " Type to filter...".to_string()
            } else if app.shortcuts_query.is_empty() {
                " Press [/] to search...".to_string()
            } else {
                format!("  {}", app.shortcuts_query)
            };

            let search_block = Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(search_style)
                .title(Span::styled(" Search ", search_style));

            f.render_widget(
                Paragraph::new(search_text).block(search_block),
                tab_area,
            );
        } else {
            let tab_inner = tab_area.inner(ratatui::layout::Margin {
                horizontal: 1,
                vertical: 0,
            });

            let mut tab_spans: Vec<Span> = Vec::new();
            for (i, cat) in categories.iter().enumerate() {
                let is_active = i == app.shortcuts_selected_tab;

                let short_name = match cat.as_str() {
                    "Essential Controls" => "Essential",
                    "Edit & History" => "Edit",
                    "File & Project" => "File",
                    "Selection & Editing" => "Selection",
                    "Search & Replace" => "Search",
                    "Navigation & Motion" => "Navigate",
                    "Production Tools" => "Production",
                    other => other,
                };

                if is_active {
                    tab_spans.push(Span::styled(
                        format!(" {} ", short_name),
                        Style::default()
                            .fg(bg)
                            .bg(mode_bg)
                            .add_modifier(Modifier::BOLD),
                    ));
                } else {
                    tab_spans.push(Span::styled(
                        format!(" {} ", short_name),
                        Style::default().fg(dim_color),
                    ));
                }
                if i < categories.len() - 1 {
                    tab_spans.push(Span::styled(" ", Style::default()));
                }
            }

            f.render_widget(
                Paragraph::new(Line::from(tab_spans)).alignment(Alignment::Center),
                Rect::new(tab_inner.x, tab_inner.y + 1, tab_inner.width, 1),
            );
        }

        let sep_line = Line::from(Span::styled(
            "─".repeat(inner_area.width.saturating_sub(2) as usize),
            Style::default().fg(dim_color),
        ));
        f.render_widget(
            Paragraph::new(sep_line).alignment(Alignment::Center),
            main_chunks[1],
        );

        let scroll_idx = app.shortcuts_state.selected().unwrap_or(0);
        let available_h = content_area.height as usize;
        let total_items = visible_shortcuts.len();

        let scroll_offset = if scroll_idx >= available_h.saturating_sub(2) {
            scroll_idx.saturating_sub(available_h.saturating_sub(3))
        } else {
            0
        };

        let mut rows = Vec::new();

        for sc in visible_shortcuts.iter() {
            let key_color = sc.color.resolve(theme);
            
            rows.push(Row::new(vec![
                Cell::from(Span::styled(format!(" {:<10}", sc.key), Style::default().fg(key_color).add_modifier(Modifier::BOLD))),
                Cell::from(Span::styled(format!(" {:<16}", sc.label), Style::default().fg(fg).add_modifier(Modifier::BOLD))),
                Cell::from(Span::styled(format!(" {}", sc.desc), Style::default().fg(dim_color))),
            ]));
        }

        if rows.is_empty() {
            rows.push(Row::new(vec![
                Cell::from(""),
                Cell::from(Span::styled(
                    if query.is_empty() { "No shortcuts found" } else { "No matches found" },
                    Style::default().fg(dim_color).add_modifier(Modifier::ITALIC),
                )),
                Cell::from(""),
            ]));
        }

        let table = Table::new(
            rows,
            [
                Constraint::Length(14),
                Constraint::Length(20),
                Constraint::Min(20),
            ],
        )
        .header(
            Row::new(vec![
                Cell::from(Span::styled(" KEY", Style::default().fg(mode_bg).add_modifier(Modifier::BOLD))),
                Cell::from(Span::styled(" ACTION", Style::default().fg(mode_bg).add_modifier(Modifier::BOLD))),
                Cell::from(Span::styled(" DESCRIPTION", Style::default().fg(mode_bg).add_modifier(Modifier::BOLD))),
            ])
            .bottom_margin(1)
        )
        .column_spacing(4);

        f.render_stateful_widget(table, content_area, &mut app.shortcuts_state);

        let hint_spans = if app.is_shortcuts_searching {
            vec![
                Span::styled(" [Esc] ", Style::default().fg(mode_bg).add_modifier(Modifier::BOLD)),
                Span::styled("Close Search  ", Style::default().fg(dim_color)),
            ]
        } else {
            vec![
                Span::styled(" [←/→] ", Style::default().fg(mode_bg).add_modifier(Modifier::BOLD)),
                Span::styled("Category  ", Style::default().fg(dim_color)),
                Span::styled(" [↑/↓] ", Style::default().fg(mode_bg).add_modifier(Modifier::BOLD)),
                Span::styled("Scroll  ", Style::default().fg(dim_color)),
                Span::styled(" [/] ", Style::default().fg(mode_bg).add_modifier(Modifier::BOLD)),
                Span::styled("Search  ", Style::default().fg(dim_color)),
                Span::styled(" [Esc] ", Style::default().fg(mode_bg).add_modifier(Modifier::BOLD)),
                Span::styled("Close", Style::default().fg(dim_color)),
            ]
        };

        let scroll_indicator = if total_items > available_h {
            format!(" {}/{} ", scroll_offset + 1, total_items)
        } else {
            String::new()
        };

        let mut footer_spans = hint_spans;
        if !scroll_indicator.is_empty() {
            footer_spans.push(Span::styled("  ", Style::default()));
            footer_spans.push(Span::styled(
                scroll_indicator,
                Style::default().fg(dim_color).add_modifier(Modifier::ITALIC),
            ));
        }

        f.render_widget(
            Paragraph::new(Line::from(footer_spans)).alignment(Alignment::Center),
            footer_hint_area,
        );
    }

    // ── Footer rendering (Zen Style) ────────────────────────────────────────
    if footer_area.height > 0 {
        let dim_style = theme.secondary_style();
        let sep = " | ";
        let sep_style = dim_style;

        let mut spans = Vec::new();

        // Opening bracket
        spans.push(Span::styled("[ ", sep_style));

        // Mode label (title-case, calmer than SCREAMING)
        spans.push(Span::styled(
            mode_str.trim(),
            Style::default().fg(mode_bg).add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(sep, sep_style));

        // Filename + dirty/lock indicators
        let fname = app
            .file
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "New Script".to_string());
        let dirty_str = if app.dirty { "*" } else { "" };
        let lock_str = if app.config.production_lock {
            " [L]"
        } else {
            ""
        };
        spans.push(Span::styled(
            format!("{}{}{}", fname, dirty_str, lock_str),
            Style::default().fg(mode_bg),
        ));

        // Saved indicator
        if let Some(time) = app.save_indicator_timer {
            let elapsed = time.elapsed().as_secs_f32();
            if elapsed < 2.0 {
                spans.push(Span::styled(
                    "  [X] Saved",
                    theme.success_style().add_modifier(Modifier::BOLD),
                ));
            }
        }

        spans.push(Span::styled(sep, sep_style));

        // Center content: command, search, status, or hint
        if app.mode == AppMode::Command {
            let cmd_style = if app.command_error {
                theme.error_style().add_modifier(Modifier::BOLD)
            } else {
                Style::default().add_modifier(Modifier::BOLD)
            };
            spans.push(Span::styled("/", cmd_style));
            spans.push(Span::styled(&app.command_input, cmd_style));

            if !app.command_input.is_empty() && !app.command_error {
                let commands = app.get_command_completions();
                let input_lower = app.command_input.to_lowercase();
                if let Some(first_match) = commands.iter().find(|&c| {
                    c.to_lowercase().starts_with(&input_lower) && c.to_lowercase() != input_lower
                }) {
                    let remainder = &first_match[app.command_input.len()..];
                    spans.push(Span::styled(
                        remainder.to_string(),
                        Style::default().fg(dim_color),
                    ));
                }
            }

            if app.command_input.is_empty() && !app.command_error {
                spans.push(Span::styled(
                    " type a command...",
                    Style::default().fg(dim_color),
                ));
            }
        } else if matches!(
            app.mode,
            AppMode::Search
                | AppMode::PromptSave
                | AppMode::PromptFilename
                | AppMode::ReplaceOne
                | AppMode::ReplaceAll
        ) {
            match app.mode {
                AppMode::Search => {
                    let prompt_base = if app.last_search.is_empty() {
                        "Search: ".to_string()
                    } else {
                        format!("Search [{}]: ", app.last_search)
                    };

                    let mut count_msg = String::new();
                    if !app.search_matches.is_empty() {
                        let cur = app.current_match_idx.map(|idx| idx + 1).unwrap_or(0);
                        count_msg = format!(" [{}/{}]", cur, app.search_matches.len());
                    }

                    spans.push(Span::raw(format!("{}{}", prompt_base, app.search_query)));
                    if !count_msg.is_empty() {
                        spans.push(Span::styled(count_msg, theme.secondary_style()));
                    }
                    spans.push(Span::styled(
                        " [Alt+^/v] Navigate",
                        theme.secondary_style(),
                    ));
                }
                AppMode::ReplaceOne => {
                    spans.push(Span::raw(format!("Replace: {} ", app.command_input)))
                }
                AppMode::ReplaceAll => {
                    spans.push(Span::raw(format!("Replace All: {} ", app.command_input)))
                }
                AppMode::PromptSave => spans.push(Span::raw("Save modified script? (y/n/c) ")),
                AppMode::PromptFilename => {
                    spans.push(Span::raw(format!("Filename: {} ", app.filename_input)))
                }
                _ => {}
            }
        } else if app.status_msg.is_some() || app.show_search_highlight {
            if let Some(msg) = &app.status_msg {
                let style = if app.command_error {
                    theme.error_style()
                } else {
                    theme.secondary_style().add_modifier(Modifier::ITALIC)
                };
                spans.push(Span::styled(msg, style));
            }

            if app.show_search_highlight && !app.search_matches.is_empty() {
                spans.push(Span::styled(
                    " [Alt+^/v] Nav [r] Replace [R] Replace All",
                    theme.secondary_style(),
                ));
            }
        } else if app.mode != AppMode::Home && app.mode != AppMode::IndexCards {
            let scene_name = app.get_current_scene_name();
            spans.push(Span::styled(
                scene_name,
                Style::default().fg(mode_bg).add_modifier(Modifier::BOLD),
            ));
            spans.push(Span::styled("F1 Reference", theme.secondary_style()));
        }

        // Sprint progress (if active)
        if let Some(GoalType::Sprint {
            start_time,
            duration,
            start_words,
            ..
        }) = &app.active_goal
        {
            let elapsed = start_time.elapsed();
            let pct = (elapsed.as_secs_f64() / duration.as_secs_f64()).min(1.0);
            let bar_width = 8;
            let filled = (pct * bar_width as f64) as usize;
            let empty = bar_width - filled;

            let remaining = duration.saturating_sub(elapsed);
            let rem_min = remaining.as_secs() / 60;
            let rem_sec = remaining.as_secs() % 60;

            let current_words = app.total_word_count();
            let words_written = current_words.saturating_sub(*start_words);

            let sprint_msg = format!(
                " | Sprint [{}{}] {:02}:{:02} +{}w",
                "█".repeat(filled),
                "░".repeat(empty),
                rem_min,
                rem_sec,
                words_written
            );
            spans.push(Span::styled(sprint_msg, Style::default().fg(mode_bg)));
        }

        // Right-side info: word count, line count, cursor position
        let mut right_spans = Vec::new();

        let current_context_mode = if app.mode == AppMode::Command || app.mode == AppMode::Search {
            app.previous_mode
        } else {
            app.mode
        };

        if current_context_mode == AppMode::IndexCards {
            let sections = app.index_cards.iter().filter(|c| c.is_section).count();
            let scenes = app.index_cards.len() - sections;

            right_spans.push(Span::styled(sep, sep_style));
            right_spans.push(Span::styled(
                "[?] Quick Help",
                dim_style,
            ));
            right_spans.push(Span::styled(sep, sep_style));
            right_spans.push(Span::styled(
                format!("{} Sections, {} Scenes", sections, scenes),
                dim_style,
            ));
        } else {
            let word_count = app.total_word_count();
            let pos_str = format!("Ln {}, Col {}", app.cursor_y + 1, app.cursor_x + 1);

            right_spans.push(Span::styled(sep, sep_style));
            right_spans.push(Span::styled(
                format!("{} words", word_count),
                dim_style,
            ));
            right_spans.push(Span::styled(sep, sep_style));
            right_spans.push(Span::styled(pos_str, dim_style));
        }
        // Closing bracket
        right_spans.push(Span::styled(" ]", sep_style));

        let left_width: usize = spans
            .iter()
            .map(|s| UnicodeWidthStr::width(s.content.as_ref()))
            .sum();
        let right_width: usize = right_spans
            .iter()
            .map(|s| UnicodeWidthStr::width(s.content.as_ref()))
            .sum();
        let total_width = footer_area.width as usize;

        if total_width > left_width + right_width {
            let pad_len = total_width - left_width - right_width;
            spans.push(Span::raw(" ".repeat(pad_len)));
        }

        spans.extend(right_spans);
        f.render_widget(Paragraph::new(Line::from(spans)), footer_area);

        // Cursor Handling for Footer Modes
        if matches!(
            app.mode,
            AppMode::Search
                | AppMode::Command
                | AppMode::PromptSave
                | AppMode::PromptFilename
                | AppMode::ReplaceOne
                | AppMode::ReplaceAll
        ) && footer_area.height > 0
        {
            // Calculate prefix width of the left side content
            let fname = app
                .file
                .as_ref()
                .and_then(|p| p.file_name())
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| "New Script".to_string());
            let dirty_str = if app.dirty { "*" } else { "" };
            let lock_str = if app.config.production_lock {
                " [L]"
            } else {
                ""
            };

            let prefix_w = 2 // "[ "
                + UnicodeWidthStr::width(mode_str.trim())
                + 3 // " | "
                + UnicodeWidthStr::width(fname.as_str()) + UnicodeWidthStr::width(dirty_str) + UnicodeWidthStr::width(lock_str)
                + 3; // " | "

            let (input_prefix, input_content) = match app.mode {
                AppMode::Search => {
                    let pb = if app.last_search.is_empty() {
                        "Search: ".to_string()
                    } else {
                        format!("Search [{}]: ", app.last_search)
                    };
                    (pb, app.search_query.clone())
                }
                AppMode::ReplaceOne => ("Replace: ".to_string(), app.command_input.clone()),
                AppMode::ReplaceAll => ("Replace All: ".to_string(), app.command_input.clone()),
                AppMode::Command => ("/".to_string(), app.command_input.clone()),
                AppMode::PromptFilename => ("Filename: ".to_string(), app.filename_input.clone()),
                AppMode::PromptSave => {
                    ("Save modified script? (y/n/c) ".to_string(), "".to_string())
                }
                _ => (String::new(), String::new()),
            };

            let cur_x = footer_area.x
                + (prefix_w
                    + UnicodeWidthStr::width(input_prefix.as_str())
                    + UnicodeWidthStr::width(input_content.as_str())) as u16;
            f.set_cursor_position((cur_x, footer_area.y));
        }
    }

    // -- Screen Blink Effect --
    if app.flash_timer.is_some() {
        f.render_widget(
            Block::default().style(Style::default().bg(theme.primary_fg())),
            area,
        );
    }

    // -- Cursor Handling --
    if app.mode != AppMode::Command && app.mode != AppMode::Home {
        let (vis_row, vis_x) = find_visual_cursor(&app.layout, app.cursor_y, app.cursor_x);

        match app.mode {
            AppMode::Normal => {
                let cur_screen_y =
                    text_area.y + pad_top as u16 + (vis_row.saturating_sub(app.scroll)) as u16;
                let cur_screen_x = text_area.x + global_pad + vis_x;
                if cur_screen_y < text_area.y + text_area.height {
                    f.set_cursor_position((cur_screen_x, cur_screen_y));
                }
            }
            _ => {}
        }
    }

    // -- Minimalist Home Screen --
    if app.mode == AppMode::Home {
        draw_home(f, app);
    }

    if app.mode == AppMode::FilePicker {
        draw_file_picker(f, app, area);
    }

    if app.mode == AppMode::Snapshots {
        draw_snapshots(f, app);
    }

    if app.mode == AppMode::SprintStat {
        draw_sprint_stats(f, app);
    }

    if app.mode == AppMode::XRay {
        draw_xray(f, app);
    }

    if current_view_mode == AppMode::IndexCards {
        draw_index_cards(f, app, text_area);
    }

    match app.mode {
        AppMode::StructurePicker => {
            panes::structure_picker::draw_structure_picker(f, app);
        }
        AppMode::ThemePicker => {
            panes::theme_picker::draw_theme_picker(f, app, area);
        }
        AppMode::SettingsPane => {
            panes::settings::draw_settings_modal(f, app, area);
        }
        _ => {}
    }

    if app.show_quick_help {
        draw_quick_help(f, app, area);
    }
}
