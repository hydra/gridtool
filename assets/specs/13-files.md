# 13 — Document Files: New, Load, Save, Save As

A document can be saved to a file and loaded again. The file describes the document
completely, so loading it re-creates the exact document state (`DOC-1`).

## What is saved

**FILE-1 — Contents.** A document file holds:
- the image reference (`FILE-3`) and the image size in pixels
- the origin (`ORIGIN-1`)
- the calibration (`SCALE-20`): the point set and method, both distances and the unit
- every measurement (`MEAS-1`), in list order, with its object id, endpoints and unit
- every grid (`GRID-1`), in list order, with its object id, alignment source (a scale
  line, or a measurement by object id, which may refer to a deleted measurement), shape,
  sizes A and B (both, even for square cells), unit, colour and opacity
- the object id counter (`OBJ-1`), so ids are never reused after loading

**FILE-2 — Not saved.** These are **not** saved:
- the history (`HIST`). A loaded document starts with an empty history.
- view state (`VIEW`): pan, zoom and rotation. A loaded document starts with the default
  view (`VIEW-2`).
- presentation state: the tool mode, the active object and tool progress.

**FILE-3 — The image is referenced, not embedded.** The file stores the image's path:
- **Relative** when the image is in the document file's folder or below it, written with
  `/` separators (for example `images/photo.png`), so a folder holding both can be moved.
- **Absolute** otherwise.

When loading, a relative reference is resolved against the document file's folder.

## File format

**FILE-4 — Format.** Files use the RON text format, with the extension `.ron`. A file is
written in full to a temporary file next to the target and then moved into place, so a
failed save never leaves a half-written document.

**FILE-5 — Versioned description, separate from runtime state.** The file holds a
*description* of the document in a versioned format. It is not a dump of the runtime
data structures:
- **Tagged.** The top level names its version, for example `V1(( … ))`.
- **Converted on save.** Saving converts the runtime document to the current version's
  description.
- **Converted on load.** Loading converts a description of **any** supported version back
  to the runtime document.
- **Never edited.** A released version's description never changes. Format changes add a
  new version, and the loader keeps accepting every earlier version. The automated tests
  MUST include a literal sample of each released version that must still load.

**FILE-6 — Version 1.** Version 1 is a record with these fields:
- `image`: string (`FILE-3`)
- `width`, `height`: whole numbers
- `origin`: `(x, y)`
- `calibration`:
  - `points`: an optional `(method: ThreePoint|FourPoint, points: [(x, y), …])`
  - `distance_a`, `distance_b`: optional numbers
  - `unit`: `Centimetre|Metre`
- `measurements`: a list of `(id, start, end, unit)`
- `grids`: a list of `(id, source, shape, size_a, size_b, unit, color: (r, g, b), opacity)`
  - `source` is `ScaleLineA`, `ScaleLineB`, or `Measurement(<id>)`
  - `shape` is `Square|Rectangle`
- `next_object_id`: a whole number

**FILE-7 — Validation on load.** A loaded description is checked as strictly as actions
check their inputs (`HIST-2`), so a hand-edited or damaged file cannot produce an invalid
document. Loading fails when:
- the size is zero
- a point (scale point, measurement endpoint or origin) is outside the document
- the number of scale points does not match the method
- a distance or cell size is not finite and > 0
- an opacity is outside [0, 1]
- an object id is 0, used twice, or not below `next_object_id`

**FILE-8 — Loading the image.** After the description is valid, the referenced image is
decoded (`IMG-5`). Loading fails if the image cannot be read or decoded, or if its size
differs from the stored size.

## Unsaved changes

**FILE-9 — Tracking.** An open document **has unsaved changes** when it differs from the
document as it was last loaded or saved:
- The comparison is by value. Undoing back to the saved state therefore means no unsaved
  changes.
- A document that has never been saved (one started from an image) always has unsaved
  changes.
- Opening a document by loading, or saving it, sets the reference point.

## Tool bar

**FILE-10 — File buttons.** The main tool bar (`SHELL-5`) starts with the file buttons, in
this order, followed by a 16 px spacer:

| Button | Icon meaning (current Material Symbols icon) | Tooltip | Enabled |
|---|---|---|---|
| New | new document (`note_add`) | "`New`" | always |
| Load | open folder (`folder_open`) | "`Load…`" | always |
| Save | save (`save`) | "`Save`" | only when a document is open **and** it has unsaved changes (`FILE-9`) |
| Save as | save as (`save_as`) | "`Save as…`" | when a document is open |

## Commands

**FILE-11 — Unsaved changes dialog.** New, Load and Open image replace the open document.
If it has unsaved changes, they first show a modal dialog:
- **Title:** "`Unsaved changes`"
- **Question:** "`Do you want to save now?`"
- **Buttons:** "`Yes`", "`No`", "`Cancel`", left to right.
- **Default:** **Cancel** has keyboard focus when the dialog opens, so Enter picks it.
  Escape, or clicking outside the dialog, also means Cancel.

The answers:
- **Yes:** save as **Save** would (`FILE-14`), which may ask for a file name. If the save
  completes, carry out the command. If it is cancelled or fails, stop and keep the
  document as it was.
- **No:** carry out the command without saving.
- **Cancel:** close the dialog. Nothing else changes, including the document, its history
  and its file.

With no unsaved changes, or no document open, the command is carried out straight away.

**FILE-12 — New.** Closes the document and forgets its file name. The application returns
to its start-up state with no document open (`SHELL-3`).

**FILE-13 — Load.** Shows the native open dialog, titled "`Load document`", with one
filter, "`GridTool document`" (`*.ron`).
- **Cancelled:** nothing changes.
- **Success:** the loaded document replaces the open one, with an empty history and the
  default view. Its file becomes the document's file, and it has no unsaved changes.
- **Failure** (`FILE-7`, `FILE-8`, or an unreadable file): an error dialog titled
  "`Could not load document`" shows "`<path>: <reason>`" with one "`OK`" button. The open
  document, if any, is kept.

**FILE-14 — Save.** If the document has a file (it was loaded or saved before), write to
it without asking. Otherwise behave as **Save as**.

**FILE-15 — Save as.** Always asks for a file name:
- **Dialog.** The native save dialog, titled "`Save document as`", with the
  "`GridTool document`" (`*.ron`) filter. It starts with the document's file, or for a
  never-saved document a file named after the image (`photo.png` → `photo.ron`) in the
  image's folder.
- **Cancelled:** nothing is saved, and any command waiting on the save (`FILE-11`) is
  abandoned.
- **Extension.** If the chosen name does not end in `.ron` (any letter case), `.ron` is
  appended.
- **Overwriting.** The native dialog asks before returning an existing file. If `.ron`
  was appended and the resulting file exists, the application asks itself with a modal
  dialog:
  - title "`File already exists`"
  - message "`<name> already exists. Do you want to replace it?`"
  - buttons "`Yes`" and "`No`", with **No** the default

  No abandons the save.
- **Success:** the file becomes the document's file, and the document has no unsaved
  changes.
- **Failure:** an error dialog titled "`Could not save document`" shows
  "`<path>: <reason>`". The document keeps its unsaved changes.

## Architecture

**FILE-16 — Headless.** Converting, saving, loading and validating are part of the core
and do not depend on the UI (`ARCH-3`), so a command-line front end can load and save
documents. The UI only supplies file names (through an interface that tests can replace)
and shows the dialogs.

## Acceptance criteria

- **Round trip.** A document using every feature, saved and loaded, is exactly equal to
  the original. That means:
  - four-point calibration, both distances, metre unit
  - measurements with a gap in their ids, with mixed units
  - a rectangle grid with every property changed
  - a grid aligned to a deleted measurement
  - a moved origin

  Duplicating a grid then gives the same next id in both.
- The written text starts with `V1(`.
- A literal version 1 sample loads with the expected values.
- Each `FILE-7` failure is rejected with its reason.
- Image references:
  - `a.png` beside the document is stored as `a.png`
  - an image in `images/` below it is stored as `images/b.png`
  - an image elsewhere is stored as an absolute path
  - each resolves back to the original path
- `plan` gets `.ron`, `plan.RON` is unchanged, and `plan.txt` becomes `plan.txt.ron`.
- Saving over an existing file replaces it and leaves no temporary file behind.
- Loading reports read, parse, missing-image and image-size-mismatch errors.
- Session flows, with scripted file dialogs and real files:
  - **New with nothing open:** asks nothing.
  - **Freshly opened image:** has unsaved changes.
  - **New, then Cancel:** keeps the document and its history. The dialog's title,
    question, buttons and default are as `FILE-11` says.
  - **New, then No:** resets without showing any file dialog.
  - **New, then Yes:** saves, then resets. A following document asks for a new file name.
  - **New, then Yes, then cancelling the save dialog:** keeps the document.
  - **Save:**
    - It asks once, adding `.ron`.
    - A later Save writes to the same file without asking.
  - **Save dialog suggestion:** first `photo.ron` beside the image, then the current file.
  - **Undo back to the saved state:** no unsaved changes. One more undo brings them back.
  - **Load:** restores the saved document, with no history, no unsaved changes, and the
    loaded file as its file.
  - **Load and Open image:** also ask about unsaved changes.
  - **Failed load:** shows the error dialog and keeps the open document.
  - **Save as to a name whose `.ron` file exists:**
    - It asks about replacing.
    - No keeps the old file.
    - Yes replaces it.
  - **Save as to an existing `.ron` file returned by the dialog:** does not ask again.
  - **Failed save** (missing folder): shows the error dialog and keeps unsaved changes.
- Dialog, simulated with key events:
  - Enter on a freshly shown Unsaved changes dialog answers Cancel.
  - Escape on the overwrite dialog answers No.
