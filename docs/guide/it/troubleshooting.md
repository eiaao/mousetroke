---
title: "Risoluzione dei problemi"
description: "Correggi i gesti del mouse che non si attivano, le autorizzazioni di macOS, le scorciatoie mancanti, l'input di testo con IME e gli errori Lua."
order: 12
---

# Risoluzione dei problemi

## I gesti non si attivano

Prova prima un gesto integrato semplice in una normale finestra del browser non a schermo intero.

- Controlla il menu dell'area di notifica/barra dei menu. Su Windows seleziona **Riprendi i gesti** se proposto; su Mac clicca di nuovo **Metti in pausa i gesti** se è spuntato. Controlla **Preferenze → Tasto pausa / riprendi** se una scorciatoia ha messo in pausa il motore.
- Esci dallo schermo intero. La soppressione automatica a schermo intero può fermare sia i gesti del mouse sia le azioni di bordo anche quando la pausa manuale è spenta.
- Seleziona l'app nelle impostazioni e controlla **Disattiva i gesti in questa app**. Controlla anche l'interruttore **Attivo** / **Disattivato** della singola associazione, **Usa anche i gesti globali**, il pulsante di attivazione ed eventuali passaggi extra.
- Muoviti prontamente dopo aver premuto il pulsante e disegna un tracciato più lungo. Regola **Quanto restare fermo perché diventi un clic** e **Sensibilità del tracciato** nelle Preferenze.
- Se manca il feedback, controlla **Mostra la scia** e **Mostra il nome del gesto mentre disegni**. Una scia nascosta non significa di per sé che il riconoscimento sia spento.

Su Windows l'input usa `SendInput`. Un'istanza avviata normalmente può non riuscire a inviare tasti/clic a un'app elevata a causa di UIPI di Windows. Prova prima con un'app non elevata; se devi controllare una finestra di amministratore, esegui Mousetroke alla stessa elevazione. Questo non abilita l'invio al desktop protetto.

Su Mac, controlla **Accessibilità** per la copia installata. Se la configurazione chiede ancora **Monitoraggio input**, concedi anche quello. Le modifiche alle autorizzazioni possono richiedere qualche secondo per essere rilevate. Usa **Riavvia Mousetroke** quando proposto se l'input ancora non parte. L'autorizzazione Registrazione dello schermo non è richiesta. Spostare o sostituire una copia tramite il browser può richiedere di nuovo l'approvazione alla prima apertura.

## SmartScreen o Gatekeeper blocca la prima apertura

Windows: procurati l'eseguibile ufficiale dalla pagina delle versioni, verifica la checksum, poi usa **Ulteriori informazioni → Esegui comunque**. Mac: installa in Applicazioni, poi usa **Impostazioni di Sistema → Privacy e sicurezza → Apri comunque**; le versioni meno recenti di macOS supportano anche **clic destro → Apri**. Vedi l'[installazione](getting-started.md) per la sequenza completa.

## Un'applicazione non riceve i tasti

Prova la scorciatoia sulla tastiera fisica. Controlla che l'applicazione sia attiva, oppure attiva **Attiva prima la finestra sotto il cursore**. Conferma la preferenza sul bersaglio del gesto e usa la [sintassi corretta](key-sequences.md): `^` è Control, mentre `#` è Command su Mac. Separa i passaggi dipendenti con un breve `{SLEEP 100}` se l'app richiede tempo per aprire un menu o una finestra di dialogo.

Le sequenze di tasti su Mac saltano i token mouse/rotella, i token dei tasti multimediali/volume, Insert, Print Screen, Break e F21–F24. Usa **Media e sistema** per le azioni multimediali/volume. Le opzioni non supportate sono contrassegnate nell'editor; controlla anche il log.

Ctrl+Alt+Del è una Secure Attention Sequence di Windows e non si può inviare come input sintetico ordinario. Usa Ctrl+Shift+Esc per Gestione attività, Win+L per bloccare, oppure `shutdown /l` per disconnetterti. I dettagli sono nella [sintassi di invio dei tasti](key-sequences.md#perché-ctrlaltdel-non-può-essere-inviato).

## L'IME o l'output di testo è errato

I caratteri ordinari dell'invio tasti rappresentano le posizioni della tastiera USA, non testo Unicode arbitrario. Le sole lettere maiuscole non aggiungono Shift. Il Lua `ms.keys(text, true)` emette testo Unicode ma continua a interpretare i caratteri della sintassi come `^` e `{`.

Per testi ricchi di punteggiatura, testo non latino o app con gestione IME insolita, usa la [ricetta incolla-e-ripristina con gli appunti](lua-examples.md#incollare-e-ripristinare-il-testo-degli-appunti). Il ripristino con queste API preserva solo il testo, non immagini o altri formati degli appunti. Alcune app richiedono un'attesa più lunga prima del ripristino degli appunti.

## Errori Lua

Assicurati che il codice definisca `function ms_action(...) ... end`. Il runtime scrive gli errori di caricamento e di esecuzione nel [log di runtime](config.md#posizioni-dei-file). Aprilo tramite **Informazioni → Apri registro**. Usa `ms.log` per i valori e `ms.show_toast` per un rapido controllo a schermo. Una funzione non supportata su Mac può solo registrare un avviso e restituire `nil`; non trattare un valore mancante come un successo.

Le chiamate di scoperta/stato delle finestre su Mac richiedono l'autorizzazione Accessibilità; le query sullo stato dei tasti restituiscono false senza di essa. Controlla prima quell'autorizzazione se mancano risultati. Always-on-top tra app e luminosità numerica restano non supportati; le azioni integrate dei tasti di luminosità non forniscono una percentuale. Su entrambe le piattaforme `ms.get_brightness` / `ms.set_brightness` restituiscono nil in caso di fallimento.

Il limite di allocazione Lua è 8 MiB. Non c'è timeout per l'esecuzione degli script; evita i cicli infiniti e vedi [Nozioni di base su Lua](lua-basics.md#runtime-e-limiti) per il comportamento bloccante.

## Desktop remoto e giochi

Desktop remoto, macchine virtuali e applicazioni 3D/giochi possono richiedere gli stessi pulsanti del mouse per il trascinamento o gestire l'input sintetico in modo diverso. Aggiungi un'[esclusione per app](per-app.md) oppure metti in pausa mentre li usi. I giochi in finestra possono richiedere un'esclusione esplicita perché il rilevamento a schermo intero non si applica. Prova le scorciatoie in una normale app locale per distinguere un problema di sequenza di tasti dalla gestione dell'input dell'applicazione.

## Segnalare un bug

Usa **Informazioni → Commenti e suggerimenti → Apri** oppure [GitHub Issues](https://github.com/eiaao/mousetroke/issues). Includi la versione dell'app, la versione del sistema operativo, l'applicazione interessata, gesto e azione, risultato atteso e i passaggi per riprodurre. Allega il log pertinente da **Informazioni → Apri registro** e una configurazione esportata minimale se utile. Controlla gli allegati per testo privato, titoli di finestre, percorsi o argomenti di comando prima di condividerli.
