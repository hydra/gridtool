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
| [02-application-shell.md](02-application-shell.md) | Window, layout, tool bar, tool modes, side panel | `SHELL` |
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
| [13-files.md](13-files.md) | New, Load, Save, Save as; unsaved changes; closing the window; closing the window; the `.ron` file format | `FILE` |
| [14-areas.md](14-areas.md) | Measure area tool, area measurements, Areas table | `AREA` |

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
| **Document** | The thing being edited: an image-sized rectangle plus its calibration data, measurements, areas, grids and origin. |
| **Document file** | A `.ron` file describing a document in a versioned format, referring to its image by path (`FILE`). |
| **Unsaved changes** | The open document differs from the document as last loaded or saved, or it has never been saved. |
| **Image coordinates** | Pixel coordinates on the un-rotated image. The origin is at the top-left corner, x increases to the right and y increases downwards. Values may be fractional. |
| **Screen coordinates** | Positions in the application window, in logical screen units. |
| **View** | How the document is displayed: pan, zoom and rotation. View state is never part of the document. |
| **Action** | A single change to the document. It can be undone and redone. |
| **Tool mode** | The active tool. It decides what mouse input on the canvas does. |
| **Calibration points** | Points placed on the image that define two reference lines, A and B. |
| **Scale** | The real-world length of one image pixel along the image's X axis and along its Y axis. |
| **Measurement** | A line on the document, with a start and an end point, whose real-world length is derived from the scale. Called a **point-to-point** measurement in the UI. |
| **Area** | An area measurement: a closed outline through 3 or more points on the document, whose real-world area is derived from the scale. |
| **Object** | Something the user places on the document and can select: measurements, areas and grids. Each has a stable object id. |
| **Active object** | The single selected object. Measurements and areas show a purple selection box with resize handles, and their points can be dragged; grids show their properties row. It is presentation state, not part of the document. |
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
