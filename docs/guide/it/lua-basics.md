---
title: "Nozioni di base su Lua"
description: "Scrivi il tuo primo script Lua per i gesti del mouse, impara gli argomenti di ms_action, esegui il debug con i log e comprendi i limiti del runtime."
order: 8
---

# Nozioni di base su Lua

## Quando usare Lua

Usa un'[azione semplice](actions.md) per una scorciatoia, un'operazione sulla finestra, un URL, un'applicazione o un comando. Scegli Lua quando un gesto deve distinguere in base all'applicazione, trasformare testo o combinare operazioni. Mousetroke integra **Lua 5.4**.

## Il primo script

1. Crea o seleziona un gesto nelle impostazioni.
2. Sotto **Azione**, scegli **Script Lua**. Sostituisci l'intera area di testo con il codice seguente.
3. Attendi l'indicazione di salvataggio, poi attivalo fuori dalle impostazioni.

```lua
function ms_action(pos, wid, pid, exe, title, mode)
  ms.log("Target: %s, mode: %s", exe, mode)
  ms.show_toast("My first Lua gesture", 1500)
end
```

`function` inizia una funzione e `end` la conclude. Mousetroke carica il codice, poi chiama `ms_action` una volta. Metti il tuo lavoro dentro quella funzione. I nomi tra parentesi ricevono il contesto; puoi omettere gli argomenti inutilizzati. `ms.log` scrive nel log di runtime e `ms.show_toast` mostra un messaggio per 1500 millisecondi.

Il pulsante **Introduzione a Lua** dell'editor apre questa pagina. Segui le [ricette Lua](lua-examples.md) per far digitare un testo a un gesto del mouse o incollare tramite gli appunti.

## Argomenti di ingresso

La firma completa è `ms_action(pos, wid, pid, exe, title, mode)`:

| Argomento | Tipo Lua | Significato |
|---|---|---|
| `pos` | table `{x=integer, y=integer}` | Ultima posizione del cursore registrata quando l'azione viene inviata. Le coordinate possono essere negative su un altro monitor. |
| `wid` | integer | Identificatore della finestra bersaglio; HWND su Windows, window ID su Mac. Trattalo come identificatore opaco; può valere 0 senza bersaglio. |
| `pid` | integer | ID del processo bersaglio, oppure 0. |
| `exe` | string | Identità dell'eseguibile su Windows, di norma un percorso completo; ID bundle o percorso dell'eseguibile su Mac. Vuota se non disponibile. |
| `title` | string | Titolo della finestra bersaglio, oppure stringa vuota. Su Mac il titolo viene letto tramite Accessibilità sul worker. |
| `mode` | string | Esattamente `UnderCursor` o `Focused`, da **Preferenze → I gesti agiscono su**. Non è il pulsante di attivazione né il nome del gesto. |

Il bersaglio viene catturato per il gesto. `pos` può differire dalle coordinate d'inizio del gesto delle variabili d'azione. L'input della tastiera va comunque all'app attiva se non attivi il bersaglio. Su Mac, `ms.focus_win(wid)` può attivare solo l'app bersaglio di questo gesto. Il testo segnaposto come `%MS_PID%` non viene espanso in Lua; usa direttamente `pid`.

## Lua in cinque minuti

`local` crea una variabile. Le stringhe usano virgolette singole o doppie; `..` concatena stringhe. `--` inizia un commento. `if`, `elseif`, `else`, `end` scelgono un ramo. `nil` significa nessun valore; solo `false` e `nil` sono falsi nelle condizioni (0 e `""` sono veri).

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

Qui `string.find(s, needle, 1, true)` cerca letteralmente e restituisce una posizione oppure `nil`. Il `true` evita di interpretare la punteggiatura come pattern. `string.match` usa i pattern Lua, che differiscono dalle espressioni regolari. Una table può contenere campi con nome o una lista, il cui primo indice è 1. Una funzione di supporto può restituire un valore:

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

Le stringhe Lua interpretano le sequenze di escape prima dell'analisi della sequenza di tasti. `"\\"` rappresenta una barra rovesciata e `"\n"` un a capo. Le stringhe con parentesi quadre lunghe come `[[text]]` conservano le barre rovesciate. La [guida all'invio dei tasti](key-sequences.md) spiega il secondo livello di sintassi.

## Debug

Usa `ms.log("pid=%d title=%s", pid, title)` e consulta **Informazioni → Apri registro**:

- Windows: `%APPDATA%\mousetroke\mousetroke.log`.
- Mac: `~/Library/Logs/mousetroke/mousetroke.log`.

Gli errori di caricamento e di esecuzione compaiono lì con `gesture_script` come nome del chunk. `ms.show_toast("Reached this step")` è un rapido controllo visivo. I risultati facoltativi delle API possono essere `nil`; verificali prima di usarli in aritmetica o come table. Vedi il [riferimento API](lua-api.md) per le chiamate non supportate su Mac.

## Runtime e limiti

Ogni esecuzione di script crea una nuova VM Lua su un thread di lavoro, con un **limite di allocazione Lua di 8 MiB**. Le variabili globali non persistono tra i gesti. Il runtime non imposta timeout per numero di istruzioni o tempo trascorso. Evita i cicli infiniti. `ms.sleep(ms)` blocca il worker di quello script, non l'hook di input né il thread dell'interfaccia principale. `ms.keys` attende che la sua sequenza finisca; su Mac l'input avanza tramite il timer del thread principale mentre il worker Lua attende.

Le librerie standard disponibili sono le funzioni base, `coroutine`, `table`, `io`, `os`, `string`, `utf8`, `math` e `package`. `debug` non viene caricato; il caricamento di moduli C nativi è disattivato dalla modalità sicura di mlua. `os.date` è disponibile. Non si tratta di una sandbox di sicurezza del sistema operativo: `io`, `os` e le API di processo possono accedere a file o eseguire comandi con i tuoi diritti utente. Esegui solo script e configurazioni di cui ti fidi.

Successivo: [riferimento API](lua-api.md), [ricette copia-e-incolla](lua-examples.md), [risoluzione dei problemi](troubleshooting.md).
