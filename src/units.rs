use std::fmt;

/// Real-world length units the scale can be expressed in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LengthUnit {
    #[default]
    Centimetre,
    Metre,
}

impl LengthUnit {
    pub const ALL: [LengthUnit; 2] = [LengthUnit::Centimetre, LengthUnit::Metre];

    pub fn symbol(self) -> &'static str {
        match self {
            LengthUnit::Centimetre => "cm",
            LengthUnit::Metre => "m",
        }
    }
}

impl fmt::Display for LengthUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.symbol())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn centimetres_are_the_default() {
        assert_eq!(LengthUnit::default(), LengthUnit::Centimetre);
    }

    #[test]
    fn displays_as_symbol() {
        assert_eq!(LengthUnit::Centimetre.to_string(), "cm");
        assert_eq!(LengthUnit::Metre.to_string(), "m");
    }
}
