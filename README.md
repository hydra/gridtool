# GridTool

[![CI](https://github.com/hydra/gridtool/actions/workflows/ci.yml/badge.svg)](https://github.com/hydra/gridtool/actions/workflows/ci.yml)

## About

GridTool is a desktop application for measuring real-world objects from an image, such
as a plan, a drawing or a photo. Calibrate the image's scale from two known distances,
then measure distances and overlay real-world grids of any cell size, aligned to a scale
or measurement line. Every change can be undone, and documents can be saved and loaded.

<a href="assets/screenshots/Screenshot%202026-10-21%20173320.png"><img src="assets/screenshots/Screenshot%202026-10-21%20173320.png" width="800" alt="GridTool showing a calibrated site plan with measurements and two grids"></a>

## Features

- **Open images**: PNG, JPEG and BMP plans, drawings and photos.
- **Scale calibration**: from two lines of known length, using 3 or 4 points, with separate horizontal and vertical scales.
- **Point-to-point measurements**: real-world distances, with editable coordinates, per-measurement units (cm or m) and colours.
- **Area measurements**: convex or concave outlines with 3 or more points, filled and labelled with their area in cm² or m².
- **Grids**: real-world grids with square or rectangular cells of any size, aligned to a scale or measurement line, with adjustable colour and opacity.
- **Editing**: select objects on the canvas or in the object tree, drag their points, stretch them with a bounding box, duplicate and delete them.
- **View**: pan, zoom about the cursor, and rotate the view about a movable origin, without changing the document.
- **Undo and redo**: every change is recorded in a history panel.
- **Documents**: save and load your work, with a prompt to save unsaved changes.

## Authors

* Dominic Clifton - Product owner and technical lead.
* ClaudeAI - Coder.

## License

Licensed under either of the following licnses at your option.

- Apache License, Version 2.0 (<https://www.apache.org/licenses/LICENSE-2.0>)
- MIT license (<https://opensource.org/licenses/MIT>)

Note: Unless you explicitly state otherwise, any contribution intentionally submitted for
inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual
licensed as above, without any additional terms or conditions.
