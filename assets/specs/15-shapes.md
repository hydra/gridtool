# 15 — Drawn Shapes

The user draws arbitrary lines on the document by clicking their points, and may join a
line up to enclose an area. Nothing is measured from a drawn shape, and only the distance
between its fill lines depends on the scale calibration (`DRAW-10`).

## Tool

**DRAW-1 — Tool mode.** **Draw line** is a tool mode (`SHELL-8`):
- its button is the first of the drawing tools group, after the spacer that follows the
  Origin button (`SHELL-5`)
- its icon is line segments joined at points (`polyline`, `SHELL-6`)
- its tooltip is "`Draw line`"
- its controls are in the tool controls row (`DRAW-17`)

Clicking its button cancels whatever the previous tool was part-way through, as any
switch of tool mode does (`SHELL-10a`).

**DRAW-2 — Placing points.** In Draw line mode, each primary click inside the document
places the next point of a new shape, exactly as Measure area does (`AREA-2`):
- The points placed so far are presentation state (`ARCH-5`), not an action.
- Clicks outside the document are ignored, unless they land on a placed point.
- A primary drag does nothing, unless it starts on one of the active object's points or
  resize handles, or inside its selection box (`OBJ-9`, `OBJ-15`, `OBJ-16`). In particular, it does not pan.
- The points are discarded when the user presses Escape or leaves Draw line mode
  (`SHELL-10a`).

**DRAW-3 — Removing the last point.** At any time before the shape is finished:
- Pressing **Backspace** removes the most recently placed point (`SHELL-10b`).
- A primary click within 10 px of the most recently placed point removes it, unless it
  is the second click of a double-click (`DRAW-5`).

**DRAW-4 — Finishing as a closed shape.** A primary click within 10 px of the **first**
point joins the line up into a closed shape, once at least **3** points are placed. One
**Add shape** action (`HIST-2`) is committed, with the placed points in order and the line and
fill the tool's controls show (`DRAW-17`). Further
rules are as for areas (`AREA-4`): with fewer than 3 points, a click on the first point
does nothing, and with a single point the first point is also the last. When the first and
the last point are both within 10 px of the click, the nearer one counts, and on a tie the
last point counts.

**DRAW-5 — Finishing as an open line.** A line is finished, left open, once at least **2**
points are placed, by either:
- **double-clicking** the last point. The usual gesture is to double-click where the line
  should end: the first click places the final point and the second finishes the line.
  With a single point placed, a double-click on it removes it (`DRAW-3`).
- pressing **Enter** (`SHELL-10c`). With fewer than 2 points, Enter does nothing and keeps
  the point.

One **Add shape** action (`HIST-2`) is committed, with the placed points in order.

**DRAW-6 — After finishing.** The placed points are cleared, ready for the next shape, and
the new shape becomes the active object (`OBJ-7`), so that its controls
(`DRAW-13`) are shown straight away.

**DRAW-7 — Cursor.** While hovering over the canvas in Draw line mode, the cursor is as
for Measure area (`AREA-5`).

## Data

**DRAW-8 — Drawn shape.** A drawn shape consists of:
- **points**: in image coordinates (`DOC-3`), in order: 2 or more for an open shape, or 3
  or more for a closed one.
- **closed**: whether the last point joins the first, so that the shape encloses an area.
  A shape's closed state never changes after it is added.
- **line style**: Thin, Thick or Dashed. Thin for a new shape.
- **line colour**: orange (`#FF8000`) for a new shape.
- **line opacity**: 0 to 1. 1 (100 %) for a new shape.
- **fill style**: None, Solid, Lines or Crosshatch. None for a new shape, and always None
  for an open shape.
- **fill colour**: a colour, or none. None for a new shape. A fill with no colour is drawn in
  the shape's current line colour, so it changes along with the line colour.
- **fill spacing**: the real-world distance between neighbouring fill lines, in the fill
  unit, > 0. 1 for a new shape.
- **fill unit**: cm or m. The calibration's unit (`SCALE-20`) when the shape is added.
- **fill angle**: the real-world direction of the fill lines, in degrees clockwise from
  vertical, from 0 up to but not including 180, so 0 is vertical and 90 is horizontal. A shape
  stores one fill angle, for its current fill style. It starts at 90 for Lines (and for None and
  Solid) and 45 for Crosshatch, unless the tool says otherwise (`DRAW-20`).
- **fill opacity**: 0 to 1. 0.5 (50 %) for a new shape.

An open shape keeps its fill properties, but they have no effect.

The document holds an ordered list of shapes, separate from its other objects. New shapes
are added at the end, and the list is never sorted. Deleting a shape moves every later
shape up one place. Shapes are numbered from 1 in list order in the UI. Each shape is an
object with its own object id (`OBJ-1`).

The tool's controls can give a new shape other values (`DRAW-17`).

**DRAW-9 — Centre.** A closed shape's centre is the centroid of the enclosed area
(`AREA-8`). The centre of an open shape, or of a closed one enclosing no area, is the
centre of its bounding box (`OBJ-2`).

**DRAW-10 — Fill lines.** The fill lines of a closed shape are laid out in real-world space
(image coordinates multiplied by the calibrated X and Y scale, `SCALE-20`), as a grid's are
(`GRID-3`), so their distance and angle are true in the real world even when the two axes
have different scales. They are fixed to the document, so they turn and scale with the view:
- **Scale needed.** Without a calibrated scale, no fill lines can be laid out and none are
  drawn. A Solid fill does not need a scale.
- **Directions.** Lines has one set of parallel lines, at the fill angle. Crosshatch has
  two: one at the fill angle and one perpendicular to it. None and Solid have no fill lines.
- **Positions.** The lines of a set lie at every whole multiple of the fill spacing,
  measured perpendicular to the lines from the image's top-left corner. They therefore stay
  in place when the shape's points move, and line up between shapes with the same spacing,
  unit and angle.
- **Inside.** Each line is cut to the parts inside the outline, by the even-odd rule. The
  crossings of the line with the outline are sorted along it and paired in order; each
  pair is one segment. An edge counts as crossed when exactly one of its ends is beyond
  the line's position (strictly greater), so a line through a corner counts it
  consistently. Pairs of equal position, where a line only touches a corner, are dropped.

## Drawing

**DRAW-11 — Shapes on the canvas.** Shapes are drawn in **every** tool mode, above the
grids and below the area measurements (`GRID-6`), in list order, with their fill below
their line:
- **Line.** Through the points in order, and from the last back to the first if closed,
  in the line colour at the line opacity. Widths are in screen px, the same at every zoom:

  | Line style | Appearance |
  |---|---|
  | Thin | solid, 1 px |
  | Thick | solid, 3 px |
  | Dashed | 2 px, with 6 px dashes and 4 px gaps. The dash pattern starts afresh at each point. |

- **Fill.** Only for closed shapes, in the fill colour (or, with none, the line colour) at the
  fill opacity:
  - **None:** nothing.
  - **Solid:** the enclosed area, split into triangles as for areas (`AREA-9`).
  - **Lines, Crosshatch:** the fill lines (`DRAW-10`), 1 px wide. If neighbouring lines of a
    set would be closer than 3 screen px, only every `n`-th line of that set is drawn, for
    the smallest whole `n` that keeps them at least 3 px apart. Only lines inside the
    visible part of the canvas are generated.
- **Point markers.** Only the **active** shape shows a filled circle with a 5 px radius at
  each point, in its line colour. Other shapes show only what was drawn.

The active shape also shows its selection box (`OBJ-8`). Shapes have no labels.

An implementation MUST NOT generate dashes or fill lines for the parts of a shape outside
the canvas, since at high zoom one edge can be millions of screen px long. Dashes MUST
stay in the same place on an edge as the view pans.

**DRAW-12 — The shape being placed.** In Draw line mode, drawn as the outline being placed
is for areas (`AREA-10`), with the line and fill the tool's controls show (`DRAW-17`): point
markers, a line through the points, a 2 px dashed line on to the pointer, and a 9 px ring
around the first point once it can be closed. While the pointer is over the first point and
the shape can be closed, the dashed line is replaced by the closed outline and its fill, as
the shape will be drawn.

## Tool controls

**DRAW-13 — Controls of the active shape.** While the active object is a shape, the tool
controls row (`SHELL-2a`) shows these, left to right, wrapping if the window is too narrow:

1. The shape's label in strong text (`DRAW-15`), then a separator.
2. "`Line:`", then one selectable button per line style: "`Thin`", "`Thick`", "`Dashed`",
   with the current one selected.
3. "`Line color:`", then a 36 × 18 px swatch of the line colour, with the tooltip
   "`Choose the colour of the line`".
4. "`Line opacity:`", then a 0–100 % opacity slider (`DRAW-14`).

For a **closed** shape only, these follow, after a separator:

5. "`Fill:`", then one selectable button per fill style: "`None`", "`Solid`", "`Lines`",
   "`Crosshatch`", with the current one selected.
6. For the Lines and Crosshatch styles only:
   - "`Distance:`", then a 60 px commit-on-blur field (`FMT-9`) of the fill spacing,
     written as a grid's cell size is (`GRID-14`), with the tooltip
     "`Distance between the fill lines`", then a drop-down of the fill unit ("`cm`", "`m`"),
     with the tooltip "`Units of the distance between the fill lines`".
   - Two icon buttons that turn the fill lines a quarter turn: rotate 90° left
     (`rotate_90_degrees_ccw`, tooltip "`Rotate the fill lines 90° left`") and rotate 90°
     right (`rotate_90_degrees_cw`, tooltip "`Rotate the fill lines 90° right`").
   - "`Angle:`", then a 45 px commit-on-blur field of the fill angle, then "`°`", with the
     tooltip "`Angle of the fill lines, in degrees clockwise from vertical`".
7. For every style except None: "`Fill color:`", then a padlock (`DRAW-19`), then a
   36 × 18 px swatch of the fill colour, with the tooltip "`Choose the colour of the fill`",
   then "`Fill opacity:`" and a 0–100 % opacity slider (`DRAW-14`).
8. For the Lines and Crosshatch styles while there is no calibrated scale, a separator and
   the weak text "`Fill lines not shown: they need a scale (<reason>)`" (`DRAW-10`), with the
   reason as for a grid (`GRID-4`).

**DRAW-14 — Committing changes.** For an object, each change is one **Set style** action
(`DRAW-18`):
- Clicking a style button other than the current one commits one action. Clicking the current
  one does nothing.
- A colour swatch works as the grid's does (`GRID-15`): the colour previews live on the
  canvas while its picker is open, and one action is committed when it closes on a different
  colour.
- Ending an edit of the distance field commits one action if the text is a number > 0 that
  differs from the current value. Anything else reverts the field.
- Picking a different unit commits one action. The distance value is kept as it is and is not
  converted, as for a grid (`GRID-14`).
- Clicking either rotate button commits one action, adding or subtracting 90° and then keeping
  the result from 0 up to 180. Lines repeat every half turn, so both buttons give the same
  result, and on a Crosshatch they change the stored angle without changing what is drawn.
- Ending an edit of the angle field commits one action if the text is a number whose
  equivalent from 0 up to 180 differs from the current value (for example, `225` and `-135`
  both mean 45). Anything else reverts the field.
- An opacity slider works as the grid's does (`GRID-16`): dragging previews live on the
  canvas and commits one action on release, and clicks and keyboard changes commit
  immediately. The percentage is rounded to a whole number. Nothing is committed if the value
  is unchanged.

**DRAW-19 — Fill colour padlock.** A selectable icon button directly before the fill colour
swatch:
- **No fill colour.** The padlock is selected and shows a closed lock (`lock`), with the tooltip
  "`The fill uses the line's colour. Click to choose its own`". The swatch is disabled, shows
  the line colour, and has the tooltip "`Click the padlock to choose the colour of the fill`".
- **A fill colour.** The padlock is not selected and shows an open lock (`lock_open`), with the
  tooltip "`The fill has its own colour. Click to use the line's colour, forgetting this one`".
  The swatch can be used (`DRAW-14`).
- **Clicking it** with no fill colour sets the fill colour to the line's current colour.
  Clicking it with a fill colour sets it back to none, forgetting the colour picked. Each click
  commits one action (`DRAW-14`), so undo can restore a forgotten colour.

**DRAW-20 — Remembered fill angles.** The Draw line and Measure area tools each remember one
fill angle per fill style with lines, as presentation state (`ARCH-5`): it is not an action,
cannot be undone and is not saved. They last as long as the document is open. Each starts at the
style's default: 90 for Lines and 45 for Crosshatch.
- **Setting an angle.** Setting the fill angle, in the tool's controls or those of an object
  belonging to the tool (`OBJ-7a`), remembers it for the current fill style.
- **Existing objects.** When an object becomes active, its tool remembers the object's fill angle
  for the object's fill style. A None or Solid fill has nothing to remember.
- **Changing fill style.** Picking Lines or Crosshatch also changes the fill angle to the one
  remembered for that style, in one action for an active object, so the angle field shows it at
  once. Picking None or Solid leaves the angle as it is.

**DRAW-17 — Controls of the Draw line tool.** With no active object, in Draw line mode, the
tool controls row (`SHELL-2a`) shows "`New shape`" in strong text, a separator, and the line
and fill controls of `DRAW-13` as for a closed shape. They work as the Measure distance tool's
do (`MEAS-17`), and the fill unit follows the calibration unit until the user picks one here.
The fill applies only if the shape ends up closed; an open line is left with no fill.

**DRAW-18 — Set style.** The **Set style** action (`HIST-2`) changes one value of how a
measurement, area or drawn shape is drawn. Its label ends with the change:

| Change | Label ending |
|---|---|
| Line style | "`line style to <Thin\|Thick\|Dashed>`" |
| Line colour | "`line color to <#RRGGBB>`", in upper-case hex |
| Line opacity | "`line opacity to <percent>%`", rounded to a whole number |
| Fill style, with the fill angle that goes with it (`DRAW-20`) | "`fill style to <None\|Solid\|Lines\|Crosshatch>`" |
| Fill colour | "`fill color to <#RRGGBB>`", in upper-case hex, or "`fill color to none`" |
| Fill spacing | "`fill spacing to <value>`" |
| Fill unit | "`fill units to <cm\|m>`", keeping the spacing value |
| Fill angle | "`fill angle to <degrees>°`" |
| Fill opacity | "`fill opacity to <percent>%`", rounded to a whole number |

## Object tree

**DRAW-15 — Tree leaves.** Each shape is a leaf in the "`Shapes`" folder (`OBJ-4`), labelled
"`Shape <n> (line)`" for an open shape or "`Shape <n> (area)`" for a closed one.

## Editing

**DRAW-16 — Editing a shape.** A shape is an object (`OBJ`), so:
- picking it in the object tree makes it the active object and, if none of it is on
  screen, pans the view to its centre and zooms out if it does not fit (`OBJ-5`,
  `OBJ-5a`, `DRAW-9`)
- while active, it shows a selection box whose handles stretch every point as for an area
  (`OBJ-8`, `OBJ-9`, `OBJ-9b`), each of its points can be dragged (`OBJ-15`), and dragging inside its box moves it (`OBJ-16`)
- it can be duplicated and deleted with the object tool bar (`OBJ-12`, `OBJ-13`)
- it is saved and loaded with the document (`FILE-1`, `FILE-6b`)

Every change, including adding a shape and every style change, is an action, so it can be
undone and redone (`HIST-9`).

## Acceptance criteria

- A line needs at least 2 points and a closed shape at least 3. A new shape is Thin, orange
  at 100 % line opacity, with no fill, no fill colour, spacing 1 in the given unit,
  angle 90 and 50 % fill opacity.
- Fill angles: 180 becomes 0, 270 becomes 90, -90 becomes 90, -30 becomes 150, and a tiny
  negative angle becomes 0. 180, -1 and NaN are not valid stored angles.
- Centre (`DRAW-9`): the open shape (0, 0), (6, 0), (0, 3) has its centre at (3, 1.5), and
  the same points closed at (2, 1). A closed shape through (0, 0), (2, 2), (4, 4) has its
  centre at (2, 2).
- Fill lines (`DRAW-10`), at 1 cm per pixel unless stated:
  - The square (5, 5)–(25, 25) with spacing 10 has vertical lines at x = 10 and x = 20, each
    from y = 5 to y = 25; with spacing 20 it has one horizontal line, at y = 20.
  - A U shape, 30 wide and 20 tall with a notch from x = 10 to 20 down to y = 10, gives two
    segments at y = 5 ((0, 5)–(10, 5) and (20, 5)–(30, 5)) and one at y = 15.
  - The diamond (10, 0), (20, 10), (10, 20), (0, 10) with spacing 10 has one horizontal
    segment, (0, 10)–(20, 10): the lines touching its corners at y = 0 and y = 20 add none.
  - Only lines through the given region are generated.
  - The rectangle (10, 20)–(50, 40) with spacing 10 has 2 lines at angle 90, 4 at angle 0
    and 6 for Crosshatch; None, Solid and an open shape have none.
  - At 45°, the lines of a square are a whole number of spacings from the top-left corner,
    measured across them. A Crosshatch at 30° has lines at 30° and 120°.
  - At 2 cm per pixel, spacing 10 cm is 5 px apart, and spacing 0.2 m is 10 px apart.
  - At 0.5 cm per pixel across and 2 cm per pixel down, 10 cm is 5 px apart at angle 90 and
    20 px apart at angle 0, and lines at 45° are 45° and 10 cm apart in real-world space.
- Fill line thinning (`DRAW-11`): lines 10 px apart with a 10 px minimum gap are all kept;
  with a 15 px minimum, every second one is.
- The next shape gets the line and, only if it ends up closed, the fill chosen in the tool's
  controls.
- Actions:
  - Adding appends in order, with ids distinct from other objects' ids. Adding with a
    point outside the document is rejected.
  - Moving a point changes only that point. Resizing replaces every point and keeps the
    id, the closed state and the style.
  - Each style can be set, and changes only that shape.
  - Changing the fill unit keeps the spacing value.
  - These are rejected and leave the document unchanged: a point number that does not
    exist, a position outside the document, a resize with a different number of points, an
    opacity outside [0, 1], a fill spacing of 0, a fill angle of 180 or -1, any fill change
    on an open shape, and any shape action with an index that does not exist.
  - Duplicating copies every property with a new id, and the copy becomes the newest
    object. Deleting the middle of three shapes leaves the first and third, in order.
  - Duplicate and delete are found for a shape by its id. Adding or duplicating a shape
    activates it.
  - Add, move, resize, all eight style changes, duplicate and delete, performed in turn and
    then each undone and redone, pass through exactly the same document states.
  - Labels: "`Add line with 3 points`", "`Add closed shape with 3 points`",
    "`Move shape 1 point 3 to (1.0, 2.0)`", "`Resize shape 2`",
    "`Set shape 1 line style to Dashed`", "`Set shape 1 line color to #FF8000`",
    "`Set shape 1 line opacity to 25%`", "`Set shape 2 fill style to Lines`",
    "`Set shape 2 fill color to #FF8000`", "`Set shape 2 fill color to none`",
    "`Set shape 2 fill spacing to 50`",
    "`Set shape 2 fill units to m`", "`Set shape 2 fill angle to 22.5°`",
    "`Set shape 2 fill opacity to 50%`", "`Delete shape 3`", "`Duplicate shape 1`".
- Controls (`DRAW-13`, `DRAW-14`):
  - With no fill colour, a click on the padlock commits the line colour as the fill colour, and
    a click on the swatch opens nothing. With a fill colour, a click on the padlock commits none,
    and a click on the swatch opens the picker (`DRAW-19`).
  - A typed distance must be a new number > 0: `0`, `-1`, text and the current value commit
    nothing.
  - A typed angle of `225` commits 45, `-45` commits 135, and `180` at angle 0, or text,
    commits nothing.
  - Clicking along the row of a Lines fill reaches both rotate buttons, each committing angle
    0 from angle 90. A Crosshatch has them too, and a Solid fill does not.
  - From angle 120, either rotate button commits angle 30.
- Remembered fill angles (`DRAW-20`):
  - A new Lines fill is at 90, and a new Crosshatch at 45.
  - Changing an active object to Crosshatch commits the angle last set for Crosshatch, or 45 if
    none has been set. Solid or None keeps the angle it has.
  - Setting an angle for one style leaves the other's alone. An object newly active sets the
    value for its own style only, in its own tool only.
- Tool (`DRAW-2` – `DRAW-5`), simulated with pointer events:
  - Three clicks, then a click 5 px from the first point, commit exactly one Add shape
    action, closed, with the three points in order, and clear the placed points.
  - Two clicks with the second one repeated quickly commit one Add shape action, open, with
    the two points. Repeated slowly, the second click instead removes the point.
  - Finishing with Enter adds an open shape; with one point it does nothing and keeps it.
  - Backspace removes the last point, and Escape discards them all.
  - Enter in a focused text field does not finish the line.
- Editing, simulated with pointer events: dragging a point of the active shape by (40, 25)
  commits one Move shape point action for that point in **every** mode, with no pan.
  Dragging its right handle stretches it. Clicking inside its box keeps it active, and
  clicking outside clears it.
- Painting every combination of line and fill style at zooms up to 1024 and down to the
  minimum, with and without a scale, completes without error.
- The tree's "`Shapes`" folder is a non-object node. Its leaves are labelled
  "`Shape 1 (line)`" and "`Shape 3 (area)`".
- With an active shape, both object tool bar buttons are enabled and target it by its
  current list position.
- In the tool bar, Draw line comes directly after Origin, and it is the only mode that
  starts a new group.
