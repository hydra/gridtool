//! Draws the document and turns mouse input into view changes or document actions,
//! according to the active tool.

use eframe::egui::{
    self, Align2, Color32, CursorIcon, FontId, PointerButton, Pos2, Rect, Response, Sense, Shape,
    Stroke, TextureHandle, Vec2,
};
use gridtool::action::Action;
use gridtool::calibration::{CalibrationMethod, Line};
use gridtool::document::Document;
use gridtool::geometry::{Bounds, BoundsHandle, Point};
use gridtool::grid::GridSource;
use gridtool::imaging::DecodedImage;
use gridtool::measurement::{self, Endpoint};
use gridtool::polygon;

use super::area_tool::{AreaTool, PointHit};
use super::color::{to_color32, with_opacity};
use super::document_view::DocumentView;
use super::format::format_length;
use super::grid_display::paint_grids;
use super::grid_tool::add_grid;
use super::hit_test::nearest_within;
use super::line_hit::segment_at;
use super::measure_tool::MeasureTool;
use super::object_drag::Grip;
use super::rotate_handle;
use super::scale_tool::ScaleTool;
use super::selection::Selection;
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
/// Measurement lines and area outlines, drawn in the object's colour.
const OBJECT_LINE_WIDTH: f32 = 2.0;
/// How opaque an area's fill is, over its own colour.
const AREA_FILL_OPACITY: f32 = 0.25;
const SELECTION_COLOR: Color32 = Color32::from_rgb(128, 0, 128);
const SELECTION_STROKE: Stroke = Stroke {
    width: 1.5,
    color: SELECTION_COLOR,
};
/// Gap (screen points) between an object and its selection box, so that even a
/// perfectly level or upright line gets a box with distinct handles.
const SELECTION_PADDING: f32 = 8.0;
const RESIZE_HANDLE_SIZE: f32 = 10.0;
/// How close (in screen points) the pointer must be to a resize handle to grab it.
const RESIZE_HANDLE_GRAB_RADIUS: f32 = 8.0;
const HIGHLIGHT_STROKE: Stroke = Stroke {
    width: 5.0,
    color: Color32::from_rgba_premultiplied(255, 220, 0, 200),
};
/// How far (screen points) either side of a line still counts as hovering over it.
const LINE_HOVER_TOLERANCE: f32 = 5.0;
/// Pointer positions this close (screen points) to a line's ends are over its handles,
/// which are not part of the line's hover area.
const LINE_END_EXCLUSION: f32 = 8.0;
const SCALE_MARKER_RADIUS: f32 = 6.0;
/// Measurement ends and area corners.
const POINT_HANDLE_RADIUS: f32 = 5.0;
/// The ring around the first point of an outline being placed, once it can be closed.
const CLOSE_TARGET_RADIUS: f32 = 9.0;
const CROSSHAIR_HALF_LENGTH: f32 = 11.0;
const ORIGIN_STROKE: Stroke = Stroke {
    width: 2.0,
    color: Color32::BLUE,
};
/// Screen points, the same at every zoom level.
const ORIGIN_CIRCLE_DIAMETER: f32 = 15.0;
/// Full length of each line of the origin's cross, in screen points.
const ORIGIN_CROSS_LENGTH: f32 = 25.0;
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
    view.canvas_rect = Some(response.rect);
    let view_center = response.rect.center();
    let document_size = Vec2::new(document.width() as f32, document.height() as f32);

    handle_zoom(ui, &response, view_center, &mut view.viewport);
    let transform = view.viewport.transform(view_center, document_size);
    // The active object's points and resize handles take priority over every tool.
    let selection_input = handle_selection(ui, &response, transform, document, &mut view.selection);
    let action = match selection_input {
        SelectionInput::Dragging(action) => action,
        SelectionInput::NotDragging => match mode {
            ToolMode::Pan => {
                handle_pan(ui, &response, &mut view.viewport);
                None
            }
            ToolMode::Rotate => {
                handle_rotate(ui, &response, transform, document, view);
                None
            }
            ToolMode::Scale => {
                handle_scale(ui, &response, transform, document, &mut view.scale_tool)
            }
            ToolMode::Measure => {
                handle_measure(ui, &response, transform, document, &mut view.measure_tool)
            }
            ToolMode::Area => handle_area(ui, &response, transform, document, &mut view.area_tool),
            ToolMode::Grid => handle_grid(ui, &response, transform, document, view),
            ToolMode::Origin => handle_origin(ui, &response, transform, document),
        },
    };
    if mode != ToolMode::Grid {
        view.grid_tool.set_hovered(None);
    }
    if !view.selection.is_dragging()
        && response
            .hover_pos()
            .and_then(|pointer| grip_at(transform, document, &view.selection, pointer))
            .is_some()
    {
        ui.set_cursor_icon(CursorIcon::Grab);
    }
    // In every mode the secondary button pans. Done after the tool so its cursor wins.
    drag_pan(ui, &response, &mut view.viewport, PointerButton::Secondary);

    // Drawing order: image, grids, areas, then the lines grids align to, then tool
    // overlays.
    let transform = view.viewport.transform(view_center, document_size);
    paint_document(&painter, transform, &view.texture);
    paint_grids(&painter, transform, document, &view.selection);
    paint_areas(&painter, transform, document, &view.selection);
    paint_measurements(&painter, transform, document, &view.selection);
    match mode {
        ToolMode::Scale => {
            let points = view.scale_tool.points(document.calibration());
            paint_scale_markers(&painter, transform, &points, view.scale_tool.method());
        }
        ToolMode::Grid => {
            if let Some(points) = document.calibration().points() {
                paint_scale_markers(&painter, transform, points.points(), points.method());
            }
        }
        ToolMode::Pan
        | ToolMode::Rotate
        | ToolMode::Measure
        | ToolMode::Area
        | ToolMode::Origin => {}
    }
    paint_active_grid_source(&painter, transform, document, &view.selection);
    match mode {
        ToolMode::Rotate => {
            paint_rotate_handle(
                &painter,
                transform.image_to_screen(document.origin()),
                view.viewport.rotation_radians(),
            );
        }
        ToolMode::Origin => paint_origin(&painter, transform, document.origin()),
        ToolMode::Measure => {
            if let Some(start) = view.measure_tool.pending_start() {
                paint_pending_measurement(&painter, transform, start, response.hover_pos());
            }
        }
        ToolMode::Area => {
            paint_pending_area(&painter, transform, &view.area_tool, response.hover_pos());
        }
        ToolMode::Grid => {
            if let Some(source) = view.grid_tool.hovered() {
                paint_highlighted_source(&painter, transform, document, &view.selection, source);
            }
        }
        ToolMode::Pan | ToolMode::Scale => {}
    }
    if let Some(bounds) = view.selection.active_bounds(document) {
        paint_selection(&painter, transform, bounds);
    }
    action
}

/// Highlights the alignable line under the pointer; a click on it adds a grid.
fn handle_grid(
    ui: &egui::Ui,
    response: &Response,
    transform: DocumentTransform,
    document: &Document,
    view: &mut DocumentView,
) -> Option<Action> {
    let candidates = view.grid_tool.candidate_lines(document, &view.selection);
    let segments: Vec<(Pos2, Pos2)> = candidates
        .iter()
        .map(|&(_, start, end)| {
            (
                transform.image_to_screen(start),
                transform.image_to_screen(end),
            )
        })
        .collect();
    let source_at = |position: Pos2| {
        segment_at(
            &segments,
            position,
            LINE_HOVER_TOLERANCE,
            LINE_END_EXCLUSION,
        )
        .map(|index| candidates[index].0)
    };

    let hovered = response.hover_pos().and_then(source_at);
    view.grid_tool.set_hovered(hovered);
    if hovered.is_some() {
        ui.set_cursor_icon(CursorIcon::PointingHand);
    }

    let clicked = response
        .clicked()
        .then(|| response.interact_pointer_pos())
        .flatten()?;
    add_grid(document, source_at(clicked)?)
}

/// Whether the active object's grips are handling the primary button.
enum SelectionInput {
    /// A drag of a point or resize handle is in progress (or just ended, with its
    /// action); tools must ignore input.
    Dragging(Option<Action>),
    NotDragging,
}

/// Dragging the active object's points or resize handles, and clearing the selection
/// when the user clicks outside its bounding box. Runs in every tool mode.
fn handle_selection(
    ui: &egui::Ui,
    response: &Response,
    transform: DocumentTransform,
    document: &Document,
    selection: &mut Selection,
) -> SelectionInput {
    if selection.active().is_none() {
        return SelectionInput::NotDragging;
    }
    let Some(bounds) = selection.active_bounds(document) else {
        // Objects without a selection box (grids cover the whole document): any click
        // on the canvas counts as outside.
        if response.clicked() {
            selection.select(None);
        }
        return SelectionInput::NotDragging;
    };

    if response.drag_started_by(PointerButton::Primary)
        && let Some(origin) = press_origin(ui)
        && let Some(grip) = grip_at(transform, document, selection, origin)
    {
        selection.begin_drag(document, grip, transform.screen_to_image(origin));
    }

    if selection.is_dragging() {
        if response.dragged_by(PointerButton::Primary)
            && let Some(pointer) = response.interact_pointer_pos()
        {
            selection.drag_to(document, transform.screen_to_image(pointer));
        }
        ui.set_cursor_icon(CursorIcon::Grabbing);
        let action = response
            .drag_stopped_by(PointerButton::Primary)
            .then(|| selection.end_drag(document))
            .flatten();
        return SelectionInput::Dragging(action);
    }

    if response.clicked()
        && let Some(position) = response.interact_pointer_pos()
    {
        let margin = transform.screen_to_image_length(SELECTION_PADDING + RESIZE_HANDLE_SIZE / 2.0);
        if !bounds
            .expanded(margin)
            .contains(transform.screen_to_image(position))
        {
            selection.select(None);
        }
    }
    SelectionInput::NotDragging
}

/// The grip of the active object under screen `position`: the nearer of its nearest
/// point (within [`POINT_GRAB_RADIUS`]) and its nearest resize handle (within
/// [`RESIZE_HANDLE_GRAB_RADIUS`]).
fn grip_at(
    transform: DocumentTransform,
    document: &Document,
    selection: &Selection,
    position: Pos2,
) -> Option<Grip> {
    let bounds = selection.active_bounds(document)?;
    let points: Vec<Pos2> = selection
        .active_points(document)
        .into_iter()
        .map(|point| transform.image_to_screen(point))
        .collect();
    let point = nearest_within(&points, position, POINT_GRAB_RADIUS)
        .map(|index| (Grip::Point(index), points[index].distance(position)));
    let handle = resize_handle_at(transform, bounds, position)
        .map(|(handle, at)| (Grip::Bounds(handle), at.distance(position)));
    [point, handle]
        .into_iter()
        .flatten()
        .min_by(|(_, a), (_, b)| a.total_cmp(b))
        .map(|(grip, _)| grip)
}

/// The resize handle of `bounds` under screen `position`, if any, with its position.
fn resize_handle_at(
    transform: DocumentTransform,
    bounds: Bounds,
    position: Pos2,
) -> Option<(BoundsHandle, Pos2)> {
    let handles = resize_handle_positions(transform.bounds_corners(bounds, SELECTION_PADDING));
    let positions = handles.map(|(_, position)| position);
    nearest_within(&positions, position, RESIZE_HANDLE_GRAB_RADIUS).map(|index| handles[index])
}

/// Screen positions of the eight resize handles of a selection box with the given
/// corners (top-left, top-right, bottom-right, bottom-left).
fn resize_handle_positions(corners: [Pos2; 4]) -> [(BoundsHandle, Pos2); 8] {
    let [top_left, top_right, bottom_right, bottom_left] = corners;
    let mid = |a: Pos2, b: Pos2| a.lerp(b, 0.5);
    [
        (BoundsHandle::TopLeft, top_left),
        (BoundsHandle::Top, mid(top_left, top_right)),
        (BoundsHandle::TopRight, top_right),
        (BoundsHandle::Right, mid(top_right, bottom_right)),
        (BoundsHandle::BottomRight, bottom_right),
        (BoundsHandle::Bottom, mid(bottom_right, bottom_left)),
        (BoundsHandle::BottomLeft, bottom_left),
        (BoundsHandle::Left, mid(bottom_left, top_left)),
    ]
}

/// Dashed purple box around the active object, with a square handle at each corner and
/// edge midpoint.
fn paint_selection(painter: &egui::Painter, transform: DocumentTransform, bounds: Bounds) {
    let corners = transform.bounds_corners(bounds, SELECTION_PADDING);
    let mut outline = corners.to_vec();
    outline.push(corners[0]);
    painter.extend(Shape::dashed_line(
        &outline,
        SELECTION_STROKE,
        DASH_LENGTH,
        DASH_GAP,
    ));
    for (_, position) in resize_handle_positions(corners) {
        let handle = Rect::from_center_size(position, Vec2::splat(RESIZE_HANDLE_SIZE));
        painter.rect_filled(handle, 0.0, SELECTION_COLOR);
    }
}

/// Zooms with the wheel or pinch while the pointer is over the canvas, in every mode,
/// keeping the document point under the pointer fixed on screen.
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

fn handle_rotate(
    ui: &egui::Ui,
    response: &Response,
    transform: DocumentTransform,
    document: &Document,
    view: &mut DocumentView,
) {
    // The view rotates about the document origin, which the arrow starts from.
    let center = transform.image_to_screen(document.origin());
    let document_size = Vec2::new(document.width() as f32, document.height() as f32);
    let rotation = view.viewport.rotation_radians();

    if response.drag_started_by(PointerButton::Primary) {
        view.is_dragging_rotate_handle = press_origin(ui)
            .is_some_and(|press| rotate_handle::is_over_tip(center, rotation, press));
    }

    if view.is_dragging_rotate_handle {
        if let Some(radians) = response
            .interact_pointer_pos()
            .and_then(|pointer| rotate_handle::rotation_towards(center, pointer))
        {
            view.viewport
                .rotate_about(radians.to_degrees(), document.origin(), document_size);
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
    let gesture = PointGesture::read(ui, response, &[]);
    gesture.update_cursor(ui, response, false, false, true);

    if let Some(position) = gesture.clicked {
        let point = transform.screen_to_image(position);
        if document.contains(point) {
            return tool.place(point, document.calibration().unit());
        }
    }
    None
}

/// Each click places the next corner of an area. Clicking the first point closes the
/// outline (once it has three points); clicking the last point removes it.
fn handle_area(
    ui: &egui::Ui,
    response: &Response,
    transform: DocumentTransform,
    document: &Document,
    tool: &mut AreaTool,
) -> Option<Action> {
    let on_screen = to_screen(transform, tool.points());
    let hit_at = |position| AreaTool::hit_at(&on_screen, position, POINT_GRAB_RADIUS);
    match response.hover_pos().and_then(hit_at) {
        Some(PointHit::Last) => ui.set_cursor_icon(CursorIcon::PointingHand),
        Some(PointHit::First) if tool.can_close() => ui.set_cursor_icon(CursorIcon::PointingHand),
        _ if response.hovered() => ui.set_cursor_icon(CursorIcon::Crosshair),
        _ => {}
    }

    let clicked = response
        .clicked()
        .then(|| response.interact_pointer_pos())
        .flatten()?;
    let hit = hit_at(clicked);
    let point = transform.screen_to_image(clicked);
    if hit.is_none() && !document.contains(point) {
        return None;
    }
    tool.click(point, hit, document.calibration().unit())
}

/// A click anywhere on the image moves the document origin there.
fn handle_origin(
    ui: &egui::Ui,
    response: &Response,
    transform: DocumentTransform,
    document: &Document,
) -> Option<Action> {
    let gesture = PointGesture::read(ui, response, &[]);
    gesture.update_cursor(ui, response, false, false, true);

    let point = transform.screen_to_image(gesture.clicked?);
    (document.contains(point) && point != document.origin()).then_some(Action::SetOrigin(point))
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

    for (index, &position) in positions.iter().enumerate() {
        paint_scale_point(painter, transform, position, index + 1);
    }
}

/// A calibration point marker: circle, crosshair aligned with the image axes, and number.
fn paint_scale_point(
    painter: &egui::Painter,
    transform: DocumentTransform,
    position: Pos2,
    number: usize,
) {
    let [x_axis, y_axis] = transform.image_axes();
    paint_crosshair(
        painter,
        transform,
        position,
        SCALE_MARKER_RADIUS,
        CROSSHAIR_HALF_LENGTH,
        SCALE_STROKE,
    );
    let label_position =
        position + (x_axis + y_axis).normalized() * (LABEL_OFFSET + SCALE_MARKER_RADIUS);
    paint_label(
        painter,
        label_position,
        number.to_string(),
        SCALE_STROKE.color,
    );
}

/// A circle with a cross through it, at a fixed screen size. The cross's arms follow the
/// image axes, so they turn with the view rotation.
fn paint_crosshair(
    painter: &egui::Painter,
    transform: DocumentTransform,
    position: Pos2,
    radius: f32,
    arm_length: f32,
    stroke: Stroke,
) {
    painter.circle_stroke(position, radius, stroke);
    for axis in transform.image_axes() {
        let arm = axis * arm_length;
        painter.line_segment([position - arm, position + arm], stroke);
    }
}

/// Marks the document origin (shown only in Origin mode).
fn paint_origin(painter: &egui::Painter, transform: DocumentTransform, origin: Point) {
    paint_crosshair(
        painter,
        transform,
        transform.image_to_screen(origin),
        ORIGIN_CIRCLE_DIAMETER / 2.0,
        ORIGIN_CROSS_LENGTH / 2.0,
        ORIGIN_STROKE,
    );
}

/// The image-space ends of a grid alignment source as currently displayed.
fn source_segment(
    document: &Document,
    selection: &Selection,
    source: GridSource,
) -> Option<(Point, Point)> {
    match source {
        GridSource::ScaleLine(line) => document
            .calibration()
            .points()
            .map(|points| points.line(line)),
        GridSource::Measurement(id) => {
            let stored = &document.measurements()[document.measurement_index(id)?];
            let measurement = selection.displayed(id, stored);
            Some((measurement.start(), measurement.end()))
        }
    }
}

/// Always shows the scale line the active grid is aligned to, even in modes that hide
/// scale lines. (Measurements are drawn in every mode already.)
fn paint_active_grid_source(
    painter: &egui::Painter,
    transform: DocumentTransform,
    document: &Document,
    selection: &Selection,
) {
    let Some(index) = selection.active().and_then(|id| document.grid_index(id)) else {
        return;
    };
    let GridSource::ScaleLine(line) = document.grids()[index].source() else {
        return;
    };
    let Some(points) = document.calibration().points() else {
        return;
    };
    let (start_index, end_index) = points.method().line_endpoints(line);
    let (start, end) = points.line(line);
    let [start, end] = [start, end].map(|point| transform.image_to_screen(point));
    paint_labelled_line(painter, start, end, SCALE_STROKE, line.to_string());
    paint_scale_point(painter, transform, start, start_index + 1);
    paint_scale_point(painter, transform, end, end_index + 1);
}

/// Highlights the line the grid tool would align a new grid to.
fn paint_highlighted_source(
    painter: &egui::Painter,
    transform: DocumentTransform,
    document: &Document,
    selection: &Selection,
    source: GridSource,
) {
    if let Some((start, end)) = source_segment(document, selection, source) {
        painter.line_segment(
            [
                transform.image_to_screen(start),
                transform.image_to_screen(end),
            ],
            HIGHLIGHT_STROKE,
        );
    }
}

/// Draws every measurement as a labelled line with a handle at each end, in its colour.
fn paint_measurements(
    painter: &egui::Painter,
    transform: DocumentTransform,
    document: &Document,
    selection: &Selection,
) {
    for (index, stored) in document.measurements().iter().enumerate() {
        let measurement = selection.displayed(stored.id(), stored);
        let [start, end] =
            Endpoint::ALL.map(|endpoint| transform.image_to_screen(measurement.point(endpoint)));
        let label = match measurement.distance(document.calibration()) {
            Ok(distance) => format!(
                "{}: {} {}",
                index + 1,
                format_length(distance),
                measurement.unit()
            ),
            Err(_) => (index + 1).to_string(),
        };
        let color = to_color32(measurement.color());
        paint_labelled_line(
            painter,
            start,
            end,
            Stroke::new(OBJECT_LINE_WIDTH, color),
            label,
        );
        for position in [start, end] {
            paint_point_handle(painter, position, color);
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
    let color = to_color32(measurement::DEFAULT_COLOR);
    let start = transform.image_to_screen(start);
    if let Some(pointer) = pointer {
        painter.extend(Shape::dashed_line(
            &[start, pointer],
            Stroke::new(OBJECT_LINE_WIDTH, color),
            DASH_LENGTH,
            DASH_GAP,
        ));
    }
    paint_point_handle(painter, start, color);
}

/// Draws every area as a filled outline with a handle at each corner and its area
/// written at its centre, in its colour.
fn paint_areas(
    painter: &egui::Painter,
    transform: DocumentTransform,
    document: &Document,
    selection: &Selection,
) {
    for (index, stored) in document.areas().iter().enumerate() {
        let area = selection.displayed_area(stored.id(), stored);
        let color = to_color32(area.color());
        let on_screen = to_screen(transform, area.points());
        paint_filled_outline(painter, area.points(), &on_screen, area.color());
        for &position in &on_screen {
            paint_point_handle(painter, position, color);
        }
        let label = match area.area(document.calibration()) {
            Ok(value) => format!(
                "{}: {} {}",
                index + 1,
                format_length(value),
                area.unit().area_symbol()
            ),
            Err(_) => (index + 1).to_string(),
        };
        paint_label(
            painter,
            transform.image_to_screen(area.center()),
            label,
            color,
        );
    }
}

/// Draws the outline being placed: the points so far joined by solid lines, and a dashed
/// line on to the pointer. Over the first point, once the outline can be closed, the
/// dashed line closes the outline and the area is shown filled.
fn paint_pending_area(
    painter: &egui::Painter,
    transform: DocumentTransform,
    tool: &AreaTool,
    pointer: Option<Pos2>,
) {
    let points = tool.points();
    let on_screen = to_screen(transform, points);
    let (Some(&first), Some(&last)) = (on_screen.first(), on_screen.last()) else {
        return;
    };
    let rgb = gridtool::area::DEFAULT_COLOR;
    let stroke = Stroke::new(OBJECT_LINE_WIDTH, to_color32(rgb));
    let closing = tool.can_close()
        && pointer.and_then(|pointer| AreaTool::hit_at(&on_screen, pointer, POINT_GRAB_RADIUS))
            == Some(PointHit::First);

    if closing {
        paint_filled_outline(painter, points, &on_screen, rgb);
    } else {
        painter.add(Shape::line(on_screen.clone(), stroke));
        if let Some(pointer) = pointer {
            painter.extend(Shape::dashed_line(
                &[last, pointer],
                stroke,
                DASH_LENGTH,
                DASH_GAP,
            ));
        }
    }
    for &position in &on_screen {
        paint_point_handle(painter, position, stroke.color);
    }
    if tool.can_close() {
        painter.circle_stroke(first, CLOSE_TARGET_RADIUS, stroke);
    }
}

/// Fills the outline through `points` (image coordinates, drawn at `on_screen`) with a
/// translucent `color`, then strokes its edges. Concave outlines are triangulated.
fn paint_filled_outline(
    painter: &egui::Painter,
    points: &[Point],
    on_screen: &[Pos2],
    color: gridtool::color::Rgb,
) {
    let fill = with_opacity(color, AREA_FILL_OPACITY);
    let mut mesh = egui::Mesh::default();
    for &position in on_screen {
        mesh.colored_vertex(position, fill);
    }
    for triangle in polygon::triangulate(points) {
        let [a, b, c] = triangle.map(|index| index as u32);
        mesh.add_triangle(a, b, c);
    }
    painter.add(mesh);
    painter.add(Shape::closed_line(
        on_screen.to_vec(),
        Stroke::new(OBJECT_LINE_WIDTH, to_color32(color)),
    ));
}

fn to_screen(transform: DocumentTransform, points: &[Point]) -> Vec<Pos2> {
    points
        .iter()
        .map(|&point| transform.image_to_screen(point))
        .collect()
}

/// A measurement end or area corner.
fn paint_point_handle(painter: &egui::Painter, position: Pos2, color: Color32) {
    painter.circle_filled(position, POINT_HANDLE_RADIUS, color);
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
    /// Where the test measurement's start, image point (50, 50) of the 200×100 test
    /// document, is drawn at the default view. Its end (150, 50) is at (450, 300).
    const MEASUREMENT_START_ON_SCREEN: Pos2 = Pos2::new(350.0, 300.0);
    const MEASUREMENT_MIDDLE_ON_SCREEN: Pos2 = Pos2::new(400.0, 300.0);
    /// The selection box's right-edge handle: the end, pushed out by the padding.
    const RIGHT_RESIZE_HANDLE: Pos2 = Pos2::new(450.0 + SELECTION_PADDING, 300.0);
    const EMPTY_SPOT: Pos2 = Pos2::new(600.0, 500.0);
    /// Image points of a test triangle, and where they are drawn at the default view (the
    /// document's top-left corner is at (300, 250) on screen).
    const TRIANGLE: [(f64, f64); 3] = [(20.0, 20.0), (60.0, 20.0), (40.0, 60.0)];
    const TRIANGLE_ON_SCREEN: [Pos2; 3] = [
        Pos2::new(320.0, 270.0),
        Pos2::new(360.0, 270.0),
        Pos2::new(340.0, 310.0),
    ];
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

        fn click(&mut self, at: Pos2) {
            let press = |pressed| Event::PointerButton {
                pos: at,
                button: PointerButton::Primary,
                pressed,
                modifiers: Modifiers::NONE,
            };
            self.frame(vec![Event::PointerMoved(at)]);
            self.frame(vec![press(true)]);
            self.frame(vec![press(false)]);
            self.frame(Vec::new());
        }

        fn select_measurement(&mut self) {
            let id = self.document.measurements()[0].id();
            self.view.selection.select(Some(id));
        }

        /// Adds the triangle (20, 20), (60, 20), (40, 60) and makes it the active object.
        /// On screen its corners are at [`TRIANGLE_ON_SCREEN`].
        fn select_new_area(&mut self) {
            Action::AddArea(
                gridtool::area::Area::new(
                    TRIANGLE.iter().map(|&(x, y)| Point::new(x, y)).collect(),
                    LengthUnit::Centimetre,
                )
                .unwrap(),
            )
            .apply(&mut self.document)
            .unwrap();
            let id = self.document.areas()[0].id();
            self.view.selection.select(Some(id));
        }

        /// How far the view has been panned from its default.
        fn pan(&self) -> Vec2 {
            self.view
                .viewport
                .transform(Pos2::ZERO, Vec2::ZERO)
                .center()
                .to_vec2()
        }

        fn transform(&self) -> DocumentTransform {
            self.view
                .viewport
                .transform(SCREEN.center(), Vec2::new(200.0, 100.0))
        }
    }

    #[test]
    fn the_view_rotates_about_the_origin_and_the_arrow_starts_there() {
        let mut harness = Harness::new(ToolMode::Rotate);
        let origin = Point::new(50.0, 50.0);
        Action::SetOrigin(origin)
            .apply(&mut harness.document)
            .unwrap();
        harness
            .view
            .viewport
            .zoom_about(SCREEN.center(), Pos2::new(350.0, 300.0), 3.0);
        let pivot = harness.transform().image_to_screen(origin);

        // Zoomed ×3 about the origin, which stays at (350, 300). The tip is still 200 px
        // right of the origin; drag it to straight below.
        let tip = rotate_handle::tip(pivot, 0.0);
        harness.drag(
            PointerButton::Primary,
            tip,
            Vec2::new(-rotate_handle::ARROW_LENGTH, rotate_handle::ARROW_LENGTH),
        );

        assert!((harness.view.viewport.rotation_degrees() - 90.0).abs() < 1e-3);
        let now = harness.transform().image_to_screen(origin);
        assert!((now - pivot).length() < 1e-2, "origin moved to {now:?}");
        assert!(harness.actions.is_empty(), "rotation is not an action");
    }

    #[test]
    fn clicking_the_image_in_origin_mode_sets_the_origin() {
        let mut harness = Harness::new(ToolMode::Origin);
        // The document spans (300, 250)–(500, 350) on screen.
        harness.click(Pos2::new(320.0, 260.0));

        assert_eq!(harness.actions, [Action::SetOrigin(Point::new(20.0, 10.0))]);
    }

    #[test]
    fn clicking_outside_the_image_in_origin_mode_does_nothing() {
        let mut harness = Harness::new(ToolMode::Origin);
        harness.click(EMPTY_SPOT);
        // Clicking the current origin (the centre) changes nothing either.
        harness.click(SCREEN.center());

        assert!(harness.actions.is_empty());
    }

    #[test]
    fn measurement_is_where_the_tests_expect() {
        let harness = Harness::new(ToolMode::Measure);
        let transform = harness
            .view
            .viewport
            .transform(SCREEN.center(), Vec2::new(200.0, 100.0));

        assert_eq!(
            transform.image_to_screen(Point::new(50.0, 50.0)),
            MEASUREMENT_START_ON_SCREEN
        );
    }

    #[test]
    fn resize_handles_sit_at_corners_and_edge_midpoints() {
        let corners = [
            Pos2::new(0.0, 0.0),
            Pos2::new(10.0, 0.0),
            Pos2::new(10.0, 20.0),
            Pos2::new(0.0, 20.0),
        ];

        let handles = resize_handle_positions(corners);

        assert_eq!(handles.len(), BoundsHandle::ALL.len());
        assert!(handles.contains(&(BoundsHandle::TopLeft, Pos2::new(0.0, 0.0))));
        assert!(handles.contains(&(BoundsHandle::Right, Pos2::new(10.0, 10.0))));
        assert!(handles.contains(&(BoundsHandle::Bottom, Pos2::new(5.0, 20.0))));
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
    fn right_drag_on_a_resize_handle_pans_instead_of_resizing() {
        let mut harness = Harness::new(ToolMode::Pan);
        harness.select_measurement();

        harness.drag(PointerButton::Secondary, RIGHT_RESIZE_HANDLE, DRAG_BY);

        assert_eq!(harness.pan(), DRAG_BY);
        assert!(harness.actions.is_empty());
    }

    #[test]
    fn left_drag_on_a_resize_handle_resizes_in_every_mode() {
        for mode in ToolMode::ALL {
            let mut harness = Harness::new(mode);
            harness.select_measurement();

            harness.drag(PointerButton::Primary, RIGHT_RESIZE_HANDLE, DRAG_BY);

            // The right edge only moves horizontally; it carries the line's end.
            assert_eq!(
                harness.actions,
                [Action::ResizeMeasurement {
                    index: 0,
                    start: Point::new(50.0, 50.0),
                    end: Point::new(190.0, 50.0),
                }],
                "{mode:?}"
            );
            assert_eq!(harness.pan(), Vec2::ZERO, "{mode:?} should not pan");
        }
    }

    #[test]
    fn the_ends_of_an_inactive_measurement_cannot_be_dragged() {
        for mode in ToolMode::ALL {
            let mut harness = Harness::new(mode);

            harness.drag(PointerButton::Primary, MEASUREMENT_START_ON_SCREEN, DRAG_BY);

            assert!(harness.actions.is_empty(), "{mode:?}");
        }
    }

    #[test]
    fn clicking_outside_the_selection_box_clears_it_in_every_mode() {
        for mode in ToolMode::ALL {
            let mut harness = Harness::new(mode);
            harness.select_measurement();

            harness.click(MEASUREMENT_MIDDLE_ON_SCREEN);
            assert!(harness.view.selection.active().is_some(), "{mode:?} inside");

            harness.click(EMPTY_SPOT);
            assert_eq!(harness.view.selection.active(), None, "{mode:?} outside");
        }
    }

    fn grid_on_the_measurement(harness: &Harness) -> Action {
        let id = harness.document.measurements()[0].id();
        Action::AddGrid(gridtool::grid::Grid::new(
            GridSource::Measurement(id),
            LengthUnit::Centimetre,
        ))
    }

    #[test]
    fn clicking_a_measurement_line_in_grid_mode_adds_a_grid() {
        let mut harness = Harness::new(ToolMode::Grid);
        harness
            .view
            .grid_tool
            .set_via(super::super::grid_tool::GridVia::Measurement);

        harness.click(MEASUREMENT_MIDDLE_ON_SCREEN + Vec2::new(0.0, 4.0));

        assert_eq!(harness.actions, [grid_on_the_measurement(&harness)]);
    }

    #[test]
    fn grid_mode_ignores_line_ends_and_clicks_too_far_from_the_line() {
        let mut harness = Harness::new(ToolMode::Grid);
        harness
            .view
            .grid_tool
            .set_via(super::super::grid_tool::GridVia::Measurement);

        harness.click(MEASUREMENT_START_ON_SCREEN + Vec2::new(4.0, 0.0));
        harness.click(MEASUREMENT_MIDDLE_ON_SCREEN + Vec2::new(0.0, 7.0));

        assert!(harness.actions.is_empty());
    }

    #[test]
    fn via_scale_does_not_pick_measurement_lines() {
        let mut harness = Harness::new(ToolMode::Grid);

        harness.click(MEASUREMENT_MIDDLE_ON_SCREEN);

        assert!(harness.actions.is_empty());
    }

    #[test]
    fn hovering_a_line_in_grid_mode_highlights_it() {
        let mut harness = Harness::new(ToolMode::Grid);
        harness
            .view
            .grid_tool
            .set_via(super::super::grid_tool::GridVia::Measurement);
        let id = harness.document.measurements()[0].id();

        harness.frame(vec![Event::PointerMoved(MEASUREMENT_MIDDLE_ON_SCREEN)]);
        assert_eq!(
            harness.view.grid_tool.hovered(),
            Some(GridSource::Measurement(id))
        );

        harness.frame(vec![Event::PointerMoved(EMPTY_SPOT)]);
        assert_eq!(harness.view.grid_tool.hovered(), None);
    }

    #[test]
    fn any_click_on_the_canvas_clears_an_active_grid() {
        let mut harness = Harness::new(ToolMode::Pan);
        grid_on_the_measurement(&harness)
            .apply(&mut harness.document)
            .unwrap();
        let grid_id = harness.document.grids()[0].id();
        harness.view.selection.select(Some(grid_id));

        harness.click(MEASUREMENT_MIDDLE_ON_SCREEN);

        assert_eq!(harness.view.selection.active(), None);
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

    #[test]
    fn wheel_zoom_keeps_the_point_under_the_cursor_in_every_mode() {
        for mode in ToolMode::ALL {
            let mut harness = Harness::new(mode);
            let cursor = Pos2::new(470.0, 330.0);
            let under_cursor = harness.transform().screen_to_image(cursor);

            harness.frame(vec![Event::PointerMoved(cursor)]);
            harness.frame(vec![Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                delta: Vec2::new(0.0, 300.0),
                phase: egui::TouchPhase::Move,
                modifiers: Modifiers::NONE,
                source: egui::MouseWheelSource::Wheel,
            }]);
            for _ in 0..30 {
                harness.frame(Vec::new()); // Let smooth scrolling settle.
            }

            assert!(
                harness.view.viewport.zoom() > 1.5,
                "{mode:?} should zoom in"
            );
            let now = harness.transform().image_to_screen(under_cursor);
            assert!(
                (now - cursor).length() < 1e-2,
                "{mode:?} moved the point under the cursor to {now:?}"
            );
        }
    }

    #[test]
    fn clicking_corners_then_the_first_point_adds_an_area() {
        let mut harness = Harness::new(ToolMode::Area);
        for corner in TRIANGLE_ON_SCREEN {
            harness.click(corner);
        }
        assert!(harness.actions.is_empty());
        assert_eq!(harness.view.area_tool.points().len(), 3);

        // Within reach of the first point, not exactly on it.
        harness.click(TRIANGLE_ON_SCREEN[0] + Vec2::new(4.0, -3.0));

        let expected = gridtool::area::Area::new(
            TRIANGLE.iter().map(|&(x, y)| Point::new(x, y)).collect(),
            LengthUnit::Centimetre,
        )
        .unwrap();
        assert_eq!(harness.actions, [Action::AddArea(expected)]);
        assert!(harness.view.area_tool.points().is_empty());
    }

    #[test]
    fn clicking_the_last_point_removes_it_and_clicks_outside_are_ignored() {
        let mut harness = Harness::new(ToolMode::Area);
        harness.click(TRIANGLE_ON_SCREEN[0]);
        harness.click(TRIANGLE_ON_SCREEN[1]);

        harness.click(TRIANGLE_ON_SCREEN[1] + Vec2::new(2.0, 2.0));
        assert_eq!(harness.view.area_tool.points(), [Point::new(20.0, 20.0)]);

        harness.click(EMPTY_SPOT);
        assert_eq!(harness.view.area_tool.points().len(), 1);
        assert!(harness.actions.is_empty());
    }

    #[test]
    fn the_first_point_does_not_close_fewer_than_three_points() {
        let mut harness = Harness::new(ToolMode::Area);
        harness.click(TRIANGLE_ON_SCREEN[0]);
        harness.click(TRIANGLE_ON_SCREEN[1]);

        harness.click(TRIANGLE_ON_SCREEN[0]);

        assert_eq!(harness.view.area_tool.points().len(), 2);
        assert!(harness.actions.is_empty());
    }

    #[test]
    fn the_active_areas_points_can_be_dragged_in_every_mode() {
        for mode in ToolMode::ALL {
            let mut harness = Harness::new(mode);
            harness.select_new_area();

            harness.drag(PointerButton::Primary, TRIANGLE_ON_SCREEN[2], DRAG_BY);

            assert_eq!(
                harness.actions,
                [Action::MoveAreaPoint {
                    index: 0,
                    point: 2,
                    to: Point::new(80.0, 85.0),
                }],
                "{mode:?}"
            );
            assert_eq!(harness.pan(), Vec2::ZERO, "{mode:?} should not pan");
        }
    }

    #[test]
    fn the_active_areas_box_handles_stretch_it() {
        let mut harness = Harness::new(ToolMode::Pan);
        harness.select_new_area();
        // The right edge handle: the triangle's right edge (x = 60), pushed out by the
        // padding, halfway down (y = 40).
        let right_handle = Pos2::new(360.0 + SELECTION_PADDING, 290.0);

        harness.drag(PointerButton::Primary, right_handle, Vec2::new(40.0, 0.0));

        assert_eq!(
            harness.actions,
            [Action::ResizeArea {
                index: 0,
                points: vec![
                    Point::new(20.0, 20.0),
                    Point::new(100.0, 20.0),
                    Point::new(60.0, 60.0),
                ],
            }]
        );
    }

    #[test]
    fn the_active_measurements_ends_can_be_dragged_in_every_mode() {
        for mode in ToolMode::ALL {
            let mut harness = Harness::new(mode);
            harness.select_measurement();

            harness.drag(PointerButton::Primary, MEASUREMENT_START_ON_SCREEN, DRAG_BY);

            assert_eq!(
                harness.actions,
                [Action::MoveMeasurementPoint {
                    index: 0,
                    endpoint: Endpoint::Start,
                    to: Point::new(90.0, 75.0),
                }],
                "{mode:?}"
            );
        }
    }
}
