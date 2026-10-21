//! Draws the document and turns mouse input into view changes or document actions,
//! according to the active tool.

use eframe::egui::{
    self, Align2, Color32, CursorIcon, FontId, PointerButton, Pos2, Rect, Response, Sense, Shape,
    Stroke, TextureHandle, Vec2,
};
use gridtool::action::Action;
use gridtool::calibration::{CalibrationMethod, Line};
use gridtool::document::Document;
use gridtool::geometry::Point;
use gridtool::imaging::DecodedImage;
use gridtool::measurement::Endpoint;

use super::document_view::DocumentView;
use super::format::format_length;
use super::hit_test::nearest_within;
use super::measure_tool::{Handle, MeasureTool};
use super::rotate_handle;
use super::scale_tool::ScaleTool;
use super::tool_mode::ToolMode;
use super::viewport::{DocumentTransform, Viewport};

const OUTLINE_STROKE: Stroke = Stroke {
    width: 2.0,
    color: Color32::RED,
};
const ARROW_STROKE: Stroke = Stroke {
    width: 2.0,
    color: Color32::from_rgb(0, 170, 255),
};
const SCALE_STROKE: Stroke = Stroke {
    width: 1.5,
    color: Color32::RED,
};
const MEASUREMENT_STROKE: Stroke = Stroke {
    width: 2.0,
    color: Color32::BLUE,
};
const SCALE_MARKER_RADIUS: f32 = 6.0;
const MEASUREMENT_HANDLE_RADIUS: f32 = 5.0;
const CROSSHAIR_HALF_LENGTH: f32 = 11.0;
/// How close (in screen points) the pointer must be to a point to grab it.
const POINT_GRAB_RADIUS: f32 = 10.0;
const LABEL_OFFSET: f32 = 12.0;
const LABEL_FONT_SIZE: f32 = 14.0;
const DASH_LENGTH: f32 = 6.0;
const DASH_GAP: f32 = 4.0;
/// How strongly one point of mouse-wheel scrolling changes the zoom.
const SCROLL_ZOOM_SPEED: f32 = 0.002;

/// Uploads a decoded image to the GPU so the canvas can draw it.
pub fn upload_texture(ctx: &egui::Context, name: &str, image: &DecodedImage) -> TextureHandle {
    let color_image = egui::ColorImage::from_rgba_unmultiplied(image.size(), &image.rgba);
    ctx.load_texture(name, color_image, egui::TextureOptions::default())
}

/// Shows the canvas, returning the document action the user completed this frame, if any.
pub fn show(
    ui: &mut egui::Ui,
    mode: ToolMode,
    view: &mut DocumentView,
    document: &Document,
) -> Option<Action> {
    let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());
    let view_center = response.rect.center();
    let document_size = Vec2::new(document.width() as f32, document.height() as f32);

    handle_zoom(ui, &response, view_center, &mut view.viewport);
    if mode != ToolMode::Scale {
        view.scale_tool.cancel_drag();
    }
    if mode != ToolMode::Measure {
        view.measure_tool.cancel();
    }
    let transform = view.viewport.transform(view_center, document_size);
    let action = match mode {
        ToolMode::Pan => {
            handle_pan(ui, &response, &mut view.viewport);
            None
        }
        ToolMode::Rotate => {
            handle_rotate(ui, &response, transform.center(), view);
            None
        }
        ToolMode::Scale => handle_scale(ui, &response, transform, document, &mut view.scale_tool),
        ToolMode::Measure => {
            handle_measure(ui, &response, transform, document, &mut view.measure_tool)
        }
    };
    // In every mode the secondary button pans. Done after the tool so its cursor wins.
    drag_pan(ui, &response, &mut view.viewport, PointerButton::Secondary);

    let transform = view.viewport.transform(view_center, document_size);
    paint_document(&painter, transform, &view.texture);
    paint_measurements(&painter, transform, document, &view.measure_tool);
    match mode {
        ToolMode::Pan => {}
        ToolMode::Rotate => {
            paint_rotate_handle(
                &painter,
                transform.center(),
                view.viewport.rotation_radians(),
            );
        }
        ToolMode::Scale => {
            let points = view.scale_tool.points(document.calibration());
            paint_scale_markers(&painter, transform, &points, view.scale_tool.method());
        }
        ToolMode::Measure => {
            if let Some(start) = view.measure_tool.pending_start() {
                paint_pending_measurement(&painter, transform, start, response.hover_pos());
            }
        }
    }
    action
}

fn handle_zoom(ui: &egui::Ui, response: &Response, view_center: Pos2, viewport: &mut Viewport) {
    let Some(pointer) = response.hover_pos() else {
        return;
    };
    let (scroll, pinch) = ui.input(|input| (input.smooth_scroll_delta().y, input.zoom_delta()));
    let factor = pinch * (scroll * SCROLL_ZOOM_SPEED).exp();
    if factor != 1.0 {
        viewport.zoom_about(view_center, pointer, factor);
    }
}

fn handle_pan(ui: &egui::Ui, response: &Response, viewport: &mut Viewport) {
    if !drag_pan(ui, response, viewport, PointerButton::Primary) && response.hovered() {
        ui.set_cursor_icon(CursorIcon::Grab);
    }
}

/// Pans the view while `button` is dragging on the canvas. Returns whether it did.
fn drag_pan(
    ui: &egui::Ui,
    response: &Response,
    viewport: &mut Viewport,
    button: PointerButton,
) -> bool {
    if !response.dragged_by(button) {
        return false;
    }
    viewport.pan_by(response.drag_delta());
    ui.set_cursor_icon(CursorIcon::Grabbing);
    true
}

fn handle_rotate(ui: &egui::Ui, response: &Response, center: Pos2, view: &mut DocumentView) {
    let rotation = view.viewport.rotation_radians();

    if response.drag_started_by(PointerButton::Primary) {
        view.is_dragging_rotate_handle = press_origin(ui)
            .is_some_and(|origin| rotate_handle::is_over_tip(center, rotation, origin));
    }

    if view.is_dragging_rotate_handle {
        if let Some(radians) = response
            .interact_pointer_pos()
            .and_then(|pointer| rotate_handle::rotation_towards(center, pointer))
        {
            view.viewport.set_rotation_radians(radians);
        }
        ui.set_cursor_icon(CursorIcon::Grabbing);
    } else if response
        .hover_pos()
        .is_some_and(|pointer| rotate_handle::is_over_tip(center, rotation, pointer))
    {
        ui.set_cursor_icon(CursorIcon::Grab);
    }

    if response.drag_stopped_by(PointerButton::Primary) {
        view.is_dragging_rotate_handle = false;
    }
}

/// Pointer input relevant to tools that place points by clicking and move them by
/// dragging. Marker indices refer to the slice of screen positions it was read with.
struct PointGesture {
    /// A drag began on this marker this frame.
    grabbed: Option<usize>,
    /// The pointer position while a drag is in progress.
    drag_position: Option<Pos2>,
    /// A drag ended this frame.
    released: bool,
    /// A click (press and release without dragging) happened here this frame.
    clicked: Option<Pos2>,
    /// The marker under the hovering pointer.
    hovered: Option<usize>,
}

impl PointGesture {
    fn read(ui: &egui::Ui, response: &Response, markers: &[Pos2]) -> Self {
        let marker_at = |position| nearest_within(markers, position, POINT_GRAB_RADIUS);
        Self {
            grabbed: response
                .drag_started_by(PointerButton::Primary)
                .then(|| press_origin(ui).and_then(marker_at))
                .flatten(),
            drag_position: response
                .dragged_by(PointerButton::Primary)
                .then(|| response.interact_pointer_pos())
                .flatten(),
            released: response.drag_stopped_by(PointerButton::Primary),
            clicked: response
                .clicked()
                .then(|| response.interact_pointer_pos())
                .flatten(),
            hovered: response.hover_pos().and_then(marker_at),
        }
    }

    /// Grabbing hand while dragging, open hand over a grabbable marker, otherwise a
    /// crosshair if clicking would place a point.
    fn update_cursor(
        &self,
        ui: &egui::Ui,
        response: &Response,
        is_dragging: bool,
        can_grab: bool,
        can_place: bool,
    ) {
        if is_dragging {
            ui.set_cursor_icon(CursorIcon::Grabbing);
        } else if can_grab && self.hovered.is_some() {
            ui.set_cursor_icon(CursorIcon::Grab);
        } else if can_place && response.hovered() {
            ui.set_cursor_icon(CursorIcon::Crosshair);
        }
    }
}

fn handle_scale(
    ui: &egui::Ui,
    response: &Response,
    transform: DocumentTransform,
    document: &Document,
    tool: &mut ScaleTool,
) -> Option<Action> {
    let calibration = document.calibration();
    let is_placing = tool.is_placing(calibration);
    let markers: Vec<Pos2> = tool
        .points(calibration)
        .into_iter()
        .map(|point| transform.image_to_screen(point))
        .collect();
    let gesture = PointGesture::read(ui, response, &markers);

    if !is_placing && let Some(index) = gesture.grabbed {
        tool.begin_drag(calibration, index);
    }
    if tool.is_dragging()
        && let Some(position) = gesture.drag_position
    {
        tool.drag_to(document.clamp(transform.screen_to_image(position)));
    }
    gesture.update_cursor(ui, response, tool.is_dragging(), !is_placing, is_placing);

    if gesture.released {
        return tool.end_drag();
    }
    if is_placing && let Some(position) = gesture.clicked {
        let point = transform.screen_to_image(position);
        if document.contains(point) {
            return tool.place(calibration, point);
        }
    }
    None
}

fn handle_measure(
    ui: &egui::Ui,
    response: &Response,
    transform: DocumentTransform,
    document: &Document,
    tool: &mut MeasureTool,
) -> Option<Action> {
    let handles = tool.handles(document);
    let markers: Vec<Pos2> = handles
        .iter()
        .map(|&(_, point)| transform.image_to_screen(point))
        .collect();
    let gesture = PointGesture::read(ui, response, &markers);

    if let Some(index) = gesture.grabbed {
        tool.begin_drag(document, handles[index].0);
    }
    if tool.is_dragging()
        && let Some(position) = gesture.drag_position
    {
        tool.drag_to(document.clamp(transform.screen_to_image(position)));
    }
    gesture.update_cursor(ui, response, tool.is_dragging(), true, true);

    if gesture.released {
        return tool.end_drag();
    }
    if let Some(position) = gesture.clicked {
        let point = transform.screen_to_image(position);
        if document.contains(point) {
            return tool.place(point, document.calibration().unit());
        }
    }
    None
}

fn press_origin(ui: &egui::Ui) -> Option<Pos2> {
    ui.input(|input| input.pointer.press_origin())
}

fn paint_document(painter: &egui::Painter, transform: DocumentTransform, texture: &TextureHandle) {
    painter.add(textured_quad(transform.corners(), texture));

    // The outline's centre line sits half a stroke outside the document, so the whole
    // stroke lies around the document rather than on top of it.
    let outline = transform.corners_outset(OUTLINE_STROKE.width / 2.0);
    painter.add(Shape::closed_line(outline.to_vec(), OUTLINE_STROKE));
}

fn paint_rotate_handle(painter: &egui::Painter, center: Pos2, rotation_radians: f32) {
    let tip = rotate_handle::tip(center, rotation_radians);
    painter.arrow(center, tip - center, ARROW_STROKE);
    painter.circle_stroke(tip, rotate_handle::GRAB_RADIUS, ARROW_STROKE);
}

/// Draws the calibration lines (where both ends are placed) and a marker per point.
fn paint_scale_markers(
    painter: &egui::Painter,
    transform: DocumentTransform,
    points: &[Point],
    method: CalibrationMethod,
) {
    let positions: Vec<Pos2> = points
        .iter()
        .map(|&point| transform.image_to_screen(point))
        .collect();

    for line in Line::ALL {
        let (start, end) = method.line_endpoints(line);
        if let (Some(&start), Some(&end)) = (positions.get(start), positions.get(end)) {
            paint_labelled_line(painter, start, end, SCALE_STROKE, line.to_string());
        }
    }

    let [x_axis, y_axis] = transform.image_axes();
    for (index, &position) in positions.iter().enumerate() {
        painter.circle_stroke(position, SCALE_MARKER_RADIUS, SCALE_STROKE);
        for axis in [x_axis, y_axis] {
            let arm = axis * CROSSHAIR_HALF_LENGTH;
            painter.line_segment([position - arm, position + arm], SCALE_STROKE);
        }
        let label_position =
            position + (x_axis + y_axis).normalized() * (LABEL_OFFSET + SCALE_MARKER_RADIUS);
        paint_label(
            painter,
            label_position,
            (index + 1).to_string(),
            SCALE_STROKE.color,
        );
    }
}

/// Draws every measurement as a labelled line with a handle at each end.
fn paint_measurements(
    painter: &egui::Painter,
    transform: DocumentTransform,
    document: &Document,
    tool: &MeasureTool,
) {
    for (index, measurement) in document.measurements().iter().enumerate() {
        let [start, end] = Endpoint::ALL.map(|endpoint| {
            let point = tool.displayed_point(Handle { index, endpoint }, measurement);
            transform.image_to_screen(point)
        });
        let label = match measurement.distance(document.calibration()) {
            Ok(distance) => format!(
                "{}: {} {}",
                index + 1,
                format_length(distance),
                measurement.unit()
            ),
            Err(_) => (index + 1).to_string(),
        };
        paint_labelled_line(painter, start, end, MEASUREMENT_STROKE, label);
        for position in [start, end] {
            paint_measurement_handle(painter, position);
        }
    }
}

/// Draws the placed start of a new measurement and a dashed line to the pointer.
fn paint_pending_measurement(
    painter: &egui::Painter,
    transform: DocumentTransform,
    start: Point,
    pointer: Option<Pos2>,
) {
    let start = transform.image_to_screen(start);
    if let Some(pointer) = pointer {
        painter.extend(Shape::dashed_line(
            &[start, pointer],
            MEASUREMENT_STROKE,
            DASH_LENGTH,
            DASH_GAP,
        ));
    }
    paint_measurement_handle(painter, start);
}

fn paint_measurement_handle(painter: &egui::Painter, position: Pos2) {
    painter.circle_filled(
        position,
        MEASUREMENT_HANDLE_RADIUS,
        MEASUREMENT_STROKE.color,
    );
}

/// Draws a line with `label` beside its midpoint.
fn paint_labelled_line(
    painter: &egui::Painter,
    start: Pos2,
    end: Pos2,
    stroke: Stroke,
    label: String,
) {
    painter.line_segment([start, end], stroke);
    let normal = (end - start).normalized().rot90();
    paint_label(
        painter,
        start.lerp(end, 0.5) + normal * LABEL_OFFSET,
        label,
        stroke.color,
    );
}

fn paint_label(painter: &egui::Painter, position: Pos2, text: String, color: Color32) {
    painter.text(
        position,
        Align2::CENTER_CENTER,
        text,
        FontId::proportional(LABEL_FONT_SIZE),
        color,
    );
}

/// A mesh drawing the whole texture onto the quad with the given corners
/// (top-left, top-right, bottom-right, bottom-left).
fn textured_quad(corners: [Pos2; 4], texture: &TextureHandle) -> egui::Mesh {
    let uv = Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0));
    let uvs = [
        uv.left_top(),
        uv.right_top(),
        uv.right_bottom(),
        uv.left_bottom(),
    ];

    let mut mesh = egui::Mesh::with_texture(texture.id());
    for (pos, uv) in corners.into_iter().zip(uvs) {
        mesh.vertices.push(egui::epaint::Vertex {
            pos,
            uv,
            color: Color32::WHITE,
        });
    }
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(0, 2, 3);
    mesh
}

/// Headless tests that feed raw pointer events through a real egui context.
#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::{Event, Modifiers, RawInput};
    use gridtool::measurement::Measurement;
    use gridtool::units::LengthUnit;

    const SCREEN: Rect = Rect::from_min_max(Pos2::ZERO, Pos2::new(800.0, 600.0));
    /// Where image point (50, 50) of the 200×100 test document is drawn at default view.
    const HANDLE_ON_SCREEN: Pos2 = Pos2::new(350.0, 300.0);
    const EMPTY_SPOT: Pos2 = Pos2::new(600.0, 500.0);
    const DRAG_BY: Vec2 = Vec2::new(40.0, 25.0);

    struct Harness {
        ctx: egui::Context,
        mode: ToolMode,
        view: DocumentView,
        document: Document,
        actions: Vec<Action>,
    }

    impl Harness {
        fn new(mode: ToolMode) -> Self {
            let ctx = egui::Context::default();
            let image = DecodedImage {
                width: 1,
                height: 1,
                rgba: vec![255; 4],
            };
            let texture = upload_texture(&ctx, "test", &image);
            let mut document = Document::new(200, 100);
            Action::AddMeasurement(Measurement::new(
                Point::new(50.0, 50.0),
                Point::new(150.0, 50.0),
                LengthUnit::Centimetre,
            ))
            .apply(&mut document)
            .unwrap();
            let mut harness = Self {
                ctx,
                mode,
                view: DocumentView::new(texture),
                document,
                actions: Vec::new(),
            };
            harness.frame(Vec::new()); // Lay out once so the canvas rect is known.
            harness
        }

        fn frame(&mut self, events: Vec<Event>) {
            let input = RawInput {
                screen_rect: Some(SCREEN),
                events,
                ..Default::default()
            };
            let ctx = self.ctx.clone();
            let mut output = ctx.run_ui(input, |ui| {
                self.actions
                    .extend(show(ui, self.mode, &mut self.view, &self.document));
            });
            // There is no renderer to upload textures to.
            output.textures_delta.clear();
        }

        fn drag(&mut self, button: PointerButton, from: Pos2, by: Vec2) {
            let press = |pos, pressed| Event::PointerButton {
                pos,
                button,
                pressed,
                modifiers: Modifiers::NONE,
            };
            let to = from + by;
            self.frame(vec![Event::PointerMoved(from)]);
            self.frame(vec![press(from, true)]);
            self.frame(vec![Event::PointerMoved(from + by * 0.5)]);
            self.frame(vec![Event::PointerMoved(to)]);
            self.frame(vec![press(to, false)]);
            self.frame(Vec::new());
        }

        /// How far the view has been panned from its default.
        fn pan(&self) -> Vec2 {
            self.view
                .viewport
                .transform(Pos2::ZERO, Vec2::ZERO)
                .center()
                .to_vec2()
        }
    }

    #[test]
    fn handle_is_where_the_tests_expect() {
        let harness = Harness::new(ToolMode::Measure);
        let transform = harness
            .view
            .viewport
            .transform(SCREEN.center(), Vec2::new(200.0, 100.0));

        assert_eq!(
            transform.image_to_screen(Point::new(50.0, 50.0)),
            HANDLE_ON_SCREEN
        );
    }

    #[test]
    fn right_drag_pans_in_every_mode() {
        for mode in ToolMode::ALL {
            let mut harness = Harness::new(mode);

            harness.drag(PointerButton::Secondary, EMPTY_SPOT, DRAG_BY);

            assert_eq!(harness.pan(), DRAG_BY, "{mode:?} should pan");
            assert!(harness.actions.is_empty(), "{mode:?} should not edit");
        }
    }

    #[test]
    fn right_drag_on_a_handle_pans_instead_of_moving_it() {
        let mut harness = Harness::new(ToolMode::Measure);

        harness.drag(PointerButton::Secondary, HANDLE_ON_SCREEN, DRAG_BY);

        assert_eq!(harness.pan(), DRAG_BY);
        assert!(harness.actions.is_empty());
    }

    #[test]
    fn left_drag_on_a_handle_moves_it_without_panning() {
        let mut harness = Harness::new(ToolMode::Measure);

        harness.drag(PointerButton::Primary, HANDLE_ON_SCREEN, DRAG_BY);

        assert_eq!(harness.pan(), Vec2::ZERO);
        assert_eq!(
            harness.actions,
            [Action::MoveMeasurementPoint {
                index: 0,
                endpoint: Endpoint::Start,
                to: Point::new(90.0, 75.0),
            }]
        );
    }

    #[test]
    fn left_drag_pans_only_in_pan_mode() {
        for mode in ToolMode::ALL {
            let mut harness = Harness::new(mode);

            harness.drag(PointerButton::Primary, EMPTY_SPOT, DRAG_BY);

            let expected = if mode == ToolMode::Pan {
                DRAG_BY
            } else {
                Vec2::ZERO
            };
            assert_eq!(harness.pan(), expected, "{mode:?}");
        }
    }

    #[test]
    fn mouse_wheel_zooms_in_every_mode() {
        for mode in ToolMode::ALL {
            let mut harness = Harness::new(mode);
            let size = Vec2::new(200.0, 100.0);
            let corner =
                |h: &Harness| h.view.viewport.transform(SCREEN.center(), size).corners()[0];
            let before = corner(&harness);

            harness.frame(vec![Event::PointerMoved(SCREEN.center())]);
            harness.frame(vec![Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                delta: Vec2::new(0.0, 100.0),
                phase: egui::TouchPhase::Move,
                modifiers: Modifiers::NONE,
                source: egui::MouseWheelSource::Wheel,
            }]);
            for _ in 0..30 {
                harness.frame(Vec::new()); // Let smooth scrolling settle.
            }

            assert!(
                corner(&harness).x < before.x,
                "{mode:?} should zoom in on scroll up"
            );
        }
    }
}
