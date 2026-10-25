---
title: "API de Lua"
description: "Referencia completa de la API ms de Lua 5.4 con firmas, valores de retorno, ejemplos y notas de compatibilidad de Windows/macOS."
order: 9
---

# API de Lua

Estas son todas las funciones registradas en `ms` por el tiempo de ejecución compartido. Los ejemplos son instrucciones para colocar dentro de `ms_action`, donde `pos`, `wid`, `pid`, `exe`, `title` y `mode` están disponibles. Consulta [Fundamentos de Lua](lua-basics.md) para un script completo y las [recetas](lua-examples.md) para acciones listas para usar.

## Convenciones

Versión de Lua: **5.4** (`mlua` con `lua54` y `vendored`). `nil` significa ausencia de valor; `?` marca un parámetro opcional. Las funciones que no devuelven valores se muestran como `-> nil`. Un argumento opcional omitido equivale a nil. Pasa un booleano real para los argumentos booleanos. Los parámetros enteros truncan las fracciones finitas hacia cero y rechazan los valores fuera de su rango entero subyacente; usa enteros en los scripts.

Las tablas de posición usan campos con nombre `{x=..., y=...}`, no elementos de array. Las llamadas no admitidas en Mac suelen registrar un aviso por función y proceso; los getters devuelven nil y los setters no hacen nada. Un setter sin valor devuelto no es una comprobación de éxito. Las cadenas de la API de Lua no reciben sustitución de variables de acción.

## Flujo y respuesta

### `ms.log`

`ms.log(...) -> nil`

Acepta cualquier valor de Lua. Con una cadena de formato y más argumentos, admite `%s`, `%d`, `%f`, `%q` y `%%`; en caso contrario une los valores con espacios. `%q` no añade entrecomillado de Lua. Escribe una línea `Lua:` en el registro del tiempo de ejecución en ambas plataformas. No devuelve valor.

```lua
ms.log("pid=%d app=%s", pid, exe)
```

### `ms.sleep`

`ms.sleep(milliseconds: integer) -> nil`

Espera un número de milisegundos no negativo (64 bits sin signo). Bloquea este hilo de trabajo del script en ambas plataformas; el motor de gestos y el hilo principal continúan. Sin valor de retorno y sin tiempo límite de script.

```lua
ms.sleep(100)
```

### `ms.is_paused`

`ms.is_paused() -> boolean`

Sin parámetros. Devuelve el indicador de pausa manual del motor en ambas plataformas. No informa de la supresión automática a pantalla completa ni de una exclusión de aplicación.

```lua
ms.log(ms.is_paused())
```

### `ms.set_paused`

`ms.set_paused(paused: boolean) -> nil`

`true` pausa; `false` reanuda. Sin valor de retorno. Admitido en ambas plataformas. Windows envía un comando al hilo de la bandeja, por lo que un `is_paused()` inmediato todavía puede ver el valor antiguo. Mac lo aplica mediante el puente del hilo principal.

```lua
ms.set_paused(true)
```

### `ms.show_settings`

`ms.show_settings() -> nil`

Sin parámetros ni valor de retorno. Abre la configuración o la trae al frente en ambas plataformas.

```lua
ms.show_settings()
```

### `ms.show_toast`

`ms.show_toast(message: string, duration_ms?: integer) -> nil`

`message` es el texto que se muestra. La duración opcional, no negativa, está en milisegundos, por defecto **1200**. Muestra un mensaje superpuesto en ambas plataformas; no devuelve valor.

```lua
ms.show_toast("Done", 1500)
```

## Entrada de teclado

### `ms.keys`

`ms.keys(sequence: string, text_by_default?: boolean) -> nil`

Analiza y envía la [secuencia de teclas](key-sequences.md); el modo texto es `false` por defecto. Vuelve tras enviar; una sintaxis malformada lanza un error de Lua. Ambas plataformas admiten la entrada ordinaria de teclado y texto. Mac omite las teclas no admitidas, las acciones de ratón y rueda, y aplica sus límites de cola y texto. Espera en su cola de entrada del hilo principal, que también puede contener otras secuencias.

```lua
ms.keys("Hello", true)
```

### `ms.keys_down`

`ms.keys_down(sequence: string) -> nil`

Solo Windows: analiza en modo de teclas y pulsa teclas/modificadores sin soltarlos. Las repeticiones no se repiten en esta fase; las acciones de rueda se disparan una vez. Una sintaxis no válida lanza un error de Lua. Combínalo con `keys_up` para soltar. Mac registra que no está admitido y no hace nada. Sin valor de retorno.

```lua
ms.keys_down("^a")
ms.sleep(50)
ms.keys_up("^a")
```

### `ms.keys_up`

`ms.keys_up(sequence: string) -> nil`

Solo Windows: suelta la secuencia analizada en orden inverso de acciones, con los modificadores al final. Los pasos de espera siguen esperando. Los errores de sintaxis lanzan un error de Lua. Mac registra que no está admitido y no hace nada. Sin valor de retorno.

```lua
ms.keys_down("{SHIFT}")
ms.keys_up("{SHIFT}")
```

### `ms.is_keys_pressed`

`ms.is_keys_pressed(sequence: string) -> boolean | nil`

Ambas plataformas comprueban cada tecla del teclado **y sus modificadores de prefijo**: `"^a"` exige Control y A. Cualquier lado satisface un prefijo; los nombres explícitos como `{RCTRL}` exigen ese lado. En Mac `^` = Control, `%` = Option, `+` = Shift y `#` = Command. Varios pasos significan que todas las teclas/modificadores listados deben estar pulsados a la vez, no que se pulsaran en orden; las repeticiones no añaden requisitos. Las acciones de ratón/rueda, espera o texto hacen que la comprobación devuelva false; una secuencia vacía devuelve true. Una sintaxis no válida devuelve false. Mac lee el estado combinado de la sesión y devuelve false sin permiso de Accesibilidad, registrándolo una vez.

```lua
if ms.is_keys_pressed("^a") then
  ms.show_toast("Ctrl and A are down")
end
```

## Portapapeles y archivos

### `ms.get_clipboard_text`

`ms.get_clipboard_text() -> string | nil`

Sin parámetros. Devuelve el texto del portapapeles, o `nil` si no hay texto disponible. Admitido en ambas plataformas; esto no obtiene la selección automáticamente.

```lua
local text = ms.get_clipboard_text()
ms.log(text or "No text")
```

### `ms.set_clipboard_text`

`ms.set_clipboard_text(text: string) -> nil`

Sustituye el contenido del portapapeles por texto en ambas plataformas; sin valor de retorno ni indicador de éxito. Los demás formatos del portapapeles no se conservan. Consulta las [recetas del portapapeles](lua-examples.md).

```lua
ms.set_clipboard_text("Hello from Mousetroke")
```

### `ms.read_text`

`ms.read_text(path: string) -> string | nil`

Lee el archivo completo como UTF-8 en ambas plataformas. Devuelve `nil` ante un error de lectura o de decodificación UTF-8. Las rutas relativas usan el directorio de trabajo del proceso, no una carpeta de scripts documentada. Las lecturas cuentan para la memoria disponible.

```lua
local text = ms.read_text("config.json")
ms.log(text and #text or 0)
```

## Windows

### `ms.find_win_by_class`

`ms.find_win_by_class(pattern: string) -> table | nil`

Busca ventanas visibles de primer nivel con una **expresión regular de Rust**, sobre nombres de clase en Windows o **ID de bundle** de aplicación en Mac (por ejemplo `^com\\.apple\\.Safari$` en una cadena de Lua). Devuelve hasta 32 entradas en un array con base 1: `{text=title, hwnd=window_id}`; sin coincidencias o con una regex no válida devuelve una tabla vacía. La coincidencia distingue mayúsculas de minúsculas salvo que el patrón use una marca como `(?i)`; no son patrones de Lua. Mac lee los títulos mediante Accesibilidad, devolviendo nil sin permiso y registrándolo una vez. Su `hwnd` es un CGWindowID aceptado por `focus_win` y las API de estado de ventana. En Mac solo se buscan las ventanas actualmente en pantalla.

```lua
local wins = ms.find_win_by_class("^Notepad$")
if wins and wins[1] then ms.focus_win(wins[1].hwnd) end
```

### `ms.find_win_by_title`

`ms.find_win_by_title(pattern: string) -> table | nil`

Busca títulos de ventana en ambas plataformas con una expresión regular de Rust, devolviendo las mismas entradas en base 1 `{text, hwnd}` que `find_win_by_class`, como máximo 32. Una regex no válida devuelve una tabla vacía. Mac usa títulos de ventanas AX, requiere el permiso de Accesibilidad y busca solo las ventanas actualmente en pantalla.

```lua
local wins = ms.find_win_by_title("(?i)notepad")
if wins then ms.log(#wins) end
```

### `ms.focus_win`

`ms.focus_win(wid: integer) -> nil`

Activa una ventana o aplicación; sin valor de retorno ni indicador de éxito. Windows toma un HWND. Mac acepta el objetivo del gesto o un CGWindowID descubierto, desminimiza y eleva su ventana AX y luego activa la aplicación propietaria mediante el puente del hilo principal. Las llamadas AX de Mac se ejecutan en el hilo de trabajo del script, reutilizan la búsqueda integrada de ventanas por PID/límites y requieren el permiso de Accesibilidad; los IDs obsoletos o las ventanas AX no disponibles se ignoran. Las aplicaciones con límites de ventana solapados idénticos pueden dar lugar a ambigüedad.

```lua
if wid ~= 0 then ms.focus_win(wid) end
```

### `ms.set_win_state`

`ms.set_win_state(wid: integer, state: string) -> nil`

Windows: `"min"` minimiza, `"max"` maximiza; **cualquier otra cadena restaura**, incluidos `"normal"` y `"norm"`. Mac: `"min"` usa AXMinimized; `"max"` desminimiza, sale de la pantalla completa y pulsa el mismo AXZoomButton que la acción integrada cuando la ventana no llena el área de trabajo de su pantalla. `"normal"`, `"norm"` y `"restore"` desminimizan, salen de la pantalla completa y deshacen la ampliación al área de trabajo. Mac también acepta `"fullscreen"` mediante AXFullScreen donde la aplicación lo admite; cualquier otro nombre se ignora y se registra una vez. Los valores distinguen mayúsculas de minúsculas. Sin valor de retorno. Mac requiere el permiso de Accesibilidad; AX se ejecuta en el hilo de trabajo y la geometría de pantalla llega mediante el puente del hilo principal. Las transiciones pueden ser asíncronas.

El Zoom de Mac lo define cada aplicación: una ampliación al tamaño del contenido puede seguir informando `"normal"` y no puede restaurarse de forma fiable con esta API; las peticiones `"max"` repetidas pueden alternarlo. Para esas aplicaciones usa el conmutador integrado **Maximizar / restaurar**. No existe ninguna marca pública universal de estado de zoom AX.

```lua
if wid ~= 0 then ms.set_win_state(wid, "max") end
```

### `ms.get_win_state`

`ms.get_win_state(wid: integer) -> string | nil`

Windows devuelve `"min"`, `"max"` o `"normal"`. No es una consulta de siempre encima ni valida un identificador obsoleto; un identificador no válido puede parecer normal. Mac devuelve `"min"`, `"fullscreen"`, `"max"` (el marco coincide con el área de trabajo de una pantalla dentro de 2 puntos lógicos) o `"normal"`. Mac no tiene un getter AX público universal de estado de zoom, de modo que la ampliación al tamaño del contenido informa normal. Devuelve nil ante un permiso ausente, una ventana o estado AX no disponible o un ID obsoleto; la denegación de permiso se registra una vez.

```lua
if wid ~= 0 then ms.log(ms.get_win_state(wid)) end
```

### `ms.set_win_top`

`ms.set_win_top(wid: integer, top: boolean) -> nil`

Windows activa (`true`) o desactiva (`false`) el siempre encima sin mover ni redimensionar la ventana; sin valor de retorno. Esto fija un estado: no lo alterna ni devuelve el estado anterior. Mac no hace nada y lo registra una vez: macOS no tiene una API pública para fijar la ventana de otra aplicación encima, y la acción integrada de siempre encima tampoco está admitida. Para alternar, elige **Acciones de ventana → Siempre encima / quitar** en el editor de Windows.

```lua
if wid ~= 0 then ms.set_win_top(wid, true) end
```

## Ratón y pantalla

### `ms.get_mouse_pos`

`ms.get_mouse_pos() -> table`

Sin parámetros. Ambas plataformas devuelven `{x=integer, y=integer}` para la posición actual del cursor, que puede diferir del `pos` de entrada. Windows usa píxeles de pantalla; Mac usa puntos lógicos globales de Quartz, con Y creciendo hacia abajo. Mac devuelve `{x=0,y=0}` si falla la consulta del cursor.

```lua
local p = ms.get_mouse_pos()
ms.log("%d,%d", p.x, p.y)
```

### `ms.set_mouse_pos`

`ms.set_mouse_pos(x: integer, y: integer) -> nil`

Mueve el puntero a coordenadas globales de pantalla en ambas plataformas; sin valor de retorno. Las coordenadas pueden ser negativas. Mac envía un evento de movimiento etiquetado para no iniciar otro gesto.

```lua
local p = ms.get_mouse_pos()
ms.set_mouse_pos(p.x + 20, p.y)
```

### `ms.get_screen`

`ms.get_screen(x: integer, y: integer) -> table | nil`

Devuelve el rectángulo completo de la pantalla `{left, top, right, bottom}` como coordenadas globales enteras, no el área de trabajo utilizable. El ancho es `right-left`; el alto, `bottom-top`. Windows elige el monitor más cercano si el punto está fuera de todos los monitores. Mac elige la pantalla que contiene el punto o recurre a la primera pantalla en caché; puede devolver `nil` si no hay resultado de pantalla o del puente. Ambas plataformas admiten esta llamada.

```lua
local p = ms.get_mouse_pos()
local s = ms.get_screen(p.x, p.y)
if s then ms.log(s.right - s.left, s.bottom - s.top) end
```

### `ms.get_brightness`

`ms.get_brightness(pos?: table) -> integer | nil`

Un `{x=integer,y=integer}` opcional elige la pantalla; si se omite, es nil o la posición está incompleta, se usa el cursor actual. Devuelve un **porcentaje de 0 a 100 en caso de éxito, nil en caso de error** en ambas plataformas. Windows usa DDC/CI con una alternativa WMI. WMI usa su primer dispositivo de brillo, no el monitor del cursor. Mac actualmente devuelve nil y lo registra una vez: su backend de brillo existente solo envía eventos de las teclas de brillo de la pantalla integrada, que no permiten leer un porcentaje. El DDC externo no está admitido. Solo el envoltorio heredado de Windows `WG.GetBrightness` conserva -1 en caso de error.

```lua
local value = ms.get_brightness()
if value ~= nil then ms.show_toast(tostring(value)) end
```

### `ms.set_brightness`

`ms.set_brightness(pos: table | nil, delta: integer) -> integer | nil`

A pesar del nombre, esto ajusta en `delta` puntos porcentuales; no fija un porcentaje absoluto. Pasa una tabla con nombre `{x=...,y=...}` o **nil como primer argumento** para usar el cursor. El éxito devuelve el nuevo **porcentaje de 0 a 100**, acotado a ese rango; el error devuelve **nil en ambas plataformas**. Windows usa la misma selección DDC/CI y WMI que `get_brightness`. Solo el envoltorio heredado de Windows `WG.SetBrightness` conserva -1 en caso de error. Mac devuelve nil y lo registra una vez: el backend existente de Medios y sistema solo envía las teclas de brillo de la pantalla integrada, sin lectura numérica ni control por puntos porcentuales. Esta llamada de Lua no cambia el brillo; usa esas acciones integradas para las teclas de brillo. El DDC externo sigue sin estar admitido.

```lua
local value = ms.set_brightness(nil, 5)
ms.log(value)
```

## URL y procesos

### `ms.open`

`ms.open(url: string) -> nil`

Abre una URL con el controlador predeterminado; sin valor de retorno ni indicador de éxito. Windows usa la apertura del shell y también acepta una ruta de archivo o carpeta. Mac espera una URL; usa una URL `file://` correctamente codificada para un archivo, o `ms.cmd` para invocar `/usr/bin/open` sobre una ruta. No se realiza ninguna sustitución de variables.

```lua
ms.open("https://mousetroke.com/")
```

### `ms.run`

`ms.run(file: string, args?: string) -> nil`

Lanza con una **cadena** de argumentos opcional, no un array de Lua; no devuelve valor ni PID. Windows usa un lanzamiento ordinario sin elevar con comportamiento RunAgain. Mac acepta un ID de bundle, una ruta de aplicación o una ruta de ejecutable absoluta; los argumentos se dividen gestionando las comillas. El lanzamiento de ejecutables puede recurrir a un proceso en segundo plano si LaunchServices lo rechaza. Solicita una instancia nueva. Sin expansión de variables; usa concatenación de Lua. No se devuelve la salida del proceso.

```lua
if package.config:sub(1, 1) == "\\" then
  ms.run("cmd.exe", "/K echo Hello")
else
  ms.run("/usr/bin/say", '"Hello from Mousetroke"')
end
```

### `ms.cmd`

`ms.cmd(command: string) -> nil`

Ejecuta código de shell mediante un `cmd.exe /C` oculto en Windows o `/bin/zsh -c` en Mac. No devuelve valor, salida ni código de salida, y no espera a que el proceso termine. Mac limita a 16 los procesos en segundo plano gestionados simultáneamente y registra la finalización. Las cadenas se ejecutan tal cual; los marcadores `%MS_*%` no los expande esta API de Lua. Entrecomilla tú mismo los argumentos del shell.

```lua
if package.config:sub(1, 1) == "\\" then
  ms.cmd("taskmgr.exe")
else
  ms.cmd('/usr/bin/open -a "Activity Monitor"')
end
```

Continúa con la [sintaxis de envío de teclas](key-sequences.md), las [recetas](lua-examples.md) o las [ubicaciones de los registros](config.md#ubicaciones-de-archivos).
