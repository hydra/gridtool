# 16 — Workspace: Panels and Multiple Documents

The window below the tool bars is the **workspace**. It has two separate docking systems: the
**panels** around the outside, and the **Documents area** in the middle, in which the open
documents have their tabs.

## Panels

**WORK-1 — Panels.** There are four panels, each showing information about the active document
(`WORK-10`):

| Panel | Contents |
|---|---|
| Objects | the object tree (`OBJ-3`) |
| Point-to-point | the distance measurements table (`MEAS-10`) |
| Areas | the area measurements table (`AREA-12`) |
| History | the history list (`HIST-10`) |

A panel's title is on its tab, so the panel's content has no heading of its own. Each panel
scrolls independently.

**WORK-2 — Default layout.** From left to right: Objects, the Documents area, then a column
holding Point-to-point, Areas and History, from top to bottom, in equal shares of its height.
The three columns start with widths in the proportion 220 : 600 : 460.

**WORK-3 — Arranging panels.** The user can:
- **Resize.** Drag the divider between two neighbouring tiles.
- **Dock.** Drag a panel by its tab and drop it above, below, left or right of a panel or of the
  Documents area. A preview shows where it will go while dragging.
- **Group.** Drop a panel in the middle of another, or on its tab bar. They then share one tile,
  with a tab each, and one shows at a time. Tabs can be reordered in the same way.

**WORK-4 — The Documents area stays.** The Documents area is always there, and always in view:
- It has no tab, cannot be dragged, and cannot be closed.
- It never shares a tab group, directly or as part of a larger tile, because that could hide it.
- A panel dropped anywhere on it, including on a document, docks beside it on the nearest side,
  and is never placed inside it.
- It is fine for no panel to be on a given side of it, or for no panels to be open at all.

**WORK-5 — Closing and reopening panels.** Each panel's tab has a close button. The Panels menu
on the tool bar (`SHELL-5`) lists the panels by title in the order of `WORK-1`, each with a
checkbox showing whether it is open:
- Unticking closes the panel, and ticking reopens it with about a fifth of the workspace's width,
  at the left of the workspace for Objects and at the right for the others.
- Below the list, "`Reset layout`" returns every panel to the default layout (`WORK-2`), reopening
  any that were closed. The documents and how they are arranged are left alone.

**WORK-6 — Restoring the layout.** The arrangement of the panels is saved when the application
exits, and restored the next time it starts. Nothing else is restored: no documents are reopened,
and the window has its initial size (`SHELL-1`). A saved arrangement that cannot be read, or that
the user could not have made (`WORK-4`, or a panel appearing twice), is ignored in favour of the
default layout.

## Documents

**WORK-7 — Tabs.** Each open document has one tab, with a close button. Its title is the document's
file name with its extension or, if it has never been saved, the image's file name, followed by
"`*`" while it has unsaved changes (`FILE-9`). The same image can be open as several documents.

**WORK-8 — Arranging documents.** The user can drag a document's tab:
- within its tab group or to another, to reorder or regroup it, and
- to the edge of a document, to split the area there horizontally or vertically, in any
  arrangement.

Dividers between groups resize them. A group left with no tabs goes away, and its space goes to
the groups beside it.

**WORK-9 — The two systems are separate.** A panel can never enter the Documents area, and a
document can never leave it. A document's tab let go of anywhere outside the Documents area stays
where it was, and so does everything else.

**WORK-10 — The active document.** At most one document is active, and it is always in view. The
tool bar, tool controls row, panels and window title all show it, and Undo, Redo, Save and Save as
act on it (`SHELL-5`). With more than one document in view, the active one is outlined in the
selection colour, 2 px wide.
- Clicking a tab, or starting to drag one, makes its document active.
- Clicking or starting a drag on a document in view that is not active only makes it active. The
  tool does nothing with that click or drag, and the next one reaches the tool.
- A field being edited (`FMT-9`) when another document is made active finishes its edit on the
  document it belongs to.
- A newly opened document becomes active, as a new tab in the active document's group.
- When the active document closes, or another tab in its group is brought forward, the one in view
  in that group becomes active or, failing that, another one in view.

**WORK-11 — What belongs to each document.** Each document keeps its own view (`VIEW`), tool mode
(`SHELL-8`), active object, history (`HIST-8`), file, and whatever a tool has placed so far, such
as the points of an unfinished area. Switching between documents loses none of these.

**WORK-12 — What works on inactive documents.** The mouse wheel zooms whichever document is under
the pointer (`VIEW`), without making it active. Nothing else changes a document that is not active.

**WORK-13 — No documents.** With no document open, the Documents area is one empty area showing
the message of `SHELL-3`, and the next document opened fills it.

## Cursor

**WORK-14 — Cursor over tabs.** Hovering over a tab or the background of a tab bar, of a panel or of
a document, shows the normal cursor, since tabs already look draggable. While one is being dragged,
the cursor is the grabbing hand.

## Acceptance criteria

- Cursor (`WORK-14`): over a document's and a panel's tab and tab bar the cursor is the normal one,
  and it is the grabbing hand while dragging from each of them.
- The whole application, driven with simulated input and without a window:
  - **Opening.** A second document opens as a tab beside the first and becomes active, and the
    first document is kept.
  - **Switching.** Clicking a tab brings its document forward and makes it active. Each document
    keeps its own tool.
  - **Closing.** Closing the active document makes the one beside its tab active, and closing the
    last leaves the area empty, with the next document opening in it.
  - **Inactive documents.** With two documents in view and a tool that acts on a click, clicking
    the inactive document makes it active and changes nothing, and clicking again reaches the
    tool. Dragging on it does the same.
  - **The wheel.** Zooming over the inactive document zooms it, leaves the other alone and does not
    change which is active.
  - **Routing.** Tool actions and Undo reach the active document only.
  - **Field edits.** With a table coordinate being edited in one document, clicking another
    document commits the edit to the first and changes nothing in the second.
- Panels:
  - **Docking.** A panel dropped near each of the four edges of the Documents area docks on that
    side.
  - **Middle.** A panel dropped in the middle of the Documents area, or on a document, ends up
    beside the area, with the documents unchanged, and the Documents area shares no tab group.
  - **Grouping.** A panel dropped in the middle of another joins it, with one of the two showing.
  - **The Documents area.** Dragging it moves nothing.
  - **Document tabs.** A tab let go of over a panel changes nothing. A tab dropped on another
    document groups them, and one dropped at the edge of a document splits the area.
- Closing and reopening: a panel closes by its tab's close button and by the Panels menu, and a
  reopened one has a real share of the window. Reset layout reopens every panel. The tool bar shows
  which are open.
- Layout: a layout saved and read back is equal to the original, with no documents open in it.
  Missing, unreadable and impossible layouts give the default layout.
