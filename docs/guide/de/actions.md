---
title: "Aktionen"
description: "Eine Voreinstellung aus der Aktionsbibliothek wählen oder erweiterte Tasten-, Fenster-, App-, URL-, Befehls-, Menü- und Lua-Aktionen bearbeiten."
order: 5
---

# Aktionen

Wählen oder erstellen Sie eine Geste und klicken Sie unter **Aktion** auf **Aktion auswählen**. Suchen Sie nach der gewünschten Aufgabe oder sehen Sie eine Kategorie durch, legen Sie die nötigen Parameter fest und klicken Sie auf **Diese Aktion verwenden**. Die [Aktionsbibliothek](action-library.md) enthält alle Voreinstellungen und ihre Verfügbarkeit unter Windows und Mac, einschließlich Text, Datum und Uhrzeit, Fensterlayouts und Energieaktionen.

Vorhandene Aktionen zeigen **Ersetzen**. Voreinstellungen, die sich als bearbeitbare erweiterte Aktion darstellen lassen, bieten außerdem **In erweiterte Aktion umwandeln**; native Voreinstellungen erklären, warum die Umwandlung nicht verfügbar ist. Die Kategorie **Erweitert** öffnet Tastenfolgen, Befehle, Lua und Menüs. Die folgenden Editoren bleiben verfügbar, auch über **Aktionstyp ändern**.

## Tasten senden

Geben Sie eine [Tastenfolge](key-sequences.md) ein, oder klicken Sie auf **Aufnehmen**, drücken Sie das Kürzel und klicken Sie auf **Fertig**. Zum Beispiel kopiert `^c` unter Windows und `#c` auf dem Mac. **Syntaxhilfe** öffnet die Syntaxanleitung.

**Zuerst das Fenster unter dem Cursor aktivieren** aktiviert zuerst das Gestenziel. Ohne diese Option gehen Kürzel an die Anwendung, die gerade Tastatureingaben erhält. Das Ziel folgt außerdem **Einstellungen → Gesten wirken auf**. Für unveränderten Text wählen Sie **Text einfügen** in der [Aktionsbibliothek](action-library.md) oder nutzen die [Text-zuerst-Eingabe](key-sequences.md#text-und-escaping) über Lua.

## Fensteraktionen

Wählen Sie eine Operation, angewendet auf das Gestenziel:

| Operation | Verhalten |
|---|---|
| Minimieren | Minimiert das Fenster. |
| Maximieren / Wiederherstellen | Wechselt zwischen maximierter und wiederhergestellter Größe. |
| Schließen | Fordert das Fenster zum Schließen auf. |
| Immer im Vordergrund / aufheben | Schaltet „immer im Vordergrund“ unter Windows um; auf dem Mac nicht unterstützt und im Editor deaktiviert. |

Die Mac-Fenstersteuerung nutzt Bedienungshilfen. **Maximieren / Wiederherstellen** drückt die Zoom-Taste der App; die entstehende Größe folgt dem Verhalten dieser App. Sie fordert kein natives Vollbild an und überspringt bereits minimierte Fenster. Anwendungen können verschiedene Fensteroperationen anbieten.

## App öffnen

**App** akzeptiert unter Windows eine ausführbare Datei; **Durchsuchen…** wählt sie aus. Auf dem Mac verwenden Sie eine Bundle-ID wie `com.apple.Safari`, einen `.app`-Pfad oder einen Pfad zu einer ausführbaren Datei. **Argumente** ist optional. Auf dem Mac werden Argumente mit Anführungszeichenbehandlung getrennt; verwenden Sie für Kommandozeilen-Programme vollständige Pfade.

**Wenn bereits ausgeführt** bietet **Dazu wechseln** oder **Noch eine öffnen** an. Windows bietet zusätzlich **Als Administrator ausführen**. Der Mac unterstützt weder Rechteerhöhung noch Startoptionen mit Fensterfilter. Felder für Klasse/Titel/Regex gibt es im aktuellen Aktions-Editor nicht.

## URL öffnen

Füllen Sie **URL** und optional **Browser** aus. Lassen Sie Browser leer für den Standardbrowser. Windows akzeptiert eine Browser-Datei; der Mac eine Browser-Bundle-ID oder einen App-Pfad. Die Schaltflächen **Einfügen** ergänzen Platzhalter für markierten Text, Zwischenablagentext, App-Name, Prozess-ID oder Fenstertitel.

Für die Websuche mit markiertem Text setzen Sie die URL auf:

```text
https://www.google.com/search?q=%MS_SELECTED_TEXT%
```

Markieren Sie Text in einer Anwendung und führen Sie dann die Geste aus. Der Text wird URL-kodiert in die Vorlage eingesetzt; Such- und Übersetzungsvorlagen verwenden den gewählten Dienst auch dann, wenn die Auswahl eine Adresse ist. Nur wenn das URL-Feld ausschließlich `%MS_SELECTED_TEXT%` enthält, öffnet Mousetroke markierte Adressen mit `http://`, `https://` oder `www.` direkt; `www.` erhält ein `https://`-Präfix.

## Befehl ausführen

Geben Sie den Befehl ins Textfeld ein. **Einfügen** fügt Variablen hinzu. Windows führt ihn über ein unsichtbares `cmd.exe /C` aus; der Mac verwendet `/bin/zsh -c`. Das ist Shell-Syntax; zitieren Sie Argumente korrekt. Eingefügter Text wird nicht shell-maskiert; vermeiden Sie es, nicht vertrauenswürdigen markierten Text in Befehle einzuspeisen.

Der folgende Windows-Befehl öffnet zum Beispiel den Task-Manager:

```text
taskmgr.exe
```

Auf dem Mac verwenden Sie stattdessen **App öffnen** mit `com.apple.ActivityMonitor`.

## Medien und System

Der Editor bietet **Wiedergabe / Pause**, **Nächster Titel**, **Vorheriger Titel**, **Lauter**, **Leiser**, **Stumm**, **Heller** und **Dunkler**.

Die Windows-Helligkeit stellt um 10 Prozentpunkte über DDC/CI mit WMI-Fallback ein. Die Mac-Helligkeit sendet Helligkeitstasten für den eingebauten Bildschirm; externes DDC/CI wird nicht unterstützt. Diese Mac-Systemaktionen sind verfügbar, obwohl Lua-Helligkeitsfunktionen nicht unterstützt werden. Siehe [Bildschirmränder](screen-edges.md).

## Menü anzeigen

Klicken Sie auf **+ Menüeintrag hinzufügen**, geben Sie den **Namen des Menüeintrags** ein und wählen Sie in jedem Eintrag eine Aktion. Ordnen Sie ihn mit **Nach oben**, **Nach unten** und **Menüeintrag löschen**. Das Menü erscheint an der Gestenposition. Der Editor bietet kein Feld für die Spaltenanzahl und innerhalb eines Eintrags keinen weiteren Menütyp. Der Mac unterstützt eine Spalte; mehrspaltige Einstellungen werden als nicht unterstützt markiert.

## Lua-Skript

Fügen Sie ein vollständiges Skript ein, das `ms_action` definiert. Der Editor verlinkt auf [Lua-Grundlagen](lua-basics.md), die [API-Referenz](lua-api.md) und [Beispiele](lua-examples.md). Das Textfeld wandelt Tab in zwei Leerzeichen um.

<a id="variables"></a>

## Variablen

Verwenden Sie diese Platzhalter in **URL öffnen → URL**, **Befehl ausführen** und **App öffnen → App / Argumente**. Sie werden in Tastenfolgen-Zeichenketten, Lua-Quelltext, Menünamen oder Browser nicht ersetzt.

| Variable | Wert |
|---|---|
| `%MS_SELECTED_TEXT%` | Markierung, erfasst wenn die Aktion sie braucht; leer, wenn nicht verfügbar. |
| `%MS_CLIPBOARD_TEXT%` | Aktueller Zwischenablagentext, bei Bedarf erfasst. |
| `%MS_PID%` | Prozess-ID des Ziels. |
| `%MS_EXE%` | Identität der Ziel-App: Windows-Pfad; Mac-Bundle-ID oder Pfad zur ausführbaren Datei. |
| `%MS_APP_NAME%` | Dateinamenanteil der Identität; auf dem Mac auch eine Bundle-ID. |
| `%MS_WID%` | Plattform-Fensterkennung als Dezimaltext. |
| `%MS_WIN_NAME%` | Titel des Zielfensters; der Mac holt ihn bei Bedarf über Bedienungshilfen. |
| `%MS_MOUSE_X%`, `%MS_MOUSE_Y%` | Bildschirmkoordinaten beim Gestenstart. |
| `%MS_MOUSE_Y_FLIP%` | Unterkante des virtuellen Bildschirms minus der Y-Koordinate beim Gestenstart. |
| `%MS_TARGET_MODE%` | `UnderCursor` oder `Focused`, entsprechend der Zieleinstellung. |

Nicht verfügbare Zielvariablen können fehlen; unbekannte Platzhalter bleiben unverändert. Nur markierter Text wird in einer URL-Vorlage automatisch URL-kodiert. Andere Ersetzungen werden unverändert eingefügt. Lua nutzt [Einstiegsargumente](lua-basics.md#einstiegsargumente) und `ms`-Funktionen statt der Platzhalterersetzung.
