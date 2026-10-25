---
title: "Konfiguration, Backup und Updates"
description: "Mousetroke-Konfiguration und Protokolle finden, Einstellungen exportieren und wiederherstellen, Änderungen neu laden, die App aktualisieren oder deinstallieren."
order: 11
---

# Konfiguration, Backup und Updates

## Dateispeicherorte

| Datei | Windows | Mac |
|---|---|---|
| Konfiguration | `%APPDATA%\mousetroke\config.json` | `~/Library/Application Support/mousetroke/config.json` |
| Laufzeitprotokoll | `%APPDATA%\mousetroke\mousetroke.log` | `~/Library/Logs/mousetroke/mousetroke.log` |

Protokolle wechseln nach dem Überschreiten von 1 MB zu `mousetroke.log.old`. **Über → Dateispeicherorte** zeigt die tatsächlichen Pfade an. **Ordner öffnen** öffnet den Konfigurationsordner; **Protokoll öffnen** das Laufzeitprotokoll. Unter **Import und Export** zeigt **Im Datei-Explorer anzeigen** unter Windows oder **Im Finder anzeigen** auf dem Mac die Konfiguration an.

## Exportieren, Importieren und Wiederherstellen

Klicken Sie unter **Import und Export → Mousetroke-Konfigurationsdatei** auf **Exportieren…** und speichern Sie ein JSON-Backup. **Aus Datei importieren…** liest eine Mousetroke-Konfigurations-JSON und ersetzt die aktuelle Konfiguration, einschließlich der Einstellungen. Sie führt diese Datei nicht mit Ihren aktuellen Bindungen zusammen. Eine Zusammenführungsoption für die native Konfiguration gibt es in diesem Ablauf nicht. Sie können den Import mit Strg+Z unter Windows oder ⌘+Z auf dem Mac rückgängig machen, solange die Einstellungen geöffnet sind.

Sichern Sie vor der manuellen JSON-Bearbeitung ein Backup. Für eine direkte Wiederherstellung beenden Sie die Engine und schließen die Einstellungen, ersetzen `config.json` durch das Backup und öffnen Mousetroke anschließend erneut. Geschlossene Einstellungen verhindern, dass ihr automatisches Speichern die wiederhergestellte Datei überschreibt.

## Neuladen und Zurücksetzen

Die Engine überwacht den Datei-Zeitstempel der Konfiguration. Windows prüft etwa einmal pro Sekunde; der Mac prüft im Wartungszweig, der alle zwei Sekunden ausgeführt wird. Eine während einer Geste eingegangene Änderung wird nach dem Ende der Aufnahme angewendet. Ungültiges JSON lässt die zuvor aktive Konfiguration in Gebrauch und vermerkt einen Fehler im Protokoll.

Die aktuelle Einstellungen-Oberfläche hat keine Schaltfläche zum Zurücksetzen auf die Standardwerte. Um mit den integrierten Standardwerten zu beginnen, beenden Sie Mousetroke, schließen die Einstellungen, sichern `config.json` und entfernen oder benennen es um und öffnen Mousetroke erneut. Eine fehlende Datei wird mit plattformspezifischen Standardwerten initialisiert. Eine fehlerhafte Datei ist keine Anweisung zum Zurücksetzen.

## Updates

**Einstellungen → Automatisch nach Updates suchen** ist standardmäßig aktiviert und fragt GitHub einmal täglich ab. Schalten Sie es dort aus, um die automatische Prüfung zu stoppen. Verwenden Sie **Über → Nach Updates suchen** für eine manuelle Prüfung und **Was ist neu?** für Versionshinweise. Ein Menüeintrag für eine neue Version öffnet ebenfalls das Über-Fenster.

Unter Windows lädt **Jetzt aktualisieren** die ausführbare Datei herunter, prüft ihren SHA-256, ersetzt die laufende Kopie und startet neu. **Downloadseite öffnen** steht für die manuelle Installation zur Verfügung.

Auf dem Mac lädt **Jetzt aktualisieren** das DMG herunter, prüft SHA-256, Bundle-Identität, Version und die Signieranforderung, ersetzt die App und startet neu. Das alte Bundle wandert nach dem Beenden in den Papierkorb; schlägt das Ersetzen oder die Neustart-Übergabe fehl, wird es wiederhergestellt. **Manuell herunterladen** ist verfügbar, wenn der Installationsordner nicht beschreibbar ist oder das Update fehlschlägt. In-App-Updates vermeiden eine erneute Browser-Quarantäne-Aufforderung und bewahren die Installationsidentität, die für Berechtigungen verwendet wird. Ein frischer Download über den Browser braucht eventuell erneut **Trotzdem öffnen**.

## Deinstallation

Windows: Deaktivieren Sie **Gesten beim Windows-Start automatisch aktivieren** in den Einstellungen, wählen Sie im Infobereich **Beenden**, schließen Sie die Einstellungen und löschen Sie anschließend die ausführbare Datei sowie `%APPDATA%\mousetroke`, wenn Sie Einstellungen oder Protokolle nicht mehr brauchen.

Mac: Wählen Sie in der Menüleiste **Beenden**, schließen Sie die Einstellungen und löschen Sie `Mousetroke.app`. Um auch die Daten zu entfernen, löschen Sie `~/Library/Application Support/mousetroke` und `~/Library/Logs/mousetroke`. Entfernen Sie Mousetroke aus den Anmeldeobjekten in den Systemeinstellungen. Bewahren Sie ein exportiertes Backup auf, falls Sie später neu installieren möchten.

Siehe [Fehlerbehebung](troubleshooting.md) bei Konfigurationsfehlern und fehlenden Protokollen.
