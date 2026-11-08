---
title: "Biblioteca de gestos"
description: "Define un gesto una vez y úsalo en varias aplicaciones con acciones distintas: organiza e importa tus gestos, asígnalos a otras aplicaciones, edita gestos compartidos y notas de compatibilidad."
order: 3.5
---

# Biblioteca de gestos

La biblioteca de gestos separa cómo dibujas de lo que ocurre. Define un gesto una sola vez y dale una acción distinta en cada aplicación. Al volver a grabar un gesto compartido se actualizan todos los lugares que lo usan, y un gesto de forma conserva un único conjunto de muestras.

## Explorar la biblioteca

Haz clic en **Gestos** en la barra lateral. La lista se divide en **Gestos de ratón** y **Bordes de pantalla**. Filtra por botón de activación, usa **Sin usar** para encontrar gestos que nada usa o busca por nombre, trazo o uso. Cada tarjeta indica en cuántos lugares se usa.

Haz clic en una tarjeta para ver sus detalles. Arriba puedes volver a dibujarlo; **Se usa en** muestra cada aplicación, nombre de gesto y acción que lo usa, y al hacer clic en una fila saltas allí. En una instalación nueva los gestos integrados ya están en la biblioteca. Si actualizas desde una versión anterior, tus gestos siguen repartidos entre aplicaciones: organízalos primero como se explica abajo.

## Organizar e importar

Con la biblioteca vacía, haz clic en **Organizar e importar gestos…**; después siempre tienes **Organizar e importar…** arriba a la derecha. Primero elige el alcance:

- **Solo gestos comunes (recomendado)**: añade los gestos de **Todas las aplicaciones** y vincula los idénticos o parecidos de otras aplicaciones. Los gestos configurados solo para una aplicación siguen siendo específicos.
- **Todos los gestos**: añade los gestos compartidos y todos los específicos de cada aplicación.

Haz clic en **Siguiente** para revisar el resultado. No cambia nada hasta que confirmes:

- Los trazos idénticos se combinan automáticamente. Los trazos con las mismas direcciones que solo difieren en la longitud de los tramos se combinan por defecto.
- **Formas parecidas: confirmar**: en cada grupo elige el que quieres (**Usar este**) o **Combinar muestras**. Los grupos por debajo de la similitud recomendada no se combinan por defecto, y una forma combinada puede reconocerse de manera algo distinta.
- **Se añaden directamente**: despliégalo para desmarcar gestos. Los desmarcados no se importan y siguen siendo específicos.

Haz clic en **Importar** para terminar; el aviso permite deshacer. Dos gestos parecidos de la misma aplicación que hacen cosas distintas nunca se combinan.

## Usar un gesto en otra aplicación

En los detalles del gesto, haz clic en **Usar en otra aplicación…**, elige una aplicación y luego una acción. Las aplicaciones que ya usan el gesto no se pueden elegir; si una aplicación ya tiene el mismo trazo, aparece un aviso de conflicto; puedes asignarlo a una aplicación con los gestos desactivados, pero allí no funcionará. **Agregar aplicación…** al final de la lista agrega primero una aplicación. Después sigues en los detalles del gesto con la fila nueva resaltada, y **Configurar** en el aviso abre esa asignación.

También puedes empezar desde la aplicación. En el editor, un gesto que solo pertenece a la aplicación actual muestra **Solo en esta aplicación**:

- Haz clic en **Elegir de la biblioteca** para usar un gesto de la biblioteca. El selector admite búsqueda; al elegir uno de **Gestos en otras aplicaciones**, primero se añade a la biblioteca y se comparte con el original; **Grabar un gesto nuevo (solo en esta aplicación)** no entra en la biblioteca.
- El menú **Más acciones** incluye **Añadir a la biblioteca** y **Usar en otra aplicación…**.

## Editar un gesto compartido

En el editor, un gesto de la biblioteca muestra **Biblioteca** y dónde se usa. Al volver a dibujarlo, el cambio se aplica en todos los lugares que lo usan y se listan los nuevos conflictos:

- **Cambiar solo aquí**: el cambio queda solo en la aplicación actual; el gesto pasa a ser específico y la biblioteca y las demás aplicaciones conservan el trazo anterior.
- **Listo**: mantiene el cambio compartido y cierra el aviso.

**Cambiar** cambia a otro gesto de la biblioteca; el botón ↗ (**Ver en la biblioteca**) abre los detalles del gesto.

## Crear y eliminar

**Nuevo gesto** graba un gesto directamente en la biblioteca. Se añade al terminar de dibujar; el nombre es opcional. Asígnalo después con **Usar en otra aplicación…**.

Si eliges **Eliminar gesto** y el gesto todavía se usa, escoge:

- **Eliminar; cada uso conserva su propia copia**
- **Eliminarlo junto con todos sus usos**

Ambas opciones se pueden deshacer desde el aviso.

## Configuración y compatibilidad

La biblioteca se guarda en `gestures` dentro del archivo de configuración, y cada gesto que la usa guarda además una copia completa de su trazo. Por eso las versiones anteriores de Mousetroke pueden leer la configuración nueva y los gestos siguen funcionando. Sin embargo, si guardas la configuración en una versión anterior se pierde la biblioteca: cada gesto vuelve a ser específico y puedes organizarlos de nuevo tras actualizar. Si al editar el archivo a mano un trazo deja de coincidir con la biblioteca, ese lugar pasa a ser específico y se te avisa una vez. Ubicación de archivos y copias de seguridad: [Configuración, copia de seguridad y actualizaciones](config.md).

Siguiente: [Gestos de ratón](gestures.md), [Reglas por aplicación](per-app.md), [Biblioteca de acciones](action-library.md).
