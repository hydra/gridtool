# 10 — Objects, Object Tree and Selection

The document holds **objects** that the user places: point-to-point measurements
(`MEAS`), area measurements (`AREA`) and grids (`GRID`). Every kind fits the same model:
identity, an entry in the tree, duplicate and delete, and, where it makes sense, a
bounding box, a resize behaviour and draggable points.

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
may have zero width or height. For an area it is the smallest box containing all of its
points.

## Object tree

**OBJ-3 — Panel.** A panel docked on the **left** of the window (`SHELL-2`):
- It can be resized horizontally and has a default width of 220 px.
- It has the heading "`Objects`", then the **object tool bar** (`OBJ-12`, `OBJ-13`),
  then a separator, then a tree view.
- With no document open it shows "`No document open`" in weak text below the separator,
  and both tool bar buttons are disabled.

**OBJ-4 — Tree contents.** With a document open, the tree contains:

```
Document
├─ Measurements
│  ├─ Point-to-point
│  │  ├─ Measurement 1 (12.5 cm)
│  │  ├─ Measurement 2
│  │  └─ …
│  └─ Area
│     ├─ Area 1 (3.25 m²)
│     └─ …
└─ Grids
   ├─ Grid 1 (scale line A)
   └─ …
```

- **Root node:** "`Document`".
- **Folders:** "`Measurements`", then "`Grids`" (`GRID-11`), under the root. Inside
  "`Measurements`": "`Point-to-point`", then "`Area`".
- **Point-to-point measurements:** one leaf per measurement, in list order (`MEAS-2`)
  and never sorted, in the "`Point-to-point`" folder.
- **Areas:** one leaf per area, in list order (`AREA-6`), in the "`Area`" folder,
  labelled as `AREA-11` says.
- **Leaf label:** "`Measurement <n> (<distance> <unit symbol>)`", where `n` is the 1-based
  list position and the distance is formatted by `FMT-7`. When the distance is
  unavailable, the label is "`Measurement <n>`".
- **Folders:** start expanded and can be collapsed.
- **Selection:** only one node at a time. Nodes cannot be dragged or reordered.

**OBJ-5 — Clicking an object in the tree.** When the user selects an object leaf:
1. That object becomes the **active object** (`OBJ-6`).
2. The object is brought into view (`OBJ-5a`). This uses the object's **outline** and
   **focus point**.

   The outline is:
   - for a measurement, its line
   - for an area, its closed outline, including the area inside it
   - for a grid, its alignment line

   The focus point is:
   - for a measurement, the centre of its bounding box (the midpoint of the line)
   - for an area, the centre of its area (`AREA-8`)
   - for a grid, the midpoint of its alignment line. If that line no longer exists, the
     view does not move.

Selecting a non-object node ("`Document`", "`Measurements`", "`Point-to-point`", "`Area`"
or "`Grids`") clears the active object.

**OBJ-5a — Bringing an object into view.** The canvas is the screen area of the canvas as
last drawn. View rotation is never changed.
1. **Already in view.** If any part of the outline is on the canvas, the view does not
   move. This covers a point on the canvas, part of a line or edge crossing it, or the
   canvas being inside an area.
2. **Pan.** Otherwise the view pans so that the focus point is at the centre of the
   canvas (`VIEW-17`). The zoom is not changed yet.
3. **Zoom out to fit.** If, after the pan, any point of the outline is closer than
   **16 px** to an edge of the canvas, or beyond it, the view zooms out about the canvas
   centre. It uses the largest zoom at which every point is at least 16 px inside, so
   the focus point stays at the centre. The zoom is never increased, and stays within the
   zoom limits (`VIEW-10`).

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
  longer exists, the active object is cleared. Any drag of its points or resize handles
  in progress is abandoned.
- Because selection uses ids, deleting an *earlier* measurement leaves the same
  measurement active, even though its list position changes.

**OBJ-8 — Selection box.** While there is an active object that has a bounding box
(measurements and areas; grids do not, see `GRID-12`), the canvas draws the following on
top of everything else, in every tool mode:
- **Box.** A **dashed purple** (`#800080`) box, 1.5 px wide, with 6 px dashes and 4 px
  gaps, around the object's bounding box.
- **Padding.** The box is pushed outward from the bounding box by **8 screen px** on
  every side, so a perfectly level or upright line still gets a box with separate
  handles. The padding is the same at every zoom level.
- **Rotation.** The box rotates with the view, like the document.
- **Resize handles.** Eight filled purple squares, each **10 × 10 px** and axis-aligned
  on screen, centred on the box's four corners and the midpoints of its four edges.
- **Live updates.** During a resize or a point drag (`OBJ-15`), the box, the object and
  its table row all show the live preview.

## Resizing

**OBJ-9 — Resizing with the handles.** In **every** tool mode, the active object's
handles, and its points (`OBJ-15`), take priority over the tool:
- **Starting.** A primary-button drag that *starts* within 8 px of a handle's centre
  resizes the object. If several handles are in range, the nearest one is used. If a
  point of the object is also in range (`OBJ-15`), whichever of the two is nearer to the
  press wins.
- **No tool input.** While resizing, the tool receives no primary-button input: Pan &
  zoom mode does not pan, and the scale and measure tools do not react.
- **Panning.** A secondary-button drag on a handle pans as usual (`VIEW-11a`).
- **Cursor.** An open hand while hovering over a handle or a point, and a grabbing hand
  while dragging.
- **Committing.** When the drag ends, one **Resize measurement** action (`HIST-2`) is
  committed with both new endpoints, or for an area one **Resize area** action with all
  of its new points (`OBJ-9b`). If nothing changed, no action is committed.

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

**OBJ-9b — How a handle stretches an area.** Let the bounding box run from `min` to `max`
along an axis, and let the grabbed edge on that axis move by `d` (`dx` or `dy`). The
opposite edge stays put and every point stretches proportionally between them:
- grabbing the max edge: `v' = min + (v − min) × (size + d) / size`
- grabbing the min edge: `v' = max + (v − max) × (size − d) / size`

where `size = max − min`. An axis the handle does not grab is unchanged. Dragging an
edge past the opposite edge mirrors the points. Along an axis where the box has no size,
every point moves by `d` instead. Each point is then clamped to the document (`DOC-5`).
The unit and colour are kept.

**OBJ-10 — Clearing the selection.** In every tool mode, a primary click (press and
release without dragging) whose position lies **outside** the selection box clears the
active object. An active object without a selection box (a grid) is cleared by any
primary click on the canvas (`GRID-12`). "Outside" means outside the bounding box grown by `8 + 5` screen px on
every side (padding plus half a handle).
- Clicks inside keep the active object.
- The click is still passed to the active tool. In Measure mode, for example, it also
  places a point.

**OBJ-11 — Withdrawn.** Measurement endpoints could not be dragged directly in any mode.
This is replaced by dragging the active object's points (`OBJ-15`); the endpoints of a
measurement that is not active still cannot be dragged.

## Object tool bar

The object tool bar is one horizontal row directly below the "`Objects`" heading. Its
buttons use the tool bar icon size (`SHELL-6`), and their tooltips show even while they
are disabled (`SHELL-7`).

| # | Item | Icon meaning (current Material Symbols icon) | Tooltip |
|---|---|---|---|
| 1 | Duplicate button | copy (`content_copy`) | "`Duplicate`" |
| 2 | Delete button | trash can (`delete`), the same icon as the measurements table (`MEAS-11`) | "`Delete`" |

**OBJ-12 — Duplicate.** The Duplicate button is enabled exactly when there is an active
object and that kind of object can be duplicated. Measurements, areas and grids can all
be duplicated. Clicking it:
1. Commits one **Duplicate measurement**, **Duplicate area** or **Duplicate grid** action
   (`HIST-2`) for the active object.
2. The copy has **exactly the same properties** as the original, and only its object id
   differs (a new id, `OBJ-1`):
   - measurement: both endpoints, the unit and the colour
   - area: every point, the unit and the colour
   - grid: alignment source, cell shape, both cell sizes, unit, colour and opacity
3. The copy is appended to the end of its list, so it is the last leaf of its tree folder.
4. The copy becomes the **active object**. Because it lies exactly on top of the
   original, its selection box (if any) is in the same place.

**OBJ-13 — Delete.** The Delete button is enabled exactly when there is an active object
and that kind of object can be deleted. Measurements, areas and grids can all be deleted.
Clicking it commits one **Delete measurement**, **Delete area** or **Delete grid** action
(`HIST-2`) for the active object. The object no longer exists, so the active object is cleared
(`OBJ-7`). Grids aligned to a deleted measurement remain but are not drawn (`GRID-4`).

**OBJ-14 — Undo.** Duplicating and deleting are document actions, so they can be undone
and redone (`HIST-6`, `HIST-7`). Undo of a delete restores the object with its original
id. Undo of a duplicate removes the copy; if the copy was active, the active object is
cleared (`OBJ-7`).

**OBJ-15 — Dragging the active object's points.** In **every** tool mode, the points of
the active object can be dragged: a measurement's two endpoints, or an area's points.
- **Starting.** A primary-button drag that *starts* within 10 px of a point's screen
  position drags that point. If several points are in range, the nearest one is used.
  Against the resize handles, the nearer of the two wins (`OBJ-9`).
- **Moving.** The point moves by the pointer's movement since the press (in image
  pixels), so it does not jump to the pointer when grabbed off-centre. It is clamped to
  the document (`DOC-5`).
- **No tool input.** As for resizing, the tool receives no primary-button input while a
  point is dragged, and a secondary-button drag pans as usual.
- **Committing.** When the drag ends, one **Move measurement point** or **Move area
  point** action (`HIST-2`) is committed. If nothing changed, no action is committed.
- Only the active object's points can be dragged. The points of other objects cannot be
  grabbed.

## Acceptance criteria

- Bringing into view (`OBJ-5a`), with an 800 × 600 canvas, a 4000 × 3000 document at
  zoom 1 and no rotation (image region (1600, 1200)–(2400, 1800) visible):
  - These do not move the view: a line with one end on screen, a line crossing the
    screen with both ends off it, an area whose edges cross the screen with no point on
    it, and an area surrounding the whole screen.
  - The line (3000, 2500)–(3200, 2500) is centred, at zoom 1.
  - The line (500, 2600)–(2500, 2600) is centred at zoom 768 / 2000, with its ends 16 px
    from the left and right edges.
  - Rotated 90°, the line (100, 100)–(1100, 100) is drawn upright and fitted to the
    canvas height, at zoom 568 / 1000. The rotation stays 90°.
  - A triangle beside the canvas, whose bounding box overlaps it, counts as off screen.
- The outline of a measurement is its two ends (open); of an area, its points (closed);
  of a grid, its alignment line (open).
- Object tool bar:
  - Both buttons are disabled with no active object (and with no document open).
  - With an active measurement, area or grid, both buttons are enabled and target it by
    its current list position.
  - Duplicating a measurement, an area or a grid appends a copy equal to the original in
    every property, with a new id; the copy is the document's newest object and becomes
    active.
  - Deleting a grid keeps the order of the others. Duplicate and delete with a missing
    index are rejected and change nothing.
  - Duplicate then delete, undone twice, restores each earlier document exactly; redone
    twice, restores the final one.
  - Labels: "`Duplicate measurement 1`", "`Duplicate grid 2`", "`Delete grid 3`".
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
- Area stretching (`OBJ-9b`), with the box (10, 10)–(20, 30):
  - the right edge by 10 maps (15, 20) to (20, 20)
  - the top-left corner by (5, 10) maps (15, 20) to (17.5, 25) and keeps (20, 30)
  - the left edge by 20 mirrors (10, 10) to (30, 10)
  - a box with no height, dragged by the bottom edge by 3, moves (4, 5) to (4, 8)
- Point drags (`OBJ-15`):
  - grabbing a measurement's end 2 px off-centre and moving by (10, 5) moves the end by
    exactly (10, 5)
  - moving an area's point past the document edge clamps it; the other points stay
  - the bottom edge of the triangle (50, 50), (90, 50), (70, 90) moved up by 20 gives
    (50, 50), (90, 50), (70, 70)
  - a drag that ends where it started commits nothing
  - an active area's bounding box and points follow the drag preview
- Canvas, simulated with pointer events:
  - Dragging the right handle by (40, 25) in **every** mode commits exactly one resize,
    moving the end's x by 40 (y unchanged), with no pan.
  - Right-dragging a handle pans and does not resize.
  - Dragging an endpoint of a measurement that is not active does nothing in every
    mode.
  - Dragging a point of the active measurement or area by (40, 25) commits one move
    action for that point in every mode, with no pan.
  - Dragging the active area's right handle by 40 stretches it to the right.
  - Clicking inside the box keeps the selection, and clicking outside clears it, in every
    mode.
  - The handle positions are the 4 corners and 4 edge midpoints.
