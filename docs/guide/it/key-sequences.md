---
title: "Sintassi di invio dei tasti"
description: "Sintassi completa di invio dei tasti di Mousetroke: modificatori, gruppi, tasti con nome, ripetizioni, attese, input del mouse e scorciatoie Windows/Mac."
order: 7
---

# Sintassi di invio dei tasti

## Partire da una scorciatoia

In **Invia tasti**, incolla `^c` per inviare Ctrl+C su Windows, oppure `#c` per inviare Command+C su Mac. Puoi anche cliccare **Registra**, premere la combinazione e cliccare **Fatto**. Lua usa la stessa sintassi tramite `ms.keys(sequence)`.

I caratteri indicano di norma i tasti nelle posizioni della tastiera USA. `abc` preme A, B, C in sequenza; il maiuscolo `ABC` non aggiunge Shift. Usa `+a` per Shift+A. Per Unicode o testo fisso, vedi [testo ed escape](#testo-ed-escape) e le [ricette con gli appunti](lua-examples.md).

## Modificatori e gruppi

| Prefisso | Windows | Mac |
|---|---|---|
| `^` | Ctrl | Control ⌃ |
| `%` | Alt | Option ⌥ |
| `+` | Shift | Shift ⇧ |
| `#` | Win | Command ⌘ |

Un prefisso si applica al tasto successivo o al gruppo tra parentesi. `^+t` invia Ctrl+Shift+T. `^(ac)` applica Ctrl sia ad A sia a C. `^ac` applica Ctrl solo ad A. I gruppi possono annidarsi; usa parentesi bilanciate. I modificatori di gruppo si applicano a ogni azione di tasto, quindi non contare che il modificatore resti premuto senza interruzioni tra i tasti. Un modificatore finale da solo non è una pressione di tasto: usa ad esempio `{CTRL}` quando ti serve il tasto modificatore stesso.

Il comando Lua `ms.is_keys_pressed` controlla, su entrambe le piattaforme, l'unione di tutti i tasti e modificatori al momento della query: `^ac` richiede Control, A e C premuti insieme. Ciascun lato soddisfa un prefisso; i nomi espliciti `{LCTRL}` / `{RCTRL}` conservano il lato. Le ripetizioni vengono ignorate; i passaggi sleep, testo, mouse o rotella restituiscono false. Non riconosce una sequenza di pressioni passate.

## Tasti con nome, ripetizioni e attese

Usa `{ENTER}` per Invio. I nomi non distinguono maiuscole e minuscole. `{LEFT 3}` preme Left tre volte. Un conteggio positivo ripete un tasto; un conteggio zero o negativo vale come uno. `{SLEEP 100}` attende 100 ms; richiede un intero non negativo. Separa il nome e il suo argomento con uno spazio.

Esempio: `^c{SLEEP 100}^v` copia, attende brevemente, poi incolla su Windows. L'equivalente su Mac è `#c{SLEEP 100}#v`.

La tabella qui sotto elenca i nomi accettati dal parser, inclusi gli alias e ogni nome numerato F/tastierino. Anche una singola lettera ASCII, cifra o carattere di punteggiatura USA supportato può comparire tra parentesi graffe, come `{a}` o `{+}`.

| Nomi | Significato |
|---|---|
| `{ENTER}`, `{RETURN}` | Invio |
| `{ESC}` | Esc |
| `{TAB}` | Tab |
| `{BACKSPACE}` | Backspace |
| `{SPACE}` | Spazio |
| `{CAPSLOCK}` | Caps Lock |
| `{INSERT}`, `{INS}` | Insert (solo Windows) |
| `{DELETE}`, `{DEL}` | Delete / cancellazione in avanti |
| `{HOME}`, `{END}` | Home / End |
| `{PGUP}`, `{PGDN}` | Pagina su / Pagina giù |
| `{UP}`, `{DOWN}`, `{LEFT}`, `{RIGHT}` | Frecce di direzione |
| `{PRTSC}` | Print Screen (solo Windows) |
| `{BREAK}` | Break (solo Windows) |
| `{MEDIA_NEXT}`, `{MEDIA_PREV}`, `{MEDIA_STOP}`, `{MEDIA_PLAY}`, `{MEDIA_PLAY_PAUSE}` | Tasti multimediali (solo Windows nelle sequenze di tasti) |
| `{VOLUME_MUTE}`, `{VOLUME_DOWN}`, `{VOLUME_UP}` | Tasti del volume (solo Windows nelle sequenze di tasti) |
| `{CTRL}`, `{ALT}`, `{SHIFT}` | Control / Alt (Option su Mac) / Shift |
| `{LCTRL}`, `{RCTRL}`, `{LSHIFT}`, `{RSHIFT}`, `{LALT}`, `{RALT}` | Tasti modificatori sinistri/destri; Alt è Option su Mac |
| `{LWIN}`, `{WIN}`, `{RWIN}` | Tasto Windows; Command su Mac |
| `{F1}`, `{F2}`, `{F3}`, `{F4}`, `{F5}`, `{F6}`, `{F7}`, `{F8}`, `{F9}`, `{F10}`, `{F11}`, `{F12}`, `{F13}`, `{F14}`, `{F15}`, `{F16}`, `{F17}`, `{F18}`, `{F19}`, `{F20}`, `{F21}`, `{F22}`, `{F23}`, `{F24}` | Tasti funzione; Mac supporta F1–F20 |
| `{N0}`, `{N1}`, `{N2}`, `{N3}`, `{N4}`, `{N5}`, `{N6}`, `{N7}`, `{N8}`, `{N9}` | Cifre del tastierino numerico 0–9 |
| `{MOUSE_LEFT}`, `{MOUSE_RIGHT}`, `{MOUSE_MIDDLE}`, `{MOUSE_X1}`, `{MOUSE_X2}` | Clic del mouse (solo Windows) |
| `{MOUSE_WHEEL}`, `{MOUSE_HWHEEL}` | Rotella: es. {MOUSE_WHEEL 120}, {MOUSE_HWHEEL -120} (solo Windows) |
| `{SLEEP}` | Attesa: es. {SLEEP 100}, millisecondi |

## Pulsanti del mouse e rotella

Windows supporta `{MOUSE_LEFT}`, `{MOUSE_RIGHT}`, `{MOUSE_MIDDLE}`, `{MOUSE_X1}` e `{MOUSE_X2}` come clic. `{MOUSE_LEFT 2}` invia due clic; non riposiziona il puntatore.

`{MOUSE_WHEEL 120}` scorre verso l'alto di una tacca; `-120` verso il basso. `{MOUSE_HWHEEL 120}` scorre a destra; `-120` a sinistra. L'argomento è un delta con segno della rotella, non un conteggio di ripetizioni. È la convenzione dell'input iniettato; differisce dal segno usato dai passaggi rotella registrati nei gesti.

Mac analizza questi token ma salta le azioni mouse e rotella durante l'esecuzione dell'input. Non si possono sintetizzare clic o scorrimenti su Mac tramite `ms.keys`.

## Testo ed escape

La modalità testo è disponibile solo con il Lua `ms.keys(s, true)`; l'editor Invia tasti usa sempre la modalità tasti. I caratteri ordinari diventano testo Unicode, ma `^`, `%`, `+`, `#`, le parentesi e la parentesi graffa di apertura avviano comunque la sintassi. I singoli caratteri tra parentesi graffe restano azioni di tasto anche in modalità testo.

Per le scorciatoie dentro la modalità testo, metti il tasto tra graffe: `^{c}` invia Ctrl+C e `#{c}` invia Command+C. Un `^c` senza graffe in questa modalità emette il testo `c` e scarta il modificatore. Puoi anche fare una chiamata separata `ms.keys("^c")` in modalità tasti normale.

| Carattere letterale | Scrittura nella sequenza |
|---|---|
| `^` | `{^}` |
| `%` | `{%}` |
| `+` | `{+}` |
| `#` | `{#}` |
| `(` | `{(}` |
| `)` | `{)}` |
| `{` | `{{}` |
| `}` | `}` (senza graffe) |

La barra rovesciata non è un carattere di escape delle sequenze di tasti. In una stringa Lua tra virgolette, scrivi `"\\"` per una barra rovesciata, oppure usa una stringa con parentesi quadre lunghe. Per testi arbitrari contenenti simboli della sintassi, il [metodo degli appunti](lua-examples.md#incollare-e-ripristinare-il-testo-degli-appunti) evita entrambi i livelli di escape.

## Scorciatoie comuni

Dipendono dalle impostazioni delle scorciatoie dell'applicazione di destinazione e del sistema operativo.

| Scopo | Sequenza Windows | Sequenza Mac |
|---|---|---|
| Copia | `^c` | `#c` |
| Incolla | `^v` | `#v` |
| Taglia | `^x` | `#x` |
| Seleziona tutto | `^a` | `#a` |
| Annulla | `^z` | `#z` |
| Nuova scheda | `^t` | `#t` |
| Chiudi scheda | `^w` | `#w` |
| Riapri scheda chiusa | `^+t` | `#+t` |
| Scheda successiva | `^{TAB}` | `^{TAB}` |
| Scheda precedente | `^+{TAB}` | `^+{TAB}` |
| Cambia app | `%{TAB}` | `#{TAB}` |
| Desktop precedente/successivo | `#^{LEFT}` / `#^{RIGHT}` | `^{LEFT}` / `^{RIGHT}` |
| Screenshot di un'area | `#+s` | `#+4` |
| Mostra desktop | `#d` | `{F11}` |
| Gestione attività / Monitoraggio attività | `^+{ESC}` | Usa Aprire un'app: `com.apple.ActivityMonitor` |
| Blocca lo schermo | `#l` | `^#q` |

## Perché Ctrl+Alt+Del non può essere inviato

Ctrl+Alt+Del è la Secure Attention Sequence di Windows. Windows ignora l'input sintetico ordinario per questa sequenza, quindi `^%{DEL}` non può aprire quella schermata di sicurezza. Usa `^+{ESC}` per Gestione attività, `#l` per bloccare, oppure un'azione **Eseguire un comando** contenente `shutdown /l` per disconnetterti (il che termina la sessione).

## Limiti e note per piattaforma

Il parser condiviso non ha limiti complessivi sulla lunghezza della stringa né sul conteggio delle ripetizioni. Conteggi di ripetizione e attese vengono memorizzati come interi senza segno a 32 bit; i delta della rotella come interi con segno a 32 bit. I valori fuori intervallo possono andare in overflow alla conversione, quindi usa conteggi e attese piccoli e pratici. Un nome di tasto errato o un argomento obbligatorio mancante produce un errore di analisi; l'editor valida mentre digiti.

Windows esegue le sequenze su thread di lavoro e attende per `{SLEEP ms}`. Mac invia l'input in gruppi guidati da timer. Le stringhe d'azione Mac più lunghe di 16.384 byte vengono rifiutate; per l'input Lua già analizzato, i limiti sono 16.384 azioni e 16.384 byte per azione di testo. La sua coda di attesa contiene fino a 64 elementi; l'input in eccesso viene scartato e registrato nel log.

Su Mac sono disponibili F1–F20, le cifre del tastierino e i tasti di navigazione mappati. Insert/INS, PRTSC, BREAK, F21–F24, i token multimediali/volume e i token mouse/rotella non sono supportati e vengono saltati. Usa **Media e sistema** per media e volume. `ms.keys_down` e `ms.keys_up` sono solo per Windows; vedi le [API Lua](lua-api.md).
