use super::*;
use crate::action::Action;
use crate::area::Area;
use crate::calibration::{CalibrationMethod, Line, ScalePoints};
use crate::color::Rgb;
use crate::grid::{CellDimension, CellShape, Grid, GridSource};
use crate::measurement::Measurement;
use crate::units::LengthUnit;

/// A document using every feature: calibration, measurements with a gap in their ids and
/// a changed colour, areas with every property changed, grids with every property
/// changed (one aligned to a deleted measurement), a moved origin.
fn rich_document() -> Document {
    let mut document = Document::new(200, 100);
    let measurement = |x: f64| {
        Action::AddMeasurement(Measurement::new(
            Point::new(x, 10.0),
            Point::new(x + 20.0, 60.0),
            LengthUnit::Centimetre,
        ))
    };
    let actions = [
        Action::SetScalePoints(
            ScalePoints::new(
                CalibrationMethod::FourPoint,
                vec![
                    Point::new(1.0, 2.0),
                    Point::new(101.0, 2.0),
                    Point::new(5.5, 3.0),
                    Point::new(5.5, 93.25),
                ],
            )
            .unwrap(),
        ),
        Action::SetDistance {
            line: Line::A,
            distance: Some(12.5),
        },
        Action::SetDistance {
            line: Line::B,
            distance: Some(0.3),
        },
        Action::SetUnit(LengthUnit::Metre),
        measurement(10.0),
        measurement(50.0),
        measurement(90.0),
        Action::SetMeasurementUnit {
            index: 2,
            unit: LengthUnit::Metre,
        },
        Action::SetMeasurementColor {
            index: 0,
            color: Rgb::new(200, 100, 0),
        },
        Action::AddArea(
            Area::new(
                vec![
                    Point::new(10.0, 10.0),
                    Point::new(60.0, 15.5),
                    Point::new(30.0, 40.0),
                    Point::new(20.0, 25.0),
                ],
                LengthUnit::Centimetre,
            )
            .unwrap(),
        ),
        Action::DuplicateArea { index: 0 },
        Action::SetAreaUnit {
            index: 1,
            unit: LengthUnit::Metre,
        },
        Action::SetAreaColor {
            index: 1,
            color: Rgb::new(1, 2, 3),
        },
        Action::DeleteArea { index: 0 },
        Action::AddGrid(Grid::new(GridSource::ScaleLine(Line::B), LengthUnit::Metre)),
        Action::SetGridShape {
            index: 0,
            shape: CellShape::Rectangle,
        },
        Action::SetGridSize {
            index: 0,
            dimension: CellDimension::B,
            size: 0.01,
        },
        Action::SetGridColor {
            index: 0,
            color: Rgb::new(12, 34, 56),
        },
        Action::SetGridOpacity {
            index: 0,
            opacity: 0.25,
        },
        Action::SetOrigin(Point::new(33.5, 66.25)),
    ];
    for action in actions {
        action.apply(&mut document).unwrap();
    }
    // A grid aligned to measurement 2, which is then deleted.
    let doomed = document.measurements()[1].id();
    Action::AddGrid(Grid::new(
        GridSource::Measurement(doomed),
        LengthUnit::Centimetre,
    ))
    .apply(&mut document)
    .unwrap();
    Action::DeleteMeasurement { index: 1 }
        .apply(&mut document)
        .unwrap();
    document
}

#[test]
fn a_document_survives_a_round_trip_exactly() {
    let document = rich_document();

    let text = DocumentFile::describe(&document, "photo.png".to_owned())
        .to_ron()
        .unwrap();
    let file = DocumentFile::from_ron(&text).unwrap();

    assert_eq!(file.image(), "photo.png");
    assert_eq!(file.to_document().unwrap(), document);
}

#[test]
fn the_round_trip_keeps_ids_so_new_objects_do_not_reuse_them() {
    let document = rich_document();
    let mut loaded = DocumentFile::describe(&document, String::new())
        .to_document()
        .unwrap();
    let mut original = document.clone();

    for document in [&mut loaded, &mut original] {
        Action::DuplicateGrid { index: 0 }.apply(document).unwrap();
    }

    assert_eq!(loaded, original);
}

#[test]
fn files_are_tagged_with_their_format_version() {
    let text = DocumentFile::describe(&Document::new(4, 3), "a.png".to_owned())
        .to_ron()
        .unwrap();

    assert!(text.starts_with("V2("), "{text}");
}

/// A version 1 file exactly as the first release writes it. It must always load.
const VERSION_1_SAMPLE: &str = r#"V1((
    image: "images/photo.png",
    width: 200,
    height: 100,
    origin: (x: 33.5, y: 66.25),
    calibration: (
        points: Some((method: ThreePoint, points: [(x: 0.0, y: 0.0), (x: 20.0, y: 0.0), (x: 20.0, y: 40.0)])),
        distance_a: Some(20.0),
        distance_b: None,
        unit: Metre,
    ),
    measurements: [(id: 1, start: (x: 1.0, y: 2.0), end: (x: 3.0, y: 4.0), unit: Centimetre)],
    grids: [(
        id: 3,
        source: Measurement(1),
        shape: Rectangle,
        size_a: 10.0,
        size_b: 2.5,
        unit: Centimetre,
        color: (r: 0, g: 255, b: 0),
        opacity: 0.5,
    )],
    next_object_id: 4,
))"#;

#[test]
fn version_1_files_load() {
    let file = DocumentFile::from_ron(VERSION_1_SAMPLE).unwrap();
    let document = file.to_document().unwrap();

    assert_eq!(file.image(), "images/photo.png");
    assert_eq!((document.width(), document.height()), (200, 100));
    assert_eq!(document.origin(), Point::new(33.5, 66.25));
    assert_eq!(document.calibration().distance(Line::A), Some(20.0));
    assert_eq!(document.calibration().distance(Line::B), None);
    assert_eq!(document.calibration().unit(), LengthUnit::Metre);
    let measurement = &document.measurements()[0];
    assert_eq!(measurement.end(), Point::new(3.0, 4.0));
    let grid = &document.grids()[0];
    assert_eq!(grid.source(), GridSource::Measurement(measurement.id()));
    assert_eq!(grid.cell_size(), (10.0, 2.5));
    assert_eq!(document.next_object_id(), 4);
}

/// A version 2 file exactly as the release that added areas writes it. It must always
/// load.
const VERSION_2_SAMPLE: &str = r#"V2((
    image: "photo.png",
    width: 200,
    height: 100,
    origin: (x: 100.0, y: 50.0),
    calibration: (
        points: None,
        distance_a: None,
        distance_b: None,
        unit: Centimetre,
    ),
    measurements: [(id: 1, start: (x: 1.0, y: 2.0), end: (x: 3.0, y: 4.0), unit: Metre, color: (r: 255, g: 0, b: 0))],
    areas: [(
        id: 2,
        points: [(x: 10.0, y: 10.0), (x: 50.0, y: 10.0), (x: 50.0, y: 40.0)],
        unit: Metre,
        color: (r: 0, g: 255, b: 255),
    )],
    grids: [],
    next_object_id: 3,
))"#;

#[test]
fn version_1_files_get_default_colours_and_no_areas() {
    let document = DocumentFile::from_ron(VERSION_1_SAMPLE)
        .unwrap()
        .to_document()
        .unwrap();

    assert_eq!(document.measurements()[0].color(), Rgb::BLUE);
    assert!(document.areas().is_empty());
}

#[test]
fn version_2_files_load() {
    let document = DocumentFile::from_ron(VERSION_2_SAMPLE)
        .unwrap()
        .to_document()
        .unwrap();

    let measurement = &document.measurements()[0];
    assert_eq!(measurement.color(), Rgb::new(255, 0, 0));
    assert_eq!(measurement.unit(), LengthUnit::Metre);
    let area = &document.areas()[0];
    assert_eq!(area.points().len(), 3);
    assert_eq!(area.points()[1], Point::new(50.0, 10.0));
    assert_eq!(area.unit(), LengthUnit::Metre);
    assert_eq!(area.color(), Rgb::CYAN);
    assert_eq!(document.next_object_id(), 3);
}

#[test]
fn invalid_contents_are_rejected() {
    let valid = || match DocumentFile::from_ron(VERSION_1_SAMPLE).unwrap() {
        DocumentFile::V1(file) => DocumentV2::from(file),
        DocumentFile::V2(_) => unreachable!("the sample is version 1"),
    };
    type Corruption = fn(&mut DocumentV2);
    let cases: [(Corruption, InvalidDocument); 7] = [
        (|f| f.width = 0, InvalidDocument::EmptySize),
        (
            |f| f.origin.x = 200.5,
            InvalidDocument::PointOutside(Point::new(200.5, 66.25)),
        ),
        (
            |f| {
                f.calibration
                    .points
                    .as_mut()
                    .unwrap()
                    .points
                    .pop()
                    .map(drop)
                    .unwrap()
            },
            InvalidDocument::WrongScalePointCount,
        ),
        (
            |f| f.calibration.distance_a = Some(-1.0),
            InvalidDocument::NotPositive(-1.0),
        ),
        (
            |f| f.grids[0].opacity = 1.5,
            InvalidDocument::BadOpacity(1.5),
        ),
        (|f| f.grids[0].id = 1, InvalidDocument::BadObjectId(1)),
        (|f| f.next_object_id = 3, InvalidDocument::BadObjectId(3)),
    ];

    for (corrupt, expected) in cases {
        let mut file = valid();
        corrupt(&mut file);
        assert_eq!(file.to_document(), Err(expected));
    }
}

#[test]
fn images_beside_or_below_the_document_are_referenced_relatively() {
    let folder = std::env::temp_dir().join("gridtool_docs");
    let document = folder.join("plan.ron");

    assert_eq!(image_reference(&folder.join("a.png"), &document), "a.png");
    assert_eq!(
        image_reference(&folder.join("images").join("b.png"), &document),
        "images/b.png"
    );
    let elsewhere = std::env::temp_dir().join("other").join("c.png");
    assert_eq!(
        image_reference(&elsewhere, &document),
        elsewhere.to_string_lossy()
    );

    assert_eq!(
        resolve_image("images/b.png", &document),
        folder.join("images/b.png")
    );
    assert_eq!(
        resolve_image(&elsewhere.to_string_lossy(), &document),
        elsewhere
    );
}

#[test]
fn the_document_extension_is_added_when_missing() {
    assert_eq!(with_extension(Path::new("plan")), PathBuf::from("plan.ron"));
    assert_eq!(
        with_extension(Path::new("plan.RON")),
        PathBuf::from("plan.RON")
    );
    assert_eq!(
        with_extension(Path::new("plan.txt")),
        PathBuf::from("plan.txt.ron")
    );
}

/// A fresh folder in the temp directory, removed when dropped.
struct TempFolder(PathBuf);

impl TempFolder {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "gridtool_{name}_{}_{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}

impl Drop for TempFolder {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).ok();
    }
}

fn write_png(path: &Path, width: u32, height: u32) {
    image::RgbaImage::from_pixel(width, height, image::Rgba([1, 2, 3, 255]))
        .save(path)
        .unwrap();
}

#[test]
fn saving_and_loading_restores_the_document_and_its_image() {
    let folder = TempFolder::new("save_load");
    let image_path = folder.0.join("photo.png");
    write_png(&image_path, 200, 100);
    let path = folder.0.join("plan.ron");
    let document = rich_document();

    save(&path, &document, &image_path).unwrap();
    let loaded = load(&path).unwrap();

    assert_eq!(loaded.document, document);
    assert_eq!(loaded.image_path, image_path);
    assert_eq!(loaded.image.size(), [200, 100]);
    assert!(
        fs::read_to_string(&path)
            .unwrap()
            .contains("image: \"photo.png\"")
    );
}

#[test]
fn saving_replaces_an_existing_file() {
    let folder = TempFolder::new("overwrite");
    let image_path = folder.0.join("photo.png");
    write_png(&image_path, 4, 3);
    let path = folder.0.join("plan.ron");
    fs::write(&path, "old contents").unwrap();

    save(&path, &Document::new(4, 3), &image_path).unwrap();

    assert!(load(&path).is_ok());
    assert!(!folder.0.join("plan.ron.tmp").exists());
}

#[test]
fn loading_reports_what_went_wrong() {
    let folder = TempFolder::new("load_errors");
    let path = folder.0.join("plan.ron");

    assert!(matches!(load(&path), Err(LoadError::Io(_))));

    fs::write(&path, "not a document").unwrap();
    assert!(matches!(load(&path), Err(LoadError::Parse(_))));

    save(&path, &Document::new(4, 3), &folder.0.join("missing.png")).unwrap();
    assert!(matches!(load(&path), Err(LoadError::Image { .. })));

    write_png(&folder.0.join("missing.png"), 5, 3);
    assert!(matches!(
        load(&path),
        Err(LoadError::ImageSizeMismatch {
            expected: (4, 3),
            actual: (5, 3),
            ..
        })
    ));
}

#[test]
fn invalid_areas_are_rejected() {
    let valid = || match DocumentFile::from_ron(VERSION_2_SAMPLE).unwrap() {
        DocumentFile::V2(file) => file,
        DocumentFile::V1(_) => unreachable!("the sample is version 2"),
    };

    let mut two_points = valid();
    two_points.areas[0].points.pop();
    assert_eq!(
        two_points.to_document(),
        Err(InvalidDocument::TooFewAreaPoints(2))
    );

    let mut outside = valid();
    outside.areas[0].points[0].y = -1.0;
    assert_eq!(
        outside.to_document(),
        Err(InvalidDocument::PointOutside(Point::new(10.0, -1.0)))
    );

    let mut reused_id = valid();
    reused_id.areas[0].id = 1;
    assert_eq!(
        reused_id.to_document(),
        Err(InvalidDocument::BadObjectId(1))
    );
}
