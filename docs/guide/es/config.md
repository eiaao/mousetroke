---
title: "Configuración, copia de seguridad y actualizaciones"
description: "Encuentra la configuración y los registros de Mousetroke, exporta y restaura los ajustes, recarga los cambios, actualiza la aplicación o desinstálala."
order: 11
---

# Configuración, copia de seguridad y actualizaciones

## Ubicaciones de archivos

| Archivo | Windows | Mac |
|---|---|---|
| Configuración | `%APPDATA%\mousetroke\config.json` | `~/Library/Application Support/mousetroke/config.json` |
| Registro del tiempo de ejecución | `%APPDATA%\mousetroke\mousetroke.log` | `~/Library/Logs/mousetroke/mousetroke.log` |

Los registros rotan a `mousetroke.log.old` tras superar 1 MB. **Acerca de → Ubicaciones de archivos** muestra las rutas reales. **Abrir carpeta** abre el directorio de configuración; **Abrir registro** abre el registro del tiempo de ejecución. En **Importar y exportar**, **Mostrar en el Explorador de archivos** en Windows o **Mostrar en Finder** en Mac revela la configuración.

## Exportar, importar y restaurar

En **Importar y exportar → Archivo de configuración de Mousetroke**, haz clic en **Exportar…** y guarda una copia de seguridad JSON. **Importar desde un archivo…** lee un JSON de configuración de Mousetroke y sustituye la configuración actual, incluidas las preferencias. No combina ese archivo con tus asignaciones actuales. No hay ninguna opción de combinación de configuración nativa en este flujo. Puedes deshacer la importación con Ctrl+Z en Windows o ⌘+Z en Mac mientras la configuración esté abierta.

Antes de editar el JSON a mano, exporta una copia de seguridad. Para una restauración directa, cierra el motor y la ventana de configuración, sustituye `config.json` por la copia de seguridad y vuelve a abrir Mousetroke. Mantener la configuración cerrada evita que su autoguardado sobrescriba el archivo restaurado.

## Recargar y restablecer

El motor vigila la marca de tiempo del archivo de configuración. Windows comprueba aproximadamente una vez por segundo; Mac comprueba en la rama de mantenimiento programada cada dos segundos. Un cambio recibido durante un gesto se aplica cuando termina la captura. Un JSON no válido deja en uso la configuración activa anterior y registra un error en el registro.

La interfaz de configuración actual no tiene ningún botón de restablecimiento a los valores predeterminados. Para empezar con los valores integrados, sal de Mousetroke y cierra la configuración, haz una copia de seguridad y elimina o renombra `config.json`, y vuelve a abrir. Un archivo ausente se inicializa con los valores predeterminados de la plataforma. Un archivo mal formado no es una petición de restablecimiento.

## Actualizaciones

**Preferencias → Buscar actualizaciones automáticamente** está activado de forma predeterminada y consulta GitHub una vez al día. Desactívalo ahí para detener las comprobaciones automáticas. Usa **Acerca de → Buscar actualizaciones** para una comprobación manual y **Novedades** para las notas de la versión. Un elemento de menú de nueva versión también abre Acerca de.

En Windows, **Actualizar ahora** descarga el ejecutable, verifica su SHA-256, sustituye la copia en ejecución y reinicia. **Abrir página de descarga** está disponible para la instalación manual.

En Mac, **Actualizar ahora** descarga el DMG, comprueba el SHA-256, la identidad del bundle, la versión y el requisito de firma, sustituye la aplicación y reinicia. El bundle antiguo va a la Papelera tras el apagado; si falla la sustitución o el relevo al reiniciar, se restaura. **Descargar manualmente** está disponible si la carpeta de instalación no permite escritura o si la actualización falla. Las actualizaciones desde la aplicación evitan otro aviso de cuarentena del navegador y conservan la identidad de instalación usada para los permisos. Una descarga nueva desde el navegador puede necesitar **Abrir igualmente** de nuevo.

## Desinstalar

Windows: desactiva **Activar los gestos automáticamente al arrancar** en Preferencias, elige **Salir** en la bandeja, cierra la configuración y luego elimina el ejecutable y `%APPDATA%\mousetroke` si ya no necesitas los ajustes ni los registros.

Mac: elige **Salir** en la barra de menús, cierra la configuración y elimina `Mousetroke.app`. Para eliminar también los datos, borra `~/Library/Application Support/mousetroke` y `~/Library/Logs/mousetroke`. Quita Mousetroke de los ítems de inicio de sesión en Ajustes del Sistema. Conserva una copia exportada por si vuelves a instalarlo.

Consulta la [solución de problemas](troubleshooting.md) para los errores de configuración y los registros ausentes.
