//! Whether and how a grid is drawn at the current zoom. Only the lines inside the visible
//! part of the canvas are generated, so arbitrarily small cells can be drawn when zoomed
//! in far enough.

use std::fmt;

use eframe::egui::{self, Stroke};
use gridtool::document::Document;
use gridtool::grid::{Grid, GridError};

use super::color::with_opacity;
use super::selection::Selection;
use super::viewport::DocumentTransform;

/// Grids whose neighbouring lines would be closer than this on screen are not drawn:
/// they would fill the view with a solid colour.
pub const MIN_SCREEN_SPACING: f32 = 3.0;
const GRID_LINE_WIDTH: f32 = 1.0;

/// Why a grid is not drawn.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GridHidden {
    /// The grid cannot be laid out at all.
    Invalid(GridError),
    /// The cells are too small to see at the current zoom.
    TooSmallAtThisZoom,
}

impl fmt::Display for GridHidden {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid(error) => error.fmt(f),
            Self::TooSmallAtThisZoom => write!(
                f,
                "the cells are under {MIN_SCREEN_SPACING} px on screen; zoom in to see them"
            ),
        }
    }
}

/// Whether `grid` is drawn at `zoom` (screen points per image pixel).
pub fn visibility(grid: &Grid, document: &Document, zoom: f32) -> Result<(), GridHidden> {
    let spacing = grid.line_spacing(document).map_err(GridHidden::Invalid)?;
    if spacing * f64::from(zoom) < f64::from(MIN_SCREEN_SPACING) {
        Err(GridHidden::TooSmallAtThisZoom)
    } else {
        Ok(())
    }
}

/// Draws every visible grid, as currently displayed (including any live preview).
pub fn paint_grids(
    painter: &egui::Painter,
    transform: DocumentTransform,
    document: &Document,
    selection: &Selection,
) {
    let region = transform.image_bounds_of(painter.clip_rect());
    for stored in document.grids() {
        let grid = selection.displayed_grid(stored.id(), stored);
        if visibility(&grid, document, transform.zoom()).is_err() {
            continue;
        }
        let Ok(lines) = grid.lines(document, region) else {
            continue;
        };
        let stroke = Stroke::new(GRID_LINE_WIDTH, with_opacity(grid.color(), grid.opacity()));
        for (start, end) in lines {
            painter.line_segment(
                [
                    transform.image_to_screen(start),
                    transform.image_to_screen(end),
                ],
                stroke,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gridtool::action::Action;
    use gridtool::calibration::{CalibrationMethod, Line, ScalePoints};
    use gridtool::geometry::Point;
    use gridtool::grid::{CellDimension, GridSource};
    use gridtool::units::LengthUnit;

    /// A 100×50 document at 1 cm per pixel, with a 0.01 cm grid on scale line A.
    fn document_with_tiny_grid() -> Document {
        let mut document = Document::new(100, 50);
        let points = ScalePoints::new(
            CalibrationMethod::ThreePoint,
            vec![
                Point::new(0.0, 0.0),
                Point::new(20.0, 0.0),
                Point::new(20.0, 40.0),
            ],
        )
        .unwrap();
        for action in [
            Action::SetScalePoints(points),
            Action::SetDistance {
                line: Line::A,
                distance: Some(20.0),
            },
            Action::SetDistance {
                line: Line::B,
                distance: Some(40.0),
            },
            Action::AddGrid(Grid::new(
                GridSource::ScaleLine(Line::A),
                LengthUnit::Centimetre,
            )),
            Action::SetGridSize {
                index: 0,
                dimension: CellDimension::Side,
                size: 0.01,
            },
        ] {
            action.apply(&mut document).unwrap();
        }
        document
    }

    #[test]
    fn tiny_cells_are_hidden_until_zoomed_in_far_enough() {
        let document = document_with_tiny_grid();
        let grid = &document.grids()[0];

        // 0.01 px apart in the image: 3 px on screen needs a zoom of 300.
        assert_eq!(
            visibility(grid, &document, 299.0),
            Err(GridHidden::TooSmallAtThisZoom)
        );
        assert_eq!(visibility(grid, &document, 300.0), Ok(()));
    }

    #[test]
    fn invalid_grids_report_why() {
        let mut document = document_with_tiny_grid();
        Action::SetDistance {
            line: Line::A,
            distance: None,
        }
        .apply(&mut document)
        .unwrap();

        assert!(matches!(
            visibility(&document.grids()[0], &document, 1.0),
            Err(GridHidden::Invalid(GridError::NoScale(_)))
        ));
    }
}
