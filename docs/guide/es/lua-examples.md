---
title: "Recetas de Lua"
description: "Copia scripts de Lua listos para usar en gestos de ratón: texto, pegado desde el portapapeles, fechas, búsqueda web, ventanas, brillo y pausa."
order: 10
---

# Recetas de Lua

Pega un bloque completo en la acción **Script de Lua** de un gesto. Cada bloque define `ms_action`. Prueba primero las recetas de texto en un documento desechable. [Fundamentos de Lua](lua-basics.md) explica el editor y [API de Lua](lua-api.md) enumera la compatibilidad por plataforma.

Las recetas multiplataforma usan `package.config` para elegir `^` (Control en Windows) o `#` (Command en Mac). Las recetas del portapapeles conservan **solo texto**, no imágenes ni otros formatos. Sus esperas de 200 ms son ejemplos, no una garantía para todas las aplicaciones; auméntalas si el destino es lento. Sustituyen el contenido del portapapeles temporalmente y restauran el texto guardado, aunque copies otra cosa durante ese intervalo.

## Escribir texto fijo

Finalidad: insertar una frase reutilizable con un solo gesto de ratón.

```lua
function ms_action()
  ms.keys("Hello from Mousetroke.", true)
end
```

El modo texto admite Unicode en ambas plataformas. Sigue analizando los símbolos de sintaxis; usa la receta del portapapeles para la puntuación arbitraria como `%`, `+` o las llaves.

## Pegar y restaurar el texto del portapapeles

Finalidad: pegar texto literal y luego restaurar el texto que estaba en el portapapeles.

```lua
function ms_action()
  local mod = package.config:sub(1, 1) == "\\" and "^" or "#"
  local saved = ms.get_clipboard_text()
  ms.set_clipboard_text("Price: 50% + tax {example}")
  ms.keys(mod .. "v")
  ms.sleep(200)
  ms.set_clipboard_text(saved or "")
end
```

Esto evita el escape de la secuencia de teclas para el texto insertado. Si no había texto en el portapapeles, el portapapeles final queda con texto vacío; los demás formatos originales no se pueden restaurar con estas API.

## Texto distinto por aplicación

Finalidad: usar la identidad del ejecutable objetivo del gesto para elegir una frase.

```lua
function ms_action(pos, wid, pid, exe)
  local name = string.lower(exe)
  local text = "Hello from Mousetroke"
  if string.find(name, "firefox", 1, true) then
    text = "Hello Firefox"
  elseif name == "com.apple.safari" then
    text = "Hello Safari"
  end
  ms.keys(text, true)
end
```

Las identidades de Windows suelen ser rutas completas de ejecutable; Mac normalmente suministra un ID de bundle. Esto comprueba el objetivo capturado, mientras que la escritura va a la aplicación activa. Usa primero `ms.focus_win(wid)` si quieres activar la aplicación objetivo.

## Insertar la fecha de hoy

Finalidad: insertar la fecha local como año-mes-día.

```lua
function ms_action()
  ms.keys(os.date("%Y-%m-%d"), true)
end
```

`os.date` está disponible en el tiempo de ejecución. Los caracteres `%` son sintaxis de formato de fecha de Lua; los dígitos y guiones resultantes son seguros para la entrada en modo texto.

## Buscar el texto seleccionado en un sitio

Finalidad: copiar la selección y abrir una consulta de Google codificada en UTF-8.

```lua
local function url_encode(text)
  return (text:gsub("([^A-Za-z0-9%-._~])", function(c)
    return string.format("%%%02X", string.byte(c))
  end))
end

function ms_action()
  local mod = package.config:sub(1, 1) == "\\" and "^" or "#"
  local saved = ms.get_clipboard_text()
  ms.set_clipboard_text("")
  ms.keys(mod .. "c")
  ms.sleep(200)
  local text = ms.get_clipboard_text() or ""
  ms.set_clipboard_text(saved or "")
  if text ~= "" then
    ms.open("https://www.google.com/search?q=" .. url_encode(text))
  else
    ms.show_toast("No selected text copied")
  end
end
```

Selecciona texto en la aplicación activa antes de disparar. Vaciar primero el portapapeles evita buscar texto antiguo del portapapeles si Copiar no hace nada. Lua no sustituye automáticamente el texto seleccionado; la acción más sencilla de [Abrir una URL](actions.md#abrir-una-url) puede recoger la selección por ti.

## Envolver el texto seleccionado entre comillas o corchetes

Finalidad: sustituir el texto seleccionado por una versión entre comillas en un campo editable.

```lua
function ms_action()
  local mod = package.config:sub(1, 1) == "\\" and "^" or "#"
  local saved = ms.get_clipboard_text()
  ms.set_clipboard_text("")
  ms.keys(mod .. "c")
  ms.sleep(200)
  local text = ms.get_clipboard_text() or ""
  if text ~= "" then
    ms.set_clipboard_text('"' .. text .. '"')
    ms.keys(mod .. "v")
    ms.sleep(200)
  end
  ms.set_clipboard_text(saved or "")
end
```

Para corchetes, sustituye la expresión de envoltura por `"[" .. text .. "]"`. Copiar debe dejar el texto seleccionado para que Pegar lo sustituya. Las páginas de solo lectura no se pueden editar así.

## Alternar el siempre encima

Finalidad: alternar el estado de siempre encima de una ventana de Windows. Elige **Acciones de ventana → Siempre encima / quitar** para esta receta: esa acción puede consultar y alternar el estado real. La API de Lua solo fija el estado pedido; no tiene ninguna consulta del estado superior y cada gesto recibe una máquina virtual nueva. Mac no admite el control del siempre encima entre aplicaciones.

Si quieres un gesto de Lua aparte para «fijar esta ventana» en Windows, usa:

```lua
function ms_action(pos, wid)
  if package.config:sub(1, 1) ~= "\\" then
    ms.show_toast("Always-on-top is unavailable on Mac")
    return
  end
  if wid ~= 0 then ms.set_win_top(wid, true) end
end
```

Usa `false` en lugar de `true` para un gesto aparte de desfijar. Un booleano global de Lua se reiniciaría en cada ejecución y no implementaría una alternancia.

## Maximizar o restaurar

Finalidad: alternar el estado maximizado de la ventana objetivo en Windows o la ampliación al área de trabajo en Mac.

```lua
function ms_action(pos, wid)
  if wid == 0 then return end
  local state = ms.get_win_state(wid)
  if state == nil then
    ms.show_toast("Window unavailable; check Accessibility permission on Mac")
    return
  end
  ms.set_win_state(wid, state == "max" and "normal" or "max")
end
```

Mac usa el mismo botón de zoom AX que la acción de ventana integrada. La consulta de estado reconoce las ventanas del tamaño del área de trabajo; las aplicaciones que amplían al tamaño de su contenido pueden seguir informando normal, así que usa **Acciones de ventana → Maximizar / restaurar** en esas aplicaciones. Un permiso de Accesibilidad ausente o una ventana no disponible da nil. No hay ninguna API de Lua de posición o tamaño de ventana para mover una ventana a otro monitor.

## Brillo de la pantalla bajo el cursor

Finalidad: subir en 5 puntos porcentuales el brillo de la pantalla bajo el cursor en Windows.

```lua
function ms_action()
  local p = ms.get_mouse_pos()
  local value = ms.set_brightness(p, 5)
  if value ~= nil then
    ms.show_toast("Brightness: " .. value)
  else
    ms.show_toast("Brightness API unavailable for this screen/platform")
  end
end
```

Cambia `5` por `-5` para un segundo gesto que baje el brillo. Windows necesita hardware compatible con DDC/CI o WMI; WMI recurre a su primer dispositivo de brillo, que puede ser otra pantalla. El éxito es un porcentaje de 0 a 100; el error es nil en ambas plataformas. Aquí Mac devuelve nil: su backend existente solo envía teclas de brillo y no puede devolver un porcentaje ni aplicar un delta en puntos porcentuales. Usa **Medios y sistema → Subir brillo / Bajar brillo** para las teclas de brillo de la pantalla integrada, sin selección de la pantalla del cursor ni DDC/CI externo.

## Abrir una carpeta

Finalidad: abrir la carpeta Descargas del usuario actual con el explorador de archivos de la plataforma.

```lua
function ms_action()
  if package.config:sub(1, 1) == "\\" then
    ms.cmd('explorer.exe "%USERPROFILE%\\Downloads"')
  else
    ms.cmd('/usr/bin/open "$HOME/Downloads"')
  end
end
```

Estas variables de entorno las expande el shell del sistema operativo, no las variables de acción de Mousetroke. Esto asume la ubicación habitual de Descargas.

## Ejecutar un programa con argumentos

Finalidad: pasar una cadena de argumentos a un programa.

```lua
function ms_action()
  if package.config:sub(1, 1) == "\\" then
    ms.run("cmd.exe", "/K echo Hello from Mousetroke")
  else
    ms.run("/usr/bin/say", '"Hello from Mousetroke"')
  end
end
```

Windows abre una ventana de comandos con un mensaje. Mac pronuncia la frase con `say`. `ms.run` acepta una sola cadena de argumentos, no una tabla, y no devuelve ningún identificador de proceso.

## Abrir el Administrador de tareas o Activity Monitor

Finalidad: abrir el visor de procesos de la plataforma.

```lua
function ms_action()
  if package.config:sub(1, 1) == "\\" then
    ms.keys("^+{ESC}")
  else
    ms.run("com.apple.ActivityMonitor")
  end
end
```

Windows usa Ctrl+Shift+Esc. Mac lanza Activity Monitor por ID de bundle.

## Bloquear la pantalla

Finalidad: bloquear la sesión actual con su atajo del sistema.

```lua
function ms_action()
  if package.config:sub(1, 1) == "\\" then
    ms.keys("#l")
  else
    ms.keys("^#q")
  end
end
```

Mac usa Control+Command+Q. Windows usa Win+L; Ctrl+Alt+Del no puede sintetizarse como una secuencia de teclas ordinaria.

## Pausar los gestos durante diez minutos

Finalidad: pausar temporalmente el motor y reanudarlo mientras Mousetroke sigue en ejecución.

```lua
function ms_action()
  ms.set_paused(true)
  ms.show_toast("Gestures paused for ten minutes", 2000)
  ms.sleep(10 * 60 * 1000)
  ms.set_paused(false)
  ms.show_toast("Gestures resumed", 1500)
end
```

Esto funciona en ambas plataformas y mantiene un hilo de Lua dormido durante diez minutos. No es un temporizador persistente: salir o reiniciar lo termina. Puedes reanudar manualmente durante la espera, pero el script también ejecuta la reanudación cuando termina su espera. Evita lanzar varias copias. Consulta los [límites del tiempo de ejecución](lua-basics.md#tiempo-de-ejecución-y-límites).
