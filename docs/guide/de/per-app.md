---
title: "App-spezifische Regeln"
description: "Apps hinzufügen, globale Mausgesten überschreiben, Apps ausschließen, die Maus-Ziehen brauchen, und die Vollbild-Autopause verstehen."
order: 6
---

# App-spezifische Regeln

## Eine Anwendung hinzufügen

Klicken Sie in der Seitenleiste auf **App hinzufügen…**. Wählen Sie **Eigene Gesten für die App** oder **Gesten darin deaktivieren** und dann:

- Wählen Sie einen Eintrag unter **Wird ausgeführt**.
- Klicken Sie auf **Fenster auswählen** und dann auf das Fenster der Anwendung; Esc oder Rechtsklick bricht ab.
- Verwenden Sie **Durchsuchen…**, um die Anwendung zu wählen.
- Ziehen Sie eine Anwendung oder ihre Verknüpfung in das Einstellungsfenster. Der Mac akzeptiert `.app`-Pakete und ausführbare Dateien; seine Bereiche verwenden, wenn verfügbar, Bundle-IDs.

Windows-Regeln können einen exe-Namen oder einen vollständigen Pfad unabhängig von der Groß-/Kleinschreibung treffen. Mac-App-Identitäten sehen üblicherweise wie `com.apple.Safari` aus. Verwenden Sie die Auswahl, statt die Identität zu raten. Der App-Bereich erscheint in der Seitenleiste und kann eigene Mausgesten und Bildschirmränder haben.

## App-spezifische und globale Gesten

**Alle Apps** enthält die globalen Gesten. Eine app-spezifische Geste hat Vorrang vor der entsprechenden globalen Geste. Ist **Auch globale Gesten verwenden** aktiviert, bleiben globale Bindungen verfügbar, wenn die Bindungen der App nicht passen. Deaktivieren Sie die Option, um nur die Gesten dieser App zu verwenden.

Fügen Sie zum Beispiel einen Browser hinzu, erstellen Sie eine `↓→`-Geste mit rechter Taste und weisen Sie ihr dort eine andere Aktion zu. Andere Apps behalten die globale `↓→`-Aktion. Um ein vorhandenes Setup zu kopieren, verwenden Sie [Duplizieren](gestures.md#suchen-duplizieren-und-deaktivieren) im jeweiligen Bereich und bearbeiten Sie die Bindung.

## Gesten in einer App deaktivieren

Wählen Sie die App in der Seitenleiste und aktivieren Sie **Gesten in dieser App deaktivieren**. Das schaltet eigene Gesten, globale Gesten und Rand-Auslöser aus. Rechte, mittlere und seitliche Tasten werden normal durchgereicht. Die gespeicherten Bindungen bleiben; **Gesten wieder einschalten** stellt sie wieder her. **Diese App entfernen** entfernt den app-spezifischen Bereich.

Das ist nützlich für Remotedesktop, virtuelle Maschinen, 3D-Modellierung, Design-Werkzeuge und Spiele-Editoren, die Rechts-/Mittelklick-Ziehen verwenden. Die Liste **Ausschalten empfohlen** der Auswahl enthält bekannte Anwendungen aus diesen Kategorien. Erkannte installierte Apps lassen sich gemeinsam hinzufügen.

## Vollbild und erster Start

Die Vollbild-Autopause ist automatisch: Die Engine vermeidet den Start von Gesten und Rand-Auslösern, solange die App im Vordergrund als Vollbild erkannt wird. Das gilt für Vollbildspiele, Video und Präsentationen; es ist vom manuellen Pausenzustand getrennt. Unter Windows prüft die Erkennung ein randloses Fenster, das seinen Monitor bedeckt, und schließt Desktop-/Shell-Fenster aus. Der Mac gleicht das Fenster der Vordergrund-App mit den Bildschirmgrenzen ab. Eine App, die wie Vollbild aussieht, kann sich auf den beiden Plattformen daher unterschiedlich verhalten.

Für ein Fenster-Spiel oder Remotedesktop fügen Sie eine explizite Ausnahme hinzu. Um überall zu pausieren, verwenden Sie den [Befehl in Taskleiste/Menüleiste oder das Pause-Kürzel](getting-started.md#pausieren).

Die Anleitung des ersten Starts zeigt erkannte Apps, die häufig eigenes Maus-Ziehen brauchen. Markierte Einträge werden beim Abschließen deaktiviert. Prüfen Sie sie, bevor Sie fortfahren. Die Mac-Grundkonfiguration enthält außerdem deaktivierte Bereiche für ihre empfohlenen Ausschluss-Apps; diese haben keine app-spezifischen Bindungen. Sie können Ausschlüsse später über die Seitenleiste ändern.

Weiter: [Aktionen](actions.md), [Konfigurations-Backup](config.md), [Fehlerbehebung](troubleshooting.md).
