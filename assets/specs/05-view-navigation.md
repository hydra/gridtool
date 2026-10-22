# 05 — View Navigation

The view controls how the document is displayed. All of it is presentation state: it is
not part of the document, it is not recorded as actions, and undo/redo does not affect
it (`ARCH-4`).

## View state

**VIEW-1.** The view state has three parts:
- **pan**: a 2-D offset in screen units
- **zoom**: a scale factor, in screen units per image pixel
- **rotation**: an angle in degrees

**VIEW-2 — Defaults.** Pan is `(0,0)`, zoom is `1.0` (100 %) and rotation is `0°`. Every
newly opened document starts with these defaults. A failed open leaves no document, so
there is no view to keep.

**VIEW-3 — Transform.** Let `c` be the centre of the canvas area and `p` an image point.
The point's screen position is:

```
screen(p) = c + pan + R(rotation) · ((p − size/2) · zoom)
```

- `size` is the document size.
- `R(θ)` rotates by θ in screen space, where y points down. A positive θ therefore
  appears **clockwise**: `R(θ)·(x, y) = (x·cosθ − y·sinθ, x·sinθ + y·cosθ)`.
- So the document's centre is drawn at `c + pan`.

**VIEW-3a — Rotation pivots about the document origin.** The rotation centre is always
the document's origin `o` (`ORIGIN-1`). Changing the rotation from `θ` to `θ'`, by any
route (arrow handle or text field), also adjusts the pan so that the origin stays at the
same screen position:

```
pan' = pan + (R(θ) − R(θ')) · ((o − size/2) · zoom)
```

With the default origin (the image centre) the pan does not change. Moving the origin
does not move the view, because `VIEW-3` does not depend on it. Only later rotations
pivot about the new origin.

**VIEW-4 — Inverse transform.** The inverse of `VIEW-3` maps a screen position back to
image coordinates. Mapping an image point to the screen and back MUST give the original
point to within 0.001 image px, at any pan, zoom and rotation.

**VIEW-5 — Image axes.** The view provides the screen directions of the image's X and Y
axes as unit vectors: `R(rotation)·(1,0)` and `R(rotation)·(0,1)`.

## Drawing the document

**VIEW-6.** The document is drawn as a textured quadrilateral. Its four corners are the
document corners mapped through `VIEW-3`, and the whole image is drawn across them. At
zoom 1 and rotation 0, one image pixel covers one screen unit. Anything outside the
canvas area is clipped.

**VIEW-7 — Bounding box.** A **red** (`#FF0000`) outline, 2 px wide, is drawn around the
document in every tool mode. The outline lies entirely **outside** the document and
never covers image pixels:
- the outline's centre line follows the document's corners, pushed outward by half the
  stroke width (1 px) on every side
- the 1 px is in screen units, so it is the same at every zoom level
- the outline rotates with the document

## Zoom (all tool modes)

**VIEW-8.** While the pointer is over the canvas, the mouse wheel zooms in every tool
mode. Each frame's zoom factor is:

```
factor = pinchZoom × exp(verticalScrollDelta × 0.002)
```

- `verticalScrollDelta` is the smoothed vertical scroll amount in screen units, positive
  when scrolling up or away.
- `pinchZoom` is the platform's pinch or Ctrl+wheel zoom factor, or `1` if there is
  none.

**VIEW-9 — Zoom about the cursor.** Zooming keeps the image point under the pointer
fixed on the screen. If `a` is the pointer position, the new pan is:

```
pan' = (a − c) − ((a − c − pan) × zoom'/zoom)
```

This holds in **every** tool mode, with no exceptions, including Rotate view and Origin.
Wheel zoom always zooms in on the document at the point under the cursor. In Rotate view
mode the origin and its arrow (`VIEW-13`) can therefore move on screen while zooming. The
arrow itself keeps its fixed length.

**VIEW-10 — Zoom limits.** Zoom is clamped to the range `[0.02, 1024]` (2 % – 102 400 %). The high limit lets very small grid cells be inspected (`GRID-18`).

## Pan & zoom mode

**VIEW-11.**
- Dragging anywhere on the canvas with the primary mouse button moves the pan by the
  pointer's movement, so the document follows the pointer exactly.
- The cursor is an open hand while hovering over the canvas and a closed (grabbing) hand
  while dragging.
- Panning with the *primary* button works only in this mode. In other modes the primary
  button belongs to the tool.

**VIEW-11a — Right-drag pan (all tool modes).**
- In **every** tool mode, dragging on the canvas with the **secondary** (right) mouse
  button pans the view by the pointer's movement, exactly as in `VIEW-11`.
- The cursor is a grabbing hand while right-dragging.
- A right-drag never triggers a tool action, even when it starts on a calibration point,
  a point or resize handle of the active object, or the rotate arrow. It is never
  recorded in the history.
- All tool gestures use the **primary** button only: placing points, dragging points or
  handles, and dragging the rotate arrow (`SCALE-5`, `SCALE-7`, `MEAS-6`, `AREA-2`,
  `OBJ-9`, `OBJ-15`, `VIEW-14`).
- Together with wheel zoom (`VIEW-8`), this means the view can always be navigated
  without changing tool.
- The active object's resize handles take priority over every tool's primary-button
  gestures (`OBJ-9`).

**VIEW-17 — Centre on a point.** The view can be panned so that a given image point is
drawn at the centre of the canvas, keeping zoom and rotation. From `VIEW-3`:
`pan = −R(rotation)·((p − size/2)·zoom)`. Used when an object picked in the object tree
is off screen (`OBJ-5a`).

## Rotate view mode

**VIEW-12 — Options row.** The tool options row contains:
- the label "`View rotation (°):`"
- a single-line text field 80 px wide, which shows and edits the view rotation

**VIEW-13 — Arrow handle.** The canvas shows an arrow:
- **colour** `rgb(0,170,255)`, 2 px wide
- **start** the document origin's screen position (`ORIGIN-1`), the rotation centre
- **length** **100 screen units**, fixed: the same at every zoom level, measured on screen
  and never scaled with the image. If the origin is near or beyond the canvas edge, the
  arrow is clipped there like any other canvas content.
- **direction** the view rotation: at 0° it points right, and positive angles turn it
  clockwise

A circle with a 12 px radius, drawn in the same stroke, marks the arrow tip.

**VIEW-14 — Dragging the arrow.**
- A primary-button drag that *starts* within 12 px of the arrow tip rotates the view. While dragging,
  the rotation is the direction from the origin's screen position to the pointer, so the
  arrow follows the pointer. The view pivots about the origin (`VIEW-3a`), so the
  arrow's start stays put.
- If the pointer is exactly on the origin, the rotation does not change.
- A drag that starts anywhere else does nothing.
- The cursor is an open hand while hovering within 12 px of the tip, and a grabbing hand
  while dragging the arrow.

**VIEW-15 — Normalisation.** Stored rotation is always in the range `[0, 360)` degrees:
- any input angle is wrapped into this range, so −90 becomes 270 and 450 becomes 90
- a value that would round up to exactly 360 becomes 0
- a non-finite value becomes 0

**VIEW-16 — Rotation text field.**
- While the field does **not** have keyboard focus, it shows the current rotation
  formatted by `FMT-2`.
- While it **has** focus, each edit that parses as a valid angle (`FMT-3`) is applied to
  the view immediately.
- Text that does not parse is ignored. When focus leaves, the field shows the current
  rotation again.

## Acceptance criteria

- With default view, image point `(0,0)` of a 100×50 document lies at `c + (−50,−25)`.
- Panning by `(15,−20)` moves the document centre by exactly that amount.
- These canvas input scenarios are simulated with real pointer events and no window:
  - A right-drag pans by exactly the pointer's movement in every mode, and records no
    action.
  - A right-drag that starts on a resize handle of the active object pans and leaves the
    object where it was.
  - A left-drag on a point or resize handle of the active object moves it, records one
    action, and does not pan.
  - A left-drag on empty canvas pans only in Pan & zoom mode.
  - Scrolling the wheel up zooms in, in every mode.
- Zooming by 2.5 about an arbitrary pointer keeps the image point under that pointer
  fixed.
- Zoom saturates at 1024 and 0.02.
- At 90° rotation the top-left corner of a 100×50 document is drawn at
  `c + (25, −50)`, and the image X axis points down the screen.
- Screen↔image round-trip holds at an arbitrary rotation, zoom and pan.
- With a 3 px outset at zoom 4, the corners of a 10×10 document lie 3 screen px beyond
  the scaled corners.
- Rotation normalisation: 0→0, 123.5→123.5, 360→0, 450→90, −90→270, −720→0,
  −1e−7→less than 360, NaN→0, ∞→0.
- The arrow tip is at `start + (100, 0)` at 0°, and at `start + (0, 100)` at 90°.
- The arrow tip can be grabbed at a distance of up to 12 px, and not at 13 px.
- After zooming ×8 about an off-centre point and panning, rotating about the pivot
  (80, 10) to 90°, 33° or 300° keeps the pivot's screen position.
- Canvas, simulated, with the origin at (50, 50) and zoom ×3: the arrow tip is 100 px
  right of the origin on screen. Dragging it to 100 px below the origin sets 90°, keeps
  the origin's screen position, and records no action.
- Canvas, simulated, in **every** mode (including Rotate view and Origin): scrolling to
  zoom in keeps the image point under the cursor at the cursor's screen position.
- Pointer direction maps to rotation: right gives 0, down gives π/2, left gives π.
  Pointing at the exact centre gives no rotation.
