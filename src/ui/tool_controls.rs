//! The row under the tool bar. It is always there, at least as tall as the tool bar, so the
//! canvas does not move when its contents change. It shows the controls of the active object
//! or, with none, of the active tool, and wraps onto more rows when they do not fit. A tool
//! with no controls shows help text instead.

use eframe::egui;
use gridtool::action::{Action, StyleTarget};
use gridtool::document::Document;
use gridtool::object::{ObjectId, ObjectKind};
use gridtool::shape::DrawnShape;
use gridtool::style::{StyleEdit, Styled};

use super::document_view::DocumentView;
use super::grid_bar::GridBar;
use super::new_object_settings::NewObjectSettings;
use super::rotation_bar::RotationBar;
use super::scale_bar::ScaleBar;
use super::selection::Selection;
use super::style_controls::{StyleControls, StyleEvents};
use super::tool_mode::ToolMode;
use super::unit_picker;

const PAN_HELP: &str = "Drag to pan the view. Scroll to zoom.";
const ORIGIN_HELP: &str =
    "Click on the image to move the origin, the point the view rotates about.";

/// "Shape <n> (line)" or "Shape <n> (area)".
pub fn shape_label(index: usize, shape: &DrawnShape) -> String {
    let kind = if shape.is_closed() { "area" } else { "line" };
    format!("Shape {} ({kind})", index + 1)
}

#[derive(Default)]
pub struct ToolControls {
    rotation_bar: RotationBar,
    scale_bar: ScaleBar,
    grid_bar: GridBar,
    style: StyleControls,
}

impl ToolControls {
    /// Shows the row, at least `min_height` tall, returning the action the user completed,
    /// if any. There is nothing to control without an open document.
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        mode: ToolMode,
        min_height: f32,
        open: Option<(&mut DocumentView, &Document)>,
    ) -> Option<Action> {
        egui::Panel::top("tool_controls")
            .show(ui, |ui| {
                ui.set_min_height(min_height);
                ui.horizontal_wrapped(|ui| {
                    ui.set_row_height(min_height);
                    let (view, document) = open?;
                    let active = view.selection.active().and_then(|id| {
                        let kind = document.object_kind(id)?;
                        Some((id, kind))
                    });
                    match active {
                        Some((id, kind)) => self.object_controls(ui, id, kind, view, document),
                        None => self.tool_options(ui, mode, view, document),
                    }
                })
                .inner
            })
            .inner
    }

    fn object_controls(
        &mut self,
        ui: &mut egui::Ui,
        id: ObjectId,
        kind: ObjectKind,
        view: &mut DocumentView,
        document: &Document,
    ) -> Option<Action> {
        let selection = &mut view.selection;
        // Every widget must be drawn each frame, so collect actions eagerly.
        let mut actions = Vec::new();
        match kind {
            ObjectKind::Grid => {
                let zoom = view.viewport.zoom();
                actions.extend(self.grid_bar.show(ui, document, selection, zoom));
            }
            ObjectKind::Measurement => {
                let index = document.measurement_index(id)?;
                let stored = &document.measurements()[index];
                ui.strong(format!("Measurement {}", index + 1));
                ui.separator();
                actions.extend(
                    unit_picker::show(
                        ui,
                        ("object_unit", id),
                        stored.unit(),
                        "Units of this measurement",
                    )
                    .map(|unit| Action::SetMeasurementUnit { index, unit }),
                );
                ui.separator();
                let displayed = selection.displayed(id, stored);
                let target = StyleTarget::Measurement(index);
                let settings = &mut view.measure_tool.settings;
                actions.extend(self.object_style(
                    ui, id, target, &**stored, &displayed, selection, settings, document,
                ));
            }
            ObjectKind::Area => {
                let index = document.area_index(id)?;
                let stored = &document.areas()[index];
                ui.strong(format!("Area {}", index + 1));
                ui.separator();
                actions.extend(
                    unit_picker::show_area(
                        ui,
                        ("object_unit", id),
                        stored.unit(),
                        "Units of this area",
                    )
                    .map(|unit| Action::SetAreaUnit { index, unit }),
                );
                ui.separator();
                let displayed = selection.displayed_area(id, stored);
                let target = StyleTarget::Area(index);
                let settings = &mut view.area_tool.settings;
                actions.extend(self.object_style(
                    ui, id, target, &**stored, &displayed, selection, settings, document,
                ));
            }
            ObjectKind::Shape => {
                let index = document.shape_index(id)?;
                let stored = &document.shapes()[index];
                ui.strong(shape_label(index, stored));
                ui.separator();
                let displayed = selection.displayed_drawn(id, stored);
                let target = StyleTarget::Shape(index);
                let settings = &mut view.line_tool.settings;
                actions.extend(self.object_style(
                    ui, id, target, &**stored, &displayed, selection, settings, document,
                ));
            }
        }
        actions.into_iter().next()
    }

    /// The style controls of an object: edits being chosen preview on the canvas, and a
    /// completed one becomes an action. The fill angles last set are kept with the `settings`
    /// of the object's tool.
    #[allow(clippy::too_many_arguments)]
    fn object_style<T: Styled>(
        &mut self,
        ui: &mut egui::Ui,
        id: ObjectId,
        target: StyleTarget,
        stored: &T,
        displayed: &T,
        selection: &mut Selection,
        settings: &mut NewObjectSettings,
        document: &Document,
    ) -> Option<Action> {
        let events = self.style.show(
            ui,
            egui::Id::unique(("object_style", id)),
            stored,
            displayed,
            scale_problem(document),
            settings.fill_angles(),
        );
        if let (Some(StyleEdit::FillAngle(degrees)), Some(fill)) = (events.commit, stored.fill()) {
            settings.remember_fill_angle(fill.style, degrees);
        }
        if events.clear_preview {
            selection.clear_style_preview();
        }
        if let Some(edit) = events.preview {
            selection.preview_style(edit);
        }
        events.commit.map(|edit| Action::SetStyle { target, edit })
    }

    fn tool_options(
        &mut self,
        ui: &mut egui::Ui,
        mode: ToolMode,
        view: &mut DocumentView,
        document: &Document,
    ) -> Option<Action> {
        match mode {
            ToolMode::Pan => {
                ui.weak(PAN_HELP);
                None
            }
            ToolMode::Origin => {
                ui.weak(ORIGIN_HELP);
                None
            }
            ToolMode::Rotate => {
                self.rotation_bar.show(ui, &mut view.viewport, document);
                None
            }
            ToolMode::Scale => self.scale_bar.show(ui, document, &mut view.scale_tool),
            ToolMode::Grid => {
                view.grid_tool.show_options(ui, document);
                None
            }
            ToolMode::Measure => {
                let settings = &mut view.measure_tool.settings;
                ui.strong("New measurement");
                ui.separator();
                let unit = settings.unit(document.calibration().unit());
                let tooltip = "Units of new measurements";
                if let Some(unit) = unit_picker::show(ui, "new_measurement_unit", unit, tooltip) {
                    settings.set_unit(unit);
                }
                ui.separator();
                self.new_object_style(ui, "new_measurement", settings, document);
                None
            }
            ToolMode::Area => {
                let settings = &mut view.area_tool.settings;
                ui.strong("New area");
                ui.separator();
                let unit = settings.unit(document.calibration().unit());
                let tooltip = "Units of new areas";
                if let Some(unit) = unit_picker::show_area(ui, "new_area_unit", unit, tooltip) {
                    settings.set_unit(unit);
                }
                ui.separator();
                self.new_object_style(ui, "new_area", settings, document);
                None
            }
            ToolMode::DrawLine => {
                ui.strong("New shape");
                ui.separator();
                let settings = &mut view.line_tool.settings;
                self.new_object_style(ui, "new_shape", settings, document);
                None
            }
        }
    }

    /// The style controls of a tool: the next object it adds takes on every edit at once.
    fn new_object_style(
        &mut self,
        ui: &mut egui::Ui,
        salt: &'static str,
        settings: &mut NewObjectSettings,
        document: &Document,
    ) {
        let style = settings.style(document.calibration().unit());
        let StyleEvents {
            preview, commit, ..
        } = self.style.show(
            ui,
            egui::Id::unique(salt),
            &style,
            &style,
            scale_problem(document),
            settings.fill_angles(),
        );
        let edits: [Option<StyleEdit>; 2] = [preview, commit];
        for edit in edits.into_iter().flatten() {
            settings.apply(edit);
        }
    }
}

/// Why there is no scale, if there is none.
fn scale_problem(document: &Document) -> Option<String> {
    document
        .calibration()
        .scale()
        .err()
        .map(|reason| reason.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::{Event, Modifiers, PointerButton, Pos2, RawInput, Rect};
    use gridtool::area::Area;
    use gridtool::color::Rgb;
    use gridtool::geometry::Point;
    use gridtool::imaging::DecodedImage;
    use gridtool::measurement::Measurement;
    use gridtool::style::{FillStyle, LineStyle};
    use gridtool::units::LengthUnit;

    use crate::ui::canvas::upload_texture;

    const WIDE: f32 = 1800.0;
    const NARROW: f32 = 300.0;
    /// Not the height of the real tool bar: what matters is that the row keeps to it.
    const MIN_HEIGHT: f32 = 30.0;
    /// Inside the single row of controls.
    const ROW_Y: f32 = 20.0;

    fn triangle() -> DrawnShape {
        DrawnShape::closed(
            vec![
                Point::new(1.0, 1.0),
                Point::new(20.0, 1.0),
                Point::new(10.0, 20.0),
            ],
            LengthUnit::Centimetre,
        )
        .unwrap()
    }

    struct Harness {
        ctx: egui::Context,
        controls: ToolControls,
        view: DocumentView,
        document: Document,
        mode: ToolMode,
        width: f32,
    }

    impl Harness {
        fn new(mode: ToolMode) -> Self {
            let ctx = egui::Context::default();
            egui_material_icons::initialize(&ctx);
            let image = DecodedImage {
                width: 1,
                height: 1,
                rgba: vec![255; 4],
            };
            let view = DocumentView::new(upload_texture(&ctx, "test", &image));
            Self {
                ctx,
                controls: ToolControls::default(),
                view,
                document: Document::new(100, 100),
                mode,
                width: WIDE,
            }
        }

        fn with_active_shape(mut self, style: FillStyle) -> Self {
            Action::AddShape(triangle())
                .apply(&mut self.document)
                .unwrap();
            Action::SetStyle {
                target: StyleTarget::Shape(0),
                edit: StyleEdit::FillStyle(style, 90.0),
            }
            .apply(&mut self.document)
            .unwrap();
            let id = self.document.shapes()[0].id();
            self.view.selection.select(Some(id));
            self
        }

        /// One frame of the row with `events`, returning the action it completed and where
        /// the rest of the window (the canvas) starts.
        fn frame(&mut self, events: Vec<Event>) -> (Option<Action>, f32) {
            let input = RawInput {
                screen_rect: Some(Rect::from_min_max(Pos2::ZERO, Pos2::new(self.width, 800.0))),
                events,
                ..Default::default()
            };
            let mut action = None;
            let mut canvas_top = 0.0;
            let ctx = self.ctx.clone();
            let mut output = ctx.run_ui(input, |ui| {
                action = self.controls.show(
                    ui,
                    self.mode,
                    MIN_HEIGHT,
                    Some((&mut self.view, &self.document)),
                );
                canvas_top = ui.available_rect_before_wrap().top();
            });
            // There is no renderer to upload textures to.
            output.textures_delta.clear();
            (action, canvas_top)
        }

        /// Whether a frame of the row draws a line of text containing `text`.
        fn shows_text(&mut self, text: &str) -> bool {
            let input = RawInput {
                screen_rect: Some(Rect::from_min_max(Pos2::ZERO, Pos2::new(self.width, 800.0))),
                ..Default::default()
            };
            let ctx = self.ctx.clone();
            let mut output = ctx.run_ui(input, |ui| {
                self.controls.show(
                    ui,
                    self.mode,
                    MIN_HEIGHT,
                    Some((&mut self.view, &self.document)),
                );
            });
            output.textures_delta.clear();
            output.shapes.iter().any(|clipped| match &clipped.shape {
                egui::Shape::Text(shape) => shape.galley.job.text.contains(text),
                _ => false,
            })
        }

        /// The canvas's top once the row has settled.
        fn settled_canvas_top(&mut self) -> f32 {
            self.frame(Vec::new());
            self.frame(Vec::new()).1
        }

        /// Clicks along the row, a few points apart, collecting what each click completes.
        fn actions_from_clicking_along_the_row(&mut self) -> Vec<Action> {
            let mut actions = Vec::new();
            for x in (0..self.width as u32).step_by(6) {
                let pos = Pos2::new(x as f32, ROW_Y);
                let button = |pressed| Event::PointerButton {
                    pos,
                    button: PointerButton::Primary,
                    pressed,
                    modifiers: Modifiers::NONE,
                };
                for events in [
                    vec![Event::PointerMoved(pos)],
                    vec![button(true)],
                    vec![button(false)],
                    Vec::new(),
                ] {
                    actions.extend(self.frame(events).0);
                }
            }
            actions
        }
    }

    #[test]
    fn labels_say_whether_the_shape_is_a_line_or_an_area() {
        let line = DrawnShape::open(
            vec![Point::default(), Point::new(1.0, 1.0)],
            LengthUnit::Centimetre,
        )
        .unwrap();

        assert_eq!(shape_label(0, &line), "Shape 1 (line)");
        assert_eq!(shape_label(2, &triangle()), "Shape 3 (area)");
    }

    #[test]
    fn the_canvas_stays_put_whatever_the_row_shows() {
        let mut harness = Harness::new(ToolMode::Pan);
        let top = harness.settled_canvas_top();
        assert!(top >= MIN_HEIGHT, "{top}");

        for mode in ToolMode::ALL {
            harness.mode = mode;
            assert_eq!(harness.settled_canvas_top(), top, "{mode:?}");
        }

        let mut harness = Harness::new(ToolMode::Pan).with_active_shape(FillStyle::Solid);
        assert_eq!(harness.settled_canvas_top(), top, "active shape");
    }

    #[test]
    fn only_tools_with_no_controls_show_help() {
        for mode in ToolMode::ALL {
            let mut harness = Harness::new(mode);
            harness.settled_canvas_top();

            assert_eq!(
                harness.shows_text(PAN_HELP),
                mode == ToolMode::Pan,
                "{mode:?}"
            );
            assert_eq!(
                harness.shows_text(ORIGIN_HELP),
                mode == ToolMode::Origin,
                "{mode:?}"
            );
        }
    }

    #[test]
    fn an_active_objects_controls_replace_the_help() {
        let mut harness = Harness::new(ToolMode::Pan).with_active_shape(FillStyle::Solid);
        harness.settled_canvas_top();

        assert!(!harness.shows_text(PAN_HELP));
    }

    #[test]
    fn help_wraps_in_a_narrow_window() {
        let mut harness = Harness::new(ToolMode::Origin);
        let wide = harness.settled_canvas_top();

        harness.width = 150.0;
        assert!(harness.settled_canvas_top() > wide);
    }

    #[test]
    fn the_row_is_also_there_without_a_document() {
        let ctx = egui::Context::default();
        let mut controls = ToolControls::default();
        let mut canvas_top = 0.0;
        for _ in 0..2 {
            let mut output = ctx.run_ui(RawInput::default(), |ui| {
                controls.show(ui, ToolMode::Pan, MIN_HEIGHT, None);
                canvas_top = ui.available_rect_before_wrap().top();
            });
            output.textures_delta.clear();
        }

        assert!(canvas_top >= MIN_HEIGHT, "{canvas_top}");
    }

    #[test]
    fn the_row_grows_when_its_controls_wrap_and_shrinks_back() {
        let mut harness = Harness::new(ToolMode::DrawLine);
        let wide = harness.settled_canvas_top();

        harness.width = NARROW;
        let narrow = harness.settled_canvas_top();
        assert!(narrow > wide, "{narrow} is not below {wide}");

        harness.width = WIDE;
        assert_eq!(harness.settled_canvas_top(), wide);
    }

    #[test]
    fn showing_the_row_for_any_fill_style_commits_nothing() {
        for style in FillStyle::ALL {
            let mut harness = Harness::new(ToolMode::DrawLine).with_active_shape(style);

            assert_eq!(harness.frame(Vec::new()).0, None);
        }
    }

    #[test]
    fn an_active_object_shows_its_own_controls_instead_of_the_tools() {
        let mut harness = Harness::new(ToolMode::DrawLine).with_active_shape(FillStyle::Solid);

        let actions = harness.actions_from_clicking_along_the_row();

        let edits_active_shape = |action: &Action| {
            matches!(
                action,
                Action::SetStyle {
                    target: StyleTarget::Shape(0),
                    ..
                }
            )
        };
        assert!(actions.iter().any(edits_active_shape), "{actions:?}");
        assert_eq!(
            harness.view.line_tool.settings,
            NewObjectSettings::for_shapes()
        );
    }

    #[test]
    fn the_rotate_buttons_turn_the_fill_lines_a_quarter_turn() {
        let mut harness = Harness::new(ToolMode::Pan).with_active_shape(FillStyle::Lines);

        let actions = harness.actions_from_clicking_along_the_row();

        let rotate = Action::SetStyle {
            target: StyleTarget::Shape(0),
            edit: StyleEdit::FillAngle(0.0),
        };
        let rotations = actions.iter().filter(|&action| *action == rotate);
        assert!(rotations.count() >= 2, "{actions:?}");
    }

    #[test]
    fn the_rotate_buttons_are_also_there_for_crosshatch_but_not_a_solid_fill() {
        let rotates = |style| {
            Harness::new(ToolMode::Pan)
                .with_active_shape(style)
                .actions_from_clicking_along_the_row()
                .iter()
                .any(|action| {
                    matches!(
                        action,
                        Action::SetStyle {
                            edit: StyleEdit::FillAngle(_),
                            ..
                        }
                    )
                })
        };

        assert!(rotates(FillStyle::Crosshatch));
        assert!(!rotates(FillStyle::Solid));
    }

    #[test]
    fn changing_an_active_areas_fill_style_uses_the_angle_last_set_for_it() {
        let mut harness = Harness::new(ToolMode::Area);
        Action::AddArea(Area::new(triangle().points().to_vec(), LengthUnit::Centimetre).unwrap())
            .apply(&mut harness.document)
            .unwrap();
        let id = harness.document.areas()[0].id();
        harness.view.selection.select(Some(id));
        let chose_crosshatch_at = |harness: &mut Harness, degrees| {
            let edit = StyleEdit::FillStyle(FillStyle::Crosshatch, degrees);
            harness
                .actions_from_clicking_along_the_row()
                .contains(&Action::SetStyle {
                    target: StyleTarget::Area(0),
                    edit,
                })
        };

        assert!(chose_crosshatch_at(&mut harness, 45.0));

        harness
            .view
            .area_tool
            .settings
            .remember_fill_angle(FillStyle::Crosshatch, 30.0);
        assert!(chose_crosshatch_at(&mut harness, 30.0));
    }

    #[test]
    fn setting_an_active_objects_fill_angle_is_remembered_by_its_tool() {
        let mut harness = Harness::new(ToolMode::DrawLine).with_active_shape(FillStyle::Lines);

        let actions = harness.actions_from_clicking_along_the_row();

        assert!(actions.iter().any(|action| matches!(
            action,
            Action::SetStyle {
                edit: StyleEdit::FillAngle(0.0),
                ..
            }
        )));
        let angles = harness.view.line_tool.settings.fill_angles();
        assert_eq!(angles.for_style(FillStyle::Lines), Some(0.0));
        assert_eq!(angles.for_style(FillStyle::Crosshatch), Some(45.0));
    }

    #[test]
    fn a_tools_controls_change_what_it_will_add_and_not_the_document() {
        let mut harness = Harness::new(ToolMode::Measure);

        let actions = harness.actions_from_clicking_along_the_row();

        assert_eq!(actions, []);
        let style = harness.view.measure_tool.settings.style(LengthUnit::Metre);
        assert_eq!(style.line.style, LineStyle::Dashed, "last one clicked");
    }

    #[test]
    fn an_active_measurement_can_have_its_line_edited() {
        let mut harness = Harness::new(ToolMode::Measure);
        Action::AddMeasurement(Measurement::new(
            Point::new(1.0, 1.0),
            Point::new(9.0, 9.0),
            LengthUnit::Centimetre,
        ))
        .apply(&mut harness.document)
        .unwrap();
        let id = harness.document.measurements()[0].id();
        harness.view.selection.select(Some(id));

        let actions = harness.actions_from_clicking_along_the_row();

        assert!(
            actions.contains(&Action::SetStyle {
                target: StyleTarget::Measurement(0),
                edit: StyleEdit::LineStyle(LineStyle::Dashed),
            }),
            "{actions:?}"
        );
        assert_eq!(
            harness
                .view
                .measure_tool
                .settings
                .style(LengthUnit::Metre)
                .line
                .color,
            Rgb::BLUE
        );
    }
}
