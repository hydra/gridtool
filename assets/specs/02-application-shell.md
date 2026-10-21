# 02 — Application Shell

## Window

**SHELL-1.** The application opens one native window titled "`GridTool`", with an
initial inner size of 1280 × 800.

## Layout

**SHELL-2.** The window is divided into these regions, from top to bottom:

1. **Main tool bar.** One horizontal row, always visible.
2. **Tool options row.** A second horizontal row directly below the tool bar. It is shown
   only when the active tool mode has options *and* a document is open. Its contents
   depend on the mode: see `VIEW-12` (Rotate) and `SCALE-3` (Scale). The Pan & zoom and
   Measure distance modes have no options row.
3. Below the rows, side by side:
   - **Canvas.** Fills the remaining space and shows the document (see `VIEW`, `SCALE`,
     `MEAS`).
   - **Side panel.** Docked on the right-hand side (see `SHELL-11`).

**SHELL-3.** When no document is open, the canvas area shows the centred message
"`Open an image to get started.`"

**SHELL-4.** When the last attempt to open an image failed, the canvas area shows the
error message centred, in the toolkit's error colour (see `IMG-6`).

## Main tool bar

**SHELL-5.** The tool bar contains these items, left to right:

| # | Item | Icon meaning (current glyph) | Tooltip |
|---|---|---|---|
| 1 | Open image button | open folder (🗁) | "`Open image…`" |
| 2 | Horizontal spacer, 16 px | — | — |
| 3 | Pan & zoom mode button | hand (✋) | "`Pan & zoom`" |
| 4 | Rotate view mode button | rotate arrows (🔄) | "`Rotate view`" |
| 5 | Scale mode button | ruler (📏) | "`Scale`" |
| 6 | Measure distance mode button | measuring tool. A tape measure is preferred, but Unicode has none, so the current glyph is a set square (📐) | "`Measure distance`" |
| 7 | Horizontal spacer, 16 px | — | — |
| 8 | Undo button | undo arrow (↩) | "`Undo`" |
| 9 | Redo button | redo arrow (↪) | "`Redo`" |

**SHELL-6.** Tool bar icons are drawn at 18 px. The icon glyphs MUST be available on
every supported platform. Bundle the icon font if necessary, rather than relying on
system fonts.

**SHELL-6a — Icon verification.** Every icon used anywhere in the UI MUST be defined in
one place.
- An automated test MUST check that each icon is exactly one character and that the
  bundled fonts contain a glyph for it. System fonts don't count, because they are not
  guaranteed to be present.
- If no suitable glyph exists for an icon's meaning, use a plain-text fallback instead.
  The delete button, for example, falls back to an "x" button (`MEAS-11`).

**SHELL-7.** Tooltips MUST appear on hover, including when the button is disabled.

## Tool modes

**SHELL-8.** There is exactly one active tool mode at a time. The modes are **Pan &
zoom**, **Rotate view**, **Scale** and **Measure distance**. **Pan & zoom is the
default** when the application starts.

**SHELL-9.** The mode buttons work like a radio group. The active mode's button is shown
as selected, and clicking a mode button makes that mode active. The active mode is
application state and stays the same when a different image is opened.

**SHELL-10.** New modes will be added later. The list of modes MUST be defined in one
place, so that adding a mode needs no duplicated tool bar code.

## Side panel

**SHELL-11.** The side panel is docked on the right of the window:
- It can be resized horizontally and has a default width of 460 px.
- It is split vertically into two sections:
  1. **Upper section**: the measurements table (`MEAS-10`), with a default height of
     280 px.
  2. **Lower section**: the history list (`HIST-10`), filling the remaining height.
- The divider between the sections can be dragged up and down to resize them.
- Each section scrolls independently.

## Acceptance criteria

- The default tool mode is Pan & zoom.
- Every mode has a distinct icon and a distinct name.
- In the tool bar, Measure distance comes directly after Scale.
