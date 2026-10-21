use eframe::egui::Pos2;

/// Index of the position nearest to `target` that lies within `radius`, if any.
pub fn nearest_within(positions: &[Pos2], target: Pos2, radius: f32) -> Option<usize> {
    positions
        .iter()
        .enumerate()
        .map(|(index, position)| (index, position.distance(target)))
        .filter(|&(_, distance)| distance <= radius)
        .min_by(|(_, a), (_, b)| a.total_cmp(b))
        .map(|(index, _)| index)
}

#[cfg(test)]
mod tests {
    use super::*;

    const POSITIONS: [Pos2; 3] = [
        Pos2::new(0.0, 0.0),
        Pos2::new(10.0, 0.0),
        Pos2::new(100.0, 100.0),
    ];

    #[test]
    fn finds_the_nearest_position_in_range() {
        assert_eq!(
            nearest_within(&POSITIONS, Pos2::new(6.0, 0.0), 8.0),
            Some(1)
        );
        assert_eq!(
            nearest_within(&POSITIONS, Pos2::new(4.0, 0.0), 8.0),
            Some(0)
        );
    }

    #[test]
    fn ignores_positions_out_of_range() {
        assert_eq!(nearest_within(&POSITIONS, Pos2::new(50.0, 50.0), 8.0), None);
        assert_eq!(nearest_within(&[], Pos2::ZERO, 8.0), None);
    }
}
