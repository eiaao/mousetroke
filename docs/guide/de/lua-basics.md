---
title: "Lua-Grundlagen"
description: "Das erste Lua-Skript für eine Mausgeste schreiben, ms_action-Argumente kennenlernen, mit Protokollen debuggen und Laufzeitgrenzen verstehen."
order: 8
---

# Lua-Grundlagen

## Wann Lua verwenden

Verwenden Sie für ein Kürzel, eine Fensteroperation, eine URL, eine Anwendung oder einen Befehl eine [einfache Aktion](actions.md). Wählen Sie Lua, wenn eine Geste nach der Anwendung verzweigen, Text umwandeln oder Operationen kombinieren soll. Mousetroke bettet **Lua 5.4** ein.

## Ihr erstes Skript

1. Erstellen oder wählen Sie eine Geste in den Einstellungen.
2. Wählen Sie unter **Aktion** die Option **Lua-Skript**. Ersetzen Sie das gesamte Textfeld durch den folgenden Code.
3. Warten Sie auf die Anzeige des Speicherns und lösen Sie die Geste außerhalb der Einstellungen aus.

```lua
function ms_action(pos, wid, pid, exe, title, mode)
  ms.log("Target: %s, mode: %s", exe, mode)
  ms.show_toast("My first Lua gesture", 1500)
end
```

`function` beginnt eine Funktion und `end` beendet sie. Mousetroke lädt den Code und ruft dann `ms_action` einmal auf. Legen Sie Ihre Arbeit innerhalb dieser Funktion ab. Die Namen in Klammern empfangen den Kontext; ungenutzte Argumente können Sie weglassen. `ms.log` schreibt in das Laufzeitprotokoll und `ms.show_toast` zeigt 1500 Millisekunden lang eine Meldung an.

Der Editor öffnet über **Lua-Einstieg** diese Seite. Folgen Sie den [Lua-Rezepten](lua-examples.md), um mit einer Mausgeste Text auszugeben oder über die Zwischenablage einzufügen.

## Einstiegsargumente

Der vollständige Einstieg ist `ms_action(pos, wid, pid, exe, title, mode)`:

| Argument | Lua-Typ | Bedeutung |
|---|---|---|
| `pos` | table `{x=integer, y=integer}` | Zuletzt erfasste Cursorposition beim Auslösen der Aktion. Koordinaten können auf einem anderen Monitor negativ sein. |
| `wid` | integer | Fensterkennung des Ziels; Windows HWND, Mac-Fenster-ID. Behandeln Sie sie als opake Kennung; ohne Ziel kann sie 0 sein. |
| `pid` | integer | Prozess-ID des Ziels oder 0. |
| `exe` | string | Identität der ausführbaren Datei unter Windows, normalerweise ein vollständiger Pfad; Mac-Bundle-ID oder Pfad zur ausführbaren Datei. Leer, wenn nicht verfügbar. |
| `title` | string | Titel des Zielfensters oder eine leere Zeichenkette. Der Mac holt den Titel im Worker über Bedienungshilfen. |
| `mode` | string | Genau `UnderCursor` oder `Focused`, aus **Einstellungen → Gesten wirken auf**. Es ist weder die Auslösertaste noch der Gestenname. |

Das Ziel wird für die Geste erfasst. `pos` kann von den Startkoordinaten der Geste in den Aktionsvariablen abweichen. Tastatureingaben gehen weiterhin an die aktive App, solange Sie das Ziel nicht aktivieren. Auf dem Mac kann `ms.focus_win(wid)` nur die Ziel-App dieser Geste aktivieren. Platzhaltertext wie `%MS_PID%` wird in Lua nicht ersetzt; verwenden Sie `pid` direkt.

## Ein Fünf-Minuten-Lua-Grundkurs

`local` erstellt eine Variable. Zeichenketten verwenden einfache oder doppelte Anführungszeichen; `..` verbindet Zeichenketten. `--` beginnt einen Kommentar. `if`, `elseif`, `else`, `end` wählen einen Zweig. `nil` bedeutet „kein Wert“; nur `false` und `nil` gelten in Bedingungen als falsch (0 und `""` sind wahr).

```lua
function ms_action(pos, wid, pid, exe)
  local message = "Hello"
  if string.find(string.lower(exe), "firefox", 1, true) then
    message = "Hello, Firefox"
  else
    message = message .. " from Mousetroke"
  end
  ms.show_toast(message)
end
```

Hier sucht `string.find(s, needle, 1, true)` wörtlich und liefert eine Position oder `nil`. Das `true` verhindert, dass Satzzeichen als Muster interpretiert werden. `string.match` verwendet Lua-Muster, die sich von regulären Ausdrücken unterscheiden. Eine Tabelle kann benannte Felder oder eine Liste speichern, deren erster Index 1 ist. Eine Hilfsfunktion kann einen Wert zurückgeben:

```lua
local function app_name(identity)
  return string.match(identity, "([^/\\]+)$") or identity
end

function ms_action(pos, wid, pid, exe)
  local target = {name = app_name(exe), process = pid}
  local lines = {target.name, tostring(target.process)}
  ms.show_toast(table.concat(lines, " / "))
end
```

Lua-Zeichenketten interpretieren Escape-Sequenzen, bevor die Tastenfolge geparst wird. `"\\"` steht für einen Backslash und `"\n"` für einen Zeilenumbruch. Long-Bracket-Zeichenketten wie `[[text]]` bewahren Backslashes. Die [Anleitung zu Tasten senden](key-sequences.md) erklärt die zweite Syntaxebene.

## Debugging

Verwenden Sie `ms.log("pid=%d title=%s", pid, title)` und sehen Sie unter **Über → Protokoll öffnen** nach:

- Windows: `%APPDATA%\mousetroke\mousetroke.log`.
- Mac: `~/Library/Logs/mousetroke/mousetroke.log`.

Ladefehler und Ausführungsfehler erscheinen dort mit `gesture_script` als Chunk-Name. `ms.show_toast("Reached this step")` ist eine schnelle Sichtprüfung. Optionale API-Ergebnisse können `nil` sein; prüfen Sie sie vor Arithmetik oder Tabellenzugriff. Nicht unterstützte Mac-Aufrufe finden Sie in der [API-Referenz](lua-api.md).

## Laufzeit und Grenzen

Jede Skriptausführung erzeugt eine frische Lua-VM in einem Worker-Thread mit einem **Lua-Speicherlimit von 8 MiB**. Globale Variablen bleiben zwischen Gesten nicht erhalten. Die Laufzeit setzt kein Befehlszähler- oder Zeitlimit. Vermeiden Sie Endlosschleifen. `ms.sleep(ms)` blockiert den Worker dieses Skripts, nicht den Eingabe-Hook oder den Haupt-UI-Thread. `ms.keys` wartet, bis seine Folge abgeschlossen ist; auf dem Mac treibt der Timer des Hauptthreads die Eingaben voran, während der Lua-Worker wartet.

Verfügbare Standardbibliotheken sind die Basisfunktionen, `coroutine`, `table`, `io`, `os`, `string`, `utf8`, `math` und `package`. `debug` ist nicht geladen; das Laden nativer C-Module ist durch den abgesicherten Modus von mlua deaktiviert. `os.date` ist verfügbar. Das ist keine Sicherheitssandbox des Betriebssystems: `io`, `os` und die Prozess-APIs können mit Ihren Benutzerrechten auf Dateien zugreifen oder Befehle ausführen. Führen Sie nur Skripte und Konfigurationen aus, denen Sie vertrauen.

Weiter: [API-Referenz](lua-api.md), [Kopieren-und-Einfügen-Rezepte](lua-examples.md), [Fehlerbehebung](troubleshooting.md).
