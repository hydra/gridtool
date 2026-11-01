use super::*;
use crate::calibration::CalibrationMethod;
use crate::document::Outline;
use crate::editor::Editor;

mod shapes;

fn three_points() -> ScalePoints {
    ScalePoints::new(
        CalibrationMethod::ThreePoint,
        vec![
            Point::new(10.0, 10.0),
            Point::new(90.0, 10.0),
            Point::new(90.0, 40.0),
        ],
    )
    .unwrap()
}

fn document_with_points() -> Document {
    let mut document = Document::new(100, 50);
    Action::SetScalePoints(three_points())
        .apply(&mut document)
        .unwrap();
    document
}

#[test]
fn sets_scale_points() {
    let document = document_with_points();

    assert_eq!(document.calibration().points(), Some(&three_points()));
}

#[test]
fn rejects_scale_points_outside_the_document() {
    let mut document = Document::new(10, 10);
    let before = document.clone();

    let result = Action::SetScalePoints(three_points()).apply(&mut document);

    assert!(matches!(result, Err(ActionError::PointOutsideDocument(_))));
    assert_eq!(document, before);
}

#[test]
fn moves_a_scale_point() {
    let mut document = document_with_points();

    Action::MoveScalePoint {
        index: 2,
        to: Point::new(50.0, 45.0),
    }
    .apply(&mut document)
    .unwrap();

    let points = document.calibration().points().unwrap().points();
    assert_eq!(points[2], Point::new(50.0, 45.0));
}

#[test]
fn moving_requires_existing_points_and_a_valid_index() {
    let mut empty = Document::new(100, 50);
    let move_to_origin = |index| Action::MoveScalePoint {
        index,
        to: Point::default(),
    };

    assert_eq!(
        move_to_origin(0).apply(&mut empty),
        Err(ActionError::NoScalePoints)
    );

    let mut document = document_with_points();
    assert!(matches!(
        move_to_origin(3).apply(&mut document),
        Err(ActionError::Calibration(
            CalibrationError::PointIndexOutOfRange { .. }
        ))
    ));
}

#[test]
fn sets_and_clears_distances() {
    let mut document = Document::new(100, 50);

    Action::SetDistance {
        line: Line::B,
        distance: Some(12.5),
    }
    .apply(&mut document)
    .unwrap();
    assert_eq!(document.calibration().distance(Line::B), Some(12.5));

    Action::SetDistance {
        line: Line::B,
        distance: None,
    }
    .apply(&mut document)
    .unwrap();
    assert_eq!(document.calibration().distance(Line::B), None);
}

#[test]
fn rejects_non_positive_distances() {
    let mut document = Document::new(100, 50);

    for invalid in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        let action = Action::SetDistance {
            line: Line::A,
            distance: Some(invalid),
        };
        assert!(
            action.apply(&mut document).is_err(),
            "{invalid} should be rejected"
        );
    }
}

#[test]
fn sets_the_unit() {
    let mut document = Document::new(100, 50);

    Action::SetUnit(LengthUnit::Metre)
        .apply(&mut document)
        .unwrap();

    assert_eq!(document.calibration().unit(), LengthUnit::Metre);
}

#[test]
fn sets_the_origin_inside_the_document_only() {
    let mut document = Document::new(100, 50);

    Action::SetOrigin(Point::new(10.0, 40.0))
        .apply(&mut document)
        .unwrap();
    assert_eq!(document.origin(), Point::new(10.0, 40.0));

    let before = document.clone();
    assert!(matches!(
        Action::SetOrigin(Point::new(100.5, 10.0)).apply(&mut document),
        Err(ActionError::PointOutsideDocument(_))
    ));
    assert_eq!(document, before);
    assert_eq!(
        Action::SetOrigin(Point::new(12.0, 3.5)).to_string(),
        "Set origin to (12.0, 3.5)"
    );
}

#[test]
fn describes_itself_for_the_history_list() {
    assert_eq!(
        Action::SetScalePoints(three_points()).to_string(),
        "Set 3-point scale points"
    );
    assert_eq!(
        Action::MoveScalePoint {
            index: 0,
            to: Point::new(1.0, 2.0)
        }
        .to_string(),
        "Move scale point 1 to (1.0, 2.0)"
    );
    assert_eq!(
        Action::SetDistance {
            line: Line::A,
            distance: Some(50.0)
        }
        .to_string(),
        "Set distance A to 50"
    );
    assert_eq!(
        Action::SetUnit(LengthUnit::Metre).to_string(),
        "Set units to m"
    );
    assert_eq!(
        Action::AddMeasurement(measurement(1.0, 9.0)).to_string(),
        "Add measurement from (1.0, 1.0) to (9.0, 9.0)"
    );
    assert_eq!(
        Action::MoveMeasurementPoint {
            index: 1,
            endpoint: Endpoint::End,
            to: Point::new(3.0, 4.0)
        }
        .to_string(),
        "Move measurement 2 end to (3.0, 4.0)"
    );
    assert_eq!(
        Action::SetMeasurementUnit {
            index: 0,
            unit: LengthUnit::Metre
        }
        .to_string(),
        "Set measurement 1 units to m"
    );
    assert_eq!(
        Action::DeleteMeasurement { index: 2 }.to_string(),
        "Delete measurement 3"
    );
}

fn measurement(from: f64, to: f64) -> Measurement {
    Measurement::new(
        Point::new(from, from),
        Point::new(to, to),
        LengthUnit::Centimetre,
    )
}

fn document_with_measurements(count: usize) -> Document {
    let mut document = Document::new(100, 50);
    for n in 0..count {
        Action::AddMeasurement(measurement(n as f64, n as f64 + 10.0))
            .apply(&mut document)
            .unwrap();
    }
    document
}

#[test]
fn adds_measurements_in_order() {
    let document = document_with_measurements(3);

    let starts: Vec<_> = document
        .measurements()
        .iter()
        .map(|m| m.start().x)
        .collect();
    assert_eq!(starts, [0.0, 1.0, 2.0]);
}

#[test]
fn rejects_measurements_outside_the_document() {
    let mut document = Document::new(100, 50);

    let result = Action::AddMeasurement(measurement(0.0, 60.0)).apply(&mut document);

    assert!(matches!(result, Err(ActionError::PointOutsideDocument(_))));
    assert!(document.measurements().is_empty());
}

#[test]
fn moves_a_measurement_endpoint() {
    let mut document = document_with_measurements(2);

    Action::MoveMeasurementPoint {
        index: 1,
        endpoint: Endpoint::Start,
        to: Point::new(40.0, 20.0),
    }
    .apply(&mut document)
    .unwrap();

    let moved = &document.measurements()[1];
    assert_eq!(moved.start(), Point::new(40.0, 20.0));
    assert_eq!(moved.end(), Point::new(11.0, 11.0));
}

#[test]
fn rejects_moving_a_measurement_endpoint_outside_the_document() {
    let mut document = document_with_measurements(1);
    let before = document.clone();

    let result = Action::MoveMeasurementPoint {
        index: 0,
        endpoint: Endpoint::End,
        to: Point::new(101.0, 0.0),
    }
    .apply(&mut document);

    assert!(result.is_err());
    assert_eq!(document, before);
}

#[test]
fn resizes_a_measurement_keeping_its_id_and_unit() {
    let mut document = document_with_measurements(2);
    let id = document.measurements()[1].id();

    Action::ResizeMeasurement {
        index: 1,
        start: Point::new(5.0, 6.0),
        end: Point::new(70.0, 40.0),
    }
    .apply(&mut document)
    .unwrap();

    let resized = &document.measurements()[1];
    assert_eq!(resized.id(), id);
    assert_eq!(resized.start(), Point::new(5.0, 6.0));
    assert_eq!(resized.end(), Point::new(70.0, 40.0));
    assert_eq!(resized.unit(), LengthUnit::Centimetre);
}

#[test]
fn rejects_resizing_outside_the_document_or_a_missing_measurement() {
    let mut document = document_with_measurements(1);
    let before = document.clone();
    let resize = |index, end| Action::ResizeMeasurement {
        index,
        start: Point::new(1.0, 1.0),
        end,
    };

    assert!(
        resize(0, Point::new(1.0, 51.0))
            .apply(&mut document)
            .is_err()
    );
    assert!(
        resize(1, Point::new(2.0, 2.0))
            .apply(&mut document)
            .is_err()
    );
    assert_eq!(document, before);
}

#[test]
fn describes_a_resize() {
    let action = Action::ResizeMeasurement {
        index: 0,
        start: Point::new(1.0, 2.0),
        end: Point::new(3.0, 4.0),
    };

    assert_eq!(
        action.to_string(),
        "Resize measurement 1 to (1.0, 2.0) - (3.0, 4.0)"
    );
}

#[test]
fn sets_a_measurement_unit() {
    let mut document = document_with_measurements(2);

    Action::SetMeasurementUnit {
        index: 0,
        unit: LengthUnit::Metre,
    }
    .apply(&mut document)
    .unwrap();

    assert_eq!(document.measurements()[0].unit(), LengthUnit::Metre);
    assert_eq!(document.measurements()[1].unit(), LengthUnit::Centimetre);
    assert_eq!(document.calibration().unit(), LengthUnit::Centimetre);
}

#[test]
fn deletes_a_measurement_keeping_the_order_of_the_rest() {
    let mut document = document_with_measurements(3);

    Action::DeleteMeasurement { index: 1 }
        .apply(&mut document)
        .unwrap();

    let starts: Vec<_> = document
        .measurements()
        .iter()
        .map(|m| m.start().x)
        .collect();
    assert_eq!(starts, [0.0, 2.0]);
}

#[test]
fn measurement_actions_reject_missing_indices() {
    let mut document = document_with_measurements(2);
    let before = document.clone();
    let out_of_range = ActionError::MeasurementIndexOutOfRange { index: 2, count: 2 };

    for action in [
        Action::MoveMeasurementPoint {
            index: 2,
            endpoint: Endpoint::Start,
            to: Point::default(),
        },
        Action::SetMeasurementUnit {
            index: 2,
            unit: LengthUnit::Metre,
        },
        Action::DeleteMeasurement { index: 2 },
    ] {
        assert_eq!(action.apply(&mut document), Err(out_of_range.clone()));
    }
    assert_eq!(document, before);
}

fn document_with_grid() -> Document {
    let mut document = document_with_points();
    Action::AddGrid(Grid::new(
        GridSource::ScaleLine(Line::A),
        LengthUnit::Centimetre,
    ))
    .apply(&mut document)
    .unwrap();
    document
}

#[test]
fn adds_a_grid_with_its_own_object_id() {
    let mut document = document_with_grid();
    Action::AddMeasurement(measurement(1.0, 2.0))
        .apply(&mut document)
        .unwrap();

    let grid_id = document.grids()[0].id();
    assert_ne!(grid_id, document.measurements()[0].id());
    assert!(document.contains_object(grid_id));
    assert_eq!(document.grid_index(grid_id), Some(0));
}

#[test]
fn rejects_a_grid_whose_source_does_not_exist() {
    let mut document = Document::new(100, 50);

    let result = Action::AddGrid(Grid::new(
        GridSource::ScaleLine(Line::B),
        LengthUnit::Centimetre,
    ))
    .apply(&mut document);

    assert_eq!(result, Err(ActionError::GridSourceMissing));
    assert!(document.grids().is_empty());
}

#[test]
fn edits_grid_properties() {
    let mut document = document_with_grid();

    for action in [
        Action::SetGridShape {
            index: 0,
            shape: CellShape::Rectangle,
        },
        Action::SetGridSize {
            index: 0,
            dimension: CellDimension::B,
            size: 4.5,
        },
        Action::SetGridColor {
            index: 0,
            color: Rgb::new(1, 2, 3),
        },
        Action::SetGridOpacity {
            index: 0,
            opacity: 0.25,
        },
    ] {
        action.apply(&mut document).unwrap();
    }

    let grid = &document.grids()[0];
    assert_eq!(grid.shape(), CellShape::Rectangle);
    assert_eq!(grid.cell_size(), (10.0, 4.5));
    assert_eq!(grid.color(), Rgb::new(1, 2, 3));
    assert_eq!(grid.opacity(), 0.25);
}

#[test]
fn rejects_invalid_grid_edits() {
    let mut document = document_with_grid();
    let before = document.clone();

    for action in [
        Action::SetGridSize {
            index: 0,
            dimension: CellDimension::Side,
            size: 0.0,
        },
        Action::SetGridSize {
            index: 0,
            dimension: CellDimension::A,
            size: f64::NAN,
        },
        Action::SetGridOpacity {
            index: 0,
            opacity: 1.5,
        },
        Action::SetGridShape {
            index: 1,
            shape: CellShape::Square,
        },
    ] {
        assert!(action.apply(&mut document).is_err(), "{action:?}");
    }
    assert_eq!(document, before);
}

#[test]
fn sets_a_grid_unit_keeping_the_size_values() {
    let mut document = document_with_grid();

    Action::SetGridUnit {
        index: 0,
        unit: LengthUnit::Metre,
    }
    .apply(&mut document)
    .unwrap();

    let grid = &document.grids()[0];
    assert_eq!(grid.unit(), LengthUnit::Metre);
    assert_eq!(grid.cell_size(), (10.0, 10.0));
    assert_eq!(document.calibration().unit(), LengthUnit::Centimetre);
}

#[test]
fn rejects_setting_the_unit_of_a_missing_grid() {
    let mut document = document_with_grid();
    let before = document.clone();

    let result = Action::SetGridUnit {
        index: 1,
        unit: LengthUnit::Metre,
    }
    .apply(&mut document);

    assert_eq!(
        result,
        Err(ActionError::GridIndexOutOfRange { index: 1, count: 1 })
    );
    assert_eq!(document, before);
}

#[test]
fn duplicates_a_measurement_with_a_new_id_and_the_same_properties() {
    let mut document = document_with_measurements(2);
    Action::SetMeasurementUnit {
        index: 0,
        unit: LengthUnit::Metre,
    }
    .apply(&mut document)
    .unwrap();

    Action::DuplicateMeasurement { index: 0 }
        .apply(&mut document)
        .unwrap();

    let [original, _, copy] = document.measurements() else {
        panic!("expected three measurements");
    };
    assert_eq!(**copy, **original);
    assert_ne!(copy.id(), original.id());
    assert_eq!(document.newest_object(), Some(copy.id()));
}

#[test]
fn duplicates_a_grid_with_a_new_id_and_the_same_properties() {
    let mut document = document_with_grid();
    for action in [
        Action::SetGridShape {
            index: 0,
            shape: CellShape::Rectangle,
        },
        Action::SetGridSize {
            index: 0,
            dimension: CellDimension::B,
            size: 4.5,
        },
        Action::SetGridColor {
            index: 0,
            color: Rgb::new(1, 2, 3),
        },
        Action::SetGridOpacity {
            index: 0,
            opacity: 0.25,
        },
        Action::DuplicateGrid { index: 0 },
    ] {
        action.apply(&mut document).unwrap();
    }

    let [original, copy] = document.grids() else {
        panic!("expected two grids");
    };
    assert_eq!(**copy, **original);
    assert_ne!(copy.id(), original.id());
    assert_eq!(document.newest_object(), Some(copy.id()));
}

#[test]
fn deletes_a_grid_keeping_the_order_of_the_rest() {
    let mut document = document_with_grid();
    for _ in 0..2 {
        Action::DuplicateGrid { index: 0 }
            .apply(&mut document)
            .unwrap();
    }
    let ids: Vec<_> = document.grids().iter().map(|grid| grid.id()).collect();

    Action::DeleteGrid { index: 1 }
        .apply(&mut document)
        .unwrap();

    let remaining: Vec<_> = document.grids().iter().map(|grid| grid.id()).collect();
    assert_eq!(remaining, [ids[0], ids[2]]);
}

#[test]
fn duplicate_and_delete_reject_missing_indices() {
    let mut document = document_with_grid();
    Action::AddMeasurement(measurement(1.0, 2.0))
        .apply(&mut document)
        .unwrap();
    let before = document.clone();

    assert_eq!(
        Action::DuplicateMeasurement { index: 1 }.apply(&mut document),
        Err(ActionError::MeasurementIndexOutOfRange { index: 1, count: 1 })
    );
    for action in [
        Action::DuplicateGrid { index: 1 },
        Action::DeleteGrid { index: 1 },
    ] {
        assert_eq!(
            action.apply(&mut document),
            Err(ActionError::GridIndexOutOfRange { index: 1, count: 1 })
        );
    }
    assert_eq!(document, before);
}

#[test]
fn finds_the_duplicate_and_delete_actions_for_an_object() {
    let mut document = document_with_grid();
    Action::AddMeasurement(measurement(1.0, 2.0))
        .apply(&mut document)
        .unwrap();
    let grid = document.grids()[0].id();
    let measurement = document.measurements()[0].id();

    assert_eq!(
        Action::duplicate_object(&document, measurement),
        Some(Action::DuplicateMeasurement { index: 0 })
    );
    assert_eq!(
        Action::delete_object(&document, measurement),
        Some(Action::DeleteMeasurement { index: 0 })
    );
    assert_eq!(
        Action::duplicate_object(&document, grid),
        Some(Action::DuplicateGrid { index: 0 })
    );
    assert_eq!(
        Action::delete_object(&document, grid),
        Some(Action::DeleteGrid { index: 0 })
    );

    Action::DeleteGrid { index: 0 }
        .apply(&mut document)
        .unwrap();
    assert_eq!(Action::duplicate_object(&document, grid), None);
    assert_eq!(Action::delete_object(&document, grid), None);
}

#[test]
fn only_adding_and_duplicating_activate_the_new_object() {
    assert!(Action::DuplicateMeasurement { index: 0 }.activates_new_object());
    assert!(Action::DuplicateGrid { index: 0 }.activates_new_object());
    assert!(
        Action::AddGrid(Grid::new(
            GridSource::ScaleLine(Line::A),
            LengthUnit::Centimetre
        ))
        .activates_new_object()
    );
    assert!(Action::AddMeasurement(measurement(1.0, 2.0)).activates_new_object());
    assert!(!Action::DeleteGrid { index: 0 }.activates_new_object());
}

#[test]
fn describes_duplicate_and_delete_actions() {
    assert_eq!(
        Action::DuplicateMeasurement { index: 0 }.to_string(),
        "Duplicate measurement 1"
    );
    assert_eq!(
        Action::DuplicateGrid { index: 1 }.to_string(),
        "Duplicate grid 2"
    );
    assert_eq!(Action::DeleteGrid { index: 2 }.to_string(), "Delete grid 3");
}

#[test]
fn describes_grid_actions() {
    let labels = [
        (
            Action::AddGrid(Grid::new(
                GridSource::ScaleLine(Line::A),
                LengthUnit::Centimetre,
            )),
            "Add grid aligned to scale line A",
        ),
        (
            Action::SetGridShape {
                index: 0,
                shape: CellShape::Rectangle,
            },
            "Set grid 1 cell shape to Rectangle",
        ),
        (
            Action::SetGridSize {
                index: 1,
                dimension: CellDimension::Side,
                size: 12.0,
            },
            "Set grid 2 size L to 12",
        ),
        (
            Action::SetGridUnit {
                index: 0,
                unit: LengthUnit::Metre,
            },
            "Set grid 1 units to m",
        ),
        (
            Action::SetGridColor {
                index: 0,
                color: Rgb::new(255, 0, 0),
            },
            "Set grid 1 color to #FF0000",
        ),
        (
            Action::SetGridOpacity {
                index: 0,
                opacity: 0.25,
            },
            "Set grid 1 opacity to 25%",
        ),
    ];
    for (action, label) in labels {
        assert_eq!(action.to_string(), label);
    }
}

fn triangle(offset: f64) -> Area {
    Area::new(
        vec![
            Point::new(offset, 10.0),
            Point::new(offset + 20.0, 10.0),
            Point::new(offset + 10.0, 30.0),
        ],
        LengthUnit::Centimetre,
    )
    .unwrap()
}

fn document_with_areas(count: usize) -> Document {
    let mut document = Document::new(100, 50);
    for n in 0..count {
        Action::AddArea(triangle(n as f64 * 10.0))
            .apply(&mut document)
            .unwrap();
    }
    document
}

#[test]
fn adds_areas_in_order_with_their_own_ids() {
    let mut document = document_with_areas(2);
    Action::AddMeasurement(measurement(1.0, 2.0))
        .apply(&mut document)
        .unwrap();

    let [first, second] = document.areas() else {
        panic!("expected two areas");
    };
    assert_eq!(**first, triangle(0.0));
    assert_eq!(second.points()[0], Point::new(10.0, 10.0));
    assert_ne!(first.id(), second.id());
    assert_eq!(document.area_index(second.id()), Some(1));
    assert!(document.contains_object(first.id()));
    assert_ne!(document.measurements()[0].id(), second.id());
}

#[test]
fn rejects_areas_outside_the_document() {
    let mut document = Document::new(100, 50);

    let result = Action::AddArea(triangle(90.0)).apply(&mut document);

    assert!(matches!(result, Err(ActionError::PointOutsideDocument(_))));
    assert!(document.areas().is_empty());
}

#[test]
fn moves_one_area_point() {
    let mut document = document_with_areas(1);

    Action::MoveAreaPoint {
        index: 0,
        point: 2,
        to: Point::new(15.0, 45.0),
    }
    .apply(&mut document)
    .unwrap();

    let area = &document.areas()[0];
    assert_eq!(area.points()[2], Point::new(15.0, 45.0));
    assert_eq!(area.points()[0], Point::new(0.0, 10.0));
}

#[test]
fn resizes_an_area_keeping_its_id_unit_and_colour() {
    let mut document = document_with_areas(1);
    let id = document.areas()[0].id();
    let points = vec![
        Point::new(0.0, 0.0),
        Point::new(40.0, 0.0),
        Point::new(20.0, 50.0),
    ];

    Action::ResizeArea {
        index: 0,
        points: points.clone(),
    }
    .apply(&mut document)
    .unwrap();

    let area = &document.areas()[0];
    assert_eq!(area.id(), id);
    assert_eq!(area.points(), points);
    assert_eq!(area.unit(), LengthUnit::Centimetre);
    assert_eq!(area.line().color, Rgb::CYAN);
}

#[test]
fn rejects_invalid_area_edits() {
    let mut document = document_with_areas(1);
    let before = document.clone();
    let missing = ActionError::AreaIndexOutOfRange { index: 1, count: 1 };

    let cases = [
        (
            Action::MoveAreaPoint {
                index: 0,
                point: 3,
                to: Point::default(),
            },
            ActionError::AreaPointIndexOutOfRange { point: 3, count: 3 },
        ),
        (
            Action::MoveAreaPoint {
                index: 0,
                point: 0,
                to: Point::new(0.0, 51.0),
            },
            ActionError::PointOutsideDocument(Point::new(0.0, 51.0)),
        ),
        (
            Action::ResizeArea {
                index: 0,
                points: vec![Point::default(); 4],
            },
            ActionError::AreaPointCountChanged {
                expected: 3,
                actual: 4,
            },
        ),
        (
            Action::ResizeArea {
                index: 1,
                points: vec![Point::default(); 3],
            },
            missing.clone(),
        ),
        (
            Action::SetAreaUnit {
                index: 1,
                unit: LengthUnit::Metre,
            },
            missing.clone(),
        ),
        (Action::DuplicateArea { index: 1 }, missing.clone()),
        (Action::DeleteArea { index: 1 }, missing),
    ];
    for (action, error) in cases {
        assert_eq!(action.apply(&mut document), Err(error), "{action:?}");
    }
    assert_eq!(document, before);
}

#[test]
fn sets_area_and_measurement_units_and_colours() {
    let mut document = document_with_areas(1);
    Action::AddMeasurement(measurement(1.0, 2.0))
        .apply(&mut document)
        .unwrap();

    for action in [
        Action::SetAreaUnit {
            index: 0,
            unit: LengthUnit::Metre,
        },
        Action::SetStyle {
            target: StyleTarget::Area(0),
            edit: StyleEdit::LineColor(Rgb::new(9, 8, 7)),
        },
        Action::SetStyle {
            target: StyleTarget::Measurement(0),
            edit: StyleEdit::LineColor(Rgb::new(1, 2, 3)),
        },
    ] {
        action.apply(&mut document).unwrap();
    }

    let area = &document.areas()[0];
    assert_eq!(area.unit(), LengthUnit::Metre);
    assert_eq!(area.line().color, Rgb::new(9, 8, 7));
    assert_eq!(document.measurements()[0].line().color, Rgb::new(1, 2, 3));
    assert_eq!(document.calibration().unit(), LengthUnit::Centimetre);
}

#[test]
fn duplicates_and_deletes_areas() {
    let mut document = document_with_areas(2);
    Action::SetStyle {
        target: StyleTarget::Area(0),
        edit: StyleEdit::LineColor(Rgb::new(1, 2, 3)),
    }
    .apply(&mut document)
    .unwrap();

    Action::DuplicateArea { index: 0 }
        .apply(&mut document)
        .unwrap();
    let ids: Vec<_> = document.areas().iter().map(|area| area.id()).collect();
    let [original, _, copy] = document.areas() else {
        panic!("expected three areas");
    };
    assert_eq!(**copy, **original);
    assert_ne!(copy.id(), original.id());
    assert_eq!(document.newest_object(), Some(copy.id()));

    Action::DeleteArea { index: 1 }
        .apply(&mut document)
        .unwrap();
    let remaining: Vec<_> = document.areas().iter().map(|area| area.id()).collect();
    assert_eq!(remaining, [ids[0], ids[2]]);
}

#[test]
fn finds_the_duplicate_and_delete_actions_for_an_area() {
    let document = document_with_areas(1);
    let id = document.areas()[0].id();

    assert_eq!(
        Action::duplicate_object(&document, id),
        Some(Action::DuplicateArea { index: 0 })
    );
    assert_eq!(
        Action::delete_object(&document, id),
        Some(Action::DeleteArea { index: 0 })
    );
    assert!(Action::DuplicateArea { index: 0 }.activates_new_object());
    assert!(Action::AddArea(triangle(0.0)).activates_new_object());
}

#[test]
fn the_focus_point_of_an_area_is_its_centre() {
    let document = document_with_areas(1);
    let id = document.areas()[0].id();

    assert_eq!(
        document.object_focus_point(id),
        Some(document.areas()[0].center())
    );
}

#[test]
fn the_outline_of_an_area_is_its_closed_points() {
    let document = document_with_areas(1);
    let area = &document.areas()[0];

    assert_eq!(
        document.object_outline(area.id()),
        Some(Outline {
            points: area.points().to_vec(),
            closed: true,
        })
    );
}

#[test]
fn the_outline_of_a_measurement_is_its_open_line() {
    let mut document = Document::new(100, 50);
    let added = measurement(1.0, 2.0);
    Action::AddMeasurement(added.clone())
        .apply(&mut document)
        .unwrap();

    assert_eq!(
        document.object_outline(document.measurements()[0].id()),
        Some(Outline {
            points: vec![added.start(), added.end()],
            closed: false,
        })
    );
}

#[test]
fn the_outline_of_a_grid_is_its_alignment_line() {
    let document = document_with_grid();
    let grid = &document.grids()[0];
    let (start, end) = grid.source_line(&document).unwrap();

    assert_eq!(
        document.object_outline(grid.id()),
        Some(Outline {
            points: vec![start, end],
            closed: false,
        })
    );
    assert_eq!(document.object_outline(ObjectId::new(999)), None);
}

#[test]
fn describes_area_and_colour_actions() {
    let labels = [
        (Action::AddArea(triangle(0.0)), "Add area with 3 points"),
        (
            Action::MoveAreaPoint {
                index: 0,
                point: 2,
                to: Point::new(1.0, 2.0),
            },
            "Move area 1 point 3 to (1.0, 2.0)",
        ),
        (
            Action::ResizeArea {
                index: 1,
                points: Vec::new(),
            },
            "Resize area 2",
        ),
        (
            Action::SetAreaUnit {
                index: 0,
                unit: LengthUnit::Metre,
            },
            "Set area 1 units to m",
        ),
        (
            Action::SetStyle {
                target: StyleTarget::Area(0),
                edit: StyleEdit::LineColor(Rgb::CYAN),
            },
            "Set area 1 line color to #00FFFF",
        ),
        (Action::DeleteArea { index: 2 }, "Delete area 3"),
        (Action::DuplicateArea { index: 0 }, "Duplicate area 1"),
        (
            Action::SetStyle {
                target: StyleTarget::Measurement(1),
                edit: StyleEdit::LineColor(Rgb::new(255, 0, 0)),
            },
            "Set measurement 2 line color to #FF0000",
        ),
    ];
    for (action, label) in labels {
        assert_eq!(action.to_string(), label);
    }
}

fn document_with_area_and_measurement() -> Document {
    let mut document = document_with_areas(1);
    Action::AddMeasurement(measurement(1.0, 2.0))
        .apply(&mut document)
        .unwrap();
    document
}

#[test]
fn styles_a_measurement_and_an_area() {
    use crate::style::{FillStyle, LineStyle};

    let mut document = document_with_area_and_measurement();
    let line_edit = StyleEdit::LineStyle(LineStyle::Dashed);
    for action in [
        Action::SetStyle {
            target: StyleTarget::Measurement(0),
            edit: line_edit,
        },
        Action::SetStyle {
            target: StyleTarget::Area(0),
            edit: line_edit,
        },
        Action::SetStyle {
            target: StyleTarget::Area(0),
            edit: StyleEdit::FillStyle(FillStyle::Crosshatch, 90.0),
        },
    ] {
        action.apply(&mut document).unwrap();
    }

    assert_eq!(document.measurements()[0].line().style, LineStyle::Dashed);
    assert_eq!(document.areas()[0].line().style, LineStyle::Dashed);
    assert_eq!(
        document.areas()[0].stored_fill().style,
        FillStyle::Crosshatch
    );
}

#[test]
fn rejects_invalid_style_edits_of_measurements_and_areas() {
    let mut document = document_with_area_and_measurement();
    let before = document.clone();
    let cases = [
        (
            StyleTarget::Measurement(0),
            StyleEdit::FillOpacity(0.5),
            ActionError::HasNoFill(StyleTarget::Measurement(0)),
        ),
        (
            StyleTarget::Measurement(0),
            StyleEdit::LineOpacity(1.5),
            ActionError::InvalidOpacity(1.5),
        ),
        (
            StyleTarget::Area(0),
            StyleEdit::FillSpacing(0.0),
            ActionError::InvalidFillSpacing(0.0),
        ),
        (
            StyleTarget::Area(0),
            StyleEdit::FillAngle(360.0),
            ActionError::InvalidFillAngle(360.0),
        ),
        (
            StyleTarget::Area(9),
            StyleEdit::LineOpacity(0.5),
            ActionError::AreaIndexOutOfRange { index: 9, count: 1 },
        ),
        (
            StyleTarget::Measurement(9),
            StyleEdit::LineOpacity(0.5),
            ActionError::MeasurementIndexOutOfRange { index: 9, count: 1 },
        ),
    ];
    for (target, edit, error) in cases {
        let action = Action::SetStyle { target, edit };
        assert_eq!(action.apply(&mut document), Err(error), "{action:?}");
    }
    assert_eq!(document, before);
}

#[test]
fn rejects_new_objects_with_invalid_styles() {
    let mut document = document_with_areas(1);
    let mut measurement = measurement(1.0, 2.0);
    measurement.line_mut().opacity = 1.5;
    assert_eq!(
        Action::AddMeasurement(measurement).apply(&mut document),
        Err(ActionError::InvalidOpacity(1.5))
    );

    let mut area = (*document.areas()[0]).clone();
    area.fill_mut().unwrap().spacing = -1.0;
    assert_eq!(
        Action::AddArea(area).apply(&mut document),
        Err(ActionError::InvalidFillSpacing(-1.0))
    );
}

#[test]
fn moving_replaces_every_point_and_keeps_everything_else() {
    let mut document = document_with_measurements(1);
    Action::AddArea(triangle(0.0)).apply(&mut document).unwrap();
    let area_points: Vec<Point> = document.areas()[0]
        .points()
        .iter()
        .map(|point| Point::new(point.x + 1.0, point.y + 2.0))
        .collect();
    let (measurement_before, area_before) = (
        document.measurements()[0].clone(),
        document.areas()[0].clone(),
    );

    Action::MoveMeasurement {
        index: 0,
        start: Point::new(1.0, 2.0),
        end: Point::new(11.0, 12.0),
    }
    .apply(&mut document)
    .unwrap();
    Action::MoveArea {
        index: 0,
        points: area_points.clone(),
    }
    .apply(&mut document)
    .unwrap();

    let measurement = &document.measurements()[0];
    assert_eq!(measurement.start(), Point::new(1.0, 2.0));
    assert_eq!(measurement.end(), Point::new(11.0, 12.0));
    assert_eq!(measurement.id(), measurement_before.id());
    assert_eq!(measurement.unit(), measurement_before.unit());
    let area = &document.areas()[0];
    assert_eq!(area.points(), area_points);
    assert_eq!(area.id(), area_before.id());
    assert_eq!(area.unit(), area_before.unit());
}

#[test]
fn moves_outside_the_document_or_of_missing_objects_are_rejected() {
    let mut document = document_with_measurements(1);
    Action::AddArea(triangle(0.0)).apply(&mut document).unwrap();
    let before = document.clone();
    let inside = Point::new(1.0, 1.0);
    let outside = Point::new(500.0, 1.0);
    let cases = [
        (
            Action::MoveMeasurement {
                index: 0,
                start: inside,
                end: outside,
            },
            ActionError::PointOutsideDocument(outside),
        ),
        (
            Action::MoveMeasurement {
                index: 1,
                start: inside,
                end: inside,
            },
            ActionError::MeasurementIndexOutOfRange { index: 1, count: 1 },
        ),
        (
            Action::MoveArea {
                index: 0,
                points: vec![outside; 3],
            },
            ActionError::PointOutsideDocument(outside),
        ),
        (
            Action::MoveArea {
                index: 0,
                points: vec![inside],
            },
            ActionError::AreaPointCountChanged {
                expected: 3,
                actual: 1,
            },
        ),
        (
            Action::MoveArea {
                index: 1,
                points: vec![inside; 3],
            },
            ActionError::AreaIndexOutOfRange { index: 1, count: 1 },
        ),
    ];

    for (action, error) in cases {
        assert_eq!(action.apply(&mut document), Err(error), "{action:?}");
    }
    assert_eq!(document, before);
}

#[test]
fn move_labels_name_the_object() {
    let point = Point::new(1.0, 2.0);
    let cases = [
        (
            Action::MoveMeasurement {
                index: 0,
                start: point,
                end: Point::new(3.0, 4.0),
            },
            "Move measurement 1 to (1.0, 2.0) - (3.0, 4.0)",
        ),
        (
            Action::MoveArea {
                index: 1,
                points: vec![point],
            },
            "Move area 2",
        ),
        (
            Action::MoveShape {
                index: 2,
                points: vec![point],
            },
            "Move shape 3",
        ),
    ];
    for (action, label) in cases {
        assert_eq!(action.to_string(), label);
    }
}
