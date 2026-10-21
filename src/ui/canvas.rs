use eframe::egui::{self, Color32, CursorIcon, Pos2, Rect, Sense, Shape, Stroke, TextureHandle};

use crate::document::Document;
use crate::imaging::DecodedImage;
use crate::rotate_handle;
use crate::tool_mode::ToolMode;
use crate::viewport::{DocumentTransform, Viewport};

const OUTLINE_STROKE: Stroke = Stroke {
    width: 2.0,
    color: Color32::RED,
};
const ARROW_STROKE: Stroke = Stroke {
    width: 2.0,
    color: Color32::from_rgb(0, 170, 255),
};
/// How strongly one point of mouse-wheel scrolling changes the zoom.
const SCROLL_ZOOM_SPEED: f32 = 0.002;

/// Uploads a decoded image to the GPU so the canvas can draw it.
pub fn upload_texture(ctx: &egui::Context, name: &str, image: &DecodedImage) -> TextureHandle {
    let color_image = egui::ColorImage::from_rgba_unmultiplied(image.size(), &image.rgba);
    ctx.load_texture(name, color_image, egui::TextureOptions::default())
}

/// Interactive view of the document: draws it and turns mouse input into view or
/// document changes according to the active tool.
#[derive(Default)]
pub struct Canvas {
    viewport: Viewport,
    is_dragging_rotate_handle: bool,
}

impl Canvas {
    /// Returns to the default view (document centered at 100% zoom).
    pub fn reset_view(&mut self) {
        *self = Self::default();
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        mode: ToolMode,
        document: &mut Document,
        texture: &TextureHandle,
    ) {
        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());
        let view_center = response.rect.center();

        self.handle_zoom(ui, &response, view_center);
        match mode {
            ToolMode::Pan => self.handle_pan(ui, &response),
            ToolMode::Rotate => self.handle_rotate(ui, &response, view_center, document),
        }

        let transform = self.viewport.transform(view_center, document.rotation_radians());
        paint_document(&painter, &transform, document, texture);
        if mode == ToolMode::Rotate {
            paint_rotate_handle(&painter, &transform, document);
        }
    }

    fn handle_zoom(&mut self, ui: &egui::Ui, response: &egui::Response, view_center: Pos2) {
        let Some(pointer) = response.hover_pos() else {
            return;
        };
        let (scroll, pinch) = ui.input(|input| (input.smooth_scroll_delta().y, input.zoom_delta()));
        let factor = pinch * (scroll * SCROLL_ZOOM_SPEED).exp();
        if factor != 1.0 {
            self.viewport.zoom_about(view_center, pointer, factor);
        }
    }

    fn handle_pan(&mut self, ui: &egui::Ui, response: &egui::Response) {
        if response.dragged() {
            self.viewport.pan_by(response.drag_delta());
            ui.set_cursor_icon(CursorIcon::Grabbing);
        } else if response.hovered() {
            ui.set_cursor_icon(CursorIcon::Grab);
        }
    }

    fn handle_rotate(
        &mut self,
        ui: &egui::Ui,
        response: &egui::Response,
        view_center: Pos2,
        document: &mut Document,
    ) {
        let center = self
            .viewport
            .transform(view_center, document.rotation_radians())
            .center();
        let rotation = document.rotation_radians();

        if response.drag_started() {
            let press_origin = ui.input(|input| input.pointer.press_origin());
            self.is_dragging_rotate_handle = press_origin
                .is_some_and(|origin| rotate_handle::is_over_tip(center, rotation, origin));
        }

        if self.is_dragging_rotate_handle {
            if let Some(radians) = response
                .interact_pointer_pos()
                .and_then(|pointer| rotate_handle::rotation_towards(center, pointer))
            {
                document.set_rotation_radians(radians);
            }
            ui.set_cursor_icon(CursorIcon::Grabbing);
        } else if response
            .hover_pos()
            .is_some_and(|pointer| rotate_handle::is_over_tip(center, rotation, pointer))
        {
            ui.set_cursor_icon(CursorIcon::Grab);
        }

        if response.drag_stopped() {
            self.is_dragging_rotate_handle = false;
        }
    }
}

fn paint_document(
    painter: &egui::Painter,
    transform: &DocumentTransform,
    document: &Document,
    texture: &TextureHandle,
) {
    painter.add(textured_quad(transform.corners(document.size()), texture));

    // The outline's centre line sits half a stroke outside the document, so the whole
    // stroke lies around the document rather than on top of it.
    let outline = transform.corners_outset(document.size(), OUTLINE_STROKE.width / 2.0);
    painter.add(Shape::closed_line(outline.to_vec(), OUTLINE_STROKE));
}

fn paint_rotate_handle(painter: &egui::Painter, transform: &DocumentTransform, document: &Document) {
    let center = transform.center();
    let tip = rotate_handle::tip(center, document.rotation_radians());
    painter.arrow(center, tip - center, ARROW_STROKE);
    painter.circle_stroke(tip, rotate_handle::GRAB_RADIUS, ARROW_STROKE);
}

/// A mesh drawing the whole texture onto the quad with the given corners
/// (top-left, top-right, bottom-right, bottom-left).
fn textured_quad(corners: [Pos2; 4], texture: &TextureHandle) -> egui::Mesh {
    let uv = Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0));
    let uvs = [uv.left_top(), uv.right_top(), uv.right_bottom(), uv.left_bottom()];

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
