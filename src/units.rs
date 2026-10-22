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

    /// The symbol of the matching unit of area, e.g. `cm²`.
    pub fn area_symbol(self) -> &'static str {
        match self {
            LengthUnit::Centimetre => "cm²",
            LengthUnit::Metre => "m²",
        }
    }

    /// Converts `value` from this unit to `target`.
    pub fn convert(self, value: f64, target: LengthUnit) -> f64 {
        value * self.metres_per_unit() / target.metres_per_unit()
    }

    /// Converts an area `value` from this unit squared to `target` squared.
    pub fn convert_area(self, value: f64, target: LengthUnit) -> f64 {
        let factor = self.metres_per_unit() / target.metres_per_unit();
        value * factor * factor
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
    fn converts_areas_by_the_square_of_the_length_factor() {
        assert_eq!(
            LengthUnit::Metre.convert_area(1.5, LengthUnit::Centimetre),
            15_000.0
        );
        let metres = LengthUnit::Centimetre.convert_area(20_000.0, LengthUnit::Metre);
        assert!((metres - 2.0).abs() < 1e-12, "{metres}");
        assert_eq!(LengthUnit::Metre.area_symbol(), "m²");
        assert_eq!(LengthUnit::Centimetre.area_symbol(), "cm²");
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
