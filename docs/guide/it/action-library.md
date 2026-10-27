---
title: "Libreria delle azioni"
description: "Scegli azioni pronte all’uso, imposta i parametri e consulta il catalogo completo per Windows e Mac."
order: 5.5
---

# Libreria delle azioni

Le azioni predefinite descrivono ciò che vuoi fare, come chiudere una scheda o centrare una finestra. Mousetroke salva l’ID e i parametri dell’azione e sceglie l’implementazione per Windows o Mac quando la esegue. Le scorciatoie di browser ed editor dipendono comunque dal supporto dell’app che le riceve.

## Scegliere un’azione

1. Crea o seleziona un gesto, poi clicca **Scegli azione**, oppure **Cambia** su un’azione esistente.
2. Cerca per nome, descrizione, alias o notazione della scorciatoia, come `ctrl+w` su Windows o `cmd+w` / `⌘w` su Mac. La ricerca comprende tutte le categorie. Senza una ricerca puoi esplorare **Frequenti**, **Browser**, **Modifica**, **Testo**, **Finestre**, **Desktop e sistema**, **Media**, **Apri** o **Mousetroke**.
3. Scegli un risultato. Le azioni con parametri aprono un modulo: inserisci testo, scegli un formato di data o un servizio di ricerca, oppure indica un URL, un file, una cartella o un’app. Clicca **Usa questa azione** per confermare. Le azioni non disponibili sono disabilitate e indicano il limite della piattaforma o l’implementazione prevista per la seconda fase.
4. Prova il gesto fuori dalle impostazioni. Se il nome del gesto è vuoto, riprende quello dell’azione selezionata. Le voci di menu usano lo stesso selettore.

All’apertura del selettore, il campo di ricerca riceve il focus. Usa le frecce su e giù per scorrere i risultati disponibili e Invio per sceglierne uno. Esc chiude prima il modulo dei parametri, poi il selettore. Nei campi di testo su più righe, Ctrl+Enter o Command+Enter conferma il modulo.

**Converti in azione avanzata** sostituisce un’azione predefinita con i tasti, l’operazione sulla finestra, l’URL, l’app, il comando o il tasto di sistema modificabili della piattaforma attuale. La conferma spiega che la conversione è irreversibile: il risultato non segue più le impostazioni predefinite della piattaforma o della lingua. Le azioni native per disposizione delle finestre, alimentazione, testo letterale, data e ora, copia come testo semplice, pausa e impostazioni restano predefinite; le loro schede spiegano perché non si possono convertire. La categoria **Avanzate**, in fondo all’elenco, offre invio di tasti, comandi, Lua e menu. Il [riferimento delle azioni avanzate](actions.md) descrive anche gli editor esistenti.

## Testo, finestre e alimentazione

- **Inserisci testo** restituisce il testo letteralmente, inclusi spazi, interruzioni di riga, caratteri cinesi, emoji e caratteri come `^+#{ENTER}`. **Incolla e ripristina appunti** è il metodo predefinito: usa temporaneamente gli appunti e ne ripristina il **testo** precedente dopo una breve attesa. Se contengono immagini, HTML, elenchi di file o altri dati non testuali, oppure non è possibile controllarli, il testo viene digitato direttamente senza modificare gli appunti. `paste_delay_ms` compare solo con `paste`: l’interfaccia consente 50–5000 ms, con 150 ms come valore predefinito. **Digita caratteri** usa l’input Unicode senza gli appunti. Entrambi i metodi inviano l’input all’app che riceve la tastiera.
- **Inserisci data e ora** usa l’orologio locale del sistema e il metodo di uscita scelto. I formati includono data ISO, data cinese, data statunitense, ora e data con ora. Un modello personalizzato accetta `YYYY`, `MM`, `DD`, `HH`, `mm` e `ss`, distinguendo maiuscole e minuscole. Gli altri caratteri sono letterali; gli apici singoli proteggono il testo letterale e `''` inserisce un apice. Per esempio, `YYYY-MM-DD HH:mm:ss` produce una marca temporale. Gli apici non chiusi vengono rifiutati.
- **Copia selezione come testo semplice** invia Ctrl+C / Command+C, attende fino a 500 ms un cambiamento di versione degli appunti, poi sostituisce il contenuto copiato con il solo testo. Se la copia non termina o non contiene testo, segnala un errore e conserva il contenuto degli appunti. Prima di usarla, seleziona il testo e mantieni l’app in primo piano.
- Le azioni di disposizione usano lo stesso bersaglio del gesto delle altre azioni sulle finestre, secondo **Preferenze → I gesti agiscono su**. Windows usa il posizionamento nativo delle finestre; Mac usa Accessibilità e l’area utilizzabile dello schermo. Le dimensioni minime e le restrizioni delle app possono impedire di occupare esattamente metà schermo. Su Mac esci prima dallo schermo intero nativo. **Sposta sul monitor successivo** scorre gli schermi in un ordine stabile, conserva posizione relativa e stato di ingrandimento, e non fa nulla con un solo schermo.
- Dopo **Spegni schermo**, un input può riattivare gli schermi. **Metti in stop** richiede la normale sospensione del sistema, che l’account o i criteri del sistema possono negare. Se un’azione nativa fallisce, viene scritta una riga nel log e appare un breve messaggio sullo schermo. **Sospendi / riprendi gesti** usa lo stesso stato di pausa dell’icona nell’area di notifica e della scorciatoia; usa uno dei due per riprendere i gesti sospesi. **Apri impostazioni** apre la finestra abituale.

## Catalogo

Le tabelle seguenti sono generate dal registro e includono tutte le 87 voci. ✓ indica che l’azione è implementata sulla piattaforma; la disponibilità dipende comunque dall’app, dall’hardware e dalle autorizzazioni necessarie. Il promemoria dei gesti è previsto per la seconda fase.

<!-- action-library:generated:start -->

### Browser

| Azione / ID | Funzione e ambito d’uso | Parametri | Windows | Mac |
|---|---|---|---|---|
| Indietro<br>`browser.back` | Funziona nei browser web. | — | ✓ | ✓ |
| Avanti<br>`browser.forward` | Funziona nei browser web. | — | ✓ | ✓ |
| Aggiorna<br>`browser.reload` | Funziona nei browser web. | — | ✓ | ✓ |
| Ricarica senza cache<br>`browser.hard_reload` | Funziona nei browser web. | — | ✓ | ✓ |
| Nuova scheda<br>`browser.new_tab` | Browser, editor e altre app che supportano questa scorciatoia. | — | ✓ | ✓ |
| Chiudi scheda<br>`browser.close_tab` | Browser, editor e altre app che supportano questa scorciatoia. | — | ✓ | ✓ |
| Riapri scheda chiusa<br>`browser.reopen_tab` | Funziona nei browser web. | — | ✓ | ✓ |
| Scheda precedente<br>`browser.prev_tab` | Browser, editor e altre app che supportano questa scorciatoia. | — | ✓ | ✓ |
| Scheda successiva<br>`browser.next_tab` | Browser, editor e altre app che supportano questa scorciatoia. | — | ✓ | ✓ |
| Nuova finestra<br>`browser.new_window` | Browser, editor e altre app che supportano questa scorciatoia. | — | ✓ | ✓ |
| Nuova finestra privata<br>`browser.private_window` | Chrome, Edge e Safari; Firefox usa una scorciatoia diversa. | — | ✓ | ✓ |
| Vai alla barra degli indirizzi<br>`browser.address_bar` | Funziona nei browser web. | — | ✓ | ✓ |
| Trova nella pagina<br>`browser.find` | Browser, editor e altre app che supportano questa scorciatoia. | — | ✓ | ✓ |
| Aggiungi pagina ai preferiti<br>`browser.bookmark` | Funziona nei browser web. | — | ✓ | ✓ |
| Ingrandisci<br>`view.zoom_in` | Browser, editor e altre app che supportano questa scorciatoia. | — | ✓ | ✓ |
| Riduci<br>`view.zoom_out` | Browser, editor e altre app che supportano questa scorciatoia. | — | ✓ | ✓ |
| Reimposta zoom<br>`view.zoom_reset` | Browser, editor e altre app che supportano questa scorciatoia. | — | ✓ | ✓ |

### Modifica

| Azione / ID | Funzione e ambito d’uso | Parametri | Windows | Mac |
|---|---|---|---|---|
| Vai in cima<br>`view.top` | Browser, editor e altre app che supportano questa scorciatoia. | — | ✓ | ✓ |
| Vai in fondo<br>`view.bottom` | Browser, editor e altre app che supportano questa scorciatoia. | — | ✓ | ✓ |
| Pagina su<br>`view.page_up` | Browser, editor e altre app che supportano questa scorciatoia. | — | ✓ | ✓ |
| Pagina giù<br>`view.page_down` | Browser, editor e altre app che supportano questa scorciatoia. | — | ✓ | ✓ |
| Copia<br>`edit.copy` | Browser, editor e altre app che supportano questa scorciatoia. | — | ✓ | ✓ |
| Taglia<br>`edit.cut` | Browser, editor e altre app che supportano questa scorciatoia. | — | ✓ | ✓ |
| Incolla<br>`edit.paste` | Browser, editor e altre app che supportano questa scorciatoia. | — | ✓ | ✓ |
| Incolla e Invio<br>`edit.paste_enter` | Browser, editor e altre app che supportano questa scorciatoia. | — | ✓ | ✓ |
| Incolla come testo semplice<br>`edit.paste_plain` | App con incolla senza formattazione; la scorciatoia può variare. | — | ✓ | ✓ |
| Seleziona tutto<br>`edit.select_all` | Browser, editor e altre app che supportano questa scorciatoia. | — | ✓ | ✓ |
| Annulla<br>`edit.undo` | Browser, editor e altre app che supportano questa scorciatoia. | — | ✓ | ✓ |
| Ripeti<br>`edit.redo` | Browser, editor e altre app che supportano questa scorciatoia. | — | ✓ | ✓ |
| Salva<br>`edit.save` | Browser, editor e altre app che supportano questa scorciatoia. | — | ✓ | ✓ |
| Invio<br>`key.enter` | Browser, editor e altre app che supportano questa scorciatoia. | — | ✓ | ✓ |
| Backspace<br>`key.backspace` | Browser, editor e altre app che supportano questa scorciatoia. | — | ✓ | ✓ |
| Delete<br>`key.delete` | Browser, editor e altre app che supportano questa scorciatoia. | — | ✓ | ✓ |
| Annulla (Esc)<br>`key.escape` | Browser, editor e altre app che supportano questa scorciatoia. | — | ✓ | ✓ |
| Schermo intero<br>`view.fullscreen` | Browser, editor e altre app che supportano questa scorciatoia. | — | ✓ | ✓ |

### Testo

| Azione / ID | Funzione e ambito d’uso | Parametri | Windows | Mac |
|---|---|---|---|---|
| Inserisci testo<br>`text.type` | Campi di testo; incolla e ripristina gli appunti per impostazione predefinita. | Testo (`text`): obbligatorio<br>Metodo di inserimento (`method`): Incolla e ripristina appunti (`paste`) / Digita caratteri (`type`); predefinito: Incolla e ripristina appunti<br>Attesa dopo incolla (ms) (`paste_delay_ms`): predefinito: `150`; solo se Metodo di inserimento è Incolla e ripristina appunti | ✓ | ✓ |
| Inserisci data e ora<br>`text.datetime` | Campi di testo; usa data e ora locali. | Formato data e ora (`format`): 2026-10-24 (`date_iso`) / 2026年10月24日 (`date_local`) / 10/24/2026 (`date_us`) / 14:30 (`time`) / 2026-10-24 14:30 (`datetime`) / Personalizzato (`custom`); predefinito: 2026-10-24<br>Formato personalizzato (`custom_format`): predefinito: `YYYY-MM-DD`; solo se Formato data e ora è Personalizzato<br>Metodo di inserimento (`method`): Incolla e ripristina appunti (`paste`) / Digita caratteri (`type`); predefinito: Incolla e ripristina appunti<br>Attesa dopo incolla (ms) (`paste_delay_ms`): predefinito: `150`; solo se Metodo di inserimento è Incolla e ripristina appunti | ✓ | ✓ |
| Ricerca web<br>`text.search` | Usa il browser predefinito; seleziona prima il testo nell’app attuale. | Motore di ricerca (`engine`): Bing (`bing`) / Google (`google`) / Baidu (`baidu`) / DuckDuckGo (`duckduckgo`) / Personalizzato (`custom`); Baidu con l’interfaccia in cinese semplificato; Google nelle altre lingue<br>URL personalizzato (`url`): predefinito: `https://www.google.com/search?q=%MS_SELECTED_TEXT%`; solo se Motore di ricerca è Personalizzato | ✓ | ✓ |
| Traduci il testo selezionato<br>`text.translate` | Usa il browser predefinito; seleziona prima il testo nell’app attuale. | Servizio di traduzione (`service`): Google Traduttore (`google`) / DeepL (`deepl`) / Traduttore Bing (`bing`); predefinito: Google Traduttore<br>Lingua di destinazione (`target_language`): Segui lingua dell’interfaccia (`ui`) / 简体中文 (`zh-CN`) / English (`en-US`) / español (`es-ES`) / Français (`fr-FR`) / Deutsch (`de-DE`) / 日本語 (`ja-JP`) / Português (Brasil) (`pt-BR`) / Русский (`ru-RU`) / Italiano (`it-IT`) / 한국어 (`ko-KR`); predefinito: Segui lingua dell’interfaccia | ✓ | ✓ |
| Apri l’URL selezionato<br>`text.open_selected_url` | Usa il browser predefinito; seleziona prima il testo nell’app attuale. | — | ✓ | ✓ |
| Copia selezione come testo semplice<br>`text.copy_plain` | App che copiano testo; rimuove la formattazione dagli appunti. | — | ✓ | ✓ |

### Finestre

| Azione / ID | Funzione e ambito d’uso | Parametri | Windows | Mac |
|---|---|---|---|---|
| Riduci a icona<br>`window.minimize` | Si applica alla finestra di destinazione del gesto. | — | ✓ | ✓<br>Richiede il permesso Accessibilità. |
| Ingrandisci / ripristina<br>`window.toggle_maximize` | Si applica alla finestra di destinazione del gesto. | — | ✓ | ✓<br>Richiede il permesso Accessibilità. |
| Chiudi finestra<br>`window.close` | Si applica alla finestra di destinazione del gesto. | — | ✓ | ✓<br>Richiede il permesso Accessibilità. |
| Esci dall’app<br>`app.quit` | Si applica alla finestra di destinazione del gesto. | — | ✓ | ✓ |
| Termina processo<br>`app.kill` | Forza la chiusura dell’app; il lavoro non salvato andrà perso. | — | ✓ | ✓ |
| Sempre in primo piano<br>`window.toggle_topmost` | Attiva o disattiva sempre in primo piano per la finestra di destinazione. | — | ✓ | Questa azione è supportata solo su Windows. |
| Disponi nella metà sinistra<br>`window.snap_left` | Si applica alla finestra di destinazione del gesto. | — | ✓ | ✓<br>Richiede il permesso Accessibilità. |
| Disponi nella metà destra<br>`window.snap_right` | Si applica alla finestra di destinazione del gesto. | — | ✓ | ✓<br>Richiede il permesso Accessibilità. |
| Centra finestra<br>`window.center` | Si applica alla finestra di destinazione del gesto. | — | ✓ | ✓<br>Richiede il permesso Accessibilità. |
| Sposta sul monitor successivo<br>`window.next_monitor` | Si applica alla finestra di destinazione del gesto. | — | ✓ | ✓<br>Richiede il permesso Accessibilità. |
| Nascondi l’app attuale<br>`app.hide` | Nascondi l’applicazione corrente. | — | Questa azione è supportata solo su macOS. | ✓ |

### Desktop e sistema

| Azione / ID | Funzione e ambito d’uso | Parametri | Windows | Mac |
|---|---|---|---|---|
| Cambia app<br>`desktop.switch_app` | Azione di sistema; alcuni comportamenti dipendono dalle scorciatoie configurate. | — | ✓ | ✓ |
| Visualizzazione attività / Mission Control<br>`desktop.task_view` | Azione di sistema; alcuni comportamenti dipendono dalle scorciatoie configurate. | — | ✓ | ✓ |
| Mostra desktop<br>`desktop.show` | Azione di sistema; alcuni comportamenti dipendono dalle scorciatoie configurate. | — | ✓ | ✓ |
| Desktop virtuale precedente<br>`desktop.prev` | Azione di sistema; alcuni comportamenti dipendono dalle scorciatoie configurate. | — | ✓ | ✓ |
| Desktop virtuale successivo<br>`desktop.next` | Azione di sistema; alcuni comportamenti dipendono dalle scorciatoie configurate. | — | ✓ | ✓ |
| Menu Start / Spotlight<br>`desktop.launcher` | Azione di sistema; alcuni comportamenti dipendono dalle scorciatoie configurate. | — | ✓ | ✓ |
| Apri gestore file<br>`desktop.files` | Azione di sistema; alcuni comportamenti dipendono dalle scorciatoie configurate. | — | ✓ | ✓ |
| Ritaglio<br>`capture.region` | Azione di sistema; alcuni comportamenti dipendono dalle scorciatoie configurate. | — | ✓ | ✓ |
| Cattura e salva lo schermo<br>`capture.screen` | Azione di sistema; alcuni comportamenti dipendono dalle scorciatoie configurate. | — | ✓ | ✓ |
| Strumenti di cattura e registrazione<br>`capture.tool` | Azione di sistema; alcuni comportamenti dipendono dalle scorciatoie configurate. | — | ✓ | ✓ |
| Cronologia appunti<br>`desktop.clipboard_history` | Apri la cronologia degli appunti. | — | ✓ | Questa azione è supportata solo su Windows. |
| Emoji e simboli<br>`desktop.emoji` | Azione di sistema; alcuni comportamenti dipendono dalle scorciatoie configurate. | — | ✓ | ✓ |
| Centro notifiche<br>`desktop.notifications` | Apri il Centro Notifiche. | — | ✓ | Questa azione è supportata solo su Windows. |
| Blocca schermo<br>`system.lock` | Azione di sistema; alcuni comportamenti dipendono dalle scorciatoie configurate. | — | ✓ | ✓ |
| Gestione attività / Monitoraggio Attività<br>`system.task_manager` | Azione di sistema; alcuni comportamenti dipendono dalle scorciatoie configurate. | — | ✓ | ✓ |
| Finestra Uscita forzata<br>`system.force_quit_dialog` | Apri la finestra Uscita forzata applicazioni. | — | Questa azione è supportata solo su macOS. | ✓ |
| Impostazioni di sistema<br>`system.settings` | Azione di sistema; alcuni comportamenti dipendono dalle scorciatoie configurate. | — | ✓ | ✓ |
| Calcolatrice<br>`system.calculator` | Azione di sistema; alcuni comportamenti dipendono dalle scorciatoie configurate. | — | ✓ | ✓ |
| Spegni schermo<br>`system.display_off` | Azione di sistema; alcuni comportamenti dipendono dalle scorciatoie configurate. | — | ✓ | ✓ |
| Metti in stop<br>`system.sleep` | Azione di sistema; alcuni comportamenti dipendono dalle scorciatoie configurate. | — | ✓ | ✓ |

### Media

| Azione / ID | Funzione e ambito d’uso | Parametri | Windows | Mac |
|---|---|---|---|---|
| Riproduci / pausa<br>`media.play_pause` | Tasti multimediali di sistema; richiede il supporto del lettore. | — | ✓ | ✓ |
| Traccia precedente<br>`media.previous` | Tasti multimediali di sistema; richiede il supporto del lettore. | — | ✓ | ✓ |
| Traccia successiva<br>`media.next` | Tasti multimediali di sistema; richiede il supporto del lettore. | — | ✓ | ✓ |
| Alza il volume<br>`media.volume_up` | Tasti multimediali di sistema; richiede il supporto del lettore. | — | ✓ | ✓ |
| Abbassa il volume<br>`media.volume_down` | Tasti multimediali di sistema; richiede il supporto del lettore. | — | ✓ | ✓ |
| Silenzia<br>`media.mute` | Tasti multimediali di sistema; richiede il supporto del lettore. | — | ✓ | ✓ |
| Più luminoso<br>`display.brightness_up` | Luminosità di sistema; dipende da schermo e driver. | — | ✓ | ✓ |
| Meno luminoso<br>`display.brightness_down` | Luminosità di sistema; dipende da schermo e driver. | — | ✓ | ✓ |

### Apri

| Azione / ID | Funzione e ambito d’uso | Parametri | Windows | Mac |
|---|---|---|---|---|
| Apri sito web<br>`open.website` | Apre l’URL specificato nel browser predefinito. | URL (`url`): predefinito: `https://mousetroke.com/` | ✓ | ✓ |
| Apri cartella<br>`open.folder` | Apre il percorso specificato con l’app predefinita di sistema. | Percorso (`path`): obbligatorio | ✓ | ✓ |
| Apri file<br>`open.file` | Apre il percorso specificato con l’app predefinita di sistema. | Percorso (`path`): obbligatorio | ✓ | ✓ |
| Apri o attiva app<br>`open.app` | Apre l’app o attiva la finestra esistente. | App (`app`): obbligatorio | ✓ | ✓ |

### Mousetroke

| Azione / ID | Funzione e ambito d’uso | Parametri | Windows | Mac |
|---|---|---|---|---|
| Sospendi / riprendi gesti<br>`mousetroke.toggle_pause` | Sospende o riprende il riconoscimento gesti di Mousetroke. | — | ✓ | ✓ |
| Apri impostazioni<br>`mousetroke.settings` | Apre le impostazioni di Mousetroke. | — | ✓ | ✓ |
| Mostra elenco gesti<br>`mousetroke.cheatsheet` | Elenca i gesti dell’app attuale; previsto per la fase 2. | — | Questa azione è prevista per la fase 2. | Questa azione è prevista per la fase 2. |

<!-- action-library:generated:end -->

Dopo aver modificato il registro o le traduzioni, imposta `UPDATE_ACTION_DOCS=1` ed esegui `cargo test -p mousetroke-core --test action_docs` per aggiornare i cataloghi di tutte e dieci le lingue. Esegui lo stesso comando senza la variabile per verificare che le tabelle salvate nel repository siano aggiornate.
