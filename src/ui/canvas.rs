//! Draws the document and turns mouse input into view changes or document actions,
//! according to the active tool.

use eframe::egui::{
    self, Align2, Color32, CursorIcon, FontId, Pos2, Rect, Response, Sense, Shape, Stroke,
    TextureHandle, Vec2,
};
use gridtool::action::Action;
use gridtool::calibration::{CalibrationMethod, Line};
use gridtool::document::Document;
use gridtool::geometry::Point;
use gridtool::imaging::DecodedImage;

use super::document_view::DocumentView;
use super::hit_test::nearest_within;
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
const MARKER_STROKE: Stroke = Stroke {
    width: 1.5,
    color: Color32::RED,
};
const MARKER_RADIUS: f32 = 6.0;
const CROSSHAIR_HALF_LENGTH: f32 = 11.0;
const MARKER_GRAB_RADIUS: f32 = 10.0;
const LABEL_OFFSET: f32 = 12.0;
const LABEL_FONT_SIZE: f32 = 14.0;
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
    let action = match mode {
        ToolMode::Pan => {
            handle_pan(ui, &response, &mut view.viewport);
            None
        }
        ToolMode::Rotate => {
            handle_rotate(ui, &response, view_center, document_size, view);
            None
        }
        ToolMode::Scale => {
            let transform = view.viewport.transform(view_center, document_size);
            handle_scale(ui, &response, transform, document, &mut view.scale_tool)
        }
    };

    let transform = view.viewport.transform(view_center, document_size);
    paint_document(&painter, transform, &view.texture);
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
    if response.dragged() {
        viewport.pan_by(response.drag_delta());
        ui.set_cursor_icon(CursorIcon::Grabbing);
    } else if response.hovered() {
        ui.set_cursor_icon(CursorIcon::Grab);
    }
}

fn handle_rotate(
    ui: &egui::Ui,
    response: &Response,
    view_center: Pos2,
    document_size: Vec2,
    view: &mut DocumentView,
) {
    let center = view.viewport.transform(view_center, document_size).center();
    let rotation = view.viewport.rotation_radians();

    if response.drag_started() {
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

    if response.drag_stopped() {
        view.is_dragging_rotate_handle = false;
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
    let marker_positions: Vec<Pos2> = tool
        .points(calibration)
        .into_iter()
        .map(|point| transform.image_to_screen(point))
        .collect();
    let marker_at =
        |position: Pos2| nearest_within(&marker_positions, position, MARKER_GRAB_RADIUS);

    if response.drag_started()
        && !is_placing
        && let Some(index) = press_origin(ui).and_then(marker_at)
    {
        tool.begin_drag(calibration, index);
    }

    if tool.is_dragging() {
        if let Some(pointer) = response.interact_pointer_pos() {
            tool.drag_to(document.clamp(transform.screen_to_image(pointer)));
        }
        ui.set_cursor_icon(CursorIcon::Grabbing);
    } else if let Some(pointer) = response.hover_pos() {
        if is_placing {
            ui.set_cursor_icon(CursorIcon::Crosshair);
        } else if marker_at(pointer).is_some() {
            ui.set_cursor_icon(CursorIcon::Grab);
        }
    }

    if response.drag_stopped() {
        return tool.end_drag();
    }
    if response.clicked() && is_placing {
        let point = transform.screen_to_image(response.interact_pointer_pos()?);
        if document.contains(point) {
            return tool.place(calibration, point);
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
            painter.line_segment([start, end], MARKER_STROKE);
            let normal = (end - start).normalized().rot90();
            paint_label(painter, start.lerp(end, 0.5) + normal * LABEL_OFFSET, line);
        }
    }

    let [x_axis, y_axis] = transform.image_axes();
    for (index, &position) in positions.iter().enumerate() {
        painter.circle_stroke(position, MARKER_RADIUS, MARKER_STROKE);
        for axis in [x_axis, y_axis] {
            let arm = axis * CROSSHAIR_HALF_LENGTH;
            painter.line_segment([position - arm, position + arm], MARKER_STROKE);
        }
        let label_position =
            position + (x_axis + y_axis).normalized() * (LABEL_OFFSET + MARKER_RADIUS);
        paint_label(painter, label_position, index + 1);
    }
}

fn paint_label(painter: &egui::Painter, position: Pos2, text: impl ToString) {
    painter.text(
        position,
        Align2::CENTER_CENTER,
        text.to_string(),
        FontId::proportional(LABEL_FONT_SIZE),
        MARKER_STROKE.color,
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
