---
title: "Angoli e bordi dello schermo"
description: "Configura gli angoli attivi, lo scorrimento sui bordi per volume e luminosità, i clic sui bordi e i gesti di sfregamento su Windows e Mac."
order: 4
---

# Angoli e bordi dello schermo

## Provare le impostazioni predefinite

Sotto **Tutte le app → Bordi dello schermo**, ogni associazione di angolo o bordo è modificabile.

| Attivazione | Predefinito Windows | Predefinito Mac |
|---|---|---|
| Spingere nell'angolo in basso a sinistra | Menu Start | Spotlight |
| Spingere nell'angolo in basso a destra | Mostra desktop | Mostra desktop (F11) |
| Spingere nell'angolo in alto a sinistra | Cambia app (Alt+Tab) | Mission Control (Control+Up) |
| Spingere nell'angolo in alto a destra | Visualizzazione attività (Win+Tab) | Associazione disattivata inizialmente |
| Scorrere sul bordo superiore | Alza/abbassa il volume | Alza/abbassa il volume |
| Clic centrale sul bordo superiore | Silenzia | Silenzia |
| Clic destro sul bordo superiore | Riproduci / pausa | Riproduci / pausa |
| Scorrere sul bordo inferiore | Aumenta/riduci luminosità | Le associazioni per la luminosità sono disattivate inizialmente |
| Sfregare su/giù il bordo destro due volte | Associazione Gestione attività; abilita prima Esegui appena riconosciuto | Monitoraggio attività |

Le azioni degli angoli non richiedono pulsanti: spingi il puntatore nell'angolo. Scorrimento e clic sui bordi richiedono che il puntatore tocchi il bordo scelto. Sfregamento significa muoversi lungo il bordo nelle direzioni registrate, invece di girare la rotella.

## Configurare un attivatore di bordo

1. Scegli **Bordi dello schermo → Nuovo bordo** nell'ambito dell'app desiderato o in **Tutte le app**.
2. Sotto **Come inizia**, scegli **Bordo dello schermo** e seleziona un lato o un angolo nel diagramma.
3. Lascia vuoto **Poi** per un'azione d'angolo, oppure usa **+ Aggiungi un passaggio** per aggiungere rotella su/giù, un clic del mouse o un tracciato lungo il bordo.
4. Scegli un'[azione](actions.md). Per il volume, scegli **Media e sistema → Alza il volume** o **Abbassa il volume**. Attiva **Esegui appena riconosciuto** per lo scorrimento ripetuto o per lo sfregamento senza pulsanti.

Per lo sfregamento, aggiungi **Disegnare un tracciato** e registra le direzioni alternate. Un esempio utile è `↑↓↑↓` contro il bordo destro. Prova prima con un'azione innocua, come un toast Lua, prima di assegnare qualcosa di invasivo.

## Luminosità e differenze tra piattaforme

Su Windows, **Aumenta luminosità** / **Riduci luminosità** parte dal monitor sotto il cursore, regola di 10 punti percentuali e mostra il risultato. Prova prima DDC/CI, poi WMI. I display esterni devono supportare e consentire DDC/CI. Il fallback WMI usa il primo dispositivo di luminosità WMI, di norma un pannello integrato; non corrisponde al monitor del cursore. Se entrambe le strade falliscono, compare un messaggio a schermo. Vedi le [ricette Lua](lua-examples.md#luminosità-sotto-il-cursore) per un passo più piccolo.

Su Mac, **Media e sistema → Aumenta luminosità** / **Riduci luminosità** invia gli eventi dei tasti di luminosità di sistema per il display integrato. Non seleziona lo schermo sotto il cursore e non supporta i display DDC/CI esterni. Le associazioni predefinite del bordo inferiore sono disattivate. Anche le API Lua per la luminosità non sono supportate su Mac.

Su Mac, Mousetroke avvisa se un angolo attivo di sistema si sovrappone a un angolo configurato. Cambia una delle due assegnazioni per evitare azioni in competizione. Le scorciatoie di sistema come F11 o Control+Up dipendono dalle impostazioni delle scorciatoie di macOS.

Anche le esclusioni delle app e il rilevamento a schermo intero sopprimono gli attivatori di bordo. Vedi le [regole per app](per-app.md) quando un bordo funziona in un'app ma non in un'altra.
