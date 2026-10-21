use std::fmt;

/// A position in document (image) pixel coordinates: origin at the top-left corner of
/// the un-rotated image, x to the right, y down.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({:.1}, {:.1})", self.x, self.y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn displays_with_one_decimal_place() {
        assert_eq!(Point::new(12.0, 3.456).to_string(), "(12.0, 3.5)");
    }
}
