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

    /// Converts `value` from this unit to `target`.
    pub fn convert(self, value: f64, target: LengthUnit) -> f64 {
        value * self.metres_per_unit() / target.metres_per_unit()
    }

    fn metres_per_unit(self) -> f64 {
        match self {
            LengthUnit::Centimetre => 0.01,
            LengthUnit::Metre => 1.0,
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

    #[test]
    fn converts_between_units() {
        assert_eq!(
            LengthUnit::Centimetre.convert(150.0, LengthUnit::Metre),
            1.5
        );
        assert_eq!(
            LengthUnit::Metre.convert(2.5, LengthUnit::Centimetre),
            250.0
        );
        assert_eq!(LengthUnit::Metre.convert(2.5, LengthUnit::Metre), 2.5);
    }
}
