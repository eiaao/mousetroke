---
title: "Libreria dei gesti"
description: "Definisci un gesto una volta e usalo in più app con azioni diverse: riordina e importa i gesti esistenti, assegnali ad altre app, modifica i gesti condivisi, note di compatibilità."
order: 3.5
---

# Libreria dei gesti

La libreria dei gesti separa come disegni da ciò che succede. Definisci un gesto una volta e assegnagli un'azione diversa in ogni app. Registrando di nuovo un gesto condiviso si aggiornano tutti i punti che lo usano, e un gesto a forma conserva un solo insieme di campioni.

## Sfogliare la libreria

Clicca **Gesti** nella barra laterale. L'elenco è diviso in **Gesti del mouse** e **Bordi dello schermo**. Filtra per pulsante di attivazione, usa **Non usati** per trovare i gesti non usati o cerca per nome, tracciato o utilizzo. Ogni scheda mostra in quanti punti è usato il gesto.

Clicca una scheda per aprirne i dettagli. In alto puoi ridisegnarlo; **Usato in** elenca ogni app, nome del gesto e azione che lo usa, e cliccando una riga ci arrivi. In una nuova installazione i gesti predefiniti sono già nella libreria. Dopo l'aggiornamento da una versione precedente i tuoi gesti sono ancora sparsi tra le app: riordinali prima come descritto sotto.

## Riordinare e importare

Con la libreria vuota clicca **Organizza e importa i gesti…**; in seguito trovi sempre **Organizza e importa…** in alto a destra. Scegli prima l'ambito:

- **Solo gesti comuni (consigliato)**: aggiunge i gesti di **Tutte le app** e collega quelli identici o simili nelle altre app. I gesti impostati solo per un'app restano specifici.
- **Tutti i gesti**: aggiunge i gesti condivisi e tutti quelli specifici delle app.

Clicca **Avanti** per controllare il risultato. Nulla cambia finché non confermi:

- I tracciati identici vengono uniti automaticamente. I tracciati con le stesse direzioni che differiscono solo nella lunghezza dei segmenti vengono uniti per impostazione predefinita.
- **Forme simili: confermare**: per ogni gruppo scegli quello da tenere (**Usa questo**) oppure **Combina i campioni**. I gruppi sotto la somiglianza consigliata non vengono uniti per impostazione predefinita, e una forma unita può essere riconosciuta in modo leggermente diverso.
- **Aggiunti direttamente**: espandilo per deselezionare gesti. I gesti deselezionati non vengono importati e restano specifici della loro app.

Clicca **Importa** per finire; l'avviso permette di annullare. Due gesti simili nella stessa app che fanno cose diverse non vengono mai uniti.

## Usare un gesto in un'altra app

Nei dettagli del gesto clicca **Usa in un’altra app…**, scegli un'app e poi un'azione. Le app che usano già il gesto non sono selezionabili; se un'app ha già lo stesso tracciato compare un avviso di conflitto; puoi assegnarlo a un'app con i gesti disattivati, ma lì non funzionerà. **Aggiungi app…** in fondo all'elenco aggiunge prima un'app. Poi resti nei dettagli del gesto con la nuova riga evidenziata, e **Modifica** nell'avviso apre quell'associazione.

Puoi anche partire dall'app. Nell'editor, un gesto che appartiene solo all'app corrente mostra **Solo in questa app**:

- Clicca **Scegli dalla libreria** per usare un gesto della libreria. Il selettore è ricercabile; scegliendo uno dei **Gesti in altre app** viene prima aggiunto alla libreria e condiviso con l'originale; **Registra un nuovo gesto (solo in questa app)** non entra nella libreria.
- Il menu **Altre azioni** contiene **Aggiungi alla libreria** e **Usa in un’altra app…**.

## Modificare un gesto condiviso

Nell'editor, un gesto della libreria mostra **Libreria** e dove è usato. Se lo ridisegni, la modifica vale ovunque sia usato e vengono elencati i nuovi conflitti:

- **Modifica solo qui**: la modifica resta solo nell'app corrente; il gesto diventa specifico, mentre la libreria e le altre app mantengono il tracciato precedente.
- **Fatto**: mantiene la modifica condivisa e chiude l'avviso.

**Cambia** passa a un altro gesto della libreria; il pulsante ↗ (**Mostra nella libreria**) apre i dettagli del gesto.

## Creare ed eliminare

**Nuovo gesto** registra un gesto direttamente nella libreria. Viene aggiunto quando finisci di disegnare; il nome è facoltativo. Assegnalo poi con **Usa in un’altra app…**.

Se scegli **Elimina gesto** mentre il gesto è ancora in uso, scegli:

- **Elimina; ogni utilizzo mantiene una propria copia**
- **Elimina insieme a tutti gli utilizzi**

Entrambe si possono annullare dall'avviso.

## Configurazione e compatibilità

La libreria è salvata in `gestures` nel file di configurazione, e ogni gesto che la usa conserva anche una copia completa del suo tracciato. Le versioni precedenti di Mousetroke possono quindi leggere la nuova configurazione e i gesti continuano a funzionare. Salvando le impostazioni in una versione precedente, però, la libreria va persa: ogni gesto torna specifico della sua app e puoi riordinare di nuovo dopo l'aggiornamento. Se una modifica manuale del file rende un tracciato diverso dalla libreria, quel punto diventa specifico e ricevi un avviso una volta. Posizione dei file e backup: [Configurazione, backup e aggiornamenti](config.md).

Successivo: [Gesti del mouse](gestures.md), [Regole per app](per-app.md), [Libreria delle azioni](action-library.md).
