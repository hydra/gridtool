use super::*;
use crate::calibration::Line;
use crate::object::ObjectKind;
use crate::style::LineStyle;

/// Two segments: (10, 10) to (40, 10) to (40, 50).
fn route() -> Route {
    Route::new(
        vec![
            Point::new(10.0, 10.0),
            Point::new(40.0, 10.0),
            Point::new(40.0, 50.0),
        ],
        LengthUnit::Centimetre,
    )
    .unwrap()
}

/// A 100 × 60 document holding [`route`] and a measurement after it.
fn document_with_route() -> Document {
    let mut document = Document::new(100, 60);
    Action::AddRoute(route()).apply(&mut document).unwrap();
    Action::AddMeasurement(measurement(1.0, 2.0))
        .apply(&mut document)
        .unwrap();
    document
}

#[test]
fn adds_routes_with_their_own_ids() {
    let document = document_with_route();

    let [stored] = document.routes() else {
        panic!("expected one route");
    };
    assert_eq!(**stored, route());
    assert_eq!(document.route_index(stored.id()), Some(0));
    assert_eq!(document.object_kind(stored.id()), Some(ObjectKind::Route));
    assert!(document.contains_object(stored.id()));
    assert_ne!(document.measurements()[0].id(), stored.id());
    assert!(Action::AddRoute(route()).activates_new_object());
    assert!(Action::DuplicateRoute { index: 0 }.activates_new_object());
}

#[test]
fn a_route_is_measured_along_every_segment() {
    let mut document = document_with_route();
    // 2 px to the centimetre.
    Action::SetScalePoints(
        ScalePoints::new(
            CalibrationMethod::TwoPoint,
            vec![Point::new(0.0, 0.0), Point::new(20.0, 0.0)],
        )
        .unwrap(),
    )
    .apply(&mut document)
    .unwrap();
    Action::SetDistance {
        line: Line::A,
        distance: Some(10.0),
    }
    .apply(&mut document)
    .unwrap();

    let stored = &document.routes()[0];
    assert_eq!(
        stored.segment_lengths(document.calibration()).unwrap(),
        vec![15.0, 20.0]
    );
    assert_eq!(stored.length(document.calibration()).unwrap(), 35.0);
}

#[test]
fn moving_a_point_moves_the_ends_of_both_segments_it_joins() {
    let mut document = document_with_route();

    Action::MoveRoutePoint {
        index: 0,
        point: 1,
        to: Point::new(45.0, 20.0),
    }
    .apply(&mut document)
    .unwrap();

    let segments: Vec<_> = document.routes()[0].segments().collect();
    assert_eq!(segments[0].1, Point::new(45.0, 20.0));
    assert_eq!(segments[1].0, Point::new(45.0, 20.0));
}

#[test]
fn moves_resizes_and_changes_the_unit_of_a_route_keeping_the_rest() {
    let mut document = document_with_route();
    let id = document.routes()[0].id();
    let points = vec![
        Point::new(0.0, 0.0),
        Point::new(5.0, 5.0),
        Point::new(9.0, 1.0),
    ];

    for action in [
        Action::MoveRoute {
            index: 0,
            points: points.clone(),
        },
        Action::ResizeRoute {
            index: 0,
            points: points.clone(),
        },
    ] {
        let mut moved = document.clone();
        action.apply(&mut moved).unwrap();
        assert_eq!(moved.routes()[0].points(), points);
        assert_eq!(moved.routes()[0].id(), id);
        assert_eq!(moved.routes()[0].unit(), LengthUnit::Centimetre);
    }

    Action::SetRouteUnit {
        index: 0,
        unit: LengthUnit::Metre,
    }
    .apply(&mut document)
    .unwrap();
    assert_eq!(document.routes()[0].unit(), LengthUnit::Metre);
    assert_eq!(document.routes()[0].points(), route().points());
}

#[test]
fn a_route_can_change_its_line_but_has_no_fill() {
    let mut document = document_with_route();
    let target = StyleTarget::Route(0);

    Action::SetStyle {
        target,
        edit: StyleEdit::LineStyle(LineStyle::Dashed),
    }
    .apply(&mut document)
    .unwrap();
    assert_eq!(document.routes()[0].line().style, LineStyle::Dashed);

    let before = document.clone();
    assert!(matches!(
        Action::SetStyle {
            target,
            edit: StyleEdit::FillOpacity(0.5),
        }
        .apply(&mut document),
        Err(ActionError::HasNoFill(_))
    ));
    assert_eq!(document, before);
}

#[test]
fn rejects_invalid_route_edits() {
    let mut document = document_with_route();
    let before = document.clone();

    let outside = Route::new(
        vec![Point::new(1.0, 1.0), Point::new(101.0, 1.0)],
        LengthUnit::Centimetre,
    )
    .unwrap();
    let rejected = [
        Action::AddRoute(outside),
        Action::MoveRoutePoint {
            index: 0,
            point: 3,
            to: Point::new(1.0, 1.0),
        },
        Action::MoveRoutePoint {
            index: 0,
            point: 0,
            to: Point::new(1.0, 61.0),
        },
        Action::MoveRoutePoint {
            index: 1,
            point: 0,
            to: Point::new(1.0, 1.0),
        },
        Action::ResizeRoute {
            index: 0,
            points: vec![Point::new(1.0, 1.0), Point::new(2.0, 2.0)],
        },
        Action::SetRouteUnit {
            index: 2,
            unit: LengthUnit::Metre,
        },
        Action::DeleteRoute { index: 1 },
        Action::DuplicateRoute { index: 1 },
    ];
    for action in rejected {
        assert!(action.apply(&mut document).is_err(), "{action:?}");
    }
    assert_eq!(document, before);
}

#[test]
fn duplicates_and_deletes_routes() {
    let mut document = document_with_route();
    let original = document.routes()[0].id();

    let duplicate = Action::duplicate_object(&document, original).unwrap();
    assert_eq!(duplicate, Action::DuplicateRoute { index: 0 });
    duplicate.apply(&mut document).unwrap();
    let copy = document.routes()[1].id();
    assert_ne!(copy, original);
    assert_eq!(*document.routes()[1], *document.routes()[0]);
    assert_eq!(document.newest_object(), Some(copy));

    let delete = Action::delete_object(&document, original).unwrap();
    assert_eq!(delete, Action::DeleteRoute { index: 0 });
    delete.apply(&mut document).unwrap();
    assert_eq!(document.routes().len(), 1);
    assert_eq!(document.routes()[0].id(), copy);
}

#[test]
fn every_route_action_can_be_undone_and_redone() {
    let actions = [
        Action::AddRoute(route()),
        Action::MoveRoutePoint {
            index: 0,
            point: 2,
            to: Point::new(41.0, 51.0),
        },
        Action::MoveRoute {
            index: 0,
            points: vec![
                Point::new(1.0, 1.0),
                Point::new(2.0, 2.0),
                Point::new(3.0, 3.0),
            ],
        },
        Action::SetRouteUnit {
            index: 0,
            unit: LengthUnit::Metre,
        },
        Action::SetStyle {
            target: StyleTarget::Route(0),
            edit: StyleEdit::LineOpacity(0.5),
        },
        Action::DuplicateRoute { index: 0 },
        Action::DeleteRoute { index: 0 },
    ];
    let mut editor = Editor::new(Document::new(100, 60));
    let mut states = vec![editor.document().clone()];
    for action in actions {
        assert!(editor.perform(action.clone()).unwrap(), "{action:?}");
        states.push(editor.document().clone());
    }

    for state in states.iter().rev().skip(1) {
        assert!(editor.undo());
        assert_eq!(editor.document(), state);
    }
    for state in states.iter().skip(1) {
        assert!(editor.redo());
        assert_eq!(editor.document(), state);
    }
}

#[test]
fn the_outline_and_focus_of_a_route_are_its_open_path_and_box_centre() {
    let document = document_with_route();
    let id = document.routes()[0].id();

    assert_eq!(
        document.object_outline(id),
        Some(Outline {
            points: route().points().to_vec(),
            closed: false,
        })
    );
    assert_eq!(
        document.object_focus_point(id),
        Some(Point::new(25.0, 30.0))
    );
}

#[test]
fn describes_route_actions() {
    let labels = [
        (Action::AddRoute(route()), "Add route with 3 points"),
        (
            Action::MoveRoutePoint {
                index: 0,
                point: 1,
                to: Point::new(3.0, 4.0),
            },
            "Move route 1 point 2 to (3.0, 4.0)",
        ),
        (
            Action::ResizeRoute {
                index: 1,
                points: Vec::new(),
            },
            "Resize route 2",
        ),
        (
            Action::MoveRoute {
                index: 1,
                points: Vec::new(),
            },
            "Move route 2",
        ),
        (
            Action::SetRouteUnit {
                index: 0,
                unit: LengthUnit::Metre,
            },
            "Set route 1 units to m",
        ),
        (Action::DeleteRoute { index: 2 }, "Delete route 3"),
        (Action::DuplicateRoute { index: 0 }, "Duplicate route 1"),
        (
            Action::SetStyle {
                target: StyleTarget::Route(0),
                edit: StyleEdit::LineStyle(LineStyle::Dashed),
            },
            "Set route 1 line style to Dashed",
        ),
    ];
    for (action, label) in labels {
        assert_eq!(action.to_string(), label);
    }
}
