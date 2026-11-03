//! The Print dialog's printers on Windows: the names the spooler keeps (`EnumPrinters`, level 4)
//! and the default printer, without asking any driver. craft-print's list also reads each driver's
//! PrintCapabilities for two-sided support first, which takes seconds per printer and about 48 s
//! for one that can't be reached (storytold/pdfcraft#2192). Jobs are still checked when sent: the
//! driver's ticket refuses a two-sided job it can't print.

use pdfcraft_engine::print::spool::{self, Printer};
use pdfcraft_ui_egui::PrinterSource;

pub fn source() -> PrinterSource {
    std::sync::Arc::new(|| match spooler_names() {
        Ok(names) => printer_list(names, default_printer().as_deref()),
        Err(e) => {
            log::warn!("printers: EnumPrinters failed ({e}); reading every driver instead");
            spool::printers()
        }
    })
}

fn spooler_names() -> winsafe::SysResult<Vec<String>> {
    use winsafe::co::PRINTER_ENUM;
    Ok(winsafe::EnumPrinters4(PRINTER_ENUM::LOCAL | PRINTER_ENUM::CONNECTIONS)?.iter().filter_map(|p| p.pPrinterName()).collect())
}

/// The user's default printer; `None` when there is none.
fn default_printer() -> Option<String> {
    winsafe::GetDefaultPrinter().ok().filter(|n| !n.is_empty())
}

/// Sorted by name and each once, as craft-print lists them; two-sided support unknown.
fn printer_list(names: impl IntoIterator<Item = String>, default: Option<&str>) -> Vec<Printer> {
    let mut names: Vec<String> = names.into_iter().filter(|n| !n.trim().is_empty()).collect();
    names.sort();
    names.dedup();
    names.into_iter().map(|name| Printer { default: default == Some(name.as_str()), name, description: String::new(), two_sided: None }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_are_sorted_listed_once_and_the_default_is_marked() {
        let names = ["uniFLOW", r"\\srv\Copier", "", "  ", "Microsoft Print to PDF", "uniFLOW"].map(String::from);
        let list = printer_list(names, Some("uniFLOW"));
        let shown: Vec<(&str, bool)> = list.iter().map(|p| (p.name.as_str(), p.default)).collect();
        assert_eq!(shown, [("Microsoft Print to PDF", false), (r"\\srv\Copier", false), ("uniFLOW", true)]);
        assert!(list.iter().all(|p| p.two_sided.is_none()));
        assert!(printer_list(["A".to_string()], Some("Gone")).iter().all(|p| !p.default), "a default that isn't listed marks nothing");
        assert!(printer_list(Vec::new(), None).is_empty());
    }

    /// The same printers as craft-print's full read, with no driver asked about two-sided support.
    #[test]
    fn the_spooler_names_come_without_reading_any_driver() {
        let fast = source()();
        assert!(fast.iter().all(|p| p.two_sided.is_none()), "{fast:?}");
        let names = |list: &[Printer]| list.iter().map(|p| (p.name.clone(), p.default)).collect::<Vec<_>>();
        assert_eq!(names(&fast), names(&spool::printers()));
    }
}
