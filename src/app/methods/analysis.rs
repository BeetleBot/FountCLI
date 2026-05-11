use crate::formatting::StringCaseExt;
use crate::app::{App, AppMode, EnsembleItem, CharacterItem, NavigatorItem};
use crate::layout::{strip_sigils, find_visual_cursor};
use crate::types::LineType;
use crate::parser::Parser;

impl App {
    pub fn total_word_count(&self) -> usize {
        self.lines
            .iter()
            .map(|l| l.split_whitespace().count())
            .sum()
    }

    pub fn total_page_count(&self) -> usize {
        self.layout
            .iter()
            .filter_map(|r| r.page_num)
            .next_back()
            .unwrap_or(1)
    }

    pub fn get_current_scene_name(&self) -> String {
        if self.layout.is_empty() {
            return "Untitled Scene".to_string();
        }
        let (vis_row_idx, _) = find_visual_cursor(&self.layout, self.cursor_y, self.cursor_x);
        for i in (0..=vis_row_idx.min(self.layout.len() - 1)).rev() {
            if self.layout[i].line_type == LineType::SceneHeading {
                let row = &self.layout[i];
                let mut label = strip_sigils(&row.raw_text, row.line_type).to_uppercase();

                // Strip scene numbering from label if it was part of raw text
                if let Some(idx) = label.rfind(" #") {
                    label.truncate(idx);
                }

                let label = label.trim();
                if let Some(num) = &row.scene_num {
                    return format!("{}. {}", num, label);
                } else {
                    return label.to_string();
                }
            }
        }
        "Untitled Scene".to_string()
    }

    pub fn current_page_number(&self) -> usize {
        let (vis_row_idx, _) = find_visual_cursor(&self.layout, self.cursor_y, self.cursor_x);
        for i in (0..=vis_row_idx).rev() {
            if let Some(p) = self.layout[i].page_num {
                return p;
            }
        }
        1
    }

    pub fn open_scene_navigator(&mut self) {
        self.nav_original_pos = Some((self.cursor_y, self.cursor_x));
        self.scenes.clear();
        let mut current_scene: Option<NavigatorItem> = None;

        for row in &self.layout {
            // Pick up explicit scene colors from notes
            if row.line_type == LineType::Note {
                if let Some(start) = row.raw_text.find("[[") {
                    if let Some(end) = row.raw_text[start..].find("]]") {
                        let content = &row.raw_text[start + 2..start + end];
                        if content.to_lowercase().starts_with("sceneclr:") {
                            if let Some(ref mut s) = current_scene {
                                s.color = row.override_color;
                            }
                        }
                    }
                }
            }

            if row.line_type == LineType::Section {
                if let Some(s) = current_scene.take() {
                    self.scenes.push(s);
                }
                let label = strip_sigils(&row.raw_text, row.line_type)
                    .trim()
                    .to_string();
                current_scene = Some(NavigatorItem {
                    line_idx: row.line_idx,
                    label,
                    is_section: true,
                    synopses: Vec::new(),
                    color: row.override_color,
                    ..Default::default()
                });
            } else if row.line_type == LineType::SceneHeading {
                if let Some(s) = current_scene.take() {
                    self.scenes.push(s);
                }
                let mut raw_heading = strip_sigils(&row.raw_text, row.line_type).to_string();
                while let Some(start) = raw_heading.find("[[") {
                    if let Some(end_offset) = raw_heading[start..].find("]]") {
                        raw_heading.replace_range(start..start + end_offset + 2, "");
                    } else {
                        break;
                    }
                }
                let label = raw_heading.trim().to_uppercase_1to1();
                current_scene = Some(NavigatorItem {
                    line_idx: row.line_idx,
                    label,
                    is_section: false,
                    scene_num: row.scene_num.clone(),
                    synopses: Vec::new(),
                    color: row.override_color,
                });
            } else if row.line_type == LineType::Synopsis {
                if let Some(ref mut s) = current_scene {
                    let note_text = strip_sigils(&row.raw_text, row.line_type).to_string();
                    if !note_text.is_empty() {
                        s.synopses.push(note_text);
                    }
                }
            }
        }
        if let Some(s) = current_scene {
            self.scenes.push(s);
        }

        if self.scenes.is_empty() {
            self.set_status("No scenes found");
        } else {
            self.mode = AppMode::SceneNavigator;
            self.selected_scene = 0;
            for (idx, item) in self.scenes.iter().enumerate() {
                if item.line_idx <= self.cursor_y {
                    self.selected_scene = idx;
                } else {
                    break;
                }
            }
            self.navigator_state.select(Some(self.selected_scene));
        }
    }

    /// Normalizes a character name by stripping parenthetical extensions
    /// like (V.O.), (CONT'D), (O.S.), etc. so that EDWARD (V.O.) and
    /// EDWARD (CONT'D) both collapse to EDWARD.
    pub fn normalize_character_name(raw: &str) -> String {
        let trimmed = raw.trim();
        if let Some(idx) = trimmed.find('(') {
            trimmed[..idx].trim().to_uppercase()
        } else {
            trimmed.to_uppercase()
        }
    }

    pub fn open_character_sidebar(&mut self) {
        self.nav_original_pos = Some((self.cursor_y, self.cursor_x));
        use std::collections::HashMap;
        self.character_stats.clear();
        let mut stats_map: HashMap<String, CharacterItem> = HashMap::new();
        let mut current_scene = "Untitled Scene".to_string();
        let mut current_character: Option<String> = None;

        for row in &self.layout {
            if row.line_type == LineType::SceneHeading {
                let mut raw_heading = strip_sigils(&row.raw_text, row.line_type).to_string();
                while let Some(start) = raw_heading.find("[[") {
                    if let Some(end_offset) = raw_heading[start..].find("]]") {
                        raw_heading.replace_range(start..start + end_offset + 2, "");
                    } else {
                        break;
                    }
                }
                current_scene = raw_heading.trim().to_uppercase_1to1();
            }

            if row.line_type == LineType::Character || row.line_type == LineType::DualDialogueCharacter {
                let raw_name = strip_sigils(&row.raw_text, row.line_type).trim().to_string();
                let name = Self::normalize_character_name(&raw_name);
                let entry = stats_map
                    .entry(name.clone())
                    .or_insert_with(|| CharacterItem {
                        name: name.clone(),
                        ..Default::default()
                    });
                entry.dialogue_blocks += 1;
                if !entry
                    .appears_in_scenes
                    .iter()
                    .any(|(s, _)| s == &current_scene)
                {
                    entry
                        .appears_in_scenes
                        .push((current_scene.clone(), row.line_idx));
                    entry.scenes_count += 1;
                }
                current_character = Some(name);
            } else if row.line_type == LineType::Dialogue {
                if let Some(name) = &current_character
                    && let Some(entry) = stats_map.get_mut(name) {
                        let words = row.raw_text.split_whitespace().count();
                        entry.word_count += words;
                    }
            } else if row.line_type != LineType::Parenthetical {
                current_character = None;
            }
        }

        let mut stats: Vec<CharacterItem> = stats_map.into_values().collect();
        // Sort by dialogue prominence
        stats.sort_by(|a, b| {
            (b.dialogue_blocks * 10 + b.word_count).cmp(&(a.dialogue_blocks * 10 + a.word_count))
        });

        self.character_stats = stats;
        self.selected_character = 0;
        self.refresh_ensemble_list();
        self.selected_ensemble_idx = 0;
        self.ensemble_state.select(Some(0));
        self.mode = AppMode::CharacterNavigator;
    }

    pub fn compute_xray(&mut self) {
        use std::collections::HashMap;
        use crate::app::{XRayData, XRayCharacter, XRayScene, PacingBlock, XRaySceneBreakdown};
        use crate::types::LINES_PER_PAGE;
        use crate::layout::SCENE_NUM_RE;

        let mut char_stats: HashMap<String, (usize, usize)> = HashMap::new(); // name -> (words, lines)
        let mut current_character: Option<String> = None;

        // Scene tracking
        let mut scenes: Vec<XRayScene> = Vec::new();
        let mut current_scene_label = String::new();
        let mut current_scene_num: Option<String> = None;
        let mut current_scene_line_idx: usize = 0;
        let mut current_scene_visual_rows: usize = 0;
        let mut in_scene = false;

        // Pacing: per-page action vs dialogue counts
        let mut pacing_map: HashMap<usize, (usize, usize)> = HashMap::new(); // page -> (action, dialogue)
        let mut current_page: usize = 1;

        // Breakdown: Department/Key -> List of unique assets
        let mut global_breakdown: std::collections::BTreeMap<String, std::collections::BTreeSet<String>> = std::collections::BTreeMap::new();
        let mut scene_breakdowns: Vec<XRaySceneBreakdown> = Vec::new();
        let mut current_scene_tags: std::collections::BTreeMap<String, std::collections::BTreeSet<String>> = std::collections::BTreeMap::new();
        let mut last_scene_label = String::new();
        let mut last_scene_num = None;

        for (_i, (line, &lt)) in self.lines.iter().zip(self.types.iter()).enumerate() {
            if lt == LineType::SceneHeading {
                if !last_scene_label.is_empty() || !current_scene_tags.is_empty() {
                    scene_breakdowns.push(XRaySceneBreakdown {
                        label: last_scene_label.clone(),
                        scene_num: last_scene_num.clone(),
                        breakdown: std::mem::take(&mut current_scene_tags),
                    });
                }
                let mut label = strip_sigils(line, lt).to_string();
                while let Some(start) = label.find("[[") {
                    if let Some(end_offset) = label[start..].find("]]") {
                        label.replace_range(start..start + end_offset + 2, "");
                    } else { break; }
                }
                last_scene_label = label.trim().to_uppercase();
                // Find scene num if possible (optional for breakdown but nice)
                last_scene_num = if line.trim_end().ends_with('#') && let Some(caps) = SCENE_NUM_RE.captures(line) {
                    Some(caps[2].trim().to_string())
                } else { None };
            }

            // Extract tags from ANY line (including hidden ones)
            let mut start_search = 0;
            while let Some(start) = line[start_search..].find("[[") {
                let abs_start = start_search + start;
                if let Some(end) = line[abs_start..].find("]]") {
                    let abs_end = abs_start + end;
                    let content = &line[abs_start + 2..abs_end];
                    if let Some((key, val)) = content.split_once(':') {
                        let key = key.trim().to_uppercase();
                        if !key.is_empty() {
                            for v in val.split(',') {
                                let v_trimmed = v.trim();
                                if !v_trimmed.is_empty() {
                                    global_breakdown.entry(key.clone()).or_default().insert(v_trimmed.to_string());
                                    current_scene_tags.entry(key.clone()).or_default().insert(v_trimmed.to_string());
                                }
                            }
                        }
                    }
                    start_search = abs_end + 2;
                } else { break; }
            }
        }
        // Last scene
        if !last_scene_label.is_empty() || !current_scene_tags.is_empty() {
            scene_breakdowns.push(XRaySceneBreakdown {
                label: last_scene_label,
                scene_num: last_scene_num,
                breakdown: std::mem::take(&mut current_scene_tags),
            });
        }

        for row in &self.layout {
            // Track page boundaries
            if let Some(p) = row.page_num {
                current_page = p;
            }

            match row.line_type {
                LineType::SceneHeading => {
                    // Close previous scene
                    if in_scene {
                        let page_count = current_scene_visual_rows as f32 / LINES_PER_PAGE as f32;
                        scenes.push(XRayScene {
                            label: current_scene_label.clone(),
                            scene_num: current_scene_num.clone(),
                            page_count,
                            is_over_limit: page_count > 3.0,
                            line_idx: current_scene_line_idx,
                        });
                    }

                    let mut label = strip_sigils(&row.raw_text, row.line_type).to_string();
                    // Strip inline notes
                    while let Some(start) = label.find("[[") {
                        if let Some(end_offset) = label[start..].find("]]") {
                            label.replace_range(start..start + end_offset + 2, "");
                        } else {
                            break;
                        }
                    }
                    current_scene_label = label.trim().to_uppercase();
                    current_scene_num = row.scene_num.clone();
                    current_scene_line_idx = row.line_idx;
                    current_scene_visual_rows = 1;
                    in_scene = true;

                    let entry = pacing_map.entry(current_page).or_insert((0, 0));
                    entry.0 += 1; // Scene headings count as action
                }
                LineType::Character | LineType::DualDialogueCharacter => {
                    let raw_name = strip_sigils(&row.raw_text, row.line_type).trim().to_string();
                    let name = Self::normalize_character_name(&raw_name);
                    current_character = Some(name);
                    if in_scene {
                        current_scene_visual_rows += 1;
                    }
                }
                LineType::Dialogue => {
                    if let Some(ref name) = current_character {
                        let words = row.raw_text.split_whitespace().count();
                        let entry = char_stats.entry(name.clone()).or_insert((0, 0));
                        entry.0 += words;
                        entry.1 += 1;
                    }
                    if in_scene {
                        current_scene_visual_rows += 1;
                    }
                    let entry = pacing_map.entry(current_page).or_insert((0, 0));
                    entry.1 += 1; // dialogue line
                }
                LineType::Parenthetical => {
                    if in_scene {
                        current_scene_visual_rows += 1;
                    }
                    let entry = pacing_map.entry(current_page).or_insert((0, 0));
                    entry.1 += 1; // parenthetical counts as dialogue
                }
                LineType::Action | LineType::Shot => {
                    current_character = None;
                    if in_scene {
                        current_scene_visual_rows += 1;
                    }
                    let entry = pacing_map.entry(current_page).or_insert((0, 0));
                    entry.0 += 1; // action line
                }
                LineType::Transition => {
                    current_character = None;
                    if in_scene {
                        current_scene_visual_rows += 1;
                    }
                    let entry = pacing_map.entry(current_page).or_insert((0, 0));
                    entry.0 += 1;
                }
                LineType::Empty => {
                    if in_scene {
                        current_scene_visual_rows += 1;
                    }
                }
                _ => {
                    if row.line_type != LineType::Parenthetical {
                        current_character = None;
                    }
                    if in_scene {
                        current_scene_visual_rows += 1;
                    }
                }
            }
        }

        // Close last scene
        if in_scene {
            let page_count = current_scene_visual_rows as f32 / LINES_PER_PAGE as f32;
            scenes.push(XRayScene {
                label: current_scene_label.clone(),
                scene_num: current_scene_num,
                page_count,
                is_over_limit: page_count > 3.0,
                line_idx: current_scene_line_idx,
            });
        }

        // Build character list
        let total_dialogue_words: usize = char_stats.values().map(|(w, _)| w).sum();
        let mut characters: Vec<XRayCharacter> = char_stats
            .into_iter()
            .map(|(name, (word_count, dialogue_lines))| {
                let percentage = if total_dialogue_words > 0 {
                    (word_count as f32 / total_dialogue_words as f32) * 100.0
                } else {
                    0.0
                };
                XRayCharacter {
                    name,
                    word_count,
                    dialogue_lines,
                    percentage,
                }
            })
            .collect();
        characters.sort_by(|a, b| b.word_count.cmp(&a.word_count));

        // Build pacing blocks
        let max_page = pacing_map.keys().max().copied().unwrap_or(1);
        let pacing: Vec<PacingBlock> = (1..=max_page)
            .map(|p| {
                let (action, dialogue) = pacing_map.get(&p).copied().unwrap_or((0, 0));
                PacingBlock {
                    page: p,
                    action_lines: action,
                    dialogue_lines: dialogue,
                }
            })
            .collect();

        let has_breakdowns = !scene_breakdowns.is_empty();
        self.xray_data = Some(XRayData {
            characters,
            total_dialogue_words,
            scenes,
            pacing_map: pacing,
            global_breakdown: global_breakdown,
            scene_breakdown: scene_breakdowns,
        });
        self.xray_scroll = 0;
        self.xray_tab = 0;
        self.xray_breakdown_idx = 0;
        if has_breakdowns {
            self.xray_breakdown_state.select(Some(0));
        } else {
            self.xray_breakdown_state.select(None);
        }
        self.mode = AppMode::XRay;
    }

    pub fn refresh_ensemble_list(&mut self) {
        self.ensemble_items.clear();
        for i in 0..self.character_stats.len() {
            let item = self.character_stats[i].clone();

            // Character Header
            self.ensemble_items.push(EnsembleItem::CharacterHeader(i));

            // Stat: Scenes (with Hint)
            let scene_hint = if item.is_expanded {
                Some("(Cast in Scenes ↓)".to_string())
            } else {
                Some("(Tab to show)".to_string())
            };
            self.ensemble_items.push(EnsembleItem::Stat(
                format!("Scenes: {}", item.scenes_count),
                scene_hint,
                false,
            ));

            // Scene Links (if expanded)
            if item.is_expanded {
                for (j, (scene_name, line_idx)) in item.appears_in_scenes.iter().enumerate() {
                    let is_last_scene = j == item.appears_in_scenes.len() - 1;
                    self.ensemble_items.push(EnsembleItem::SceneLink(
                        scene_name.clone(),
                        *line_idx,
                        is_last_scene,
                    ));
                }
            }

            // Stat: Dialogues
            self.ensemble_items.push(EnsembleItem::Stat(
                format!("Dialogues: {}", item.dialogue_blocks),
                None,
                false,
            ));

            // Stat: Words (Last stat in tree)
            self.ensemble_items.push(EnsembleItem::Stat(
                format!("Words: {}", item.word_count),
                None,
                true,
            ));

            // Separator
            self.ensemble_items.push(EnsembleItem::Separator);
        }
    }

    pub fn parse_document(&mut self) {
        self.types = Parser::parse(&self.lines);

        // Forced Uppercase Transformation for key elements
        for i in 0..self.lines.len() {
            let lt = self.types[i];
            if matches!(
                lt,
                LineType::SceneHeading
                    | LineType::Character
                    | LineType::DualDialogueCharacter
                    | LineType::Transition
            ) {
                // Determine the clean upper version to avoid unnecessary updates
                let current = &self.lines[i];
                let upper = current.to_uppercase_1to1();
                if *current != upper {
                    self.lines[i] = upper;
                    self.dirty = true;
                }
            }
        }

        // Production lock: auto-assign suffixed numbers to new scenes
        if self.config.production_lock {
            self.auto_number_locked_scenes();
        }

        self.characters.clear();
        self.locations.clear();

        for (i, t) in self.types.iter().enumerate() {
            if *t == LineType::Character || *t == LineType::DualDialogueCharacter {
                let full_name = self.lines[i]
                    .trim_start_matches('@')
                    .trim_end_matches('^')
                    .trim();
                let name = if let Some(idx) = full_name.find('(') {
                    full_name[..idx].trim()
                } else {
                    full_name
                };
                if !name.is_empty() {
                    self.characters.insert(name.to_uppercase_1to1());
                }
            } else if *t == LineType::SceneHeading {
                let scene = self.lines[i].trim().to_uppercase_1to1();
                let mut loc_str = scene.as_str();
                let mut matched = false;

                if loc_str.starts_with('.') && !loc_str.starts_with("..") {
                    loc_str = &loc_str[1..];
                } else {
                    let prefixes = [
                        "INT. ",
                        "EXT. ",
                        "EST. ",
                        "INT/EXT. ",
                        "I/E. ",
                        "E/I. ",
                        "I./E. ",
                        "E./I. ",
                        "INT ",
                        "EXT ",
                        "EST ",
                        "INT/EXT ",
                        "I/E ",
                        "E/I ",
                    ];
                    for p in prefixes {
                        if let Some(rest) = loc_str.strip_prefix(p) {
                            loc_str = rest;
                            matched = true;
                            break;
                        }
                    }
                    if !matched && let Some((_, rest)) = loc_str.split_once(". ") {
                        loc_str = rest;
                    }
                }

                let mut final_loc = loc_str.trim().to_string();

                if final_loc.ends_with('#')
                    && let Some(idx) = final_loc.rfind(" #")
                {
                    final_loc.truncate(idx);
                    final_loc = final_loc.trim().to_string();
                }

                if !final_loc.is_empty() {
                    self.locations.insert(final_loc);
                }
            }
        }
        self.update_index_cards();
        self.update_metadata();
    }
}

impl crate::app::App {
    pub fn calculate_scene_height(&self, item: &NavigatorItem) -> usize {
        if item.is_section {
            return 2; // Section name + spacer
        }

        let max_w: usize = 45; // Match the wider navigator sidebar
        let mut height: usize = 0;

        // Heading wrapping
        let mut current_line_len: usize = 0;
        let heading_indent: usize = 5; // prefix(3) + connector(2)
        for word in item.label.split_whitespace() {
            if current_line_len + word.len() + heading_indent + 1 > max_w {
                height += 1;
                current_line_len = 0;
            }
            if current_line_len > 0 {
                current_line_len += 1;
            }
            current_line_len += word.len();
        }
        if current_line_len > 0 || height == 0 {
            height += 1;
        }

        // Synopsis wrapping
        for syn in &item.synopses {
            let mut current_line_len: usize = 0;
            let max_syn_w = 34; // Sync with UI
            let mut syn_lines: usize = 0;
            for word in syn.split_whitespace() {
                if current_line_len + word.len() + 1 > max_syn_w {
                    syn_lines += 1;
                    current_line_len = word.len();
                } else {
                    if current_line_len > 0 {
                        current_line_len += 1;
                    }
                    current_line_len += word.len();
                }
            }
            if current_line_len > 0 {
                syn_lines += 1;
            }
            height += syn_lines;
        }

        height += 1; // Empty separator line or ending spacer
        height
    }
}
