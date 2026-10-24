---
title: "Sintaxis de envío de teclas"
description: "Sintaxis completa de envío de teclas de Mousetroke: modificadores, grupos, teclas con nombre, repeticiones, retardos, entrada de ratón y atajos de Windows/Mac."
order: 7
---

# Sintaxis de envío de teclas

## Empezar con un atajo

En **Enviar teclas**, pega `^c` para enviar Ctrl+C en Windows, o `#c` para enviar Command+C en Mac. También puedes hacer clic en **Grabar**, pulsar la combinación y hacer clic en **Hecho**. Lua usa la misma sintaxis mediante `ms.keys(sequence)`.

Los caracteres significan, por lo general, teclas en las posiciones del teclado estadounidense. `abc` pulsa A, B, C seguidas; `ABC` en mayúsculas no añade Shift. Usa `+a` para Shift+A. Para Unicode o texto fijo, consulta el [texto y el escape](#texto-y-escape) y las [recetas con el portapapeles](lua-examples.md).

## Modificadores y grupos

| Prefijo | Windows | Mac |
|---|---|---|
| `^` | Ctrl | Control ⌃ |
| `%` | Alt | Option ⌥ |
| `+` | Shift | Shift ⇧ |
| `#` | Win | Command ⌘ |

Un prefijo se aplica a la tecla siguiente o a un grupo entre paréntesis. `^+t` envía Ctrl+Shift+T. `^(ac)` aplica Ctrl tanto a A como a C. `^ac` aplica Ctrl solo a A. Los grupos pueden anidarse; usa paréntesis equilibrados. Los modificadores de grupo se aplican a cada acción de tecla, así que no cuentes con que el modificador permanezca pulsado continuamente entre teclas. Un modificador suelto al final no es una pulsación de tecla: usa `{CTRL}`, por ejemplo, cuando necesites la tecla modificadora en sí.

`ms.is_keys_pressed` de Lua comprueba la unión de todas las teclas y modificadores en el momento de la consulta en ambas plataformas: `^ac` exige Control, A y C pulsados a la vez. Cualquier lado satisface un prefijo; los nombres explícitos `{LCTRL}` / `{RCTRL}` conservan su lado. Las repeticiones se ignoran; los pasos de espera, texto, ratón o rueda devuelven false. No reconoce una secuencia de pulsaciones anteriores.

## Teclas con nombre, repeticiones y retardos

Usa `{ENTER}` para Intro. Los nombres no distinguen mayúsculas de minúsculas. `{LEFT 3}` pulsa Izquierda tres veces. Un recuento positivo repite una tecla; un recuento cero o negativo se reduce a uno. `{SLEEP 100}` espera 100 ms; exige un entero no negativo. Separa un nombre y su argumento con un espacio.

Ejemplo: `^c{SLEEP 100}^v` copia, espera un momento y luego pega en Windows. El equivalente en Mac es `#c{SLEEP 100}#v`.

La tabla siguiente enumera los nombres que acepta el analizador, incluidos los alias y todos los nombres numerados de F y del teclado numérico. Una única letra ASCII, dígito o carácter de puntuación admitido del teclado estadounidense también puede aparecer entre llaves, como `{a}` o `{+}`.

| Nombres | Significado |
|---|---|
| `{ENTER}`, `{RETURN}` | Intro |
| `{ESC}` | Escape |
| `{TAB}` | Tab |
| `{BACKSPACE}` | Retroceso |
| `{SPACE}` | Espacio |
| `{CAPSLOCK}` | Bloq Mayús |
| `{INSERT}`, `{INS}` | Insertar (solo Windows) |
| `{DELETE}`, `{DEL}` | Suprimir / suprimir hacia adelante |
| `{HOME}`, `{END}` | Inicio / Fin |
| `{PGUP}`, `{PGDN}` | Re Pág / Av Pág |
| `{UP}`, `{DOWN}`, `{LEFT}`, `{RIGHT}` | Teclas de flechas |
| `{PRTSC}` | Imprimir pantalla (solo Windows) |
| `{BREAK}` | Pausa (solo Windows) |
| `{MEDIA_NEXT}`, `{MEDIA_PREV}`, `{MEDIA_STOP}`, `{MEDIA_PLAY}`, `{MEDIA_PLAY_PAUSE}` | Teclas multimedia (solo Windows en secuencias de teclas) |
| `{VOLUME_MUTE}`, `{VOLUME_DOWN}`, `{VOLUME_UP}` | Teclas de volumen (solo Windows en secuencias de teclas) |
| `{CTRL}`, `{ALT}`, `{SHIFT}` | Control / Alt (Option en Mac) / Shift |
| `{LCTRL}`, `{RCTRL}`, `{LSHIFT}`, `{RSHIFT}`, `{LALT}`, `{RALT}` | Teclas modificadoras izquierda/derecha; Alt es Option en Mac |
| `{LWIN}`, `{WIN}`, `{RWIN}` | Tecla Windows; Command en Mac |
| `{F1}`, `{F2}`, `{F3}`, `{F4}`, `{F5}`, `{F6}`, `{F7}`, `{F8}`, `{F9}`, `{F10}`, `{F11}`, `{F12}`, `{F13}`, `{F14}`, `{F15}`, `{F16}`, `{F17}`, `{F18}`, `{F19}`, `{F20}`, `{F21}`, `{F22}`, `{F23}`, `{F24}` | Teclas de función; Mac admite F1–F20 |
| `{N0}`, `{N1}`, `{N2}`, `{N3}`, `{N4}`, `{N5}`, `{N6}`, `{N7}`, `{N8}`, `{N9}` | Dígitos del teclado numérico 0–9 |
| `{MOUSE_LEFT}`, `{MOUSE_RIGHT}`, `{MOUSE_MIDDLE}`, `{MOUSE_X1}`, `{MOUSE_X2}` | Clics de ratón (solo Windows) |
| `{MOUSE_WHEEL}`, `{MOUSE_HWHEEL}` | Rueda: p. ej. {MOUSE_WHEEL 120}, {MOUSE_HWHEEL -120} (solo Windows) |
| `{SLEEP}` | Retardo: p. ej. {SLEEP 100}, en milisegundos |

## Botones y rueda del ratón

Windows admite `{MOUSE_LEFT}`, `{MOUSE_RIGHT}`, `{MOUSE_MIDDLE}`, `{MOUSE_X1}` y `{MOUSE_X2}` como clics. `{MOUSE_LEFT 2}` envía dos clics; no reposiciona el puntero.

`{MOUSE_WHEEL 120}` desplaza hacia arriba un escalón; `-120` hacia abajo. `{MOUSE_HWHEEL 120}` desplaza hacia la derecha; `-120` hacia la izquierda. El argumento es un delta con signo de la rueda, no un recuento de repeticiones. Esta es la convención de la entrada inyectada; difiere del signo que usan los pasos de rueda grabados en los gestos.

Mac analiza estos tokens pero omite las acciones de ratón y rueda durante la ejecución de la entrada. No pueden usarse para sintetizar clics o desplazamientos en Mac mediante `ms.keys`.

## Texto y escape

El modo texto solo está disponible con Lua `ms.keys(s, true)`; el editor de Enviar teclas usa siempre el modo de teclas. Los caracteres ordinarios se convierten en texto Unicode, pero `^`, `%`, `+`, `#`, los paréntesis y la llave de apertura siguen iniciando sintaxis. Los caracteres sueltos entre llaves siguen siendo acciones de tecla en el modo texto.

Para los atajos dentro del modo texto, pon la tecla entre llaves: `^{c}` envía Ctrl+C y `#{c}` envía Command+C. Un `^c` sin llaves en este modo emite el texto `c` y descarta el modificador. También puedes hacer una llamada aparte `ms.keys("^c")` en modo de teclas normal.

| Carácter literal | Escritura de la secuencia |
|---|---|
| `^` | `{^}` |
| `%` | `{%}` |
| `+` | `{+}` |
| `#` | `{#}` |
| `(` | `{(}` |
| `)` | `{)}` |
| `{` | `{{}` |
| `}` | `}` (sin llaves) |

La barra invertida no es un carácter de escape de las secuencias de teclas. En una cadena de Lua entre comillas, escribe `"\\"` para una barra invertida, o usa una cadena con corchetes largos. Para texto arbitrario que contenga símbolos de sintaxis, el [método del portapapeles](lua-examples.md#pegar-y-restaurar-el-texto-del-portapapeles) evita ambas capas de escape.

## Atajos habituales

Estos dependen de la configuración de atajos de la aplicación de destino y del sistema operativo.

| Finalidad | Secuencia en Windows | Secuencia en Mac |
|---|---|---|
| Copiar | `^c` | `#c` |
| Pegar | `^v` | `#v` |
| Cortar | `^x` | `#x` |
| Seleccionar todo | `^a` | `#a` |
| Deshacer | `^z` | `#z` |
| Nueva pestaña | `^t` | `#t` |
| Cerrar pestaña | `^w` | `#w` |
| Reabrir pestaña cerrada | `^+t` | `#+t` |
| Pestaña siguiente | `^{TAB}` | `^{TAB}` |
| Pestaña anterior | `^+{TAB}` | `^+{TAB}` |
| Cambiar de aplicación | `%{TAB}` | `#{TAB}` |
| Escritorio anterior/siguiente | `#^{LEFT}` / `#^{RIGHT}` | `^{LEFT}` / `^{RIGHT}` |
| Recorte de pantalla | `#+s` | `#+4` |
| Mostrar el escritorio | `#d` | `{F11}` |
| Administrador de tareas / Activity Monitor | `^+{ESC}` | Usa Abrir una aplicación: `com.apple.ActivityMonitor` |
| Bloquear la pantalla | `#l` | `^#q` |

## Por qué no se puede enviar Ctrl+Alt+Del

Ctrl+Alt+Del es la secuencia de atención segura de Windows. Windows ignora la entrada sintética ordinaria para esta secuencia, así que `^%{DEL}` no puede abrir esa pantalla de seguridad. Usa `^+{ESC}` para el Administrador de tareas, `#l` para bloquear, o una acción de **Ejecutar un comando** que contenga `shutdown /l` para cerrar la sesión (esto termina la sesión).

## Límites y notas de plataforma

El analizador compartido no tiene ningún límite global de longitud de cadena ni de recuento de repeticiones. Los recuentos de repetición y los retardos se guardan como valores de 32 bits sin signo; los deltas de rueda como valores de 32 bits con signo. Los valores fuera de rango pueden desbordarse al convertirse, así que usa recuentos y retardos pequeños y prácticos. Un nombre de tecla erróneo o un argumento obligatorio ausente produce un error de análisis; el editor valida mientras escribes.

Windows ejecuta las secuencias en hilos de trabajo y duerme durante los `{SLEEP ms}`. Mac envía la entrada en lotes dirigidos por temporizador. Las cadenas de acciones de Mac de más de 16 384 bytes se rechazan; para la entrada de Lua ya analizada, los límites son 16 384 acciones y 16 384 bytes por acción de texto. Su cola de espera admite hasta 64 entradas; la entrada sobrante se descarta y se registra.

En Mac, F1–F20, los dígitos del teclado numérico y las teclas de navegación asignadas están disponibles. Insert/INS, PRTSC, BREAK, F21–F24, los tokens multimedia/volumen y los tokens de ratón/rueda no están admitidos y se omiten. Usa **Medios y sistema** para multimedia y volumen. `ms.keys_down` y `ms.keys_up` son solo para Windows; consulta la [API de Lua](lua-api.md).
