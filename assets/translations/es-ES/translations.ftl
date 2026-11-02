theme-button-light = ☀ Claro
theme-button-dark = 🌙 Oscuro
theme-button-system = 💻 Sistema
settings-theme = Tema

# format "<language in native language> (<country in native language>)
language-es-ES = Español (España)
language-en-US = English (United States)
settings-language = Idioma

## Tool bar

toolbar-new = Nuevo…
toolbar-load = Cargar…
toolbar-save = Guardar
toolbar-save-as = Guardar como…
toolbar-undo = Deshacer
toolbar-redo = Rehacer
toolbar-about = Acerca de
toolbar-settings = Ajustes
toolbar-panels = Paneles
toolbar-reset-layout = Restablecer disposición

tool-pan = Desplazar y ampliar
tool-rotate = Girar vista
tool-scale = Escala
tool-measure = Medir distancia
tool-area = Medir área
tool-grid = Cuadrícula
tool-origin = Origen
tool-draw-line = Dibujar línea

## Workspace

panel-documents = Documentos
panel-objects = Objetos
panel-point-to-point = Punto a punto
panel-areas = Áreas
panel-history = Historial

placeholder-open-image = Abra una imagen para empezar.
history-opened-image = Imagen abierta

## Dialogs

dialog-yes = Sí
dialog-no = No
dialog-cancel = Cancelar
dialog-ok = Aceptar
dialog-unsaved-title = Cambios sin guardar
dialog-unsaved-message = ¿Desea guardar { $name } ahora?
dialog-overwrite-title = El archivo ya existe
dialog-overwrite-message = { $file } ya existe. ¿Desea reemplazarlo?
dialog-error-open-image = No se pudo abrir la imagen
dialog-error-load = No se pudo cargar el documento
dialog-error-save = No se pudo guardar el documento
document-unsaved-name = documento sin guardar
about-icon = Icono de { $app }

picker-open-image = Abrir imagen
picker-images = Imágenes
picker-document-filter = Documento de GridTool
picker-load-document = Cargar documento
picker-save-document = Guardar documento como

crash-title = { $app } se ha bloqueado
crash-closing = { $app } ha tenido un problema y debe cerrarse. Los cambios que no se guardaron se han perdido.
crash-report-saved = Se ha guardado un informe de error en:
crash-report-help = Para ayudar a solucionarlo, adjúntelo a una incidencia en { $url }/issues. No se envía nada automáticamente.
crash-no-report = No se pudo guardar un informe de error.

## Tables and the object tree

column-color = Color
column-area = Área
column-units = Unidades
column-distance = Distancia

areas-empty = Aún no hay áreas. Use la herramienta Medir área para añadir una.
area-color-tooltip = Elegir el color de línea de esta área
area-unit-tooltip = Unidades de esta área
area-delete-tooltip = Eliminar área

measurements-empty = Aún no hay mediciones. Use la herramienta Medir distancia para añadir una.
measurement-color-tooltip = Elegir el color de línea de esta medición
measurement-unit-tooltip = Unidades de esta medición
measurement-delete-tooltip = Eliminar medición

tree-duplicate = Duplicar
tree-delete = Eliminar
tree-no-document = Ningún documento abierto
tree-document = Documento
tree-measurements = Mediciones
tree-point-to-point = Punto a punto
tree-area = Área
tree-grids = Cuadrículas
tree-shapes = Formas
tree-grid = Cuadrícula { $number } ({ $source })
tree-source-scale-line = línea de escala { $line }
tree-source-measurement = medición { $number }
tree-source-missing = falta la línea de alineación

object-measurement = Medición { $number }
object-measurement-with-length = Medición { $number } ({ $length } { $unit })
object-area = Área { $number }
object-area-with-value = Área { $number } ({ $value } { $unit })
object-shape-line = Forma { $number } (línea)
object-shape-area = Forma { $number } (área)

## Tool controls

help-no-document = Abra una imagen para empezar
help-pan = Arrastre para desplazar la vista. Use la rueda para ampliar.
help-origin = Haga clic en la imagen para mover el origen, el punto sobre el que gira la vista.

new-measurement = Nueva medición
new-measurement-unit-tooltip = Unidades de las nuevas mediciones
new-area = Nueva área
new-area-unit-tooltip = Unidades de las nuevas áreas
new-shape = Nueva forma

rotation-label = Giro de la vista (°):

scale-problem = Escala: { $reason }
scale-result = X: { $x } { $unit }/px   Y: { $y } { $unit }/px
scale-units-one-tooltip = Unidades de la distancia A
scale-units-two-tooltip = Unidades de las distancias A y B

grid-title = Cuadrícula { $number }
grid-hidden = Cuadrícula no mostrada: { $reason }
grid-too-small = las celdas miden menos de { $spacing } px en pantalla; amplíe para verlas
grid-side-tooltip = Longitud del lado de cada celda cuadrada
grid-size-a-tooltip = Tamaño de celda a lo largo de la línea de alineación
grid-size-b-tooltip = Tamaño de celda perpendicular a la línea de alineación
grid-unit-tooltip = Unidades del tamaño de celda
grid-shape-label = Forma de celda:
grid-color-label = Color:
grid-color-tooltip = Elegir el color de la cuadrícula
grid-opacity-label = Opacidad:
grid-via-scale = Por escala
grid-via-measurement = Por medición
grid-hint-place-scale = Coloque primero los puntos de escala (herramienta Escala).
grid-hint-click-scale = Haga clic en una línea de escala para añadir una cuadrícula alineada con ella.
grid-hint-add-measurement = Añada primero una medición (herramienta Medir distancia).
grid-hint-click-measurement = Haga clic en una línea de medición para añadir una cuadrícula alineada con ella.

style-line-label = Línea:
style-line-color-label = Color de línea:
style-line-color-tooltip = Elegir el color de la línea
style-line-opacity-label = Opacidad de línea:
style-fill-label = Relleno:
style-fill-opacity-label = Opacidad de relleno:
style-fill-color-label = Color de relleno:
style-fill-lines-hidden = Líneas de relleno no mostradas: necesitan una escala ({ $reason })
style-spacing-label = Distancia:
style-spacing-tooltip = Distancia entre las líneas de relleno
style-spacing-unit-tooltip = Unidades de la distancia entre las líneas de relleno
style-rotate-left-tooltip = Girar las líneas de relleno { $degrees }° a la izquierda
style-rotate-right-tooltip = Girar las líneas de relleno { $degrees }° a la derecha
style-angle-label = Ángulo:
style-angle-tooltip = Ángulo de las líneas de relleno, en grados en sentido horario desde la vertical
style-fill-locked-tooltip = El relleno usa el color de la línea. Haga clic para elegir uno propio
style-fill-locked-swatch-tooltip = Haga clic en el candado para elegir el color del relleno
style-fill-own-tooltip = El relleno tiene color propio. Haga clic para usar el color de la línea y olvidar este
style-fill-own-swatch-tooltip = Elegir el color del relleno

## Names of things in the document

method-two-point = 2 puntos
method-three-point = 3 puntos
method-four-point = 4 puntos

cell-shape-square = Cuadrado
cell-shape-rectangle = Rectángulo

line-style-thin = Fina
line-style-thick = Gruesa
line-style-dashed = Discontinua

fill-style-none = Ninguno
fill-style-solid = Sólido
fill-style-lines = Líneas
fill-style-crosshatch = Trama cruzada

endpoint-start = inicio
endpoint-end = fin

target-measurement = medición { $number }
target-area = área { $number }
target-shape = forma { $number }

## Why something could not be done

error-point-outside = el punto { $point } está fuera del documento
error-no-scale-points = no hay puntos de escala que mover
error-invalid-distance = la distancia debe ser un número positivo, se recibió { $distance }
error-measurement-missing = la medición { $number } no existe (hay { $count })
error-area-missing = el área { $number } no existe (hay { $count })
error-area-point-missing = el punto de área { $number } no existe (hay { $count })
error-area-point-count = a un área de { $expected } puntos no se le pueden dar { $actual } puntos
error-grid-missing = la cuadrícula { $number } no existe (hay { $count })
error-grid-source-missing = la línea de alineación de la cuadrícula no existe
error-invalid-cell-size = el tamaño de celda debe ser un número positivo, se recibió { $size }
error-invalid-opacity = la opacidad debe estar entre 0 y 1, se recibió { $opacity }
error-shape-missing = la forma { $number } no existe (hay { $count })
error-shape-point-missing = el punto de forma { $number } no existe (hay { $count })
error-shape-point-count = a una forma de { $expected } puntos no se le pueden dar { $actual } puntos
error-no-fill = { $target } no encierra ningún área, así que no tiene relleno
error-invalid-fill-spacing = la separación del relleno debe ser un número positivo, se recibió { $spacing }
error-invalid-fill-angle = el ángulo del relleno debe ser de 0 a menos de 360, se recibió { $degrees }

error-area-too-few-points = un área necesita al menos { $minimum } puntos, se recibieron { $count }
error-shape-too-few-points = una forma no puede tener solo { $count } puntos

error-calibration-point-count = la calibración de { $method } necesita { $expected } puntos, se recibieron { $actual }
error-calibration-point-missing = el punto { $number } no existe (hay { $count })

error-scale-missing-points = coloque los puntos de calibración
error-scale-missing-distance = introduzca la distancia de la línea { $line }
error-scale-zero-length = la línea { $line } tiene longitud cero
error-scale-indeterminate = las líneas A y B necesitan orientaciones distintas para separar X e Y
error-scale-inconsistent = las distancias no se ajustan a las líneas

error-grid-source-gone = su línea de alineación ya no existe
error-grid-source-zero-length = su línea de alineación tiene longitud cero
error-grid-no-scale = necesita una escala ({ $reason })
error-grid-too-dense = las celdas son demasiado pequeñas para dibujarlas

error-image-unsupported = formato de imagen no admitido: { $path }
error-image-read = no se pudo leer el archivo: { $error }
error-image-decode = no se pudo descodificar la imagen: { $error }

error-document-empty = el documento no tiene tamaño
error-document-point-outside = el punto { $point } está fuera del documento
error-document-scale-point-count = el número de puntos de escala no coincide con el método
error-document-two-point-version = esta versión del archivo no admite una escala de 2 puntos
error-document-area-points = un área tiene { $count } puntos, pero necesita al menos 3
error-document-shape-points = una forma tiene demasiados pocos puntos ({ $count })
error-document-filled-open-shape = la forma { $id } tiene relleno pero no está cerrada
error-document-not-positive = { $value } debería ser un número positivo
error-document-opacity = la opacidad { $value } no está entre 0 y 1
error-document-fill-angle = el ángulo de relleno { $value } está fuera de rango
error-document-object-id = el identificador de objeto { $id } no es válido o está repetido

error-save-encode = no se pudo codificar el documento: { $error }
error-save-write = no se pudo escribir el archivo: { $error }

error-load-read = no se pudo leer el archivo: { $error }
error-load-parse = el archivo no es un documento de GridTool: { $error }
error-load-invalid = el documento no es válido: { $reason }
error-load-image = no se pudo abrir la imagen { $path }: { $error }
error-load-image-size = la imagen { $path } mide { $actual_width }×{ $actual_height } píxeles, pero el documento espera { $expected_width }×{ $expected_height }

## The history panel

action-set-scale-points = Colocar puntos de escala de { $method }
action-move-scale-point = Mover punto de escala { $number } a { $to }
action-set-distance = Establecer distancia { $line } en { $distance }
action-clear-distance = Borrar distancia { $line }
action-set-unit = Establecer unidades en { $unit }
action-set-origin = Establecer origen en { $origin }
action-add-measurement = Añadir medición de { $start } a { $end }
action-move-measurement-point = Mover { $endpoint } de la medición { $number } a { $to }
action-resize-measurement = Redimensionar medición { $number } a { $start } - { $end }
action-move-measurement = Mover medición { $number } a { $start } - { $end }
action-set-measurement-unit = Establecer unidades de la medición { $number } en { $unit }
action-delete-measurement = Eliminar medición { $number }
action-duplicate-measurement = Duplicar medición { $number }
action-add-area = Añadir área de { $count } puntos
action-move-area-point = Mover punto { $point } del área { $number } a { $to }
action-resize-area = Redimensionar área { $number }
action-move-area = Mover área { $number }
action-set-area-unit = Establecer unidades del área { $number } en { $unit }
action-delete-area = Eliminar área { $number }
action-duplicate-area = Duplicar área { $number }
action-add-grid-scale-line = Añadir cuadrícula alineada con la línea de escala { $line }
action-add-grid-measurement = Añadir cuadrícula alineada con una medición
action-set-grid-shape = Establecer forma de celda de la cuadrícula { $number } en { $shape }
action-set-grid-size = Establecer tamaño { $dimension } de la cuadrícula { $number } en { $size }
action-set-grid-unit = Establecer unidades de la cuadrícula { $number } en { $unit }
action-set-grid-color = Establecer color de la cuadrícula { $number } en { $color }
action-set-grid-opacity = Establecer opacidad de la cuadrícula { $number } en { $percent }%
action-delete-grid = Eliminar cuadrícula { $number }
action-duplicate-grid = Duplicar cuadrícula { $number }
action-add-closed-shape = Añadir forma cerrada de { $count } puntos
action-add-line = Añadir línea de { $count } puntos
action-move-shape-point = Mover punto { $point } de la forma { $number } a { $to }
action-resize-shape = Redimensionar forma { $number }
action-move-shape = Mover forma { $number }
action-delete-shape = Eliminar forma { $number }
action-duplicate-shape = Duplicar forma { $number }
action-set-style = Cambiar en { $target }: { $edit }

edit-line-style = estilo de línea a { $style }
edit-line-color = color de línea a { $color }
edit-line-opacity = opacidad de línea a { $percent }%
edit-fill-style = estilo de relleno a { $style }
edit-fill-color = color de relleno a { $color }
edit-fill-color-none = color de relleno a ninguno
edit-fill-spacing = separación del relleno a { $spacing }
edit-fill-unit = unidades del relleno a { $unit }
edit-fill-angle = ángulo del relleno a { $degrees }°
edit-fill-opacity = opacidad de relleno a { $percent }%
