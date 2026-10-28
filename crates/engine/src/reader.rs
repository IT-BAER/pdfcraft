//! The reader profile: a plain PDF reader build. Only the commands and tool groups listed here are
//! offered; everything else stays in the code but is hidden and refused by the frontend.

/// Commands the reader profile offers, by [`crate::commands::CommandSpec::id`].
pub const COMMANDS: &[&str] = &[
    // File
    "file.open",
    "file.open_recent",
    "file.clear_recent",
    "file.pin_folder",
    "page.combine",
    "file.save",
    "file.save_as",
    "file.close",
    "file.close_all",
    "file.revert",
    "print.dialog",
    "file.properties",
    "protect.properties",
    // Edit
    "edit.undo",
    "edit.redo",
    "edit.find",
    "edit.advanced_search",
    "edit.copy",
    "edit.column_select",
    "edit.snapshot",
    "form.clear",
    "app.preferences",
    // View
    "view.palette",
    "view.layout.single",
    "view.layout.two_up",
    "view.continuous",
    "view.page_gaps",
    "view.layout.cover",
    "view.fit_width_scrolling",
    "view.fit_one_page",
    "view.fit_visible",
    "view.marquee_zoom",
    "view.full_screen",
    "view.read_mode",
    "view.next_tab",
    "view.previous_tab",
    "view.previous_view",
    "view.next_view",
    "view.focus_page_input",
    "ui.dock.reset",
    "view.theme",
    "view.theme.system",
    "view.theme.light",
    "view.theme.dark",
    "comment.list",
    "form.fields",
    "sign.panel",
    // Comments
    "comment.note",
    "comment.freetext",
    "comment.highlight",
    "comment.underline",
    "comment.strikeout",
    "comment.squiggly",
    "comment.ink",
    "comment.line",
    "comment.arrow",
    "comment.square",
    "comment.circle",
    "comment.polygon",
    "comment.polyline",
    "comment.cloud",
    "comment.callout",
    "comment.caret",
    "comment.replace",
    "comment.attach",
    "comment.eraser",
    "comment.stamp",
    "comment.hide_all",
    "comment.import",
    "comment.export",
    // Fill & Sign
    "sign.fill.text",
    "sign.fill.check",
    "sign.fill.cross",
    "sign.fill.dot",
    "sign.fill.line",
    "sign.fill.date",
    "sign.fill.signature",
    "sign.fill.initials",
    "sign.fill.signature.change",
    "sign.fill.signature.remove",
    "sign.fill.initials.remove",
    "sign.fill.initials.change",
    // Certificates
    "sign.digital",
    "sign.validate",
    // Measure
    "measure.distance",
    "measure.perimeter",
    "measure.area",
    "measure.scale",
    "measure.info",
    "measure.snap",
    "measure.export",
    // Pages
    "page.organize",
    "page.rotate",
    "page.rotate_ccw",
    "page.rotate_dialog",
    "page.delete",
    "page.extract",
    "page.insert",
    // Help
    "help.shortcuts",
    "help.about",
];

/// Tool groups the reader profile offers, by [`crate::catalog::ToolGroup::id`].
pub const TOOL_GROUPS: &[&str] = &["fill_sign", "comment", "stamp", "measure", "certificate", "organize", "combine"];

/// Whether the reader profile offers the command `id`.
pub fn allows_command(id: &str) -> bool {
    COMMANDS.contains(&id)
}

/// Whether the reader profile offers the tool group `id`.
pub fn allows_group(id: &str) -> bool {
    TOOL_GROUPS.contains(&id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn every_command_is_registered_once() {
        let mut seen = HashSet::new();
        for id in COMMANDS {
            assert!(crate::commands::command(id).is_some(), "{id} is not a registered command");
            assert!(seen.insert(*id), "{id} is listed twice");
        }
    }

    #[test]
    fn every_group_is_in_the_catalogue_once() {
        let mut seen = HashSet::new();
        for id in TOOL_GROUPS {
            assert!(crate::catalog::group(id).is_some(), "{id} is not a tool group");
            assert!(seen.insert(*id), "{id} is listed twice");
        }
    }

    #[test]
    fn pro_features_are_not_offered() {
        for id in ["export.docx", "edit.edit_text", "redact.mark", "page.split", "create.file", "ai.summary"] {
            assert!(!allows_command(id), "{id}");
        }
        for id in ["edit", "redact", "scan", "ai", "export"] {
            assert!(!allows_group(id), "{id}");
        }
        assert!(allows_command("page.rotate") && allows_group("organize"));
    }
}
