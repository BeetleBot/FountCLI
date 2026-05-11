use std::path::PathBuf;
use ratatui::layout::Rect;
use crate::app::{App, AppMode, FilePickerAction};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::io;
use std::fs;

impl App {
    pub fn handle_panes(&mut self, key: KeyEvent, update_target_x: &mut bool, text_changed: &mut bool, cursor_moved: &mut bool) -> io::Result<bool> {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let shift = key.modifiers.contains(KeyModifiers::SHIFT);
        let alt = key.modifiers.contains(KeyModifiers::ALT);
        match self.mode {
                AppMode::Search => {
                    match key.code {
                        KeyCode::Esc => {
                            self.mode = AppMode::Normal;
                            self.set_status("Cancelled");
                            self.show_search_highlight = false;
                            self.search_query.clear();
                        }
                        KeyCode::Char('c') | KeyCode::Char('g') if ctrl => {
                            self.mode = AppMode::Normal;
                            self.set_status("Cancelled");
                            self.show_search_highlight = false;
                            self.search_query.clear();
                        }
                        KeyCode::Enter => {
                            self.execute_search();
                            *update_target_x = true;
                            *cursor_moved = true;
                        }
                        KeyCode::Up if alt => {
                            self.jump_to_match(false);
                            *cursor_moved = true;
                            *update_target_x = true;
                        }
                        KeyCode::Down if alt => {
                            self.jump_to_match(true);
                            *cursor_moved = true;
                            *update_target_x = true;
                        }
                        KeyCode::Backspace => {
                            self.search_query.pop();
                            self.update_search_regex();
                        }
                        KeyCode::Char(c) if !ctrl => {
                            self.search_query.push(c);
                            self.update_search_regex();
                        }
                        _ => {}
                    }
                    return Ok(false);
                }
                AppMode::PromptSave => {
                    match key.code {
                        KeyCode::Char('y') | KeyCode::Char('Y') if !ctrl => {
                            if self.file.is_some() && self.save().is_ok() {
                                if self.exit_after_save {
                                    self.close_current_buffer();
                                    return Ok(true);
                                }
                                self.mode = AppMode::Normal;
                                return Ok(false);
                            }
                            self.filename_input = self
                                .file
                                .as_ref()
                                .map(|p| p.to_string_lossy().into_owned())
                                .unwrap_or_default();
                            self.mode = AppMode::PromptFilename;
                        }
                        KeyCode::Char('n') | KeyCode::Char('N') if !ctrl => {
                            if self.exit_after_save {
                                self.close_current_buffer();
                                return Ok(true);
                            }
                            self.mode = AppMode::Normal;
                        }
                        KeyCode::Esc => {
                            self.mode = AppMode::Normal;
                            self.set_status("Cancelled");
                        }
                        KeyCode::Char('c') | KeyCode::Char('g') if ctrl => {
                            self.mode = AppMode::Normal;
                            self.set_status("Cancelled");
                        }
                        _ => {}
                    }
                    return Ok(false);
                }
                AppMode::PromptFilename => {
                    match key.code {
                        KeyCode::Esc => {
                            self.mode = AppMode::Normal;
                            self.set_status("Cancelled");
                        }
                        KeyCode::Char('c') | KeyCode::Char('g') if ctrl => {
                            self.mode = AppMode::Normal;
                            self.set_status("Cancelled");
                        }
                        KeyCode::Enter => {
                            if !self.filename_input.trim().is_empty() {
                                self.file = Some(PathBuf::from(self.filename_input.trim()));
                                match self.save() {
                                    Ok(_) => {
                                        if self.exit_after_save {
                                            self.close_current_buffer();
                                            return Ok(true);
                                        }
                                        self.mode = AppMode::Normal;
                                    }
                                    Err(e) => {
                                        self.set_status(&format!("Error saving: {}", e));
                                        self.mode = AppMode::Normal;
                                    }
                                }
                            } else {
                                self.set_status("Cancelled");
                                self.mode = AppMode::Normal;
                            }
                        }
                        KeyCode::Backspace => {
                            self.filename_input.pop();
                        }
                        KeyCode::Char(c) if !ctrl => {
                            self.filename_input.push(c);
                        }
                        _ => {}
                    }
                    return Ok(false);
                }
                AppMode::SettingsPane => {
                    let settings_count = 12;
                    match key.code {
                        KeyCode::Esc => {
                            self.mode = AppMode::Normal;
                        }
                        KeyCode::Up => {
                            self.selected_setting = if self.selected_setting == 0 {
                                settings_count - 1
                            } else {
                                self.selected_setting - 1
                            };
                        }
                        KeyCode::Down => {
                            self.selected_setting = (self.selected_setting + 1) % settings_count;
                        }
                        KeyCode::Enter | KeyCode::Char(' ') => {
                            match self.selected_setting {
                                0 => {
                                    self.config.focus_mode = !self.config.focus_mode;
                                    let _ = crate::config::Config::save_setting("focus_mode", self.config.focus_mode);
                                }
                                1 => {
                                    self.config.show_line_numbers = !self.config.show_line_numbers;
                                    let _ = crate::config::Config::save_setting("line_numbers", self.config.show_line_numbers);
                                }
                                2 => {
                                    self.config.typewriter_mode = !self.config.typewriter_mode;
                                    let _ = crate::config::Config::save_setting("typewriter_mode", self.config.typewriter_mode);
                                }
                                3 => {
                                    self.config.hide_markup = !self.config.hide_markup;
                                    let _ = crate::config::Config::save_setting("hide_markup", self.config.hide_markup);
                                }
                                4 => {
                                    self.config.highlight_active_action = !self.config.highlight_active_action;
                                    let _ = crate::config::Config::save_setting("highlight_active_action", self.config.highlight_active_action);
                                }
                                5 => {
                                    self.config.show_page_numbers = !self.config.show_page_numbers;
                                    let _ = crate::config::Config::save_setting("show_page_numbers", self.config.show_page_numbers);
                                }
                                6 => {
                                    self.config.show_scene_numbers = !self.config.show_scene_numbers;
                                    let _ = crate::config::Config::save_setting("show_scene_numbers", self.config.show_scene_numbers);
                                }
                                7 => {
                                    self.config.auto_contd = !self.config.auto_contd;
                                    let _ = crate::config::Config::save_setting("auto_contd", self.config.auto_contd);
                                }
                                8 => {
                                    if !self.config.auto_save {
                                        self.config.auto_save = true;
                                        self.config.auto_save_interval = 60;
                                    } else {
                                        match self.config.auto_save_interval {
                                            60 => self.config.auto_save_interval = 180,
                                            180 => self.config.auto_save_interval = 300,
                                            300 => self.config.auto_save_interval = 600,
                                            600 => {
                                                self.config.auto_save = false;
                                                self.config.auto_save_interval = 300;
                                            }
                                            _ => self.config.auto_save_interval = 60,
                                        }
                                    }
                                    let _ = crate::config::Config::save_setting("auto_save", self.config.auto_save);
                                    let _ = crate::config::Config::save_string_setting("auto_save_interval", &self.config.auto_save_interval.to_string());
                                }
                                9 => {
                                    self.config.autocomplete = !self.config.autocomplete;
                                    let _ = crate::config::Config::save_setting("autocomplete", self.config.autocomplete);
                                }
                                10 => {
                                    self.config.auto_paragraph_breaks = !self.config.auto_paragraph_breaks;
                                    let _ = crate::config::Config::save_setting("auto_paragraph_breaks", self.config.auto_paragraph_breaks);
                                }
                                11 => {
                                    self.config.use_nerd_fonts = !self.config.use_nerd_fonts;
                                    let _ = crate::config::Config::save_setting("use_nerd_fonts", self.config.use_nerd_fonts);
                                }
                                _ => {}
                            }
                            *text_changed = true;

                            self.update_layout();
                        }
                        _ => {}
                    }
                    return Ok(false);
                }
                AppMode::Shortcuts => {
                    let total_cats = crate::app::shortcuts::get_categories(
                        &crate::app::shortcuts::get_all_shortcuts(),
                    ).len();

                    if self.is_shortcuts_searching {
                        match key.code {
                            KeyCode::Esc | KeyCode::Enter => {
                                self.is_shortcuts_searching = false;
                            }
                            KeyCode::Backspace => {
                                self.shortcuts_query.pop();
                                self.shortcuts_state.select(Some(0));
                            }
                            KeyCode::Char(c) if !ctrl => {
                                self.shortcuts_query.push(c);
                                self.shortcuts_state.select(Some(0));
                            }
                            _ => {}
                        }
                    } else {
                        match key.code {
                            KeyCode::Esc | KeyCode::F(1) => {
                                self.mode = AppMode::Normal;
                                self.shortcuts_query.clear();
                                self.is_shortcuts_searching = false;
                                self.shortcuts_selected_tab = 0;
                            }
                            KeyCode::Char('/') => {
                                self.is_shortcuts_searching = true;
                                self.shortcuts_query.clear();
                            }
                            KeyCode::Char('h') if ctrl => {
                                self.open_scene_navigator();
                            }
                            KeyCode::Char('p') if ctrl => {
                                self.mode = AppMode::SettingsPane;
                                self.selected_setting = 0;
                            }
                            KeyCode::Char('f') if ctrl => {}
                            KeyCode::Tab | KeyCode::Right | KeyCode::Char('l') => {
                                if total_cats > 0 {
                                    self.shortcuts_selected_tab = (self.shortcuts_selected_tab + 1) % total_cats;
                                    self.shortcuts_state.select(Some(0));
                                }
                            }
                            KeyCode::BackTab | KeyCode::Left | KeyCode::Char('h') => {
                                if total_cats > 0 {
                                    self.shortcuts_selected_tab = if self.shortcuts_selected_tab == 0 {
                                        total_cats - 1
                                    } else {
                                        self.shortcuts_selected_tab - 1
                                    };
                                    self.shortcuts_state.select(Some(0));
                                }
                            }
                            KeyCode::Up | KeyCode::Char('k') => {
                                let i = self.shortcuts_state.selected().unwrap_or(0);
                                self.shortcuts_state.select(Some(i.saturating_sub(1)));
                            }
                            KeyCode::Down | KeyCode::Char('j') => {
                                let i = self.shortcuts_state.selected().unwrap_or(0);
                                self.shortcuts_state.select(Some(i.saturating_add(1)));
                            }
                            KeyCode::PageUp => {
                                let i = self.shortcuts_state.selected().unwrap_or(0);
                                self.shortcuts_state.select(Some(i.saturating_sub(10)));
                            }
                            KeyCode::PageDown => {
                                let i = self.shortcuts_state.selected().unwrap_or(0);
                                self.shortcuts_state.select(Some(i.saturating_add(10)));
                            }
                            KeyCode::Home => {
                                self.shortcuts_state.select(Some(0));
                            }
                            _ => {}
                        }
                    }
                    return Ok(false);
                }
                AppMode::ExportPane => {
                    let screenplay_options_count = 9;
                    let reports_options_count = 2;
                    let current_options_count = if self.export_tab == 0 { screenplay_options_count } else { reports_options_count };

                    match key.code {
                        KeyCode::Esc => {
                            self.mode = AppMode::Normal;
                        }
                        KeyCode::Char('c') | KeyCode::Char('e') | KeyCode::Char('g') if ctrl => {
                            self.mode = AppMode::Normal;
                        }
                        KeyCode::Left | KeyCode::Char('h') => {
                            if self.export_tab > 0 {
                                self.export_tab -= 1;
                                self.selected_export_option = 0;
                            }
                        }
                        KeyCode::Right | KeyCode::Char('l') => {
                            if self.export_tab < 1 {
                                self.export_tab += 1;
                                self.selected_export_option = 0;
                            }
                        }
                        KeyCode::Char('1') => {
                            self.export_tab = 0;
                            self.selected_export_option = 0;
                        }
                        KeyCode::Char('2') => {
                            self.export_tab = 1;
                            self.selected_export_option = 0;
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            self.selected_export_option = if self.selected_export_option == 0 {
                                current_options_count - 1
                            } else {
                                self.selected_export_option - 1
                            };
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            self.selected_export_option = (self.selected_export_option + 1) % current_options_count;
                        }
                        KeyCode::Enter | KeyCode::Char(' ') | KeyCode::Tab => {
                            if self.export_tab == 0 {
                                // Screenplay Options
                                match self.selected_export_option {
                                    0 => {
                                        let formats = ["pdf", "fountain"];
                                        if let Some(idx) = formats.iter().position(|&x| x == self.config.export_format.as_str()) {
                                            self.config.export_format = formats[(idx + 1) % formats.len()].to_string();
                                        } else {
                                            self.config.export_format = "pdf".to_string();
                                        }
                                        let _ = crate::config::Config::save_string_setting("export_format", &self.config.export_format);
                                    }
                                    1 => {
                                        if self.config.paper_size == "a4" {
                                            self.config.paper_size = "letter".to_string();
                                        } else {
                                            self.config.paper_size = "a4".to_string();
                                        }
                                        let _ = crate::config::Config::save_string_setting("paper_size", &self.config.paper_size);
                                    }
                                    2 => {
                                        if self.config.export_font == "courier_prime" {
                                            self.config.export_font = "courier_prime_sans".to_string();
                                        } else {
                                            self.config.export_font = "courier_prime".to_string();
                                        }
                                        let _ = crate::config::Config::save_string_setting("export_font", &self.config.export_font);
                                    }
                                    3 => {
                                        self.config.export_bold_scene_headings = !self.config.export_bold_scene_headings;
                                        let _ = crate::config::Config::save_setting("export_bold_scene_headings", self.config.export_bold_scene_headings);
                                    }
                                    4 => {
                                        if self.config.mirror_scene_numbers == crate::config::MirrorOption::Off {
                                            self.config.mirror_scene_numbers = crate::config::MirrorOption::ExportOnly;
                                            let _ = crate::config::Config::save_string_setting("mirror_scene_numbers", "export");
                                        } else {
                                            self.config.mirror_scene_numbers = crate::config::MirrorOption::Off;
                                            let _ = crate::config::Config::save_string_setting("mirror_scene_numbers", "off");
                                        }
                                    }
                                    5 => {
                                        self.config.export_sections = !self.config.export_sections;
                                        let _ = crate::config::Config::save_setting("export_sections", self.config.export_sections);
                                    }
                                    6 => {
                                        self.config.export_synopses = !self.config.export_synopses;
                                        let _ = crate::config::Config::save_setting("export_synopses", self.config.export_synopses);
                                    }
                                    7 => {
                                        self.config.include_title_page = !self.config.include_title_page;
                                        let _ = crate::config::Config::save_setting("include_title_page", self.config.include_title_page);
                                    }
                                    8 => {
                                        let (ext, default_name) = match self.config.export_format.as_str() {
                                            "pdf" => ("pdf", "screenplay.pdf"),
                                            "fountain" => ("fountain", "screenplay.fountain"),
                                            _ => ("pdf", "screenplay.pdf"),
                                        };
                                        self.open_file_picker(FilePickerAction::ExportScript, vec![ext.to_string()], Some(default_name.to_string()));
                                    }
                                    _ => {}
                                }
                            } else {
                                // Reports Options
                                match self.selected_export_option {
                                    0 => {
                                        let formats = ["csv_scene", "csv_char", "csv_location", "csv_notes", "csv_breakdown", "txt_dialogue"];
                                        if let Some(idx) = formats.iter().position(|&x| x == self.config.report_format.as_str()) {
                                            self.config.report_format = formats[(idx + 1) % formats.len()].to_string();
                                        } else {
                                            self.config.report_format = "csv_scene".to_string();
                                        }
                                        let _ = crate::config::Config::save_string_setting("report_format", &self.config.report_format);
                                    }
                                    1 => {
                                        let (ext, default_name) = match self.config.report_format.as_str() {
                                            "csv_scene" => ("csv", "scene_list.csv"),
                                            "csv_char" => ("csv", "character_report.csv"),
                                            "csv_location" => ("csv", "location_report.csv"),
                                            "csv_notes" => ("csv", "notes_report.csv"),
                                            "csv_breakdown" => ("csv", "script_breakdown.csv"),
                                            "txt_dialogue" => ("txt", "dialogue_only.txt"),
                                            _ => ("csv", "report.csv"),
                                        };
                                        self.open_file_picker(FilePickerAction::ExportReport, vec![ext.to_string()], Some(default_name.to_string()));
                                    }
                                    _ => {}
                                }
                            }
                        }
                        _ => {}
                    }
                    return Ok(false);
                }
                AppMode::Home => {
                    let home_items = 5 + self.recent_files.len().min(5);
                    match key.code {
                        KeyCode::Esc => {
                            // If there's an actual file loaded, dismiss home
                            if self.file.is_some() || !self.lines.iter().all(|l| l.is_empty()) {
                                self.mode = AppMode::Normal;
                            }
                        }
                        KeyCode::Char('c') | KeyCode::Char('g') if ctrl => {
                            // Ctrl+C/G always dismisses
                            self.mode = AppMode::Normal;
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            if self.home_selected > 0 {
                                self.home_selected -= 1;
                            } else {
                                self.home_selected = home_items.saturating_sub(1);
                            }
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            self.home_selected = (self.home_selected + 1) % home_items;
                        }
                        KeyCode::Enter | KeyCode::Char(' ') | KeyCode::Char('\n') |
                        KeyCode::Char('n') | KeyCode::Char('N') |
                        KeyCode::Char('o') | KeyCode::Char('O') |
                        KeyCode::Char('t') | KeyCode::Char('T') |
                        KeyCode::Char('q') | KeyCode::Char('Q') => {
                            match key.code {
                                KeyCode::Char('n') | KeyCode::Char('N') => self.home_selected = 0,
                                KeyCode::Char('s') | KeyCode::Char('S') => self.home_selected = 1,
                                KeyCode::Char('o') | KeyCode::Char('O') => self.home_selected = 2,
                                KeyCode::Char('t') | KeyCode::Char('T') => self.home_selected = 3,
                                KeyCode::Char('q') | KeyCode::Char('Q') => self.home_selected = 4,
                                _ => {},
                            }
                            match self.home_selected {
                                0 => {
                                    // New File
                                    let lines = vec![String::new()];
                                    let revised_lines = vec![false; lines.len()];
                                    let new_buf = crate::app::BufferState {
                                        lines,
                                        revised_lines,
                                        ..Default::default()
                                    };
                                    self.buffers.push(new_buf);
                                    let new_idx = self.buffers.len() - 1;
                                    self.has_multiple_buffers = self.buffers.len() > 1;
                                    self.switch_buffer(new_idx);
                                    self.mode = AppMode::Normal;
                                    self.set_status("New buffer");
                                    *text_changed = true;
                                    *cursor_moved = true;
                                }
                                1 => {
                                    // New file with Structure
                                    if self.structures.is_empty() {
                                        self.load_structures();
                                    }
                                    if self.structures.is_empty() {
                                        self.set_status("No structures found in Structures.md");
                                    } else {
                                        self.mode = AppMode::StructurePicker;
                                        self.structure_selected = 0;
                                    }
                                }
                                2 => {
                                    // Open File via TUI picker
                                    self.open_file_picker(FilePickerAction::Open, vec!["fountain".to_string()], None);
                                }
                                3 => {
                                    // Tutorial
                                    let tutorial_text = include_str!("../../../assets/tutorial.fountain");
                                    let lines: Vec<String> = tutorial_text.lines().map(|s: &str| s.to_string()).collect();
                                    let revised_lines = vec![false; lines.len()];
                                    let new_buf = crate::app::BufferState {
                                        lines,
                                        file: None,
                                        is_tutorial: true,
                                        revised_lines,
                                        ..Default::default()
                                    };
                                    self.buffers.push(new_buf);
                                    let new_idx = self.buffers.len() - 1;
                                    self.has_multiple_buffers = self.buffers.len() > 1;
                                    self.switch_buffer(new_idx);
                                    self.parse_document();
                                    self.update_autocomplete();
                                    self.update_layout();
                                    self.mode = AppMode::Normal;
                                    self.set_status("Tutorial loaded! Enjoy the show.");
                                    *text_changed = true;
                                    *cursor_moved = true;
                                }
                                4 => {
                                    // Exit App
                                    return Ok(true);
                                }
                                _ => {
                                    // Recent Files
                                    let recent_idx = self.home_selected - 5;
                                    if recent_idx < self.recent_files.len() {
                                        let path = self.recent_files[recent_idx].clone();
                                        if let Ok(content) = fs::read_to_string(&path) {
                                            let lines: Vec<String> = content.replace('\t', "    ")
                                                .lines()
                                                .map(|s| s.to_string())
                                                .collect();
                                            let lines = if lines.is_empty() { vec![String::new()] } else { lines };
                                            let revised_lines = vec![false; lines.len()];
                                            let new_buf = crate::app::BufferState {
                                                lines,
                                                file: Some(path.clone()),
                                                revised_lines,
                                                ..Default::default()
                                            };
                                            self.buffers.push(new_buf);
                                            let new_idx = self.buffers.len() - 1;
                                            self.has_multiple_buffers = self.buffers.len() > 1;
                                            self.switch_buffer(new_idx);
                                            self.add_recent_file(path.clone());
                                            self.mode = AppMode::Normal;
                                            self.parse_document();
                                            self.update_autocomplete();
                                            self.update_layout();
                                            let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
                                            self.set_status(&format!("Opened: {}", name));
                                            *text_changed = true;
                                            *cursor_moved = true;
                                        } else {
                                            self.set_status("Error opening recent file");
                                            self.recent_files.remove(recent_idx);
                                            self.save_recent_files();
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                    return Ok(false);
                }
                AppMode::StructurePicker => {
                    match key.code {
                        KeyCode::Esc => {
                            self.mode = AppMode::Home;
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            if self.structure_selected > 0 {
                                self.structure_selected -= 1;
                            } else {
                                self.structure_selected = self.structures.len().saturating_sub(1);
                            }
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            if !self.structures.is_empty() {
                                self.structure_selected = (self.structure_selected + 1) % self.structures.len();
                            }
                        }
                        KeyCode::Enter => {
                            self.apply_selected_structure();
                            self.parse_document();
                            self.update_autocomplete();
                            self.update_layout();
                            *text_changed = true;
                            *cursor_moved = true;
                        }
                        _ => {}
                    }
                    return Ok(false);
                }
                AppMode::FilePicker => {
                    if let Some(ref mut state) = self.file_picker {
                        if state.show_overwrite_confirm {
                            match key.code {
                                KeyCode::Left | KeyCode::Right | KeyCode::Char('h') | KeyCode::Char('l') => {
                                    state.overwrite_confirmed = !state.overwrite_confirmed;
                                }
                                KeyCode::Enter => {
                                    if state.overwrite_confirmed {
                                        state.show_overwrite_confirm = false;
                                        if let Some(path) = state.target_path.clone() {
                                            if let Err(e) = self.handle_file_picker_choice(path) {
                                                self.set_error(&format!("Error: {}", e));
                                            }
                                        }
                                    } else {
                                        state.show_overwrite_confirm = false;
                                    }
                                }
                                KeyCode::Char('y') | KeyCode::Char('Y') => {
                                    state.show_overwrite_confirm = false;
                                    if let Some(path) = state.target_path.clone() {
                                        if let Err(e) = self.handle_file_picker_choice(path) {
                                            self.set_error(&format!("Error: {}", e));
                                        }
                                    }
                                }
                                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                                    state.show_overwrite_confirm = false;
                                }
                                _ => {}
                            }
                            return Ok(false);
                        }
                    }

                    match key.code {
                        KeyCode::Esc => {
                            self.mode = AppMode::Normal;
                            self.file_picker = None;
                        }
                        KeyCode::Tab => {
                            if let Some(ref mut state) = self.file_picker {
                                state.filename_input.clear();
                                state.naming_mode = true;
                            }
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            if let Some(ref mut state) = self.file_picker {
                                let current = state.list_state.selected().unwrap_or(0);
                                if current > 0 {
                                    state.list_state.select(Some(current - 1));
                                }
                            }
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            if let Some(ref mut state) = self.file_picker {
                                let current = state.list_state.selected().unwrap_or(0);
                                let max = state.items.len() + (if state.action != FilePickerAction::Open && !state.filename_input.is_empty() { 1 } else { 0 });
                                if current + 1 < max {
                                    state.list_state.select(Some(current + 1));
                                }
                            }
                        }
                        KeyCode::Enter => {
                            match self.file_picker_enter() {
                                Ok(true) => return Ok(true),
                                Ok(false) => {}
                                Err(e) => self.set_error(&format!("Error: {}", e)),
                            }
                        }
                        KeyCode::Backspace => {
                            if let Some(ref mut state) = self.file_picker {
                                if state.action != FilePickerAction::Open {
                                    state.filename_input.pop();
                                } else {
                                    // Navigate up directory
                                    if let Some(parent) = state.current_dir.parent().map(|p| p.to_path_buf()) {
                                        state.current_dir = parent;
                                        state.items = crate::app::file_picker::get_dir_items(&state.current_dir);
                                        state.list_state.select(Some(0));
                                    }
                                }
                            }
                        }
                        KeyCode::Char(c) => {
                            if let Some(ref mut state) = self.file_picker
                                && state.action != FilePickerAction::Open {
                                    state.filename_input.push(c);
                                }
                        }
                        _ => {}
                    }
                    return Ok(false);
                }
                AppMode::Snapshots => {
                    match key.code {
                        KeyCode::Esc => {
                            self.mode = AppMode::Normal;
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            let current = self.snapshot_list_state.selected().unwrap_or(0);
                            if current > 0 {
                                self.snapshot_list_state.select(Some(current - 1));
                            }
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            let current = self.snapshot_list_state.selected().unwrap_or(0);
                            if current + 1 < self.snapshots.len() {
                                self.snapshot_list_state.select(Some(current + 1));
                            }
                        }
                        KeyCode::Enter | KeyCode::Char('r') => {
                            let selected = self.snapshot_list_state.selected().unwrap_or(0);
                            if let Err(e) = self.restore_snapshot(selected, false) {
                                self.set_error(&format!("Restore failed: {}", e));
                            }
                        }
                        KeyCode::Char('o') => {
                            let selected = self.snapshot_list_state.selected().unwrap_or(0);
                            if let Err(e) = self.restore_snapshot(selected, true) {
                                self.set_error(&format!("Restore failed: {}", e));
                            }
                        }
                        _ => {}
                    }
                    return Ok(false);
                }
                AppMode::SprintStat => {
                    match key.code {
                        KeyCode::Esc | KeyCode::Char('q') => self.mode = AppMode::Normal,
                        KeyCode::Up | KeyCode::Char('k') => {
                            let current = self.sprint_stats_state.selected().unwrap_or(0);
                            if current > 0 {
                                self.sprint_stats_state.select(Some(current - 1));
                            }
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            let current = self.sprint_stats_state.selected().unwrap_or(0);
                            if current + 1 < self.sprint_history.len() {
                                self.sprint_stats_state.select(Some(current + 1));
                            }
                        }
                        KeyCode::Char('e') => self.export_sprint_data(),
                        _ => {}
                    }
                    return Ok(false);
                }
                AppMode::XRay => {
                    match key.code {
                        KeyCode::Esc | KeyCode::Char('q') => {
                            self.mode = AppMode::Normal;
                            self.xray_data = None;
                        }
                        KeyCode::Left | KeyCode::Char('h') => {
                            if self.xray_tab > 0 {
                                self.xray_tab -= 1;
                                self.xray_scroll = 0;
                            }
                        }
                        KeyCode::Right | KeyCode::Char('l') => {
                            if self.xray_tab < 2 {
                                self.xray_tab += 1;
                                self.xray_scroll = 0;
                            }
                        }
                        KeyCode::Char('1') => { self.xray_tab = 0; self.xray_scroll = 0; }
                        KeyCode::Char('2') => { self.xray_tab = 1; self.xray_scroll = 0; }
                        KeyCode::Char('3') => { self.xray_tab = 2; self.xray_scroll = 0; }
                        KeyCode::Up | KeyCode::Char('k') => {
                            self.xray_scroll = self.xray_scroll.saturating_sub(1);
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            self.xray_scroll += 1;
                        }
                        KeyCode::PageUp => {
                            self.xray_scroll = self.xray_scroll.saturating_sub(10);
                        }
                        KeyCode::PageDown => {
                            self.xray_scroll += 10;
                        }
                        _ => {}
                    }
                    return Ok(false);
                }
                AppMode::IndexCards => {
                    if self.show_quick_help {
                        match key.code {
                            KeyCode::Esc | KeyCode::Char('?') | KeyCode::Enter | KeyCode::Char('q') => {
                                self.show_quick_help = false;
                            }
                            _ => {}
                        }
                        return Ok(false);
                    }

                    let cards = self.index_cards.clone();
                    let cards_count = cards.len();
                    let _columns = 3;

                    if self.is_card_editing {
                        match key.code {
                            KeyCode::Esc => {
                                self.is_card_editing = false;
                                self.is_heading_editing = false;
                                self.card_input_buffer.clear();
                            }
                            KeyCode::Enter => {
                                let idx = self.selected_card_idx;
                                let mut h = String::new();
                                let mut s = String::new();
                                
                                if let Some(card) = cards.get(idx) {
                                    h = card.heading.clone();
                                    s = card.synopsis.clone();
                                }

                                if self.is_heading_editing {
                                    self.update_card_content(idx, self.card_input_buffer.clone(), s);
                                    self.is_heading_editing = false;
                                    let updated_cards = &self.index_cards;
                                    self.card_input_buffer = updated_cards.get(idx).map(|c| c.synopsis.clone()).unwrap_or_default();
                                    self.set_status("Editing Synopsis... [Enter] to finish");
                                } else {
                                    self.update_card_content(idx, h, self.card_input_buffer.clone());
                                    self.is_card_editing = false;
                                    self.card_input_buffer.clear();
                                    self.set_status("Card updated");
                                }
                                *text_changed = true;
                            }
                            KeyCode::Backspace => {
                                self.card_input_buffer.pop();
                            }
                            KeyCode::Char(c) if !ctrl => {
                                self.card_input_buffer.push(c);
                                *text_changed = true;
                            }
                            _ => {}
                        }
                    } else {
                        match key.code {
                            KeyCode::Esc | KeyCode::Char('q') => {
                                self.mode = AppMode::Normal;
                                if let Some(card) = cards.get(self.selected_card_idx) {
                                    self.cursor_y = card.start_line;
                                    self.cursor_x = 0;
                                    *cursor_moved = true;
                                    *update_target_x = true;
                                }
                            }
                            KeyCode::Up => {
                                let shift = key.modifiers.contains(KeyModifiers::SHIFT);
                                if shift {
                                    if self.selected_card_idx > 0 {
                                        self.swap_cards(self.selected_card_idx, self.selected_card_idx - 1);
                                        self.selected_card_idx -= 1;
                                        *text_changed = true;
                                    }
                                } else {
                                    // Grid-aware Up: find card above
                                    let area = Rect::new(0, 0, 100, 100); // Dummy area for relative layout
                                    let rects = self.calculate_index_card_layout(area);
                                    if let Some(cur_rect) = rects.get(self.selected_card_idx) {
                                        let mut best_idx = self.selected_card_idx.saturating_sub(1);
                                        let mut min_dist = f32::MAX;
                                        
                                        for (i, rect) in rects.iter().enumerate().take(self.selected_card_idx) {
                                            if rect.y < cur_rect.y {
                                                let dx = (rect.x as i32 - cur_rect.x as i32).abs();
                                                let dy = (rect.y as i32 - cur_rect.y as i32).abs();
                                                let dist = (dx as f32) + (dy as f32 * 2.0); // Prioritize vertical alignment
                                                if dist < min_dist {
                                                    min_dist = dist;
                                                    best_idx = i;
                                                }
                                            }
                                        }
                                        self.selected_card_idx = best_idx;
                                    } else {
                                        self.selected_card_idx = self.selected_card_idx.saturating_sub(1);
                                    }
                                }
                            }
                            KeyCode::Down => {
                                let shift = key.modifiers.contains(KeyModifiers::SHIFT);
                                if shift {
                                    if self.selected_card_idx + 1 < cards_count {
                                        self.swap_cards(self.selected_card_idx, self.selected_card_idx + 1);
                                        self.selected_card_idx += 1;
                                        *text_changed = true;
                                    }
                                } else {
                                    // Grid-aware Down: find card below
                                    let area = Rect::new(0, 0, 100, 100);
                                    let rects = self.calculate_index_card_layout(area);
                                    if let Some(cur_rect) = rects.get(self.selected_card_idx) {
                                        let mut best_idx = (self.selected_card_idx + 1).min(cards_count.saturating_sub(1));
                                        let mut min_dist = f32::MAX;
                                        
                                        for (i, rect) in rects.iter().enumerate().skip(self.selected_card_idx + 1) {
                                            if rect.y > cur_rect.y {
                                                let dx = (rect.x as i32 - cur_rect.x as i32).abs();
                                                let dy = (rect.y as i32 - cur_rect.y as i32).abs();
                                                let dist = (dx as f32) + (dy as f32 * 2.0);
                                                if dist < min_dist {
                                                    min_dist = dist;
                                                    best_idx = i;
                                                }
                                            }
                                        }
                                        self.selected_card_idx = best_idx;
                                    } else {
                                        self.selected_card_idx = (self.selected_card_idx + 1).min(cards_count.saturating_sub(1));
                                    }
                                }
                            }
                            KeyCode::Left => {
                                self.selected_card_idx = self.selected_card_idx.saturating_sub(1);
                            }
                            KeyCode::Right => {
                                if self.selected_card_idx + 1 < cards_count {
                                    self.selected_card_idx += 1;
                                }
                            }
                            KeyCode::Enter => {
                                if let Some(card) = cards.get(self.selected_card_idx) {
                                    self.is_card_editing = true;
                                    self.is_heading_editing = true;
                                    self.card_input_buffer = card.heading.clone();
                                    self.set_status("Editing Title... [Enter] to next");
                                }
                            }
                            KeyCode::Char('?') => {
                                self.show_quick_help = true;
                            }
                            KeyCode::Char('n') | KeyCode::Char('N') => {
                                let is_section = shift || key.code == KeyCode::Char('N');
                                self.add_card(self.selected_card_idx, is_section);
                                *text_changed = true;
                                *cursor_moved = true;
                            }
                            KeyCode::Char('/') => {
                                self.previous_mode = self.mode;
                                self.mode = AppMode::Command;
                                self.command_input.clear();
                                self.command_error = false;
                            }
                            KeyCode::Char('z') if ctrl && shift => {
                                if self.redo() {
                                    self.set_status("Redo applied");
                                    *text_changed = true;
                                }
                            }
                            KeyCode::Char('z') if ctrl => {
                                if self.undo() {
                                    self.set_status("Undo applied");
                                    *text_changed = true;
                                }
                            }
                            KeyCode::Delete | KeyCode::Backspace => {
                                self.delete_card(self.selected_card_idx);
                                *text_changed = true;
                            }
                            _ => {}
                        }
                    }
                    return Ok(false);
                }
                AppMode::ThemePicker => {
                    let themes = self.theme_manager.list_themes();
                    match key.code {
                        KeyCode::Esc => {
                            self.mode = AppMode::Normal;
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            let current = self.theme_picker_state.selected().unwrap_or(0);
                            let new_idx = if current == 0 {
                                themes.len().saturating_sub(1)
                            } else {
                                current - 1
                            };
                            self.theme_picker_state.select(Some(new_idx));
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            let current = self.theme_picker_state.selected().unwrap_or(0);
                            let new_idx = if current >= themes.len().saturating_sub(1) {
                                0
                            } else {
                                current + 1
                            };
                            self.theme_picker_state.select(Some(new_idx));
                        }
                        KeyCode::Enter => {
                            if let Some(idx) = self.theme_picker_state.selected() {
                                if idx < themes.len() {
                                    let name = themes[idx].clone();
                                    if self.theme_manager.set_theme(&name) {
                                        self.theme = self.theme_manager.current_theme.clone();
                                        self.config.theme = self.theme.name.clone();
                                        let _ = crate::config::Config::save_string_setting("theme", &self.theme.name);
                                        self.set_status(&format!("Theme set to {}", self.theme.name));
                                        self.update_layout();
                                    }
                                }
                            }
                            self.mode = AppMode::Normal;
                        }
                        _ => {}
                    }
                    return Ok(false);
                }
            _ => {}
        }
        Ok(false)
    }
}
