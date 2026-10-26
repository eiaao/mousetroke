---
title: "Gesti del mouse"
description: "Registra tracciati e forme del mouse, combina pulsanti, rotella e tasti della tastiera e regola il riconoscimento dei gesti."
order: 3
---

# Gesti del mouse

## Registrare un tracciato direzionale

Apri **Tutte le app → Gesti del mouse → Nuovo gesto**. Dagli un nome, poi scegli **Come inizia**: **Tenere premuto il pulsante destro**, **Tenere premuto il pulsante centrale**, **Tenere premuto il pulsante laterale 1** o **Tenere premuto il pulsante laterale 2**. I pulsanti laterali corrispondono a X1 e X2.

Nell'area di disegno, tieni premuto il pulsante sinistro o destro, disegna e rilascia. L'area di registrazione accetta entrambi i pulsanti, indipendentemente dall'attivatore scelto. I segmenti retti diventano un tracciato direzionale, come `←` o `↓→`; ridisegna per sostituirlo. Clicca **Scegli azione**, cerca o esplora la [libreria delle azioni](action-library.md), poi seleziona un’azione predefinita e compila i parametri necessari. Usa **Avanzate** per tasti, comandi, Lua o menu personalizzati; consulta [Azioni](actions.md). Prova il gesto fuori dalle impostazioni con il pulsante di attivazione effettivo.

## Registrare una forma

Le curve come C, O o S sono gesti a forma. Il registratore rileva automaticamente una linea o una forma. Se serve, clicca **Usare come forma** o **Usare come tracciato** dopo aver disegnato, oppure scegli **+ Aggiungi un passaggio → Disegnare una forma**.

Disegna la stessa forma 2–3 volte, mantenendo lo stesso punto di partenza e la stessa direzione. Rilascia dopo ogni campione. Serve almeno un campione valido per salvare; il registratore conserva fino a cinque campioni e può rimuovere i singoli campioni. Fai attenzione ai suoi messaggi su coerenza e conflitti. I gesti a forma vengono eseguiti quando rilasci il pulsante di attivazione.

## Aggiungere pulsanti, rotella o tasti

Sotto **Poi**, clicca **+ Aggiungi un passaggio**. Puoi aggiungere **Disegnare un tracciato**, **Disegnare una forma**, un pulsante del mouse sotto **Cliccare con un pulsante del mouse**, **Su** / **Giù** sotto **Girare la rotella**, oppure **Tenere premuto un tasto**. L'ultima opzione registra il tasto della tastiera che premi.

Per esempio, parti dal pulsante destro e aggiungi un clic sinistro, oppure aggiungi la rotella verso l'alto per eseguire ripetutamente un'azione sul volume mentre tieni premuto l'attivatore. I passaggi sono ordinati: eseguili nell'ordine mostrato mantenendo premuto il pulsante di attivazione. Rimuovi un passaggio con il suo pulsante ×.

Per i gesti direzionali, **Esegui appena riconosciuto** esegue senza attendere il rilascio. È adatto alle azioni ripetute della rotella. Le forme attendono sempre il rilascio. **Non mostrare il nome del gesto mentre disegni** nasconde il nome per quell'associazione.

## Riconoscimento e aspetto

Sotto **Preferenze → Riconoscimento**:

| Impostazione | Effetto |
|---|---|
| Quanto restare fermo perché diventi un clic | Predefinito 500 ms; cursore 150–1500 ms. Restare fermi per questo tempo senza muoversi fa passare un normale clic/trascinamento. |
| Sensibilità del tracciato | Predefinito 30 px; cursore 10–80 px. Valori più bassi riconoscono tracciati più corti ma rendono più facili i gesti involontari. |
| I gesti agiscono su | Scegli **la finestra sotto il cursore** (predefinito) o **la finestra attiva**. |

Sotto **Aspetto**, **Mostra la scia**, **Spessore della scia**, **Colore della scia**, **Al riconoscimento**, **Sfuma al termine**, **Mostra il nome del gesto mentre disegni** e **Colore del nome** controllano il feedback. Qui non c'è un cursore per la soglia di riconoscimento delle forme.

## Trovare, copiare e disattivare

Premi **Ctrl+F** su Windows o **⌘+F** su Mac per cercare nell'elenco attuale per nome, tracciato o azione. Usa **Gesti del mouse** / **Bordi dello schermo** e i filtri per pulsante/posizione; **Disattivati** mostra le voci disattivate.

Seleziona un gesto e clicca **Duplica** per farne una copia (Ctrl+D / ⌘+D). Dai alla copia un tracciato diverso: disegni identici nello stesso ambito vanno in conflitto, quindi non contare sull'ordine dell'elenco per scegliere un'azione. Usa l'interruttore **Attivo** / **Disattivato** dell'editor, oppure Spazio su una riga selezionata dell'elenco, per disattivare un gesto senza eliminarlo. Le associazioni disattivate non riservano più da sole il loro attivatore.

Successivo: [bordi dello schermo](screen-edges.md), [sostituzioni per app](per-app.md), [sintassi di invio dei tasti](key-sequences.md).
