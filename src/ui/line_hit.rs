//! Hit-testing lines (not their end handles) in screen space.

use eframe::egui::Pos2;

/// Shortest distance from `point` to the segment `a`–`b`.
pub fn distance_to_segment(point: Pos2, a: Pos2, b: Pos2) -> f32 {
    let segment = b - a;
    let length_sq = segment.length_sq();
    if length_sq == 0.0 {
        return point.distance(a);
    }
    let t = ((point - a).dot(segment) / length_sq).clamp(0.0, 1.0);
    point.distance(a + segment * t)
}

/// Index of the segment nearest `point` that is within `tolerance` of it. Points within
/// `end_exclusion` of either end of a segment do not count for that segment, so the
/// handles at the ends of a line are not part of its hover area.
pub fn segment_at(
    segments: &[(Pos2, Pos2)],
    point: Pos2,
    tolerance: f32,
    end_exclusion: f32,
) -> Option<usize> {
    segments
        .iter()
        .enumerate()
        .filter(|(_, (a, b))| {
            point.distance(*a) > end_exclusion && point.distance(*b) > end_exclusion
        })
        .map(|(index, &(a, b))| (index, distance_to_segment(point, a, b)))
        .filter(|&(_, distance)| distance <= tolerance)
        .min_by(|(_, a), (_, b)| a.total_cmp(b))
        .map(|(index, _)| index)
}

#[cfg(test)]
mod tests {
    use super::*;

    const HORIZONTAL: (Pos2, Pos2) = (Pos2::new(0.0, 0.0), Pos2::new(100.0, 0.0));
    const VERTICAL: (Pos2, Pos2) = (Pos2::new(50.0, -50.0), Pos2::new(50.0, 50.0));

    #[test]
    fn distance_is_perpendicular_within_the_segment_and_to_the_ends_outside_it() {
        let (a, b) = HORIZONTAL;

        assert_eq!(distance_to_segment(Pos2::new(30.0, 4.0), a, b), 4.0);
        assert_eq!(distance_to_segment(Pos2::new(-3.0, 4.0), a, b), 5.0);
        assert_eq!(distance_to_segment(Pos2::new(5.0, 5.0), a, a), 50f32.sqrt());
    }

    #[test]
    fn hits_a_line_within_the_tolerance() {
        let segments = [HORIZONTAL];

        assert_eq!(
            segment_at(&segments, Pos2::new(30.0, 4.0), 5.0, 8.0),
            Some(0)
        );
        assert_eq!(segment_at(&segments, Pos2::new(30.0, 6.0), 5.0, 8.0), None);
    }

    #[test]
    fn ignores_the_ends_of_a_line() {
        let segments = [HORIZONTAL];

        assert_eq!(segment_at(&segments, Pos2::new(3.0, 0.0), 5.0, 8.0), None);
        assert_eq!(segment_at(&segments, Pos2::new(97.0, 1.0), 5.0, 8.0), None);
        assert_eq!(
            segment_at(&segments, Pos2::new(9.0, 0.0), 5.0, 8.0),
            Some(0)
        );
    }

    #[test]
    fn picks_the_nearest_of_crossing_lines() {
        let segments = [HORIZONTAL, VERTICAL];

        assert_eq!(
            segment_at(&segments, Pos2::new(53.0, 1.0), 5.0, 8.0),
            Some(0)
        );
        assert_eq!(
            segment_at(&segments, Pos2::new(51.0, 3.0), 5.0, 8.0),
            Some(1)
        );
    }
}
