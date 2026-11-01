use super::*;

fn open_shape() -> DrawnShape {
    DrawnShape::open(
        vec![
            Point::new(1.0, 1.0),
            Point::new(20.0, 1.0),
            Point::new(20.0, 30.0),
        ],
        LengthUnit::Centimetre,
    )
    .unwrap()
}

fn closed_shape() -> DrawnShape {
    DrawnShape::closed(
        vec![
            Point::new(10.0, 10.0),
            Point::new(40.0, 10.0),
            Point::new(40.0, 40.0),
        ],
        LengthUnit::Centimetre,
    )
    .unwrap()
}

/// A document holding [`open_shape`], then [`closed_shape`].
fn document_with_shapes() -> Document {
    let mut document = Document::new(100, 50);
    for shape in [open_shape(), closed_shape()] {
        Action::AddShape(shape).apply(&mut document).unwrap();
    }
    document
}

#[test]
fn adds_shapes_in_order_with_their_own_ids() {
    let mut document = document_with_shapes();
    Action::AddMeasurement(measurement(1.0, 2.0))
        .apply(&mut document)
        .unwrap();

    let [line, closed] = document.shapes() else {
        panic!("expected two shapes");
    };
    assert_eq!(**line, open_shape());
    assert_eq!(**closed, closed_shape());
    assert_ne!(line.id(), closed.id());
    assert_eq!(document.shape_index(closed.id()), Some(1));
    assert!(document.contains_object(line.id()));
    assert_ne!(document.measurements()[0].id(), closed.id());
}

#[test]
fn moves_and_resizes_a_shape_keeping_its_style() {
    let mut document = document_with_shapes();
    let id = document.shapes()[0].id();
    let points = vec![
        Point::new(0.0, 0.0),
        Point::new(5.0, 0.0),
        Point::new(5.0, 5.0),
    ];

    Action::SetShapeLineStyle {
        index: 0,
        style: LineStyle::Thick,
    }
    .apply(&mut document)
    .unwrap();
    Action::MoveShapePoint {
        index: 0,
        point: 1,
        to: Point::new(30.0, 2.0),
    }
    .apply(&mut document)
    .unwrap();
    assert_eq!(document.shapes()[0].points()[1], Point::new(30.0, 2.0));
    assert_eq!(document.shapes()[0].points()[0], Point::new(1.0, 1.0));

    Action::ResizeShape {
        index: 0,
        points: points.clone(),
    }
    .apply(&mut document)
    .unwrap();
    let shape = &document.shapes()[0];
    assert_eq!(shape.id(), id);
    assert_eq!(shape.points(), points);
    assert_eq!(shape.line_style(), LineStyle::Thick);
    assert!(!shape.is_closed());
}

#[test]
fn sets_every_style_of_a_closed_shape() {
    let mut document = document_with_shapes();
    for action in [
        Action::SetShapeLineStyle {
            index: 1,
            style: LineStyle::Dashed,
        },
        Action::SetShapeLineColor {
            index: 1,
            color: Rgb::new(1, 2, 3),
        },
        Action::SetShapeLineOpacity {
            index: 1,
            opacity: 0.25,
        },
        Action::SetShapeFillStyle {
            index: 1,
            style: FillStyle::Crosshatch,
        },
        Action::SetShapeFillColor {
            index: 1,
            color: Rgb::new(4, 5, 6),
        },
        Action::SetShapeFillSpacing {
            index: 1,
            spacing: 50.0,
        },
        Action::SetShapeFillUnit {
            index: 1,
            unit: LengthUnit::Metre,
        },
        Action::SetShapeFillAngle {
            index: 1,
            degrees: 45.0,
        },
        Action::SetShapeFillOpacity {
            index: 1,
            opacity: 0.75,
        },
    ] {
        action.apply(&mut document).unwrap();
    }

    let shape = &document.shapes()[1];
    assert_eq!(shape.line_style(), LineStyle::Dashed);
    assert_eq!(shape.line_color(), Rgb::new(1, 2, 3));
    assert_eq!(shape.line_opacity(), 0.25);
    assert_eq!(shape.fill_style(), FillStyle::Crosshatch);
    assert_eq!(shape.fill_color(), Rgb::new(4, 5, 6));
    // Changing the unit keeps the value.
    assert_eq!(shape.fill_spacing(), 50.0);
    assert_eq!(shape.fill_unit(), LengthUnit::Metre);
    assert_eq!(shape.fill_angle(), 45.0);
    assert_eq!(shape.fill_opacity(), 0.75);
    assert_eq!(*document.shapes()[0], open_shape());
}

#[test]
fn a_line_can_change_its_line_style_colour_and_opacity() {
    let mut document = document_with_shapes();

    for action in [
        Action::SetShapeLineStyle {
            index: 0,
            style: LineStyle::Dashed,
        },
        Action::SetShapeLineColor {
            index: 0,
            color: Rgb::new(1, 2, 3),
        },
        Action::SetShapeLineOpacity {
            index: 0,
            opacity: 0.5,
        },
    ] {
        action.apply(&mut document).unwrap();
    }

    assert_eq!(document.shapes()[0].line_style(), LineStyle::Dashed);
    assert_eq!(document.shapes()[0].line_color(), Rgb::new(1, 2, 3));
    assert_eq!(document.shapes()[0].line_opacity(), 0.5);
}

#[test]
fn rejects_invalid_shape_edits() {
    let mut document = document_with_shapes();
    let before = document.clone();
    let missing = ActionError::ShapeIndexOutOfRange { index: 2, count: 2 };
    let not_closed = ActionError::ShapeNotClosed { index: 0 };

    let cases = [
        (
            Action::MoveShapePoint {
                index: 0,
                point: 3,
                to: Point::default(),
            },
            ActionError::ShapePointIndexOutOfRange { point: 3, count: 3 },
        ),
        (
            Action::MoveShapePoint {
                index: 0,
                point: 0,
                to: Point::new(0.0, 51.0),
            },
            ActionError::PointOutsideDocument(Point::new(0.0, 51.0)),
        ),
        (
            Action::ResizeShape {
                index: 0,
                points: vec![Point::default(); 4],
            },
            ActionError::ShapePointCountChanged {
                expected: 3,
                actual: 4,
            },
        ),
        (
            Action::SetShapeLineStyle {
                index: 2,
                style: LineStyle::Thick,
            },
            missing.clone(),
        ),
        (
            Action::SetShapeLineOpacity {
                index: 0,
                opacity: 1.5,
            },
            ActionError::InvalidOpacity(1.5),
        ),
        (
            Action::SetShapeFillStyle {
                index: 0,
                style: FillStyle::Solid,
            },
            not_closed.clone(),
        ),
        (
            Action::SetShapeFillColor {
                index: 0,
                color: Rgb::new(1, 2, 3),
            },
            not_closed.clone(),
        ),
        (
            Action::SetShapeFillSpacing {
                index: 0,
                spacing: 10.0,
            },
            not_closed.clone(),
        ),
        (
            Action::SetShapeFillUnit {
                index: 0,
                unit: LengthUnit::Metre,
            },
            not_closed.clone(),
        ),
        (
            Action::SetShapeFillAngle {
                index: 0,
                degrees: 45.0,
            },
            not_closed.clone(),
        ),
        (
            Action::SetShapeLineColor {
                index: 2,
                color: Rgb::new(1, 2, 3),
            },
            missing.clone(),
        ),
        (
            Action::SetShapeFillAngle {
                index: 1,
                degrees: 180.0,
            },
            ActionError::InvalidFillAngle(180.0),
        ),
        (
            Action::SetShapeFillAngle {
                index: 1,
                degrees: -1.0,
            },
            ActionError::InvalidFillAngle(-1.0),
        ),
        (
            Action::SetShapeFillOpacity {
                index: 0,
                opacity: 0.5,
            },
            not_closed,
        ),
        (
            Action::SetShapeFillSpacing {
                index: 1,
                spacing: 0.0,
            },
            ActionError::InvalidFillSpacing(0.0),
        ),
        (
            Action::SetShapeFillOpacity {
                index: 1,
                opacity: -0.5,
            },
            ActionError::InvalidOpacity(-0.5),
        ),
        (Action::DuplicateShape { index: 2 }, missing.clone()),
        (Action::DeleteShape { index: 2 }, missing),
    ];
    for (action, error) in cases {
        assert_eq!(action.apply(&mut document), Err(error), "{action:?}");
    }
    assert_eq!(document, before);

    let outside = DrawnShape::open(
        vec![Point::default(), Point::new(101.0, 0.0)],
        LengthUnit::Centimetre,
    )
    .unwrap();
    assert_eq!(
        Action::AddShape(outside).apply(&mut document),
        Err(ActionError::PointOutsideDocument(Point::new(101.0, 0.0)))
    );
    assert_eq!(document, before);
}

#[test]
fn duplicates_and_deletes_shapes() {
    let mut document = document_with_shapes();
    Action::SetShapeFillStyle {
        index: 1,
        style: FillStyle::Solid,
    }
    .apply(&mut document)
    .unwrap();

    Action::DuplicateShape { index: 1 }
        .apply(&mut document)
        .unwrap();
    let ids: Vec<_> = document.shapes().iter().map(|shape| shape.id()).collect();
    let [_, original, copy] = document.shapes() else {
        panic!("expected three shapes");
    };
    assert_eq!(**copy, **original);
    assert_ne!(copy.id(), original.id());
    assert_eq!(document.newest_object(), Some(copy.id()));

    Action::DeleteShape { index: 1 }
        .apply(&mut document)
        .unwrap();
    let remaining: Vec<_> = document.shapes().iter().map(|shape| shape.id()).collect();
    assert_eq!(remaining, [ids[0], ids[2]]);
}

#[test]
fn every_shape_action_can_be_undone_and_redone() {
    let mut editor = Editor::new(Document::new(100, 50));
    let actions = [
        Action::AddShape(closed_shape()),
        Action::MoveShapePoint {
            index: 0,
            point: 1,
            to: Point::new(45.0, 5.0),
        },
        Action::ResizeShape {
            index: 0,
            points: open_shape().points().to_vec(),
        },
        Action::SetShapeLineStyle {
            index: 0,
            style: LineStyle::Thick,
        },
        Action::SetShapeLineColor {
            index: 0,
            color: Rgb::new(1, 2, 3),
        },
        Action::SetShapeLineOpacity {
            index: 0,
            opacity: 0.5,
        },
        Action::SetShapeFillStyle {
            index: 0,
            style: FillStyle::Lines,
        },
        Action::SetShapeFillColor {
            index: 0,
            color: Rgb::new(4, 5, 6),
        },
        Action::SetShapeFillSpacing {
            index: 0,
            spacing: 5.0,
        },
        Action::SetShapeFillUnit {
            index: 0,
            unit: LengthUnit::Metre,
        },
        Action::SetShapeFillAngle {
            index: 0,
            degrees: 90.0,
        },
        Action::SetShapeFillOpacity {
            index: 0,
            opacity: 0.25,
        },
        Action::DuplicateShape { index: 0 },
        Action::DeleteShape { index: 0 },
    ];
    let mut states = vec![editor.document().clone()];
    for action in actions {
        assert_eq!(editor.perform(action.clone()), Ok(true), "{action:?}");
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
fn finds_the_duplicate_and_delete_actions_for_a_shape() {
    let document = document_with_shapes();
    let id = document.shapes()[1].id();

    assert_eq!(
        Action::duplicate_object(&document, id),
        Some(Action::DuplicateShape { index: 1 })
    );
    assert_eq!(
        Action::delete_object(&document, id),
        Some(Action::DeleteShape { index: 1 })
    );
    assert!(Action::DuplicateShape { index: 0 }.activates_new_object());
    assert!(Action::AddShape(open_shape()).activates_new_object());
}

#[test]
fn the_outline_and_focus_of_a_shape_follow_whether_it_is_closed() {
    let document = document_with_shapes();
    let [line, closed] = document.shapes() else {
        panic!("expected two shapes");
    };

    assert_eq!(
        document.object_outline(line.id()),
        Some(Outline {
            points: line.points().to_vec(),
            closed: false,
        })
    );
    assert_eq!(
        document.object_outline(closed.id()),
        Some(Outline {
            points: closed.points().to_vec(),
            closed: true,
        })
    );
    assert_eq!(document.object_focus_point(line.id()), Some(line.center()));
    assert_eq!(
        document.object_focus_point(closed.id()),
        Some(closed.center())
    );
}

#[test]
fn describes_shape_actions() {
    let labels = [
        (Action::AddShape(open_shape()), "Add line with 3 points"),
        (
            Action::AddShape(closed_shape()),
            "Add closed shape with 3 points",
        ),
        (
            Action::MoveShapePoint {
                index: 0,
                point: 2,
                to: Point::new(1.0, 2.0),
            },
            "Move shape 1 point 3 to (1.0, 2.0)",
        ),
        (
            Action::ResizeShape {
                index: 1,
                points: Vec::new(),
            },
            "Resize shape 2",
        ),
        (
            Action::SetShapeLineStyle {
                index: 0,
                style: LineStyle::Dashed,
            },
            "Set shape 1 line style to Dashed",
        ),
        (
            Action::SetShapeLineOpacity {
                index: 0,
                opacity: 0.25,
            },
            "Set shape 1 line opacity to 25%",
        ),
        (
            Action::SetShapeLineColor {
                index: 0,
                color: Rgb::new(255, 128, 0),
            },
            "Set shape 1 line color to #FF8000",
        ),
        (
            Action::SetShapeFillStyle {
                index: 1,
                style: FillStyle::Lines,
            },
            "Set shape 2 fill style to Lines",
        ),
        (
            Action::SetShapeFillColor {
                index: 1,
                color: Rgb::new(255, 128, 0),
            },
            "Set shape 2 fill color to #FF8000",
        ),
        (
            Action::SetShapeFillSpacing {
                index: 1,
                spacing: 50.0,
            },
            "Set shape 2 fill spacing to 50",
        ),
        (
            Action::SetShapeFillUnit {
                index: 1,
                unit: LengthUnit::Metre,
            },
            "Set shape 2 fill units to m",
        ),
        (
            Action::SetShapeFillAngle {
                index: 1,
                degrees: 22.5,
            },
            "Set shape 2 fill angle to 22.5°",
        ),
        (
            Action::SetShapeFillOpacity {
                index: 1,
                opacity: 0.5,
            },
            "Set shape 2 fill opacity to 50%",
        ),
        (Action::DeleteShape { index: 2 }, "Delete shape 3"),
        (Action::DuplicateShape { index: 0 }, "Duplicate shape 1"),
    ];
    for (action, label) in labels {
        assert_eq!(action.to_string(), label);
    }
}
