---
title: "Lua-Rezepte"
description: "Fertige Lua-Skripte für Mausgesten kopieren: Text, Zwischenablage einfügen, Datum, Websuche, Fenster, Helligkeit und Pausieren."
order: 10
---

# Lua-Rezepte

Fügen Sie einen vollständigen Block in die Aktion **Lua-Skript** einer Geste ein. Jeder Block definiert `ms_action`. Probieren Sie Text-Rezepte zuerst in einem entbehrlichen Textdokument aus. [Lua-Grundlagen](lua-basics.md) erklärt den Editor, und [Lua-API](lua-api.md) listet die Plattformunterstützung auf.

Plattformübergreifende Rezepte verwenden `package.config`, um `^` (Strg unter Windows) oder `#` (Command auf dem Mac) zu wählen. Zwischenablagen-Rezepte bewahren **nur Text**, keine Bilder oder anderen Formate. Ihre 200-ms-Wartezeiten sind Beispiele, keine Garantie der Anwendung; verlängern Sie sie, wenn das Ziel langsam ist. Sie ersetzen den Zwischenablageninhalt vorübergehend und stellen den gesicherten Text wieder her, selbst wenn Sie in der Zwischenzeit etwas anderes kopieren.

## Festen Text eingeben

Ziel: mit einer Mausgeste einen wiederverwendbaren Satz einfügen.

```lua
function ms_action()
  ms.keys("Hello from Mousetroke.", true)
end
```

Der Text-zuerst-Modus unterstützt auf beiden Plattformen Unicode. Er parst weiterhin Syntaxzeichen; verwenden Sie für beliebige Satzzeichen wie `%`, `+` oder Klammern das Zwischenablagen-Rezept.

## Zwischenablagentext einfügen und wiederherstellen

Ziel: wörtlichen Text einfügen und danach den Text wiederherstellen, der zuvor in der Zwischenablage lag.

```lua
function ms_action()
  local mod = package.config:sub(1, 1) == "\\" and "^" or "#"
  local saved = ms.get_clipboard_text()
  ms.set_clipboard_text("Price: 50% + tax {example}")
  ms.keys(mod .. "v")
  ms.sleep(200)
  ms.set_clipboard_text(saved or "")
end
```

Das umgeht das Escaping der Tastenfolge für den eingefügten Text. Gab es keinen Zwischenablagentext, enthält die Zwischenablage am Ende leeren Text; andere ursprüngliche Formate lassen sich mit diesen APIs nicht wiederherstellen.

## Unterschiedlicher Text pro App

Ziel: anhand der Identität der ausführbaren Datei des Gestenziels eine Formulierung wählen.

```lua
function ms_action(pos, wid, pid, exe)
  local name = string.lower(exe)
  local text = "Hello from Mousetroke"
  if string.find(name, "firefox", 1, true) then
    text = "Hello Firefox"
  elseif name == "com.apple.safari" then
    text = "Hello Safari"
  end
  ms.keys(text, true)
end
```

Windows-Identitäten sind normalerweise vollständige Pfade zu ausführbaren Dateien; der Mac liefert üblicherweise eine Bundle-ID. Dies prüft das erfasste Ziel, während die Eingabe an die aktive App geht. Verwenden Sie zuerst `ms.focus_win(wid)`, wenn Sie die Ziel-App aktivieren möchten.

## Das heutige Datum einfügen

Ziel: das lokale Datum als Jahr-Monat-Tag einfügen.

```lua
function ms_action()
  ms.keys(os.date("%Y-%m-%d"), true)
end
```

`os.date` steht in der Laufzeit zur Verfügung. Die `%`-Zeichen sind Syntax des Lua-Datumsformats; die entstehenden Ziffern und Bindestriche sind für die Text-zuerst-Eingabe sicher.

## Markierten Text auf einer Website suchen

Ziel: die Markierung kopieren und eine UTF-8-kodierte Google-Anfrage öffnen.

```lua
local function url_encode(text)
  return (text:gsub("([^A-Za-z0-9%-._~])", function(c)
    return string.format("%%%02X", string.byte(c))
  end))
end

function ms_action()
  local mod = package.config:sub(1, 1) == "\\" and "^" or "#"
  local saved = ms.get_clipboard_text()
  ms.set_clipboard_text("")
  ms.keys(mod .. "c")
  ms.sleep(200)
  local text = ms.get_clipboard_text() or ""
  ms.set_clipboard_text(saved or "")
  if text ~= "" then
    ms.open("https://www.google.com/search?q=" .. url_encode(text))
  else
    ms.show_toast("No selected text copied")
  end
end
```

Markieren Sie vor dem Auslösen Text in der aktiven App. Das vorherige Leeren der Zwischenablage verhindert die Suche nach altem Zwischenablagentext, falls Kopieren nichts bewirkt. Lua hat keine automatische Ersetzung von markiertem Text; die einfachere [Aktion URL öffnen](actions.md#url-öffnen) kann die Markierung für Sie einsammeln.

## Markierten Text in Anführungszeichen oder Klammern einschließen

Ziel: den markierten Text in einem bearbeitbaren Feld durch eine zitierte Fassung ersetzen.

```lua
function ms_action()
  local mod = package.config:sub(1, 1) == "\\" and "^" or "#"
  local saved = ms.get_clipboard_text()
  ms.set_clipboard_text("")
  ms.keys(mod .. "c")
  ms.sleep(200)
  local text = ms.get_clipboard_text() or ""
  if text ~= "" then
    ms.set_clipboard_text('"' .. text .. '"')
    ms.keys(mod .. "v")
    ms.sleep(200)
  end
  ms.set_clipboard_text(saved or "")
end
```

Für Klammern ersetzen Sie den Ausdruck zum Einschließen durch `"[" .. text .. "]"`. Kopieren muss den Text markiert lassen, damit Einfügen ihn ersetzt. Schreibgeschützte Seiten lassen sich auf diese Weise nicht bearbeiten.

## Immer im Vordergrund umschalten

Ziel: den Zustand „immer im Vordergrund“ eines Windows-Fensters umschalten. Wählen Sie für dieses Rezept **Fensteraktionen → Immer im Vordergrund / aufheben**: Diese Aktion kann den tatsächlichen Zustand abfragen und umschalten. Die Lua-API setzt nur einen gewünschten Zustand; sie hat keinen Getter dafür, und jede Geste erhält eine frische VM. Der Mac unterstützt die app-übergreifende Steuerung von „immer im Vordergrund“ nicht.

Wenn Sie unter Windows eine zusätzliche Lua-Geste „dieses Fenster anheften“ möchten, verwenden Sie:

```lua
function ms_action(pos, wid)
  if package.config:sub(1, 1) ~= "\\" then
    ms.show_toast("Always-on-top is unavailable on Mac")
    return
  end
  if wid ~= 0 then ms.set_win_top(wid, true) end
end
```

Verwenden Sie `false` statt `true` für eine separate Geste zum Lösen. Ein globales Lua-Boolean würde bei jeder Ausführung zurückgesetzt und keinen Umschalter umsetzen.

## Maximieren oder wiederherstellen

Ziel: den maximierten Zustand des Zielfensters unter Windows oder den Arbeitsbereich-Zoom auf dem Mac umschalten.

```lua
function ms_action(pos, wid)
  if wid == 0 then return end
  local state = ms.get_win_state(wid)
  if state == nil then
    ms.show_toast("Window unavailable; check Accessibility permission on Mac")
    return
  end
  ms.set_win_state(wid, state == "max" and "normal" or "max")
end
```

Der Mac verwendet dieselbe AX-Zoom-Taste wie die integrierte Fensteraktion. Die Zustandsabfrage erkennt Fenster in Arbeitsbereichsgröße; Apps, die auf ihre Inhaltsgröße zoomen, melden eventuell weiterhin normal — verwenden Sie für diese Apps **Fensteraktionen → Maximieren / Wiederherstellen**. Eine fehlende Bedienungshilfen-Berechtigung oder ein nicht verfügbares Fenster liefert nil. Eine Lua-API für Fensterposition/-größe zum Verschieben eines Fensters auf einen anderen Monitor gibt es nicht.

## Helligkeit unter dem Cursor

Ziel: die Helligkeit des Bildschirms unter dem Cursor unter Windows um 5 Prozentpunkte erhöhen.

```lua
function ms_action()
  local p = ms.get_mouse_pos()
  local value = ms.set_brightness(p, 5)
  if value ~= nil then
    ms.show_toast("Brightness: " .. value)
  else
    ms.show_toast("Brightness API unavailable for this screen/platform")
  end
end
```

Ändern Sie `5` in `-5` für eine zweite, dunklere Geste. Windows braucht kompatible DDC/CI- oder WMI-Hardware; WMI fällt auf sein erstes Helligkeitsgerät zurück, was ein anderer Bildschirm sein kann. Erfolg ist ein Prozentsatz von 0–100; Fehler ist nil auf beiden Plattformen. Der Mac liefert hier nil: sein bestehendes Backend sendet nur Helligkeitstasten und kann weder einen Prozentsatz liefern noch ein Prozentpunkt-Delta anwenden. Verwenden Sie **Medien und System → Heller / Dunkler** für Helligkeitstasten des eingebauten Bildschirms, ohne Auswahl des Cursor-Bildschirms oder externes DDC/CI.

## Einen Ordner öffnen

Ziel: den Ordner „Downloads“ des aktuellen Benutzers mit dem Dateibrowser der Plattform öffnen.

```lua
function ms_action()
  if package.config:sub(1, 1) == "\\" then
    ms.cmd('explorer.exe "%USERPROFILE%\\Downloads"')
  else
    ms.cmd('/usr/bin/open "$HOME/Downloads"')
  end
end
```

Diese Umgebungsvariablen werden von der Shell des Betriebssystems expandiert, nicht von Mousetrokes Aktionsvariablen. Das nimmt den üblichen Speicherort von „Downloads“ an.

## Ein Programm mit Argumenten starten

Ziel: einem Programm eine Argumentzeichenkette übergeben.

```lua
function ms_action()
  if package.config:sub(1, 1) == "\\" then
    ms.run("cmd.exe", "/K echo Hello from Mousetroke")
  else
    ms.run("/usr/bin/say", '"Hello from Mousetroke"')
  end
end
```

Windows öffnet ein Befehlsfenster mit einer Meldung. Der Mac spricht den Satz mit `say` aus. `ms.run` akzeptiert eine Argumentzeichenkette, keine Tabelle, und liefert kein Prozess-Handle.

## Task-Manager oder Activity Monitor öffnen

Ziel: die Prozessanzeige der Plattform öffnen.

```lua
function ms_action()
  if package.config:sub(1, 1) == "\\" then
    ms.keys("^+{ESC}")
  else
    ms.run("com.apple.ActivityMonitor")
  end
end
```

Windows verwendet Strg+Umschalt+Esc. Der Mac startet Activity Monitor über die Bundle-ID.

## Den Bildschirm sperren

Ziel: die aktuelle Sitzung mit ihrem Systemkürzel sperren.

```lua
function ms_action()
  if package.config:sub(1, 1) == "\\" then
    ms.keys("#l")
  else
    ms.keys("^#q")
  end
end
```

Der Mac verwendet Control+Command+Q. Windows verwendet Win+L; Strg+Alt+Entf lässt sich nicht als gewöhnliche Tastenfolge synthetisieren.

## Gesten für zehn Minuten pausieren

Ziel: die Engine vorübergehend pausieren und danach fortsetzen, während Mousetroke weiterläuft.

```lua
function ms_action()
  ms.set_paused(true)
  ms.show_toast("Gestures paused for ten minutes", 2000)
  ms.sleep(10 * 60 * 1000)
  ms.set_paused(false)
  ms.show_toast("Gestures resumed", 1500)
end
```

Das funktioniert auf beiden Plattformen und hält einen Lua-Worker zehn Minuten lang schlafend. Es ist kein dauerhafter Timer: Beenden oder Neustart beendet ihn. Sie können während der Wartezeit manuell fortsetzen, aber das Skript stellt am Ende der Wartezeit dennoch auf fortgesetzt. Vermeiden Sie es, mehrere Kopien zu starten. Siehe [Laufzeitgrenzen](lua-basics.md#laufzeit-und-grenzen).
