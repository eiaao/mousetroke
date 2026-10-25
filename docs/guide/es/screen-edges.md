---
title: "Esquinas y bordes de la pantalla"
description: "Configura esquinas activas, desplazamiento por el borde para el volumen y el brillo, clics en los bordes y gestos de frotado en Windows y Mac."
order: 4
---

# Esquinas y bordes de la pantalla

## Probar los valores predeterminados

En **Todas las aplicaciones → Bordes de pantalla**, todas las asignaciones de esquina o borde son editables.

| Disparador | Valor predeterminado en Windows | Valor predeterminado en Mac |
|---|---|---|
| Empujar hacia la esquina inferior izquierda | Menú Inicio | Spotlight |
| Empujar hacia la esquina inferior derecha | Mostrar el escritorio | Mostrar el escritorio (F11) |
| Empujar hacia la esquina superior izquierda | Cambiar de aplicación (Alt+Tab) | Mission Control (Control+Up) |
| Empujar hacia la esquina superior derecha | Vista de tareas (Win+Tab) | Asignación desactivada al principio |
| Rueda en el borde superior | Subir/bajar volumen | Subir/bajar volumen |
| Clic central en el borde superior | Silenciar | Silenciar |
| Clic derecho en el borde superior | Reproducir / pausar | Reproducir / pausar |
| Rueda en el borde inferior | Subir/bajar brillo | Las asignaciones de brillo están desactivadas al principio |
| Frotar el borde derecho arriba/abajo dos veces | Asignación del Administrador de tareas; activa antes Ejecutar en cuanto se reconozca | Activity Monitor |

Las acciones de esquina no necesitan botón: empuja el puntero contra la esquina. El desplazamiento y los clics de borde necesitan que el puntero toque el borde elegido. Frotar significa moverse a lo largo del borde en las direcciones grabadas, en lugar de girar la rueda.

## Configurar un disparador de borde

1. Elige **Bordes de pantalla → Nuevo borde** en el ámbito de la aplicación deseada o en **Todas las aplicaciones**.
2. En **Cómo empieza**, elige **Borde de pantalla** y selecciona un lado o una esquina en el diagrama.
3. Deja **Después** vacío para una acción de esquina, o usa **+ Añadir un paso** para añadir rueda arriba/abajo, un clic de ratón o un trazo a lo largo del borde.
4. Elige una [acción](actions.md). Para el volumen, elige **Medios y sistema → Subir volumen** o **Bajar volumen**. Activa **Ejecutar en cuanto se reconozca** para el desplazamiento repetido o el frotado sin botones.

Para frotar, añade **Dibujar un trazado** y graba las direcciones alternas. Un ejemplo útil es `↑↓↑↓` contra el borde derecho. Prueba primero con una acción inofensiva, como un aviso de Lua, antes de asignar algo disruptivo.

## Brillo y diferencias entre plataformas

**Subir brillo** / **Bajar brillo** en Windows empieza con la pantalla bajo el cursor, ajusta en 10 puntos porcentuales y muestra el resultado. Prueba primero con DDC/CI y luego con WMI. Las pantallas externas deben admitir DDC/CI y permitirlo. La alternativa WMI usa el primer dispositivo de brillo de WMI, normalmente un panel integrado; no se corresponde con la pantalla del cursor. Si ambas vías fallan, aparece un mensaje en pantalla. Consulta las [recetas de Lua](lua-examples.md#brillo-de-la-pantalla-bajo-el-cursor) para un paso más pequeño.

**Medios y sistema → Subir brillo** / **Bajar brillo** en Mac envía eventos de las teclas de brillo del sistema para la pantalla integrada. No selecciona la pantalla bajo el cursor y no admite pantallas externas DDC/CI. Las asignaciones del borde inferior están desactivadas de forma predeterminada. Las API de Lua de brillo tampoco se admiten en Mac.

En Mac, Mousetroke avisa si una esquina activa del sistema se solapa con una esquina configurada. Cambia una de las dos asignaciones para evitar acciones enfrentadas. Los atajos del sistema como F11 o Control+Up dependen de la configuración de atajos de macOS.

Las exclusiones de aplicaciones y la detección de pantalla completa también suprimen los disparadores de borde. Consulta las [reglas por aplicación](per-app.md) cuando un borde funcione en una aplicación y en otra no.
