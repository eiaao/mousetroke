---
title: "Gestos de ratón"
description: "Graba trazados y formas con el ratón, combina botones, rueda y teclas, y ajusta el reconocimiento de gestos."
order: 3
---

# Gestos de ratón

## Grabar un trazo direccional

Abre **Todas las aplicaciones → Gestos de ratón → Nuevo gesto**. Ponle nombre y elige **Cómo empieza**: **Mantener pulsado el botón derecho**, **Mantener pulsado el botón central**, **Mantener pulsado el botón lateral 1** o **Mantener pulsado el botón lateral 2**. Los botones laterales corresponden a X1 y X2.

En el área de dibujo, mantén pulsado el botón izquierdo o derecho, dibuja y suelta. El lienzo de grabación acepta cualquiera de los dos botones con independencia del disparador que hayas elegido. Los segmentos rectos se convierten en un trazo direccional, como `←` o `↓→`; vuelve a dibujar para sustituirlo. Elige una [acción](actions.md) y prueba después el gesto fuera de la configuración con su botón disparador real.

## Grabar una forma

Las curvas como C, O o S son gestos de forma. La grabadora detecta automáticamente si es un trazo o una forma. Si es necesario, haz clic en **Usar como forma** o **Usar como trazado** después de dibujar, o elige **+ Añadir un paso → Dibujar una forma**.

Dibuja la misma forma 2–3 veces, manteniendo el mismo punto de inicio y la misma dirección. Suelta después de cada muestra. Se necesita al menos una muestra válida para guardar; la grabadora conserva hasta cinco muestras y puede eliminar muestras individuales. Atiende a sus mensajes de coherencia y de conflicto. Los gestos de forma se ejecutan al soltar el botón disparador.

## Añadir botones, rueda o teclas

En **Después**, haz clic en **+ Añadir un paso**. Puedes añadir **Dibujar un trazado**, **Dibujar una forma**, un botón del ratón en **Hacer clic con un botón del ratón**, **Arriba** / **Abajo** en **Girar la rueda**, o **Mantener una tecla**. La última opción graba la tecla que pulses.

Por ejemplo, empieza con el botón derecho y añade un clic izquierdo, o añade rueda hacia arriba para ejecutar una acción de volumen repetidamente mientras mantienes el disparador. Los pasos están ordenados: ejecútalos en el orden mostrado conservando el botón disparador. Quita un paso con su botón ×.

En los gestos direccionales, **Ejecutar en cuanto se reconozca** los ejecuta sin esperar a que sueltes. Es idóneo para acciones repetidas de la rueda. Las formas esperan siempre a que sueltes. **No mostrar el nombre del gesto al dibujar** oculta el nombre de esa asignación.

## Reconocimiento y aspecto

En **Preferencias → Reconocimiento**:

| Ajuste | Efecto |
|---|---|
| Tiempo de espera antes de tratarlo como clic | Predeterminado: 500 ms; control deslizante de 150–1500 ms. Mantén el botón sin moverte durante este tiempo para que pase como clic/arrastre normal. |
| Sensibilidad del trazado | Predeterminado: 30 px; control deslizante de 10–80 px. Los valores más pequeños reconocen trazos más cortos, pero facilitan los gestos accidentales. |
| Los gestos actúan sobre | Elige **la ventana bajo el cursor** (predeterminado) o **la ventana activa**. |

En **Aspecto**, **Mostrar el rastro**, **Grosor del rastro**, **Color del rastro**, **Al reconocerse**, **Desvanecer al terminar**, **Mostrar el nombre del gesto al dibujar** y **Color del nombre del gesto** controlan la respuesta visual. Aquí no hay ningún control deslizante de umbral de reconocimiento de formas.

## Buscar, copiar y desactivar

Pulsa **Ctrl+F** en Windows o **⌘+F** en Mac para buscar en la lista actual por nombre, trazado o acción. Usa **Gestos de ratón** / **Bordes de pantalla** y los filtros de botón/posición; **Desactivado** muestra las entradas desactivadas.

Selecciona un gesto y haz clic en **Duplicar** para hacer una copia (Ctrl+D / ⌘+D). Dale a la copia un trazado distinto: los dibujos idénticos en un mismo ámbito entran en conflicto, así que no te fíes del orden de la lista para elegir una acción. Usa el conmutador **Activado** / **Desactivado** del editor, o Espacio sobre una fila seleccionada de la lista, para desactivarlo sin borrarlo. Las asignaciones desactivadas dejan por sí solas de reservar su disparador.

Siguiente: [bordes de pantalla](screen-edges.md), [reglas por aplicación](per-app.md), [sintaxis de envío de teclas](key-sequences.md).
