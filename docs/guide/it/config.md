---
title: "Configurazione, backup e aggiornamenti"
description: "Trova configurazione e log di Mousetroke, esporta e ripristina le impostazioni, ricarica le modifiche, aggiorna l'app o disinstallala."
order: 11
---

# Configurazione, backup e aggiornamenti

## Posizioni dei file

| File | Windows | Mac |
|---|---|---|
| Configurazione | `%APPDATA%\mousetroke\config.json` | `~/Library/Application Support/mousetroke/config.json` |
| Log di runtime | `%APPDATA%\mousetroke\mousetroke.log` | `~/Library/Logs/mousetroke/mousetroke.log` |

I log ruotano su `mousetroke.log.old` dopo aver superato 1 MB. **Informazioni → Posizioni dei file** mostra i percorsi effettivi. **Apri cartella** apre la cartella di configurazione; **Apri registro** apre il log di runtime. Sotto **Importa ed esporta**, **Mostra in Esplora file** su Windows o **Mostra nel Finder** su Mac rivela la configurazione.

## Esportare, importare e ripristinare

Sotto **Importa ed esporta → File di configurazione di Mousetroke**, clicca **Esporta…** e salva un backup JSON. **Importa da file…** legge un JSON di configurazione di Mousetroke e sostituisce la configurazione attuale, preferenze incluse. Non unisce quel file nelle associazioni attuali. In questo flusso non esiste un'opzione di unione della configurazione nativa. Puoi annullare l'importazione con Ctrl+Z su Windows o ⌘+Z su Mac mentre le impostazioni sono aperte.

Prima di modificare il JSON a mano, esporta un backup. Per un ripristino diretto, chiudi il motore e le impostazioni, sostituisci `config.json` con il backup, poi riapri Mousetroke. Tenere le impostazioni chiuse evita che il salvataggio automatico sovrascriva il file ripristinato.

## Ricaricare e reimpostare

Il motore osserva il timestamp del file di configurazione. Windows controlla circa una volta al secondo; Mac controlla nel ramo di manutenzione pianificato ogni due secondi. Una modifica ricevuta durante un gesto viene applicata al termine della cattura. Un JSON non valido lascia in uso la configurazione attiva precedente e registra un errore nel log.

L'interfaccia attuale delle impostazioni non ha un pulsante di ripristino dei valori predefiniti. Per ripartire dai valori integrati, chiudi Mousetroke e le impostazioni, fai un backup e rimuovi o rinomina `config.json`, poi riapri. Un file mancante viene inizializzato con i valori predefiniti specifici della piattaforma. Un file malformato non è una richiesta di ripristino.

## Aggiornamenti

**Preferenze → Cerca aggiornamenti automaticamente** è attivo per impostazione predefinita e controlla su GitHub una volta al giorno. Disattivalo lì per fermare i controlli automatici. Usa **Informazioni → Cerca aggiornamenti** per un controllo manuale e **Novità** per le note di versione. Anche una voce di menu sulla nuova versione apre Informazioni.

Su Windows, **Aggiorna ora** scarica l'eseguibile, verifica il suo SHA-256, sostituisce la copia in esecuzione e riavvia. **Apri la pagina di download** è disponibile per l'installazione manuale.

Su Mac, **Aggiorna ora** scarica il DMG, verifica SHA-256, identità del bundle, versione e requisito di firma, sostituisce l'app e riavvia. Il vecchio bundle va nel Cestino dopo l'arresto; una sostituzione o un passaggio di riavvio non riuscito lo ripristina. **Scarica manualmente** è disponibile se la cartella di installazione non è scrivibile o l'aggiornamento fallisce. Gli aggiornamenti dall'app evitano un altro avviso di quarantena del browser e preservano l'identità di installazione usata per le autorizzazioni. Un nuovo download dal browser può richiedere di nuovo **Apri comunque**.

## Disinstallazione

Windows: disattiva **Attiva i gesti all'avvio di Windows** nelle Preferenze, **Esci** dall'area di notifica, chiudi le impostazioni, poi elimina l'eseguibile e `%APPDATA%\mousetroke` se non ti servono più impostazioni o log.

Mac: **Esci** dalla barra dei menu, chiudi le impostazioni ed elimina `Mousetroke.app`. Per rimuovere anche i dati, elimina `~/Library/Application Support/mousetroke` e `~/Library/Logs/mousetroke`. Rimuovi Mousetroke dagli elementi login nelle Impostazioni di Sistema. Conserva un backup esportato se potresti reinstallare.

Vedi la [risoluzione dei problemi](troubleshooting.md) per errori di configurazione e log mancanti.
