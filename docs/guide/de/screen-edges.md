---
title: "Bildschirmecken und -ränder"
description: "Hot Corners, Randscrollen für Lautstärke und Helligkeit, Randklicks und Reib-Gesten unter Windows und Mac konfigurieren."
order: 4
---

# Bildschirmecken und -ränder

## Die Standardeinstellungen ausprobieren

Unter **Alle Apps → Bildschirmränder** ist jede Ecken- oder Randbindung bearbeitbar.

| Auslöser | Windows-Standard | Mac-Standard |
|---|---|---|
| In die untere linke Ecke stoßen | Startmenü | Spotlight |
| In die untere rechte Ecke stoßen | Desktop anzeigen | Desktop anzeigen (F11) |
| In die obere linke Ecke stoßen | App wechseln (Alt+Tab) | Mission Control (Control+Up) |
| In die obere rechte Ecke stoßen | Taskansicht (Win+Tab) | Bindung anfangs deaktiviert |
| Am oberen Rand scrollen | Lauter / Leiser | Lauter / Leiser |
| Mittelklick am oberen Rand | Stumm | Stumm |
| Rechtsklick am oberen Rand | Wiedergabe / Pause | Wiedergabe / Pause |
| Am unteren Rand scrollen | Heller / Dunkler | Helligkeitsbindungen anfangs deaktiviert |
| Rechten Rand zweimal auf/ab reiben | Task-Manager-Bindung; zuerst **Sobald erkannt ausführen** aktivieren | Activity Monitor |

Eckenaktionen brauchen keine Taste: Stoßen Sie den Zeiger in die Ecke. Randscrollen und -klicks erfordern, dass der Zeiger den gewählten Rand berührt. Reiben bedeutet, entlang des Randes in den aufgezeichneten Richtungen zu fahren, statt das Rad zu drehen.

## Einen Rand-Auslöser konfigurieren

1. Wählen Sie **Bildschirmränder → Neuer Rand-Auslöser** im gewünschten App-Bereich oder in **Alle Apps**.
2. Wählen Sie unter **Wie sie beginnt** die Option **Bildschirmrand** und markieren Sie eine Seite oder Ecke im Diagramm.
3. Lassen Sie **Danach** für eine Eckenaktion leer oder fügen Sie mit **+ Schritt hinzufügen** Rad nach oben/unten, einen Mausklick oder eine Zeichenfolge entlang des Randes hinzu.
4. Wählen Sie eine [Aktion](actions.md). Für Lautstärke wählen Sie **Medien und System → Lauter** oder **Leiser**. Aktivieren Sie **Sobald erkannt ausführen** für wiederholtes Scrollen oder klickfreies Reiben.

Für das Reiben fügen Sie **Zeichenfolge zeichnen** hinzu und zeichnen die wechselnden Richtungen auf. Ein nützliches Beispiel ist `↑↓↑↓` am rechten Rand. Testen Sie es mit einer harmlosen Aktion wie einem Lua-Hinweis, bevor Sie etwas Störendes zuweisen.

## Helligkeit und Plattformunterschiede

**Heller** / **Dunkler** unter Windows beginnt mit dem Bildschirm unter dem Cursor, stellt um 10 Prozentpunkte ein und zeigt das Ergebnis an. Es versucht DDC/CI, dann WMI. Externe Displays müssen DDC/CI unterstützen und erlauben. Der WMI-Fallback verwendet das erste WMI-Helligkeitsgerät, meist ein eingebautes Panel; es wird nicht dem Bildschirm unter dem Cursor zugeordnet. Schlagen beide Wege fehl, erscheint eine Bildschirmmeldung. Für einen kleineren Schritt siehe [Lua-Rezepte](lua-examples.md#helligkeit-unter-dem-cursor).

**Medien und System → Heller** / **Dunkler** auf dem Mac sendet systemeigene Helligkeitstasten-Ereignisse für das eingebaute Display. Es wählt nicht den Bildschirm unter dem Cursor und unterstützt keine externen DDC/CI-Displays. Die Standardbindungen am unteren Rand sind deaktiviert. Lua-Helligkeits-APIs werden auf dem Mac ebenfalls nicht unterstützt.

Auf dem Mac warnt Mousetroke, wenn eine System-Hot Corner eine konfigurierte Ecke überlappt. Ändern Sie eine der Zuweisungen, um konkurrierende Aktionen zu vermeiden. Systemkurzbefehle wie F11 oder Control+Up hängen von den macOS-Kurzbefehleinstellungen ab.

App-Ausschlüsse und die Vollbilderkennung unterdrücken auch Rand-Auslöser. Siehe [App-spezifische Regeln](per-app.md), wenn ein Rand in einer App funktioniert und in einer anderen nicht.
