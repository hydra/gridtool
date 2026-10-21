# 01 — Architecture

## Requirements

**ARCH-1 — Headless core.** All document data and every operation that changes a
document MUST live in a core component. That component MUST have no dependency on any
UI toolkit, windowing system or native dialog.

**ARCH-2 — Presentation is separate.** Presentation code (drawing, input handling, view
state, widgets) MUST NOT change the document directly. It reads the document and emits
actions (see `HIST`). Presentation code MUST NOT be part of the core.

**ARCH-3 — Headless operability.** Every action that can be applied to a document MUST
also be possible without a GUI: from automated unit tests and from a command-line
front end. A command-line front end is not required yet, but nothing in the core may
prevent one from being added.

**ARCH-4 — View state is not document state.** Pan, zoom and view rotation (see `VIEW`)
are presentation state. They MUST NOT be stored in the document, recorded as actions,
or affected by undo/redo.

**ARCH-5 — Transient tool state is not document state.** In-progress interactions MUST
stay in presentation state until they are committed as an action. Examples are points
placed before a set is complete, and the live position of a point being dragged.

**ARCH-6 — Image decoding is core.** Decoding image files into pixel data MUST NOT
depend on the UI toolkit. This lets a headless front end load images.

**ARCH-7 — Unit tests.** Business logic MUST be covered by automated unit tests. This
includes document actions, history, calibration maths, view transforms, hit-testing,
and text parsing and formatting. The tests MUST run without opening a window.

**ARCH-8 — Platform abstraction.** Native services such as the file-open dialog MUST
sit behind an interface, so that logic which uses them can be tested with a fake.

**ARCH-9 — Code quality.** The code SHOULD follow clean-code principles (DRY, SOLID)
and SHOULD be organised into cohesive modules.

## Acceptance criteria

- A test builds a document, applies every action type, undoes and redoes them, and
  checks the resulting calibration scale. No UI components are created.
- The core component compiles and its tests run with no UI toolkit linked or imported.
