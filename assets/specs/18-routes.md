# 18 — Route Measurements

The user measures a path that is not straight, such as a boundary, a wall or a cable run, by
clicking the points along it. A route is a series of straight segments, from each point to
the next. Its total length and each segment's length are calculated from the document's
scale calibration (`SCALE-17`). A route is never closed: to measure all the way round, the
user clicks back on the first point, which adds a point there.

## Tool

**ROUTE-1 — Tool mode.** **Measure route** is a tool mode (`SHELL-8`):
- its button comes directly after the Measure area button (`SHELL-5`)
- its icon is a path between two points (`route`, `SHELL-6`)
- its tooltip is "`Measure route`"
- its controls are in the tool controls row (`ROUTE-17`)

Clicking its button cancels whatever the previous tool was part-way through, as any switch of
tool mode does (`SHELL-10a`).

**ROUTE-2 — Placing points.** In Measure route mode, each primary click inside the document
places the next point of a new route, as Measure area does (`AREA-2`):
- The points placed so far are presentation state (`ARCH-5`), not an action.
- Clicks outside the document are ignored, unless they land on the last placed point.
- A primary drag does nothing, unless it starts on one of the active object's points or
  resize handles, or inside its selection box (`OBJ-9`, `OBJ-15`, `OBJ-16`). In particular,
  it does not pan.
- The points are discarded when the user presses Escape or leaves Measure route mode
  (`SHELL-10a`).
- A click on the **first** point places the next point there, like a click anywhere else. It
  does not finish or close the route.

**ROUTE-3 — Removing the last point.** At any time before the route is finished:
- Pressing **Backspace** removes the most recently placed point (`SHELL-10b`).
- A primary click within 10 px of the most recently placed point removes it, unless it is the
  second click of a double-click (`ROUTE-4`).

**ROUTE-4 — Finishing.** A route is finished once at least **2** points are placed, by either:
- **double-clicking** the last point. The usual gesture is to double-click where the route
  should end: the first click places the final point and the second finishes the route. With
  a single point placed, a double-click on it removes it (`ROUTE-3`).
- pressing **Enter** (`SHELL-10c`). With fewer than 2 points, Enter does nothing and keeps the
  point.

Then:
1. One **Add route** action (`HIST-2`) is committed, with the placed points in order and the
   unit and line the tool's controls show (`ROUTE-17`).
2. The placed points are cleared, ready for the next route.
3. The new route becomes the active object (`OBJ-7a`), so that its controls (`ROUTE-16`) and
   its tree leaf are selected straight away.

When the first and the last point are both within 10 px of a click, the last point counts.

**ROUTE-5 — Cursor.** While hovering over the canvas in Measure route mode, the cursor is:
- a pointing hand over the last point
- a crosshair elsewhere, including over the first point
- as `OBJ-17` says over the active object's selection box, points and resize handles

## Data

**ROUTE-6 — Route.** A route consists of:
- **points**: 2 or more points in image coordinates (`DOC-3`), in order. Segment `k` runs from
  point `k` to point `k + 1`, so a route with `n` points has `n − 1` segments, and each point
  but the first and the last is the end of one segment and the start of the next. Points may
  repeat; a segment from a point to the same place has length 0.
- **unit**: the length unit its lengths are reported in (`SCALE-2`). It is independent of the
  calibration unit and of other objects.
- **line style**, **line colour** and **line opacity**, as for a drawn shape (`DRAW-8`):
  Thick, **magenta** (`#FF00FF`) and 100 % for a new route.

A route has no fill. The tool's controls can give a new route other values (`ROUTE-17`).

The document holds an ordered list of routes, separate from the lists of measurements and
areas. New routes are added at the end, and the list is never sorted. Deleting a route moves
every later route up one place. Routes are numbered from 1 in list order in the UI. Each
route is an object with its own object id (`OBJ-1`).

**ROUTE-7 — Calculated lengths.** A route's lengths are derived, never stored, so they always
reflect the current calibration:
1. Each segment's length is calculated as a measurement's distance is (`MEAS-4`), from its
   two points, in the route's unit. If the scale is unavailable, every length is
   unavailable, for the same reason (`SCALE-18`).
2. The route's **total length** is the sum of its segments' lengths.

**ROUTE-8 — Centre.** A route's centre is the centre of its bounding box (`OBJ-2`). It is where
the view pans to when the route is picked in the object tree while it is off screen (`OBJ-5`,
`OBJ-5a`). For that test the route is an open line: it is on screen if any point or part of a
segment is.

## Drawing

**ROUTE-9 — Routes on the canvas.** Routes are drawn in **every** tool mode, above the areas and
below the point-to-point measurements (`GRID-6`), each in its line colour:
- **Line.** An open line through the points in order, in its line style, colour and opacity,
  drawn as a drawn shape's line is (`DRAW-11`). Only the line uses the opacity.
- **Point markers.** A filled circle with a 5 px radius at each point.
- **Segment labels.** Each segment's length, formatted by `FMT-7` and without a unit, placed
  as a measurement's label is (`MEAS-9`): centred at the segment's midpoint and offset 12 px
  along its perpendicular, in a 14 px proportional font. They are left out when the lengths
  are unavailable.
- **Total label.** "`<n>: <total> <unit symbol>`", with the total formatted by `FMT-7`, or just
  "`<n>`" when the lengths are unavailable, in a 14 px proportional font. It is placed just
  beyond the last point, 12 px on in the direction of the last segment, and aligned so that it
  reads on from the line instead of covering it: for example, starting at that spot when the
  last segment runs to the right, and ending at it when it runs to the left.

The active route also shows its selection box (`OBJ-8`).

**ROUTE-10 — The route being placed.** In Measure route mode, with the line the tool's controls
show (`ROUTE-17`):
- each placed point as a point marker, in the line colour
- a line through the placed points, in order, in the line style, colour and opacity
- while the pointer is over the canvas, a 2 px dashed line in the line colour (6 px dashes,
  4 px gaps) from the last point to the pointer

There is no ring around the first point and no closed preview, since a route is never closed.

## Object tree

**ROUTE-11 — Tree leaves.** Each route is a leaf in the "`Routes`" folder inside
"`Measurements`", after the "`Area`" folder (`OBJ-4`), labelled
"`Route <n> (<total> <unit symbol>)`", with the total formatted by `FMT-7`, or "`Route <n>`"
when the lengths are unavailable.

## Routes table

**ROUTE-12 — Location.** The table is the Routes panel (`WORK-1`), showing the active document.
- With no document open, the panel shows "`No document open`" in weak text.
- With a document that has no routes, it shows
  "`No routes yet. Use the Measure route tool to add one.`" in weak text.

**ROUTE-13 — Table.** The table scrolls vertically and horizontally and has striped rows. It has
one header row followed, for each route in list order (`ROUTE-6`), by one **route row** and,
while that route is open (`ROUTE-14`), one **segment row** per segment, in order:

| Column header | Route row | Segment row |
|---|---|---|
| "`#`" | An open/close toggle (`ROUTE-14`) and the 1-based route number | "`<route>.<segment>`" in weak text, both 1-based, e.g. "`2.3`" |
| "`Color`" | A colour swatch, 28 × 14 px, filled with the route's line colour, with the tooltip "`Choose the line colour of this route`" (`ROUTE-15`) | (blank) |
| "`x1`", "`y1`" | (blank) | Commit-on-blur fields (`FMT-9`) for the segment's start point, formatted by `FMT-6` |
| "`x2`", "`y2`" | (blank) | Commit-on-blur fields for the segment's end point |
| "`Length`" | The total length (`ROUTE-7`) | The segment's length |
| "`Units`" | A drop-down of length units, as in the Point-to-point table, with the tooltip "`Units of this route`" (`ROUTE-15`) | (blank) |
| (blank) | A delete button with a trash-can icon (`delete`, `SHELL-6`) at the inline icon size, and the tooltip "`Delete route`" (`ROUTE-15`) | (blank) |

Lengths are formatted by `FMT-7` and without a unit, or shown as a weak "`—`" if unavailable.
Hovering the "`—`" shows "`Scale: <reason>`" (`SCALE-18`). The coordinate fields share the
table's spare width as the Point-to-point table's do (`MEAS-11`).

While a route's points or resize handles are dragged (`OBJ-9`, `OBJ-15`), its rows show the
live points and lengths.

**ROUTE-14 — Opening and closing routes.** Each route row starts **closed**, showing no segment
rows. Its toggle is a right-pointing chevron (`chevron_right`) with the tooltip
"`Show the segments of this route`" while closed, and a down-pointing chevron (`expand_more`)
with the tooltip "`Hide the segments of this route`" while open; clicking it opens or closes
the route. Which routes are open is presentation state, remembered separately for each open
document while the application runs, and is not saved.

**ROUTE-15 — Editing in the table.**
- **Coordinates.** Committing a coordinate field with a number that puts the point inside
  the document, and changes it, commits one **Move route point** action for that point. The
  end of segment `k` is the start of segment `k + 1`, so editing either moves both. Any other
  text, or a point outside the document, commits nothing, and the field shows the stored
  value again (`FMT-9`).
- **Unit.** Choosing a unit different from the route's current unit commits one
  **Set route unit** action. The calibration unit and other objects are not affected.
- **Colour.** The colour swatch works as for measurements (`MEAS-15`). When the picker closes
  on a different colour, one **Set style** action is committed for that route.
- **Deleting.** Clicking the delete button commits one **Delete route** action. The click does
  not make the route active.

**ROUTE-15a — Picking rows.** The row of the active route is selected.
- Clicking a **route row** makes its route active and brings it into view as picking it in the
  object tree does (`OBJ-6a`, `ROUTE-8`).
- Clicking a **segment row** makes its route active. Then, unless both ends of the segment are
  inside the canvas, the view pans so that the middle of the segment's bounding box is at the
  centre of the canvas (`VIEW-17`) and, if either end is then closer than 16 px to an edge
  of the canvas, or beyond it, zooms out about the centre as `OBJ-5a` step 3 does. The
  rotation is never changed. A segment that is already wholly on screen leaves the view
  alone.

## Editing

**ROUTE-16 — Editing a route.** A route is an object (`OBJ`), so:
- picking it in the object tree makes it the active object and brings it into view
  (`ROUTE-8`)
- while active, it shows a selection box whose handles stretch every point (`OBJ-8`,
  `OBJ-9`, `OBJ-9b`), each of its points can be dragged (`OBJ-15`), and dragging inside its
  box moves it (`OBJ-16`). Dragging a point commits one **Move route point** action, a
  resize handle one **Resize route** action, and the box one **Move route** action.
- it can be duplicated and deleted with the object tool bar (`OBJ-12`, `OBJ-13`)
- it is saved and loaded with the document (`FILE-1`, `FILE-6h`)

While the active object is a route, the tool controls row (`SHELL-2a`) shows, left to right,
wrapping if the window is too narrow:
1. "`Route <n>`" in strong text, then a separator.
2. A drop-down of length units, with the tooltip "`Units of this route`". Picking a different
   one commits one **Set route unit** action.
3. A separator, then the line controls of `DRAW-13` (style, colour, opacity), which commit as
   `DRAW-14` says. There are no fill controls.

**ROUTE-17 — Controls of the Measure route tool.** With no active object, in Measure route mode,
the row shows:
1. "`New route`" in strong text, then a separator.
2. A drop-down of length units, with the tooltip "`Units of new routes`".
3. A separator, then the same line controls as `ROUTE-16`.

They work as the Measure distance tool's do (`MEAS-17`). The unit follows the calibration unit
until the user picks one here.

## Acceptance criteria

- A route needs at least 2 points. A new route has a Thick magenta line.
- Calculation (`ROUTE-7`): at 2 px to the centimetre, the route (10, 10), (40, 10), (40, 50) has
  segments of 15 cm and 20 cm, and a total of 35 cm. Without a scale the lengths are
  unavailable.
- Actions:
  - Adding appends in order, with ids distinct from other objects' ids. Adding with a point
    outside the document is rejected.
  - Moving a point moves the end of one segment and the start of the next.
  - Moving and resizing replace every point and keep the id and unit; setting the unit keeps
    the points.
  - A route's line can be restyled; setting a fill on it is rejected and changes nothing.
  - These are rejected and leave the document unchanged: a point number that does not exist, a
    position outside the document, a resize with a different number of points, and any route
    action with an index that does not exist.
  - Duplicating copies every property with a new id, and the copy becomes the newest object
    and is activated. Duplicate and delete are found for a route by its id.
  - Every route action can be undone and redone.
  - The route's outline is open and its focus is the centre of its bounding box.
  - Labels: "`Add route with 3 points`", "`Move route 1 point 2 to (3.0, 4.0)`",
    "`Resize route 2`", "`Move route 2`", "`Set route 1 units to m`", "`Delete route 3`",
    "`Duplicate route 1`", "`Set route 1 line style to Dashed`".
- Tool (`ROUTE-2` – `ROUTE-4`), simulated with pointer events where they apply:
  - Clicking three points, then the first point, places a fourth point there, and a
    double-click on it then commits exactly one Add route action with the four points in
    order, and clears the placed points.
  - A slow second click on the last point removes it. A double-click on a lone point removes it.
  - Enter finishes with 2 or more points and does nothing, keeping the point, with 1.
  - The new route gets the unit and line chosen in the tool's controls.
  - Clicking inside the active object's selection box in Measure route mode starts a new route
    and clears the active object.
- Table (`ROUTE-13` – `ROUTE-15a`):
  - The columns are "`#`", "`Color`", "`x1`", "`y1`", "`x2`", "`y2`", "`Length`", "`Units`" and a
    blank one.
  - Editing the y of segment 1's end of the route (10, 10), (20, 10), (20, 30) to " 15 " commits
    Move route point for point 2 to (20, 15). Text that is not a number, a point outside the
    document, an unchanged value, and a point or route that does not exist commit nothing.
  - Routes start closed and stay open once opened. The selected row of a route counts the
    segment rows of the open routes before it.
  - A route row picks its route, a segment row picks the segment, and clicking delete picks
    nothing.
  - Picking a segment wholly on screen leaves the view alone; picking one partly off screen
    centres it and zooms out until both ends are 16 px inside the canvas.
- Files: see `FILE-6h`.
- In the tool bar, Measure route comes directly after Measure area, and Grid directly after
  Measure route. A route makes the Measure route tool active.
