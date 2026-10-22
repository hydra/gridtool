# 04 — Document Model

## Requirements

**DOC-1 — Contents.** A document consists of:
- **size**: a width and height in whole image pixels
- **calibration**: the scale calibration data (see `SCALE-20`)
- **measurements**: an ordered list of distance measurements (see `MEAS-1`, `MEAS-2`).
  Their distances are derived from the calibration, not stored (`MEAS-3`).
- **areas**: an ordered list of area measurements (see `AREA-6`). Their areas are derived
  from the calibration, not stored (`AREA-7`).
- **grids**: an ordered list of grids, with all their display properties (see `GRID-1`,
  `GRID-2`).
- **origin**: a point inside the document that the view rotates about. It defaults to the
  image centre (`ORIGIN-1`).
- **object id counter**: the next object id to allocate. Every object carries a stable
  id (`OBJ-1`).

View state (pan, zoom, rotation) is **not** part of the document (`ARCH-4`).

**DOC-2 — Initial size.** A document created from an image has the same width and height
in pixels as the image.

**DOC-3 — Coordinate system.** Positions in a document are in image coordinates: origin
at the top-left of the un-rotated image, x to the right, y downwards, in image pixels.
Values may be fractional.

**DOC-4 — Bounds.** A point is *inside* the document when `0 ≤ x ≤ width` and
`0 ≤ y ≤ height`. The edges count as inside.

**DOC-5 — Clamping.** The document provides a clamp operation. It returns the nearest
point inside the document by clamping x to `[0, width]` and y to `[0, height]`
separately.

**DOC-6 — Mutation.** The document changes only by applying actions (`HIST-1`). Its
fields are not otherwise writable from outside the core.

**DOC-7 — Value semantics.** Two documents can be compared for equality, and a document
can be copied. The history uses both of these (`HIST-4`, `HIST-5`).

## Acceptance criteria

- A document created from a 640 × 480 image is 640 × 480.
- `(0,0)`, `(width,height)` and interior points are inside. `(-0.1, y)` and
  `(x, height+0.1)` are outside.
- Clamping `(-5, 70)` on a 100 × 50 document gives `(0, 50)`. Clamping a point that is
  already inside returns it unchanged.
