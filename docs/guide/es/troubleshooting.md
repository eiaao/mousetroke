---
title: "Solución de problemas"
description: "Arregla los gestos de ratón que no se disparan, los permisos de macOS, los atajos que no llegan, la entrada de texto con IME y los errores de Lua."
order: 12
---

# Solución de problemas

## Los gestos no se disparan

Prueba primero un gesto integrado sencillo en una ventana normal del navegador, no a pantalla completa.

- Comprueba el menú de la bandeja o de la barra de menús. En Windows selecciona **Reanudar gestos** si aparece; en Mac haz clic de nuevo en **Pausar gestos** si está marcado. Comprueba **Preferencias → Atajo para pausar / reanudar** si un atajo pausó el motor.
- Sal de la pantalla completa. La supresión automática a pantalla completa puede detener tanto los gestos de ratón como las acciones de borde aunque la pausa manual esté desactivada.
- Selecciona la aplicación en la configuración y comprueba **Desactivar gestos en esta aplicación**. Revisa también el conmutador **Activado** / **Desactivado** de cada asignación, **Usar también los gestos globales**, el botón disparador y cualquier paso adicional.
- Muévete con presteza tras pulsar el botón y dibuja un trazo más largo. Ajusta el **Tiempo de espera antes de tratarlo como clic** y la **Sensibilidad del trazado** en Preferencias.
- Si falta la respuesta visual, comprueba **Mostrar el rastro** y **Mostrar el nombre del gesto al dibujar**. Que el rastro esté oculto no significa por sí solo que el reconocimiento esté desactivado.

En Windows, la entrada usa `SendInput`. Una instancia iniciada con normalidad puede no poder enviar teclas o clics a una aplicación elevada por la UIPI de Windows. Prueba primero contra una aplicación no elevada; si necesitas controlar una ventana de administrador, ejecuta Mousetroke con la elevación correspondiente. Esto no habilita el envío al escritorio seguro.

En Mac, comprueba **Accesibilidad** para la copia instalada. Si la configuración sigue pidiendo **Monitorización de entrada**, concédesela también. Los cambios de permiso pueden tardar unos segundos en detectarse. Usa **Reiniciar Mousetroke** cuando se ofrezca si la entrada sigue sin iniciarse. No se requiere Grabación de pantalla. Mover o sustituir una copia a través del navegador puede exigir de nuevo la aprobación de primera apertura.

## SmartScreen o Gatekeeper bloquean la primera apertura

Windows: consigue el ejecutable oficial desde la página de versiones, verifica la suma de comprobación y luego usa **Más información → Ejecutar de todos modos**. Mac: instala en Aplicaciones y luego usa **Ajustes del Sistema → Privacidad y seguridad → Abrir igualmente**; las versiones antiguas de macOS también admiten **clic derecho → Abrir**. Consulta la [instalación](getting-started.md) para la secuencia completa.

## Una aplicación no recibe las teclas

Prueba el atajo en el teclado físico. Comprueba si la aplicación está activa, o activa **Activar antes la ventana bajo el cursor**. Confirma la preferencia de objetivo del gesto y usa la [sintaxis correcta](key-sequences.md): `^` es Control, mientras que `#` es Command en Mac. Separa los pasos dependientes con un breve `{SLEEP 100}` si la aplicación necesita tiempo para abrir un menú o un cuadro de diálogo.

Las secuencias de teclas de Mac omiten los tokens de ratón/rueda, los tokens de teclas multimedia/volumen, Insert, Imprimir pantalla, Break y F21–F24. Usa **Medios y sistema** para las acciones multimedia y de volumen. Las opciones no admitidas se marcan en el editor; inspecciona también el registro.

Ctrl+Alt+Del es una secuencia de atención segura de Windows y no puede enviarse como entrada sintética ordinaria. Usa Ctrl+Shift+Esc para el Administrador de tareas, Win+L para bloquear, o `shutdown /l` para cerrar la sesión. Los detalles están en la [sintaxis de envío de teclas](key-sequences.md#por-qué-no-se-puede-enviar-ctrlaltdel).

## El IME o la salida de texto fallan

Los caracteres ordinarios de las secuencias de teclas representan posiciones del teclado estadounidense, no texto Unicode arbitrario. Las mayúsculas por sí solas no añaden Shift. Lua `ms.keys(text, true)` emite texto Unicode pero sigue interpretando los caracteres de sintaxis como `^` y `{`.

Para texto con mucha puntuación, texto no latino o aplicaciones con un manejo inusual del IME, usa la [receta de pegar y restaurar el portapapeles](lua-examples.md#pegar-y-restaurar-el-texto-del-portapapeles). La restauración con estas API conserva solo texto, no imágenes ni otros formatos del portapapeles. Algunas aplicaciones necesitan un retardo mayor antes de la restauración del portapapeles.

## Errores de Lua

Asegúrate de que el código define `function ms_action(...) ... end`. El tiempo de ejecución escribe los errores de carga y de ejecución en el [registro del tiempo de ejecución](config.md#ubicaciones-de-archivos). Ábrelo con **Acerca de → Abrir registro**. Usa `ms.log` para valores y `ms.show_toast` como punto de control visual breve. Una función no admitida en Mac puede solo registrar un aviso y devolver `nil`; no tomes una ausencia de valor como un éxito.

Las llamadas de Mac de descubrimiento y estado de ventanas requieren el permiso de Accesibilidad; las consultas de estado de teclas devuelven false sin él. Comprueba primero ese permiso si faltan resultados. El siempre encima entre aplicaciones y el brillo numérico siguen sin estar admitidos; las acciones integradas de teclas de brillo no proporcionan un porcentaje. En ambas plataformas, `ms.get_brightness` / `ms.set_brightness` devuelven nil en caso de error.

El límite de asignación de Lua es 8 MiB. No hay tiempo límite de ejecución de scripts; evita los bucles infinitos y consulta los [Fundamentos de Lua](lua-basics.md#tiempo-de-ejecución-y-límites) para el comportamiento de bloqueo.

## Escritorio remoto y juegos

El escritorio remoto, las máquinas virtuales y las aplicaciones 3D o de juegos pueden necesitar los mismos botones del ratón para arrastrar o pueden gestionar la entrada sintética de forma distinta. Añade una [exclusión por aplicación](per-app.md) o pausa mientras los usas. Los juegos en ventana pueden necesitar una exclusión explícita porque la detección de pantalla completa no se aplica. Prueba los atajos en una aplicación local ordinaria para distinguir un problema de secuencia de teclas del manejo de entrada de la aplicación.

## Informar de un error

Usa **Acerca de → Comentarios y sugerencias → Abrir** o [GitHub Issues](https://github.com/eiaao/mousetroke/issues). Incluye la versión de la aplicación, la versión del sistema operativo, la aplicación afectada, el gesto y la acción, el resultado esperado y los pasos para reproducirlo. Adjunta el registro pertinente desde **Acerca de → Abrir registro** y una configuración exportada mínima si resulta útil. Revisa los adjuntos en busca de texto privado, títulos de ventanas, rutas o argumentos de comandos antes de compartirlos.
