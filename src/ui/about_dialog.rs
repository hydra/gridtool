//! The About dialog. What it says comes from where it is already written: the authors and
//! repository from `Cargo.toml`, and the description from the README, by way of the build script.

use eframe::egui::{self, ColorImage, RichText, TextureHandle, TextureOptions};

use super::dialog::DialogButton;

const ABOUT: &str = include_str!(concat!(env!("OUT_DIR"), "/about.txt"));
const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
const WIDTH: f32 = 420.0;
const ICON_SIZE: f32 = 96.0;
const NAME_SIZE: f32 = 32.0;
const SPACING: f32 = 12.0;

pub struct AboutDialog {
    icon: TextureHandle,
    /// The OK button has been given the keyboard focus, which happens once.
    focused: bool,
}

impl AboutDialog {
    pub fn new(ctx: &egui::Context) -> Self {
        let icon = crate::app_icon();
        let image =
            ColorImage::from_rgba_unmultiplied([icon.width as _, icon.height as _], &icon.rgba);
        // Shown far smaller than it is stored, so it needs mipmaps to stay smooth.
        let options = TextureOptions::LINEAR.with_mipmap_mode(Some(egui::TextureFilter::Linear));
        Self {
            icon: ctx.load_texture("about_icon", image, options),
            focused: false,
        }
    }

    /// Shows the dialog over everything else, returning whether it has been closed.
    pub fn show(&mut self, ctx: &egui::Context) -> bool {
        let modal = egui::Modal::new(egui::Id::unique("about_dialog")).show(ctx, |ui| {
            ui.set_width(WIDTH);
            ui.vertical_centered(|ui| {
                ui.add(
                    egui::Image::new(&self.icon)
                        .fit_to_exact_size(egui::Vec2::splat(ICON_SIZE))
                        .alt_text(format!("{} icon", crate::APP_NAME)),
                );
                ui.add_space(SPACING);
                ui.label(RichText::new(crate::APP_NAME).size(NAME_SIZE).strong());
                ui.label(primary_author(AUTHORS));
                ui.add_space(SPACING);
                ui.label(ABOUT);
                ui.add_space(SPACING);
                ui.hyperlink(crate::REPOSITORY);
                ui.add_space(SPACING);
                let ok = ui.button(DialogButton::Ok.label());
                if !self.focused {
                    ok.request_focus();
                    self.focused = true;
                }
                ok.clicked()
            })
            .inner
        });
        modal.inner || modal.should_close()
    }
}

/// The first of Cargo's authors, which it separates with colons.
fn primary_author(authors: &str) -> &str {
    authors.split(':').next().unwrap_or(authors).trim()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::readme_about::about_text;
    use eframe::egui::output::OutputCommand;
    use eframe::egui::{Event, FullOutput, Key, Modifiers, PointerButton, Pos2, RawInput, Rect};

    const README: &str = include_str!("../../README.md");
    const WINDOW: Rect = Rect::from_min_max(Pos2::ZERO, Pos2::new(1280.0, 800.0));

    struct Harness {
        ctx: egui::Context,
        dialog: AboutDialog,
    }

    impl Harness {
        fn new() -> Self {
            let ctx = egui::Context::default();
            let dialog = AboutDialog::new(&ctx);
            let mut harness = Self { ctx, dialog };
            for _ in 0..3 {
                harness.frame(Vec::new());
            }
            harness
        }

        /// One frame with `events`: whether the dialog closed, and what it output.
        fn frame(&mut self, events: Vec<Event>) -> (bool, FullOutput) {
            let input = RawInput {
                screen_rect: Some(WINDOW),
                events,
                ..Default::default()
            };
            let mut closed = false;
            let ctx = self.ctx.clone();
            let mut output = ctx.run_ui(input, |ui| closed = self.dialog.show(ui.ctx()));
            // There is no renderer to upload textures to.
            output.textures_delta.clear();
            (closed, output)
        }

        /// The text shapes of a frame, as (text, its rectangle, how it is laid out).
        fn texts(&mut self) -> Vec<(String, Rect, std::sync::Arc<egui::Galley>)> {
            let (_, output) = self.frame(Vec::new());
            output
                .shapes
                .iter()
                .filter_map(|clipped| match &clipped.shape {
                    egui::Shape::Text(text) => Some((
                        text.galley.job.text.clone(),
                        text.visual_bounding_rect(),
                        text.galley.clone(),
                    )),
                    _ => None,
                })
                .collect()
        }

        fn position_of(&mut self, wanted: &str) -> Rect {
            let texts = self.texts();
            texts
                .iter()
                .find(|(text, ..)| text == wanted)
                .unwrap_or_else(|| panic!("{wanted} is not shown"))
                .1
        }

        /// Clicks at `pos`, returning whether the dialog closed and the commands it output.
        fn click(&mut self, pos: Pos2) -> (bool, Vec<OutputCommand>) {
            self.frame(vec![Event::PointerMoved(pos)]);
            let button = |pressed| Event::PointerButton {
                pos,
                button: PointerButton::Primary,
                pressed,
                modifiers: Modifiers::NONE,
            };
            self.frame(vec![button(true)]);
            let (closed, output) = self.frame(vec![button(false)]);
            (closed, output.platform_output.commands)
        }
    }

    #[test]
    fn the_about_text_is_the_text_of_the_readmes_section() {
        let about = ABOUT;

        assert_eq!(about, about_text(README));
        assert!(
            about.starts_with("GridTool is a desktop application for measuring real-world"),
            "{about}"
        );
        assert!(
            about.ends_with("documents can be saved and loaded."),
            "{about}"
        );
        assert!(about.contains("from an image, such as a plan"), "{about}");
        assert!(!about.contains('<'), "no screenshot in {about}");
        assert!(!about.contains('\n'), "one paragraph in {about}");
    }

    #[test]
    fn the_primary_author_is_the_first() {
        assert_eq!(primary_author("Ann Lee:Bo Ng"), "Ann Lee");
        assert_eq!(primary_author("Ann Lee"), "Ann Lee");
        assert_eq!(primary_author(AUTHORS), "Dominic Clifton");
    }

    #[test]
    fn cargo_lists_every_author_in_the_readme() {
        let names: Vec<&str> = README
            .lines()
            .skip_while(|line| line.trim() != "## Authors")
            .filter_map(|line| line.strip_prefix("* "))
            .filter_map(|line| line.split(" - ").next())
            .collect();

        assert!(!names.is_empty());
        let authors: Vec<&str> = AUTHORS.split(':').collect();
        assert_eq!(authors, names);
    }

    #[test]
    fn it_shows_everything_centred_in_order() {
        let mut harness = Harness::new();
        let in_order = [
            "GridTool",
            "Dominic Clifton",
            ABOUT,
            crate::REPOSITORY,
            "OK",
        ]
        .map(|text| harness.position_of(text));

        for pair in in_order.windows(2) {
            assert!(pair[0].bottom() <= pair[1].top(), "{pair:?}");
        }
        let centre = in_order[0].center().x;
        for rect in in_order {
            assert!((rect.center().x - centre).abs() < 2.0, "{rect:?}");
        }
        assert!(
            in_order[0].height() > in_order[1].height() * 1.5,
            "the name is large"
        );
    }

    #[test]
    fn the_about_text_wraps_with_every_row_centred() {
        let mut harness = Harness::new();
        let texts = harness.texts();
        let (.., galley) = texts
            .iter()
            .find(|(text, ..)| text == ABOUT)
            .expect("the text is shown");

        assert!(galley.rows.len() > 1);
        let centres: Vec<f32> = galley
            .rows
            .iter()
            .map(|row| row.rect().center().x)
            .collect();
        for centre in &centres {
            assert!((centre - centres[0]).abs() < 1.0, "{centres:?}");
        }
    }

    #[test]
    fn the_icon_is_shown_centred_above_the_name() {
        let mut harness = Harness::new();
        let name = harness.position_of("GridTool");
        let (_, output) = harness.frame(Vec::new());

        let icon = output
            .shapes
            .iter()
            .find_map(|clipped| match &clipped.shape {
                egui::Shape::Rect(rect) if rect.fill_texture_id() == harness.dialog.icon.id() => {
                    Some(rect.rect)
                }
                _ => None,
            })
            .expect("the icon is drawn");

        assert_eq!(icon.size(), egui::Vec2::splat(ICON_SIZE));
        assert!((icon.center().x - name.center().x).abs() < 2.0);
        assert!(icon.bottom() <= name.top());
    }

    #[test]
    fn clicking_the_link_opens_the_repository() {
        let mut harness = Harness::new();
        let link = harness.position_of(crate::REPOSITORY).center();

        let (closed, commands) = harness.click(link);

        assert!(!closed);
        assert!(
            commands.iter().any(|command| matches!(
                command,
                OutputCommand::OpenUrl(open) if open.url == "https://github.com/hydra/gridtool"
            )),
            "{commands:?}"
        );
    }

    #[test]
    fn ok_closes_it() {
        let mut harness = Harness::new();
        let ok = harness.position_of("OK").center();

        let (closed, _) = harness.click(ok);

        assert!(closed);
    }

    #[test]
    fn enter_and_escape_close_it() {
        for key in [Key::Enter, Key::Escape] {
            let mut harness = Harness::new();
            let press = |pressed| Event::Key {
                key,
                physical_key: None,
                pressed,
                repeat: false,
                modifiers: Modifiers::NONE,
            };

            let (closed, _) = harness.frame(vec![press(true), press(false)]);

            assert!(closed, "{key:?}");
        }
    }

    #[test]
    fn clicking_on_it_elsewhere_leaves_it_open() {
        let mut harness = Harness::new();
        let name = harness.position_of("GridTool").center();

        let (closed, _) = harness.click(name);

        assert!(!closed);
    }
}
