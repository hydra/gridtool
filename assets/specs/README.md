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

## Conventions

- Each requirement has a stable ID such as `SCALE-12`. IDs are never reused. A
  requirement that is removed is marked **Withdrawn** and keeps its ID.
- **MUST** means the requirement is mandatory. **SHOULD** means it is recommended.
  **MAY** means it is optional.
- Lengths written as "px" or "points" are **logical screen units** (device-independent
  pixels), unless a requirement says they are *image pixels*.
- Text in `code style` that appears inside quotation marks must be reproduced exactly.
- Icons are described by meaning. The current glyph is given for reference, and any
  icon with the same meaning is acceptable.
- Each area ends with **acceptance criteria**. These are concrete scenarios that an
  implementation's automated tests must cover.

## Glossary

| Term | Meaning |
|---|---|
| **Document** | The thing being edited: an image-sized rectangle plus its calibration data and measurements. |
| **Image coordinates** | Pixel coordinates on the un-rotated image. The origin is at the top-left corner, x increases to the right and y increases downwards. Values may be fractional. |
| **Screen coordinates** | Positions in the application window, in logical screen units. |
| **View** | How the document is displayed: pan, zoom and rotation. View state is never part of the document. |
| **Action** | A single change to the document. It can be undone and redone. |
| **Tool mode** | The active tool. It decides what mouse input on the canvas does. |
| **Calibration points** | Points placed on the image that define two reference lines, A and B. |
| **Scale** | The real-world length of one image pixel along the image's X axis and along its Y axis. |
| **Measurement** | A line on the document, with a start and an end point, whose real-world length is derived from the scale. |

## Change log

| Date | Change |
|---|---|
| 2026-10-21 | Initial specification: image loading, view navigation, scale calibration, undo/redo. |
| 2026-10-21 | Added distance measurements (`09`), Measure distance tool, split side panel (`SHELL-11`), measurement actions (`HIST-2`), formats `FMT-6`–`FMT-8`, and commit-on-blur fields (`FMT-9`). |
| 2026-10-21 | Added icon verification (`SHELL-6a`) and the delete-button glyph fallback (`MEAS-11`). |
| 2026-10-21 | Added right-drag pan in all modes (`VIEW-11a`); tool gestures are now primary-button only. |
