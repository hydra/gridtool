theme-button-light = ☀ Light
theme-button-dark = 🌙 Dark
theme-button-system = 💻 System
settings-theme = Theme

# format "<language in native language> (<country in native language>)
language-es-ES = Español (España)
language-en-US = English (United States)
settings-language = Language

## Tool bar

toolbar-new = New…
toolbar-load = Load…
toolbar-save = Save
toolbar-save-as = Save as…
toolbar-undo = Undo
toolbar-redo = Redo
toolbar-about = About
toolbar-settings = Settings
toolbar-panels = Panels
toolbar-reset-layout = Reset layout

tool-pan = Pan & zoom
tool-rotate = Rotate view
tool-scale = Scale
tool-measure = Measure distance
tool-area = Measure area
tool-route = Measure route
tool-grid = Grid
tool-origin = Origin
tool-draw-line = Draw line

## Workspace

panel-documents = Documents
panel-objects = Objects
panel-point-to-point = Point-to-point
panel-areas = Areas
panel-routes = Routes
panel-history = History

placeholder-open-image = Open an image to get started.
history-opened-image = Opened image

## Dialogs

dialog-yes = Yes
dialog-no = No
dialog-cancel = Cancel
dialog-ok = OK
dialog-unsaved-title = Unsaved changes
dialog-unsaved-message = Do you want to save { $name } now?
dialog-overwrite-title = File already exists
dialog-overwrite-message = { $file } already exists. Do you want to replace it?
dialog-error-open-image = Could not open image
dialog-error-load = Could not load document
dialog-error-save = Could not save document
document-unsaved-name = unsaved document
about-icon = { $app } icon

picker-open-image = Open image
picker-images = Images
picker-document-filter = GridTool document
picker-load-document = Load document
picker-save-document = Save document as

crash-title = { $app } crashed
crash-closing = { $app } had a problem and has to close. Changes that were not saved have been lost.
crash-report-saved = A crash report was saved to:
crash-report-help = To help get this fixed, please attach it to an issue at { $url }/issues. Nothing is sent automatically.
crash-no-report = A crash report could not be saved.

## Tables and the object tree

column-color = Color
column-area = Area
column-units = Units
column-distance = Distance
column-length = Length

areas-empty = No areas yet. Use the Measure area tool to add one.
area-color-tooltip = Choose the line colour of this area
area-unit-tooltip = Units of this area
area-delete-tooltip = Delete area

routes-empty = No routes yet. Use the Measure route tool to add one.
route-color-tooltip = Choose the line colour of this route
route-unit-tooltip = Units of this route
route-delete-tooltip = Delete route
route-expand-tooltip = Show the segments of this route
route-collapse-tooltip = Hide the segments of this route

measurements-empty = No measurements yet. Use the Measure distance tool to add one.
measurement-color-tooltip = Choose the line colour of this measurement
measurement-unit-tooltip = Units of this measurement
measurement-delete-tooltip = Delete measurement

tree-duplicate = Duplicate
tree-delete = Delete
tree-no-document = No document open
tree-document = Document
tree-measurements = Measurements
tree-point-to-point = Point-to-point
tree-area = Area
tree-routes = Routes
tree-grids = Grids
tree-shapes = Shapes
tree-grid = Grid { $number } ({ $source })
tree-source-scale-line = scale line { $line }
tree-source-measurement = measurement { $number }
tree-source-missing = alignment line missing

object-measurement = Measurement { $number }
object-measurement-with-length = Measurement { $number } ({ $length } { $unit })
object-area = Area { $number }
object-area-with-value = Area { $number } ({ $value } { $unit })
object-route = Route { $number }
object-route-with-length = Route { $number } ({ $length } { $unit })
object-shape-line = Shape { $number } (line)
object-shape-area = Shape { $number } (area)

## Tool controls

help-no-document = Open an image to get started
help-pan = Drag to pan the view. Scroll to zoom.
help-origin = Click on the image to move the origin, the point the view rotates about.

new-measurement = New measurement
new-measurement-unit-tooltip = Units of new measurements
new-area = New area
new-area-unit-tooltip = Units of new areas
new-route = New route
new-route-unit-tooltip = Units of new routes
new-shape = New shape

rotation-label = View rotation (°):

scale-problem = Scale: { $reason }
scale-result = X: { $x } { $unit }/px   Y: { $y } { $unit }/px
scale-units-one-tooltip = Units of distance A
scale-units-two-tooltip = Units of distances A and B

grid-title = Grid { $number }
grid-hidden = Grid not shown: { $reason }
grid-too-small = the cells are under { $spacing } px on screen; zoom in to see them
grid-side-tooltip = Side length of each square cell
grid-size-a-tooltip = Cell size along the alignment line
grid-size-b-tooltip = Cell size perpendicular to the alignment line
grid-unit-tooltip = Units of the cell size
grid-shape-label = Cell shape:
grid-color-label = Color:
grid-color-tooltip = Choose the grid colour
grid-opacity-label = Opacity:
grid-via-scale = Via scale
grid-via-measurement = Via measurement
grid-hint-place-scale = Place the scale points first (Scale tool).
grid-hint-click-scale = Click a scale line to add a grid aligned to it.
grid-hint-add-measurement = Add a measurement first (Measure distance tool).
grid-hint-click-measurement = Click a measurement line to add a grid aligned to it.

style-line-label = Line:
style-line-color-label = Line color:
style-line-color-tooltip = Choose the colour of the line
style-line-opacity-label = Line opacity:
style-fill-label = Fill:
style-fill-opacity-label = Fill opacity:
style-fill-color-label = Fill color:
style-fill-lines-hidden = Fill lines not shown: they need a scale ({ $reason })
style-spacing-label = Distance:
style-spacing-tooltip = Distance between the fill lines
style-spacing-unit-tooltip = Units of the distance between the fill lines
style-rotate-left-tooltip = Rotate the fill lines { $degrees }° left
style-rotate-right-tooltip = Rotate the fill lines { $degrees }° right
style-angle-label = Angle:
style-angle-tooltip = Angle of the fill lines, in degrees clockwise from vertical
style-fill-locked-tooltip = The fill uses the line's colour. Click to choose its own
style-fill-locked-swatch-tooltip = Click the padlock to choose the colour of the fill
style-fill-own-tooltip = The fill has its own colour. Click to use the line's colour, forgetting this one
style-fill-own-swatch-tooltip = Choose the colour of the fill

## Names of things in the document

method-two-point = 2-point
method-three-point = 3-point
method-four-point = 4-point

cell-shape-square = Square
cell-shape-rectangle = Rectangle

line-style-thin = Thin
line-style-thick = Thick
line-style-dashed = Dashed

fill-style-none = None
fill-style-solid = Solid
fill-style-lines = Lines
fill-style-crosshatch = Crosshatch

endpoint-start = start
endpoint-end = end

target-measurement = measurement { $number }
target-area = area { $number }
target-shape = shape { $number }
target-route = route { $number }

## Why something could not be done

error-point-outside = point { $point } is outside the document
error-no-scale-points = there are no scale points to move
error-invalid-distance = distance must be a positive number, got { $distance }
error-measurement-missing = measurement { $number } does not exist (there are { $count })
error-area-missing = area { $number } does not exist (there are { $count })
error-area-point-missing = area point { $number } does not exist (there are { $count })
error-area-point-count = an area with { $expected } points cannot be given { $actual } points
error-grid-missing = grid { $number } does not exist (there are { $count })
error-grid-source-missing = the grid's alignment line does not exist
error-invalid-cell-size = cell size must be a positive number, got { $size }
error-invalid-opacity = opacity must be between 0 and 1, got { $opacity }
error-shape-missing = shape { $number } does not exist (there are { $count })
error-shape-point-missing = shape point { $number } does not exist (there are { $count })
error-shape-point-count = a shape with { $expected } points cannot be given { $actual } points
error-route-missing = route { $number } does not exist (there are { $count })
error-route-point-missing = route point { $number } does not exist (there are { $count })
error-route-point-count = a route with { $expected } points cannot be given { $actual } points
error-no-fill = { $target } encloses no area, so has no fill
error-invalid-fill-spacing = fill spacing must be a positive number, got { $spacing }
error-invalid-fill-angle = fill angle must be from 0 up to 360, got { $degrees }
error-invalid-rotation = the view rotation must be a finite number, got { $degrees }

error-area-too-few-points = an area needs at least { $minimum } points, got { $count }
error-shape-too-few-points = a shape cannot have only { $count } points
error-route-too-few-points = a route needs at least { $minimum } points, got { $count }

error-calibration-point-count = { $method } calibration needs { $expected } points, got { $actual }
error-calibration-point-missing = point { $number } does not exist (there are { $count })

error-scale-missing-points = place the calibration points
error-scale-missing-distance = enter the distance of line { $line }
error-scale-zero-length = line { $line } has zero length
error-scale-indeterminate = lines A and B need different orientations to separate X and Y
error-scale-inconsistent = the distances do not fit the lines

error-grid-source-gone = its alignment line no longer exists
error-grid-source-zero-length = its alignment line has zero length
error-grid-no-scale = it needs a scale ({ $reason })
error-grid-too-dense = the cells are too small to draw

error-image-unsupported = unsupported image format: { $path }
error-image-read = could not read file: { $error }
error-image-decode = could not decode image: { $error }

error-document-empty = the document has no size
error-document-point-outside = point { $point } is outside the document
error-document-scale-point-count = the number of scale points does not match the method
error-document-two-point-version = this version of the file cannot have a 2-point scale
error-document-area-points = an area has { $count } points, but needs at least 3
error-document-shape-points = a shape has too few points ({ $count })
error-document-route-points = a route has { $count } points, but needs at least 2
error-document-filled-open-shape = shape { $id } has a fill but is not closed
error-document-not-positive = { $value } should be a positive number
error-document-opacity = opacity { $value } is not between 0 and 1
error-document-fill-angle = fill angle { $value } is out of range
error-document-rotation = view rotation { $value } is out of range
error-document-object-id = object id { $id } is invalid or used twice

error-save-encode = could not encode the document: { $error }
error-save-write = could not write the file: { $error }

error-load-read = could not read the file: { $error }
error-load-parse = the file is not a GridTool document: { $error }
error-load-invalid = the document is invalid: { $reason }
error-load-image = could not open the image { $path }: { $error }
error-load-image-size = the image { $path } is { $actual_width }×{ $actual_height } pixels, but the document expects { $expected_width }×{ $expected_height }

## The history panel

action-set-scale-points = Set { $method } scale points
action-move-scale-point = Move scale point { $number } to { $to }
action-set-distance = Set distance { $line } to { $distance }
action-clear-distance = Clear distance { $line }
action-set-unit = Set units to { $unit }
action-set-origin = Set origin to { $origin }
action-set-rotation = Set view rotation to { $degrees }°
action-add-measurement = Add measurement from { $start } to { $end }
action-move-measurement-point = Move measurement { $number } { $endpoint } to { $to }
action-resize-measurement = Resize measurement { $number } to { $start } - { $end }
action-move-measurement = Move measurement { $number } to { $start } - { $end }
action-set-measurement-unit = Set measurement { $number } units to { $unit }
action-delete-measurement = Delete measurement { $number }
action-duplicate-measurement = Duplicate measurement { $number }
action-add-area = Add area with { $count } points
action-move-area-point = Move area { $number } point { $point } to { $to }
action-resize-area = Resize area { $number }
action-move-area = Move area { $number }
action-set-area-unit = Set area { $number } units to { $unit }
action-delete-area = Delete area { $number }
action-duplicate-area = Duplicate area { $number }
action-add-grid-scale-line = Add grid aligned to scale line { $line }
action-add-grid-measurement = Add grid aligned to a measurement
action-set-grid-shape = Set grid { $number } cell shape to { $shape }
action-set-grid-size = Set grid { $number } size { $dimension } to { $size }
action-set-grid-unit = Set grid { $number } units to { $unit }
action-set-grid-color = Set grid { $number } color to { $color }
action-set-grid-opacity = Set grid { $number } opacity to { $percent }%
action-delete-grid = Delete grid { $number }
action-duplicate-grid = Duplicate grid { $number }
action-add-closed-shape = Add closed shape with { $count } points
action-add-line = Add line with { $count } points
action-move-shape-point = Move shape { $number } point { $point } to { $to }
action-resize-shape = Resize shape { $number }
action-move-shape = Move shape { $number }
action-delete-shape = Delete shape { $number }
action-duplicate-shape = Duplicate shape { $number }
action-add-route = Add route with { $count } points
action-move-route-point = Move route { $number } point { $point } to { $to }
action-resize-route = Resize route { $number }
action-move-route = Move route { $number }
action-set-route-unit = Set route { $number } units to { $unit }
action-delete-route = Delete route { $number }
action-duplicate-route = Duplicate route { $number }
action-set-style = Set { $target } { $edit }

edit-line-style = line style to { $style }
edit-line-color = line color to { $color }
edit-line-opacity = line opacity to { $percent }%
edit-fill-style = fill style to { $style }
edit-fill-color = fill color to { $color }
edit-fill-color-none = fill color to none
edit-fill-spacing = fill spacing to { $spacing }
edit-fill-unit = fill units to { $unit }
edit-fill-angle = fill angle to { $degrees }°
edit-fill-opacity = fill opacity to { $percent }%
