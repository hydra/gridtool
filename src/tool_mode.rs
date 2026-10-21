/// The active canvas tool, which decides how mouse input on the canvas is interpreted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ToolMode {
    /// Drag to move the view; scroll to zoom.
    #[default]
    Pan,
    /// Drag the arrow handle to rotate the document about its center.
    Rotate,
}

impl ToolMode {
    /// Every mode, in tool bar order.
    pub const ALL: [ToolMode; 2] = [ToolMode::Pan, ToolMode::Rotate];

    pub fn icon(self) -> &'static str {
        match self {
            ToolMode::Pan => "✋",
            ToolMode::Rotate => "🔄",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            ToolMode::Pan => "Pan & zoom",
            ToolMode::Rotate => "Rotate",
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
    fn modes_have_distinct_icons_and_names() {
        let icons: HashSet<_> = ToolMode::ALL.iter().map(|mode| mode.icon()).collect();
        let names: HashSet<_> = ToolMode::ALL.iter().map(|mode| mode.name()).collect();

        assert_eq!(icons.len(), ToolMode::ALL.len());
        assert_eq!(names.len(), ToolMode::ALL.len());
    }
}
