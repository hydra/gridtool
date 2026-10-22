//! Sections stacked top to bottom, each taking a fraction of the available height, with
//! a draggable divider between neighbours. Dragging a divider moves height between the
//! two sections it separates; the others keep theirs.

use eframe::egui::{self, CursorIcon, Rect, Sense, Stroke, UiBuilder};

/// Height of each divider's grab area, in screen points.
const DIVIDER_HEIGHT: f32 = 6.0;
/// The smallest a section can be dragged to, in screen points.
const MIN_SECTION_HEIGHT: f32 = 40.0;

#[derive(Debug, Clone, PartialEq)]
pub struct VerticalSplit {
    /// Each section's share of the height available to sections; they add up to 1.
    fractions: Vec<f32>,
}

impl VerticalSplit {
    /// Sections with the given shares of the height, scaled so that they add up to 1.
    pub fn new(fractions: &[f32]) -> Self {
        let total: f32 = fractions.iter().sum();
        Self {
            fractions: fractions.iter().map(|fraction| fraction / total).collect(),
        }
    }

    /// Each section's share of the height.
    #[cfg(test)]
    pub fn fractions(&self) -> &[f32] {
        &self.fractions
    }

    /// Moves the divider below section `index` down by `delta` (a share of the height;
    /// negative moves it up). The section above grows and the one below shrinks by the
    /// same amount, and neither becomes smaller than `min`.
    pub fn move_divider(&mut self, index: usize, delta: f32, min: f32) {
        let (Some(&above), Some(&below)) =
            (self.fractions.get(index), self.fractions.get(index + 1))
        else {
            return;
        };
        let min = min.min((above + below) / 2.0);
        let delta = delta.clamp(min - above, below - min);
        self.fractions[index] = above + delta;
        self.fractions[index + 1] = below - delta;
    }

    /// The sections' rectangles within `area`, top to bottom, leaving room for the
    /// dividers between them.
    pub fn section_rects(&self, area: Rect) -> Vec<Rect> {
        let dividers = self.fractions.len().saturating_sub(1) as f32 * DIVIDER_HEIGHT;
        let height = (area.height() - dividers).max(0.0);
        let mut top = area.top();
        self.fractions
            .iter()
            .map(|fraction| {
                let rect = Rect::from_x_y_ranges(area.x_range(), top..=top + fraction * height);
                top = rect.bottom() + DIVIDER_HEIGHT;
                rect
            })
            .collect()
    }

    /// Lays the sections out in the rest of `ui`, handling divider drags, and calls
    /// `show_section` with each section's index and a `ui` confined to it.
    pub fn show<R>(
        &mut self,
        ui: &mut egui::Ui,
        id_salt: &str,
        mut show_section: impl FnMut(usize, &mut egui::Ui) -> R,
    ) -> Vec<R> {
        let area = ui.available_rect_before_wrap();
        let rects = self.section_rects(area);
        let section_height = rects.iter().map(Rect::height).sum::<f32>();

        for (index, pair) in rects.windows(2).enumerate() {
            let divider = Rect::from_x_y_ranges(area.x_range(), pair[0].bottom()..=pair[1].top());
            let response = ui.interact(
                divider,
                ui.scope_id().with((id_salt, "divider", index)),
                Sense::drag(),
            );
            if response.dragged() && section_height > 0.0 {
                let delta = response.drag_delta().y / section_height;
                self.move_divider(index, delta, MIN_SECTION_HEIGHT / section_height);
            }
            if response.hovered() || response.dragged() {
                ui.set_cursor_icon(CursorIcon::ResizeVertical);
            }
            let stroke = if response.hovered() || response.dragged() {
                ui.visuals().widgets.active.bg_stroke
            } else {
                ui.visuals().widgets.noninteractive.bg_stroke
            };
            let y = divider.center().y;
            ui.painter().hline(
                divider.x_range(),
                y,
                Stroke::new(stroke.width.max(1.0), stroke.color),
            );
        }

        let results = rects
            .iter()
            .enumerate()
            .map(|(index, &rect)| {
                ui.scope_builder(UiBuilder::new().max_rect(rect), |ui| {
                    ui.set_clip_rect(rect.intersect(ui.clip_rect()));
                    show_section(index, ui)
                })
                .inner
            })
            .collect();
        ui.allocate_rect(area, Sense::hover());
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::Pos2;

    fn assert_fractions(split: &VerticalSplit, expected: &[f32]) {
        for (actual, expected) in split.fractions().iter().zip(expected) {
            assert!((actual - expected).abs() < 1e-6, "{:?}", split.fractions());
        }
    }

    #[test]
    fn fractions_are_scaled_to_add_up_to_one() {
        assert_fractions(&VerticalSplit::new(&[1.0, 1.0, 2.0]), &[0.25, 0.25, 0.5]);
        assert_fractions(
            &VerticalSplit::new(&[0.33, 0.34, 0.33]),
            &[0.33, 0.34, 0.33],
        );
    }

    #[test]
    fn a_divider_moves_height_between_its_two_sections_only() {
        let mut split = VerticalSplit::new(&[0.33, 0.34, 0.33]);

        split.move_divider(1, 0.1, 0.05);
        assert_fractions(&split, &[0.33, 0.44, 0.23]);

        split.move_divider(0, -0.13, 0.05);
        assert_fractions(&split, &[0.2, 0.57, 0.23]);
    }

    #[test]
    fn sections_do_not_shrink_below_the_minimum() {
        let mut split = VerticalSplit::new(&[0.33, 0.34, 0.33]);

        split.move_divider(0, -1.0, 0.05);
        assert_fractions(&split, &[0.05, 0.62, 0.33]);

        split.move_divider(1, 1.0, 0.05);
        assert_fractions(&split, &[0.05, 0.9, 0.05]);

        // A missing divider changes nothing.
        split.move_divider(2, 0.1, 0.05);
        assert_fractions(&split, &[0.05, 0.9, 0.05]);
    }

    #[test]
    fn sections_share_the_height_left_after_the_dividers() {
        let split = VerticalSplit::new(&[0.25, 0.25, 0.5]);
        let area = Rect::from_min_max(Pos2::new(0.0, 100.0), Pos2::new(50.0, 512.0));

        let rects = split.section_rects(area);

        // 412 - 2 dividers of 6 = 400 points for sections.
        let heights: Vec<f32> = rects.iter().map(Rect::height).collect();
        assert_eq!(heights, [100.0, 100.0, 200.0]);
        assert_eq!(rects[0].top(), 100.0);
        assert_eq!(rects[1].top(), 206.0);
        assert_eq!(rects[2].bottom(), 512.0);
        assert!(rects.iter().all(|rect| rect.width() == 50.0));
    }
}
