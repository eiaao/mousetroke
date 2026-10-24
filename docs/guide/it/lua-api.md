---
title: "API Lua"
description: "Riferimento completo dell'API ms per Lua 5.4, con firme, valori restituiti, esempi e note sul supporto Windows/macOS."
order: 9
---

# API Lua

Queste sono tutte le funzioni registrate su `ms` dal runtime condiviso. Gli esempi sono istruzioni da collocare dentro `ms_action`, dove `pos`, `wid`, `pid`, `exe`, `title` e `mode` sono disponibili. Vedi [Nozioni di base su Lua](lua-basics.md) per uno script completo e le [ricette](lua-examples.md) per azioni pronte all'uso.

## Convenzioni

Versione Lua: **5.4** (`mlua` con `lua54` e `vendored`). `nil` significa nessun valore; `?` contrassegna un parametro facoltativo. Le funzioni che non restituiscono valori sono mostrate come `-> nil`. Un argomento facoltativo omesso equivale a nil. Passa un booleano reale per gli argomenti booleani. I parametri interi troncano le frazioni finite verso lo zero e rifiutano i valori fuori dall'intervallo del tipo intero sottostante; usa interi negli script.

Le table di posizione usano campi con nome `{x=..., y=...}`, non elementi di array. Le chiamate non supportate su Mac di norma producono una riga di log per funzione per processo; i getter restituiscono nil e i setter non fanno nulla. Un setter senza valore restituito non è un controllo di successo. Le stringhe delle API Lua non ricevono la sostituzione delle variabili d'azione.

## Flusso e feedback

### `ms.log`

`ms.log(...) -> nil`

Accetta qualunque valore Lua. Con una stringa di formato e altri argomenti supporta `%s`, `%d`, `%f`, `%q` e `%%`; altrimenti unisce i valori con spazi. `%q` non aggiunge le virgolette Lua. Scrive una riga `Lua:` nel log di runtime su entrambe le piattaforme. Non restituisce nulla.

```lua
ms.log("pid=%d app=%s", pid, exe)
```

### `ms.sleep`

`ms.sleep(milliseconds: integer) -> nil`

Attende un numero non negativo di millisecondi (intero senza segno a 64 bit). Blocca il worker di questo script su entrambe le piattaforme; il motore dei gesti/thread principale continua. Nessun valore restituito e nessun timeout di script installato.

```lua
ms.sleep(100)
```

### `ms.is_paused`

`ms.is_paused() -> boolean`

Nessun parametro. Restituisce lo stato di pausa manuale del motore su entrambe le piattaforme. Non riporta la soppressione automatica a schermo intero né un'esclusione di app.

```lua
ms.log(ms.is_paused())
```

### `ms.set_paused`

`ms.set_paused(paused: boolean) -> nil`

`true` mette in pausa; `false` riprende. Nessun valore restituito. Supportato su entrambe le piattaforme. Windows invia un comando al thread dell'area di notifica, quindi un `is_paused()` immediato può ancora leggere il valore precedente. Mac applica tramite il bridge del thread principale.

```lua
ms.set_paused(true)
```

### `ms.show_settings`

`ms.show_settings() -> nil`

Nessun parametro né valore restituito. Apre le impostazioni o le porta in primo piano su entrambe le piattaforme.

```lua
ms.show_settings()
```

### `ms.show_toast`

`ms.show_toast(message: string, duration_ms?: integer) -> nil`

`message` è il testo mostrato. La durata facoltativa non negativa è in millisecondi, predefinita **1200**. Mostra un messaggio overlay su entrambe le piattaforme; non restituisce nulla.

```lua
ms.show_toast("Done", 1500)
```

## Input da tastiera

### `ms.keys`

`ms.keys(sequence: string, text_by_default?: boolean) -> nil`

Analizza e invia la [sequenza di tasti](key-sequences.md); la modalità testo è predefinita a `false`. Restituisce dopo l'invio; una sintassi malformata genera un errore Lua. Entrambe le piattaforme supportano l'input ordinario di tastiera/testo. Mac salta i tasti non supportati, le azioni mouse e rotella, e applica i propri limiti di coda/testo. Attende la propria coda di input del thread principale, che può contenere anche altre sequenze.

```lua
ms.keys("Hello", true)
```

### `ms.keys_down`

`ms.keys_down(sequence: string) -> nil`

Solo Windows: analizza in modalità tasti e preme tasti/modificatori senza rilasciarli. Le ripetizioni non vengono ripetute in questa fase; le azioni rotella scattano una volta. Una sintassi non valida genera un errore Lua. Abbinala a `keys_up` per rilasciare. Su Mac registra il log non supportato e non fa nulla. Nessun valore restituito.

```lua
ms.keys_down("^a")
ms.sleep(50)
ms.keys_up("^a")
```

### `ms.keys_up`

`ms.keys_up(sequence: string) -> nil`

Solo Windows: rilascia la sequenza analizzata in ordine d'azione inverso, con i modificatori rilasciati per ultimi. I passaggi sleep attendono comunque. Gli errori di sintassi generano un errore Lua. Su Mac registra il log non supportato e non fa nulla. Nessun valore restituito.

```lua
ms.keys_down("{SHIFT}")
ms.keys_up("{SHIFT}")
```

### `ms.is_keys_pressed`

`ms.is_keys_pressed(sequence: string) -> boolean | nil`

Entrambe le piattaforme controllano ogni tasto della tastiera **insieme ai modificatori del suo prefisso**: `"^a"` richiede Control e A. Ciascun lato soddisfa un prefisso; i nomi espliciti come `{RCTRL}` richiedono quel lato. Su Mac `^` = Control, `%` = Option, `+` = Shift e `#` = Command. Più passaggi significano che tutti i tasti/modificatori elencati devono essere premuti insieme, non che siano stati premuti in ordine; le ripetizioni non aggiungono requisiti. Le azioni mouse/rotella, sleep o testo rendono la verifica falsa; una sequenza vuota restituisce true. Una sintassi non valida restituisce false. Su Mac legge lo stato combinato della sessione e restituisce false senza l'autorizzazione Accessibilità, registrando il log una volta.

```lua
if ms.is_keys_pressed("^a") then
  ms.show_toast("Ctrl and A are down")
end
```

## Appunti e file

### `ms.get_clipboard_text`

`ms.get_clipboard_text() -> string | nil`

Nessun parametro. Restituisce il testo degli appunti, oppure `nil` se il testo non è disponibile. Supportato su entrambe le piattaforme; non recupera automaticamente la selezione.

```lua
local text = ms.get_clipboard_text()
ms.log(text or "No text")
```

### `ms.set_clipboard_text`

`ms.set_clipboard_text(text: string) -> nil`

Sostituisce il contenuto degli appunti con il testo su entrambe le piattaforme; nessun valore restituito né indicazione di successo. Gli altri formati degli appunti non vengono preservati. Vedi le [ricette con gli appunti](lua-examples.md).

```lua
ms.set_clipboard_text("Hello from Mousetroke")
```

### `ms.read_text`

`ms.read_text(path: string) -> string | nil`

Legge l'intero file come UTF-8 su entrambe le piattaforme. Restituisce `nil` in caso di errore di lettura o di decodifica UTF-8. I percorsi relativi usano la cartella di lavoro del processo, non una cartella documentata per gli script. Le letture concorrono al limite di memoria disponibile.

```lua
local text = ms.read_text("config.json")
ms.log(text and #text or 0)
```

## Finestre

### `ms.find_win_by_class`

`ms.find_win_by_class(pattern: string) -> table | nil`

Cerca le finestre visibili di primo livello con una **espressione regolare Rust**, su Windows sui nomi di classe, su Mac sugli **ID bundle** delle applicazioni (per esempio `^com\\.apple\\.Safari$` in una stringa Lua). Restituisce fino a 32 voci in un array con base 1: `{text=title, hwnd=window_id}`; nessuna corrispondenza o regex non valida dà una table vuota. La corrispondenza distingue maiuscole e minuscole salvo che il pattern usi un flag come `(?i)`; non si tratta di pattern Lua. Su Mac legge i titoli tramite Accessibilità, restituendo nil senza autorizzazione e registrando il log una volta. Il suo `hwnd` è un CGWindowID accettato da `focus_win` e dalle API sullo stato delle finestre. Su Mac si cercano solo le finestre attualmente a schermo.

```lua
local wins = ms.find_win_by_class("^Notepad$")
if wins and wins[1] then ms.focus_win(wins[1].hwnd) end
```

### `ms.find_win_by_title`

`ms.find_win_by_title(pattern: string) -> table | nil`

Cerca i titoli delle finestre su entrambe le piattaforme con un'espressione regolare Rust, restituendo le stesse voci `{text, hwnd}` con base 1 di `find_win_by_class`, al massimo 32. Una regex non valida dà una table vuota. Su Mac usa i titoli AX delle finestre, richiede l'autorizzazione Accessibilità e cerca solo le finestre attualmente a schermo.

```lua
local wins = ms.find_win_by_title("(?i)notepad")
if wins then ms.log(#wins) end
```

### `ms.focus_win`

`ms.focus_win(wid: integer) -> nil`

Attiva una finestra/app; nessun valore restituito né indicazione di successo. Windows accetta un HWND. Mac accetta il bersaglio del gesto o un CGWindowID scoperto, ripristina dalla riduzione a icona e alza la sua finestra AX, poi attiva l'app proprietaria tramite il bridge del thread principale. Su Mac le chiamate AX girano sul worker dello script, riutilizzano la ricerca finestre integrata per PID/limiti e richiedono l'autorizzazione Accessibilità; ID non più validi o finestre AX non disponibili vengono ignorati. App con limiti di finestre identici e sovrapposti possono essere ambigue.

```lua
if wid ~= 0 then ms.focus_win(wid) end
```

### `ms.set_win_state`

`ms.set_win_state(wid: integer, state: string) -> nil`

Windows: `"min"` riduce a icona, `"max"` ingrandisce; **ogni altra stringa ripristina**, incluse `"normal"` e `"norm"`. Mac: `"min"` usa AXMinimized; `"max"` ripristina dalla riduzione a icona/esce dallo schermo intero e preme lo stesso AXZoomButton dell'azione integrata quando la finestra non riempie l'area di lavoro del proprio schermo. `"normal"`, `"norm"` e `"restore"` ripristinano dalla riduzione a icona/escono dallo schermo intero e annullano lo zoom sull'area di lavoro. Mac accetta anche `"fullscreen"` tramite AXFullScreen dove l'app lo supporta; ogni altro nome viene ignorato e registrato una volta. I valori distinguono maiuscole e minuscole. Nessun valore restituito. Mac richiede l'autorizzazione Accessibilità; AX gira sul worker e la geometria dello schermo passa attraverso il bridge del thread principale. Le transizioni possono essere asincrone.

Lo Zoom su Mac è definito dall'app: uno zoom alle dimensioni del contenuto può comunque risultare `"normal"` e non si può ripristinare in modo affidabile con questa API; richieste `"max"` ripetute possono attivarlo e disattivarlo. Per queste app usa l'interruttore integrato **Ingrandisci / ripristina**. Non esiste un flag AX pubblico universale per lo stato di zoom.

```lua
if wid ~= 0 then ms.set_win_state(wid, "max") end
```

### `ms.get_win_state`

`ms.get_win_state(wid: integer) -> string | nil`

Windows restituisce `"min"`, `"max"` o `"normal"`. Non è una query always-on-top e non valida un handle non più valido; un handle non valido può risultare normal. Mac restituisce `"min"`, `"fullscreen"`, `"max"` (il riquadro corrisponde a un'area di lavoro dello schermo entro 2 punti logici) oppure `"normal"`. Mac non ha un getter AX pubblico universale per lo stato di zoom, quindi uno zoom alle dimensioni del contenuto risulta normal. Restituisce nil per autorizzazione mancante, finestra/stato AX non disponibile o ID non più valido; il rifiuto dell'autorizzazione viene registrato una volta.

```lua
if wid ~= 0 then ms.log(ms.get_win_state(wid)) end
```

### `ms.set_win_top`

`ms.set_win_top(wid: integer, top: boolean) -> nil`

Su Windows imposta (`true`) o rimuove (`false`) lo stato sempre in primo piano senza spostare/ridimensionare la finestra; nessun valore restituito. Imposta uno stato, non lo alterna né restituisce lo stato precedente. Su Mac non fa nulla e registra il log una volta: macOS non ha un'API pubblica per tenere in primo piano la finestra di un'altra app, e anche l'azione integrata always-on-top non è supportata. Per un interruttore, scegli **Azioni sulla finestra → Sempre in primo piano / annulla** nell'editor Windows.

```lua
if wid ~= 0 then ms.set_win_top(wid, true) end
```

## Mouse e schermo

### `ms.get_mouse_pos`

`ms.get_mouse_pos() -> table`

Nessun parametro. Entrambe le piattaforme restituiscono `{x=integer, y=integer}` per la posizione attuale del cursore, che può differire dal `pos` di ingresso. Windows usa pixel dello schermo; Mac usa punti logici globali di Quartz, con Y crescente verso il basso. Mac restituisce `{x=0,y=0}` se la query del cursore fallisce.

```lua
local p = ms.get_mouse_pos()
ms.log("%d,%d", p.x, p.y)
```

### `ms.set_mouse_pos`

`ms.set_mouse_pos(x: integer, y: integer) -> nil`

Sposta il puntatore alle coordinate globali dello schermo su entrambe le piattaforme; nessun valore restituito. Le coordinate possono essere negative. Mac invia un evento di movimento contrassegnato, così non avvia un altro gesto.

```lua
local p = ms.get_mouse_pos()
ms.set_mouse_pos(p.x + 20, p.y)
```

### `ms.get_screen`

`ms.get_screen(x: integer, y: integer) -> table | nil`

Restituisce il rettangolo completo dello schermo `{left, top, right, bottom}` come coordinate globali intere, non l'area di lavoro utilizzabile. La larghezza è `right-left`; l'altezza `bottom-top`. Windows sceglie il monitor più vicino se il punto è fuori da tutti i monitor. Mac sceglie il display contenente, oppure ricade sul primo display in cache; può restituire `nil` se non è disponibile alcun risultato schermo/bridge. Entrambe le piattaforme supportano questa chiamata.

```lua
local p = ms.get_mouse_pos()
local s = ms.get_screen(p.x, p.y)
if s then ms.log(s.right - s.left, s.bottom - s.top) end
```

### `ms.get_brightness`

`ms.get_brightness(pos?: table) -> integer | nil`

La `{x=integer,y=integer}` facoltativa sceglie lo schermo; omessa/nil o una posizione incompleta ricade sul cursore attuale. Restituisce **0–100 percento in caso di successo, nil in caso di fallimento** su entrambe le piattaforme. Windows usa DDC/CI con fallback WMI. WMI usa il suo primo dispositivo di luminosità, non un monitor corrispondente al cursore. Mac attualmente restituisce nil e registra il log una volta: il suo backend di luminosità esistente invia solo eventi dei tasti di luminosità del display integrato, che non possono leggere una percentuale. DDC esterno non è supportato. Solo il wrapper legacy Windows `WG.GetBrightness` mantiene -1 in caso di fallimento.

```lua
local value = ms.get_brightness()
if value ~= nil then ms.show_toast(tostring(value)) end
```

### `ms.set_brightness`

`ms.set_brightness(pos: table | nil, delta: integer) -> integer | nil`

Nonostante il nome, regola di `delta` punti percentuali; non imposta una percentuale assoluta. Passa una table con nome `{x=...,y=...}` oppure **nil come primo argomento** per usare il cursore. In caso di successo restituisce la nuova **percentuale 0–100**, limitata a quell'intervallo; in caso di fallimento restituisce **nil su entrambe le piattaforme**. Windows usa la stessa selezione DDC/CI e WMI di `get_brightness`. Solo il wrapper legacy Windows `WG.SetBrightness` mantiene -1 in caso di fallimento. Mac restituisce nil e registra il log una volta: il backend Media e sistema esistente invia solo i tasti di luminosità del display integrato, senza lettura numerica né controllo a punti percentuali. Questa chiamata Lua non modifica la luminosità; per i tasti di luminosità usa quelle azioni integrate. DDC esterno resta non supportato.

```lua
local value = ms.set_brightness(nil, 5)
ms.log(value)
```

## URL e processi

### `ms.open`

`ms.open(url: string) -> nil`

Apre un URL con il gestore predefinito; nessun valore restituito né indicazione di successo. Windows usa shell open e accetta anche un percorso di file/cartella. Mac si aspetta un URL; per un file usa un URL `file://` codificato correttamente, oppure `ms.cmd` per invocare `/usr/bin/open` su un percorso. Non viene eseguita alcuna sostituzione di variabili.

```lua
ms.open("https://mousetroke.com/")
```

### `ms.run`

`ms.run(file: string, args?: string) -> nil`

Avvia con una **stringa** di argomenti facoltativa, non un array Lua; non restituisce valori né PID. Windows usa il lancio ordinario senza elevazione con comportamento RunAgain. Mac accetta un ID bundle, un percorso app o un percorso eseguibile assoluto; gli argomenti vengono suddivisi gestendo le virgolette. Il lancio dell'eseguibile può ricadere su un processo in background se LaunchServices lo rifiuta. Richiede una nuova istanza. Nessuna espansione di variabili; usa la concatenazione Lua. L'output del processo non viene restituito.

```lua
if package.config:sub(1, 1) == "\\" then
  ms.run("cmd.exe", "/K echo Hello")
else
  ms.run("/usr/bin/say", '"Hello from Mousetroke"')
end
```

### `ms.cmd`

`ms.cmd(command: string) -> nil`

Esegue codice shell tramite `cmd.exe /C` nascosto su Windows oppure `/bin/zsh -c` su Mac. Non restituisce valori, output né codice d'uscita e non attende il completamento del processo. Mac limita a 16 i processi in background gestiti contemporaneamente e registra il completamento. Le stringhe vengono eseguite così come fornite; i segnaposto `%MS_*%` non vengono espansi da questa API Lua. Quota tu gli argomenti della shell.

```lua
if package.config:sub(1, 1) == "\\" then
  ms.cmd("taskmgr.exe")
else
  ms.cmd('/usr/bin/open -a "Activity Monitor"')
end
```

Continua con la [sintassi di invio dei tasti](key-sequences.md), le [ricette](lua-examples.md) o le [posizioni dei log](config.md#posizioni-dei-file).
