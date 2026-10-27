---
title: "Primi passi"
description: "Installa Mousetroke su Windows o Mac, disegna il tuo primo gesto del mouse, apri le impostazioni e metti in pausa i gesti."
order: 2
---

# Primi passi

## Installare e aprire

Su Windows, scarica `mousetroke.exe` dall'[ultima versione](https://github.com/eiaao/mousetroke/releases/latest). È un eseguibile portabile: salvalo in una posizione definitiva ed eseguilo con un doppio clic. Confronta l'SHA-256 del download con `SHA256SUMS.txt` presente in quella pagina delle versioni.

Sono supportati Windows 10 1809 o successivo e Windows 11, a 64 bit. Le impostazioni usano Microsoft Edge WebView2, incluso in Windows 11. L'eseguibile non è ancora firmato. Se SmartScreen blocca il primo avvio, scegli **Ulteriori informazioni → Esegui comunque** dopo aver verificato di aver scaricato la versione ufficiale.

Su Mac, scarica `Mousetroke.dmg`, aprilo e trascina **Mousetroke** in **Applicazioni**. L'app universale supporta Apple silicon e Intel su macOS 11 o successivo. Se aperta dal DMG, l'app propone **Sposta in Applicazioni** e riapre la copia installata.

La beta per Mac è autofirmata e non notarizzata. Se viene bloccata, usa **Impostazioni di Sistema → Privacy e sicurezza → Apri comunque**; sulle versioni meno recenti di macOS usa **clic destro → Apri**. Durante la configurazione, clicca su **Consenti…** e attiva Mousetroke in **Accessibilità**. Attendi **Tutto pronto: puoi usare i gesti**. L'autorizzazione Registrazione dello schermo non è richiesta. La configurazione propone **Monitoraggio input** se l'ascolto continua a non funzionare sui sistemi che lo richiedono; attivala quando viene richiesto.

## Aprire le impostazioni e concludere la guida

Su Windows, clicca con il pulsante sinistro sull'icona nell'area di notifica per aprire le impostazioni. Clicca con il destro per **Impostazioni…**, **Metti in pausa i gesti** / **Riprendi i gesti**, **Avvio automatico**, **Apri cartella di configurazione** ed **Esci**. Quando è disponibile un aggiornamento può comparire anche una voce sulla nuova versione.

Su Mac, usa l'icona nella barra dei menu per aprire **Impostazioni…**, mettere in pausa/riprendere o uscire con **Esci**. Le voci su autorizzazioni e aggiornamenti compaiono quando servono. **Informazioni su Mousetroke** apre la pagina Informazioni.

La guida iniziale spiega come disegnare, usare i bordi dello schermo, escludere app e mettere in pausa. Controlla le app spuntate prima di concludere: in quelle app i gesti resteranno disattivati. Puoi riaprirla con **Informazioni → Rivedi la guida iniziale**. Le modifiche nelle impostazioni vengono salvate automaticamente; un tratto incompleto deve essere registrato prima di poter salvare.

## Disegnare il primo gesto

1. Apri una pagina del browser con cronologia di navigazione.
2. Tieni premuto il pulsante destro del mouse, muovi verso sinistra per disegnare `←`, poi rilascia. Si torna indietro.
3. Prova `↓→` su una scheda sacrificabile per chiuderla, oppure `↙` per ridurre la finestra a icona.

Le azioni integrate seguono la piattaforma: le scorciatoie del browser usano Ctrl su Windows e Command su Mac dove opportuno. L'elenco completo modificabile si trova sotto **Tutte le app → Gesti del mouse**. Alcune azioni integrate sono disattivate, in particolare quelle non disponibili su Mac.

Un semplice clic destro continua ad aprire il menu contestuale dell'applicazione. Restare fermi abbastanza a lungo fa inoltre passare il pulsante per il clic o il trascinamento normale; regola quel tempo in [Preferenze](gestures.md#riconoscimento-e-aspetto).

## Creare un gesto

Apri **Tutte le app → Gesti del mouse → Nuovo gesto**, scegli il pulsante di attivazione e registra il disegno. Clicca **Scegli azione** per cercare o esplorare la [libreria delle azioni](action-library.md), seleziona un’azione predefinita e compila i parametri. Le modifiche vengono salvate automaticamente quando il gesto è valido; provalo fuori dalle impostazioni. Consulta [Gesti del mouse](gestures.md) per i dettagli sulla registrazione.

## Pausa

Su Windows scegli **Metti in pausa i gesti**, poi **Riprendi i gesti** per ripristinarli. Su Mac, **Metti in pausa i gesti** mantiene la stessa etichetta: il segno di spunta indica la pausa manuale e cliccando di nuovo si riprende. Il tasto predefinito per pausa/riprendi è **Win+Esc** su Windows e **Control+Option+Command+P** su Mac. Cambialo sotto **Preferenze → Tasto pausa / riprendi**.

Giochi, video e presentazioni a schermo intero sopprimono automaticamente i gesti mentre la finestra in primo piano viene rilevata a schermo intero. Vedi le [regole per app](per-app.md) per le esclusioni e la [risoluzione dei problemi](troubleshooting.md) se nulla si attiva.
