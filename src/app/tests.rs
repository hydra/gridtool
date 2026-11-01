//! Tests of the whole application, driven without a window: real layouts, simulated pointer
//! input and scripted file dialogs.

use std::fs;
use std::path::{Path, PathBuf};

use eframe::egui::{Event, Modifiers, PointerButton, Pos2, RawInput, Rect, Vec2};
use gridtool::geometry::Point;

use super::*;
use crate::file_picker::{DocumentFilePicker, ImageFilePicker};

const WINDOW: Rect = Rect::from_min_max(Pos2::ZERO, Pos2::new(1280.0, 800.0));
const IMAGE_WIDTH: u32 = 2000;
const IMAGE_HEIGHT: u32 = 1500;
/// How far above a canvas the middle of its tab is, and how far in from its left edge.
const TAB_ABOVE: f32 = 12.0;
const TAB_IN: f32 = 20.0;
/// From the middle of a canvas, where a new document's origin is, to where a click moves it.
const OFF_CENTRE: Vec2 = Vec2::new(20.0, 15.0);

/// Always picks the same files, so that the file dialogs need no user.
#[derive(Clone)]
struct Picks {
    image: PathBuf,
    document: PathBuf,
}

impl ImageFilePicker for Picks {
    fn pick_image(&self) -> Option<PathBuf> {
        Some(self.image.clone())
    }
}

impl DocumentFilePicker for Picks {
    fn pick_document(&self) -> Option<PathBuf> {
        Some(self.document.clone())
    }

    fn pick_save_path(&self, _suggestion: &Path) -> Option<PathBuf> {
        Some(self.document.clone())
    }
}

/// The whole application, with a temporary image bigger than its window to open.
struct Fixture {
    folder: PathBuf,
    ctx: egui::Context,
    app: GridToolApp,
    picks: Picks,
    time: f64,
    /// The cursor asked for by the last frame.
    cursor: egui::CursorIcon,
}

impl Fixture {
    fn new(name: &str) -> Self {
        let folder =
            std::env::temp_dir().join(format!("gridtool_app_{name}_{}", std::process::id()));
        fs::create_dir_all(&folder).unwrap();
        let picks = Picks {
            image: folder.join("plan.png"),
            document: folder.join("plan.ron"),
        };
        image::RgbaImage::new(IMAGE_WIDTH, IMAGE_HEIGHT)
            .save(&picks.image)
            .unwrap();
        let session = Session::new(Box::new(picks.clone()), Box::new(picks.clone()));
        let ctx = egui::Context::default();
        egui_material_icons::initialize(&ctx);
        let mut fixture = Self {
            folder,
            ctx,
            app: GridToolApp::new(session, Workspace::default()),
            picks,
            time: 0.0,
            cursor: egui::CursorIcon::Default,
        };
        fixture.settle();
        fixture
    }

    fn frame(&mut self, events: Vec<Event>) {
        self.time += 0.1;
        let input = RawInput {
            screen_rect: Some(WINDOW),
            time: Some(self.time),
            events,
            ..Default::default()
        };
        let ctx = self.ctx.clone();
        let mut output = ctx.run_ui(input, |ui| self.app.show(ui));
        // There is no renderer to upload textures to.
        output.textures_delta.clear();
        self.cursor = output.platform_output.cursor_icon;
    }

    fn settle(&mut self) {
        for _ in 0..5 {
            self.frame(Vec::new());
        }
    }

    /// Opens another document with New, returning it once it has been laid out.
    fn new_document(&mut self) -> DocumentId {
        let ctx = self.ctx.clone();
        self.app.session.open_image(&ctx);
        self.settle();
        self.app.session.active_id().expect("the document opened")
    }

    fn press(&mut self, pos: Pos2, pressed: bool) {
        self.frame(vec![Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        }]);
    }

    fn click(&mut self, pos: Pos2) {
        self.frame(vec![Event::PointerMoved(pos)]);
        self.press(pos, true);
        self.press(pos, false);
        self.settle();
    }

    fn drag(&mut self, from: Pos2, to: Pos2) {
        self.frame(vec![Event::PointerMoved(from)]);
        self.press(from, true);
        for step in 1..=8 {
            let along = step as f32 / 8.0;
            self.frame(vec![Event::PointerMoved(from + (to - from) * along)]);
        }
        self.frame(vec![Event::PointerMoved(to)]);
        self.press(to, false);
        self.settle();
    }

    fn workspace(&self) -> &Workspace {
        &self.app.workspace
    }

    fn open(&mut self) -> &mut OpenDocument {
        self.app.session.document_mut().expect("a document is open")
    }

    fn active(&self) -> Option<DocumentId> {
        self.app.session.active_id()
    }

    fn canvas_rect(&self, document: DocumentId) -> Rect {
        self.workspace()
            .area()
            .canvas_rect_of(document)
            .expect("the document is in view")
    }

    fn tab_of_canvas(&self, canvas: Rect) -> Pos2 {
        Pos2::new(canvas.left() + TAB_IN, canvas.top() - TAB_ABOVE)
    }

    fn panel_rect(&self, panel: Panel) -> Rect {
        self.workspace()
            .panel_rect(panel)
            .expect("the panel is open")
    }

    fn panel_tab(&self, panel: Panel) -> Pos2 {
        self.tab_of_canvas(self.panel_rect(panel))
    }

    /// Checks that the whole of the active document is inside its canvas.
    fn assert_active_document_fits_its_canvas(&mut self) {
        self.settle();
        let view = &self.open().view;
        let canvas = view.canvas_rect.expect("the canvas was drawn");
        let size = Vec2::new(IMAGE_WIDTH as f32, IMAGE_HEIGHT as f32);
        let corners = view.viewport.transform(canvas.center(), size).corners();
        assert!(
            corners.iter().all(|corner| canvas.contains(*corner)),
            "{corners:?} outside {canvas:?}"
        );
        assert!(view.viewport.zoom() < 1.0);
    }

    /// Two documents, split side by side, the second active.
    fn two_documents_side_by_side(&mut self) -> (DocumentId, DocumentId) {
        let first = self.new_document();
        let second = self.new_document();
        self.app.workspace.area_mut().split_off(second);
        self.settle();
        assert_eq!(self.workspace().area().visible_documents(), [first, second]);
        (first, second)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.folder);
    }
}

#[test]
fn an_image_opened_with_new_fits_the_canvas() {
    let mut fixture = Fixture::new("open_image");

    fixture.new_document();

    fixture.assert_active_document_fits_its_canvas();
    let canvas = fixture.open().view.canvas_rect.unwrap();
    assert!(
        canvas.width() < WINDOW.width(),
        "the panels have their share"
    );
}

#[test]
fn a_loaded_document_fits_the_canvas() {
    let mut fixture = Fixture::new("load");
    let id = fixture.new_document();
    let path = fixture.picks.document.clone();
    fixture.open().save_to(&path).unwrap();
    let ctx = fixture.ctx.clone();
    fixture.app.session.close(&ctx, id);
    fixture.settle();

    fixture.app.session.load(&ctx);

    fixture.assert_active_document_fits_its_canvas();
}

#[test]
fn zooming_after_opening_is_kept() {
    let mut fixture = Fixture::new("zoom_kept");
    fixture.new_document();
    let center = fixture.open().view.canvas_rect.unwrap().center();
    let fitted = fixture.open().view.viewport.zoom();
    fixture.open().view.viewport.zoom_about(center, center, 2.0);

    fixture.settle();

    assert_eq!(fixture.open().view.viewport.zoom(), fitted * 2.0);
}

#[test]
fn fitting_leaves_the_document_and_its_history_alone() {
    let mut fixture = Fixture::new("document_unchanged");
    let ctx = fixture.ctx.clone();
    fixture.app.session.open_image(&ctx);
    let before = fixture.open().editor.document().clone();

    fixture.settle();

    assert_eq!(fixture.open().editor.document(), &before);
    assert!(!fixture.open().editor.can_undo());
}

#[test]
fn new_adds_a_tab_beside_the_active_document_and_keeps_the_others() {
    let mut fixture = Fixture::new("new_adds");

    let first = fixture.new_document();
    let second = fixture.new_document();

    assert_eq!(fixture.workspace().area().groups(), [vec![first, second]]);
    assert_eq!(fixture.workspace().area().visible_documents(), [second]);
    assert_eq!(fixture.active(), Some(second));
}

#[test]
fn clicking_a_tab_brings_its_document_forward_and_makes_it_active() {
    let mut fixture = Fixture::new("tab_click");
    let first = fixture.new_document();
    let second = fixture.new_document();
    let canvas = fixture.canvas_rect(second);

    fixture.click(fixture.tab_of_canvas(canvas));

    assert_eq!(fixture.active(), Some(first));
    assert_eq!(fixture.workspace().area().visible_documents(), [first]);
}

#[test]
fn closing_the_active_document_makes_the_one_beside_its_tab_active() {
    let mut fixture = Fixture::new("close_active");
    let first = fixture.new_document();
    let second = fixture.new_document();
    let ctx = fixture.ctx.clone();

    // A never-saved document asks first.
    fixture.app.session.close(&ctx, second);
    assert_eq!(
        fixture.app.session.dialog(),
        Some(crate::ui::dialog::Dialog::unsaved_changes("plan.png"))
    );
    fixture
        .app
        .session
        .answer(&ctx, crate::ui::dialog::DialogButton::No);
    fixture.settle();

    assert_eq!(fixture.active(), Some(first));
    assert_eq!(fixture.workspace().area().groups(), [vec![first]]);
}

#[test]
fn closing_the_last_document_empties_the_area_and_a_new_one_opens_in_it() {
    let mut fixture = Fixture::new("close_last");
    let only = fixture.new_document();
    let ctx = fixture.ctx.clone();
    fixture.app.session.close(&ctx, only);
    fixture
        .app
        .session
        .answer(&ctx, crate::ui::dialog::DialogButton::No);
    fixture.settle();
    assert_eq!(fixture.active(), None);
    assert!(fixture.workspace().area().groups().is_empty());

    let next = fixture.new_document();

    assert_eq!(fixture.workspace().area().groups(), [vec![next]]);
}

#[test]
fn clicking_another_document_in_view_only_makes_it_active() {
    let mut fixture = Fixture::new("click_activates");
    let (first, second) = fixture.two_documents_side_by_side();
    for document in [first, second] {
        fixture
            .app
            .session
            .document_by_id_mut(document)
            .unwrap()
            .view
            .mode = ToolMode::Origin;
    }
    let target = fixture.canvas_rect(first).center() + OFF_CENTRE;
    let before = fixture
        .app
        .session
        .document_by_id(first)
        .unwrap()
        .editor
        .document()
        .origin();

    fixture.click(target);

    assert_eq!(fixture.active(), Some(first));
    let open = fixture.app.session.document_by_id(first).unwrap();
    assert_eq!(
        open.editor.document().origin(),
        before,
        "the tool did nothing"
    );
    assert!(!open.editor.can_undo());

    // Now that it is active, the same click reaches the tool.
    fixture.click(target);
    assert!(fixture.open().editor.can_undo());
}

#[test]
fn dragging_on_another_document_only_makes_it_active() {
    let mut fixture = Fixture::new("drag_activates");
    let (first, second) = fixture.two_documents_side_by_side();
    let canvas = fixture.canvas_rect(first);
    fixture
        .app
        .session
        .document_by_id_mut(first)
        .unwrap()
        .view
        .mode = ToolMode::Origin;
    let _ = second;

    fixture.drag(canvas.center(), canvas.center() + Vec2::new(30.0, 30.0));

    assert_eq!(fixture.active(), Some(first));
    assert!(!fixture.open().editor.can_undo());
}

#[test]
fn the_wheel_zooms_whichever_document_is_under_the_pointer() {
    let mut fixture = Fixture::new("wheel");
    let (first, second) = fixture.two_documents_side_by_side();
    let zoom_of = |fixture: &Fixture, id| {
        fixture
            .app
            .session
            .document_by_id(id)
            .unwrap()
            .view
            .viewport
            .zoom()
    };
    let (first_zoom, second_zoom) = (zoom_of(&fixture, first), zoom_of(&fixture, second));
    let over_first = fixture.canvas_rect(first).center();

    fixture.frame(vec![Event::PointerMoved(over_first)]);
    for _ in 0..3 {
        fixture.frame(vec![Event::Zoom(1.5)]);
    }

    assert!(zoom_of(&fixture, first) > first_zoom);
    assert_eq!(zoom_of(&fixture, second), second_zoom);
    assert_eq!(fixture.active(), Some(second), "zooming does not activate");
}

#[test]
fn the_tool_belongs_to_each_document() {
    let mut fixture = Fixture::new("tool_per_document");
    let first = fixture.new_document();
    fixture.open().view.mode = ToolMode::Measure;
    let second = fixture.new_document();
    assert_eq!(fixture.open().view.mode, ToolMode::default());
    fixture.open().view.mode = ToolMode::Origin;

    fixture.app.session.activate(first);
    fixture.settle();
    assert_eq!(fixture.active_mode_of_test(), ToolMode::Measure);

    fixture.app.session.activate(second);
    fixture.settle();
    assert_eq!(fixture.active_mode_of_test(), ToolMode::Origin);
}

impl Fixture {
    fn active_mode_of_test(&self) -> ToolMode {
        self.app.active_mode()
    }
}

#[test]
fn a_tool_action_only_reaches_the_active_document() {
    let mut fixture = Fixture::new("action_routing");
    let (first, second) = fixture.two_documents_side_by_side();
    fixture.open().view.mode = ToolMode::Origin;

    let target = fixture.canvas_rect(second).center() + OFF_CENTRE;
    fixture.click(target);

    let session = &fixture.app.session;
    assert!(session.document_by_id(second).unwrap().editor.can_undo());
    assert!(!session.document_by_id(first).unwrap().editor.can_undo());
}

#[test]
fn panels_show_the_active_document_and_undo_acts_on_it() {
    let mut fixture = Fixture::new("undo_routing");
    let first = fixture.new_document();
    fixture
        .open()
        .perform(Action::SetOrigin(Point::new(1.0, 1.0)));
    let second = fixture.new_document();
    fixture
        .open()
        .perform(Action::SetOrigin(Point::new(2.0, 2.0)));
    let ctx = fixture.ctx.clone();

    fixture.app.handle(&ctx, TopPanelAction::Undo);

    let session = &fixture.app.session;
    assert!(session.document_by_id(first).unwrap().editor.can_undo());
    assert!(!session.document_by_id(second).unwrap().editor.can_undo());
}

/// Types into the first coordinate field of the Point-to-point table, which it finds by clicking
/// from just past the row's number and colour swatch.
fn start_editing_a_coordinate(fixture: &mut Fixture, text: &str) {
    let panel = fixture.panel_rect(Panel::PointToPoint).left_top();
    let candidates = (20..70).step_by(4).flat_map(|y| {
        (60..160)
            .step_by(10)
            .map(move |x| panel + Vec2::new(x as f32, y as f32))
    });
    for pos in candidates {
        fixture.click(pos);
        if fixture.ctx.memory(|memory| memory.focused().is_some()) {
            break;
        }
    }
    fixture.frame(vec![Event::Key {
        key: egui::Key::A,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::COMMAND,
    }]);
    fixture.frame(vec![Event::Text(text.to_owned())]);
}

#[test]
fn a_field_being_edited_finishes_on_its_own_document_when_another_is_clicked() {
    let mut fixture = Fixture::new("edit_commit");
    let (first, second) = fixture.two_documents_side_by_side();
    let measurement = gridtool::measurement::Measurement::new(
        Point::new(10.0, 10.0),
        Point::new(100.0, 10.0),
        gridtool::units::LengthUnit::Centimetre,
    );
    fixture.open().perform(Action::AddMeasurement(measurement));
    fixture.settle();
    assert_eq!(fixture.active(), Some(second));
    assert!(fixture.ctx.memory(|memory| memory.focused().is_none()));

    start_editing_a_coordinate(&mut fixture, "55");
    let focused = fixture.ctx.memory(|memory| memory.focused().is_some());
    assert!(focused, "the test has reached a field");
    let target = fixture.canvas_rect(first).center();

    fixture.click(target);

    assert_eq!(fixture.active(), Some(first));
    assert!(fixture.ctx.memory(|memory| memory.focused().is_none()));
    let session = &fixture.app.session;
    let edited = &session.document_by_id(second).unwrap().editor;
    assert_eq!(edited.document().measurements()[0].start().x, 55.0);
    assert!(!session.document_by_id(first).unwrap().editor.can_undo());
}

#[test]
fn a_panel_dropped_in_the_middle_of_the_documents_area_docks_beside_it() {
    let mut fixture = Fixture::new("panel_on_documents");
    let document = fixture.new_document();
    let middle = fixture.workspace().documents_rect().unwrap().center();
    let from = fixture.panel_tab(Panel::History);

    fixture.drag(from, middle);

    assert!(!fixture.workspace().documents_share_a_tab_group());
    let documents = fixture.workspace().documents_rect().unwrap();
    let history = fixture.panel_rect(Panel::History);
    assert!(
        !documents.intersects(history.shrink(1.0)),
        "{documents:?} and {history:?} overlap"
    );
    assert_eq!(fixture.workspace().area().groups(), [vec![document]]);
    assert_eq!(fixture.workspace().area().visible_documents(), [document]);
}

/// Where to drop a panel on the Documents area, and whether the panel then is on that side of it.
type DockCase = (fn(Rect) -> Pos2, fn(Rect, Rect) -> bool);

#[test]
fn a_panel_can_be_docked_on_every_side_of_the_documents_area() {
    let cases: [DockCase; 4] = [
        (
            |r| Pos2::new(r.left() + 4.0, r.center().y),
            |d, p| p.right() <= d.left(),
        ),
        (
            |r| Pos2::new(r.right() - 4.0, r.center().y),
            |d, p| p.left() >= d.right(),
        ),
        (
            |r| Pos2::new(r.center().x, r.top() + 4.0),
            |d, p| p.bottom() <= d.top(),
        ),
        (
            |r| Pos2::new(r.center().x, r.bottom() - 4.0),
            |d, p| p.top() >= d.bottom(),
        ),
    ];
    for (index, (target, is_on_that_side)) in cases.into_iter().enumerate() {
        let mut fixture = Fixture::new(&format!("dock_{index}"));
        let documents = fixture.workspace().documents_rect().unwrap();
        let from = fixture.panel_tab(Panel::History);

        fixture.drag(from, target(documents));

        let documents = fixture.workspace().documents_rect().unwrap();
        let history = fixture.panel_rect(Panel::History);
        assert!(
            is_on_that_side(documents, history),
            "case {index}: {documents:?} and {history:?}"
        );
        assert!(!fixture.workspace().documents_share_a_tab_group());
    }
}

#[test]
fn panels_can_be_grouped_as_tabs() {
    let mut fixture = Fixture::new("panels_group");
    let from = fixture.panel_tab(Panel::History);
    let onto = fixture.panel_rect(Panel::Areas).center();

    fixture.drag(from, onto);

    // Grouped, only one of the two is laid out, and each has the column's full height.
    assert!(fixture.workspace().is_open(Panel::History));
    assert!(fixture.workspace().is_open(Panel::Areas));
    let open_rects = [Panel::History, Panel::Areas]
        .map(|panel| fixture.workspace().panel_rect(panel))
        .iter()
        .filter(|rect| rect.is_some())
        .count();
    assert_eq!(open_rects, 1);
}

#[test]
fn the_documents_area_cannot_be_dragged() {
    let mut fixture = Fixture::new("documents_fixed");
    let before = fixture.workspace().documents_rect().unwrap();
    // Where its tab would be, if it had one, and its own surface.
    let target = fixture.panel_rect(Panel::Objects).center();

    fixture.drag(before.center_top() + Vec2::new(0.0, 4.0), target);

    assert_eq!(fixture.workspace().documents_rect().unwrap(), before);
}

#[test]
fn a_panel_dropped_on_a_document_stays_out_of_the_documents_area() {
    let mut fixture = Fixture::new("panel_on_document");
    let document = fixture.new_document();
    let onto = fixture.canvas_rect(document).center();
    let from = fixture.panel_tab(Panel::Areas);

    fixture.drag(from, onto);

    assert_eq!(fixture.workspace().area().groups(), [vec![document]]);
    assert!(fixture.workspace().is_open(Panel::Areas));
}

#[test]
fn a_document_tab_dropped_on_a_panel_stays_where_it_was() {
    let mut fixture = Fixture::new("tab_on_panel");
    let (first, second) = fixture.two_documents_side_by_side();
    let (first_rect, second_rect) = (fixture.canvas_rect(first), fixture.canvas_rect(second));
    let from = fixture.tab_of_canvas(first_rect);
    let onto = fixture.panel_rect(Panel::Objects).center();

    fixture.drag(from, onto);

    assert_eq!(fixture.canvas_rect(first), first_rect);
    assert_eq!(fixture.canvas_rect(second), second_rect);
    assert!(fixture.workspace().is_open(Panel::Objects));
    assert_eq!(fixture.workspace().area().groups().len(), 2);
}

#[test]
fn a_document_tab_can_be_regrouped_and_split_inside_the_area() {
    let mut fixture = Fixture::new("tab_regroup");
    let (first, second) = fixture.two_documents_side_by_side();

    // Onto the other document's middle: grouped together.
    let from = fixture.tab_of_canvas(fixture.canvas_rect(first));
    let onto = fixture.canvas_rect(second).center();
    fixture.drag(from, onto);
    let groups = fixture.workspace().area().groups();
    assert_eq!(groups.len(), 1, "{groups:?}");
    assert_eq!(groups[0].len(), 2, "{groups:?}");

    // Onto the bottom of the now only group: split vertically.
    let visible = fixture.workspace().area().visible_documents()[0];
    let canvas = fixture.canvas_rect(visible);
    let from = fixture.tab_of_canvas(canvas);
    fixture.drag(from, canvas.center_bottom() - Vec2::new(0.0, 4.0));
    assert_eq!(fixture.workspace().area().groups().len(), 2);
    assert_eq!(fixture.workspace().area().visible_documents().len(), 2);
    let [a, b] = [first, second].map(|id| fixture.canvas_rect(id));
    assert!(
        a.bottom() <= b.top() || b.bottom() <= a.top(),
        "{a:?} {b:?}"
    );
}

#[test]
fn panels_can_be_closed_reopened_and_the_layout_reset() {
    let mut workspace = Workspace::default();
    assert!(Panel::ALL.iter().all(|panel| workspace.is_open(*panel)));

    workspace.toggle(Panel::Areas);
    assert!(!workspace.is_open(Panel::Areas));

    workspace.toggle(Panel::Areas);
    assert!(workspace.is_open(Panel::Areas));

    workspace.toggle(Panel::History);
    workspace.reset_layout();
    assert!(Panel::ALL.iter().all(|panel| workspace.is_open(*panel)));
}

#[test]
fn a_reopened_panel_gets_a_real_share_of_the_window() {
    let mut fixture = Fixture::new("reopen");
    for panel in [Panel::Objects, Panel::History] {
        fixture.app.workspace.toggle(panel);
        fixture.settle();
        assert!(fixture.workspace().panel_rect(panel).is_none());

        fixture.app.workspace.toggle(panel);
        fixture.settle();

        let rect = fixture.panel_rect(panel);
        assert!(rect.width() > WINDOW.width() * 0.1, "{panel:?}: {rect:?}");
    }
    assert!(!fixture.workspace().documents_share_a_tab_group());
}

#[test]
fn closing_a_panels_tab_closes_it_but_the_documents_area_has_no_close_button() {
    let mut fixture = Fixture::new("close_panel");
    let panel = fixture.panel_rect(Panel::History);
    let bar_right = panel.left() + TAB_IN * 4.0;
    let mut closed = false;
    // The close button is at the right of the tab; search along the tab bar for it.
    let mut x = panel.left();
    while x < bar_right && !closed {
        fixture.click(Pos2::new(x, panel.top() - TAB_ABOVE));
        closed = !fixture.workspace().is_open(Panel::History);
        x += 4.0;
    }

    assert!(closed);
    assert!(fixture.workspace().documents_rect().is_some());
}

#[test]
fn the_layout_is_restored_but_no_documents_are() {
    let mut fixture = Fixture::new("restore");
    fixture.new_document();
    fixture.app.workspace.toggle(Panel::Areas);
    let onto = fixture
        .workspace()
        .documents_rect()
        .unwrap()
        .center_bottom();
    let from = fixture.panel_tab(Panel::History);
    fixture.drag(from, onto - Vec2::new(0.0, 4.0));

    let layout = fixture.workspace().save_layout().unwrap();
    let restored = Workspace::from_saved_layout(Some(&layout));

    assert!(!restored.is_open(Panel::Areas));
    assert!(restored.is_open(Panel::History));
    assert!(restored.area().groups().is_empty());
    assert!(restored.has_same_layout_as(fixture.workspace()));
}

#[test]
fn a_layout_that_cannot_be_used_falls_back_to_the_default() {
    for layout in [None, Some("nonsense"), Some("(root: None)")] {
        assert!(
            Workspace::from_saved_layout(layout).has_same_layout_as(&Workspace::default()),
            "{layout:?}"
        );
    }
}

/// The cursor with the pointer resting at `pos`.
fn cursor_over(fixture: &mut Fixture, pos: Pos2) -> egui::CursorIcon {
    fixture.frame(vec![Event::PointerMoved(pos)]);
    fixture.settle();
    fixture.cursor
}

/// The cursor part-way through dragging from `from` to `over`, which then lets go at the start.
fn cursor_while_dragging(fixture: &mut Fixture, from: Pos2, over: Pos2) -> egui::CursorIcon {
    fixture.frame(vec![Event::PointerMoved(from)]);
    fixture.press(from, true);
    for step in 1..=4 {
        let along = step as f32 / 4.0;
        fixture.frame(vec![Event::PointerMoved(from + (over - from) * along)]);
    }
    let cursor = fixture.cursor;
    fixture.frame(vec![Event::PointerMoved(from)]);
    fixture.press(from, false);
    fixture.settle();
    cursor
}

#[test]
fn tabs_and_tab_bars_show_the_normal_cursor_until_they_are_dragged() {
    let mut fixture = Fixture::new("tab_cursor");
    let (first, _second) = fixture.two_documents_side_by_side();
    let document = fixture.canvas_rect(first);
    let panel = fixture.panel_rect(Panel::History);
    let document_tab = fixture.tab_of_canvas(document);
    let panel_tab = fixture.panel_tab(Panel::History);
    let bar_of = |rect: Rect| Pos2::new(rect.center().x, rect.top() - TAB_ABOVE);

    for pos in [document_tab, bar_of(document), panel_tab, bar_of(panel)] {
        assert_eq!(
            cursor_over(&mut fixture, pos),
            egui::CursorIcon::Default,
            "at {pos:?}"
        );
    }

    for from in [document_tab, bar_of(document)] {
        let over = document.center();
        assert_eq!(
            cursor_while_dragging(&mut fixture, from, over),
            egui::CursorIcon::Grabbing,
            "from {from:?}"
        );
    }
    for from in [panel_tab, bar_of(panel)] {
        let over = panel.center();
        assert_eq!(
            cursor_while_dragging(&mut fixture, from, over),
            egui::CursorIcon::Grabbing,
            "from {from:?}"
        );
    }
}

#[test]
fn the_tool_bar_reports_the_open_panels() {
    let mut fixture = Fixture::new("toolbar_panels");
    fixture.app.workspace.toggle(Panel::Areas);

    let state = fixture.app.tool_bar_state();

    for panel in Panel::ALL {
        assert_eq!(state.open_panels[panel.index()], panel != Panel::Areas);
    }
}
