//! The four editable coordinate columns (x1, y1, x2, y2) of a table of lines, shared by the
//! Point-to-point and Routes tables: how wide their fields are, and which axis each edits.
//!
//! The fields are at least wide enough for the widest coordinate. Any width the table leaves
//! unused in its section is shared equally between them, so they grow and shrink with the
//! panel, and once the panel stops changing width they settle and stop changing too.

use eframe::egui;
use gridtool::geometry::Point;

/// The widest coordinate a field must show in full: six digits and a decimal point.
const WIDEST_COORDINATE: &str = "0000.00";
/// One coordinate column per axis of each end of a line.
pub const COORDINATE_COLUMNS: usize = 4;
/// Spare (or missing) table width smaller than this, in points, is left alone, so that
/// pixel rounding cannot keep the fields changing width.
const WIDTH_TOLERANCE: f32 = 2.0;

/// The headers of the coordinate columns, in order.
pub const HEADERS: [&str; COORDINATE_COLUMNS] = ["x1", "y1", "x2", "y2"];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Axis {
    X,
    Y,
}

impl Axis {
    pub const ALL: [Axis; 2] = [Axis::X, Axis::Y];

    pub fn of(self, point: Point) -> f64 {
        match self {
            Axis::X => point.x,
            Axis::Y => point.y,
        }
    }

    pub fn with(self, point: Point, value: f64) -> Point {
        match self {
            Axis::X => Point::new(value, point.y),
            Axis::Y => Point::new(point.x, value),
        }
    }
}

/// How wide the table was laid out last frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TableLayout {
    field_width: f32,
    table_width: f32,
    /// Whether the frame before used the same field width. The grid places its columns
    /// by the widths of the frame before, so only then does `table_width` reflect
    /// `field_width`.
    settled: bool,
}

/// The widths of a table's coordinate fields, from one frame to the next.
#[derive(Debug, Default)]
pub struct CoordinateColumns {
    layout: Option<TableLayout>,
}

impl CoordinateColumns {
    /// The width of each coordinate field this frame, for a table about to be laid out in
    /// `ui`.
    pub fn field_width(&self, ui: &egui::Ui) -> f32 {
        coordinate_field_width(self.layout, ui.available_width(), min_field_width(ui))
    }

    /// Records that the table came out `table_width` wide with fields `field_width` wide.
    pub fn laid_out(&mut self, field_width: f32, table_width: f32) {
        let settled = self
            .layout
            .is_some_and(|last| last.field_width == field_width);
        self.layout = Some(TableLayout {
            field_width,
            table_width,
            settled,
        });
    }

    #[cfg(test)]
    pub fn layout(&self) -> Option<TableLayout> {
        self.layout
    }

    #[cfg(test)]
    pub fn field_width_of(layout: TableLayout) -> f32 {
        layout.field_width
    }
}

/// The narrowest a coordinate field may be: wide enough to show [`WIDEST_COORDINATE`]
/// inside the field's padding.
pub fn min_field_width(ui: &egui::Ui) -> f32 {
    let font = egui::TextStyle::Body.resolve(ui.style());
    let text = ui
        .painter()
        .layout_no_wrap(
            WIDEST_COORDINATE.to_owned(),
            font,
            egui::Color32::PLACEHOLDER,
        )
        .size()
        .x;
    // The theme pads a text field by the button padding on each side; allow for the
    // cursor too.
    text + ui.spacing().button_padding.x * 2.0 + 2.0
}

/// The width of each coordinate field this frame. Before the first layout, fields start at
/// `min_width`. Once the last layout has settled, the space the table left unused (or
/// overran by) is shared equally between the coordinate columns, but no field is narrower
/// than `min_width`. While it has not settled, or the difference is within
/// [`WIDTH_TOLERANCE`], the width is kept, so the fields never oscillate.
fn coordinate_field_width(last: Option<TableLayout>, available: f32, min_width: f32) -> f32 {
    let Some(last) = last else {
        return min_width;
    };
    let spare = available - last.table_width;
    if !last.settled || spare.abs() < WIDTH_TOLERANCE {
        return last.field_width.max(min_width);
    }
    (last.field_width + spare / COORDINATE_COLUMNS as f32).max(min_width)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coordinate_fields_start_at_the_minimum_width() {
        assert_eq!(coordinate_field_width(None, 500.0, 60.0), 60.0);
    }

    #[test]
    fn coordinate_fields_share_spare_width_and_shrink_back_to_the_minimum() {
        let last = TableLayout {
            field_width: 60.0,
            table_width: 400.0,
            settled: true,
        };

        // 100 points spare, shared by four columns.
        assert_eq!(coordinate_field_width(Some(last), 500.0, 60.0), 85.0);
        // 40 points too wide: each field gives up 10.
        let wide = TableLayout {
            field_width: 85.0,
            table_width: 500.0,
            settled: true,
        };
        assert_eq!(coordinate_field_width(Some(wide), 460.0, 60.0), 75.0);
        // Never narrower than the minimum, even if the table overflows.
        assert_eq!(coordinate_field_width(Some(wide), 200.0, 60.0), 60.0);
    }

    #[test]
    fn coordinate_fields_keep_their_width_until_the_layout_settles_or_within_tolerance() {
        let unsettled = TableLayout {
            field_width: 60.0,
            table_width: 400.0,
            settled: false,
        };
        assert_eq!(coordinate_field_width(Some(unsettled), 500.0, 60.0), 60.0);

        let settled = TableLayout {
            settled: true,
            ..unsettled
        };
        assert_eq!(coordinate_field_width(Some(settled), 401.5, 60.0), 60.0);
        assert_eq!(coordinate_field_width(Some(settled), 398.5, 60.0), 60.0);
    }

    /// Models the grid placing its columns by the previous frame's widths, as egui does.
    #[test]
    fn coordinate_fields_settle_without_oscillating_when_the_grid_lags_a_frame() {
        const OTHER_COLUMNS: f32 = 200.0;
        const AVAILABLE: f32 = 523.0;
        let mut columns = CoordinateColumns::default();
        let mut laid_out_width = 60.0;
        let mut widths = Vec::new();
        for _ in 0..12 {
            let field_width = coordinate_field_width(columns.layout, AVAILABLE, 60.0);
            let table_width = OTHER_COLUMNS + COORDINATE_COLUMNS as f32 * laid_out_width;
            columns.laid_out(field_width, table_width);
            laid_out_width = field_width;
            widths.push(field_width);
        }

        let target = (AVAILABLE - OTHER_COLUMNS) / COORDINATE_COLUMNS as f32;
        assert!(
            widths[4..].iter().all(|width| *width == target),
            "{widths:?}"
        );
    }

    #[test]
    fn the_minimum_field_width_fits_the_widest_coordinate() {
        let ctx = egui::Context::default();
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            let font = egui::TextStyle::Body.resolve(ui.style());
            let text_width = ui
                .painter()
                .layout_no_wrap("9999.99".to_owned(), font, egui::Color32::WHITE)
                .size()
                .x;
            let padding = ui.spacing().button_padding.x * 2.0;
            assert!(min_field_width(ui) >= text_width + padding);
        });
        output.textures_delta.clear();
    }

    #[test]
    fn an_axis_reads_and_replaces_its_own_coordinate() {
        let point = Point::new(3.0, 4.0);

        assert_eq!(Axis::X.of(point), 3.0);
        assert_eq!(Axis::Y.with(point, 9.0), Point::new(3.0, 9.0));
    }
}
