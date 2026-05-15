use super::*;

    #[test]
    fn test_integration() {
        let tutorial_text = r#"Title: Fount Tutorial
Credit: Written by
Author: René Coignard
Draft date: Version 0.2.17
Contact:
contact@renecoignard.com

INT. FLAT IN WOLFEN-NORD - DAY

RENÉ sits at his desk, typing.

RENÉ
(turning round)
Oh, hello there. It seems you've found my terminal Rust port of Beat. Sit back and I'll show you how everything works.

I sometimes write screenplays on my Gentoo laptop, and doing it in plain nano isn't terribly comfortable (I work entirely in the terminal there). So I decided to put this port of Beat together. I used Beat's source code as a reference when writing Fount, so things work more or less the same way.

As you may have already noticed, the navigation is rather reminiscent of nano, because I did look at its source code and took inspiration, for the sake of authenticity. I'm rather fond of it, and I hope you will be too. Not quite as nerdy as vim, but honestly, I'm an average nano enjoyer and I'm not ashamed of it.

Anyway, let's get into it.

EXT. NORDPARK - DAY

As I mentioned, things work much the same as in Beat. If you start a line with **int.** or **ext.**, Fount will automatically turn it into a scene heading. You can also use tab: on an empty line, it will first turn it into a character cue, then a scene heading, and then a transition. If you simply start typing IN CAPS ON AN EMPTY LINE, LIKE SO, the text will automatically become a character cue.

You can also use notes...

/* Two sailors are walking along the deck, when one turns to the other and says: */

SAILOR
I'm not a sailor, actually.

Fount automatically inserts two blank lines after certain elements, just as Beat does, though this can be adjusted in the configuration file. There's a sample config in the repository; do make use of it. Bonus: try enabling typewriter mode and see what happens.

To create a transition, simply write in capitals and end with a colon, like so...

CUT TO:

That alone is quite enough to write a proper screenplay. But there's more! For instance, we also have these...

/*

A multi-line comment.

For very, very, very long notes.

*/

[[Comments can look like this as well. They don't differ much from other comment types, but for compatibility with Beat, all the same comment types are supported.]]

# This is a new section

= And this is a synopsis.

INT. EDEKA - ABEND

Unlike Beat, there's no full render or PDF export here, but you can always save your screenplay and open it in Beat to do that. In Beat, synopses wouldn't appear in the rendered script, nor would comments. Which is why they share the same colour here, incidentally.

As you may have noticed, there's support for **bold text**, *italics*, and even _underlined text_. When your cursor isn't on a line containing these markers, they'll be hidden from view. Move onto the line, and you'll see all the asterisks and underscores that produce the formatting.

Centred text is supported as well, and works like this...

>Centred text<

You can also force transitions...

>AN ABRUPT TRANSITION TO THE NEXT SCENE:

EXT. WOLFEN(BITTERFELD) RAILWAY STATION - MORNING

Lyrics are supported too, using a tilde at the start of the line...

~Meine Damen, meine Herrn, danke
~Dass Sie mit uns reisen
~Zu abgefahrenen Preisen
~Auf abgefahrenen Gleisen
~Für Ihre Leidensfähigkeit, danken wir spontan
~Sänk ju for träweling wis Deutsche Bahn

That's Wise Guys. Onwards.

EXT. LEIPZIG HBF - MORNING

Well, do have a go on it, write something from scratch, or edit this screenplay. You might even turn up a bug or two; if so, please do let me know :-) Everything seemed to behave itself while I was putting this tutorial together, and I hope it all runs just as smoothly for you. I hope you enjoy working in Fount.

[[marker Speaking of which, I named the application after a certain Charlotte I once knew, who wrote quite wonderful screenplays.]]
[[marker blue The colour of these comment markers can be changed, as you can see.]]

You can find more information about the Fountain markup language at https://www.fountain.io/

And Beat itself, of course: https://www.beat-app.fi/

> FADE OUT"#;

        let mut app = create_empty_app();
        app.config.mirror_scene_numbers = crate::config::MirrorOption::Off;
        app.config.export_sections = false;
        app.config.export_synopses = false;
        app.lines = tutorial_text.lines().map(|s| s.to_string()).collect();
        app.cursor_y = 0;
        app.cursor_x = 0;

        app.parse_document();
        app.update_layout();

        let get_exact_idx =
            |search_str: &str| -> usize { app.lines.iter().position(|l| l == search_str).unwrap() };
        let get_idx = |search_str: &str| -> usize {
            app.lines
                .iter()
                .position(|l| l.starts_with(search_str))
                .unwrap()
        };

        let meta_title_idx = get_idx("Title:");
        let meta_val_idx = get_idx("contact@renecoignard");
        let scene1_idx = get_idx("INT. FLAT");

        let char1_idx = get_exact_idx("RENÉ");

        let paren_idx = get_idx("(turning round)");
        let dial_idx = get_idx("Oh, hello there");
        let boneyard1_idx = get_idx("/* Two sailors");
        let trans1_idx = get_exact_idx("CUT TO:");
        let boneyard_multiline_idx = get_exact_idx("/*");
        let section_idx = get_idx("# This is");
        let syn_idx = get_idx("= And this");
        let inline_note_idx = get_idx("[[Comments");
        let markup_idx = get_idx("As you may have noticed, there's support for");
        let center_idx = get_exact_idx(">Centred text<");
        let force_trans_idx = get_idx(">AN ABRUPT");
        let lyric1_idx = get_idx("~Meine Damen");
        let lyric6_idx = get_idx("~Sänk ju");
        let note_marker_idx = get_idx("[[marker blue");
        let fade_out_idx = get_exact_idx("> FADE OUT");

        assert_eq!(app.types[meta_title_idx], LineType::MetadataTitle);
        assert_eq!(app.types[meta_val_idx], LineType::MetadataValue);
        assert_eq!(app.types[scene1_idx], LineType::SceneHeading);
        assert_eq!(app.types[char1_idx], LineType::Character);
        assert_eq!(app.types[paren_idx], LineType::Parenthetical);
        assert_eq!(app.types[dial_idx], LineType::Dialogue);
        assert_eq!(app.types[boneyard1_idx], LineType::Boneyard);
        assert_eq!(app.types[trans1_idx], LineType::Transition);
        assert_eq!(app.types[boneyard_multiline_idx], LineType::Boneyard);
        assert_eq!(app.types[section_idx], LineType::Section);
        assert_eq!(app.types[syn_idx], LineType::Synopsis);
        assert_eq!(app.types[inline_note_idx], LineType::Note);
        assert_eq!(app.types[center_idx], LineType::Centered);
        assert_eq!(app.types[force_trans_idx], LineType::Transition);
        assert_eq!(app.types[lyric1_idx], LineType::Lyrics);
        assert_eq!(app.types[lyric6_idx], LineType::Lyrics);
        assert_eq!(app.types[note_marker_idx], LineType::Note);
        assert_eq!(app.types[fade_out_idx], LineType::Transition);

        let layout_markup = app
            .layout
            .iter()
            .find(|r| r.line_idx == markup_idx)
            .unwrap();
        let styles = &layout_markup.fmt.char_styles;
        assert!(styles.iter().any(|s| s.contains(crate::formatting::StyleBits::BOLD)));
        assert!(styles.iter().any(|s| s.contains(crate::formatting::StyleBits::ITALIC)));
        assert!(styles.iter().any(|s| s.contains(crate::formatting::StyleBits::UNDERLINED)));

        let layout_note = app
            .layout
            .iter()
            .find(|r| r.line_idx == note_marker_idx)
            .unwrap();
        assert!(layout_note.override_color.is_some());
        assert_eq!(
            layout_note.override_color.unwrap(),
            ratatui::style::Color::Blue
        );

        let layout_scene = app
            .layout
            .iter()
            .find(|r| r.line_idx == scene1_idx)
            .unwrap();
        assert_eq!(layout_scene.scene_num.as_deref(), Some("1"));

        let layout_trans = app
            .layout
            .iter()
            .find(|r| r.line_idx == trans1_idx)
            .unwrap();
        let expected_indent = crate::types::PAGE_WIDTH.saturating_sub(7);
        assert_eq!(layout_trans.indent, expected_indent);
        assert_eq!(layout_trans.raw_text, "CUT TO:");

        assert!(app.characters.contains("RENÉ"));
        assert!(app.characters.contains("SAILOR"));
        assert!(app.locations.contains("FLAT IN WOLFEN-NORD - DAY"));

        let total_vis_lines = app.layout.len();
        assert!(total_vis_lines > 0, "Layout must not be empty");

        let test_coordinates: Vec<(usize, usize, String, usize)> = app
            .layout
            .iter()
            .filter_map(|r| {
                if r.is_phantom {
                    None
                } else {
                    Some((r.line_idx, r.char_start, r.raw_text.clone(), r.char_end))
                }
            })
            .collect();

        for (line_idx, char_start, raw_text, char_end) in test_coordinates {
            app.cursor_y = line_idx;
            app.cursor_x = char_start;
            app.report_cursor_position();

            let status = app
                .status_msg
                .as_ref()
                .expect("Status message should be set");

            let line_part = status.split(',').next().unwrap();
            let fraction_part = line_part.split(' ').nth(1).unwrap();

            let cur_line_str = fraction_part.split('/').next().unwrap();
            let reported_line: usize = cur_line_str.parse().unwrap();

            let total_lines_str = fraction_part.split('/').nth(1).unwrap();
            let _reported_total: usize = total_lines_str.parse().unwrap();

            assert_eq!(
                reported_line,
                line_idx + 1,
                "Mismatch at logical line {} (text: '{}'). Expected logical line {}, but got {}",
                line_idx,
                raw_text,
                line_idx + 1,
                reported_line
            );

            app.cursor_x = char_end;
            app.report_cursor_position();
            assert!(
                app.status_msg.is_some(),
                "report_cursor_position panicked or failed at the end of logical line {}",
                line_idx
            );
        }

        let coords: Vec<(usize, usize, usize)> = app
            .layout
            .iter()
            .filter(|r| !r.is_phantom)
            .flat_map(|row| {
                (row.char_start..=row.char_end).map(move |cx| (row.line_idx, cx, row.char_start))
            })
            .collect();

        let mut prev_char = 0usize;
        let mut prev_line = 0usize;

        for (line_idx, cx, _) in coords {
            app.cursor_y = line_idx;
            app.cursor_x = cx;
            app.report_cursor_position();

            let status = app.status_msg.as_ref().unwrap();
            let parts: Vec<&str> = status.split(", ").collect();

            let cur_line: usize = parts[0]
                .split('/')
                .next()
                .unwrap()
                .split_whitespace()
                .nth(1)
                .unwrap()
                .parse()
                .unwrap();
            let cur_char: usize = parts[2]
                .split('/')
                .next()
                .unwrap()
                .split_whitespace()
                .nth(1)
                .unwrap()
                .parse()
                .unwrap();

            assert!(
                cur_line >= prev_line,
                "line went backwards at y={} x={}: {} -> {}",
                line_idx,
                cx,
                prev_line,
                cur_line
            );
            assert!(
                cur_char >= prev_char,
                "char went backwards at y={} x={}: {} -> {}",
                line_idx,
                cx,
                prev_char,
                cur_char
            );

            prev_char = cur_char;
            prev_line = cur_line;
        }

        app.cursor_y = app
            .lines
            .iter()
            .position(|l| l.starts_with("INT. FLAT"))
            .unwrap();
        app.cursor_x = 0;
        app.update_layout();
        app.report_cursor_position();
        assert_eq!(
            app.status_msg.as_deref(),
            Some("line 8/93 (8%), col 1/31 (3%), char 126/4082 (3%)")
        );

        app.cursor_y = app
            .lines
            .iter()
            .position(|l| l.starts_with(">AN ABRUPT"))
            .unwrap();
        app.cursor_x = 0;
        app.update_layout();
        app.report_cursor_position();
        assert_eq!(
            app.status_msg.as_deref(),
            Some("line 67/93 (72%), col 1/41 (2%), char 2976/4082 (72%)")
        );

        app.cursor_y = app.lines.iter().position(|l| l == "> FADE OUT").unwrap();
        app.cursor_x = app.lines[app.cursor_y].chars().count();
        app.update_layout();
        app.report_cursor_position();
        assert_eq!(
            app.status_msg.as_deref(),
            Some("line 93/93 (100%), col 11/11 (100%), char 4082/4082 (100%)")
        );

        app.cursor_y = usize::MAX;
        app.update_layout();

        let render = crate::export::export_document(&app.layout, &app.lines, &app.config, &app.theme, false);

        let reference_render = r#"                      Title: Fount Tutorial
                      Credit: Written by
                      Author: René Coignard
                      Draft date: Version 0.2.17
                      Contact:
                        contact@renecoignard.com

     1      INT. FLAT IN WOLFEN-NORD - DAY                                    1.

            RENÉ sits at his desk, typing.

                                RENÉ
                            (turning round)
                       Oh, hello there. It seems you've
                       found my terminal Rust port of
                       Beat. Sit back and I'll show you
                       how everything works.

            I sometimes write screenplays on my Gentoo laptop, and doing
            it in plain nano isn't terribly comfortable (I work entirely
            in the terminal there). So I decided to put this port of
            Beat together. I used Beat's source code as a reference when
            writing Fount, so things work more or less the same way.

            As you may have already noticed, the navigation is rather
            reminiscent of nano, because I did look at its source code
            and took inspiration, for the sake of authenticity. I'm
            rather fond of it, and I hope you will be too. Not quite as
            nerdy as vim, but honestly, I'm an average nano enjoyer and
            I'm not ashamed of it.

            Anyway, let's get into it.

     2      EXT. NORDPARK - DAY

            As I mentioned, things work much the same as in Beat. If you
            start a line with int. or ext., Fount will automatically
            turn it into a scene heading. You can also use tab: on an
            empty line, it will first turn it into a character cue, then
            a scene heading, and then a transition. If you simply start
            typing IN CAPS ON AN EMPTY LINE, LIKE SO, the text will
            automatically become a character cue.

            You can also use notes...

                                SAILOR
                       I'm not a sailor, actually.

            Fount automatically inserts two blank lines after certain
            elements, just as Beat does, though this can be adjusted in
            the configuration file. There's a sample config in the
            repository; do make use of it. Bonus: try enabling
            typewriter mode and see what happens.

            To create a transition, simply write in capitals and end
            with a colon, like so...

                                                                 CUT TO:

            That alone is quite enough to write a proper screenplay. But
            there's more! For instance, we also have these...                 2.

     3      INT. EDEKA - ABEND

            Unlike Beat, there's no full render or PDF export here, but
            you can always save your screenplay and open it in Beat to
            do that. In Beat, synopses wouldn't appear in the rendered
            script, nor would comments. Which is why they share the same
            colour here, incidentally.

            As you may have noticed, there's support for bold text,
            italics, and even underlined text. When your cursor isn't on
            a line containing these markers, they'll be hidden from
            view. Move onto the line, and you'll see all the asterisks
            and underscores that produce the formatting.

            Centred text is supported as well, and works like this...

                                    Centred text

            You can also force transitions...

                                 AN ABRUPT TRANSITION TO THE NEXT SCENE:

     4      EXT. WOLFEN(BITTERFELD) RAILWAY STATION - MORNING

            Lyrics are supported too, using a tilde at the start of the
            line...

                          Meine Damen, meine Herrn, danke
                              Dass Sie mit uns reisen
                              Zu abgefahrenen Preisen
                              Auf abgefahrenen Gleisen
                   Für Ihre Leidensfähigkeit, danken wir spontan
                      Sänk ju for träweling wis Deutsche Bahn

            That's Wise Guys. Onwards.

     5      EXT. LEIPZIG HBF - MORNING

            Well, do have a go on it, write something from scratch, or
            edit this screenplay. You might even turn up a bug or two;
            if so, please do let me know :-) Everything seemed to behave
            itself while I was putting this tutorial together, and I
            hope it all runs just as smoothly for you. I hope you enjoy
            working in Fount.

            You can find more information about the Fountain markup
            language at https://www.fountain.io/                              3.

            And Beat itself, of course: https://www.beat-app.fi/

                                                                FADE OUT
"#;

        assert_eq!(
            render, reference_render,
            "Reference render does not match expected output."
        );
    }
