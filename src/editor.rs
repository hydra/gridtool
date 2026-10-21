//! A document plus its undo/redo history.

use crate::action::{Action, ActionError};
use crate::document::Document;

struct HistoryEntry {
    action: Action,
    before: Document,
    after: Document,
}

/// Owns a [`Document`] and applies [`Action`]s to it, recording them so they can be
/// undone and redone.
pub struct Editor {
    document: Document,
    history: Vec<HistoryEntry>,
    /// Number of history entries currently applied; entries after this can be redone.
    applied: usize,
}

impl Editor {
    pub fn new(document: Document) -> Self {
        Self {
            document,
            history: Vec::new(),
            applied: 0,
        }
    }

    pub fn document(&self) -> &Document {
        &self.document
    }

    /// Applies `action` and records it, discarding anything that could be redone.
    ///
    /// Returns `Ok(false)` without recording anything if the action would not change
    /// the document. On error the document and history are unchanged.
    pub fn perform(&mut self, action: Action) -> Result<bool, ActionError> {
        let mut after = self.document.clone();
        action.apply(&mut after)?;
        if after == self.document {
            return Ok(false);
        }

        self.history.truncate(self.applied);
        let before = std::mem::replace(&mut self.document, after.clone());
        self.history.push(HistoryEntry {
            action,
            before,
            after,
        });
        self.applied += 1;
        Ok(true)
    }

    pub fn can_undo(&self) -> bool {
        self.applied > 0
    }

    pub fn can_redo(&self) -> bool {
        self.applied < self.history.len()
    }

    /// Reverts the most recent applied action. Returns `false` if there is nothing to undo.
    pub fn undo(&mut self) -> bool {
        if !self.can_undo() {
            return false;
        }
        self.applied -= 1;
        self.document = self.history[self.applied].before.clone();
        true
    }

    /// Re-applies the most recently undone action. Returns `false` if there is nothing
    /// to redo.
    pub fn redo(&mut self) -> bool {
        if !self.can_redo() {
            return false;
        }
        self.document = self.history[self.applied].after.clone();
        self.applied += 1;
        true
    }

    /// Every recorded action, oldest first, including those that have been undone.
    pub fn actions(&self) -> impl ExactSizeIterator<Item = &Action> {
        self.history.iter().map(|entry| &entry.action)
    }

    /// How many of [`Self::actions`] are currently applied.
    pub fn applied_count(&self) -> usize {
        self.applied
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calibration::{CalibrationMethod, Line, ScalePoints};
    use crate::geometry::Point;
    use crate::units::LengthUnit;

    fn set_distance_a(distance: f64) -> Action {
        Action::SetDistance {
            line: Line::A,
            distance: Some(distance),
        }
    }

    fn distance_a(editor: &Editor) -> Option<f64> {
        editor.document().calibration().distance(Line::A)
    }

    fn new_editor() -> Editor {
        Editor::new(Document::new(200, 100))
    }

    #[test]
    fn duplicating_and_deleting_can_be_undone_and_redone() {
        let mut editor = new_editor();
        editor
            .perform(Action::AddMeasurement(
                crate::measurement::Measurement::new(
                    Point::new(1.0, 1.0),
                    Point::new(9.0, 9.0),
                    LengthUnit::Centimetre,
                ),
            ))
            .unwrap();
        let added = editor.document().clone();

        editor
            .perform(Action::DuplicateMeasurement { index: 0 })
            .unwrap();
        let duplicated = editor.document().clone();
        editor
            .perform(Action::DeleteMeasurement { index: 0 })
            .unwrap();
        let deleted = editor.document().clone();
        assert_eq!(duplicated.measurements().len(), 2);
        assert_eq!(deleted.measurements().len(), 1);

        assert!(editor.undo());
        assert_eq!(editor.document(), &duplicated);
        assert!(editor.undo());
        assert_eq!(editor.document(), &added);
        assert!(editor.redo());
        assert!(editor.redo());
        assert_eq!(editor.document(), &deleted);
    }

    #[test]
    fn starts_with_nothing_to_undo_or_redo() {
        let editor = new_editor();

        assert!(!editor.can_undo());
        assert!(!editor.can_redo());
        assert_eq!(editor.actions().len(), 0);
    }

    #[test]
    fn performing_records_the_action() {
        let mut editor = new_editor();

        assert_eq!(editor.perform(set_distance_a(5.0)), Ok(true));

        assert_eq!(distance_a(&editor), Some(5.0));
        assert!(editor.can_undo());
        assert_eq!(editor.actions().collect::<Vec<_>>(), [&set_distance_a(5.0)]);
        assert_eq!(editor.applied_count(), 1);
    }

    #[test]
    fn undo_and_redo_restore_document_states() {
        let mut editor = new_editor();
        editor.perform(set_distance_a(5.0)).unwrap();
        editor.perform(set_distance_a(7.0)).unwrap();

        assert!(editor.undo());
        assert_eq!(distance_a(&editor), Some(5.0));
        assert!(editor.undo());
        assert_eq!(distance_a(&editor), None);
        assert!(!editor.undo());

        assert!(editor.redo());
        assert_eq!(distance_a(&editor), Some(5.0));
        assert!(editor.redo());
        assert_eq!(distance_a(&editor), Some(7.0));
        assert!(!editor.redo());
    }

    #[test]
    fn a_new_action_discards_redoable_actions() {
        let mut editor = new_editor();
        for distance in 1..=5 {
            editor.perform(set_distance_a(f64::from(distance))).unwrap();
        }
        editor.undo();
        editor.undo();
        assert!(editor.can_redo());

        editor.perform(Action::SetUnit(LengthUnit::Metre)).unwrap();

        assert!(!editor.can_redo());
        assert_eq!(editor.actions().len(), 4);
        assert_eq!(editor.applied_count(), 4);
        assert_eq!(distance_a(&editor), Some(3.0));
    }

    #[test]
    fn actions_that_change_nothing_are_not_recorded() {
        let mut editor = new_editor();

        assert_eq!(
            editor.perform(Action::SetUnit(LengthUnit::default())),
            Ok(false)
        );

        assert!(!editor.can_undo());
    }

    #[test]
    fn failed_actions_leave_document_and_history_unchanged() {
        let mut editor = new_editor();
        editor.perform(set_distance_a(5.0)).unwrap();
        editor.undo();

        let result = editor.perform(Action::MoveScalePoint {
            index: 0,
            to: Point::default(),
        });

        assert!(result.is_err());
        assert_eq!(distance_a(&editor), None);
        assert!(editor.can_redo(), "a failed action must not clear redo");
    }

    #[test]
    fn full_three_point_calibration_workflow() {
        let mut editor = new_editor();
        let points = ScalePoints::new(
            CalibrationMethod::ThreePoint,
            vec![
                Point::new(0.0, 0.0),
                Point::new(100.0, 0.0),
                Point::new(100.0, 50.0),
            ],
        )
        .unwrap();

        editor.perform(Action::SetScalePoints(points)).unwrap();
        editor.perform(set_distance_a(2.0)).unwrap();
        editor
            .perform(Action::SetDistance {
                line: Line::B,
                distance: Some(2.0),
            })
            .unwrap();
        editor.perform(Action::SetUnit(LengthUnit::Metre)).unwrap();

        let scale = editor.document().calibration().scale().unwrap();
        assert_eq!(scale.x_per_pixel, 0.02);
        assert_eq!(scale.y_per_pixel, 0.04);
        assert_eq!(scale.unit, LengthUnit::Metre);

        editor
            .perform(Action::MoveScalePoint {
                index: 2,
                to: Point::new(100.0, 100.0),
            })
            .unwrap();
        assert_eq!(
            editor.document().calibration().scale().unwrap().y_per_pixel,
            0.02
        );

        editor.undo();
        assert_eq!(
            editor.document().calibration().scale().unwrap().y_per_pixel,
            0.04
        );
    }
}
