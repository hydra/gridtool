# GridTool

[![CI](https://github.com/hydra/gridtool/actions/workflows/ci.yml/badge.svg)](https://github.com/hydra/gridtool/actions/workflows/ci.yml)

## About

GridTool is a desktop application for measuring real-world objects from an image, such
as a plan, a drawing or a photo. Calibrate the image's scale from one or two known distances,
then measure distances and overlay real-world grids of any cell size, aligned to a scale
or measurement line. Every change can be undone, and documents can be saved and loaded.

## Screenshot

<a href="assets/screenshots/Screenshot%202026-11-01%20213112.png"><img src="assets/screenshots/Screenshot%202026-11-01%20213112.png" width="800" alt="GridTool with two documents side by side, a site plan with a grid and filled areas and a calibrated aerial photo with a measured area, and the Objects, History, Point-to-point and Areas panels docked around them"></a>

## Features

- **Open images**: PNG, JPEG and BMP plans, drawings and photos.
- **Scale calibration**: from a single line of known length, using 2 points, such as the scale bar on a map, or from two lines of known length, using 3 or 4 points, with separate horizontal and vertical scales.
- **Point-to-point measurements**: real-world distances, with editable coordinates, per-measurement units (cm or m) and line styles.
- **Area measurements**: convex or concave outlines with 3 or more points, labelled with their area in cm² or m², with their own line and fill styles.
- **Grids**: real-world grids with square or rectangular cells of any size, aligned to a scale or measurement line, with adjustable colour and opacity.
- **Drawing**: draw lines through any number of points, finishing with a double-click or Enter, or click the first point to join them into an area. Choose thin, thick or dashed lines, fill areas solid or with lines or crosshatching at any real-world distance and angle, and set the colour and opacity of each.
- **Tool controls**: a row under the tool bar for the active object, or for the next object a tool adds: units, line and fill styles, colours and opacity.
- **Editing**: select objects on the canvas or in the object tree, drag their points, stretch them with a bounding box, duplicate and delete them.
- **View**: pan, zoom about the cursor, and rotate the view about a movable origin, without changing the document.
- **Undo and redo**: every change is recorded in a history panel.
- **Documents**: save and load your work, with a prompt to save unsaved changes.
- **Theme and language**: system, light or dark theme, in English or Spanish.

## Authors

* Dominic Clifton - Product owner and technical lead.
* ClaudeAI - Coder.

## Support me

If you find this tool useful, please consider support me using the links below:

* ko-fi - https://ko-fi.com/dominicclifton

## AI/LLM usage

Although the code in this project is not human written, many, many humans together made this possible. LLMs don't have anything to learn from unless we show them the way, humans are required to guide them and correct them.

## Languages

Multiple languages are supported and can be changed on-the-fly.
English is the primary language. Spanish was machine translated.

Contributions/PRs for corrections and other languages welcomed.

## License

Licensed under either of the following licnses at your option.

- Apache License, Version 2.0 (<https://www.apache.org/licenses/LICENSE-2.0>)
- MIT license (<https://opensource.org/licenses/MIT>)

Note: Unless you explicitly state otherwise, any contribution intentionally submitted for
inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual
licensed as above, without any additional terms or conditions.
