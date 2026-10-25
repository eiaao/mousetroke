---
title: "Acciones"
description: "Elige enviar teclas, acciones de ventana, aplicaciones, URL, comandos, teclas multimedia, menús o Lua, y usa las variables de gesto."
order: 5
---

# Acciones

Selecciona un gesto y elige su acción en **Acción**. El editor ofrece ocho tipos. La mayoría de las tareas solo necesitan una acción sencilla; usa [Lua](lua-basics.md) cuando necesites condiciones u operaciones múltiples.

## Enviar teclas

Introduce una [secuencia de teclas](key-sequences.md), o haz clic en **Grabar**, pulsa el atajo y haz clic en **Hecho**. Por ejemplo, `^c` copia en Windows; `#c` copia en Mac. **Ayuda de sintaxis** abre la guía de sintaxis.

**Activar antes la ventana bajo el cursor** activa primero el objetivo del gesto. Sin esta opción, los atajos van a la aplicación que reciba la entrada de teclado. El objetivo también sigue **Preferencias → Los gestos actúan sobre**. No hay ninguna opción de texto plano en este editor; la [entrada en modo texto](key-sequences.md#texto-y-escape) está disponible mediante Lua.

## Acciones de ventana

Elige una operación, aplicada al objetivo del gesto:

| Operación | Comportamiento |
|---|---|
| Minimizar | Minimiza la ventana. |
| Maximizar / restaurar | Alterna entre el tamaño maximizado y el restaurado. |
| Cerrar | Pide a la ventana que se cierre. |
| Siempre encima / quitar | Alterna el siempre encima en Windows; no admitido y desactivado en el editor de Mac. |

El control de ventanas en Mac usa Accesibilidad. **Maximizar / restaurar** pulsa el botón de Zoom de la aplicación, de modo que el tamaño resultante sigue el comportamiento de esa aplicación; no solicita la pantalla completa nativa y omite las ventanas ya minimizadas. Las aplicaciones pueden diferir en qué operaciones de ventana exponen.

## Abrir una aplicación

**Aplicación** acepta un ejecutable en Windows; **Examinar…** lo selecciona. En Mac usa un ID de bundle como `com.apple.Safari`, una ruta `.app` o una ruta de ejecutable. **Argumentos** es opcional. En Mac, los argumentos se dividen gestionando las comillas; usa rutas completas para los ejecutables de línea de comandos.

**Si ya está en ejecución** ofrece **Cambiar a ella** o **Abrir otra instancia**. Windows también ofrece **Ejecutar como administrador**. Mac no admite la elevación ni las opciones de lanzamiento con filtro de ventana. No hay campos de clase/título/expresión regular en el editor de acciones actual.

## Abrir una URL

Rellena **URL** y, opcionalmente, **Navegador**. Deja Navegador vacío para usar el navegador predeterminado. Windows acepta un ejecutable de navegador; Mac acepta un ID de bundle de navegador o una ruta de aplicación. Los botones **Insertar** añaden marcadores para el texto seleccionado, el texto del portapapeles, el nombre de la aplicación, el Id. de proceso o el título de la ventana.

Para la búsqueda web con texto seleccionado, pon esta URL:

```text
https://www.google.com/search?q=%MS_SELECTED_TEXT%
```

Selecciona texto en una aplicación y luego realiza el gesto. El texto seleccionado se codifica como URL. Si ya empieza por `http://`, `https://` o `www.`, Mousetroke abre esa dirección directamente; a `www.` se le añade el prefijo `https://`.

## Ejecutar un comando

Introduce el comando en el área de texto. **Insertar** añade variables. Windows lo ejecuta mediante un `cmd.exe /C` oculto; Mac usa `/bin/zsh -c`. Esto es sintaxis de shell, así que entrecomilla correctamente los argumentos. El texto sustituido no se escapa para el shell; evita pasar texto seleccionado no fiable a los comandos.

Por ejemplo, el siguiente comando de Windows abre el Administrador de tareas:

```text
taskmgr.exe
```

En Mac, usa **Abrir una aplicación** con `com.apple.ActivityMonitor` en su lugar.

## Medios y sistema

El editor ofrece **Reproducir / pausar**, **Pista siguiente**, **Pista anterior**, **Subir volumen**, **Bajar volumen**, **Silenciar**, **Subir brillo** y **Bajar brillo**.

El brillo en Windows se ajusta en 10 puntos porcentuales usando DDC/CI con una alternativa WMI. El brillo en Mac envía las teclas de brillo de la pantalla integrada; el DDC/CI externo no está admitido. Estas acciones de sistema de Mac están disponibles aunque las funciones de Lua de brillo no lo estén. Consulta los [bordes de pantalla](screen-edges.md).

## Mostrar un menú

Haz clic en **+ Añadir un elemento de menú**, introduce el **Nombre del elemento de menú** y luego elige una acción dentro de cada elemento. Usa **Subir**, **Bajar** y **Eliminar el elemento de menú** para organizarlo. El menú aparece en la posición del gesto. El editor no expone ningún campo de número de columnas ni ofrece otro tipo de menú dentro de un elemento. Mac admite una sola columna; los ajustes de varias columnas se marcan como no admitidos.

## Script de Lua

Pega un script completo que defina `ms_action`. El editor enlaza a [Fundamentos de Lua](lua-basics.md), la [referencia de la API](lua-api.md) y los [ejemplos](lua-examples.md). El área de texto acepta Tab como dos espacios.

<a id="variables"></a>

## Variables

Usa estos marcadores en **Abrir una URL → URL**, **Ejecutar un comando** y **Abrir una aplicación → Aplicación / Argumentos**. No se expanden en las cadenas de teclas, el código Lua, los nombres de menú ni en Navegador.

| Variable | Valor |
|---|---|
| `%MS_SELECTED_TEXT%` | Selección capturada cuando la acción la necesita; vacía si no está disponible. |
| `%MS_CLIPBOARD_TEXT%` | Texto actual del portapapeles, capturado bajo demanda. |
| `%MS_PID%` | Id. de proceso del objetivo. |
| `%MS_EXE%` | Identidad del ejecutable objetivo: ruta en Windows; ID de bundle o ruta de ejecutable en Mac. |
| `%MS_APP_NAME%` | Parte del nombre de archivo de la identidad, que en Mac puede ser un ID de bundle. |
| `%MS_WID%` | Identificador de ventana de la plataforma como texto decimal. |
| `%MS_WIN_NAME%` | Título de la ventana objetivo; en Mac se obtiene mediante Accesibilidad cuando es necesario. |
| `%MS_MOUSE_X%`, `%MS_MOUSE_Y%` | Coordenadas de pantalla del inicio del gesto. |
| `%MS_MOUSE_Y_FLIP%` | Coordenada inferior de la pantalla virtual menos la coordenada Y del inicio del gesto. |
| `%MS_TARGET_MODE%` | `UnderCursor` o `Focused`, según la preferencia de objetivo. |

Las variables de objetivo no disponibles pueden faltar; los marcadores desconocidos quedan sin cambios. Solo el texto seleccionado recibe codificación de URL automática en una plantilla de URL. Las demás sustituciones se insertan tal cual. Lua usa los [argumentos de entrada](lua-basics.md#argumentos-de-entrada) y las funciones de `ms` en lugar de la expansión de marcadores.
