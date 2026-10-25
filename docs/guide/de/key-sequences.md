---
title: "Syntax für Tasten senden"
description: "Die vollständige Mousetroke-Syntax für Tasten senden: Modifikatoren, Gruppen, benannte Tasten, Wiederholungen, Verzögerungen, Mauseingaben und Windows-/Mac-Kürzel."
order: 7
---

# Syntax für Tasten senden

## Mit einem Kürzel beginnen

Fügen Sie in **Tasten senden** `^c` ein, um unter Windows Strg+C zu senden, oder `#c` für Command+C auf dem Mac. Sie können auch auf **Aufnehmen** klicken, die Kombination drücken und auf **Fertig** klicken. Lua verwendet dieselbe Syntax über `ms.keys(sequence)`.

Zeichen bedeuten normalerweise Tasten an US-Tastaturpositionen. `abc` drückt A, B, C nacheinander; Großschreibung `ABC` fügt kein Shift hinzu. Verwenden Sie `+a` für Shift+A. Für Unicode oder festen Text siehe [Text und Escaping](#text-und-escaping) und [Zwischenablagen-Rezepte](lua-examples.md).

## Modifikatoren und Gruppen

| Präfix | Windows | Mac |
|---|---|---|
| `^` | Strg | Control ⌃ |
| `%` | Alt | Option ⌥ |
| `+` | Umschalt | Shift ⇧ |
| `#` | Win | Command ⌘ |

Ein Präfix wirkt auf die nächste Taste oder die geklammerte Gruppe. `^+t` sendet Strg+Umschalt+T. `^(ac)` wendet Strg auf A und C an. `^ac` wendet Strg nur auf A an. Gruppen können verschachtelt werden; verwenden Sie paarige Klammern. Gruppen-Modifikatoren hängen an jeder Tastenaktion; verlassen Sie sich nicht darauf, dass der Modifikator zwischen den Tasten durchgehend gehalten bleibt. Ein bloßer Modifikator am Ende ist kein Tastendruck: Verwenden Sie zum Beispiel `{CTRL}`, wenn Sie die Modifikatortaste selbst brauchen.

Lua `ms.is_keys_pressed` prüft auf beiden Plattformen die Vereinigung aller Tasten und Modifikatoren zum Abfragezeitpunkt: `^ac` erfordert Control, A und C gleichzeitig gedrückt. Jede Seite erfüllt ein Präfix; explizite `{LCTRL}` / `{RCTRL}`-Namen behalten ihre Seite. Wiederholungen werden ignoriert; Sleep-, Text-, Maus- oder Rad-Schritte liefern false. Eine Abfolge früherer Tastendrücke wird nicht erkannt.

## Benannte Tasten, Wiederholungen und Verzögerungen

Verwenden Sie `{ENTER}` für die Eingabetaste. Namen sind unabhängig von der Groß-/Kleinschreibung. `{LEFT 3}` drückt die linke Pfeiltaste dreimal. Eine positive Anzahl wiederholt eine Taste; null oder ein negativer Wert fällt auf eins zurück. `{SLEEP 100}` wartet 100 ms; es erfordert eine nichtnegative ganze Zahl. Trennen Sie einen Namen und sein Argument mit einem Leerzeichen.

Beispiel: `^c{SLEEP 100}^v` kopiert, wartet kurz und fügt unter Windows ein. Das Mac-Äquivalent ist `#c{SLEEP 100}#v`.

Die Tabelle unten führt alle vom Parser akzeptierten Namen auf, einschließlich Aliasen und aller nummerierten F-/Zehnertastatur-Namen. Auch ein einzelner ASCII-Buchstabe, eine Ziffer oder ein unterstütztes US-Satzzeichen kann in Klammern erscheinen, etwa `{a}` oder `{+}`.

| Namen | Bedeutung |
|---|---|
| `{ENTER}`, `{RETURN}` | Eingabetaste |
| `{ESC}` | Esc |
| `{TAB}` | Tabulator |
| `{BACKSPACE}` | Rücktaste |
| `{SPACE}` | Leertaste |
| `{CAPSLOCK}` | Feststelltaste |
| `{INSERT}`, `{INS}` | Einfg (nur Windows) |
| `{DELETE}`, `{DEL}` | Entf / Vorwärts-Entf |
| `{HOME}`, `{END}` | Pos1 / Ende |
| `{PGUP}`, `{PGDN}` | Bild auf / Bild ab |
| `{UP}`, `{DOWN}`, `{LEFT}`, `{RIGHT}` | Pfeiltasten |
| `{PRTSC}` | Druck (nur Windows) |
| `{BREAK}` | Untbr (nur Windows) |
| `{MEDIA_NEXT}`, `{MEDIA_PREV}`, `{MEDIA_STOP}`, `{MEDIA_PLAY}`, `{MEDIA_PLAY_PAUSE}` | Medientasten (nur Windows in Tastenfolgen) |
| `{VOLUME_MUTE}`, `{VOLUME_DOWN}`, `{VOLUME_UP}` | Lautstärketasten (nur Windows in Tastenfolgen) |
| `{CTRL}`, `{ALT}`, `{SHIFT}` | Strg / Alt (auf dem Mac Option) / Umschalt |
| `{LCTRL}`, `{RCTRL}`, `{LSHIFT}`, `{RSHIFT}`, `{LALT}`, `{RALT}` | Modifikatortasten links/rechts; Alt ist auf dem Mac Option |
| `{LWIN}`, `{WIN}`, `{RWIN}` | Windows-Taste; auf dem Mac Command |
| `{F1}`, `{F2}`, `{F3}`, `{F4}`, `{F5}`, `{F6}`, `{F7}`, `{F8}`, `{F9}`, `{F10}`, `{F11}`, `{F12}`, `{F13}`, `{F14}`, `{F15}`, `{F16}`, `{F17}`, `{F18}`, `{F19}`, `{F20}`, `{F21}`, `{F22}`, `{F23}`, `{F24}` | Funktionstasten; der Mac unterstützt F1–F20 |
| `{N0}`, `{N1}`, `{N2}`, `{N3}`, `{N4}`, `{N5}`, `{N6}`, `{N7}`, `{N8}`, `{N9}` | Ziffern 0–9 der Zehnertastatur |
| `{MOUSE_LEFT}`, `{MOUSE_RIGHT}`, `{MOUSE_MIDDLE}`, `{MOUSE_X1}`, `{MOUSE_X2}` | Mausklicks (nur Windows) |
| `{MOUSE_WHEEL}`, `{MOUSE_HWHEEL}` | Rad: z. B. {MOUSE_WHEEL 120}, {MOUSE_HWHEEL -120} (nur Windows) |
| `{SLEEP}` | Verzögerung: z. B. {SLEEP 100}, Millisekunden |

## Maustasten und Rad

Windows unterstützt `{MOUSE_LEFT}`, `{MOUSE_RIGHT}`, `{MOUSE_MIDDLE}`, `{MOUSE_X1}` und `{MOUSE_X2}` als Klicks. `{MOUSE_LEFT 2}` sendet zwei Klicks; der Zeiger wird dabei nicht verschoben.

`{MOUSE_WHEEL 120}` scrollt eine Raste nach oben; `-120` scrollt nach unten. `{MOUSE_HWHEEL 120}` scrollt nach rechts; `-120` nach links. Das Argument ist ein vorzeichenbehaftetes Rad-Delta, keine Wiederholungsanzahl. Das ist die Konvention der injizierten Eingabe; sie unterscheidet sich vom Vorzeichen der aufgezeichneten Rad-Schritte von Gesten.

Der Mac parst diese Zeichen, überspringt Maus- und Rad-Aktionen aber bei der Ausführung der Eingabe. Sie lassen sich nicht verwenden, um Mac-Klicks oder Scrollen über `ms.keys` zu erzeugen.

## Text und Escaping

Der Text-zuerst-Modus ist nur mit Lua `ms.keys(s, true)` verfügbar; der Editor **Tasten senden** verwendet immer den Tastenmodus. Gewöhnliche Zeichen werden zu Unicode-Text, aber `^`, `%`, `+`, `#`, Klammern und die öffnende geschweifte Klammer starten weiterhin die Syntax. Eingeklammerte Einzelzeichen bleiben im Text-zuerst-Modus Tastenaktionen.

Für Kürzel innerhalb des Text-zuerst-Modus klammern Sie die Taste: `^{c}` sendet Strg+C und `#{c}` sendet Command+C. Ein ungeklammertes `^c` gibt in diesem Modus den Text `c` aus und verwirft den Modifikator. Sie können auch einen separaten `ms.keys("^c")`-Aufruf im normalen Tastenmodus verwenden.

| Literalzeichen | Schreibweise in der Folge |
|---|---|
| `^` | `{^}` |
| `%` | `{%}` |
| `+` | `{+}` |
| `#` | `{#}` |
| `(` | `{(}` |
| `)` | `{)}` |
| `{` | `{{}` |
| `}` | `}` (ungeklammert) |

Der Backslash ist kein Escape-Zeichen der Tastenfolge. In einer zitierten Lua-Zeichenkette schreiben Sie `"\\"` für einen Backslash oder verwenden eine Long-Bracket-Zeichenkette. Für beliebigen Text mit Syntaxzeichen vermeidet die [Zwischenablagen-Methode](lua-examples.md#zwischenablagentext-einfügen-und-wiederherstellen) beide Ebenen des Escapings.

## Häufige Kürzel

Diese hängen von den Kürzeleinstellungen der Zielanwendung und des Betriebssystems ab.

| Zweck | Windows-Folge | Mac-Folge |
|---|---|---|
| Kopieren | `^c` | `#c` |
| Einfügen | `^v` | `#v` |
| Ausschneiden | `^x` | `#x` |
| Alles auswählen | `^a` | `#a` |
| Rückgängig | `^z` | `#z` |
| Neuer Tab | `^t` | `#t` |
| Tab schließen | `^w` | `#w` |
| Geschlossenen Tab wieder öffnen | `^+t` | `#+t` |
| Nächster Tab | `^{TAB}` | `^{TAB}` |
| Vorheriger Tab | `^+{TAB}` | `^+{TAB}` |
| App wechseln | `%{TAB}` | `#{TAB}` |
| Vorheriger/nächster Desktop | `#^{LEFT}` / `#^{RIGHT}` | `^{LEFT}` / `^{RIGHT}` |
| Screenshot-Bereich | `#+s` | `#+4` |
| Desktop anzeigen | `#d` | `{F11}` |
| Task-Manager / Activity Monitor | `^+{ESC}` | App öffnen verwenden: `com.apple.ActivityMonitor` |
| Bildschirm sperren | `#l` | `^#q` |

## Warum Strg+Alt+Entf nicht gesendet werden kann

Strg+Alt+Entf ist die Secure-Attention-Sequence von Windows. Windows ignoriert gewöhnliche synthetische Eingaben für diese Folge, daher kann `^%{DEL}` diesen Sicherheitsbildschirm nicht öffnen. Verwenden Sie `^+{ESC}` für den Task-Manager, `#l` zum Sperren oder eine Aktion **Befehl ausführen** mit `shutdown /l` zum Abmelden (sie beendet die Sitzung).

## Grenzen und Plattformhinweise

Der gemeinsame Parser hat keine Obergrenze für die Zeichenkettenlänge oder die Wiederholungsanzahl. Wiederholungsanzahlen und Verzögerungen werden als vorzeichenlose 32-Bit-Werte gespeichert; Rad-Deltas als vorzeichenbehaftete 32-Bit-Werte. Werte außerhalb des Bereichs können beim Umwandeln überlaufen; verwenden Sie kleine, praktikable Anzahlen und Verzögerungen. Ein falscher Tastenname oder ein fehlendes erforderliches Argument erzeugt einen Einlesefehler; der Editor prüft bereits während der Eingabe.

Windows führt Folgen auf Worker-Threads aus und wartet für `{SLEEP ms}`. Der Mac sendet Eingaben in timergesteuerten Stapeln. Mac-Aktionszeichenketten über 16.384 Byte werden abgelehnt; für bereits geparste Lua-Eingaben gelten 16.384 Aktionen und 16.384 Byte pro Textaktion. Seine Warteschlange fasst bis zu 64 Einträge; überschüssige Eingaben werden verworfen und protokolliert.

Auf dem Mac sind F1–F20, Ziffern der Zehnertastatur und zugeordnete Navigationstasten verfügbar. Insert/INS, PRTSC, BREAK, F21–F24, Medien-/Lautstärke-Zeichen und Maus-/Rad-Zeichen werden nicht unterstützt und übersprungen. Verwenden Sie **Medien und System** für Medien und Lautstärke. `ms.keys_down` und `ms.keys_up` gibt es nur unter Windows; siehe [Lua-API](lua-api.md).
