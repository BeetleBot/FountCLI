use crate::app::App;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, BorderType, Paragraph, Wrap},
};

pub fn draw_index_cards(f: &mut Frame, app: &mut App, area: Rect) {
    let theme = &app.theme;
    let accent = Color::from(theme.ui.normal_mode_bg.clone());
    let normal_fg = theme.primary_fg();
    let selection_bg = Color::from(theme.ui.selection_bg.clone());
    let selection_fg = Color::from(theme.ui.selection_fg.clone());
    let bg_color = theme.primary_bg();
    
    let section_color = theme.syntax.section.clone()
        .map(Color::from)
        .unwrap_or(accent);

    let cards = &app.index_cards;
    if cards.is_empty() {
        return;
    }

    let card_rects = app.calculate_index_card_layout(area);

    // --- SCROLLING LOGIC ---
    let selected_rect = card_rects[app.selected_card_idx.min(card_rects.len()-1)];
    let view_height = area.height;
    
    // Auto-adjust scroll based on selection
    if selected_rect.y < app.card_row_offset as u16 {
        app.card_row_offset = selected_rect.y as usize;
    } else if selected_rect.y + selected_rect.height > (app.card_row_offset as u16 + view_height) {
        app.card_row_offset = (selected_rect.y + selected_rect.height - view_height) as usize;
    }

    // --- RENDERING ---
    for (i, (card, &raw_rect)) in cards.iter().zip(card_rects.iter()).enumerate() {
        let relative_y = raw_rect.y as i32 - app.card_row_offset as i32;
        
        // Skip cards that are completely above or completely below the viewport
        if relative_y + raw_rect.height as i32 <= 0 || relative_y >= view_height as i32 {
            continue;
        }

        // Calculate visible part of the card
        let draw_y = (area.y as i32 + relative_y).max(area.y as i32);
        let mut draw_h = raw_rect.height as i32;

        // Clip the height if it overflows the bottom
        if draw_y + draw_h > (area.y + area.height) as i32 {
            draw_h = (area.y + area.height) as i32 - draw_y;
        }
        
        // Skip if height became zero or negative due to clipping
        if draw_h <= 0 {
            continue;
        }

        let card_rect = Rect::new(
            area.x + raw_rect.x,
            draw_y as u16,
            raw_rect.width,
            draw_h as u16
        );

        let is_selected = i == app.selected_card_idx;
        let base_style = Style::default().bg(bg_color);
        
        if card.is_section {
            // --- RENDER SECTION BANNER ---
            let mut border_style = Style::default().fg(section_color);
            if is_selected {
                border_style = border_style.fg(accent).add_modifier(Modifier::BOLD);
            }
            
            let block = Block::default()
                .borders(Borders::ALL)
                .border_type(if is_selected { BorderType::Double } else { BorderType::Plain })
                .border_style(border_style)
                .style(base_style);
            
            f.render_widget(block, card_rect);

            // Label
            let tab_rect = Rect::new(card_rect.x + 1, card_rect.y, 12, 1);
            f.render_widget(Paragraph::new(Line::from(vec![
                Span::styled(" SECTION ", border_style),
            ])), tab_rect);

            // Heading & Synopsis
            let inner = Rect::new(card_rect.x + 2, card_rect.y + 1, card_rect.width.saturating_sub(4), card_rect.height.saturating_sub(2));
            let mut lines = Vec::new();
            
            let heading_text = if is_selected && app.is_card_editing && app.is_heading_editing {
                format!("{}|", app.card_input_buffer)
            } else {
                card.heading.to_uppercase()
            };
            
            lines.push(Line::from(Span::styled(heading_text, Style::default().fg(section_color).add_modifier(Modifier::BOLD))));
            
            let syn_text = if is_selected && app.is_card_editing && !app.is_heading_editing {
                format!("{}|", app.card_input_buffer)
            } else {
                if card.synopsis.is_empty() { "Empty section...".to_string() } else { card.synopsis.clone() }
            };
            
            lines.push(Line::from(Span::styled(syn_text, Style::default().fg(normal_fg).add_modifier(Modifier::ITALIC))));
            
            f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: true }), inner);

        } else {
            // --- RENDER SCENE CARD ---
            let mut border_style = theme.secondary_style();
            if is_selected {
                border_style = Style::default().fg(accent).add_modifier(Modifier::BOLD);
            }
            
            let block = Block::default()
                .borders(Borders::ALL)
                .border_type(if is_selected { BorderType::Thick } else { BorderType::Plain })
                .border_style(border_style)
                .style(base_style);
                
            f.render_widget(block, card_rect);
            
            // Header Bar
            let header_bar_rect = Rect::new(card_rect.x + 1, card_rect.y, card_rect.width.saturating_sub(2), 1.min(card_rect.height));
            let header_label = if let Some(ref num) = card.scene_num {
                format!(" SCENE {} ", num)
            } else {
                " SCENE ".to_string()
            };

            let label_style = if let Some(c) = card.color { 
                Style::default().fg(c).add_modifier(Modifier::BOLD)
            } else { 
                Style::default().fg(accent).add_modifier(Modifier::BOLD)
            };
            
            f.render_widget(Paragraph::new(Line::from(vec![
                Span::styled(header_label, label_style),
            ])), header_bar_rect);

            // Content Area
            let inner = Rect::new(card_rect.x + 1, card_rect.y + 1, card_rect.width.saturating_sub(2), card_rect.height.saturating_sub(2));
            let mut card_lines = Vec::new();
            
            // Heading
            let heading_content = if is_selected && app.is_card_editing && app.is_heading_editing {
                format!("{}|", app.card_input_buffer)
            } else {
                let h = card.heading.trim_start_matches('.').to_string();
                if h.is_empty() { "[No Heading]".to_string() } else { h }
            };
            
            let heading_style = if is_selected && app.is_card_editing && app.is_heading_editing {
                theme.warning_style().add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(normal_fg).add_modifier(Modifier::BOLD)
            };
            
            card_lines.push(Line::from(Span::styled(heading_content, if let Some(c) = card.color { heading_style.fg(c) } else { heading_style })));
            
            // Synopsis
            let syn_content = if is_selected && app.is_card_editing && !app.is_heading_editing {
                format!("{}|", app.card_input_buffer)
            } else if !card.synopsis.is_empty() {
                card.synopsis.clone()
            } else if !card.preview.is_empty() {
                card.preview.clone()
            } else {
                "Plan...".to_string()
            };
            
            let syn_style = if is_selected && app.is_card_editing && !app.is_heading_editing {
                Style::default().fg(selection_fg).bg(selection_bg)
            } else if !card.synopsis.is_empty() {
                Style::default().fg(normal_fg).add_modifier(Modifier::ITALIC)
            } else {
                theme.secondary_style().add_modifier(Modifier::ITALIC)
            };
            
            card_lines.push(Line::from(Span::styled(syn_content, syn_style)));
            
            f.render_widget(Paragraph::new(card_lines).wrap(Wrap { trim: true }), inner);
        }
    }

    // Help Hint
    let help_hint = Span::styled(" [?] Help ", Style::default().fg(accent).add_modifier(Modifier::BOLD));
    let hint_w = 10;
    let hint_area = Rect::new(
        area.x + area.width.saturating_sub(hint_w + 2),
        area.y + area.height.saturating_sub(1),
        hint_w,
        1
    );
    f.render_widget(Paragraph::new(Line::from(help_hint)), hint_area);
}
