# 02 — Application Shell

## Window

**SHELL-1.** The application opens one native window titled "`GridTool`", with an
initial inner size of 1280 × 800.

## Layout

**SHELL-2.** The window is divided into these regions, from top to bottom:

1. **Main tool bar.** One horizontal row, always visible.
2. **Tool options row.** A second horizontal row directly below the tool bar. It is shown
   only when the active tool mode has options *and* a document is open. Its contents
   depend on the mode: see `VIEW-12` (Rotate) and `SCALE-3` (Scale). The Pan mode has
   no options row.
3. Below the rows, side by side:
   - **Canvas.** Fills the remaining space and shows the document (see `VIEW`, `SCALE`).
   - **History panel.** Docked on the right-hand side (see `HIST-10`).

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
| 6 | Horizontal spacer, 16 px | — | — |
| 7 | Undo button | undo arrow (↩) | "`Undo`" |
| 8 | Redo button | redo arrow (↪) | "`Redo`" |

**SHELL-6.** Tool bar icons are drawn at 18 px. The icon glyphs MUST be available on
every supported platform. Bundle the icon font if necessary, rather than relying on
system fonts.

**SHELL-7.** Tooltips MUST appear on hover, including when the button is disabled.

## Tool modes

**SHELL-8.** There is exactly one active tool mode at a time. The modes are **Pan &
zoom**, **Rotate view** and **Scale**. **Pan & zoom is the default** when the
application starts.

**SHELL-9.** The mode buttons work like a radio group. The active mode's button is shown
as selected, and clicking a mode button makes that mode active. The active mode is
application state and stays the same when a different image is opened.

**SHELL-10.** New modes will be added later. The list of modes MUST be defined in one
place, so that adding a mode needs no duplicated tool bar code.

## Acceptance criteria

- The default tool mode is Pan & zoom.
- Every mode has a distinct icon and a distinct name.
