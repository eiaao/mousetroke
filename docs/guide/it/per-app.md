---
title: "Regole per app"
description: "Aggiungi app, sostituisci i gesti del mouse globali, escludi le app che richiedono il trascinamento del mouse e comprendi la sospensione automatica a schermo intero."
order: 6
---

# Regole per app

## Aggiungere un'applicazione

Clicca **Aggiungi app…** nella barra laterale. Scegli **Gesti solo per essa** o **Disattiva i gesti al suo interno**, poi:

- Seleziona una voce sotto **In esecuzione**.
- Clicca **Scegli una finestra**, poi clicca la finestra dell'applicazione; Esc o il clic destro annulla.
- Usa **Sfoglia…** per scegliere l'applicazione.
- Trascina un'applicazione o il suo collegamento nella finestra delle impostazioni. Mac accetta bundle `.app` ed eseguibili; i suoi ambiti usano gli ID bundle quando disponibili.

Le regole Windows possono corrispondere a un nome di eseguibile o a un percorso completo, senza distinzione tra maiuscole e minuscole. Le identità delle app su Mac di norma hanno l'aspetto di `com.apple.Safari`. Usa il selettore invece di indovinare l'identità. L'ambito dell'app compare nella barra laterale e può avere i propri gesti del mouse e bordi dello schermo.

## Gesti specifici per app e gesti globali

**Tutte le app** contiene i gesti globali. Un gesto specifico per un'app ha priorità sul gesto globale corrispondente. Con **Usa anche i gesti globali** spuntato, le associazioni globali restano disponibili quando quelle dell'app non corrispondono. Deseleionalo per usare solo i gesti di quell'app.

Per esempio, aggiungi un browser, crea un gesto `↓→` col pulsante destro e assegnagli lì un'azione diversa. Le altre app mantengono l'azione globale di `↓→`. Per copiare una configurazione esistente, usa [Duplica](gestures.md#trovare-copiare-e-disattivare) dentro il suo ambito e modifica l'associazione.

## Disattivare i gesti in un'app

Seleziona l'app nella barra laterale e attiva **Disattiva i gesti in questa app**. Questo disattiva i gesti propri dell'app, i gesti globali e gli attivatori di bordo. I pulsanti destro, centrale e laterali passano normalmente. Le associazioni salvate restano; **Riattiva i gesti** le ripristina. **Rimuovi questa app** elimina l'ambito specifico dell'app.

È utile per desktop remoto, macchine virtuali, modellazione 3D, strumenti di progettazione ed editor di giochi che usano il trascinamento col destro/centrale. L'elenco **Consigliato disattivarli** del selettore include applicazioni note di queste categorie. Le app installate rilevate si possono aggiungere tutte insieme.

## Schermo intero e prima esecuzione

La sospensione automatica a schermo intero è appunto automatica: il motore evita di avviare gesti e attivatori di bordo mentre l'app in primo piano viene rilevata a schermo intero. Copre giochi a schermo intero, video e presentazioni; è separata dallo stato di pausa manuale. Su Windows, il rilevamento controlla una finestra senza bordi che copre il suo monitor ed esclude le finestre desktop/shell. Su Mac confronta la finestra dell'app in primo piano con i limiti dello schermo. Un'app dall'aspetto a schermo intero può quindi comportarsi diversamente sulle due piattaforme.

Per un gioco in finestra o per il desktop remoto, aggiungi un'esclusione esplicita. Per mettere in pausa ovunque, usa il [comando dall'area di notifica/barra dei menu o il tasto di pausa](getting-started.md#pausa).

La guida iniziale mostra le app rilevate che di norma richiedono un proprio trascinamento del mouse. Le voci spuntate vengono disattivate al termine. Controllale prima di continuare. La configurazione integrata di Mac include anche ambiti disattivati per le app la cui esclusione è consigliata; questi non hanno associazioni specifiche. Puoi modificare le esclusioni in seguito dalla barra laterale.

Successivo: [azioni](actions.md), [backup della configurazione](config.md), [risoluzione dei problemi](troubleshooting.md).
