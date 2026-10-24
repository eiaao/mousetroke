---
title: "Reglas por aplicación"
description: "Añade aplicaciones, sobrescribe los gestos globales, excluye las que necesitan arrastrar con el ratón y entiende la pausa automática a pantalla completa."
order: 6
---

# Reglas por aplicación

## Añadir una aplicación

Haz clic en **Agregar aplicación…** en la barra lateral. Elige **Definir gestos específicos** o **Desactivar gestos en ella** y, a continuación:

- Selecciona una entrada en **En ejecución**.
- Haz clic en **Elegir una ventana** y luego haz clic en la ventana de la aplicación; Esc o el clic derecho cancelan.
- Usa **Examinar…** para elegir la aplicación.
- Suelta una aplicación o su acceso directo en la ventana de configuración. Mac admite bundles `.app` y archivos ejecutables; sus ámbitos usan ID de bundle cuando están disponibles.

Las reglas de Windows pueden coincidir con un nombre de ejecutable o una ruta completa, sin distinguir mayúsculas de minúsculas. Las identidades de aplicaciones de Mac suelen tener el aspecto de `com.apple.Safari`. Usa el selector en lugar de adivinar la identidad. El ámbito de la aplicación aparece en la barra lateral y puede tener sus propios gestos de ratón y bordes de pantalla.

## Gestos específicos y globales

**Todas las aplicaciones** contiene los gestos globales. Un gesto específico de una aplicación tiene prioridad sobre el gesto global correspondiente. Con **Usar también los gestos globales** marcado, las asignaciones globales siguen disponibles cuando las de la aplicación no coinciden. Desmárcalo para usar solo los gestos de esa aplicación.

Por ejemplo, añade un navegador, crea un gesto de botón derecho `↓→` y asigna allí una acción distinta. Las demás aplicaciones conservan la acción global de `↓→`. Para copiar una configuración existente, usa [Duplicar](gestures.md#buscar-copiar-y-desactivar) dentro de su ámbito y edita la asignación.

## Desactivar los gestos en una aplicación

Selecciona la aplicación en la barra lateral y activa **Desactivar gestos en esta aplicación**. Esto desactiva sus propios gestos, los gestos globales y los disparadores de borde. Los botones derecho, central y laterales pasan con normalidad. Las asignaciones guardadas se conservan; **Reactivar gestos** las restaura. **Quitar esta aplicación** elimina el ámbito específico de la aplicación.

Esto es útil para el escritorio remoto, las máquinas virtuales, el modelado 3D, las herramientas de diseño y los editores de juegos que usan el arrastre con el botón derecho o central. La lista **Se sugiere desactivarlos** del selector incluye aplicaciones conocidas de estas categorías. Las aplicaciones instaladas detectadas se pueden añadir en grupo.

## Pantalla completa y primera ejecución

La pausa automática a pantalla completa es automática: el motor evita iniciar gestos y disparadores de borde mientras la aplicación en primer plano se detecta a pantalla completa. Cubre los juegos, el vídeo y las presentaciones a pantalla completa; es independiente del estado de pausa manual. En Windows, la detección comprueba una ventana sin bordes que cubra su monitor y excluye las ventanas del escritorio o del shell. Mac compara la ventana de la aplicación en primer plano con los límites de la pantalla. Por eso, una aplicación con aspecto de pantalla completa puede comportarse de forma distinta en las dos plataformas.

Para un juego en ventana o un escritorio remoto, añade una exclusión explícita. Para pausar en todas partes, usa el [comando de la bandeja o de la barra de menús, o el atajo de pausa](getting-started.md#pausa).

La guía de primera ejecución muestra las aplicaciones detectadas que suelen necesitar su propio arrastre con el ratón. Las entradas marcadas quedan desactivadas al terminar. Revísalas antes de continuar. La configuración integrada de Mac también incluye ámbitos desactivados para sus aplicaciones de exclusión sugeridas; estos no tienen asignaciones específicas. Puedes cambiar las exclusiones más tarde desde la barra lateral.

Siguiente: [acciones](actions.md), [copia de seguridad de la configuración](config.md), [solución de problemas](troubleshooting.md).
