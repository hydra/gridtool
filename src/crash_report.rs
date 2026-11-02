//! What happens when the application panics: `human-panic` saves a crash report, and a dialog
//! says where, since a windowed application has no console to say it on.

use std::panic;
use std::path::Path;

use gridtool::tr;
use human_panic::{Metadata, PanicStyle, handle_dump, print_msg};
use rfd::{MessageButtons, MessageDialog, MessageLevel};

/// Makes panics save a crash report and tell the user. A debug build, or one run with
/// `RUST_BACKTRACE` set, keeps the usual panic message for whoever is developing it.
pub fn install() {
    if PanicStyle::default() != PanicStyle::Human {
        return;
    }
    let metadata =
        Metadata::new(crate::APP_NAME, env!("CARGO_PKG_VERSION")).repository(crate::REPOSITORY);
    panic::set_hook(Box::new(move |info| {
        // First, so that the report is safe whatever goes wrong with telling the user.
        let report = handle_dump(&metadata, info);
        // There may be no console, which is not a problem.
        let _ = print_msg(report.as_deref(), &metadata);
        MessageDialog::new()
            .set_level(MessageLevel::Error)
            .set_title(tr!("crash-title", { app: crate::APP_NAME }))
            .set_description(crash_message(report.as_deref()))
            .set_buttons(MessageButtons::Ok)
            .show();
    }));
}

/// What the dialog says, given where the crash report was saved, if it could be.
fn crash_message(report: Option<&Path>) -> String {
    let closing = tr!("crash-closing", { app: crate::APP_NAME });
    match report {
        Some(report) => format!(
            "{closing}\n\n{}\n{}\n\n{}",
            tr!("crash-report-saved"),
            report.display(),
            tr!("crash-report-help", { url: crate::REPOSITORY })
        ),
        None => format!("{closing}\n\n{}", tr!("crash-no-report")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_message_says_where_the_report_is_and_where_to_send_it() {
        let message = crash_message(Some(Path::new("reports/report-1.toml")));

        assert!(message.contains("GridTool had a problem"), "{message}");
        assert!(message.contains("reports/report-1.toml"), "{message}");
        assert!(
            message.contains("https://github.com/hydra/gridtool/issues"),
            "{message}"
        );
    }

    #[test]
    fn the_message_says_so_if_no_report_could_be_saved() {
        let message = crash_message(None);

        assert!(message.contains("could not be saved"), "{message}");
        assert!(!message.contains("/issues"), "{message}");
    }
}
