---
title: "Erste Schritte"
description: "Mousetroke unter Windows oder Mac installieren, die erste Mausgeste zeichnen, Einstellungen öffnen und Gesten pausieren."
order: 2
---

# Erste Schritte

## Installieren und öffnen

Unter Windows laden Sie `mousetroke.exe` von der [neuesten Veröffentlichung](https://github.com/eiaao/mousetroke/releases/latest) herunter. Es ist eine portable ausführbare Datei: Speichern Sie sie an einem dauerhaften Ort und starten Sie sie per Doppelklick. Vergleichen Sie den SHA-256 des Downloads mit `SHA256SUMS.txt` auf der Veröffentlichungsseite.

Unterstützt werden Windows 10 1809 und höher sowie Windows 11, 64-Bit. Die Einstellungen nutzen Microsoft Edge WebView2, das in Windows 11 enthalten ist. Die ausführbare Datei ist noch nicht codesigniert. Falls SmartScreen den ersten Start blockiert, wählen Sie **Weitere Informationen → Trotzdem ausführen**, nachdem Sie sich vergewissert haben, dass Sie die offizielle Veröffentlichung geladen haben.

Auf dem Mac laden Sie `Mousetroke.dmg` herunter, öffnen es und ziehen **Mousetroke** in **Programme**. Die Universal-App unterstützt Apple Silicon und Intel unter macOS 11 oder höher. Wird sie innerhalb des DMG geöffnet, bietet die App **In „Programme“ verschieben** an und öffnet die installierte Kopie erneut.

Die Mac-Beta ist selbstsigniert und nicht notarisiert. Wird sie blockiert, verwenden Sie **Systemeinstellungen → Datenschutz & Sicherheit → Trotzdem öffnen**; unter älterem macOS **Rechtsklick → Öffnen**. Klicken Sie in der Einrichtung auf **Erlauben…** und aktivieren Sie Mousetroke unter **Bedienungshilfen**. Warten Sie auf **Bereit — Sie können jetzt Gesten verwenden**. Die Berechtigung für Bildschirmaufnahme ist nicht erforderlich. Die Einrichtung bietet die **Eingabeüberwachung** an, wenn die Eingabeaufnahme auf Systemen, die sie benötigen, weiterhin fehlschlägt; aktivieren Sie sie auf Aufforderung.

## Einstellungen öffnen und die Anleitung abschließen

Unter Windows klicken Sie mit der linken Maustaste auf das Symbol im Infobereich, um die Einstellungen zu öffnen. Ein Rechtsklick bietet **Einstellungen…**, **Gesten pausieren** / **Gesten fortsetzen**, **Automatisch starten**, **Konfigurationsordner öffnen** und **Beenden**. Ein Eintrag für eine neue Version erscheint zusätzlich, wenn ein Update verfügbar ist.

Auf dem Mac öffnen Sie über das Symbol in der Menüleiste **Einstellungen…**, pausieren oder setzen fort oder wählen **Beenden**. Berechtigungs- und Update-Einträge erscheinen bei Bedarf. **Über Mousetroke** öffnet das Über-Fenster.

Die Anleitung des ersten Starts erklärt das Zeichnen, Bildschirmränder, App-Ausnahmen und das Pausieren. Prüfen Sie die markierten Apps, bevor Sie sie abschließen: In diesen Apps werden Gesten ausgeschaltet. Sie können sie über **Über → Anleitung der ersten Schritte ansehen** erneut öffnen. Änderungen in den Einstellungen werden automatisch gespeichert; eine unvollständige Zeichnung muss erst aufgenommen werden, bevor gespeichert werden kann.

## Die erste Geste zeichnen

1. Öffnen Sie eine Browserseite mit Navigationsverlauf.
2. Halten Sie die rechte Maustaste gedrückt, bewegen Sie die Maus nach links, um `←` zu zeichnen, und lassen Sie dann los. Das führt „Zurück“ aus.
3. Probieren Sie `↓→` in einem entbehrlichen Tab aus, um ihn zu schließen, oder `↙`, um das Fenster zu minimieren.

Die integrierten Aktionen folgen der Plattform: Browser-Kürzel verwenden, wo passend, Strg unter Windows und Command auf dem Mac. Die vollständige, bearbeitbare Liste steht unter **Alle Apps → Mausgesten**. Einige integrierte Gesten sind deaktiviert, insbesondere Aktionen, die auf dem Mac nicht verfügbar sind.

Ein einfacher Rechtsklick öffnet weiterhin das Kontextmenü der Anwendung. Bleibt die Maus lange genug still, wird die Taste ebenfalls für normales Klicken oder Ziehen durchgereicht; passen Sie diese Zeit in den [Einstellungen](gestures.md#erkennung-und-erscheinungsbild) an.

## Pausieren

Unter Windows wählen Sie **Gesten pausieren**, dann **Gesten fortsetzen**, um sie wiederherzustellen. Auf dem Mac behält **Gesten pausieren** dieselbe Bezeichnung: Das Häkchen zeigt die manuelle Pause an, und ein erneuter Klick setzt fort. Das Standard-Kürzel für Pause/Fortsetzen ist **Win+Esc** unter Windows und **Control+Wahl+Command+P** auf dem Mac. Ändern Sie es unter **Einstellungen → Kürzel für Pause / Fortsetzen**.

Vollbildspiele, Videos und Präsentationen unterdrücken Gesten automatisch, solange das Vordergrundfenster als Vollbild erkannt wird. Siehe [App-spezifische Regeln](per-app.md) für Ausnahmen und [Fehlerbehebung](troubleshooting.md), wenn nichts auslöst.
