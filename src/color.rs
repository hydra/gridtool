//! Colours of document objects.

use std::fmt;

/// An opaque 8-bit-per-channel colour.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub const BLUE: Rgb = Rgb::new(0, 0, 255);
    pub const GREEN: Rgb = Rgb::new(0, 255, 0);
    pub const CYAN: Rgb = Rgb::new(0, 255, 255);

    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

impl fmt::Display for Rgb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn displays_as_hex() {
        assert_eq!(Rgb::new(0, 255, 16).to_string(), "#00FF10");
        assert_eq!(Rgb::CYAN.to_string(), "#00FFFF");
    }
}
