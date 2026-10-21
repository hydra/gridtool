# 03 — Image Loading

## Requirements

**IMG-1 — Supported formats.** The application opens images with these file extensions:
`png`, `jpg`, `jpeg` and `bmp`. Extension matching is case-insensitive, so `PNG` and
`JpEg` are accepted. A file with no extension, or whose name is only an extension such
as `.png`, is not supported. The list of extensions MUST be defined once and used both
for the dialog filter and for validation.

**IMG-2 — Choosing a file.** Clicking the Open image button shows the platform's native
file-open dialog:
- the title is "`Open image`"
- there is one filter, named "`Images`", that limits the listing to the supported
  extensions
- the user can select one file

**IMG-3 — Cancel.** If the user cancels the dialog, nothing changes. The current document,
its history and its view are all kept.

**IMG-4 — Re-validation.** The dialog filter can be bypassed, for example by typing a
file name. So after a file is chosen, its extension MUST be checked again before the
file is read. An unsupported extension produces an error and the file is not read.

**IMG-5 — Decoding.** The file is read and decoded into 8-bit-per-channel RGBA pixels at
the image's native width and height. Transparency is kept.

**IMG-6 — Errors.** If opening fails, the current document is closed and the error is
shown in the canvas area (`SHELL-4`). The message has one of these forms:
- "`unsupported image format: <path>`"
- "`could not read file: <system error>`"
- "`could not decode image: <decoder error>`"

**IMG-7 — Success.** On success, the previous document (if any) is discarded together
with its history and view state. A new document is created from the image (`DOC-2`) with
an empty history (`HIST`), and the view returns to its defaults (`VIEW-2`).

**IMG-8 — Display.** The decoded image is drawn as the document's content on the canvas
and fills the document rectangle exactly (`VIEW-6`).

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
