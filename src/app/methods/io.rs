use std::{fs, io};
use std::path::PathBuf;
use crate::app::App;

impl App {
    pub fn emergency_save(&mut self) {
        let mut to_save = Vec::new();
        to_save.push((self.file.clone(), &self.lines, self.dirty));

        for (i, buf) in self.buffers.iter().enumerate() {
            if i != self.current_buf_idx {
                to_save.push((buf.file.clone(), &buf.lines, buf.dirty));
            }
        }

        for (file, lines, dirty) in to_save {
            if !dirty || lines.is_empty() || (lines.len() == 1 && lines[0].is_empty()) {
                continue;
            }

            let dir = file
                .as_ref()
                .and_then(|p| p.parent())
                .filter(|p| !p.as_os_str().is_empty())
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

            let base_name = file
                .as_ref()
                .and_then(|p| p.file_name())
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| "fount".to_string());

            let mut backup_path = dir.join(format!("{}.save", base_name));
            let mut counter = 1;

            while backup_path.exists() && counter <= 1000 {
                backup_path = dir.join(format!("{}.save.{}", base_name, counter));
                counter += 1;
            }

            if counter <= 1000 {
                let content = lines.join("\n");
                let _ = std::fs::write(&backup_path, content);
            }
        }
    }

    pub fn set_status(&mut self, msg: &str) {
        self.status_msg = Some(msg.to_string());
    }

    pub fn clear_status(&mut self) {
        self.status_msg = None;
    }

    pub fn load_recent_files(&mut self) {
        if let Some(proj_dirs) = directories::ProjectDirs::from("", "", "Fount") {
            let path = proj_dirs.data_dir().join("recent.txt");
            if let Ok(content) = fs::read_to_string(path) {
                self.recent_files = content
                    .lines()
                    .map(PathBuf::from)
                    .filter(|p| p.exists())
                    .collect();
            }
        }
    }

    pub fn save_recent_files(&self) {
        if let Some(proj_dirs) = directories::ProjectDirs::from("", "", "Fount") {
            let path = proj_dirs.data_dir().join("recent.txt");
            let content = self.recent_files
                .iter()
                .map(|p| p.to_string_lossy())
                .collect::<Vec<_>>()
                .join("\n");
            let _ = fs::write(path, content);
        }
    }

    pub fn add_recent_file(&mut self, path: PathBuf) {
        let path = path.canonicalize().unwrap_or(path);
        self.recent_files.retain(|p| p != &path);
        self.recent_files.insert(0, path);
        if self.recent_files.len() > 10 {
            self.recent_files.truncate(10);
        }
        self.save_recent_files();
    }

    pub fn save_as(&mut self, path: PathBuf) -> io::Result<()> {
        if self.is_tutorial {
            self.set_status("Cannot save the tutorial buffer. Press Ctrl+X to exit.");
            return Ok(());
        }
        let mut content = self.lines.join("\n");
        content.push_str(&self.get_revision_block());
        fs::write(&path, content)?;
        self.file = Some(path.clone());
        self.dirty = false;
        self.add_recent_file(path.clone());
        self.set_status(&format!(
            "Saved as {}",
            path.display()
        ));
        self.save_indicator_timer = Some(std::time::Instant::now());
        Ok(())
    }

    pub fn export_fountain(&self, path: &std::path::Path) -> std::io::Result<()> {
        let mut content = self.lines.join("\n");
        content.push_str(&self.get_revision_block());
        std::fs::write(path, content)
    }

    pub fn export_pdf(&self, path: &std::path::Path) -> std::io::Result<()> {
        let fountain_text = self.lines.join("\n");
        let paper_size = if self.config.paper_size.to_lowercase() == "letter" {
            crate::pdf::LETTER
        } else {
            crate::pdf::A4
        };

        crate::pdf::export_to_pdf(
            &fountain_text,
            path,
            paper_size,
            self.config.export_bold_scene_headings,
            self.config.mirror_scene_numbers.clone(),
            self.config.export_sections,
            self.config.export_synopses,
            self.config.export_font.clone(),
            self.revised_lines.clone(),
        )
    }

    pub fn export_scene_csv(&self, path: &std::path::Path) -> std::io::Result<()> {
        let mut csv = String::new();
        csv.push_str("Scene Number,Int/Ext,Location,Time,Estimated Length (8ths)\n");

        let mut current_scene = None;
        let mut scene_lines = 0;
        let mut scenes_data = Vec::new();

        for row in &self.layout {
            if row.line_type == crate::types::LineType::SceneHeading {
                if let Some((s_num, heading)) = current_scene.take() {
                    scenes_data.push((s_num, heading, scene_lines));
                }

                let s_num = row.scene_num.clone().unwrap_or_default();
                let heading = crate::layout::strip_sigils(&row.raw_text, row.line_type).to_string();
                current_scene = Some((s_num, heading));
                scene_lines = 1;
            } else if current_scene.is_some() {
                scene_lines += 1;
            }
        }

        if let Some((s_num, heading)) = current_scene.take() {
            scenes_data.push((s_num, heading, scene_lines));
        }

        for (s_num, heading, visual_lines) in scenes_data {
            let eights_total = visual_lines as f32 / 7.0;
            let eights_rounded = eights_total.round() as usize;

            let full_pages = eights_rounded / 8;
            let remaining_eighths = eights_rounded % 8;

            let length_str = if full_pages > 0 && remaining_eighths > 0 {
                format!("{} {}/8", full_pages, remaining_eighths)
            } else if full_pages > 0 {
                format!("{}", full_pages)
            } else if remaining_eighths > 0 {
                format!("{}/8", remaining_eighths)
            } else {
                "1/8".to_string()
            };

            let mut int_ext = String::new();
            let loc;
            let mut time = String::new();
            let h = heading.to_uppercase();
            if let Some((ie, rest)) = h.split_once('.') {
                int_ext = ie.trim().to_string();
                if let Some((l, t)) = rest.split_once('-') {
                    loc = l.trim().to_string();
                    time = t.trim().to_string();
                } else {
                    loc = rest.trim().to_string();
                }
            } else {
                loc = h;
            }

            csv.push_str(&format!(
                "\"{}\",\"{}\",\"{}\",\"{}\",\"{}\"\n",
                s_num, int_ext, loc, time, length_str
            ));
        }

        std::fs::write(path, csv)
    }

    pub fn export_character_csv(&self, path: &std::path::Path) -> std::io::Result<()> {
        let mut csv = String::new();
        csv.push_str("Character,Dialogue Words,Scenes\n");

        let mut char_word_counts = std::collections::HashMap::new();
        let mut char_scenes = std::collections::HashMap::new();

        let mut current_scene = String::new();
        let mut current_char = String::new();

        for row in &self.layout {
            match row.line_type {
                crate::types::LineType::SceneHeading => {
                    if let Some(snum) = &row.scene_num {
                        current_scene = snum.clone();
                    } else {
                        current_scene = String::new();
                    }
                }
                crate::types::LineType::Character
                | crate::types::LineType::DualDialogueCharacter => {
                    let mut name = crate::layout::strip_sigils(&row.raw_text, row.line_type)
                        .trim()
                        .to_string();
                    if let Some(idx) = name.find('(') {
                        name = name[..idx].trim().to_string(); // Strip (V.O.) and (CONT'D)
                    }
                    current_char = name.to_uppercase();
                    if !current_scene.is_empty() {
                        let scenes: &mut std::collections::HashSet<String> =
                            char_scenes.entry(current_char.clone()).or_default();
                        scenes.insert(current_scene.clone());
                    }
                }
                crate::types::LineType::Dialogue => {
                    let text = crate::layout::strip_sigils(&row.raw_text, row.line_type);
                    if !current_char.is_empty() {
                        let words = text.split_whitespace().count();
                        *char_word_counts.entry(current_char.clone()).or_insert(0) += words;
                    }
                }
                _ => {
                    if row.line_type != crate::types::LineType::Parenthetical {
                        current_char = String::new();
                    }
                }
            }
        }

        let mut sorted_chars: Vec<_> = char_word_counts.into_iter().collect();
        sorted_chars.sort_by(|a, b| b.1.cmp(&a.1));

        for (ch, words) in sorted_chars {
            let scenes = char_scenes.get(&ch).cloned().unwrap_or_default();
            let mut scene_list: Vec<_> = scenes.into_iter().collect();
            scene_list.sort();
            let scenes_str = scene_list.join(", ");
            csv.push_str(&format!("\"{}\",{},\"{}\"\n", ch, words, scenes_str));
        }

        std::fs::write(path, csv)
    }

    pub fn export_location_csv(&self, path: &std::path::Path) -> std::io::Result<()> {
        let mut csv = String::new();
        csv.push_str("Location,Int/Ext,Time,Scenes\n");

        let mut locations: std::collections::HashMap<(String, String, String), Vec<String>> = std::collections::HashMap::new();

        for row in &self.layout {
            if row.line_type == crate::types::LineType::SceneHeading {
                let s_num = row.scene_num.clone().unwrap_or_default();
                let heading = crate::layout::strip_sigils(&row.raw_text, row.line_type).to_string();
                
                let mut int_ext = String::new();
                let loc;
                let mut time = String::new();
                let h = heading.to_uppercase();
                if let Some((ie, rest)) = h.split_once('.') {
                    int_ext = ie.trim().to_string();
                    if let Some((l, t)) = rest.split_once('-') {
                        loc = l.trim().to_string();
                        time = t.trim().to_string();
                    } else {
                        loc = rest.trim().to_string();
                    }
                } else {
                    loc = h;
                }

                let key = (loc, int_ext, time);
                locations.entry(key).or_default().push(if s_num.is_empty() { String::from("-") } else { s_num });
            }
        }

        let mut sorted_locs: Vec<_> = locations.into_iter().collect();
        sorted_locs.sort_by(|a, b| a.0.0.cmp(&b.0.0).then(a.0.1.cmp(&b.0.1)).then(a.0.2.cmp(&b.0.2)));

        for ((loc, int_ext, time), scenes) in sorted_locs {
            csv.push_str(&format!("\"{}\",\"{}\",\"{}\",\"{}\"\n", loc, int_ext, time, scenes.join(", ")));
        }

        std::fs::write(path, csv)
    }

    pub fn export_note_csv(&self, path: &std::path::Path) -> std::io::Result<()> {
        let mut csv = String::new();
        csv.push_str("Type,Scene,Content\n");
        
        let mut current_scene = String::new();

        for row in &self.layout {
            if row.line_type == crate::types::LineType::SceneHeading {
                current_scene = row.scene_num.clone().unwrap_or_default();
                if current_scene.is_empty() {
                    current_scene = crate::layout::strip_sigils(&row.raw_text, row.line_type).trim().to_string();
                }
            } else if row.line_type == crate::types::LineType::Note || row.line_type == crate::types::LineType::Boneyard {
                let note_type = if row.line_type == crate::types::LineType::Note { "Note" } else { "Boneyard" };
                let text = crate::layout::strip_sigils(&row.raw_text, row.line_type).to_string();
                let clean_text = text.replace("\"", "\"\"");
                csv.push_str(&format!("\"{}\",\"{}\",\"{}\"\n", note_type, current_scene, clean_text));
            }
        }
        std::fs::write(path, csv)
    }

    pub fn export_breakdown_csv(&self, path: &std::path::Path) -> std::io::Result<()> {
        let mut csv = String::new();
        csv.push_str("Scene Number,Int/Ext,Location,Time,Length,Cast,Props,VFX,SFX,Wardrobe,Makeup,Music,Tags\n");

        #[derive(Default)]
        struct SceneBreakdown {
            num: String,
            int_ext: String,
            location: String,
            time: String,
            length: String,
            cast: std::collections::BTreeSet<String>,
            props: std::collections::BTreeSet<String>,
            vfx: std::collections::BTreeSet<String>,
            sfx: std::collections::BTreeSet<String>,
            wardrobe: std::collections::BTreeSet<String>,
            makeup: std::collections::BTreeSet<String>,
            music: std::collections::BTreeSet<String>,
            tags: std::collections::BTreeSet<String>,
        }

        let mut scene_data: Vec<SceneBreakdown> = Vec::new();
        let mut current_scene: Option<SceneBreakdown> = None;
        let mut scene_visual_lines = 0;

        // Pass 1: Collect scene metadata and basic structure from layout
        // (Layout is better for scene headings and pagination-based length)
        for row in &self.layout {
            if row.line_type == crate::types::LineType::SceneHeading {
                if let Some(mut s) = current_scene.take() {
                    let eights_total = scene_visual_lines as f32 / 7.0;
                    let eights_rounded = eights_total.round() as usize;
                    let full_pages = eights_rounded / 8;
                    let remaining_eighths = eights_rounded % 8;
                    s.length = if full_pages > 0 && remaining_eighths > 0 {
                        format!("{} {}/8", full_pages, remaining_eighths)
                    } else if full_pages > 0 {
                        format!("{}", full_pages)
                    } else if remaining_eighths > 0 {
                        format!("{}/8", remaining_eighths)
                    } else { "1/8".to_string() };
                    scene_data.push(s);
                }

                let mut s = SceneBreakdown::default();
                s.num = row.scene_num.clone().unwrap_or_default();
                let heading = crate::layout::strip_sigils(&row.raw_text, row.line_type).to_uppercase();
                if let Some((ie, rest)) = heading.split_once('.') {
                    s.int_ext = ie.trim().to_string();
                    if let Some((l, t)) = rest.split_once('-') {
                        s.location = l.trim().to_string();
                        s.time = t.trim().to_string();
                    } else { s.location = rest.trim().to_string(); }
                } else { s.location = heading; }

                current_scene = Some(s);
                scene_visual_lines = 1;
            } else if current_scene.is_some() {
                scene_visual_lines += 1;
            }
        }

        if let Some(mut s) = current_scene.take() {
            let eights_total = scene_visual_lines as f32 / 7.0;
            let eights_rounded = eights_total.round() as usize;
            let full_pages = eights_rounded / 8;
            let remaining_eighths = eights_rounded % 8;
            s.length = if full_pages > 0 && remaining_eighths > 0 {
                format!("{} {}/8", full_pages, remaining_eighths)
            } else if full_pages > 0 {
                format!("{}", full_pages)
            } else if remaining_eighths > 0 {
                format!("{}/8", remaining_eighths)
            } else { "1/8".to_string() };
            scene_data.push(s);
        }

        // Pass 2: Collect tags from raw lines and associate with scenes
        let mut scene_idx = 0;
        for (i, (line, &lt)) in self.lines.iter().zip(self.types.iter()).enumerate() {
            if lt == crate::types::LineType::SceneHeading && i > 0 {
                if scene_idx + 1 < scene_data.len() {
                    scene_idx += 1;
                }
            }

            if let Some(s) = scene_data.get_mut(scene_idx) {
                // Character name collection (special case, not a tag)
                if lt == crate::types::LineType::Character || lt == crate::types::LineType::DualDialogueCharacter {
                    let mut name = crate::layout::strip_sigils(line, lt).trim().to_string();
                    if let Some(idx) = name.find('(') { name = name[..idx].trim().to_string(); }
                    if !name.is_empty() { s.cast.insert(name.to_uppercase()); }
                }

                // Tag extraction
                let mut start_search = 0;
                while let Some(start) = line[start_search..].find("[[") {
                    let abs_start = start_search + start;
                    if let Some(end) = line[abs_start..].find("]]") {
                        let abs_end = abs_start + end;
                        let content = &line[abs_start + 2..abs_end];
                        if let Some((key, val)) = content.split_once(':') {
                            let key = key.trim().to_uppercase();
                            let values: Vec<_> = val.split(',').map(|v| v.trim()).filter(|v| !v.is_empty()).collect();
                            for v in values {
                                match key.as_str() {
                                    "CAST" => { s.cast.insert(v.to_uppercase()); }
                                    "PROPS" => { s.props.insert(v.to_string()); }
                                    "VFX" => { s.vfx.insert(v.to_string()); }
                                    "SFX" => { s.sfx.insert(v.to_string()); }
                                    "WARDROBE" => { s.wardrobe.insert(v.to_string()); }
                                    "MAKEUP" => { s.makeup.insert(v.to_string()); }
                                    "MUSIC" => { s.music.insert(v.to_string()); }
                                    _ => { s.tags.insert(format!("{}:{}", key, v)); }
                                }
                            }
                        }
                        start_search = abs_end + 2;
                    } else { break; }
                }
            }
        }

        // Final CSV generation
        for s in scene_data {
            let escape_csv = |set: &std::collections::BTreeSet<String>| -> String {
                let joined = set.iter().cloned().collect::<Vec<_>>().join(", ");
                joined.replace("\"", "\"\"")
            };

            csv.push_str(&format!(
                "\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\"\n",
                s.num, s.int_ext, s.location, s.time, s.length,
                escape_csv(&s.cast), escape_csv(&s.props), escape_csv(&s.vfx), escape_csv(&s.sfx),
                escape_csv(&s.wardrobe), escape_csv(&s.makeup), escape_csv(&s.music), escape_csv(&s.tags)
            ));
        }

        std::fs::write(path, csv)
    }

    pub fn export_dialogue_txt(&self, path: &std::path::Path) -> std::io::Result<()> {
        let mut out = String::new();
        let mut is_first = true;
        for row in &self.layout {
            match row.line_type {
                crate::types::LineType::Character |
                crate::types::LineType::DualDialogueCharacter |
                crate::types::LineType::Parenthetical |
                crate::types::LineType::Dialogue => {
                    let text = crate::layout::strip_sigils(&row.raw_text, row.line_type);
                    if row.line_type == crate::types::LineType::Character || row.line_type == crate::types::LineType::DualDialogueCharacter {
                        if !is_first {
                            out.push_str("\n");
                        }
                    }
                    out.push_str(&text);
                    out.push_str("\n");
                    is_first = false;
                }
                _ => {}
            }
        }
        std::fs::write(path, out)
    }

    pub fn set_error(&mut self, msg: &str) {
        self.status_msg = Some(msg.to_string());
        self.command_error = true;
    }

    pub fn get_revision_block(&self) -> String {
        let mut revs = Vec::new();
        for (i, &rev) in self.revised_lines.iter().enumerate() {
            if rev {
                revs.push(i.to_string());
            }
        }
        if revs.is_empty() && !self.revision_mode {
            return String::new();
        }
        format!(
            "\n/*\nFOUNT_REVISIONS: {}\nREVISION_MODE: {}\n*/",
            revs.join(", "),
            if self.revision_mode { "ON" } else { "OFF" }
        )
    }

    pub fn save(&mut self) -> io::Result<()> {
        if self.is_tutorial {
            self.set_status("Cannot save the tutorial buffer. Press Ctrl+X to exit.");
            return Ok(());
        }
        if let Some(ref p) = self.file {
            let mut content = self.lines.join("\n");
            if !content.ends_with('\n') {
                content.push('\n');
            }
            fs::write(p, content)?;
            self.dirty = false;
            self.set_status(&format!("Wrote {} lines", self.lines.len()));

            // Trigger snapshot on manual save
            self.trigger_snapshot();
            self.save_indicator_timer = Some(std::time::Instant::now());
        }
        Ok(())
    }
}
