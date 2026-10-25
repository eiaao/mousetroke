---
title: "Lua-API"
description: "Vollständige Lua-5.4-ms-API-Referenz mit Signaturen, Rückgabewerten, Beispielen und Hinweisen zu Windows/macOS."
order: 9
---

# Lua-API

Das sind alle Funktionen, die die gemeinsame Laufzeit auf `ms` registriert. Die Beispiele sind Anweisungen für den Einsatz innerhalb von `ms_action`, wo `pos`, `wid`, `pid`, `exe`, `title` und `mode` verfügbar sind. Siehe [Lua-Grundlagen](lua-basics.md) für ein vollständiges Skript und [Rezepte](lua-examples.md) für fertige Aktionen.

## Konventionen

Lua-Version: **5.4** (`mlua` mit `lua54` und `vendored`). `nil` bedeutet „kein Wert“; `?` kennzeichnet einen optionalen Parameter. Funktionen ohne Rückgabewert erscheinen als `-> nil`. Ein weggelassenes optionales Argument entspricht nil. Übergeben Sie für boolesche Argumente echte boolesche Werte. Ganzzahlparameter kürzen endliche Brüche gegen null und lehnen Werte außerhalb ihres zugrunde liegenden Ganzzahlbereichs ab; verwenden Sie in Skripten Ganzzahlen.

Positionstabellen verwenden benannte Felder `{x=..., y=...}`, keine Array-Einträge. Nicht unterstützte Mac-Aufrufe protokollieren im Allgemeinen einmal pro Funktion pro Prozess; Getter liefern nil und Setter tun nichts. Ein Setter ohne Rückgabewert ist keine Erfolgsprüfung. Lua-API-Zeichenketten erhalten keine Aktionsvariablen-Ersetzung.

## Ablauf und Feedback

### `ms.log`

`ms.log(...) -> nil`

Akzeptiert beliebige Lua-Werte. Mit einer Formatzeichenkette und weiteren Argumenten werden `%s`, `%d`, `%f`, `%q` und `%%` unterstützt; andernfalls werden die Werte mit Leerzeichen verbunden. `%q` fügt kein Lua-Quoting hinzu. Schreibt auf beiden Plattformen eine `Lua:`-Zeile in das Laufzeitprotokoll. Kein Rückgabewert.

```lua
ms.log("pid=%d app=%s", pid, exe)
```

### `ms.sleep`

`ms.sleep(milliseconds: integer) -> nil`

Wartet auf eine nichtnegative Millisekundenanzahl (vorzeichenlose 64-Bit-Zahl). Blockiert auf beiden Plattformen den Worker dieses Skripts; die Gesten-Engine bzw. der Hauptthread läuft weiter. Kein Rückgabewert, und es wird kein Skript-Timeout installiert.

```lua
ms.sleep(100)
```

### `ms.is_paused`

`ms.is_paused() -> boolean`

Keine Parameter. Liefert auf beiden Plattformen das manuelle Pausenkennzeichen der Engine. Es meldet weder die automatische Vollbild-Unterdrückung noch einen App-Ausschluss.

```lua
ms.log(ms.is_paused())
```

### `ms.set_paused`

`ms.set_paused(paused: boolean) -> nil`

`true` pausiert; `false` setzt fort. Kein Rückgabewert. Auf beiden Plattformen unterstützt. Windows sendet einen Befehl an den Tray-Thread, sodass ein unmittelbares `is_paused()` noch den alten Wert sehen kann. Der Mac wendet die Änderung über die Hauptthread-Brücke an.

```lua
ms.set_paused(true)
```

### `ms.show_settings`

`ms.show_settings() -> nil`

Keine Parameter, kein Rückgabewert. Öffnet die Einstellungen auf beiden Plattformen oder holt sie nach vorn.

```lua
ms.show_settings()
```

### `ms.show_toast`

`ms.show_toast(message: string, duration_ms?: integer) -> nil`

`message` ist der angezeigte Text. Die optionale nichtnegative Dauer steht in Millisekunden, Standard **1200**. Zeigt auf beiden Plattformen eine Overlay-Meldung an; kein Rückgabewert.

```lua
ms.show_toast("Done", 1500)
```

## Tastatureingaben

### `ms.keys`

`ms.keys(sequence: string, text_by_default?: boolean) -> nil`

Parst und sendet die [Tastenfolge](key-sequences.md); der Text-zuerst-Modus ist standardmäßig `false`. Kehrt nach dem Senden zurück; fehlerhafte Syntax löst einen Lua-Fehler aus. Beide Plattformen unterstützen gewöhnliche Tastatur-/Texteingaben. Der Mac überspringt nicht unterstützte Tasten, Maus- und Rad-Aktionen und erzwingt seine Warteschlangen-/Textgrenzen. Er wartet auf seine Eingabewarteschlange im Hauptthread, die auch andere Folgen enthalten kann.

```lua
ms.keys("Hello", true)
```

### `ms.keys_down`

`ms.keys_down(sequence: string) -> nil`

Nur Windows: parst im Tastenmodus und drückt Tasten/Modifikatoren, ohne sie loszulassen. Wiederholungen werden in dieser Phase nicht wiederholt; Rad-Aktionen lösen einmal aus. Ungültige Syntax löst einen Lua-Fehler aus. Zum Loslassen mit `keys_up` kombinieren. Der Mac protokolliert die Funktion als nicht unterstützt und tut nichts. Kein Rückgabewert.

```lua
ms.keys_down("^a")
ms.sleep(50)
ms.keys_up("^a")
```

### `ms.keys_up`

`ms.keys_up(sequence: string) -> nil`

Nur Windows: löst die geparste Folge in umgekehrter Aktionsreihenfolge, mit den Modifikatoren zuletzt. Sleep-Schritte warten weiterhin. Syntaxfehler lösen einen Lua-Fehler aus. Der Mac protokolliert die Funktion als nicht unterstützt und tut nichts. Kein Rückgabewert.

```lua
ms.keys_down("{SHIFT}")
ms.keys_up("{SHIFT}")
```

### `ms.is_keys_pressed`

`ms.is_keys_pressed(sequence: string) -> boolean | nil`

Beide Plattformen prüfen jede Taste **und ihre Präfix-Modifikatoren**: `"^a"` erfordert Control und A. Jede Seite erfüllt ein Präfix; explizite Namen wie `{RCTRL}` erfordern genau diese Seite. Auf dem Mac ist `^` = Control, `%` = Option, `+` = Shift und `#` = Command. Mehrere Schritte bedeuten, dass alle aufgeführten Tasten/Modifikatoren gleichzeitig gedrückt sein müssen, nicht dass sie in dieser Reihenfolge gedrückt wurden; Wiederholungen fügen keine Anforderung hinzu. Maus-/Rad-, Sleep- oder Text-Aktionen machen die Prüfung falsch; eine leere Folge liefert true. Ungültige Syntax liefert false. Der Mac liest den kombinierten Sitzungszustand und liefert ohne Bedienungshilfen-Berechtigung false, mit einmaligem Protokolleintrag.

```lua
if ms.is_keys_pressed("^a") then
  ms.show_toast("Ctrl and A are down")
end
```

## Zwischenablage und Dateien

### `ms.get_clipboard_text`

`ms.get_clipboard_text() -> string | nil`

Keine Parameter. Liefert den Zwischenablagentext oder `nil`, wenn kein Text verfügbar ist. Auf beiden Plattformen unterstützt; die Markierung wird nicht automatisch abgerufen.

```lua
local text = ms.get_clipboard_text()
ms.log(text or "No text")
```

### `ms.set_clipboard_text`

`ms.set_clipboard_text(text: string) -> nil`

Ersetzt auf beiden Plattformen den Inhalt der Zwischenablage durch den Text; kein Rückgabewert und kein Erfolgszeichen. Andere Zwischenablagenformate bleiben nicht erhalten. Siehe [Zwischenablagen-Rezepte](lua-examples.md).

```lua
ms.set_clipboard_text("Hello from Mousetroke")
```

### `ms.read_text`

`ms.read_text(path: string) -> string | nil`

Liest auf beiden Plattformen die gesamte Datei als UTF-8. Liefert `nil` bei einem Lese- oder UTF-8-Dekodierfehler. Relative Pfade verwenden das Arbeitsverzeichnis des Prozesses, nicht einen dokumentierten Skriptordner. Gelesene Daten zählen zum verfügbaren Speicher.

```lua
local text = ms.read_text("config.json")
ms.log(text and #text or 0)
```

## Windows

### `ms.find_win_by_class`

`ms.find_win_by_class(pattern: string) -> table | nil`

Durchsucht sichtbare Fenster der obersten Ebene mit einem **Rust-regulären Ausdruck**, unter Windows anhand von Klassennamen oder auf dem Mac anhand der **Bundle-IDs** der Anwendung (zum Beispiel `^com\\.apple\\.Safari$` in einer Lua-Zeichenkette). Liefert bis zu 32 Einträge in einem 1-basierten Array: `{text=title, hwnd=window_id}`; kein Treffer oder ein ungültiger Regex ergibt eine leere Tabelle. Die Groß-/Kleinschreibung wird beachtet, sofern das Muster nicht ein Flag wie `(?i)` verwendet; dies sind keine Lua-Muster. Der Mac liest Titel über Bedienungshilfen, liefert ohne Berechtigung nil und protokolliert einmal. Sein `hwnd` ist eine CGWindowID, die von `focus_win` und den Fensterzustands-APIs akzeptiert wird. Auf dem Mac werden nur aktuell sichtbare Fenster durchsucht.

```lua
local wins = ms.find_win_by_class("^Notepad$")
if wins and wins[1] then ms.focus_win(wins[1].hwnd) end
```

### `ms.find_win_by_title`

`ms.find_win_by_title(pattern: string) -> table | nil`

Durchsucht auf beiden Plattformen Fenstertitel mit einem Rust-regulären Ausdruck und liefert dieselben 1-basierten `{text, hwnd}`-Einträge wie `find_win_by_class`, höchstens 32. Ein ungültiger Regex ergibt eine leere Tabelle. Der Mac verwendet AX-Fenstertitel, benötigt die Bedienungshilfen-Berechtigung und durchsucht nur aktuell sichtbare Fenster.

```lua
local wins = ms.find_win_by_title("(?i)notepad")
if wins then ms.log(#wins) end
```

### `ms.focus_win`

`ms.focus_win(wid: integer) -> nil`

Aktiviert ein Fenster bzw. eine App; kein Rückgabewert und kein Erfolgszeichen. Windows nimmt ein HWND. Der Mac akzeptiert das Gestenziel oder eine entdeckte CGWindowID, stellt das Fenster wieder her, hebt das AX-Fenster an und aktiviert dann die besitzende App über die Hauptthread-Brücke. Mac-AX-Aufrufe laufen im Skript-Worker, verwenden die eingebaute Fenstersuche nach PID/Grenzen und benötigen die Bedienungshilfen-Berechtigung; veraltete IDs oder nicht verfügbare AX-Fenster werden ignoriert. Apps mit identischen, überlappenden Fenstergrenzen können mehrdeutig sein.

```lua
if wid ~= 0 then ms.focus_win(wid) end
```

### `ms.set_win_state`

`ms.set_win_state(wid: integer, state: string) -> nil`

Windows: `"min"` minimiert, `"max"` maximiert; **jede andere Zeichenkette stellt wieder her**, einschließlich `"normal"` und `"norm"`. Mac: `"min"` verwendet AXMinimized; `"max"` stellt wieder her/verlässt den Vollbildmodus und drückt denselben AXZoomButton wie die integrierte Aktion, wenn das Fenster den Arbeitsbereich seines Bildschirms nicht füllt. `"normal"`, `"norm"` und `"restore"` stellen wieder her/verlassen den Vollbildmodus und machen den Arbeitsbereich-Zoom rückgängig. Der Mac akzeptiert außerdem `"fullscreen"` über AXFullScreen, sofern die App es unterstützt; jeder andere Name wird ignoriert und einmal protokolliert. Die Groß-/Kleinschreibung wird unterschieden. Kein Rückgabewert. Der Mac benötigt die Bedienungshilfen-Berechtigung; AX läuft im Worker, und die Bildschirmgeometrie kommt über die Hauptthread-Brücke. Übergänge können asynchron sein.

Der Mac-Zoom ist appdefiniert: Ein Zoom auf Inhaltsgröße kann weiterhin `"normal"` melden und lässt sich mit dieser API nicht zuverlässig wiederherstellen; wiederholte `"max"`-Anforderungen können ihn umschalten. Verwenden Sie für solche Apps den integrierten Schalter **Maximieren / Wiederherstellen**. Es gibt kein öffentliches, universelles AX-Flag für den Zoomzustand.

```lua
if wid ~= 0 then ms.set_win_state(wid, "max") end
```

### `ms.get_win_state`

`ms.get_win_state(wid: integer) -> string | nil`

Windows liefert `"min"`, `"max"` oder `"normal"`. Dies ist keine Abfrage von „immer im Vordergrund“ und validiert kein veraltetes Handle; ein ungültiges Handle kann normal erscheinen. Der Mac liefert `"min"`, `"fullscreen"`, `"max"` (der Rahmen passt innerhalb von 2 logischen Punkten zum Arbeitsbereich eines Bildschirms) oder `"normal"`. Der Mac hat keinen universellen öffentlichen AX-Getter für den Zoomzustand; Zoom auf Inhaltsgröße meldet daher normal. Die Funktion liefert nil bei fehlender Berechtigung, einem nicht verfügbaren AX-Fenster/-Zustand oder einer veralteten ID; eine verweigerte Berechtigung protokolliert einmal.

```lua
if wid ~= 0 then ms.log(ms.get_win_state(wid)) end
```

### `ms.set_win_top`

`ms.set_win_top(wid: integer, top: boolean) -> nil`

Windows setzt (`true`) oder löscht (`false`) „immer im Vordergrund“, ohne das Fenster zu verschieben oder in der Größe zu ändern; kein Rückgabewert. Dies setzt einen Zustand, schaltet ihn nicht um und liefert nicht den vorherigen Zustand. Der Mac tut nichts und protokolliert einmal: macOS hat keine öffentliche API, um Fenster anderer Apps anzuheften; auch die integrierte Aktion für „immer im Vordergrund“ wird nicht unterstützt. Für ein Umschalten wählen Sie im Windows-Editor **Fensteraktionen → Immer im Vordergrund / aufheben**.

```lua
if wid ~= 0 then ms.set_win_top(wid, true) end
```

## Maus und Bildschirm

### `ms.get_mouse_pos`

`ms.get_mouse_pos() -> table`

Keine Parameter. Beide Plattformen liefern `{x=integer, y=integer}` für die aktuelle Cursorposition, die vom Einstieg `pos` abweichen kann. Windows verwendet Bildschirmpixel; der Mac verwendet globale logische Quartz-Punkte, mit nach unten wachsendem Y. Der Mac liefert `{x=0,y=0}`, wenn die Cursorabfrage fehlschlägt.

```lua
local p = ms.get_mouse_pos()
ms.log("%d,%d", p.x, p.y)
```

### `ms.set_mouse_pos`

`ms.set_mouse_pos(x: integer, y: integer) -> nil`

Bewegt den Zeiger auf beiden Plattformen zu globalen Bildschirmkoordinaten; kein Rückgabewert. Koordinaten können negativ sein. Der Mac sendet ein markiertes Bewegungsereignis, damit dadurch keine weitere Geste startet.

```lua
local p = ms.get_mouse_pos()
ms.set_mouse_pos(p.x + 20, p.y)
```

### `ms.get_screen`

`ms.get_screen(x: integer, y: integer) -> table | nil`

Liefert das vollständige Bildschirmrechteck `{left, top, right, bottom}` als globale Ganzzahlkoordinaten, nicht den nutzbaren Arbeitsbereich. Die Breite ist `right-left`; die Höhe `bottom-top`. Windows wählt den nächsten Monitor, wenn der Punkt außerhalb aller Monitore liegt. Der Mac wählt das enthaltende Display oder fällt auf das erste zwischengespeicherte zurück; die Funktion kann `nil` liefern, wenn kein Bildschirm-/Brückenergebnis verfügbar ist. Beide Plattformen unterstützen diesen Aufruf.

```lua
local p = ms.get_mouse_pos()
local s = ms.get_screen(p.x, p.y)
if s then ms.log(s.right - s.left, s.bottom - s.top) end
```

### `ms.get_brightness`

`ms.get_brightness(pos?: table) -> integer | nil`

Das optionale `{x=integer,y=integer}` wählt den Bildschirm; weggelassen/nil oder eine unvollständige Position fällt auf den aktuellen Cursor zurück. Liefert **bei Erfolg 0–100 Prozent, bei Fehler nil** auf beiden Plattformen. Windows verwendet DDC/CI mit WMI-Fallback. WMI verwendet sein erstes Helligkeitsgerät, nicht einen zum Cursor passenden Monitor. Der Mac liefert derzeit nil und protokolliert einmal: sein bestehendes Helligkeits-Backend sendet nur Helligkeitstasten-Ereignisse des eingebauten Displays, mit denen sich kein Prozentsatz lesen lässt. Externes DDC wird nicht unterstützt. Nur der Windows-Legacy-Wrapper `WG.GetBrightness` behält -1 bei Fehler.

```lua
local value = ms.get_brightness()
if value ~= nil then ms.show_toast(tostring(value)) end
```

### `ms.set_brightness`

`ms.set_brightness(pos: table | nil, delta: integer) -> integer | nil`

Trotz des Namens stellt dies um `delta` Prozentpunkte ein; es setzt keinen absoluten Prozentsatz. Übergeben Sie eine benannte `{x=...,y=...}`-Tabelle oder **nil als erstes Argument**, um den Cursor zu verwenden. Erfolg liefert den neuen **Prozentsatz von 0–100**, begrenzt auf diesen Bereich; Fehler liefert **nil auf beiden Plattformen**. Windows verwendet dieselbe DDC/CI- und WMI-Auswahl wie `get_brightness`. Nur der Windows-Legacy-Wrapper `WG.SetBrightness` behält -1 bei Fehler. Der Mac liefert nil und protokolliert einmal: das bestehende Medien-und-System-Backend sendet nur Helligkeitstasten des eingebauten Displays, ohne numerisches Auslesen oder Kontrolle in Prozentpunkten. Durch diesen Lua-Aufruf wird keine Helligkeitsänderung vorgenommen; verwenden Sie für Helligkeitstasten jene integrierten Aktionen. Externes DDC bleibt nicht unterstützt.

```lua
local value = ms.set_brightness(nil, 5)
ms.log(value)
```

## URLs und Prozesse

### `ms.open`

`ms.open(url: string) -> nil`

Öffnet eine URL mit dem Standard-Handler; kein Rückgabewert und kein Erfolgszeichen. Windows verwendet shell open und akzeptiert auch einen Datei-/Ordnerpfad. Der Mac erwartet eine URL; verwenden Sie für eine Datei eine korrekt kodierte `file://`-URL oder `ms.cmd`, um `/usr/bin/open` auf einen Pfad anzuwenden. Es erfolgt keine Variablenersetzung.

```lua
ms.open("https://mousetroke.com/")
```

### `ms.run`

`ms.run(file: string, args?: string) -> nil`

Startet mit einer optionalen Argument-**Zeichenkette**, nicht einem Lua-Array; liefert keinen Wert und keine PID. Windows verwendet einen normalen Start ohne Rechteerhöhung mit RunAgain-Verhalten. Der Mac akzeptiert eine Bundle-ID, einen App-Pfad oder einen absoluten Pfad zu einer ausführbaren Datei; Argumente werden mit Anführungszeichenbehandlung getrennt. Der Start einer ausführbaren Datei kann auf einen Hintergrundprozess zurückfallen, wenn LaunchServices sie ablehnt. Er fordert eine neue Instanz an. Keine Variablenersetzung; verwenden Sie Lua-Konkatenation. Die Ausgabe des Prozesses wird nicht zurückgegeben.

```lua
if package.config:sub(1, 1) == "\\" then
  ms.run("cmd.exe", "/K echo Hello")
else
  ms.run("/usr/bin/say", '"Hello from Mousetroke"')
end
```

### `ms.cmd`

`ms.cmd(command: string) -> nil`

Führt Shell-Code über ein unsichtbares `cmd.exe /C` unter Windows oder `/bin/zsh -c` auf dem Mac aus. Liefert keinen Wert, keine Ausgabe und keinen Exit-Code und wartet nicht auf den Abschluss des Prozesses. Der Mac begrenzt gleichzeitig verwaltete Hintergrundprozesse auf 16 und protokolliert den Abschluss. Zeichenketten werden wie angegeben ausgeführt; `%MS_*%`-Platzhalter werden von dieser Lua-API nicht ersetzt. Zitieren Sie Shell-Argumente selbst.

```lua
if package.config:sub(1, 1) == "\\" then
  ms.cmd("taskmgr.exe")
else
  ms.cmd('/usr/bin/open -a "Activity Monitor"')
end
```

Weiter geht es mit [Syntax für Tasten senden](key-sequences.md), [Rezepten](lua-examples.md) oder [Protokollpfaden](config.md#dateispeicherorte).
