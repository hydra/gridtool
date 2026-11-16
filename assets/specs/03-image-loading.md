# 03 — Image Loading

## Requirements

**IMG-1 — Supported formats.** The application opens images with these file extensions:
`png`, `jpg`, `jpeg` and `bmp`. Extension matching is case-insensitive, so `PNG` and
`JpEg` are accepted. A file with no extension, or whose name is only an extension such
as `.png`, is not supported. The list of extensions MUST be defined once and used both
for the dialog filter and for validation.

**IMG-2 — Choosing a file.** Clicking the New button (`SHELL-5`) shows the platform's native
file-open dialog:
- the title is "`Open image`"
- there is one filter, named "`Images`", that limits the listing to the supported
  extensions
- the user can select one file

**IMG-3 — Cancel.** If the user cancels the dialog, nothing changes.

**IMG-4 — Re-validation.** The dialog filter can be bypassed, for example by typing a
file name. So after a file is chosen, its extension MUST be checked again before the
file is read. An unsupported extension produces an error and the file is not read.

**IMG-5 — Decoding.** The file is read and decoded into 8-bit-per-channel RGBA pixels at
the image's native width and height. Transparency is kept.

**IMG-6 — Errors.** If opening fails, the open documents are kept and an error dialog titled
"`Could not open image`", with one "`OK`" button, shows a message in one of these forms:
- "`unsupported image format: <path>`"
- "`could not read file: <system error>`"
- "`could not decode image: <decoder error>`"

**IMG-7 — Success.** On success, a new document is created from the image (`DOC-2`) with an empty
history (`HIST`) and the default view (`VIEW-2`), and it becomes active (`WORK-10`). The open
documents are kept. The new document has no file and has unsaved changes until it is saved (`FILE-9`).

**IMG-8 — Display.** The decoded image is drawn as the document's content on the canvas
and fills the document rectangle exactly (`VIEW-6`).

**IMG-9 — Large images.** An image of any size that decodes MUST open and display at its native
resolution, even when it is wider or taller than the largest texture the graphics device
supports. It is then shown as adjacent tiles, each within the device limit, that together
cover the document rectangle exactly with no gaps or overlaps. Exceeding the device limit
MUST NOT crash the application.

## Acceptance criteria

- Every supported extension is accepted, in any letter case.
- `gif`, `txt`, a file with no extension, and a file named only `.png` are rejected.
- A valid PNG decodes to the correct width, height and RGBA pixel values. This includes
  a partially transparent pixel.
- A valid BMP decodes to the correct dimensions.
- Bytes that are not an image produce a decode error.
- A missing file with a supported extension produces a read error.
- Cancelling the picker produces no change.
- A picked file with an unsupported extension produces an unsupported-format error.
- A picked valid file decodes, and the path it came from is kept.
- A layout of tiles for a given device limit covers the image exactly, row by row, with
  no tile wider or taller than the limit; a 4675×10976 image on an 8192 limit is two rows.
- Cropping each tile out of an image and placing them back rebuilds the original pixels.
