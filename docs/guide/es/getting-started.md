---
title: "Primeros pasos"
description: "Instala Mousetroke en Windows o Mac, dibuja tu primer gesto de ratón, abre la configuración y pausa los gestos."
order: 2
---

# Primeros pasos

## Instalar y abrir

En Windows, descarga `mousetroke.exe` desde la [última versión](https://github.com/eiaao/mousetroke/releases/latest). Es un ejecutable portátil: guárdalo donde quieras conservarlo y haz doble clic en él. Compara el SHA-256 de la descarga con `SHA256SUMS.txt` en esa página de versiones.

Se admiten Windows 10 1809 o posterior y Windows 11, de 64 bits. La configuración usa Microsoft Edge WebView2, incluido en Windows 11. El ejecutable aún no está firmado con código. Si SmartScreen bloquea el primer inicio, elige **Más información → Ejecutar de todos modos** después de comprobar que descargaste la versión oficial.

En Mac, descarga `Mousetroke.dmg`, ábrelo y arrastra **Mousetroke** a **Aplicaciones**. La aplicación universal es compatible con Apple silicon e Intel en macOS 11 o posterior. Si se abre desde dentro del DMG, la aplicación ofrece **Mover a Aplicaciones** y se vuelve a abrir desde la copia instalada.

La beta de Mac está autofirmada y no está notarizada. Si se bloquea, usa **Ajustes del Sistema → Privacidad y seguridad → Abrir igualmente**; en versiones antiguas de macOS, usa **clic derecho → Abrir**. Durante la configuración, haz clic en **Conceder…** y activa Mousetroke en **Accesibilidad**. Espera el mensaje **Todo listo: ya puedes usar los gestos**. No se requiere el permiso de Grabación de pantalla. La configuración ofrece **Monitorización de entrada** si la escucha sigue fallando en sistemas que la necesitan; actívala cuando se te pida.

## Abrir la configuración y terminar la guía

En Windows, haz clic izquierdo en el icono del área de notificación para abrir la configuración. El clic derecho ofrece **Configuración…**, **Pausar gestos** / **Reanudar gestos**, **Iniciar automáticamente**, **Abrir carpeta de configuración** y **Salir**. También puede aparecer un elemento de nueva versión cuando hay una actualización disponible.

En Mac, usa el icono de la barra de menús para abrir **Configuración…**, pausar o reanudar, o **Salir**. Los elementos de permisos y de actualización aparecen cuando procede. **Acerca de Mousetroke** abre la ventana Acerca de.

La guía de primera ejecución explica el dibujo, los bordes de pantalla, las exclusiones de aplicaciones y la pausa. Revisa las aplicaciones marcadas antes de terminar: en esas aplicaciones los gestos quedarán desactivados. Puedes volver a abrirla con **Acerca de → Ver la guía de primeros pasos**. Los cambios en la configuración se guardan automáticamente; un trazo incompleto debe grabarse antes de poder guardarse.

## Dibujar tu primer gesto

1. Abre una página del navegador con historial de navegación.
2. Mantén pulsado el botón derecho, muévete hacia la izquierda para dibujar `←` y suelta. Esto va hacia atrás.
3. Prueba `↓→` en una pestaña desechable para cerrarla, o `↙` para minimizar la ventana.

Las acciones integradas se adaptan a la plataforma: los atajos del navegador usan Ctrl en Windows y Command en Mac donde corresponde. La lista completa editable está en **Todas las aplicaciones → Gestos de ratón**. Algunas acciones integradas están desactivadas, sobre todo las no disponibles en Mac.

Un clic derecho normal sigue abriendo el menú contextual de la aplicación. Mantener el botón quieto el tiempo suficiente también deja pasar el botón para un clic o arrastre normales; ajusta ese tiempo en [Preferencias](gestures.md#reconocimiento-y-aspecto).

## Crear un gesto

Abre **Todas las aplicaciones → Gestos de ratón → Nuevo gesto**, elige el disparador y graba el dibujo. Haz clic en **Elegir acción** para buscar o explorar la [biblioteca de acciones](action-library.md), selecciona una acción predefinida y rellena sus parámetros. Los cambios se guardan automáticamente cuando el gesto es válido; pruébalo fuera de la configuración. Consulta [Gestos de ratón](gestures.md) para conocer los detalles de grabación.

## Pausa

En Windows elige **Pausar gestos** y luego **Reanudar gestos** para restaurarlos. En Mac, **Pausar gestos** conserva la misma etiqueta: su marca de verificación indica una pausa manual y, al hacer clic de nuevo, se reanudan. El atajo predeterminado de pausa/reanudación es **Win+Esc** en Windows y **Control+Option+Command+P** en Mac. Cámbialo en **Preferencias → Atajo para pausar / reanudar**.

Los juegos, vídeos y presentaciones a pantalla completa suprimen los gestos automáticamente mientras la ventana en primer plano se detecta a pantalla completa. Consulta las [reglas por aplicación](per-app.md) para las exclusiones y la [solución de problemas](troubleshooting.md) si nada se dispara.
