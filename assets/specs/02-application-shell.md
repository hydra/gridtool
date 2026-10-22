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
   The Pan & zoom, Measure distance, Measure area and Origin modes have no options row.
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
| 12 | Measure area mode button (`AREA-1`) | polygon (`pentagon`) | "`Measure area`" |
| 13 | Grid mode button | grid (`grid_on`) | "`Grid`" |
| 14 | Origin mode button | crosshair (`my_location`) | "`Origin`" (`ORIGIN-2`) |
| 15 | Horizontal spacer, 16 px | — | — |
| 16 | Undo button | undo arrow (`undo`) | "`Undo`" |
| 17 | Redo button | redo arrow (`redo`) | "`Redo`" |

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
zoom**, **Rotate view**, **Scale**, **Measure distance**, **Measure area**, **Grid** and
**Origin**. **Pan & zoom is
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
- the points placed so far for a new area (`AREA-2`)
- a drag of the active object's points or resize handles in progress (`OBJ-9`,
  `OBJ-15`); the object keeps its shape and stays active
- a rotate-handle drag in progress (`VIEW-13`); the view keeps the rotation it reached

Escape and switching mode cancel exactly the same things. The tool mode does not change,
and neither does the active object. Escape does **not** cancel the tool when it is meant
for something else:
- while a dialog is open, Escape answers the dialog (`FILE-11`)
- while a widget such as a text field has keyboard focus, Escape only takes the focus
  away from it; a further Escape then cancels the tool

**SHELL-10b — Removing the last point.** Pressing **Backspace** removes the last point
placed for a new area (`AREA-3`). Like Escape, it does nothing while a dialog is open or
while a widget such as a text field has keyboard focus (where it edits the text).

## Side panel

**SHELL-11.** The side panel is docked on the right of the window:
- It can be resized horizontally and has a default width of 460 px.
- It is split vertically into three sections, from top to bottom:

  | Section | Contents | Initial share of the height |
  |---|---|---|
  | Point-to-point | the distance measurements table (`MEAS-10`) | 33 % |
  | Areas | the area measurements table (`AREA-12`) | 34 % |
  | History | the history list (`HIST-10`) | 33 % |

- **Shares.** Each section's share is tracked as a fraction of the height available to
  the sections (the panel's height less the dividers). When the window is resized, each
  section keeps its share.
- **Dividers.** Between neighbouring sections is a divider, 6 px high, drawn as a
  horizontal line. Hovering over or dragging it shows a vertical-resize cursor.
  Dragging it up or down moves height between the section above and the section below
  only: one grows by exactly what the other loses, and the third keeps its share.
- **Minimum.** A divider cannot make a section smaller than 40 px.
- Each section scrolls independently. With no document open, the two tables show their
  heading and "`No document open`" in weak text.

## Acceptance criteria

- The default tool mode is Pan & zoom.
- Every mode has a distinct icon and a distinct name.
- In the tool bar, Measure distance comes directly after Scale, Measure area directly
  after Measure distance, and Grid directly after Measure area.
- Cancelling (`SHELL-10a`):
  - Escape with nothing focused cancels; Escape in a focused text field does not, and
    the next Escape does.
  - Cancelling drops pending scale points and a scale point drag, committing nothing,
    and the document's points are unchanged.
  - Cancelling drops a pending measurement start and the points placed for a new area.
  - Cancelling a drag of the active object commits nothing, shows the object unchanged
    and keeps it active.
  - Backspace with nothing focused is read as "remove the last point"; Backspace in a
    focused text field is not.
  - Cancelling stops a rotate-handle drag.
- Side panel (`SHELL-11`):
  - The initial shares are 33 %, 34 % and 33 %; shares given in any proportion are
    scaled to add up to 1.
  - Moving the lower divider down by 10 % of the height gives 33 %, 44 % and 23 %;
    then moving the upper divider up by 13 % gives 20 %, 57 % and 23 %.
  - Dragging a divider far past a section stops at the minimum, and the other sections
    are unchanged.
  - With 412 px for 3 sections at 25 %, 25 % and 50 %, the sections are 100, 100 and
    200 px high, separated by two 6 px dividers.
  - The panel shows every section, with and without a document open.
