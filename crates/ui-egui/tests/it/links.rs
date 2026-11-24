//! The SMS build has no links of its own: no community or project links on the home screen, in the
//! Help menu or in the About dialog, and no update check.

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use pdfcraft_ui_egui::{Dialog, PdfCraftApp};

fn harness(setup: impl FnOnce(&mut PdfCraftApp) + 'static) -> Harness<'static, PdfCraftApp> {
    let mut h = Harness::builder().with_size(egui::vec2(1400.0, 900.0)).build_eframe(move |_cc| {
        let mut app = PdfCraftApp::new();
        app.set_option("language", "en").unwrap();
        setup(&mut app);
        app
    });
    h.run_steps(4);
    h
}

#[test]
fn home_screen_has_no_links_or_marketing() {
    let h = harness(|_| {});
    for label in [
        "Discord",
        "Join the ArtCraft community",
        "Join our Discord",
        "PdfCraft on GitHub",
        "PdfCraft web page",
        "Check for updates…",
        "Privacy",
        "An open-source PDF workbench — local, private, and scriptable.",
    ] {
        assert!(h.query_by_label(label).is_none(), "{label}");
    }
    assert_eq!(h.query_all_by_label_contains("GitHub").count(), 0);
    assert_eq!(h.query_all_by_label_contains("web page").count(), 0);
    assert_eq!(h.query_all_by_label_contains("telemetry").count(), 0);
    h.get_by_label("Welcome to PdfCraft");
}

#[test]
fn help_menu_is_shortcuts_and_about_only() {
    let ids: Vec<&str> = pdfcraft_engine::commands::menu("Help").map(|c| c.id).collect();
    assert_eq!(ids, ["help.shortcuts", "help.about"]);
}

#[test]
fn about_dialog_has_a_license_line_and_a_plain_upstream_credit() {
    let h = harness(|app| app.dialog = Some(Dialog::About));
    h.get_by_label("Licensed under MIT or Apache-2.0.");
    h.get_by_label("Based on PdfCraft by the ArtCraft team and community.");
    assert_eq!(h.query_all_by_label("ArtCraft").count(), 0, "no ArtCraft mark or wordmark");
    for label in ["Contributors", "Models", "PdfCraft on GitHub", "PdfCraft web page"] {
        assert!(h.query_by_label(label).is_none(), "{label}");
    }
}

#[test]
fn help_commands_open_no_web_page() {
    for id in ["help.shortcuts", "help.about"] {
        let mut h = harness(|_| {});
        assert!(h.state_mut().execute(id), "{id}");
        assert_eq!(h.state().last_opened_url, None, "{id}");
    }
    for id in ["help.app_page", "help.github", "help.check_updates"] {
        assert!(pdfcraft_engine::commands::command(id).is_none(), "{id} is gone");
        let mut h = harness(|_| {});
        assert!(!h.state_mut().execute(id), "{id}");
    }
}
