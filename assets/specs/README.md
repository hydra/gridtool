# GridTool — Requirement Specifications

GridTool is a desktop application for loading an image of a real-world object and
calibrating its real-world scale.

These specifications describe **what** the application must do. They do not depend on
any programming language or UI toolkit. Any implementation, in any language or toolkit,
must satisfy them.

## Documents

| File | Area | ID prefix |
|---|---|---|
| [01-architecture.md](01-architecture.md) | Separation of document logic from presentation; testability | `ARCH` |
| [02-application-shell.md](02-application-shell.md) | Window, layout, tool bar, tool modes | `SHELL` |
| [03-image-loading.md](03-image-loading.md) | Choosing and decoding image files | `IMG` |
| [04-document-model.md](04-document-model.md) | The document and its coordinate system | `DOC` |
| [05-view-navigation.md](05-view-navigation.md) | Pan, zoom and view rotation (view state, not document state) | `VIEW` |
| [06-scale-calibration.md](06-scale-calibration.md) | Scale tool, calibration points, scale calculation | `SCALE` |
| [07-actions-and-history.md](07-actions-and-history.md) | Document actions, undo/redo, history panel | `HIST` |
| [08-formatting-and-parsing.md](08-formatting-and-parsing.md) | Number display, text input rules, commit-on-blur fields | `FMT` |
| [09-measurements.md](09-measurements.md) | Measure tool, measurement list and table | `MEAS` |
| [10-objects-and-selection.md](10-objects-and-selection.md) | Object ids, object tree, active object, selection box and resizing | `OBJ` |
| [11-grids.md](11-grids.md) | Grid tool, grid geometry, grid properties row | `GRID` |
| [12-origin.md](12-origin.md) | Document origin (rotation centre), Origin mode and marker | `ORIGIN` |
| [13-files.md](13-files.md) | New, Load, Save, Save as; unsaved changes; closing documents and the window; the `.ron` file format | `FILE` |
| [14-areas.md](14-areas.md) | Measure area tool, area measurements, Areas table | `AREA` |
| [15-shapes.md](15-shapes.md) | Draw line tool, drawn shapes and their styles, shape properties row | `DRAW` |
| [16-workspace.md](16-workspace.md) | Dockable panels, the Documents area, multiple documents, the active document | `WORK` |

## Conventions

- Each requirement has a stable ID such as `SCALE-12`. IDs are never reused. A
  requirement that is removed is marked **Withdrawn** and keeps its ID.
- **MUST** means the requirement is mandatory. **SHOULD** means it is recommended.
  **MAY** means it is optional.
- Lengths written as "px" or "points" are **logical screen units** (device-independent
  pixels), unless a requirement says they are *image pixels*.
- Text in `code style` that appears inside quotation marks must be reproduced exactly.
- Icons are described by meaning. The current Google Material Symbols icon name is given
  for reference (`SHELL-6`), and any icon with the same meaning is acceptable.
- Each area ends with **acceptance criteria**. These are concrete scenarios that an
  implementation's automated tests must cover.

## Glossary

| Term | Meaning |
|---|---|
| **Document** | The thing being edited: an image-sized rectangle plus its calibration data, measurements, areas, grids, drawn shapes and origin. |
| **Document file** | A `.ron` file describing a document in a versioned format, referring to its image by path (`FILE`). |
| **Unsaved changes** | An open document differs from the document as last loaded or saved, or it has never been saved. |
| **Image coordinates** | Pixel coordinates on the un-rotated image. The origin is at the top-left corner, x increases to the right and y increases downwards. Values may be fractional. |
| **Screen coordinates** | Positions in the application window, in logical screen units. |
| **View** | How the document is displayed: pan, zoom and rotation. View state is never part of the document. |
| **Action** | A single change to the document. It can be undone and redone. |
| **Active document** | The open document that the tool bar, panels and commands are showing and acting on (`WORK-10`). |
| **Tool mode** | The active tool. It decides what mouse input on the canvas does. |
| **Calibration points** | Points placed on the image that define two reference lines, A and B. |
| **Scale** | The real-world length of one image pixel along the image's X axis and along its Y axis. |
| **Measurement** | A line on the document, with a start and an end point, whose real-world length is derived from the scale. Called a **point-to-point** measurement in the UI. |
| **Area** | An area measurement: a closed outline through 3 or more points on the document, whose real-world area is derived from the scale. |
| **Drawn shape** | A line through 2 or more points on the document, or a closed outline through 3 or more, with a line and fill style. Nothing is measured from it, though its fill lines are a real-world distance apart. |
| **Object** | Something the user places on the document and can select: measurements, areas, grids and drawn shapes. Each has a stable object id. |
| **Active object** | The single selected object. Measurements, areas and drawn shapes show a purple selection box with resize handles, and their points can be dragged. Its controls show in the tool controls row, and its tool is the active tool. It is presentation state, not part of the document. |
| **Tool controls row** | The row always shown under the tool bar: the controls of the active object or, with none, of the active tool (`SHELL-2a`). |
| **Grid** | Regularly spaced lines over the whole document, aligned to a scale line or measurement line, with cells sized in real-world units. |
| **Origin** | A point on the image, stored in the document, that the view rotates about. It defaults to the image centre. |
| **Alignment source** | The scale line or measurement line a grid is aligned to. |

## Change log

| Date | Change |
|---|---|
| 2026-10-21 | Initial specification: image loading, view navigation, scale calibration, undo/redo. |
| 2026-10-21 | Added distance measurements (`09`), Measure distance tool, split side panel (`SHELL-11`), measurement actions (`HIST-2`), formats `FMT-6`–`FMT-8`, and commit-on-blur fields (`FMT-9`). |
| 2026-10-21 | Added icon verification (`SHELL-6a`) and the delete-button glyph fallback (`MEAS-11`). |
| 2026-10-21 | Added right-drag pan in all modes (`VIEW-11a`); tool gestures are now primary-button only. |
| 2026-10-21 | Added objects and selection (`10`): object ids, left object tree panel, active object, purple selection box with 8 resize handles, Resize measurement action, and centre-on-object (`VIEW-17`). Withdrew direct endpoint dragging (`MEAS-7`). The tree view uses egui_ltreeview 0.9.0, vendored and patched for egui 0.37 (`vendor/egui_ltreeview/PATCHES.md`). |
| 2026-10-21 | Added grids (`11`): Grid tool with Via scale / Via measurement, line hover and click to add, real-world grid geometry, Grids tree folder, grid properties row (shape, size, colour picker, opacity), five grid actions (`HIST-2`), object properties row (`SHELL-2`), and the drawing order (`GRID-6`). |
| 2026-10-21 | Added the object tool bar with Duplicate and Delete buttons (`OBJ-12`–`OBJ-14`), and the Duplicate measurement, Duplicate grid and Delete grid actions (`HIST-2`). |
| 2026-10-21 | Added the grid units drop-down (`GRID-14`) and the Set grid units action (`HIST-2`). A grid's unit now belongs to the grid and can be changed (`GRID-1`). |
| 2026-10-21 | Removed the whole-document 10 000-line limit that hid grids with cells of about 1 cm or less. Grids now draw only the visible region, and any cell size > 0 can be drawn, hidden only while under 3 screen px (`GRID-18`, `GRID-4`). Raised the maximum zoom from 64 to 1024 (`VIEW-10`). |
| 2026-10-21 | The rotate arrow now starts at the canvas centre instead of the document centre, so it stays a full 200 px at every zoom (`VIEW-13`). The view rotation now pivots about the canvas centre (`VIEW-3a`, `VIEW-14`). |
| 2026-10-21 | Added the document origin and Origin mode (`12`), the Set origin action (`HIST-2`), and the Origin mode button (`SHELL-5`). The origin defaults to the image centre. Rotation now pivots about the origin, and the rotate arrow starts there (`VIEW-3a`, `VIEW-13`, `VIEW-14`). This replaces the canvas-centre pivot of the previous entry. |
| 2026-10-21 | The rotate arrow is now a fixed 100 px (was 200 px) (`VIEW-13`). |
| 2026-10-21 | Wheel zoom always zooms about the point under the cursor, in every mode with no exceptions. This removes the short-lived Rotate-mode zoom about the origin (`VIEW-9`). |
| 2026-10-21 | Added document files (`13`): New, Load, Save and Save as buttons, unsaved-change tracking, the Unsaved changes and overwrite dialogs, and the versioned `.ron` format (version 1), which references the image and does not save the history. The Open image button now shows an image-file icon (`SHELL-5`). |
| 2026-10-21 | Icons are now Google Material Symbols (Rounded, filled) in a bundled icon font, named by their Material Symbols names (`SHELL-6`). This replaces the emoji glyphs and the delete button's "x" fallback. Measure distance now uses a tape measure (`measuring_tape`). Implemented with the `egui_material_icons` crate. |
| 2026-10-21 | Closing the window with unsaved changes now asks with the Unsaved changes dialog (`FILE-17`, `FILE-11`). The window title shows the document's file name, or "`unsaved document`", before the application name (`SHELL-1`). |
| 2026-10-21 | Added cancelling a tool operation (`SHELL-10a`): Escape abandons pending scale points, a pending measurement start, a scale point drag, a resize and a rotate-handle drag, exactly as switching tool mode now does. Switching mode now also discards pending scale points (`SCALE-10a`, `MEAS-5`). |
| 2026-10-22 | Added area measurements (`14`): the Measure area tool and button (`SHELL-5`), placing, removing (Backspace, `SHELL-10b`) and finishing outlines, filled convex and concave areas, the Area tree folder, the Areas table, and seven area actions (`HIST-2`). Measurements gained a colour, with a colour column in their table (`MEAS-1`, `MEAS-15`) and a Set measurement colour action. The measurements table is now headed "`Point-to-point`", and the tree has "`Point-to-point`" and "`Area`" folders inside "`Measurements`" (`OBJ-4`). The active measurement's or area's points can be dragged (`OBJ-15`), replacing `OBJ-11`, and an area's handles stretch it (`OBJ-9b`). The side panel now has three sections sharing the height 33 / 34 / 33 %, with dividers that move height between neighbours (`SHELL-11`). Files are now saved as version 2, and version 1 files are migrated (`FILE-6a`). |
| 2026-10-22 | Picking an object in the tree no longer always centres it (`OBJ-5`, `OBJ-5a`): the view stays put while any part of the object is on screen; otherwise it pans to the object's focus point, then zooms out if needed so the whole object fits with a 16 px margin. |
| 2026-10-22 | The Point-to-point table has separate "`x1`", "`y1`", "`x2`" and "`y2`" columns (`MEAS-11`). Coordinate fields are at least wide enough for "`9999.99`" and share any spare width in the section equally (`MEAS-12`). |
| 2026-11-01 | Added drawn shapes (`15`): the Draw line tool in a new drawing tools group after a spacer (`SHELL-5`), lines left open with a double-click or Enter (`SHELL-10c`) or joined up into closed shapes, three line styles, five fill styles with a fill line distance and separate line and fill opacity in a properties row (`SHELL-2`), a "`Shapes`" tree folder (`OBJ-4`), and ten shape actions (`HIST-2`). Files are now saved as version 3, and earlier versions are migrated (`FILE-6b`). The drawing order now lists areas and shapes (`GRID-6`). |
| 2026-11-01 | Revised drawn shapes before release, keeping file version 3 (`FILE-6b`). Line and fill colours can be chosen (`DRAW-8`, `DRAW-13`). The fill line distance is now a real-world value and unit, typed as for a grid, and needs a calibrated scale (`DRAW-10`). The Vertical lines and Horizontal lines fills are replaced by "`Lines`", horizontal by default, and Lines and Crosshatch can be turned with rotate 90° buttons or a typed angle (`DRAW-13`, `DRAW-14`). Four shape actions added and the fill spacing action's label changed (`HIST-2`). |
| 2026-11-01 | Added a tool controls row, always shown under the tool bar and at least as tall as it, so the canvas does not move; it wraps when too narrow (`SHELL-2a`). It replaces the tool options and object properties rows. Measurements and areas now have line styles, and areas fill styles, as drawn shapes do (`MEAS-1`, `AREA-6`, `AREA-9`), with controls for the active object (`MEAS-16`, `AREA-16`). The Measure distance, Measure area and Draw line tools have controls for the next object they add (`MEAS-17`, `AREA-17`, `DRAW-17`). An object that becomes active brings its tool, and choosing another tool clears the active object (`OBJ-7a`). Starting a new object clears the active object (`OBJ-10a`). One Set style action replaces the shape style actions and the Set measurement colour and Set area colour actions, and their labels now say "line color" (`HIST-2`, `DRAW-18`). Measurement lines change from 2 px to Thick, 3 px, and area outlines likewise. Files are now saved as version 4, and earlier versions are migrated (`FILE-6c`). |
| 2026-11-01 | A new measurement or area now becomes the active object, as a new drawn shape or grid does, so the tool controls row edits it straight away (`MEAS-6`, `AREA-4`, `OBJ-7a`). |
| 2026-11-01 | The fill line distance of a new area or drawn shape is now 1, not 10, in its fill unit (`AREA-6`, `DRAW-8`). Areas migrated from version 3 files keep 10 (`FILE-6c`), and saved objects keep their own values. |
| 2026-11-01 | A fill colour is now optional, and none by default: such a fill is drawn in the line colour (`DRAW-8`, `AREA-6`). A padlock before the fill colour swatch shows this, disables the swatch, and sets or forgets the fill's own colour (`DRAW-19`). Files are now saved as version 5, and earlier versions are migrated with each fill keeping its colour (`FILE-6d`). |
| 2026-11-01 | Fill angles are now measured from vertical, so 90 is horizontal (`DRAW-8`, `FILE-6d`), still in file version 5, with earlier versions migrated. New fills start at 90 for Lines and 45 for Crosshatch. The Draw line and Measure area tools remember one angle per fill style, set from the tool's controls and from objects, and changing an object's fill style changes its angle to the one remembered, in one action (`DRAW-20`). |
| 2026-11-01 | Scroll bars are drawn whenever there is more to scroll to, throughout the application, instead of only on hover (`SHELL-12`). |
| 2026-11-01 | The colour is now the second column of the Point-to-point and Areas tables (`MEAS-11`, `AREA-12`). A table row is selected while its object is active, so at most one is across the side panel, clicking a row makes its object active, and a newly selected row scrolls into view (`OBJ-6a`). An object made active elsewhere opens its folders and scrolls into view in the tree, which now scrolls (`OBJ-6`). |
| 2026-11-01 | The active object can be moved by dragging inside its selection box, in every tool mode, stopping at the document's edges with its form kept (`OBJ-16`), with three new actions (`HIST-2`). The cursor now shows what a drag would do: a four-way arrow inside the box, and over a resize handle a two-way arrow along its direction, which follows the handle across the box during a resize and turns with the view (`OBJ-17`), replacing the open and grabbing hands on handles (`OBJ-9`). |
| 2026-11-01 | The open and grabbing hand cursors (`VIEW-11`, `VIEW-11a`, `VIEW-14`, `OBJ-15`) are now real hands on Windows, not a four-way arrow, so they differ from the cursor for moving an object (`OBJ-17`). Windows has no such system cursors, so winit 0.30.13 is vendored and patched to draw them (`vendor/winit/PATCHES.md`). |
| 2026-11-01 | The tool controls row shows help text for the tools with no controls, Pan & zoom and Origin (`SHELL-2a`). |
| 2026-11-01 | With no document open there is now no active tool: every mode button is disabled and none is selected, and Pan & zoom becomes active when a document next opens (`SHELL-8`, `SHELL-9`). The tool controls row then says "`Open an image to get started`" (`SHELL-2a`). |
| 2026-11-01 | The application has one icon, used for the window, task bar or dock, executable file and release packages (`SHELL-1a`). |
| 2026-11-01 | A newly opened or loaded document now fits in the canvas area, zoomed out if needed, instead of always starting at 100 % (`VIEW-2`, `VIEW-2a`). |
| 2026-11-01 | Several documents can be open at once, each with its own tab, view, tool, active object and history, and the panels are now dockable (`16`). The Documents area can be split and its tabs regrouped; panels can be docked around it, grouped, closed and reopened from a new Panels menu, and their arrangement is restored at start-up. This withdraws the side panel (`SHELL-11`) and the error shown in the canvas area (`SHELL-4`), since a failed image open now shows a dialog (`IMG-6`). New now opens an image and adds a document (`FILE-12`, `IMG-2`), replacing the Open image button (`SHELL-5`). Nothing replaces an open document any more, so only closing a document or the window asks about unsaved changes, naming the document (`FILE-11`, `FILE-17`). Load of a file already open switches to it (`FILE-13`). The tool mode is per document (`SHELL-8`, `SHELL-9`). Panels have no headings, their titles being on their tabs (`OBJ-3`, `MEAS-10`, `AREA-12`, `HIST-10`). |
| 2026-11-01 | Tabs and tab bars no longer show the grab cursor on hover, only the grabbing hand while being dragged (`WORK-14`). |
| 2026-11-01 | The fill line rotate buttons now turn the lines 45° instead of 90°, so left and right now give different results (`DRAW-13`, `DRAW-14`). |
| 2026-11-01 | Fill angles now range from 0 up to 360 instead of 180 (`DRAW-8`), so rotating adds to or subtracts from the current angle without folding back at 180, wrapping around a full turn (`DRAW-14`). A typed angle must be from 0 to 360, where 360 wraps to 0, and negative or larger numbers are now rejected instead of converted. Saved files are now version 6 (`FILE-6e`), and earlier versions keep their own limit. |
| 2026-11-01 | A typed fill angle below 0, or of 360 or more, now wraps around a full turn instead of being rejected, so `-45` gives 315 and `361` gives 1 (`DRAW-14`). |
| 2026-11-01 | The Windows installer program now shows the application icon (`SHELL-1a`). |
| 2026-11-01 | A release build no longer opens a console window on Windows (`SHELL-1b`). |
| 2026-11-01 | A panic in a release build now saves a crash report and shows a dialog saying where it is (`SHELL-1c`). |
| 2026-11-01 | Added an About button at the right-hand end of the tool bar, and its dialog (`SHELL-5`, `SHELL-13`). |
