# 10 — Objects, Object Tree and Selection

The document holds **objects** that the user places. Today the only kind is the
straight-line measurement (`MEAS`). Grids and area measurements will be added later and
MUST fit the same model: identity, an entry in the tree, a bounding box and a resize
behaviour.

## Object identity

**OBJ-1 — Object id.** Every object has an **object id** that is stable for its whole
life:
- The id does not change when other objects are added, deleted or edited.
- Ids are allocated by the document from a counter that only increases. They are never
  reused within one document's history, including after a delete.
- The counter is part of the document state, so undo and redo restore it exactly
  (`HIST-6`, `HIST-7`).
- Actions MAY address objects by list position (`HIST-2`). Selection and the object tree
  MUST address them by id (`OBJ-6`).

**OBJ-2 — Bounding box.** Every object has an axis-aligned bounding box in image
coordinates. For a measurement it is the smallest box containing both endpoints, so it
may have zero width or height.

## Object tree

**OBJ-3 — Panel.** A panel docked on the **left** of the window (`SHELL-2`):
- It can be resized horizontally and has a default width of 220 px.
- It has the heading "`Objects`", then a separator, then a tree view.
- With no document open it shows "`No document open`" in weak text.

**OBJ-4 — Tree contents.** With a document open, the tree contains:

```
Document
├─ Measurements
│  ├─ Measurement 1 (12.5 cm)
│  ├─ Measurement 2
│  └─ …
└─ Grids
   ├─ Grid 1 (scale line A)
   └─ …
```

- **Root node:** "`Document`".
- **Folders:** "`Measurements`", then "`Grids`" (`GRID-11`), under the root.
- **Measurements:** one leaf per measurement, in list order (`MEAS-2`) and never sorted.
- **Leaf label:** "`Measurement <n> (<distance> <unit symbol>)`", where `n` is the 1-based
  list position and the distance is formatted by `FMT-7`. When the distance is
  unavailable, the label is "`Measurement <n>`".
- **Folders:** start expanded and can be collapsed.
- **Selection:** only one node at a time. Nodes cannot be dragged or reordered.

**OBJ-5 — Clicking an object in the tree.** When the user selects an object leaf:
1. That object becomes the **active object** (`OBJ-6`).
2. The view pans so that the object's **focus point** is at the centre of the canvas
   area. Zoom and view rotation are unchanged (`VIEW-17`). The focus point is:
   - for a measurement, the centre of its bounding box (the midpoint of the line)
   - for a grid, the midpoint of its alignment line. If that line no longer exists, the
     view does not move.

Selecting a non-object node ("`Document`", "`Measurements`" or "`Grids`") clears the
active object.

**OBJ-6 — Tree reflects the active object.** The tree always shows the active object's
leaf as selected:
- When the active object changes by any route, including clicking on the canvas
  (`OBJ-10`) or undo/redo removing it (`OBJ-7`), the tree selection updates to match.
- When there is no active object, no object leaf is selected. A selected folder node may
  stay selected.

## Active object

**OBJ-7 — Active object.** There is at most one active object at a time:
- It is identified by object id (`OBJ-1`).
- It is presentation state: not part of the document, not an action, and not affected by
  undo/redo except as below.
- After **any** document change (an action, undo or redo), if the active object no
  longer exists, the active object is cleared. Any resize in progress is abandoned.
- Because selection uses ids, deleting an *earlier* measurement leaves the same
  measurement active, even though its list position changes.

**OBJ-8 — Selection box.** While there is an active object that has a bounding box
(currently measurements; grids do not, see `GRID-12`), the canvas draws the following on
top of everything else, in every tool mode:
- **Box.** A **dashed purple** (`#800080`) box, 1.5 px wide, with 6 px dashes and 4 px
  gaps, around the object's bounding box.
- **Padding.** The box is pushed outward from the bounding box by **8 screen px** on
  every side, so a perfectly level or upright line still gets a box with separate
  handles. The padding is the same at every zoom level.
- **Rotation.** The box rotates with the view, like the document.
- **Resize handles.** Eight filled purple squares, each **10 × 10 px** and axis-aligned
  on screen, centred on the box's four corners and the midpoints of its four edges.
- **Live updates.** During a resize, the box, the object and the measurements table all
  show the live preview.

## Resizing

**OBJ-9 — Resizing with the handles.** In **every** tool mode, the active object's
handles take priority over the tool:
- **Starting.** A primary-button drag that *starts* within 8 px of a handle's centre
  resizes the object. If several handles are in range, the nearest one is used.
- **No tool input.** While resizing, the tool receives no primary-button input: Pan &
  zoom mode does not pan, and the scale and measure tools do not react.
- **Panning.** A secondary-button drag on a handle pans as usual (`VIEW-11a`).
- **Cursor.** An open hand while hovering over a handle, and a grabbing hand while
  resizing.
- **Committing.** When the drag ends, one **Resize measurement** action (`HIST-2`) is
  committed with both new endpoints. If nothing changed, no action is committed.

**OBJ-9a — How a handle moves a measurement.** Let `(dx, dy)` be the pointer's movement in
image pixels since the drag started. A handle grabs the bounding-box edges it sits on:

| Handle | Edges grabbed |
|---|---|
| top-left | left (min x) and top (min y) |
| top | top only |
| top-right | right (max x) and top |
| right | right only |
| bottom-right | right and bottom (max y) |
| bottom | bottom only |
| bottom-left | left and bottom |
| left | left only |

The endpoints follow the edges like this:
- **Which edge each endpoint is on.** Along each axis, one endpoint lies on the min edge
  and the other on the max edge. On a tie (equal coordinates), the **start** takes the
  min edge.
- **Moving.** Each grabbed edge moves by `dx` (x edges) or `dy` (y edges). Every endpoint
  coordinate lying on a grabbed edge moves with it, and all other coordinates stay put.
- **Resulting behaviour.**
  - Dragging the corner where an endpoint sits moves only that endpoint.
  - Dragging an empty corner moves one coordinate of each endpoint.
  - Dragging an edge handle changes only one axis.
  - Dragging an edge past the opposite edge simply flips the line.
- **Clamping.** Each endpoint is clamped to the document (`DOC-5`).

**OBJ-10 — Clearing the selection.** In every tool mode, a primary click (press and
release without dragging) whose position lies **outside** the selection box clears the
active object. An active object without a selection box (a grid) is cleared by any
primary click on the canvas (`GRID-12`). "Outside" means outside the bounding box grown by `8 + 5` screen px on
every side (padding plus half a handle).
- Clicks inside keep the active object.
- The click is still passed to the active tool. In Measure mode, for example, it also
  places a point.

**OBJ-11 — No direct endpoint dragging.** Measurement endpoints cannot be dragged
directly in any mode. They change only through the resize handles (`OBJ-9`) or the
coordinate fields in the measurements table (`MEAS-12`). This replaces the withdrawn
`MEAS-7`.

## Acceptance criteria

- Ids are distinct. An object keeps its id when earlier objects are deleted, and the id
  of a deleted object is not reused by the next object added.
- Measurement bounding box: the line (10,50)→(40,20) has bounds (10,20)–(40,50).
- Resize maths, using that line:
  - The top-right corner by (5, −3) moves only the end, to (45, 17).
  - The top-left corner by (−4, −6) moves the start's x to 6 and the end's y to 14.
  - The bottom edge by (99, 7) moves only the start's y, to 57.
  - The right edge by −50 flips the line, so the end's x becomes −10.
  - A level line gains height from either edge: the bottom edge moves the end, the top
    edge moves the start.
  - The unit is kept.
- **Resize measurement** replaces both endpoints and keeps the id and unit. It is
  rejected if outside the document or for a missing index, leaving the document
  unchanged. Its label is "`Resize measurement 1 to (1.0, 2.0) - (3.0, 4.0)`".
- Selection:
  - Starts empty. The active bounds follow the selected measurement.
  - A resize previews and ends with one action, clamped to the document. A resize that
    changes nothing produces no action.
  - There is no resize without an active object. Only the active measurement shows the
    preview.
  - Sync clears a deleted selection and any resize, and keeps the selection when earlier
    objects are deleted.
- Tree:
  - The tree selection follows the active object.
  - Clearing the active object deselects its leaf but keeps a folder selection.
  - Only a single object node counts as picking an object.
- View: centring on an image point puts it at the view centre at any zoom, rotation and
  previous pan, and leaves zoom and rotation unchanged.
- Canvas, simulated with pointer events:
  - Dragging the right handle by (40, 25) in **every** mode commits exactly one resize,
    moving the end's x by 40 (y unchanged), with no pan.
  - Right-dragging a handle pans and does not resize.
  - Dragging a measurement endpoint directly does nothing in every mode.
  - Clicking inside the box keeps the selection, and clicking outside clears it, in every
    mode.
  - The handle positions are the 4 corners and 4 edge midpoints.
