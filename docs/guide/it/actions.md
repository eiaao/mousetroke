---
title: "Azioni"
description: "Scegli invio tasti, azioni sulla finestra, app, URL, comandi, tasti multimediali, menu o Lua, e usa le variabili dei gesti."
order: 5
---

# Azioni

Seleziona un gesto e scegli la sua azione sotto **Azione**. L'editor offre otto tipi. La maggior parte delle attività richiede solo un'azione semplice; usa [Lua](lua-basics.md) quando servono condizioni o più operazioni.

## Inviare tasti

Inserisci una [sequenza di tasti](key-sequences.md), oppure clicca **Registra**, premi la scorciatoia e clicca **Fatto**. Per esempio, `^c` copia su Windows; `#c` copia su Mac. **Guida alla sintassi** apre la guida dedicata alla sintassi.

**Attiva prima la finestra sotto il cursore** attiva prima il bersaglio del gesto. Senza questa opzione, le scorciatoie vanno all'applicazione che riceve l'input della tastiera. Il bersaglio segue anche **Preferenze → I gesti agiscono su**. In questo editor non c'è un'opzione per il testo semplice; l'[input in modalità testo](key-sequences.md#testo-ed-escape) è disponibile tramite Lua.

## Azioni sulla finestra

Scegli un'operazione, applicata al bersaglio del gesto:

| Operazione | Comportamento |
|---|---|
| Riduci a icona | Riduce a icona la finestra. |
| Ingrandisci / ripristina | Alterna tra dimensione ingrandita e ripristinata. |
| Chiudi | Chiede alla finestra di chiudersi. |
| Sempre in primo piano / annulla | Alterna sempre in primo piano su Windows; non supportato e disattivato nell'editor Mac. |

Il controllo delle finestre su Mac usa Accessibilità. **Ingrandisci / ripristina** preme il pulsante Zoom dell'app, quindi la dimensione risultante segue il comportamento di quell'app; non richiede lo schermo intero nativo e salta le finestre già ridotte a icona. Le applicazioni possono esporre operazioni finestra diverse.

## Aprire un'app

**App** accetta un eseguibile su Windows; **Sfoglia…** lo seleziona. Su Mac usa un ID bundle come `com.apple.Safari`, un percorso `.app` oppure un percorso di eseguibile. **Argomenti** è facoltativo. Su Mac gli argomenti vengono suddivisi gestendo le virgolette; usa percorsi completi per gli eseguibili da riga di comando.

**Se già in esecuzione** offre **Passa ad essa** o **Aprine un'altra**. Windows offre anche **Esegui come amministratore**. Mac non supporta l'elevazione dei privilegi né le opzioni di avvio con filtro finestra. Nell'editor delle azioni attuale non ci sono campi classe/titolo/espressione regolare.

## Aprire un URL

Compila **URL** e, facoltativamente, **Browser**. Lascia vuoto Browser per il browser predefinito. Windows accetta un eseguibile di browser; Mac accetta un ID bundle o un percorso dell'app di browser. I pulsanti **Inserisci** aggiungono il testo selezionato, il testo degli appunti, il nome dell'app, l'ID processo o il titolo della finestra come segnaposto.

Per la ricerca web sul testo selezionato, imposta l'URL su:

```text
https://www.google.com/search?q=%MS_SELECTED_TEXT%
```

Seleziona del testo in un'applicazione, poi esegui il gesto. Il testo selezionato viene codificato per l'URL. Se inizia già con `http://`, `https://` o `www.`, Mousetroke apre direttamente quell'indirizzo; `www.` riceve il prefisso `https://`.

## Eseguire un comando

Inserisci il comando nell'area di testo. **Inserisci** aggiunge le variabili. Windows lo esegue tramite un `cmd.exe /C` nascosto; Mac usa `/bin/zsh -c`. Si tratta di sintassi shell, quindi quota correttamente gli argomenti. Il testo sostituito non viene protetto per la shell; evita di passare ai comandi testo selezionato non attendibile.

Per esempio, il seguente comando Windows apre Gestione attività:

```text
taskmgr.exe
```

Su Mac, usa **Aprire un'app** con `com.apple.ActivityMonitor`.

## Media e sistema

L'editor offre **Riproduci / pausa**, **Traccia successiva**, **Traccia precedente**, **Alza il volume**, **Abbassa il volume**, **Silenzia**, **Aumenta luminosità** e **Riduci luminosità**.

La luminosità su Windows si regola di 10 punti percentuali usando DDC/CI con fallback WMI. La luminosità su Mac invia i tasti di luminosità per lo schermo integrato; DDC/CI esterno non è supportato. Queste azioni di sistema su Mac sono disponibili anche se le funzioni Lua per la luminosità non sono supportate. Vedi [bordi dello schermo](screen-edges.md).

## Mostrare un menu

Clicca **+ Aggiungi una voce**, inserisci il **Nome della voce di menu**, poi scegli un'azione dentro ogni voce. Usa **Sposta su**, **Sposta giù** ed **Elimina la voce di menu** per organizzarlo. Il menu compare nella posizione del gesto. L'editor non espone un campo per il numero di colonne e non offre un altro tipo di menu dentro una voce. Mac supporta una sola colonna; le impostazioni multi-colonna vengono segnalate come non supportate.

## Script Lua

Incolla uno script completo che definisce `ms_action`. L'editor rimanda a [Introduzione a Lua](lua-basics.md), al [riferimento API](lua-api.md) e agli [esempi](lua-examples.md). L'area di testo accetta Tab come due spazi.

<a id="variables"></a>

## Variabili

Usa questi segnaposto in **Aprire un URL → URL**, **Eseguire un comando** e **Aprire un'app → App / Argomenti**. Non vengono espansi nelle stringhe di invio tasti, nel sorgente Lua, nei nomi delle voci di menu né in Browser.

| Variabile | Valore |
|---|---|
| `%MS_SELECTED_TEXT%` | Selezione catturata quando l'azione la richiede; vuota se non disponibile. |
| `%MS_CLIPBOARD_TEXT%` | Testo attuale degli appunti, catturato su richiesta. |
| `%MS_PID%` | ID del processo bersaglio. |
| `%MS_EXE%` | Identità dell'eseguibile bersaglio: percorso su Windows; ID bundle o percorso dell'eseguibile su Mac. |
| `%MS_APP_NAME%` | Parte del nome file dell'identità, che su Mac può essere un ID bundle. |
| `%MS_WID%` | Identificatore della finestra della piattaforma come testo decimale. |
| `%MS_WIN_NAME%` | Titolo della finestra bersaglio; su Mac viene letto tramite Accessibilità quando serve. |
| `%MS_MOUSE_X%`, `%MS_MOUSE_Y%` | Coordinate sullo schermo dell'inizio del gesto. |
| `%MS_MOUSE_Y_FLIP%` | Coordinata inferiore dello schermo virtuale meno la coordinata Y dell'inizio del gesto. |
| `%MS_TARGET_MODE%` | `UnderCursor` o `Focused`, secondo la preferenza del bersaglio. |

Le variabili del bersaglio non disponibili possono mancare; i segnaposto sconosciuti restano invariati. Solo il testo selezionato riceve la codifica URL automatica in un modello di URL. Le altre sostituzioni vengono inserite così come sono. Lua usa gli [argomenti di ingresso](lua-basics.md#argomenti-di-ingresso) e le funzioni di `ms` al posto dell'espansione dei segnaposto.
