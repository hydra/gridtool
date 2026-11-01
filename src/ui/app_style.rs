//! The look of the whole application, set once at start-up.

use eframe::egui::{self, style::ScrollStyle};

/// Applies the application's style to every theme of `ctx`. Scroll bars take up space and are
/// always drawn when there is more to scroll to, rather than appearing only on hover, so that
/// it can be seen whether something can be scrolled.
pub fn apply(ctx: &egui::Context) {
    ctx.all_styles_mut(|style| style.spacing.scroll = ScrollStyle::solid());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scroll_bars_do_not_float_in_any_theme() {
        let ctx = egui::Context::default();

        apply(&ctx);

        for theme in [egui::Theme::Dark, egui::Theme::Light] {
            assert!(!ctx.style_of(theme).spacing.scroll.floating, "{theme:?}");
        }
    }
}
