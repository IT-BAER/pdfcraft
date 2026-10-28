//! The reader profile (`PdfCraftApp::reader_profile`): only `pdfcraft_engine::reader`'s commands and
//! tool groups show anywhere, and the rest is refused when asked for directly.

use egui::{Key, Modifiers};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use pdfcraft_engine::{catalog, commands, reader};
use pdfcraft_ui_egui::{LeftPanel, PdfCraftApp, QuickTool};

/// Two pages and two bookmarks.
const FIXTURE: &[u8] = b"%PDF-1.7
1 0 obj << /Type /Catalog /Pages 2 0 R /Outlines 6 0 R >> endobj
2 0 obj << /Type /Pages /Kids [3 0 R 4 0 R] /Count 2 >> endobj
3 0 obj << /Type /Page /Parent 2 0 R /MediaBox [0 0 300 400] >> endobj
4 0 obj << /Type /Page /Parent 2 0 R /MediaBox [0 0 300 400] >> endobj
6 0 obj << /Type /Outlines /First 7 0 R /Last 8 0 R /Count 2 >> endobj
7 0 obj << /Title (Alpha section) /Parent 6 0 R /Next 8 0 R /Dest [3 0 R /Fit] >> endobj
8 0 obj << /Title (Beta section) /Parent 6 0 R /Prev 7 0 R /Dest [4 0 R /Fit] >> endobj
trailer << /Root 1 0 R >>
%%EOF";

const HIDDEN_GROUPS: [&str; 4] = ["edit", "redact", "scan", "ai"];

/// Tall enough that the longest menu (File) shows every entry without scrolling.
fn harness(profile: bool, doc: bool, setup: impl FnOnce(&mut PdfCraftApp) + 'static) -> Harness<'static, PdfCraftApp> {
    let mut h = Harness::builder().with_size(egui::vec2(1400.0, 3000.0)).build_eframe(move |_cc| {
        let mut app = PdfCraftApp::new();
        app.set_option("language", "en").unwrap();
        app.reader_profile = profile;
        if doc {
            app.open_bytes("fixture.pdf", None, FIXTURE.to_vec()).expect("fixture opens");
        }
        setup(&mut app);
        app
    });
    h.run_steps(4);
    h
}

fn label(group: &str) -> &'static str {
    catalog::group(group).unwrap_or_else(|| panic!("no tool group {group}")).label
}

fn shown(h: &Harness<'static, PdfCraftApp>, label: &str) -> bool {
    h.query_all_by_label(label).next().is_some()
}

/// Open each menu of the Menu button and return (hidden commands shown, allowed commands missing).
fn menu_bar(h: &mut Harness<'static, PdfCraftApp>) -> (Vec<&'static str>, Vec<&'static str>) {
    let (mut leaked, mut missing) = (Vec::new(), Vec::new());
    for menu in ["File", "Edit", "Pages", "View", "Help"] {
        h.get_by_label("Menu").click();
        h.run_steps(2);
        h.get_by_label(&format!("{menu} ⏵")).hover();
        h.run_steps(3);
        for spec in commands::menu(menu) {
            let allowed = reader::allows_command(spec.id);
            // Items with a shortcut carry it in their label.
            let listed = h.query_all_by_label_contains(spec.label).next().is_some();
            if !allowed && listed {
                leaked.push(spec.id);
            }
            if allowed && matches!(spec.id, "print.dialog" | "page.rotate" | "file.save_as" | "help.about") && !listed {
                missing.push(spec.id);
            }
        }
        for _ in 0..2 {
            h.key_press(Key::Escape);
            h.run_steps(2);
        }
    }
    (leaked, missing)
}

#[test]
fn the_menu_bar_lists_only_reader_commands() {
    let mut h = harness(true, true, |_| {});
    let (leaked, missing) = menu_bar(&mut h);
    assert!(leaked.is_empty(), "hidden commands in the menu bar: {leaked:?}");
    assert!(missing.is_empty(), "reader commands missing from the menu bar: {missing:?}");
}

#[test]
fn all_tools_and_home_offer_only_reader_tools() {
    let h = harness(true, false, |_| {});
    for g in HIDDEN_GROUPS.iter().chain(&["protect", "form"]) {
        assert!(!shown(&h, label(g)), "Home shows {g}");
    }
    assert!(shown(&h, label("organize")), "Home recommends Organize");
    let h = harness(true, false, |app| app.all_tools_full = true);
    for g in HIDDEN_GROUPS {
        assert!(!shown(&h, label(g)), "All tools shows {g}");
    }
    for g in ["fill_sign", "organize"] {
        assert!(shown(&h, label(g)), "All tools lacks {g}");
    }
}

#[test]
fn the_palette_finds_no_hidden_command_or_tool() {
    for query in ["word", "redact"] {
        let mut h = harness(true, true, |_| {});
        h.state_mut().set_option("palette", query).unwrap();
        h.run_steps(3);
        for spec in commands::COMMANDS.iter().filter(|c| !reader::allows_command(c.id)) {
            if spec.label.to_lowercase().contains(query) {
                assert!(!shown(&h, spec.label), "{query}: {}", spec.id);
            }
        }
        assert!(!shown(&h, label("redact")), "{query}");
    }
}

#[test]
fn hidden_commands_and_tools_are_refused() {
    let mut h = harness(true, true, |_| {});
    assert!(!h.state_mut().execute("export.docx"));
    h.run_steps(2);
    let app = h.state();
    assert!(app.dialog.is_none(), "no export dialog");
    assert_eq!(app.session.get(app.views[0].id).unwrap().can_undo(), None);
    assert!(h.state_mut().execute("page.rotate"));
    h.run_steps(2);
    let app = h.state();
    assert_eq!(app.session.get(app.views[0].id).unwrap().info.pages[0].rotation, 90);

    let before = h.state().left;
    h.state_mut().set_option("tool", "edit").unwrap();
    h.run_steps(2);
    assert_eq!(h.state().left, before, "Edit a PDF does not open");
    assert_ne!(h.state().left, LeftPanel::Tool("edit"));
}

/// `pdfcraft --dialog …` and `--quick …` (and the control channel) open only reader dialogs and tools.
#[test]
fn launch_options_open_no_hidden_dialog_or_tool() {
    for profile in [true, false] {
        let mut h = harness(profile, true, |_| {});
        let app = h.state_mut();
        assert_eq!(app.set_option("dialog", "protect").is_ok(), !profile);
        assert_eq!(app.dialog.is_some(), !profile, "Protect opens only without the profile");
        app.dialog = None;
        assert_eq!(app.set_option("quick", "redact").is_ok(), !profile);
        assert_eq!(app.quick_tool == QuickTool::Redact, !profile, "Redact is picked only without the profile");
        app.quick_tool = QuickTool::Select;
        assert_eq!(app.set_option("quick", "certify").is_ok(), !profile);
        app.set_option("quick", "highlight").unwrap();
        assert!(matches!(app.quick_tool, QuickTool::Comment(_)));
        app.set_option("dialog", "properties").unwrap();
        assert!(app.dialog.is_some());
    }
}

/// ⌘B is New bookmark, which the reader leaves out: the key does nothing and says nothing.
#[test]
fn a_hidden_commands_shortcut_does_nothing() {
    assert_eq!(commands::command("bookmark.add").and_then(|c| c.shortcut).map(|s| s.label(false)).as_deref(), Some("Ctrl+B"));
    let mut h = harness(true, true, |_| {});
    h.key_press_modifiers(Modifiers::COMMAND, Key::B);
    h.run_steps(3);
    assert!(h.query_all_by_value("Untitled").next().is_none(), "no new bookmark editor");
    assert!(h.state().toast.is_none(), "{:?}", h.state().toast);
}

#[test]
fn the_organize_panel_keeps_only_reader_page_actions() {
    let h = harness(true, true, |app| app.set_option("tool", "organize").unwrap());
    for l in ["Rotate clockwise", "Delete pages (Delete)", "Extract pages to a new document", "Insert", "Print pages"] {
        assert!(shown(&h, l), "{l}");
    }
    for l in ["Split", "Replace", "Crop pages", "Set page boxes", "Bates numbering", "Page labels"] {
        assert!(!shown(&h, l), "{l}");
    }
}

/// Strings the i18n report cannot see (a label passed through a variable, About credits) are German too.
#[test]
fn the_german_reader_shows_no_english_leftovers() {
    let h = harness(true, true, |app| {
        app.set_option("language", "de").unwrap();
        app.set_option("tool", "organize").unwrap();
    });
    assert!(shown(&h, "Seiten drucken"));
    assert!(!shown(&h, "Print pages"));
    let h = harness(true, true, |app| {
        app.set_option("language", "de").unwrap();
        app.dialog = Some(pdfcraft_ui_egui::Dialog::About);
    });
    h.get_by_label("Basiert auf PdfCraft vom ArtCraft-Team und der Community.");
    assert_eq!(h.query_all_by_label_contains("Darstellung: hayro").count(), 1);
}

/// The full app (the default) still shows everything the reader hides.
#[test]
fn without_the_profile_everything_shows() {
    let mut h = harness(false, true, |_| {});
    let (leaked, _) = menu_bar(&mut h);
    for id in ["export.docx", "create.file", "page.split"] {
        assert!(leaked.contains(&id), "{id} in the menu bar: {leaked:?}");
    }
    let h = harness(false, false, |app| app.all_tools_full = true);
    for g in HIDDEN_GROUPS {
        assert!(shown(&h, label(g)), "All tools lacks {g}");
    }
    let h = harness(false, true, |app| app.set_option("tool", "organize").unwrap());
    for l in ["Split", "Replace", "Crop pages"] {
        assert!(shown(&h, l), "{l}");
    }
    let mut h = harness(false, true, |_| {});
    h.state_mut().set_option("palette", "word").unwrap();
    h.run_steps(3);
    assert!(shown(&h, "Export to Word…"));
    let mut h = harness(false, true, |_| {});
    h.key_press_modifiers(Modifiers::COMMAND, Key::B);
    h.run_steps(3);
    assert!(h.query_all_by_value("Untitled").next().is_some(), "⌘B adds a bookmark");
}
