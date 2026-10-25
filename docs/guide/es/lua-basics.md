---
title: "Fundamentos de Lua"
description: "Escribe tu primer script de Lua para gestos de ratón, aprende los argumentos de ms_action, depura con registros y conoce los límites del tiempo de ejecución."
order: 8
---

# Fundamentos de Lua

## Cuándo usar Lua

Usa una [acción sencilla](actions.md) para un atajo, una operación de ventana, una URL, una aplicación o un comando. Elige Lua cuando un gesto deba ramificarse según la aplicación, transformar texto o combinar operaciones. Mousetroke integra **Lua 5.4**.

## Tu primer script

1. Crea o selecciona un gesto en la configuración.
2. En **Acción**, elige **Script de Lua**. Sustituye todo el área de texto por el código siguiente.
3. Espera la indicación de guardado y luego dispara el gesto fuera de la configuración.

```lua
function ms_action(pos, wid, pid, exe, title, mode)
  ms.log("Target: %s, mode: %s", exe, mode)
  ms.show_toast("My first Lua gesture", 1500)
end
```

`function` comienza una función y `end` la termina. Mousetroke carga el código y luego llama a `ms_action` una vez. Pon tu trabajo dentro de esa función. Los nombres entre paréntesis reciben el contexto; puedes omitir los argumentos que no uses. `ms.log` escribe en el registro del tiempo de ejecución y `ms.show_toast` muestra un mensaje durante 1500 milisegundos.

La opción **Introducción a Lua** del editor abre esta página. Sigue las [recetas de Lua](lua-examples.md) para que un gesto de ratón escriba texto o pegue a través del portapapeles.

## Argumentos de entrada

La entrada completa es `ms_action(pos, wid, pid, exe, title, mode)`:

| Argumento | Tipo de Lua | Significado |
|---|---|---|
| `pos` | tabla `{x=integer, y=integer}` | Última posición del cursor registrada al despachar la acción. Las coordenadas pueden ser negativas en otro monitor. |
| `wid` | integer | Identificador de la ventana objetivo; HWND en Windows, ID de ventana en Mac. Trátalo como un identificador opaco; puede ser 0 si no hay objetivo. |
| `pid` | integer | Id. de proceso del objetivo, o 0. |
| `exe` | string | Identidad del ejecutable en Windows, normalmente una ruta completa; ID de bundle o ruta de ejecutable en Mac. Vacía si no está disponible. |
| `title` | string | Título de la ventana objetivo, o una cadena vacía. Mac obtiene el título mediante Accesibilidad en el hilo de trabajo. |
| `mode` | string | Exactamente `UnderCursor` o `Focused`, según **Preferencias → Los gestos actúan sobre**. No es el botón disparador ni el nombre del gesto. |

El objetivo se captura para el gesto. `pos` puede diferir de las coordenadas de inicio del gesto de las variables de acción. La entrada de teclado sigue yendo a la aplicación activa salvo que actives el objetivo. En Mac, `ms.focus_win(wid)` puede activar solo la aplicación objetivo de este gesto. El texto de marcador como `%MS_PID%` no se expande en Lua; usa `pid` directamente.

## Una introducción a Lua en cinco minutos

`local` crea una variable. Las cadenas usan comillas simples o dobles; `..` concatena cadenas. `--` comienza un comentario. `if`, `elseif`, `else`, `end` eligen una rama. `nil` significa ausencia de valor; solo `false` y `nil` son falsos en las condiciones (0 y `""` son verdaderos).

```lua
function ms_action(pos, wid, pid, exe)
  local message = "Hello"
  if string.find(string.lower(exe), "firefox", 1, true) then
    message = "Hello, Firefox"
  else
    message = message .. " from Mousetroke"
  end
  ms.show_toast(message)
end
```

Aquí `string.find(s, needle, 1, true)` busca literalmente y devuelve una posición o `nil`. El `true` evita interpretar la puntuación como un patrón. `string.match` usa patrones de Lua, que difieren de las expresiones regulares. Una tabla puede almacenar campos con nombre o una lista, cuyo primer índice es 1. Una función auxiliar puede devolver un valor:

```lua
local function app_name(identity)
  return string.match(identity, "([^/\\]+)$") or identity
end

function ms_action(pos, wid, pid, exe)
  local target = {name = app_name(exe), process = pid}
  local lines = {target.name, tostring(target.process)}
  ms.show_toast(table.concat(lines, " / "))
end
```

Las cadenas de Lua interpretan los escapes antes del análisis de la secuencia de teclas. `"\\"` representa una barra invertida y `"\n"` un salto de línea. Las cadenas con corchetes largos como `[[text]]` conservan las barras invertidas. La [guía de envío de teclas](key-sequences.md) explica la segunda capa de sintaxis.

## Depuración

Usa `ms.log("pid=%d title=%s", pid, title)` y consulta **Acerca de → Abrir registro**:

- Windows: `%APPDATA%\mousetroke\mousetroke.log`.
- Mac: `~/Library/Logs/mousetroke/mousetroke.log`.

Los errores de carga y de ejecución aparecen allí con `gesture_script` como nombre del fragmento. `ms.show_toast("Reached this step")` es una comprobación visual rápida. Los resultados de las API opcionales pueden ser `nil`; compruébalos antes de una aritmética o de acceder a una tabla. Consulta la [referencia de la API](lua-api.md) para las llamadas no admitidas en Mac.

## Tiempo de ejecución y límites

Cada ejecución de un script crea una máquina virtual de Lua nueva en un hilo de trabajo, con un **límite de asignación de Lua de 8 MiB**. Las variables globales no persisten entre gestos. El tiempo de ejecución no impone ningún límite de instrucciones ni de reloj. Evita los bucles infinitos. `ms.sleep(ms)` bloquea el hilo de trabajo de ese script, no el gancho de entrada ni el hilo de la interfaz principal. `ms.keys` espera a que su secuencia termine; en Mac, la entrada avanza por el temporizador del hilo principal mientras el hilo de Lua espera.

Las bibliotecas estándar disponibles son las funciones base, `coroutine`, `table`, `io`, `os`, `string`, `utf8`, `math` y `package`. `debug` no se carga; la carga de módulos C nativos está desactivada por el modo seguro de mlua. `os.date` está disponible. Esto no es un entorno aislado de seguridad del sistema operativo: `io`, `os` y las API de proceso pueden acceder a archivos o ejecutar comandos con tus derechos de usuario. Ejecuta solo scripts y configuraciones en los que confíes.

Siguiente: [referencia de la API](lua-api.md), [recetas para copiar y pegar](lua-examples.md), [solución de problemas](troubleshooting.md).
