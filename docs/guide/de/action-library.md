---
title: "Aktionsbibliothek"
description: "Wählen Sie fertige Aktionen, legen Sie ihre Parameter fest und prüfen Sie den vollständigen Katalog für Windows und Mac."
order: 5.5
---

# Aktionsbibliothek

Voreinstellungen beschreiben, was Sie tun möchten, etwa einen Tab schließen oder ein Fenster zentrieren. Mousetroke speichert die ID und die Parameter der Voreinstellung und wählt beim Ausführen die passende Umsetzung für Windows oder Mac. Browser- und Editor-Kürzel funktionieren weiterhin nur, wenn die empfangende App sie unterstützt.

## Eine Aktion auswählen

1. Erstellen oder wählen Sie eine Geste und klicken Sie auf **Aktion auswählen** oder bei einer vorhandenen Aktion auf **Ersetzen**.
2. Suchen Sie nach Name, Beschreibung, Alias oder Tastenkürzel, etwa `ctrl+w` unter Windows oder `cmd+w` / `⌘w` auf dem Mac. Die Suche umfasst alle Kategorien. Ohne Suchbegriff können Sie **Häufig**, **Browser**, **Bearbeiten**, **Text**, **Fenster**, **Desktop & System**, **Medien**, **Öffnen** oder **Mousetroke** durchsehen.
3. Wählen Sie ein Ergebnis. Aktionen mit Parametern öffnen ein Formular: Geben Sie Text ein, wählen Sie ein Datumsformat oder einen Suchdienst, oder geben Sie eine URL an bzw. wählen Sie eine Datei, einen Ordner oder eine App. Bestätigen Sie mit **Diese Aktion verwenden**. Nicht verfügbare Aktionen sind deaktiviert; ein Hinweis erklärt die Plattformbeschränkung oder die geplante Umsetzung in der zweiten Phase.
4. Testen Sie die Geste außerhalb der Einstellungen. Ein leerer Gestenname übernimmt den Namen der gewählten Aktion. Menüeinträge verwenden denselben Auswahldialog.

Beim Öffnen erhält das Suchfeld den Fokus. Mit den Pfeiltasten nach oben und unten wechseln Sie zwischen verfügbaren Ergebnissen, mit Enter wählen Sie eines aus. Esc schließt zuerst das Parameterformular, dann den Auswahldialog. In mehrzeiligen Textfeldern bestätigt Ctrl+Enter oder Command+Enter das Formular.

**In erweiterte Aktion umwandeln** ersetzt eine Voreinstellung durch bearbeitbare Tasten, eine Fensteroperation, eine URL, eine App, einen Befehl oder eine Systemtaste für die aktuelle Plattform. Die Bestätigung erklärt, dass die Umwandlung nur in eine Richtung möglich ist: Das Ergebnis folgt nicht mehr den Plattform- oder Sprachvorgaben der Voreinstellung. Native Aktionen für Fensterlayouts, Energie, unveränderten Text, Datum und Uhrzeit, Kopieren als reinen Text, Pause und Einstellungen bleiben Voreinstellungen; ihre Karten erklären, warum die Umwandlung nicht verfügbar ist. Die Kategorie **Erweitert** am Listenende bietet Tastenfolgen, Befehle, Lua und Menüs. Die [Referenz für erweiterte Aktionen](actions.md) beschreibt auch die bisherigen Editoren.

## Text, Fenster und Energie

- **Text einfügen** gibt den Text unverändert aus, einschließlich Leerzeichen, Zeilenumbrüchen, chinesischer Schrift, Emoji und Zeichen wie `^+#{ENTER}`. Standardmäßig verwendet **Einfügen und Zwischenablage wiederherstellen** vorübergehend die Zwischenablage und stellt den vorherigen **Text** nach einer kurzen Wartezeit wieder her. Enthält die Zwischenablage Bilder, HTML, Dateilisten oder andere Nicht-Text-Daten oder lässt sie sich nicht prüfen, wird der Text direkt eingegeben, ohne die Zwischenablage zu verändern. `paste_delay_ms` ist nur bei `paste` sichtbar: Die Oberfläche erlaubt 50–5000 ms, der Standard ist 150 ms. **Zeichen eingeben** verwendet Unicode-Eingabe ohne Zwischenablage. Beide Methoden senden an die App, die gerade Tastatureingaben erhält.
- **Datum und Uhrzeit einfügen** verwendet die lokale Systemzeit und die gewählte Ausgabemethode. Zur Auswahl stehen ISO-Datum, chinesisches Datum, US-Datum, Uhrzeit und Datum mit Uhrzeit. Ein eigenes Muster akzeptiert `YYYY`, `MM`, `DD`, `HH`, `mm` und `ss`; Groß- und Kleinschreibung sind wichtig. Andere Zeichen werden unverändert ausgegeben. Einfache Anführungszeichen schützen wörtlichen Text, `''` fügt ein Anführungszeichen ein. Beispielsweise ergibt `YYYY-MM-DD HH:mm:ss` einen Zeitstempel. Nicht geschlossene Anführungszeichen werden abgelehnt.
- **Auswahl als reinen Text kopieren** sendet Ctrl+C / Command+C, wartet bis zu 500 ms auf eine Versionsänderung der Zwischenablage und ersetzt den kopierten Inhalt dann durch seinen reinen Text. Wird das Kopieren nicht abgeschlossen oder enthält das Ergebnis keinen Text, erscheint eine Fehlermeldung und der Inhalt bleibt erhalten. Markieren Sie vorher Text und lassen Sie die betreffende App im Vordergrund.
- Layout-Aktionen verwenden dasselbe Gestenziel wie andere Fensteraktionen, entsprechend **Einstellungen → Gesten wirken auf**. Windows verwendet die native Fensterpositionierung, Mac die Bedienungshilfen und den nutzbaren Bildschirmbereich. Mindestgrößen und App-Beschränkungen können eine genaue Halbierung verhindern. Verlassen Sie auf dem Mac zuerst den nativen Vollbildmodus. **Auf nächsten Bildschirm verschieben** durchläuft die Bildschirme in einer festen Reihenfolge und erhält relative Position und Maximierung; bei nur einem Bildschirm passiert nichts.
- Nach **Bildschirm ausschalten** lassen sich die Bildschirme durch Eingaben wieder aufwecken. **Ruhezustand** fordert den normalen Systemschlaf an; Konto- oder Systemrichtlinien können dies verhindern. Scheitert eine native Aktion, wird eine Protokollzeile geschrieben und kurz eine Bildschirmmeldung angezeigt. **Gesten pausieren / fortsetzen** verwendet denselben Pausenzustand wie das Infobereich-Menü und das Tastenkürzel; darüber können Sie pausierte Gesten wieder fortsetzen. **Einstellungen öffnen** öffnet das gewohnte Einstellungsfenster.

## Katalog

Die folgenden Tabellen werden aus der Registrierung erzeugt und enthalten alle 87 Einträge. ✓ bedeutet, dass die Aktion auf dieser Plattform umgesetzt ist; die Verfügbarkeit hängt weiterhin von App, Hardware und nötigen Berechtigungen ab. Die Gesten-Kurzübersicht ist für die zweite Phase vorgesehen.

<!-- action-library:generated:start -->

### Browser

| Aktion / ID | Funktion und Einsatzbereich | Parameter | Windows | Mac |
|---|---|---|---|---|
| Zurück<br>`browser.back` | Für Webbrowser. | — | ✓ | ✓ |
| Vorwärts<br>`browser.forward` | Für Webbrowser. | — | ✓ | ✓ |
| Aktualisieren<br>`browser.reload` | Für Webbrowser. | — | ✓ | ✓ |
| Ohne Cache neu laden<br>`browser.hard_reload` | Für Webbrowser. | — | ✓ | ✓ |
| Neuer Tab<br>`browser.new_tab` | Browser, Editoren und andere Apps mit diesem Tastenkürzel. | — | ✓ | ✓ |
| Tab schließen<br>`browser.close_tab` | Browser, Editoren und andere Apps mit diesem Tastenkürzel. | — | ✓ | ✓ |
| Geschlossenen Tab wieder öffnen<br>`browser.reopen_tab` | Für Webbrowser. | — | ✓ | ✓ |
| Vorheriger Tab<br>`browser.prev_tab` | Browser, Editoren und andere Apps mit diesem Tastenkürzel. | — | ✓ | ✓ |
| Nächster Tab<br>`browser.next_tab` | Browser, Editoren und andere Apps mit diesem Tastenkürzel. | — | ✓ | ✓ |
| Neues Fenster<br>`browser.new_window` | Browser, Editoren und andere Apps mit diesem Tastenkürzel. | — | ✓ | ✓ |
| Neues privates Fenster<br>`browser.private_window` | Chrome, Edge und Safari; Firefox nutzt ein anderes Kürzel. | — | ✓ | ✓ |
| Adressleiste fokussieren<br>`browser.address_bar` | Für Webbrowser. | — | ✓ | ✓ |
| Auf Seite suchen<br>`browser.find` | Browser, Editoren und andere Apps mit diesem Tastenkürzel. | — | ✓ | ✓ |
| Seite als Lesezeichen speichern<br>`browser.bookmark` | Für Webbrowser. | — | ✓ | ✓ |
| Vergrößern<br>`view.zoom_in` | Browser, Editoren und andere Apps mit diesem Tastenkürzel. | — | ✓ | ✓ |
| Verkleinern<br>`view.zoom_out` | Browser, Editoren und andere Apps mit diesem Tastenkürzel. | — | ✓ | ✓ |
| Zoom zurücksetzen<br>`view.zoom_reset` | Browser, Editoren und andere Apps mit diesem Tastenkürzel. | — | ✓ | ✓ |

### Bearbeiten

| Aktion / ID | Funktion und Einsatzbereich | Parameter | Windows | Mac |
|---|---|---|---|---|
| Zum Anfang<br>`view.top` | Browser, Editoren und andere Apps mit diesem Tastenkürzel. | — | ✓ | ✓ |
| Ans Ende<br>`view.bottom` | Browser, Editoren und andere Apps mit diesem Tastenkürzel. | — | ✓ | ✓ |
| Eine Seite nach oben<br>`view.page_up` | Browser, Editoren und andere Apps mit diesem Tastenkürzel. | — | ✓ | ✓ |
| Eine Seite nach unten<br>`view.page_down` | Browser, Editoren und andere Apps mit diesem Tastenkürzel. | — | ✓ | ✓ |
| Kopieren<br>`edit.copy` | Browser, Editoren und andere Apps mit diesem Tastenkürzel. | — | ✓ | ✓ |
| Ausschneiden<br>`edit.cut` | Browser, Editoren und andere Apps mit diesem Tastenkürzel. | — | ✓ | ✓ |
| Einfügen<br>`edit.paste` | Browser, Editoren und andere Apps mit diesem Tastenkürzel. | — | ✓ | ✓ |
| Einfügen und Eingabetaste<br>`edit.paste_enter` | Browser, Editoren und andere Apps mit diesem Tastenkürzel. | — | ✓ | ✓ |
| Als reinen Text einfügen<br>`edit.paste_plain` | Apps mit Einfügen ohne Formatierung; Kürzel können abweichen. | — | ✓ | ✓ |
| Alles auswählen<br>`edit.select_all` | Browser, Editoren und andere Apps mit diesem Tastenkürzel. | — | ✓ | ✓ |
| Rückgängig<br>`edit.undo` | Browser, Editoren und andere Apps mit diesem Tastenkürzel. | — | ✓ | ✓ |
| Wiederholen<br>`edit.redo` | Browser, Editoren und andere Apps mit diesem Tastenkürzel. | — | ✓ | ✓ |
| Speichern<br>`edit.save` | Browser, Editoren und andere Apps mit diesem Tastenkürzel. | — | ✓ | ✓ |
| Eingabetaste<br>`key.enter` | Browser, Editoren und andere Apps mit diesem Tastenkürzel. | — | ✓ | ✓ |
| Rücktaste<br>`key.backspace` | Browser, Editoren und andere Apps mit diesem Tastenkürzel. | — | ✓ | ✓ |
| Entf<br>`key.delete` | Browser, Editoren und andere Apps mit diesem Tastenkürzel. | — | ✓ | ✓ |
| Abbrechen (Esc)<br>`key.escape` | Browser, Editoren und andere Apps mit diesem Tastenkürzel. | — | ✓ | ✓ |
| Vollbild<br>`view.fullscreen` | Browser, Editoren und andere Apps mit diesem Tastenkürzel. | — | ✓ | ✓ |

### Text

| Aktion / ID | Funktion und Einsatzbereich | Parameter | Windows | Mac |
|---|---|---|---|---|
| Text einfügen<br>`text.type` | Texteingaben in Apps; fügt ein und stellt die Zwischenablage wieder her. | Text (`text`): erforderlich<br>Eingabemethode (`method`): Einfügen und Zwischenablage wiederherstellen (`paste`) / Zeichen eingeben (`type`); Standard: Einfügen und Zwischenablage wiederherstellen<br>Wartezeit nach Einfügen (ms) (`paste_delay_ms`): Standard: `150`; nur bei Eingabemethode: Einfügen und Zwischenablage wiederherstellen | ✓ | ✓ |
| Datum und Uhrzeit einfügen<br>`text.datetime` | Texteingaben in Apps; nutzt lokale Datums- und Zeitangaben. | Datums- und Zeitformat (`format`): 2026-10-24 (`date_iso`) / 2026年10月24日 (`date_local`) / 10/24/2026 (`date_us`) / 14:30 (`time`) / 2026-10-24 14:30 (`datetime`) / Benutzerdefiniert (`custom`); Standard: 2026-10-24<br>Eigenes Format (`custom_format`): Standard: `YYYY-MM-DD`; nur bei Datums- und Zeitformat: Benutzerdefiniert<br>Eingabemethode (`method`): Einfügen und Zwischenablage wiederherstellen (`paste`) / Zeichen eingeben (`type`); Standard: Einfügen und Zwischenablage wiederherstellen<br>Wartezeit nach Einfügen (ms) (`paste_delay_ms`): Standard: `150`; nur bei Eingabemethode: Einfügen und Zwischenablage wiederherstellen | ✓ | ✓ |
| Websuche<br>`text.search` | Nutzt den Standardbrowser; zuerst Text in der aktuellen App auswählen. | Suchmaschine (`engine`): Bing (`bing`) / Google (`google`) / Baidu (`baidu`) / DuckDuckGo (`duckduckgo`) / Benutzerdefiniert (`custom`); Baidu bei vereinfachter chinesischer Oberfläche, sonst Google<br>Eigene URL (`url`): Standard: `https://www.google.com/search?q=%MS_SELECTED_TEXT%`; nur bei Suchmaschine: Benutzerdefiniert | ✓ | ✓ |
| Ausgewählten Text übersetzen<br>`text.translate` | Nutzt den Standardbrowser; zuerst Text in der aktuellen App auswählen. | Übersetzungsdienst (`service`): Google Übersetzer (`google`) / DeepL (`deepl`) / Bing Übersetzer (`bing`); Standard: Google Übersetzer<br>Zielsprache (`target_language`): Sprache der Oberfläche verwenden (`ui`) / 简体中文 (`zh-CN`) / English (`en-US`) / español (`es-ES`) / Français (`fr-FR`) / Deutsch (`de-DE`) / 日本語 (`ja-JP`) / Português (Brasil) (`pt-BR`) / Русский (`ru-RU`) / Italiano (`it-IT`) / 한국어 (`ko-KR`); Standard: Sprache der Oberfläche verwenden | ✓ | ✓ |
| Ausgewählte URL öffnen<br>`text.open_selected_url` | Nutzt den Standardbrowser; zuerst Text in der aktuellen App auswählen. | — | ✓ | ✓ |
| Auswahl als reinen Text kopieren<br>`text.copy_plain` | Apps mit Textkopie; entfernt Formatierungen aus der Zwischenablage. | — | ✓ | ✓ |

### Fenster

| Aktion / ID | Funktion und Einsatzbereich | Parameter | Windows | Mac |
|---|---|---|---|---|
| Minimieren<br>`window.minimize` | Gilt für das Zielfenster der Geste. | — | ✓ | ✓<br>Erfordert die Berechtigung Bedienungshilfen. |
| Maximieren / Wiederherstellen<br>`window.toggle_maximize` | Gilt für das Zielfenster der Geste. | — | ✓ | ✓<br>Erfordert die Berechtigung Bedienungshilfen. |
| Fenster schließen<br>`window.close` | Gilt für das Zielfenster der Geste. | — | ✓ | ✓<br>Erfordert die Berechtigung Bedienungshilfen. |
| App beenden<br>`app.quit` | Gilt für das Zielfenster der Geste. | — | ✓ | ✓ |
| Prozess beenden<br>`app.kill` | Beendet die Ziel-App sofort; ungespeicherte Daten gehen verloren. | — | ✓ | ✓ |
| Immer im Vordergrund<br>`window.toggle_topmost` | Immer im Vordergrund für das Zielfenster umschalten. | — | ✓ | Diese Aktion wird nur unter Windows unterstützt. |
| An linke Bildschirmhälfte anpassen<br>`window.snap_left` | Gilt für das Zielfenster der Geste. | — | ✓ | ✓<br>Erfordert die Berechtigung Bedienungshilfen. |
| An rechte Bildschirmhälfte anpassen<br>`window.snap_right` | Gilt für das Zielfenster der Geste. | — | ✓ | ✓<br>Erfordert die Berechtigung Bedienungshilfen. |
| Fenster zentrieren<br>`window.center` | Gilt für das Zielfenster der Geste. | — | ✓ | ✓<br>Erfordert die Berechtigung Bedienungshilfen. |
| Auf nächsten Bildschirm verschieben<br>`window.next_monitor` | Gilt für das Zielfenster der Geste. | — | ✓ | ✓<br>Erfordert die Berechtigung Bedienungshilfen. |
| Aktuelle App ausblenden<br>`app.hide` | Aktuelle Anwendung ausblenden. | — | Diese Aktion wird nur unter macOS unterstützt. | ✓ |

### Desktop & System

| Aktion / ID | Funktion und Einsatzbereich | Parameter | Windows | Mac |
|---|---|---|---|---|
| App wechseln<br>`desktop.switch_app` | Systemaktion; Verhalten hängt teils von Systemeinstellungen für Kürzel ab. | — | ✓ | ✓ |
| Taskansicht / Mission Control<br>`desktop.task_view` | Systemaktion; Verhalten hängt teils von Systemeinstellungen für Kürzel ab. | — | ✓ | ✓ |
| Desktop anzeigen<br>`desktop.show` | Systemaktion; Verhalten hängt teils von Systemeinstellungen für Kürzel ab. | — | ✓ | ✓ |
| Vorheriger virtueller Desktop<br>`desktop.prev` | Systemaktion; Verhalten hängt teils von Systemeinstellungen für Kürzel ab. | — | ✓ | ✓ |
| Nächster virtueller Desktop<br>`desktop.next` | Systemaktion; Verhalten hängt teils von Systemeinstellungen für Kürzel ab. | — | ✓ | ✓ |
| Startmenü / Spotlight<br>`desktop.launcher` | Systemaktion; Verhalten hängt teils von Systemeinstellungen für Kürzel ab. | — | ✓ | ✓ |
| Dateimanager öffnen<br>`desktop.files` | Systemaktion; Verhalten hängt teils von Systemeinstellungen für Kürzel ab. | — | ✓ | ✓ |
| Schnappschuss<br>`capture.region` | Systemaktion; Verhalten hängt teils von Systemeinstellungen für Kürzel ab. | — | ✓ | ✓ |
| Bildschirm aufnehmen und speichern<br>`capture.screen` | Systemaktion; Verhalten hängt teils von Systemeinstellungen für Kürzel ab. | — | ✓ | ✓ |
| Bildschirmfoto- und Aufnahme-Werkzeuge<br>`capture.tool` | Systemaktion; Verhalten hängt teils von Systemeinstellungen für Kürzel ab. | — | ✓ | ✓ |
| Zwischenablageverlauf<br>`desktop.clipboard_history` | Zwischenablageverlauf öffnen. | — | ✓ | Diese Aktion wird nur unter Windows unterstützt. |
| Emojis und Symbole<br>`desktop.emoji` | Systemaktion; Verhalten hängt teils von Systemeinstellungen für Kürzel ab. | — | ✓ | ✓ |
| Mitteilungszentrale<br>`desktop.notifications` | Mitteilungszentrale öffnen. | — | ✓ | Diese Aktion wird nur unter Windows unterstützt. |
| Bildschirm sperren<br>`system.lock` | Systemaktion; Verhalten hängt teils von Systemeinstellungen für Kürzel ab. | — | ✓ | ✓ |
| Task-Manager / Aktivitätsanzeige<br>`system.task_manager` | Systemaktion; Verhalten hängt teils von Systemeinstellungen für Kürzel ab. | — | ✓ | ✓ |
| Dialog „Sofort beenden“<br>`system.force_quit_dialog` | Dialog zum sofortigen Beenden öffnen. | — | Diese Aktion wird nur unter macOS unterstützt. | ✓ |
| Systemeinstellungen<br>`system.settings` | Systemaktion; Verhalten hängt teils von Systemeinstellungen für Kürzel ab. | — | ✓ | ✓ |
| Rechner<br>`system.calculator` | Systemaktion; Verhalten hängt teils von Systemeinstellungen für Kürzel ab. | — | ✓ | ✓ |
| Bildschirm ausschalten<br>`system.display_off` | Systemaktion; Verhalten hängt teils von Systemeinstellungen für Kürzel ab. | — | ✓ | ✓ |
| Ruhezustand<br>`system.sleep` | Systemaktion; Verhalten hängt teils von Systemeinstellungen für Kürzel ab. | — | ✓ | ✓ |

### Medien

| Aktion / ID | Funktion und Einsatzbereich | Parameter | Windows | Mac |
|---|---|---|---|---|
| Wiedergabe / Pause<br>`media.play_pause` | Medientasten des Systems; Unterstützung durch den Player erforderlich. | — | ✓ | ✓ |
| Vorheriger Titel<br>`media.previous` | Medientasten des Systems; Unterstützung durch den Player erforderlich. | — | ✓ | ✓ |
| Nächster Titel<br>`media.next` | Medientasten des Systems; Unterstützung durch den Player erforderlich. | — | ✓ | ✓ |
| Lauter<br>`media.volume_up` | Medientasten des Systems; Unterstützung durch den Player erforderlich. | — | ✓ | ✓ |
| Leiser<br>`media.volume_down` | Medientasten des Systems; Unterstützung durch den Player erforderlich. | — | ✓ | ✓ |
| Stumm<br>`media.mute` | Medientasten des Systems; Unterstützung durch den Player erforderlich. | — | ✓ | ✓ |
| Heller<br>`display.brightness_up` | Systemhelligkeit; hängt von Bildschirm und Treiber ab. | — | ✓ | ✓ |
| Dunkler<br>`display.brightness_down` | Systemhelligkeit; hängt von Bildschirm und Treiber ab. | — | ✓ | ✓ |

### Öffnen

| Aktion / ID | Funktion und Einsatzbereich | Parameter | Windows | Mac |
|---|---|---|---|---|
| Website öffnen<br>`open.website` | Öffnet die URL im Standardbrowser. | URL (`url`): Standard: `https://mousetroke.com/` | ✓ | ✓ |
| Ordner öffnen<br>`open.folder` | Öffnet den Pfad mit der Standard-App des Systems. | Pfad (`path`): erforderlich | ✓ | ✓ |
| Datei öffnen<br>`open.file` | Öffnet den Pfad mit der Standard-App des Systems. | Pfad (`path`): erforderlich | ✓ | ✓ |
| App öffnen oder aktivieren<br>`open.app` | Öffnet die App oder aktiviert ihr vorhandenes Fenster. | App (`app`): erforderlich | ✓ | ✓ |

### Mousetroke

| Aktion / ID | Funktion und Einsatzbereich | Parameter | Windows | Mac |
|---|---|---|---|---|
| Gesten pausieren / fortsetzen<br>`mousetroke.toggle_pause` | Pausiert oder aktiviert die Gestenerkennung in Mousetroke. | — | ✓ | ✓ |
| Einstellungen öffnen<br>`mousetroke.settings` | Öffnet die Mousetroke-Einstellungen. | — | ✓ | ✓ |
| Gestenliste anzeigen<br>`mousetroke.cheatsheet` | Zeigt Gesten für die aktuelle App; für Phase 2 geplant. | — | Diese Aktion ist für Phase 2 geplant. | Diese Aktion ist für Phase 2 geplant. |

<!-- action-library:generated:end -->

Setzen Sie nach Änderungen an der Registrierung oder den Übersetzungen `UPDATE_ACTION_DOCS=1` und führen Sie `cargo test -p mousetroke-core --test action_docs` aus, um die Kataloge aller zehn Sprachen zu aktualisieren. Ohne diese Variable prüft derselbe Befehl, ob die eingecheckten Tabellen aktuell sind.
