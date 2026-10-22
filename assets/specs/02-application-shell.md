# 02 — Application Shell

## Window

**SHELL-1.** The application opens one native window with an initial inner size of
1280 × 800. Its title follows the usual Windows form "`<document> - GridTool`":
- With no document open, the title is "`GridTool`".
- With a document that has a file (`FILE-14`), `<document>` is the file's name with its
  extension and without its folder, for example "`plan.ron - GridTool`".
- With a document that has never been saved, `<document>` is "`unsaved document`".
- The title updates as soon as a document is opened, saved under a new name or closed.

Closing the window may first ask about unsaved changes (`FILE-17`).

## Layout

**SHELL-2.** The window is divided into these regions, from top to bottom:

1. **Main tool bar.** One horizontal row, always visible.
2. **Tool options row.** A second horizontal row directly below the tool bar. It is shown
   only when the active tool mode has options *and* a document is open. Its contents
   depend on the mode: see `VIEW-12` (Rotate), `SCALE-3` (Scale) and `GRID-8` (Grid).
   The Pan & zoom, Measure distance and Origin modes have no options row.
3. **Object properties row.** Shown below the tool options row while the active object
   has editable properties. Currently this means grids (`GRID-14`).
4. Below the rows, side by side, from left to right:
   - **Object panel.** Docked on the left-hand side (see `OBJ-3`).
   - **Canvas.** Fills the remaining space and shows the document (see `VIEW`, `SCALE`,
     `MEAS`, `OBJ`).
   - **Side panel.** Docked on the right-hand side (see `SHELL-11`).

**SHELL-3.** When no document is open, the canvas area shows the centred message
"`Open an image to get started.`"

**SHELL-4.** When the last attempt to open an image failed, the canvas area shows the
error message centred, in the toolkit's error colour (see `IMG-6`).

## Main tool bar

**SHELL-5.** The tool bar contains these items, left to right:

| # | Item | Icon meaning (current Material Symbols icon) | Tooltip |
|---|---|---|---|
| 1 | New button (`FILE-10`) | new document (`note_add`) | "`New`" |
| 2 | Load button | open folder (`folder_open`) | "`Load…`" |
| 3 | Save button | save (`save`) | "`Save`" |
| 4 | Save as button | save as (`save_as`) | "`Save as…`" |
| 5 | Horizontal spacer, 16 px | — | — |
| 6 | Open image button | image (`image`) | "`Open image…`" |
| 7 | Horizontal spacer, 16 px | — | — |
| 8 | Pan & zoom mode button | hand (`pan_tool`) | "`Pan & zoom`" |
| 9 | Rotate view mode button | rotate (`rotate_right`) | "`Rotate view`" |
| 10 | Scale mode button | ruler (`straighten`) | "`Scale`" |
| 11 | Measure distance mode button | tape measure (`measuring_tape`) | "`Measure distance`" |
| 12 | Grid mode button | grid (`grid_on`) | "`Grid`" |
| 13 | Origin mode button | crosshair (`my_location`) | "`Origin`" (`ORIGIN-2`) |
| 14 | Horizontal spacer, 16 px | — | — |
| 15 | Undo button | undo arrow (`undo`) | "`Undo`" |
| 16 | Redo button | redo arrow (`redo`) | "`Redo`" |

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

**SHELL-8.** There is exactly one active tool mode at a time. The modes are **Pan &
zoom**, **Rotate view**, **Scale**, **Measure distance**, **Grid** and **Origin**. **Pan & zoom is
the default** when the application starts.

**SHELL-9.** The mode buttons work like a radio group. The active mode's button is shown
as selected, and clicking a mode button makes that mode active. The active mode is
application state and stays the same when a different image is opened.

**SHELL-10.** New modes will be added later. The list of modes MUST be defined in one
place, so that adding a mode needs no duplicated tool bar code.

**SHELL-10a — Cancelling a tool operation.** Pressing **Escape**, or switching to another
tool mode, abandons whatever the tool is part-way through, without committing an action
or changing the document:
- the pending scale points while placing (`SCALE-4`), and any scale point drag
- the pending start point of a measurement (`MEAS-5`)
- a resize of the active object in progress (`OBJ-9`); the object keeps its size and
  stays active
- a rotate-handle drag in progress (`VIEW-13`); the view keeps the rotation it reached

Escape and switching mode cancel exactly the same things. The tool mode does not change,
and neither does the active object. Escape does **not** cancel the tool when it is meant
for something else:
- while a dialog is open, Escape answers the dialog (`FILE-11`)
- while a widget such as a text field has keyboard focus, Escape only takes the focus
  away from it; a further Escape then cancels the tool

## Side panel

**SHELL-11.** The side panel is docked on the right of the window:
- It can be resized horizontally and has a default width of 460 px.
- It is split vertically into two sections:
  1. **Upper section**: the measurements table (`MEAS-10`), with a default height of
     280 px.
  2. **Lower section**: the history list (`HIST-10`), filling the remaining height.
- The divider between the sections can be dragged up and down to resize them.
- Each section scrolls independently.

## Acceptance criteria

- The default tool mode is Pan & zoom.
- Every mode has a distinct icon and a distinct name.
- In the tool bar, Measure distance comes directly after Scale.
- Cancelling (`SHELL-10a`):
  - Escape with nothing focused cancels; Escape in a focused text field does not, and
    the next Escape does.
  - Cancelling drops pending scale points and a scale point drag, committing nothing,
    and the document's points are unchanged.
  - Cancelling drops a pending measurement start.
  - Cancelling a resize commits nothing, shows the object unchanged and keeps it active.
  - Cancelling stops a rotate-handle drag.
