//! Preferences ▸ Open each PDF in a new window: once a window shows a document, files the user
//! opens go to another window (`PdfCraftApp::open_elsewhere`) instead of a new tab.

use crate::support;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use pdfcraft_ui_egui::{OsEvent, PdfCraftApp};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

const FIXTURE: &[u8] = b"%PDF-1.7
1 0 obj << /Type /Catalog /Pages 2 0 R >> endobj
2 0 obj << /Type /Pages /Kids [3 0 R] /Count 1 >> endobj
3 0 obj << /Type /Page /Parent 2 0 R /MediaBox [0 0 300 400] >> endobj
trailer << /Root 1 0 R >>
%%EOF";

/// `names` written as PDFs into a folder of their own.
fn files(test: &str, names: &[&str]) -> (PathBuf, Vec<String>) {
    let dir = std::env::temp_dir().join(format!("pdfcraft-new-window-{}-{test}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let paths = names
        .iter()
        .map(|n| {
            let p = dir.join(n);
            std::fs::write(&p, FIXTURE).unwrap();
            p.to_string_lossy().into_owned()
        })
        .collect();
    (dir, paths)
}

/// The files handed to other windows, and an `open_elsewhere` that records them and answers `started`.
fn recorder(started: bool) -> (Rc<RefCell<Vec<String>>>, pdfcraft_ui_egui::OpenElsewhereFn) {
    let log = Rc::new(RefCell::new(Vec::new()));
    let sink = Rc::clone(&log);
    (
        log,
        Box::new(move |p: &str| {
            sink.borrow_mut().push(p.to_string());
            started
        }),
    )
}

#[test]
fn a_window_showing_a_document_sends_the_next_file_to_a_new_window() {
    let (dir, f) = files("next", &["a.pdf", "b.pdf"]);
    let (started, elsewhere) = recorder(true);
    let mut app = PdfCraftApp::new();
    app.open_in_new_window = true;
    app.open_elsewhere = Some(elsewhere);
    // An empty window (Home) opens the first file itself.
    assert!(app.open_user_path(&f[0]));
    assert_eq!(app.views.len(), 1);
    assert!(started.borrow().is_empty());
    assert!(!app.open_user_path(&f[1]), "b.pdf opens in another window");
    assert_eq!(app.views.len(), 1, "no second tab");
    assert_eq!(*started.borrow(), [f[1].clone()]);
    // A file this window already shows comes to the front here.
    app.active = None;
    app.open_recent(&f[0]);
    assert_eq!(app.active, Some(0));
    assert_eq!(started.borrow().len(), 1);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn tabs_stay_when_the_preference_is_off_or_no_window_starts() {
    let (dir, f) = files("off", &["a.pdf", "b.pdf", "c.pdf"]);
    let (failed, cannot_start) = recorder(false);
    let mut app = PdfCraftApp::new();
    app.open_in_new_window = true;
    app.open_elsewhere = Some(cannot_start);
    app.open_user_path(&f[0]);
    assert!(app.open_user_path(&f[1]), "no window started: b.pdf opens as a tab");
    assert_eq!(app.views.len(), 2);
    assert_eq!(*failed.borrow(), [f[1].clone()]);
    let (started, elsewhere) = recorder(true);
    app.open_elsewhere = Some(elsewhere);
    app.open_in_new_window = false;
    assert!(app.open_user_path(&f[2]));
    assert_eq!(app.views.len(), 3);
    assert!(started.borrow().is_empty());
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn file_open_and_another_launch_go_to_new_windows() {
    let (dir, f) = files("open", &["a.pdf", "b.pdf", "c.pdf"]);
    let (started, elsewhere) = recorder(true);
    let (first, picked, launched) = (f[0].clone(), f[1].clone(), f[2].clone());
    let launch = Rc::new(RefCell::new(Some(vec![OsEvent::Open(vec![launched])])));
    let mut h = Harness::builder().with_size(egui::vec2(1200.0, 800.0)).build_eframe(move |_cc| {
        let mut app = PdfCraftApp::new();
        app.set_option("language", "en").unwrap();
        app.open_in_new_window = true;
        app.open_elsewhere = Some(elsewhere);
        app.open_path(&first);
        app.pick_override = Some(vec![picked]);
        app
    });
    h.run_steps(2);
    h.state_mut().run_command("file.open");
    let seen = Rc::clone(&started);
    assert!(support::step_until(&mut h, move |_| !seen.borrow().is_empty()), "File ▸ Open starts a window");
    let events = Rc::clone(&launch);
    h.state_mut().os_events = Some(Box::new(move || events.borrow_mut().take().unwrap_or_default()));
    let seen = Rc::clone(&started);
    assert!(support::step_until(&mut h, move |_| seen.borrow().len() == 2), "another launch's file starts a window");
    assert_eq!(*started.borrow(), [f[1].clone(), f[2].clone()]);
    assert_eq!(h.state().views.len(), 1, "no tabs were added");
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn the_preference_is_kept_in_the_settings() {
    let mut app = PdfCraftApp::new();
    assert!(!app.open_in_new_window, "off unless asked for");
    app.set_option("open-in-new-window", "on").unwrap();
    assert!(app.open_in_new_window);
    let mut later = PdfCraftApp::new();
    later.restore(&app.persist());
    assert!(later.open_in_new_window);
    later.set_option("open-in-new-window", "off").unwrap();
    let mut again = PdfCraftApp::new();
    again.open_in_new_window = true;
    again.restore(&later.persist());
    assert!(!again.open_in_new_window, "a saved off wins over the build's default");
    assert!(later.set_option("open-in-new-window", "maybe").is_err());
}

#[test]
fn preferences_offers_the_switch_in_german() {
    let mut h = Harness::builder().with_size(egui::vec2(1000.0, 900.0)).build_eframe(|_cc| {
        let mut app = PdfCraftApp::new();
        app.set_option("language", "de").unwrap();
        app.set_option("dialog", "preferences").unwrap();
        app
    });
    h.run_steps(6);
    let label = "Jedes PDF in einem neuen Fenster öffnen";
    h.get_by_label(label).scroll_to_me();
    h.run_steps(3);
    h.get_by_label(label).click();
    h.run_steps(3);
    assert!(h.state().open_in_new_window);
}
