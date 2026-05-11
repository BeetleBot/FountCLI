use crate::app::App;
use ratatui::{
    Frame,
    layout::{Rect, Alignment},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, BorderType, Clear, Paragraph},
};

fn hex_to_rgb(hex: &str) -> (u8, u8, u8) {
    let hex = hex.trim_start_matches('#');
    if hex.len() < 6 { return (128, 128, 128); }
    (
        u8::from_str_radix(&hex[0..2], 16).unwrap_or(128),
        u8::from_str_radix(&hex[2..4], 16).unwrap_or(128),
        u8::from_str_radix(&hex[4..6], 16).unwrap_or(128),
    )
}

fn gradient_color(stops: &[(u8, u8, u8)], t: f32) -> Color {
    if stops.len() < 2 { return Color::White; }
    let t = t.clamp(0.0, 1.0);
    let seg = stops.len() - 1;
    let scaled = t * seg as f32;
    let idx = (scaled as usize).min(seg - 1);
    let lt = scaled - idx as f32;
    let (a, b) = (stops[idx], stops[idx + 1]);
    Color::Rgb(
        (a.0 as f32 + (b.0 as f32 - a.0 as f32) * lt) as u8,
        (a.1 as f32 + (b.1 as f32 - a.1 as f32) * lt) as u8,
        (a.2 as f32 + (b.2 as f32 - a.2 as f32) * lt) as u8,
    )
}

pub fn draw_home(f: &mut Frame, app: &mut App) {
    let area = f.area();
    let theme = &app.theme;

    let stops = vec![
        hex_to_rgb(&theme.ui.normal_mode_bg.0),
        hex_to_rgb(&theme.ui.navigator_mode_bg.0),
        hex_to_rgb(&theme.ui.search_mode_bg.0),
    ];

    let accent = Color::from(theme.ui.normal_mode_bg.clone());
    let sel_bg = Color::from(theme.ui.selection_bg.clone());
    let sel_fg = Color::from(theme.ui.selection_fg.clone());
    let normal_fg = theme.primary_fg();
    let normal_bg = theme.primary_bg();

    let modal_w = 76u16.min(area.width.saturating_sub(2));
    let modal_h = 32u16.min(area.height.saturating_sub(2));
    let x = area.x + (area.width.saturating_sub(modal_w)) / 2;
    let y = area.y + (area.height.saturating_sub(modal_h)) / 2;
    let modal_area = Rect::new(x, y, modal_w, modal_h);

    f.render_widget(Clear, modal_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(accent))
        .style(Style::default().bg(normal_bg).fg(normal_fg))
        .title(Span::styled(
            " [ home ] ",
            Style::default().fg(accent).add_modifier(Modifier::BOLD),
        ));
    f.render_widget(block, modal_area);

    let inner = modal_area.inner(ratatui::layout::Margin {
        horizontal: 2,
        vertical: 1,
    });

    let mut lines = Vec::new();

    /* 
    Legacy Logos (Reference Only)

    Original:
    ███████╗ ██████╗ ██╗   ██╗███╗   ██╗████████╗
    ██╔════╝██╔═══██╗██║   ██║████╗  ██║╚══██╔══╝
    █████╗  ██║   ██║██║   ██║██╔██╗ ██║   ██║   
    ██╔══╝  ██║   ██║██║   ██║██║╚██╗██║   ██║   
    ██║     ╚██████╔╝╚██████╔╝██║ ╚████║   ██║   
    ╚═╝      ╚═════╝  ╚═════╝ ╚═╝  ╚═══╝   ╚═╝   

    Slashed:
         /  ███████╗ ██████╗ ██╗   ██╗███╗   ██╗████████╗
        /   ██╔════╝██╔═══██╗██║   ██║████╗  ██║╚══██╔══╝
       /    █████╗  ██║   ██║██║   ██║██╔██╗ ██║   ██║   
      /     ██╔══╝  ██║   ██║██║   ██║██║╚██╗██║   ██║   
     /      ██║     ╚██████╔╝╚██████╔╝██║ ╚████║   ██║   
    /       ╚═╝      ╚═════╝  ╚═════╝ ╚═╝  ╚═══╝   ╚═╝   

    Fountain Pen Nib:
                  ▄▄              
                 ╱██╲             
                ╱████╲            
               ╱██████╲           
                ╲██╱╲██╱            
                 ╲╱  ╲╱             
                  ╱    ╲              
                 ╱      ╲             
                  ╲    ╱              
                   ╲╱╱╱               
                    ▀                

    Gradient Blocks:
    ░▒▓████████▓▒░▒▓██████▓▒░ ░▒▓█▓▒░░▒▓█▓▒░▒▓███████▓▒▒▓████████▓▒░
    ░▒▓█▓▒░      ░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░ ░▒▓█▓▒░   
    ░▒▓█▓▒░      ░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░ ░▒▓█▓▒░   
    ░▒▓██████▓▒░ ░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░ ░▒▓█▓▒░   
    ░▒▓█▓▒░      ░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░ ░▒▓█▓▒░   
    ░▒▓█▓▒░      ░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░ ░▒▓█▓▒░   
    ░▒▓█▓▒░      ░▒▓██████▓▒░  ░▒▓██████▓▒░░▒▓█▓▒░░▒▓█▓▒░ ░▒▓█▓▒░   

    Slanted:
         __________   ______    __    __  .__   __. .___________.  
        /  /   ____| /  __  \  |  |  |  | |  \ |  | |           | 
       /  /|  |__   |  |  |  | |  |  |  | |   \|  | `---|  |----` 
      /  / |   __|  |  |  |  | |  |  |  | |  . `  |     |  |      
     /  /  |  |     |  `--'  | |  `--'  | |  |\   |     |  |      
    /__/   |__|      \______/   \______/  |__| \__|     |__|      
    */

    // ── Gradient /FOUNT logo ──
    let logo = [
        "      ░██     ░████                                     ░██    ",
        "     ░██     ░██                                        ░██    ",
        "    ░██   ░████████  ░███████  ░██    ░██ ░████████  ░████████ ",
        "   ░██       ░██    ░██    ░██ ░██    ░██ ░██    ░██    ░██    ",
        "  ░██        ░██    ░██    ░██ ░██    ░██ ░██    ░██    ░██    ",
        " ░██         ░██    ░██    ░██ ░██   ░███ ░██    ░██    ░██    ",
        "░██          ░██     ░███████   ░█████░██ ░██    ░██     ░████ ",
    ];

    let max_w = logo.iter().map(|r| r.chars().count()).max().unwrap_or(1);

    for row in &logo {
        let chars: Vec<char> = row.chars().collect();
        let w = chars.len();
        let mut spans = Vec::new();
        for (ci, ch) in chars.iter().enumerate() {
            let t = ci as f32 / max_w.max(1) as f32;
            if *ch == ' ' {
                spans.push(Span::raw(" "));
            } else {
                spans.push(Span::styled(
                    ch.to_string(),
                    Style::default().fg(gradient_color(&stops, t)),
                ));
            }
        }
        // pad to max_w so centering aligns all rows
        for _ in w..max_w {
            spans.push(Span::raw(" "));
        }
        lines.push(Line::from(spans));
    }

    lines.push(Line::from(""));

    // ── Tagline ──
    lines.push(Line::from(vec![
        Span::styled(
            format!("v{} — Blockbusters in Terminal", env!("CARGO_PKG_VERSION")),
            Style::default().fg(accent).add_modifier(Modifier::ITALIC),
        ),
    ]));

    lines.push(Line::from(""));

    let sep_w = inner.width.saturating_sub(6) as usize;
    lines.push(Line::from(Span::styled(
        "─".repeat(sep_w),
        Style::default().fg(accent),
    )));
    lines.push(Line::from(""));

    // ── Menu ──
    let menu = ["New File", "New file with Structure", "Open File", "Tutorial", "Exit"];

    for (i, label) in menu.iter().enumerate() {
        let is_sel = i == app.home_selected;
        if is_sel {
            lines.push(Line::from(Span::styled(
                format!("  ▸ {}  ", label),
                Style::default().fg(sel_fg).bg(sel_bg).add_modifier(Modifier::BOLD),
            )));
        } else {
            lines.push(Line::from(Span::styled(
                format!("    {}  ", label),
                Style::default().fg(normal_fg),
            )));
        }
        lines.push(Line::from(""));
    }

    // ── Recent files ──
    if !app.recent_files.is_empty() {
        lines.push(Line::from(Span::styled(
            "─".repeat(sep_w),
            Style::default().fg(accent),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "[ Recent ]",
            Style::default().fg(accent).add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));

        for (i, path) in app.recent_files.iter().take(4).enumerate() {
            let idx = menu.len() + i;
            let is_sel = idx == app.home_selected;
            let name = path.file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| "Unknown".to_string());

            if is_sel {
                lines.push(Line::from(Span::styled(
                    format!("  ▸ {}  ", name),
                    Style::default().fg(sel_fg).bg(sel_bg).add_modifier(Modifier::BOLD),
                )));
            } else {
                lines.push(Line::from(Span::styled(
                    format!("    {}  ", name),
                    Style::default().fg(normal_fg),
                )));
            }
        }
    }

    f.render_widget(Paragraph::new(lines).alignment(Alignment::Center), inner);
}
