---
title: "Gestenbibliothek"
description: "Eine Geste einmal festlegen und in mehreren Apps mit unterschiedlichen Aktionen nutzen: vorhandene Gesten ordnen und übernehmen, anderen Apps zuweisen, gemeinsame Gesten bearbeiten, Hinweise zur Kompatibilität."
order: 3.5
---

# Gestenbibliothek

Die Gestenbibliothek trennt, wie Sie zeichnen, von dem, was passiert. Legen Sie eine Geste einmal fest und geben Sie ihr in jeder App eine eigene Aktion. Wenn Sie eine gemeinsame Geste neu aufnehmen, ändert sich jede Stelle, die sie nutzt; eine Formgeste behält nur einen Satz Beispiele.

## Bibliothek durchsuchen

Klicken Sie in der Seitenleiste auf **Gesten**. Die Liste ist in **Mausgesten** und **Bildschirmränder** geteilt. Filtern Sie nach Auslösetaste, finden Sie mit **Unbenutzt** ungenutzte Gesten oder suchen Sie nach Name, Zeichnung oder Verwendung. Jede Karte zeigt, an wie vielen Stellen die Geste genutzt wird.

Ein Klick auf eine Karte öffnet die Details. Oben können Sie neu zeichnen; **Verwendet in** listet jede App mit Gestenname und Aktion auf, ein Klick springt dorthin. Bei einer Neuinstallation sind die eingebauten Gesten schon in der Bibliothek. Nach dem Update von einer älteren Version sind Ihre Gesten noch auf die Apps verteilt – ordnen Sie sie zuerst wie unten beschrieben.

## Ordnen und übernehmen

Ist die Bibliothek leer, klicken Sie auf **Gesten ordnen und importieren…**; später jederzeit oben rechts auf **Ordnen und importieren…**. Wählen Sie zuerst den Umfang:

- **Nur allgemeine Gesten (empfohlen)**: übernimmt die Gesten aus **Alle Apps** und verknüpft gleiche oder ähnliche Gesten in anderen Apps. Gesten, die nur für eine App eingerichtet sind, bleiben app-spezifisch.
- **Alle Gesten**: übernimmt gemeinsame und alle app-spezifischen Gesten.

Mit **Weiter** prüfen Sie das Ergebnis. Erst Ihre Bestätigung ändert etwas:

- Identische Zeichnungen werden automatisch zusammengeführt. Striche mit gleichen Richtungen, die sich nur in der Segmentlänge unterscheiden, werden standardmäßig zusammengeführt.
- **Ähnliche Formen – bitte bestätigen**: Wählen Sie je Gruppe **Diese verwenden** oder **Beispiele zusammenführen**. Gruppen unter der empfohlenen Ähnlichkeit werden standardmäßig nicht zusammengeführt; eine zusammengeführte Form kann etwas anders erkannt werden.
- **Direkt übernommen**: Aufklappen und Gesten abwählen. Abgewählte Gesten werden nicht übernommen und bleiben app-spezifisch.

Mit **Importieren** abschließen; der Hinweis bietet Rückgängig an. Zwei ähnliche Gesten in derselben App mit unterschiedlichen Aufgaben werden nie zusammengeführt.

## Geste in einer anderen App nutzen

Klicken Sie in den Gestendetails auf **In einer anderen App verwenden…**, wählen Sie eine App und dann eine Aktion. Apps, die die Geste schon nutzen, sind nicht wählbar; hat eine App bereits dieselbe Zeichnung, erscheint eine Konfliktwarnung; Apps mit deaktivierten Gesten lassen sich zuweisen, die Geste wirkt dort aber nicht. Mit **App hinzufügen…** am Listenende fügen Sie zuerst eine App hinzu. Danach bleiben Sie in den Gestendetails, die neue Zeile ist hervorgehoben, und **Bearbeiten** im Hinweis öffnet diese Zuordnung.

Sie können auch bei der App beginnen. Im Editor zeigt eine Geste, die nur zur aktuellen App gehört, **Nur in dieser App**:

- **Aus Bibliothek wählen** wechselt zu einer Bibliotheksgeste. Die Auswahl ist durchsuchbar; eine Geste aus **Gesten in anderen Apps** wird zuerst in die Bibliothek aufgenommen und mit dem Original geteilt; **Neue Geste aufnehmen (nur in dieser App)** landet nicht in der Bibliothek.
- Das Menü **Weitere Aktionen** enthält **Zur Bibliothek hinzufügen** und **In einer anderen App verwenden…**.

## Gemeinsame Geste bearbeiten

Im Editor zeigt eine Geste aus der Bibliothek **Bibliothek** und ihre Verwendungsstellen. Zeichnen Sie sie neu, gilt die Änderung überall, und neue Konflikte werden aufgeführt:

- **Nur hier ändern**: Die Änderung gilt nur für die aktuelle App; die Geste wird app-spezifisch, Bibliothek und andere Apps behalten die alte Zeichnung.
- **Fertig**: Behält die gemeinsame Änderung und schließt den Hinweis.

**Ändern** wechselt zu einer anderen Bibliotheksgeste; die Schaltfläche ↗ (**In Bibliothek anzeigen**) öffnet die Gestendetails.

## Erstellen und löschen

**Neue Geste** nimmt eine Geste direkt in der Bibliothek auf. Sie wird hinzugefügt, sobald Sie fertig gezeichnet haben; der Name ist optional. Zuweisen können Sie sie später mit **In einer anderen App verwenden…**.

Wählen Sie **Geste löschen**, während die Geste noch genutzt wird, entscheiden Sie sich für:

- **Löschen; jede Verwendung behält eine eigene Kopie**
- **Mit allen Verwendungen löschen**

Beides lässt sich im Hinweis rückgängig machen.

## Konfiguration und Kompatibilität

Die Bibliothek steht in der Konfigurationsdatei unter `gestures`; jede Geste, die sie nutzt, speichert zusätzlich eine vollständige Kopie ihrer Zeichnung. Ältere Mousetroke-Versionen können die neue Konfiguration daher lesen, und die Gesten funktionieren weiter. Speichern Sie jedoch Einstellungen in einer älteren Version, geht die Bibliothek verloren: Jede Geste wird wieder app-spezifisch, und Sie können nach dem Update erneut ordnen. Weicht eine Zeichnung nach manueller Bearbeitung der Datei von der Bibliothek ab, wird diese Stelle app-spezifisch, und Sie erhalten einmal einen Hinweis. Dateiorte und Sicherungen: [Konfiguration, Backup und Updates](config.md).

Weiter: [Mausgesten](gestures.md), [App-spezifische Regeln](per-app.md), [Aktionsbibliothek](action-library.md).
