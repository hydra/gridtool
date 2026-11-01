# 02 — Application Shell

## Window

**SHELL-1.** The application opens one native window with an initial inner size of
1280 × 800. Its title follows the usual Windows form "`<document> - GridTool`", naming the active document (`WORK-10`):
- With no document open, the title is "`GridTool`".
- With a document that has a file (`FILE-14`), `<document>` is the file's name with its
  extension and without its folder, for example "`plan.ron - GridTool`".
- With a document that has never been saved, `<document>` is "`unsaved document`".
- The title updates as soon as a document is opened, saved under a new name or closed.

Closing the window may first ask about unsaved changes (`FILE-17`).

**SHELL-1a — Application icon.** The application has one icon, a square image, used
wherever the platform shows one for it: the window, the task bar or dock, the executable
file, the release packages, and the installer program of a package that has one. All of them come
from the same image.
- A test MUST check that the bundled icon can be loaded and is square.

**SHELL-1b — No console window.** On Windows, a release build opens only the application window,
with no console window beside it. A development build MAY keep one for diagnostics.

**SHELL-1c — Crash reports.** When a release build panics, it saves a crash report file to the
system's temporary folder, then shows a modal error dialog before closing:
- **Title:** "`GridTool crashed`"
- **Message:** that it had a problem and has to close and that unsaved changes are lost, the full
  path of the report, where to report the problem, and that nothing is sent automatically. If the
  report could not be saved, it says so instead.
- **Report first.** The report is saved before the dialog is shown, so it is kept even if showing
  the dialog fails.
- **Development.** A development build, or a build run with the `RUST_BACKTRACE` environment
  variable set, shows the usual panic output instead.
- A test MUST check the message with and without a report.

## Layout

**SHELL-2.** The window is divided into these regions, from top to bottom:

1. **Main tool bar.** One horizontal row, always visible.
2. **Tool controls row.** A second row directly below the tool bar, always visible
   (`SHELL-2a`).
3. **Workspace.** The rest of the window: the Documents area, with the panels docked around
   it (`WORK`).

**SHELL-2a — Tool controls row.** The row under the tool bar is always shown, with or
without a document, whatever it contains, so the canvas does not move when a tool or an
object is chosen.
- **Height.** Its minimum height is the height of the tool bar's contents, so with one row
  of controls, or none, it is as tall as the tool bar. If the window is too narrow for the
  controls they wrap onto more rows, making it taller. It returns to its minimum when they
  fit again.
- **Contents.** They are those of the active object if there is one, otherwise those of the
  active tool:

  | Active object | Contents |
  |---|---|
  | Measurement | `MEAS-16` |
  | Area | `AREA-16` |
  | Grid | `GRID-14` |
  | Drawn shape | `DRAW-13` |

  | Active tool, with no active object | Contents |
  |---|---|
  | Rotate | `VIEW-12` |
  | Scale | `SCALE-3` |
  | Measure distance | `MEAS-17` |
  | Measure area | `AREA-17` |
  | Grid | `GRID-8` |
  | Draw line | `DRAW-17` |
  | Pan & zoom | Help text: "`Drag to pan the view. Scroll to zoom.`" |
  | Origin | Help text: "`Click on the image to move the origin, the point the view rotates about.`" |

  With no document open, it is the help text "`Open an image to get started`", whichever
  tool was active. Help text is in weak text, and wraps like controls.

**SHELL-3.** When no document is open, the Documents area shows the centred message
"`Open an image to get started.`"

**SHELL-4.** **Withdrawn.** A failed attempt to open an image is now reported in a dialog
(`IMG-6`).

## Main tool bar

**SHELL-5.** The tool bar contains these items, left to right:

| # | Item | Icon meaning (current Material Symbols icon) | Tooltip |
|---|---|---|---|
| 1 | New button (`FILE-10`) | new document (`note_add`) | "`New…`" |
| 2 | Load button | open folder (`folder_open`) | "`Load…`" |
| 3 | Save button | save (`save`) | "`Save`" |
| 4 | Save as button | save as (`save_as`) | "`Save as…`" |
| 5 | Horizontal spacer, 16 px | — | — |
| 6 | Pan & zoom mode button | hand (`pan_tool`) | "`Pan & zoom`" |
| 7 | Rotate view mode button | rotate (`rotate_right`) | "`Rotate view`" |
| 8 | Scale mode button | ruler (`straighten`) | "`Scale`" |
| 9 | Measure distance mode button | tape measure (`measuring_tape`) | "`Measure distance`" |
| 10 | Measure area mode button (`AREA-1`) | polygon (`pentagon`) | "`Measure area`" |
| 11 | Grid mode button | grid (`grid_on`) | "`Grid`" |
| 12 | Origin mode button | crosshair (`my_location`) | "`Origin`" (`ORIGIN-2`) |
| 13 | Horizontal spacer, 16 px | — | — |
| 14 | Draw line mode button (`DRAW-1`) | line segments joined at points (`polyline`) | "`Draw line`" |
| 15 | Horizontal spacer, 16 px | — | — |
| 16 | Undo button | undo arrow (`undo`) | "`Undo`" |
| 17 | Redo button | redo arrow (`redo`) | "`Redo`" |
| 18 | Horizontal spacer, 16 px | — | — |
| 19 | Panels menu (`WORK-5`) | panels (`dashboard`) | "`Panels`" |

The buttons act on the active document (`WORK-10`). Items 14 onward up to the next spacer are the **drawing tools** group; future drawing tools
are added to it.

**SHELL-6 — Icon font.** Every icon in the UI is a **Google Material Symbols** icon
(<https://fonts.google.com/icons>), style **Rounded**, **filled**.
- **Bundled.** The icon font is bundled with the application (Apache License 2.0), so the
  icons look the same on every platform and never depend on system fonts.
- **Own family.** Icons are drawn in a font family that contains only this font, so an
  icon can never silently come from another font.
- **Sizes.** Tool bar icons are drawn at 18 px. Icons inside table rows are drawn at
  14 px.
- **Names.** The specs name each icon by its Material Symbols name, for example
  `measuring_tape`. An implementation refers to an icon by its code point in the font.

**SHELL-6a — Icon verification.** Every icon used anywhere in the UI MUST be defined in
one place.
- An automated test MUST check that each icon is exactly one character, that the icons
  are distinct, and that the bundled icon font contains a glyph for each. System fonts
  don't count, because they are not guaranteed to be present.

**SHELL-7.** Tooltips MUST appear on hover, including when the button is disabled.

## Tool modes

**SHELL-8.** Each open document has exactly one active tool mode of its own (`WORK-11`), and the
tool bar shows the active document's. With no document open there is none, as every tool works on a document's image. The modes are **Pan &
zoom**, **Rotate view**, **Scale**, **Measure distance**, **Measure area**, **Grid**,
**Origin** and **Draw line**. **Pan & zoom is the default**: every document starts with it.

**SHELL-9.** The mode buttons work like a radio group. The active mode's button is shown
as selected, and clicking a mode button makes that mode active for the active document. With no document open,
every mode button is disabled and none is shown as selected.

**SHELL-10.** New modes will be added later. The list of modes, and which of them start a
new group of buttons (`SHELL-5`), MUST be defined in one place, so that adding a mode needs
no duplicated tool bar code.

**SHELL-10a — Cancelling a tool operation.** Pressing **Escape**, or switching to another
tool mode, abandons whatever the tool is part-way through, without committing an action
or changing the document:
- the pending scale points while placing (`SCALE-4`), and any scale point drag
- the pending start point of a measurement (`MEAS-5`)
- the points placed so far for a new area (`AREA-2`) or drawn shape (`DRAW-2`)
- a drag moving the active object, or of its points or resize handles, in progress (`OBJ-9`,
  `OBJ-15`, `OBJ-16`); the object keeps its shape and place and stays active
- a rotate-handle drag in progress (`VIEW-13`); the view keeps the rotation it reached

Escape and switching mode cancel exactly the same things. Escape changes neither the tool
mode nor the active object; switching mode also clears the active object (`OBJ-7a`). Escape
does **not** cancel the tool when it is meant
for something else:
- while a dialog is open, Escape answers the dialog (`FILE-11`)
- while a widget such as a text field has keyboard focus, Escape only takes the focus
  away from it; a further Escape then cancels the tool

**SHELL-10b — Removing the last point.** Pressing **Backspace** removes the last point
placed for a new area (`AREA-3`) or drawn shape (`DRAW-3`). Like Escape, it does nothing
while a dialog is open or while a widget such as a text field has keyboard focus (where it
edits the text).

**SHELL-10c — Finishing a line.** Pressing **Enter** finishes the drawn line being placed,
leaving it open (`DRAW-5`). Like Escape, it does nothing while a dialog is open or while a
widget such as a text field has keyboard focus.

## Panels

**SHELL-11.** **Withdrawn.** The side panel is replaced by the panels of the workspace (`WORK`).

**SHELL-12 — Scroll bars.** Throughout the application, a scroll bar is drawn whenever there is
more to scroll to, without the pointer having to be over it or its area, and it takes up space
beside the content instead of covering it. Where everything fits there is no scroll bar, so its
presence shows whether something can be scrolled.

## Acceptance criteria

- Scroll bars (`SHELL-12`): they do not float over the content, in the dark or the light theme.

- The default tool mode is Pan & zoom.
- Every mode has a distinct icon and a distinct name.
- In the tool bar, Measure distance comes directly after Scale, Measure area directly
  after Measure distance, and Grid directly after Measure area. Draw line comes directly
  after Origin and starts a new group.
- Tool modes (`SHELL-8`, `SHELL-9`):
  - With no document, no tool is active, clicking along the tool bar changes no mode, and only
    New, Load and the Panels menu respond. Undo and Redo are disabled (`HIST-9`).
  - With a document, the chosen tool is active and every tool can be chosen.
- Tool controls row (`SHELL-2a`):
  - The canvas starts at the same place in every tool mode, and with a drawn shape active.
  - Without a document, the row is still at least its minimum height and shows its help text,
    which no tool mode shows with a document open.
  - Only Pan & zoom and Origin show help text, and an active object's controls replace it. In a
    narrow window it wraps.
  - In a narrow window the controls wrap and the canvas starts lower; widening it again
    returns the canvas to where it was.
  - An active object's controls edit that object and leave the tool's own settings alone.
- Cancelling (`SHELL-10a`):
  - Escape with nothing focused cancels; Escape in a focused text field does not, and
    the next Escape does.
  - Cancelling drops pending scale points and a scale point drag, committing nothing,
    and the document's points are unchanged.
  - Cancelling drops a pending measurement start and the points placed for a new area or
    drawn shape.
  - Cancelling a drag of the active object commits nothing, shows the object unchanged
    and keeps it active.
  - Backspace with nothing focused is read as "remove the last point"; Backspace in a
    focused text field is not. Enter with nothing focused is read as "finish"; Enter in a
    focused text field is not.
  - Cancelling stops a rotate-handle drag.
