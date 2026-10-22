use egui_material_icons::MaterialIcon;

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
    /// Add grids aligned to a scale line or a measurement line.
    Grid,
    /// Click on the image to move the document origin (the rotation centre).
    Origin,
}

impl ToolMode {
    /// Every mode, in tool bar order.
    pub const ALL: [ToolMode; 7] = [
        ToolMode::Pan,
        ToolMode::Rotate,
        ToolMode::Scale,
        ToolMode::Measure,
        ToolMode::Area,
        ToolMode::Grid,
        ToolMode::Origin,
    ];

    pub fn icon(self) -> MaterialIcon {
        match self {
            ToolMode::Pan => icons::PAN,
            ToolMode::Rotate => icons::ROTATE,
            ToolMode::Scale => icons::SCALE,
            ToolMode::Measure => icons::MEASURE,
            ToolMode::Area => icons::AREA,
            ToolMode::Grid => icons::GRID,
            ToolMode::Origin => icons::ORIGIN,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            ToolMode::Pan => "Pan & zoom",
            ToolMode::Rotate => "Rotate view",
            ToolMode::Scale => "Scale",
            ToolMode::Measure => "Measure distance",
            ToolMode::Area => "Measure area",
            ToolMode::Grid => "Grid",
            ToolMode::Origin => "Origin",
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
    fn area_follows_measure_and_grid_follows_area_in_the_tool_bar() {
        let position = |mode| ToolMode::ALL.iter().position(|&m| m == mode).unwrap();

        assert_eq!(position(ToolMode::Area), position(ToolMode::Measure) + 1);
        assert_eq!(position(ToolMode::Grid), position(ToolMode::Area) + 1);
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
