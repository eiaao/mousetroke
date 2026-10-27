---
title: "Biblioteca de acciones"
description: "Elige acciones listas para usar, configura sus parámetros y consulta el catálogo completo para Windows y Mac."
order: 5.5
---

# Biblioteca de acciones

Las acciones predefinidas describen lo que quieres hacer, como cerrar una pestaña o centrar una ventana. Mousetroke guarda su ID y sus parámetros y elige la implementación de Windows o Mac al ejecutarlas. Los atajos de navegador y editor siguen dependiendo de que la aplicación receptora los admita.

## Elegir una acción

1. Crea o selecciona un gesto y haz clic en **Elegir acción**, o en **Cambiar** si ya tiene una acción.
2. Busca por nombre, descripción, alias o escritura del atajo, como `ctrl+w` en Windows o `cmd+w` / `⌘w` en Mac. La búsqueda abarca todas las categorías. Sin buscar, puedes explorar **Frecuentes**, **Navegador**, **Edición**, **Texto**, **Ventanas**, **Escritorio y sistema**, **Multimedia**, **Abrir** o **Mousetroke**.
3. Elige un resultado. Las acciones con parámetros abren un formulario: introduce texto, elige un formato de fecha o un servicio de búsqueda, o indica una URL, un archivo, una carpeta o una aplicación. Haz clic en **Usar esta acción** para confirmar. Las acciones no disponibles están desactivadas e indican la limitación de plataforma o que están previstas para la segunda fase.
4. Prueba el gesto fuera de la configuración. Si el nombre del gesto está vacío, adopta el de la acción seleccionada. Los elementos de menú usan el mismo selector.

Al abrir el selector, el foco se sitúa en el campo de búsqueda. Usa las flechas arriba y abajo para recorrer los resultados disponibles y Enter para elegir. Esc cierra primero el formulario de parámetros y después el selector. En los campos de texto de varias líneas, Ctrl+Enter o Command+Enter confirma el formulario.

**Convertir en acción avanzada** sustituye una acción predefinida por sus teclas, operación de ventana, URL, aplicación, comando o tecla del sistema editables para la plataforma actual. La confirmación explica que la conversión es irreversible: el resultado deja de seguir los valores predeterminados de plataforma o idioma. Las acciones nativas de distribución de ventanas, energía, texto literal, fecha y hora, copia sin formato, pausa y configuración conservan su forma predefinida; sus tarjetas explican por qué no se pueden convertir. La categoría **Avanzadas**, al final de la lista, ofrece envío de teclas, comandos, Lua y menús. La [referencia de acciones avanzadas](actions.md) también explica los editores existentes.

## Texto, ventanas y energía

- **Insertar texto** escribe el texto literalmente, incluidos espacios, saltos de línea, caracteres chinos, emoji y caracteres como `^+#{ENTER}`. **Pegar y restaurar portapapeles** es el método predeterminado: usa temporalmente el portapapeles y restaura su **texto** anterior tras una breve espera. Si contiene imágenes, HTML, listas de archivos u otros datos no textuales, o no se puede comprobar, el texto se escribe directamente sin modificar el portapapeles. `paste_delay_ms` solo aparece con `paste`: la interfaz permite 50–5000 ms y el valor predeterminado es 150 ms. **Escribir caracteres** introduce Unicode sin usar el portapapeles. Ambos métodos envían la entrada a la aplicación que recibe el teclado.
- **Insertar fecha y hora** usa el reloj local del sistema y el método de salida elegido. Los formatos incluyen fecha ISO, fecha china, fecha de EE. UU., hora y fecha con hora. Un patrón personalizado admite `YYYY`, `MM`, `DD`, `HH`, `mm` y `ss`, distinguiendo mayúsculas y minúsculas. Los demás caracteres se escriben literalmente; las comillas simples protegen texto literal y `''` inserta una comilla. Por ejemplo, `YYYY-MM-DD HH:mm:ss` genera una marca de tiempo. Se rechazan las comillas sin cerrar.
- **Copiar selección sin formato** envía Ctrl+C / Command+C, espera hasta 500 ms a que cambie la versión del portapapeles y después sustituye el contenido copiado por su texto sin formato. Si la copia no termina o no contiene texto, muestra un error y conserva el contenido del portapapeles. Selecciona texto y mantén esa aplicación enfocada antes de usarla.
- Las acciones de distribución usan el mismo objetivo del gesto que las demás acciones de ventana, según **Preferencias → Los gestos actúan sobre**. Windows usa la colocación nativa de ventanas; Mac usa Accesibilidad y el área útil de la pantalla. Los tamaños mínimos y las restricciones de cada aplicación pueden impedir ocupar exactamente media pantalla. Sal primero de la pantalla completa nativa de Mac. **Mover a la siguiente pantalla** recorre las pantallas en un orden estable, conserva la posición relativa y la maximización, y no hace nada si solo hay una pantalla.
- **Apagar pantalla** permite volver a encender las pantallas con una entrada. **Suspender** solicita la suspensión normal del sistema, que la cuenta o las políticas del sistema pueden impedir. Si falla una acción nativa, se escribe una línea de registro y aparece un breve mensaje en pantalla. **Pausar / reanudar gestos** usa el mismo estado de pausa que la bandeja y el atajo; utiliza cualquiera de ellos para reanudar los gestos pausados. **Abrir ajustes** abre la ventana de configuración habitual.

## Catálogo

Las tablas siguientes se generan a partir del registro e incluyen las 87 entradas. ✓ indica que la acción está implementada en esa plataforma; su disponibilidad sigue dependiendo de la aplicación, el hardware y los permisos necesarios. La hoja de consulta de gestos está prevista para la segunda fase.

<!-- action-library:generated:start -->

### Navegador

| Acción / ID | Función y ámbito de uso | Parámetros | Windows | Mac |
|---|---|---|---|---|
| Atrás<br>`browser.back` | Disponible en navegadores web. | — | ✓ | ✓ |
| Adelante<br>`browser.forward` | Disponible en navegadores web. | — | ✓ | ✓ |
| Actualizar<br>`browser.reload` | Disponible en navegadores web. | — | ✓ | ✓ |
| Recargar sin caché<br>`browser.hard_reload` | Disponible en navegadores web. | — | ✓ | ✓ |
| Nueva pestaña<br>`browser.new_tab` | Navegadores, editores y otras apps que admiten este atajo. | — | ✓ | ✓ |
| Cerrar pestaña<br>`browser.close_tab` | Navegadores, editores y otras apps que admiten este atajo. | — | ✓ | ✓ |
| Reabrir pestaña cerrada<br>`browser.reopen_tab` | Disponible en navegadores web. | — | ✓ | ✓ |
| Pestaña anterior<br>`browser.prev_tab` | Navegadores, editores y otras apps que admiten este atajo. | — | ✓ | ✓ |
| Pestaña siguiente<br>`browser.next_tab` | Navegadores, editores y otras apps que admiten este atajo. | — | ✓ | ✓ |
| Nueva ventana<br>`browser.new_window` | Navegadores, editores y otras apps que admiten este atajo. | — | ✓ | ✓ |
| Nueva ventana privada<br>`browser.private_window` | Chrome, Edge y Safari; Firefox usa otro atajo. | — | ✓ | ✓ |
| Ir a la barra de direcciones<br>`browser.address_bar` | Disponible en navegadores web. | — | ✓ | ✓ |
| Buscar en la página<br>`browser.find` | Navegadores, editores y otras apps que admiten este atajo. | — | ✓ | ✓ |
| Añadir página a favoritos<br>`browser.bookmark` | Disponible en navegadores web. | — | ✓ | ✓ |
| Acercar<br>`view.zoom_in` | Navegadores, editores y otras apps que admiten este atajo. | — | ✓ | ✓ |
| Alejar<br>`view.zoom_out` | Navegadores, editores y otras apps que admiten este atajo. | — | ✓ | ✓ |
| Restablecer zoom<br>`view.zoom_reset` | Navegadores, editores y otras apps que admiten este atajo. | — | ✓ | ✓ |

### Edición

| Acción / ID | Función y ámbito de uso | Parámetros | Windows | Mac |
|---|---|---|---|---|
| Ir arriba<br>`view.top` | Navegadores, editores y otras apps que admiten este atajo. | — | ✓ | ✓ |
| Ir abajo<br>`view.bottom` | Navegadores, editores y otras apps que admiten este atajo. | — | ✓ | ✓ |
| Subir una página<br>`view.page_up` | Navegadores, editores y otras apps que admiten este atajo. | — | ✓ | ✓ |
| Bajar una página<br>`view.page_down` | Navegadores, editores y otras apps que admiten este atajo. | — | ✓ | ✓ |
| Copiar<br>`edit.copy` | Navegadores, editores y otras apps que admiten este atajo. | — | ✓ | ✓ |
| Cortar<br>`edit.cut` | Navegadores, editores y otras apps que admiten este atajo. | — | ✓ | ✓ |
| Pegar<br>`edit.paste` | Navegadores, editores y otras apps que admiten este atajo. | — | ✓ | ✓ |
| Pegar y Entrar<br>`edit.paste_enter` | Navegadores, editores y otras apps que admiten este atajo. | — | ✓ | ✓ |
| Pegar como texto sin formato<br>`edit.paste_plain` | Apps con pegado sin formato; el atajo depende de la app. | — | ✓ | ✓ |
| Seleccionar todo<br>`edit.select_all` | Navegadores, editores y otras apps que admiten este atajo. | — | ✓ | ✓ |
| Deshacer<br>`edit.undo` | Navegadores, editores y otras apps que admiten este atajo. | — | ✓ | ✓ |
| Rehacer<br>`edit.redo` | Navegadores, editores y otras apps que admiten este atajo. | — | ✓ | ✓ |
| Guardar<br>`edit.save` | Navegadores, editores y otras apps que admiten este atajo. | — | ✓ | ✓ |
| Entrar<br>`key.enter` | Navegadores, editores y otras apps que admiten este atajo. | — | ✓ | ✓ |
| Retroceso<br>`key.backspace` | Navegadores, editores y otras apps que admiten este atajo. | — | ✓ | ✓ |
| Suprimir<br>`key.delete` | Navegadores, editores y otras apps que admiten este atajo. | — | ✓ | ✓ |
| Cancelar (Esc)<br>`key.escape` | Navegadores, editores y otras apps que admiten este atajo. | — | ✓ | ✓ |
| Pantalla completa<br>`view.fullscreen` | Navegadores, editores y otras apps que admiten este atajo. | — | ✓ | ✓ |

### Texto

| Acción / ID | Función y ámbito de uso | Parámetros | Windows | Mac |
|---|---|---|---|---|
| Insertar texto<br>`text.type` | Campos de texto; por defecto pega y restaura el portapapeles. | Texto (`text`): obligatorio<br>Método de entrada (`method`): Pegar y restaurar portapapeles (`paste`) / Escribir caracteres (`type`); predeterminado: Pegar y restaurar portapapeles<br>Espera tras pegar (ms) (`paste_delay_ms`): predeterminado: `150`; solo si Método de entrada es Pegar y restaurar portapapeles | ✓ | ✓ |
| Insertar fecha y hora<br>`text.datetime` | Campos de texto; usa la fecha y hora locales. | Formato de fecha y hora (`format`): 2026-10-24 (`date_iso`) / 2026年10月24日 (`date_local`) / 10/24/2026 (`date_us`) / 14:30 (`time`) / 2026-10-24 14:30 (`datetime`) / Personalizado (`custom`); predeterminado: 2026-10-24<br>Formato personalizado (`custom_format`): predeterminado: `YYYY-MM-DD`; solo si Formato de fecha y hora es Personalizado<br>Método de entrada (`method`): Pegar y restaurar portapapeles (`paste`) / Escribir caracteres (`type`); predeterminado: Pegar y restaurar portapapeles<br>Espera tras pegar (ms) (`paste_delay_ms`): predeterminado: `150`; solo si Método de entrada es Pegar y restaurar portapapeles | ✓ | ✓ |
| Buscar en la web<br>`text.search` | Usa el navegador predeterminado; selecciona texto en la app actual. | Buscador (`engine`): Bing (`bing`) / Google (`google`) / Baidu (`baidu`) / DuckDuckGo (`duckduckgo`) / Personalizado (`custom`); Baidu con la interfaz en chino simplificado; Google en los demás idiomas<br>URL personalizada (`url`): predeterminado: `https://www.google.com/search?q=%MS_SELECTED_TEXT%`; solo si Buscador es Personalizado | ✓ | ✓ |
| Traducir texto seleccionado<br>`text.translate` | Usa el navegador predeterminado; selecciona texto en la app actual. | Servicio de traducción (`service`): Traductor de Google (`google`) / DeepL (`deepl`) / Traductor de Bing (`bing`); predeterminado: Traductor de Google<br>Idioma de destino (`target_language`): Usar idioma de la interfaz (`ui`) / 简体中文 (`zh-CN`) / English (`en-US`) / español (`es-ES`) / Français (`fr-FR`) / Deutsch (`de-DE`) / 日本語 (`ja-JP`) / Português (Brasil) (`pt-BR`) / Русский (`ru-RU`) / Italiano (`it-IT`) / 한국어 (`ko-KR`); predeterminado: Usar idioma de la interfaz | ✓ | ✓ |
| Abrir URL seleccionada<br>`text.open_selected_url` | Usa el navegador predeterminado; selecciona texto en la app actual. | — | ✓ | ✓ |
| Copiar selección sin formato<br>`text.copy_plain` | Apps que copian texto; elimina el formato del portapapeles. | — | ✓ | ✓ |

### Ventanas

| Acción / ID | Función y ámbito de uso | Parámetros | Windows | Mac |
|---|---|---|---|---|
| Minimizar<br>`window.minimize` | Se aplica a la ventana objetivo del gesto. | — | ✓ | ✓<br>Requiere permiso de Accesibilidad. |
| Maximizar / restaurar<br>`window.toggle_maximize` | Se aplica a la ventana objetivo del gesto. | — | ✓ | ✓<br>Requiere permiso de Accesibilidad. |
| Cerrar ventana<br>`window.close` | Se aplica a la ventana objetivo del gesto. | — | ✓ | ✓<br>Requiere permiso de Accesibilidad. |
| Salir de la aplicación<br>`app.quit` | Se aplica a la ventana objetivo del gesto. | — | ✓ | ✓ |
| Terminar proceso<br>`app.kill` | Fuerza el cierre de la app objetivo; se perderá lo no guardado. | — | ✓ | ✓ |
| Siempre encima<br>`window.toggle_topmost` | Alternar siempre visible en la ventana objetivo. | — | ✓ | Esta acción solo es compatible con Windows. |
| Ajustar a la mitad izquierda<br>`window.snap_left` | Se aplica a la ventana objetivo del gesto. | — | ✓ | ✓<br>Requiere permiso de Accesibilidad. |
| Ajustar a la mitad derecha<br>`window.snap_right` | Se aplica a la ventana objetivo del gesto. | — | ✓ | ✓<br>Requiere permiso de Accesibilidad. |
| Centrar ventana<br>`window.center` | Se aplica a la ventana objetivo del gesto. | — | ✓ | ✓<br>Requiere permiso de Accesibilidad. |
| Mover a la siguiente pantalla<br>`window.next_monitor` | Se aplica a la ventana objetivo del gesto. | — | ✓ | ✓<br>Requiere permiso de Accesibilidad. |
| Ocultar aplicación actual<br>`app.hide` | Ocultar la aplicación actual. | — | Esta acción solo es compatible con macOS. | ✓ |

### Escritorio y sistema

| Acción / ID | Función y ámbito de uso | Parámetros | Windows | Mac |
|---|---|---|---|---|
| Cambiar de aplicación<br>`desktop.switch_app` | Acción del sistema; depende en parte de sus atajos configurados. | — | ✓ | ✓ |
| Vista de tareas / Mission Control<br>`desktop.task_view` | Acción del sistema; depende en parte de sus atajos configurados. | — | ✓ | ✓ |
| Mostrar el escritorio<br>`desktop.show` | Acción del sistema; depende en parte de sus atajos configurados. | — | ✓ | ✓ |
| Escritorio virtual anterior<br>`desktop.prev` | Acción del sistema; depende en parte de sus atajos configurados. | — | ✓ | ✓ |
| Escritorio virtual siguiente<br>`desktop.next` | Acción del sistema; depende en parte de sus atajos configurados. | — | ✓ | ✓ |
| Menú Inicio / Spotlight<br>`desktop.launcher` | Acción del sistema; depende en parte de sus atajos configurados. | — | ✓ | ✓ |
| Abrir gestor de archivos<br>`desktop.files` | Acción del sistema; depende en parte de sus atajos configurados. | — | ✓ | ✓ |
| Recorte<br>`capture.region` | Acción del sistema; depende en parte de sus atajos configurados. | — | ✓ | ✓ |
| Capturar y guardar pantalla<br>`capture.screen` | Acción del sistema; depende en parte de sus atajos configurados. | — | ✓ | ✓ |
| Herramientas de captura y grabación<br>`capture.tool` | Acción del sistema; depende en parte de sus atajos configurados. | — | ✓ | ✓ |
| Historial del portapapeles<br>`desktop.clipboard_history` | Abrir el historial del portapapeles. | — | ✓ | Esta acción solo es compatible con Windows. |
| Emojis y símbolos<br>`desktop.emoji` | Acción del sistema; depende en parte de sus atajos configurados. | — | ✓ | ✓ |
| Centro de notificaciones<br>`desktop.notifications` | Abrir el Centro de notificaciones. | — | ✓ | Esta acción solo es compatible con Windows. |
| Bloquear pantalla<br>`system.lock` | Acción del sistema; depende en parte de sus atajos configurados. | — | ✓ | ✓ |
| Administrador de tareas / Monitor de Actividad<br>`system.task_manager` | Acción del sistema; depende en parte de sus atajos configurados. | — | ✓ | ✓ |
| Diálogo de salida forzada<br>`system.force_quit_dialog` | Abrir el diálogo Forzar salida. | — | Esta acción solo es compatible con macOS. | ✓ |
| Configuración del sistema<br>`system.settings` | Acción del sistema; depende en parte de sus atajos configurados. | — | ✓ | ✓ |
| Calculadora<br>`system.calculator` | Acción del sistema; depende en parte de sus atajos configurados. | — | ✓ | ✓ |
| Apagar pantalla<br>`system.display_off` | Acción del sistema; depende en parte de sus atajos configurados. | — | ✓ | ✓ |
| Suspender<br>`system.sleep` | Acción del sistema; depende en parte de sus atajos configurados. | — | ✓ | ✓ |

### Multimedia

| Acción / ID | Función y ámbito de uso | Parámetros | Windows | Mac |
|---|---|---|---|---|
| Reproducir / pausar<br>`media.play_pause` | Teclas multimedia del sistema; requieren un reproductor compatible. | — | ✓ | ✓ |
| Pista anterior<br>`media.previous` | Teclas multimedia del sistema; requieren un reproductor compatible. | — | ✓ | ✓ |
| Pista siguiente<br>`media.next` | Teclas multimedia del sistema; requieren un reproductor compatible. | — | ✓ | ✓ |
| Subir volumen<br>`media.volume_up` | Teclas multimedia del sistema; requieren un reproductor compatible. | — | ✓ | ✓ |
| Bajar volumen<br>`media.volume_down` | Teclas multimedia del sistema; requieren un reproductor compatible. | — | ✓ | ✓ |
| Silenciar<br>`media.mute` | Teclas multimedia del sistema; requieren un reproductor compatible. | — | ✓ | ✓ |
| Más brillo<br>`display.brightness_up` | Brillo del sistema; depende de la pantalla y sus controladores. | — | ✓ | ✓ |
| Menos brillo<br>`display.brightness_down` | Brillo del sistema; depende de la pantalla y sus controladores. | — | ✓ | ✓ |

### Abrir

| Acción / ID | Función y ámbito de uso | Parámetros | Windows | Mac |
|---|---|---|---|---|
| Abrir sitio web<br>`open.website` | Abre la URL indicada en el navegador predeterminado. | URL (`url`): predeterminado: `https://mousetroke.com/` | ✓ | ✓ |
| Abrir carpeta<br>`open.folder` | Abre la ruta indicada con la app predeterminada del sistema. | Ruta (`path`): obligatorio | ✓ | ✓ |
| Abrir archivo<br>`open.file` | Abre la ruta indicada con la app predeterminada del sistema. | Ruta (`path`): obligatorio | ✓ | ✓ |
| Abrir o cambiar a aplicación<br>`open.app` | Abre la app o activa su ventana existente. | Aplicación (`app`): obligatorio | ✓ | ✓ |

### Mousetroke

| Acción / ID | Función y ámbito de uso | Parámetros | Windows | Mac |
|---|---|---|---|---|
| Pausar / reanudar gestos<br>`mousetroke.toggle_pause` | Pausa o reanuda el reconocimiento de gestos en Mousetroke. | — | ✓ | ✓ |
| Abrir ajustes<br>`mousetroke.settings` | Abre los ajustes de Mousetroke. | — | ✓ | ✓ |
| Mostrar lista de gestos<br>`mousetroke.cheatsheet` | Lista los gestos de la app actual; previsto para la fase 2. | — | Esta acción está prevista para la fase 2. | Esta acción está prevista para la fase 2. |

<!-- action-library:generated:end -->

Después de cambiar el registro o las traducciones, establece `UPDATE_ACTION_DOCS=1` y ejecuta `cargo test -p mousetroke-core --test action_docs` para actualizar los catálogos de los diez idiomas. Ejecuta el mismo comando sin esa variable para comprobar que las tablas guardadas en el repositorio están actualizadas.
