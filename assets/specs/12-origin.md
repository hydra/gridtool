# 12 — Origin

The document has an **origin**: a point on the image that the view rotates about
(`VIEW-3a`). The user moves it in Origin mode.

## Data

**ORIGIN-1 — Origin.** The origin is a point in image coordinates (`DOC-3`). It is part
of the document (`DOC-1`), so moving it is an action and can be undone and redone
(`HIST-2`).
- **Default.** A newly created document has its origin at the centre of the image:
  `(width / 2, height / 2)`. This includes a document created by opening an image
  (`DOC-2`).
- **Always inside.** The origin is always inside the document (`DOC-4`).
- **Coordinates unchanged.** The origin does not change image coordinates. Coordinates
  shown anywhere in the UI are still measured from the image's top-left corner.

## Origin mode

**ORIGIN-2 — Tool bar button.** Origin mode has its own mode button on the main tool bar
(`SHELL-5`):
- **Icon:** a crosshair (`my_location`, `SHELL-6`).
- **Tooltip:** "`Origin`".
- **Options row:** none (`SHELL-2`).

**ORIGIN-3 — Setting the origin.** In Origin mode, a primary click (press and release
without dragging) on the canvas:
- **Inside the image:** commits one **Set origin** action (`HIST-2`) with the image point
  under the pointer. Nothing is committed if the point equals the current origin.
- **Outside the image:** does nothing.
- **Other input:** the active object's resize handles still take priority (`OBJ-9`),
  clicking outside the selection box still clears it (`OBJ-10`), and the secondary button
  and wheel still pan and zoom (`VIEW-11a`, `VIEW-8`).
- **Cursor:** a crosshair while over the canvas.

**ORIGIN-4 — Marker.** The origin is drawn **only in Origin mode**, above all document
content and below the selection box:
- **Circle.** A circle **15 px** in diameter, centred on the origin.
- **Cross.** Two lines through the origin, each **25 px** long and centred on it.
- **Stroke.** Blue (`#0000FF`), 2 px wide, for both the circle and the cross.
- **Fixed size.** All sizes are in screen units and are the same at every zoom level.
- **Image-aligned.** The cross's lines run along the **image's** X and Y axes
  (`VIEW-5`), so they turn with the view rotation and are not aligned to the screen.

## Acceptance criteria

- A new 100×50 document has its origin at (50, 25), and a 5×3 document at (2.5, 1.5).
- **Set origin** to (10, 40) moves the origin. (100.5, 10) is rejected as outside the
  document, leaving it unchanged. The label of (12, 3.5) is "`Set origin to (12.0, 3.5)`".
- Canvas, simulated with pointer events, in Origin mode:
  - A click on the image at image point (20, 10) commits exactly **Set origin (20, 10)**.
  - A click outside the image, or on the current origin, commits nothing.
- The Origin mode button has a distinct icon and name, and the icon is in the bundled
  fonts.
