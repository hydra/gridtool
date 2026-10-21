use std::cell::RefCell;
use std::collections::VecDeque;
use std::fs;
use std::rc::Rc;

use gridtool::action::Action;
use gridtool::geometry::Point;

use super::*;

/// Scripted answers for the file dialogs. Asking a dialog that has no scripted answer
/// fails the test, so tests also prove which dialogs were (not) shown.
#[derive(Default)]
struct ScriptState {
    images: VecDeque<Option<PathBuf>>,
    documents: VecDeque<Option<PathBuf>>,
    saves: VecDeque<Option<PathBuf>>,
    suggestions: Vec<PathBuf>,
}

#[derive(Default, Clone)]
struct Script(Rc<RefCell<ScriptState>>);

impl Script {
    fn image(&self, path: Option<PathBuf>) {
        self.0.borrow_mut().images.push_back(path);
    }

    fn document(&self, path: Option<PathBuf>) {
        self.0.borrow_mut().documents.push_back(path);
    }

    fn save(&self, path: Option<PathBuf>) {
        self.0.borrow_mut().saves.push_back(path);
    }

    fn suggestions(&self) -> Vec<PathBuf> {
        self.0.borrow().suggestions.clone()
    }

    fn assert_all_dialogs_used(&self) {
        let state = self.0.borrow();
        assert!(state.images.is_empty() && state.documents.is_empty() && state.saves.is_empty());
    }
}

impl ImageFilePicker for Script {
    fn pick_image(&self) -> Option<PathBuf> {
        self.0
            .borrow_mut()
            .images
            .pop_front()
            .expect("unexpected image dialog")
    }
}

impl DocumentFilePicker for Script {
    fn pick_document(&self) -> Option<PathBuf> {
        self.0
            .borrow_mut()
            .documents
            .pop_front()
            .expect("unexpected load dialog")
    }

    fn pick_save_path(&self, suggestion: &Path) -> Option<PathBuf> {
        let mut state = self.0.borrow_mut();
        state.suggestions.push(suggestion.to_path_buf());
        state.saves.pop_front().expect("unexpected save dialog")
    }
}

/// A temporary folder holding an 8×6 `photo.png`, removed when dropped.
struct Fixture {
    folder: PathBuf,
    ctx: egui::Context,
    script: Script,
    session: Session,
}

impl Fixture {
    fn new(name: &str) -> Self {
        let folder =
            std::env::temp_dir().join(format!("gridtool_session_{name}_{}", std::process::id()));
        fs::create_dir_all(&folder).unwrap();
        image::RgbaImage::from_pixel(8, 6, image::Rgba([9, 9, 9, 255]))
            .save(folder.join("photo.png"))
            .unwrap();
        let script = Script::default();
        let session = Session::new(Box::new(script.clone()), Box::new(script.clone()));
        Self {
            folder,
            ctx: egui::Context::default(),
            script,
            session,
        }
    }

    fn path(&self, name: &str) -> PathBuf {
        self.folder.join(name)
    }

    fn open_image(&mut self) {
        self.script.image(Some(self.path("photo.png")));
        self.session.replace(&self.ctx, Replacement::OpenImage);
    }

    fn edit(&mut self, x: f64) {
        self.open().perform(Action::SetOrigin(Point::new(x, 1.0)));
    }

    fn open(&mut self) -> &mut OpenDocument {
        self.session.document_mut().expect("a document is open")
    }

    fn file(&self) -> Option<PathBuf> {
        self.session.document()?.file().map(Path::to_path_buf)
    }

    fn origin(&self) -> Point {
        self.session.document().unwrap().editor.document().origin()
    }

    fn answer(&mut self, button: DialogButton) {
        assert!(self.session.dialog().is_some(), "no dialog to answer");
        self.session.answer(&self.ctx, button);
    }

    fn is_empty(&self) -> bool {
        matches!(self.session.content, Content::Empty)
    }

    /// Saves to `name` via Save, answering the save dialog.
    fn save_as_new(&mut self, name: &str) {
        self.script.save(Some(self.path(name)));
        self.session.save(&self.ctx);
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.folder).ok();
    }
}

#[test]
fn new_with_nothing_open_asks_nothing() {
    let mut fixture = Fixture::new("new_empty");

    fixture.session.replace(&fixture.ctx, Replacement::New);

    assert!(fixture.is_empty());
    assert_eq!(fixture.session.dialog(), None);
}

#[test]
fn a_never_saved_document_has_unsaved_changes() {
    let mut fixture = Fixture::new("never_saved");
    fixture.open_image();

    assert!(fixture.open().has_unsaved_changes());
}

#[test]
fn new_with_unsaved_changes_asks_and_cancel_keeps_everything() {
    let mut fixture = Fixture::new("new_cancel");
    fixture.open_image();
    fixture.edit(3.0);

    fixture.session.replace(&fixture.ctx, Replacement::New);
    let dialog = fixture.session.dialog().unwrap();
    assert_eq!(dialog, Dialog::unsaved_changes());
    assert_eq!(dialog.title, "Unsaved changes");
    assert_eq!(dialog.message, "Do you want to save now?");
    assert_eq!(
        dialog.buttons,
        [DialogButton::Yes, DialogButton::No, DialogButton::Cancel]
    );
    assert_eq!(dialog.default, DialogButton::Cancel);

    fixture.answer(DialogButton::Cancel);

    assert_eq!(fixture.session.dialog(), None);
    assert_eq!(fixture.origin(), Point::new(3.0, 1.0));
    assert!(fixture.open().editor.can_undo(), "history is kept");
}

#[test]
fn new_then_no_discards_without_saving() {
    let mut fixture = Fixture::new("new_no");
    fixture.open_image();

    fixture.session.replace(&fixture.ctx, Replacement::New);
    fixture.answer(DialogButton::No);

    assert!(fixture.is_empty());
    fixture.script.assert_all_dialogs_used();
}

#[test]
fn new_then_yes_saves_then_resets_and_forgets_the_file() {
    let mut fixture = Fixture::new("new_yes");
    fixture.open_image();
    fixture.edit(3.0);
    fixture.script.save(Some(fixture.path("plan.ron")));

    fixture.session.replace(&fixture.ctx, Replacement::New);
    fixture.answer(DialogButton::Yes);

    assert!(fixture.is_empty());
    let saved = document_file::load(&fixture.path("plan.ron")).unwrap();
    assert_eq!(saved.document.origin(), Point::new(3.0, 1.0));

    // The file is forgotten: the next document asks where to save again.
    fixture.open_image();
    fixture.save_as_new("other.ron");
    assert!(fixture.path("other.ron").exists());
}

#[test]
fn new_then_yes_but_cancelling_the_save_dialog_keeps_the_document() {
    let mut fixture = Fixture::new("new_yes_cancel");
    fixture.open_image();
    fixture.script.save(None);

    fixture.session.replace(&fixture.ctx, Replacement::New);
    fixture.answer(DialogButton::Yes);

    assert!(!fixture.is_empty());
    assert!(fixture.open().has_unsaved_changes());
    assert_eq!(fixture.session.dialog(), None);
}

#[test]
fn save_asks_for_a_file_once_then_reuses_it() {
    let mut fixture = Fixture::new("save_reuse");
    fixture.open_image();

    // The extension is added when missing.
    fixture.save_as_new("plan");
    assert!(!fixture.open().has_unsaved_changes());
    assert_eq!(fixture.file(), Some(fixture.path("plan.ron")));

    fixture.edit(5.0);
    assert!(fixture.open().has_unsaved_changes());
    fixture.session.save(&fixture.ctx); // No dialog scripted: it must not ask.

    assert!(!fixture.open().has_unsaved_changes());
    let saved = document_file::load(&fixture.path("plan.ron")).unwrap();
    assert_eq!(saved.document.origin(), Point::new(5.0, 1.0));
    fixture.script.assert_all_dialogs_used();
}

#[test]
fn the_save_dialog_suggests_the_image_name_then_the_current_file() {
    let mut fixture = Fixture::new("suggestions");
    fixture.open_image();
    fixture.save_as_new("plan.ron");

    fixture.script.save(None);
    fixture.session.save_as(&fixture.ctx);

    assert_eq!(
        fixture.script.suggestions(),
        [fixture.path("photo.ron"), fixture.path("plan.ron")]
    );
}

#[test]
fn undoing_back_to_the_saved_state_has_no_unsaved_changes() {
    let mut fixture = Fixture::new("undo_saved");
    fixture.open_image();
    fixture.edit(2.0);
    fixture.save_as_new("plan.ron");

    fixture.edit(4.0);
    assert!(fixture.open().has_unsaved_changes());
    fixture.open().undo();
    assert!(!fixture.open().has_unsaved_changes());
    fixture.open().undo();
    assert!(fixture.open().has_unsaved_changes());
}

#[test]
fn loading_restores_the_document_without_history_or_unsaved_changes() {
    let mut fixture = Fixture::new("load");
    fixture.open_image();
    fixture.edit(6.0);
    fixture.save_as_new("plan.ron");
    let saved = fixture.open().editor.document().clone();
    fixture.session.replace(&fixture.ctx, Replacement::New);

    fixture.script.document(Some(fixture.path("plan.ron")));
    fixture.session.replace(&fixture.ctx, Replacement::Load);

    let open = fixture.open();
    assert_eq!(open.editor.document(), &saved);
    assert!(!open.has_unsaved_changes());
    assert!(!open.editor.can_undo());
    assert_eq!(fixture.file(), Some(fixture.path("plan.ron")));
}

#[test]
fn load_and_open_image_also_ask_about_unsaved_changes() {
    for replacement in [Replacement::Load, Replacement::OpenImage] {
        let mut fixture = Fixture::new("other_replacements");
        fixture.open_image();

        fixture.session.replace(&fixture.ctx, replacement);

        assert_eq!(fixture.session.dialog(), Some(Dialog::unsaved_changes()));
        fixture.answer(DialogButton::Cancel);
        assert!(!fixture.is_empty());
    }
}

#[test]
fn a_failed_load_reports_the_error_and_keeps_the_open_document() {
    let mut fixture = Fixture::new("load_error");
    fixture.open_image();
    fixture.save_as_new("plan.ron");
    fs::write(fixture.path("broken.ron"), "nonsense").unwrap();

    fixture.script.document(Some(fixture.path("broken.ron")));
    fixture.session.replace(&fixture.ctx, Replacement::Load);

    let dialog = fixture.session.dialog().unwrap();
    assert_eq!(dialog.title, "Could not load document");
    assert_eq!(dialog.buttons, [DialogButton::Ok]);
    fixture.answer(DialogButton::Ok);
    assert_eq!(fixture.session.dialog(), None);
    assert_eq!(fixture.file(), Some(fixture.path("plan.ron")));
}

#[test]
fn save_as_asks_before_replacing_a_file_found_by_adding_the_extension() {
    let mut fixture = Fixture::new("overwrite");
    fs::write(fixture.path("plan.ron"), "keep me").unwrap();
    fixture.open_image();

    fixture.script.save(Some(fixture.path("plan")));
    fixture.session.save_as(&fixture.ctx);
    assert_eq!(
        fixture.session.dialog(),
        Some(Dialog::overwrite("plan.ron"))
    );
    fixture.answer(DialogButton::No);
    assert_eq!(
        fs::read_to_string(fixture.path("plan.ron")).unwrap(),
        "keep me"
    );
    assert!(fixture.open().has_unsaved_changes());

    fixture.script.save(Some(fixture.path("plan")));
    fixture.session.save_as(&fixture.ctx);
    fixture.answer(DialogButton::Yes);
    assert!(document_file::load(&fixture.path("plan.ron")).is_ok());
    assert!(!fixture.open().has_unsaved_changes());
}

#[test]
fn save_as_trusts_the_native_overwrite_confirmation() {
    let mut fixture = Fixture::new("native_overwrite");
    fs::write(fixture.path("plan.ron"), "old").unwrap();
    fixture.open_image();

    // The dialog returned the existing file itself, so it already asked.
    fixture.script.save(Some(fixture.path("plan.ron")));
    fixture.session.save_as(&fixture.ctx);

    assert_eq!(fixture.session.dialog(), None);
    assert!(document_file::load(&fixture.path("plan.ron")).is_ok());
}

#[test]
fn a_failed_save_reports_the_error() {
    let mut fixture = Fixture::new("save_error");
    fixture.open_image();

    fixture.save_as_new("missing_folder/plan.ron");

    let dialog = fixture.session.dialog().unwrap();
    assert_eq!(dialog.title, "Could not save document");
    assert!(fixture.open().has_unsaved_changes());
}

#[test]
fn closing_with_nothing_unsaved_closes_straight_away() {
    let mut fixture = Fixture::new("close_empty");
    assert!(fixture.session.request_close(&fixture.ctx));

    let mut fixture = Fixture::new("close_saved");
    fixture.open_image();
    fixture.save_as_new("plan.ron");
    assert!(fixture.session.request_close(&fixture.ctx));
    assert_eq!(fixture.session.dialog(), None);
}

#[test]
fn closing_with_unsaved_changes_asks_and_cancel_keeps_the_window_open() {
    let mut fixture = Fixture::new("close_cancel");
    fixture.open_image();

    assert!(!fixture.session.request_close(&fixture.ctx));
    assert_eq!(fixture.session.dialog(), Some(Dialog::unsaved_changes()));
    // Asking again while the dialog is open keeps the one dialog.
    assert!(!fixture.session.request_close(&fixture.ctx));
    fixture.answer(DialogButton::Cancel);

    assert_eq!(fixture.session.dialog(), None);
    assert!(fixture.open().has_unsaved_changes());
    assert!(!fixture.session.exiting);
}

#[test]
fn closing_then_no_closes_without_saving() {
    let mut fixture = Fixture::new("close_no");
    fixture.open_image();

    assert!(!fixture.session.request_close(&fixture.ctx));
    fixture.answer(DialogButton::No);

    assert!(fixture.session.exiting);
    // The close the session then asks for is let through.
    assert!(fixture.session.request_close(&fixture.ctx));
    fixture.script.assert_all_dialogs_used();
}

#[test]
fn closing_then_yes_saves_then_closes() {
    let mut fixture = Fixture::new("close_yes");
    fixture.open_image();
    fixture.edit(3.0);
    fixture.script.save(Some(fixture.path("plan.ron")));

    assert!(!fixture.session.request_close(&fixture.ctx));
    fixture.answer(DialogButton::Yes);

    assert!(fixture.session.exiting);
    let saved = document_file::load(&fixture.path("plan.ron")).unwrap();
    assert_eq!(saved.document.origin(), Point::new(3.0, 1.0));
}

#[test]
fn closing_then_yes_but_cancelling_the_save_dialog_keeps_the_window_open() {
    let mut fixture = Fixture::new("close_yes_cancel");
    fixture.open_image();
    fixture.script.save(None);

    assert!(!fixture.session.request_close(&fixture.ctx));
    fixture.answer(DialogButton::Yes);

    assert!(!fixture.session.exiting);
    assert!(fixture.open().has_unsaved_changes());
}

#[test]
fn the_window_title_names_the_document_file() {
    let mut fixture = Fixture::new("title");
    assert_eq!(fixture.session.window_title(), "GridTool");

    fixture.open_image();
    assert_eq!(
        fixture.session.window_title(),
        "unsaved document - GridTool"
    );

    fixture.save_as_new("plan");
    assert_eq!(fixture.session.window_title(), "plan.ron - GridTool");

    fixture.session.replace(&fixture.ctx, Replacement::New);
    assert_eq!(fixture.session.window_title(), "GridTool");
}
