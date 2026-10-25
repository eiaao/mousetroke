---
title: "Ricette Lua"
description: "Copia script Lua pronti all'uso per i gesti del mouse: testo, incollaggio dagli appunti, date, ricerca web, finestre, luminosità e pausa."
order: 10
---

# Ricette Lua

Incolla un blocco completo nell'azione **Script Lua** di un gesto. Ogni blocco definisce `ms_action`. Prova prima le ricette di testo in un documento sacrificabile. [Nozioni di base su Lua](lua-basics.md) spiega l'editor e [API Lua](lua-api.md) elenca il supporto delle piattaforme.

Le ricette multipiattaforma usano `package.config` per scegliere `^` (Control su Windows) o `#` (Command su Mac). Le ricette con gli appunti preservano **solo il testo**, non immagini o altri formati. Le loro attese di 200 ms sono esempi, non una garanzia per ogni applicazione; aumentale se la destinazione è lenta. Sostituiscono temporaneamente il contenuto degli appunti e ripristinano il testo salvato, anche se copi altro durante quell'intervallo.

## Digitare un testo fisso

Scopo: inserire una frase riutilizzabile con un solo gesto del mouse.

```lua
function ms_action()
  ms.keys("Hello from Mousetroke.", true)
end
```

La modalità testo supporta Unicode su entrambe le piattaforme. Continua però ad analizzare i simboli della sintassi; per la punteggiatura arbitraria come `%`, `+` o le graffe usa la ricetta con gli appunti.

## Incollare e ripristinare il testo degli appunti

Scopo: incollare un testo letterale, poi ripristinare il testo che era negli appunti.

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

Questo aggira l'escape delle sequenze di tasti per il testo inserito. Se negli appunti non c'era testo, alla fine contengono testo vuoto; gli altri formati originali non si possono ripristinare con queste API.

## Testo diverso per app

Scopo: usare l'identità dell'eseguibile bersaglio del gesto per scegliere una frase.

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

Le identità Windows sono di norma percorsi completi dell'eseguibile; Mac di norma fornisce un ID bundle. Questo controlla il bersaglio catturato, mentre la digitazione va all'app attiva. Usa prima `ms.focus_win(wid)` se vuoi attivare l'app bersaglio.

## Inserire la data di oggi

Scopo: inserire la data locale come anno-mese-giorno.

```lua
function ms_action()
  ms.keys(os.date("%Y-%m-%d"), true)
end
```

`os.date` è disponibile nel runtime. I caratteri `%` sono la sintassi Lua di formattazione delle date; le cifre e i trattini risultanti sono sicuri per l'input in modalità testo.

## Cercare il testo selezionato su un sito

Scopo: copiare la selezione e aprire una ricerca Google codificata in UTF-8.

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

Seleziona del testo nell'app attiva prima di attivare il gesto. Svuotare prima gli appunti evita di cercare il vecchio testo se Copia non fa nulla. Lua non ha sostituzione automatica del testo selezionato; la più semplice [azione Apri un URL](actions.md#aprire-un-url) può raccogliere la selezione per te.

## Racchiudere il testo selezionato tra virgolette o parentesi

Scopo: sostituire il testo selezionato con una versione tra virgolette in un campo modificabile.

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

Per le parentesi, sostituisci l'espressione di inclusione con `"[" .. text .. "]"`. Copia deve lasciare il testo selezionato, così Incolla lo sostituisce. Le pagine in sola lettura non si possono modificare in questo modo.

## Attivare/disattivare sempre in primo piano

Scopo: alternare lo stato sempre in primo piano di una finestra Windows. Scegli **Azioni sulla finestra → Sempre in primo piano / annulla** per questa ricetta: quell'azione può interrogare e alternare lo stato effettivo. L'API Lua imposta solo uno stato richiesto; non ha un getter per lo stato always-on-top e ogni gesto riceve una VM nuova. Mac non supporta il controllo always-on-top tra app.

Se vuoi un gesto Lua separato «tieni questa finestra in primo piano» su Windows, usa:

```lua
function ms_action(pos, wid)
  if package.config:sub(1, 1) ~= "\\" then
    ms.show_toast("Always-on-top is unavailable on Mac")
    return
  end
  if wid ~= 0 then ms.set_win_top(wid, true) end
end
```

Usa `false` invece di `true` per un gesto separato di rimozione. Un booleano globale Lua si azzererebbe a ogni esecuzione e non implementerebbe un interruttore.

## Ingrandire o ripristinare

Scopo: alternare lo stato ingrandito della finestra bersaglio su Windows o lo zoom sull'area di lavoro su Mac.

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

Su Mac si usa lo stesso pulsante zoom AX dell'azione finestra integrata. La query dello stato riconosce le finestre delle dimensioni dell'area di lavoro; le app che ingrandiscono alle dimensioni del contenuto possono comunque risultare normal, quindi per quelle app usa **Azioni sulla finestra → Ingrandisci / ripristina**. Un'autorizzazione Accessibilità mancante o una finestra non disponibile dà nil. Non esiste un'API Lua per posizione/dimensione delle finestre per spostare una finestra su un altro monitor.

## Luminosità sotto il cursore

Scopo: aumentare di 5 punti percentuali la luminosità dello schermo sotto il cursore su Windows.

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

Cambia `5` in `-5` per un secondo gesto che abbassa. Windows richiede hardware DDC/CI o WMI compatibile; WMI ricade sul suo primo dispositivo di luminosità, che può essere uno schermo diverso. Il successo è una percentuale 0–100; il fallimento è nil su entrambe le piattaforme. Su Mac qui si ottiene nil: il backend esistente invia solo tasti di luminosità e non può restituire una percentuale né applicare un delta a punti percentuali. Per i tasti di luminosità dello schermo integrato, senza selezione dello schermo sotto il cursore né DDC/CI esterno, usa **Media e sistema → Aumenta luminosità / Riduci luminosità**.

## Aprire una cartella

Scopo: aprire la cartella Download dell'utente attuale con il file browser della piattaforma.

```lua
function ms_action()
  if package.config:sub(1, 1) == "\\" then
    ms.cmd('explorer.exe "%USERPROFILE%\\Downloads"')
  else
    ms.cmd('/usr/bin/open "$HOME/Downloads"')
  end
end
```

Queste variabili d'ambiente vengono espanse dalla shell del sistema operativo, non dalle variabili d'azione di Mousetroke. Si assume la posizione consueta della cartella Download.

## Eseguire un programma con argomenti

Scopo: passare una stringa di argomenti a un programma.

```lua
function ms_action()
  if package.config:sub(1, 1) == "\\" then
    ms.run("cmd.exe", "/K echo Hello from Mousetroke")
  else
    ms.run("/usr/bin/say", '"Hello from Mousetroke"')
  end
end
```

Windows apre una finestra di comando che mostra un messaggio. Mac pronuncia la frase con `say`. `ms.run` accetta una singola stringa di argomenti, non una table, e non restituisce alcun handle di processo.

## Aprire Gestione attività o Monitoraggio attività

Scopo: aprire il visualizzatore di processi della piattaforma.

```lua
function ms_action()
  if package.config:sub(1, 1) == "\\" then
    ms.keys("^+{ESC}")
  else
    ms.run("com.apple.ActivityMonitor")
  end
end
```

Windows usa Ctrl+Shift+Esc. Mac avvia Monitoraggio attività tramite ID bundle.

## Bloccare lo schermo

Scopo: bloccare la sessione attuale con la sua scorciatoia di sistema.

```lua
function ms_action()
  if package.config:sub(1, 1) == "\\" then
    ms.keys("#l")
  else
    ms.keys("^#q")
  end
end
```

Mac usa Control+Command+Q. Windows usa Win+L; Ctrl+Alt+Del non si può sintetizzare come sequenza di tasti ordinaria.

## Mettere in pausa i gesti per dieci minuti

Scopo: mettere temporaneamente in pausa il motore, poi riprenderlo mentre Mousetroke resta in esecuzione.

```lua
function ms_action()
  ms.set_paused(true)
  ms.show_toast("Gestures paused for ten minutes", 2000)
  ms.sleep(10 * 60 * 1000)
  ms.set_paused(false)
  ms.show_toast("Gestures resumed", 1500)
end
```

Funziona su entrambe le piattaforme e tiene un worker Lua addormentato per dieci minuti. Non è un timer persistente: uscire o riavviare lo interrompe. Puoi riprendere manualmente durante l'attesa, ma lo script esegue comunque la ripresa quando la sua attesa finisce. Evita di avviare più copie. Vedi i [limiti del runtime](lua-basics.md#runtime-e-limiti).
