use egui_material_icons::MaterialIcon;
use gridtool::object::ObjectKind;
use gridtool::tr;

use super::icons;

/// The active canvas tool, which decides how mouse input on the canvas is interpreted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ToolMode {
    /// Drag to move the view; scroll to zoom.
    #[default]
    Pan,
    /// Drag the arrow handle to rotate the view about the document origin.
    Rotate,
    /// Place and drag calibration points to set the real-world scale.
    Scale,
    /// Add measurement lines by clicking their two ends.
    Measure,
    /// Add area measurements by clicking the corners of their outline.
    Area,
    /// Add route measurements by clicking the points along them.
    Route,
    /// Add grids aligned to a scale line or a measurement line.
    Grid,
    /// Click on the image to move the document origin (the rotation centre).
    Origin,
    /// Draw lines by clicking their points, optionally joining them to enclose an area.
    DrawLine,
}

impl ToolMode {
    /// Every mode, in tool bar order.
    pub const ALL: [ToolMode; 9] = [
        ToolMode::Pan,
        ToolMode::Rotate,
        ToolMode::Scale,
        ToolMode::Measure,
        ToolMode::Area,
        ToolMode::Route,
        ToolMode::Grid,
        ToolMode::Origin,
        ToolMode::DrawLine,
    ];

    /// The tool that adds objects of `kind`, which becomes the active tool along with one
    /// of them.
    pub fn for_object(kind: ObjectKind) -> ToolMode {
        match kind {
            ObjectKind::Measurement => ToolMode::Measure,
            ObjectKind::Area => ToolMode::Area,
            ObjectKind::Grid => ToolMode::Grid,
            ObjectKind::Shape => ToolMode::DrawLine,
            ObjectKind::Route => ToolMode::Route,
        }
    }

    /// Whether the mode starts a new group of tool bar buttons: the drawing tools.
    pub fn starts_group(self) -> bool {
        self == ToolMode::DrawLine
    }

    pub fn icon(self) -> MaterialIcon {
        match self {
            ToolMode::Pan => icons::PAN,
            ToolMode::Rotate => icons::ROTATE,
            ToolMode::Scale => icons::SCALE,
            ToolMode::Measure => icons::MEASURE,
            ToolMode::Area => icons::AREA,
            ToolMode::Route => icons::ROUTE,
            ToolMode::Grid => icons::GRID,
            ToolMode::Origin => icons::ORIGIN,
            ToolMode::DrawLine => icons::DRAW_LINE,
        }
    }

    pub fn name(self) -> String {
        match self {
            ToolMode::Pan => tr!("tool-pan"),
            ToolMode::Rotate => tr!("tool-rotate"),
            ToolMode::Scale => tr!("tool-scale"),
            ToolMode::Measure => tr!("tool-measure"),
            ToolMode::Area => tr!("tool-area"),
            ToolMode::Route => tr!("tool-route"),
            ToolMode::Grid => tr!("tool-grid"),
            ToolMode::Origin => tr!("tool-origin"),
            ToolMode::DrawLine => tr!("tool-draw-line"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn pan_is_the_default_mode() {
        assert_eq!(ToolMode::default(), ToolMode::Pan);
    }

    #[test]
    fn measure_follows_scale_in_the_tool_bar() {
        let scale = ToolMode::ALL.iter().position(|&m| m == ToolMode::Scale);
        let measure = ToolMode::ALL.iter().position(|&m| m == ToolMode::Measure);

        assert_eq!(measure, scale.map(|index| index + 1));
    }

    #[test]
    fn area_follows_measure_route_follows_area_and_grid_follows_route_in_the_tool_bar() {
        let position = |mode| ToolMode::ALL.iter().position(|&m| m == mode).unwrap();

        assert_eq!(position(ToolMode::Area), position(ToolMode::Measure) + 1);
        assert_eq!(position(ToolMode::Route), position(ToolMode::Area) + 1);
        assert_eq!(position(ToolMode::Grid), position(ToolMode::Route) + 1);
    }

    #[test]
    fn a_route_brings_the_route_tool() {
        assert_eq!(ToolMode::for_object(ObjectKind::Route), ToolMode::Route);
    }

    #[test]
    fn draw_line_starts_the_drawing_group_after_origin() {
        let position = |mode| ToolMode::ALL.iter().position(|&m| m == mode).unwrap();

        assert_eq!(position(ToolMode::DrawLine), position(ToolMode::Origin) + 1);
        let group_starts: Vec<_> = ToolMode::ALL
            .into_iter()
            .filter(|mode| mode.starts_group())
            .collect();
        assert_eq!(group_starts, [ToolMode::DrawLine]);
    }

    #[test]
    fn each_kind_of_object_belongs_to_the_tool_that_adds_it() {
        assert_eq!(
            ToolMode::for_object(ObjectKind::Measurement),
            ToolMode::Measure
        );
        assert_eq!(ToolMode::for_object(ObjectKind::Area), ToolMode::Area);
        assert_eq!(ToolMode::for_object(ObjectKind::Grid), ToolMode::Grid);
        assert_eq!(ToolMode::for_object(ObjectKind::Shape), ToolMode::DrawLine);
    }

    #[test]
    fn modes_have_distinct_icons_and_names() {
        let icons: HashSet<_> = ToolMode::ALL
            .iter()
            .map(|mode| mode.icon().codepoint)
            .collect();
        let names: HashSet<_> = ToolMode::ALL.iter().map(|mode| mode.name()).collect();

        assert_eq!(icons.len(), ToolMode::ALL.len());
        assert_eq!(names.len(), ToolMode::ALL.len());
    }
}
