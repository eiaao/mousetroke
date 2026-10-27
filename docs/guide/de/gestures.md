---
title: "Mausgesten"
description: "Maus-Zeichenfolgen und Formen aufnehmen, Maustasten, Rad und Tastaturtasten kombinieren und die Gestenerkennung justieren."
order: 3
---

# Mausgesten

## Eine gerichtete Zeichenfolge aufnehmen

Öffnen Sie **Alle Apps → Mausgesten → Neue Geste**. Benennen Sie sie und wählen Sie dann unter **Wie sie beginnt**: **Die rechte Maustaste gedrückt halten**, **Die mittlere Maustaste gedrückt halten**, **Die seitliche Taste 1 gedrückt halten** oder **Die seitliche Taste 2 gedrückt halten**. Die seitlichen Tasten entsprechen X1 und X2.

Halten Sie im Zeichenbereich die linke oder rechte Maustaste gedrückt, zeichnen Sie und lassen Sie los. Der Aufnahmebereich akzeptiert beide Tasten unabhängig von dem ausgewählten Auslöser. Gerade Segmente werden zu einer gerichteten Zeichenfolge wie `←` oder `↓→`; zeichnen Sie erneut, um sie zu ersetzen. Klicken Sie auf **Aktion auswählen**, suchen oder stöbern Sie in der [Aktionsbibliothek](action-library.md), wählen Sie eine Voreinstellung und legen Sie die nötigen Parameter fest. Eigene Tastenfolgen, Befehle, Lua oder Menüs finden Sie unter **Erweitert**; siehe [Aktionen](actions.md). Probieren Sie die Geste außerhalb der Einstellungen mit der tatsächlichen Auslösertaste aus.

## Eine Form aufnehmen

Kurven wie C, O oder S sind Formgesten. Die Aufnahme erkennt eine Linie oder Form automatisch. Klicken Sie nötigenfalls nach dem Zeichnen auf **Als Form verwenden** oder **Als Linienzug verwenden**, oder wählen Sie **+ Schritt hinzufügen → Eine Form zeichnen**.

Zeichnen Sie dieselbe Form 2–3 Mal, mit gleichem Startpunkt und gleicher Richtung. Lassen Sie nach jedem Beispiel los. Zum Speichern ist mindestens ein gültiges Beispiel nötig; die Aufnahme behält bis zu fünf Beispiele und kann einzelne Beispiele entfernen. Beachten Sie die Meldungen zu Übereinstimmung und Konflikten. Formgesten werden beim Loslassen der Auslösertaste ausgeführt.

## Maustasten, Rad oder Tasten hinzufügen

Klicken Sie unter **Danach** auf **+ Schritt hinzufügen**. Sie können **Zeichenfolge zeichnen**, **Eine Form zeichnen**, eine Maustaste unter **Mit einer Maustaste klicken**, **Hoch** / **Runter** unter **Mausrad drehen** oder **Taste gedrückt halten** hinzufügen. Die letzte Option zeichnet die Taste auf, die Sie drücken.

Beginnen Sie zum Beispiel mit der rechten Taste und fügen Sie einen Linksklick hinzu, oder fügen Sie Rad nach oben hinzu, um bei gehaltener Auslösertaste wiederholt eine Lautstärkeaktion auszulösen. Schritte sind geordnet: Führen Sie sie in der angezeigten Reihenfolge aus, während Sie die Auslösertaste gedrückt halten. Entfernen Sie einen Schritt über seine ×-Schaltfläche.

Bei gerichteten Gesten führt **Sobald erkannt ausführen** ohne Warten auf das Loslassen aus. Das passt zu wiederholten Radaktionen. Formen warten immer auf das Loslassen. **Gestenname beim Zeichnen nicht anzeigen** blendet den Namen nur für diese Bindung aus.

## Erkennung und Erscheinungsbild

Unter **Einstellungen → Erkennung**:

| Einstellung | Wirkung |
|---|---|
| Wartezeit bis zur Behandlung als Klick | Standard 500 ms; Regler 150–1500 ms. So lange ohne Bewegung halten, um einen normalen Klick bzw. ein normales Ziehen durchzureichen. |
| Empfindlichkeit der Zeichenfolge | Standard 30 px; Regler 10–80 px. Kleinere Werte erkennen kürzere Striche, erleichtern aber versehentliche Gesten. |
| Gesten wirken auf | Wählen Sie **das Fenster unter dem Cursor** (Standard) oder **das aktive Fenster**. |

Unter **Erscheinungsbild** steuern **Spur anzeigen**, **Spurbreite**, **Spurfarbe**, **Bei Treffer**, **Am Ende ausblenden**, **Gestenname beim Zeichnen anzeigen** und **Farbe des Gestennamens** das Feedback. Ein Schwellwertsregler für die Formerkennung steht hier nicht zur Verfügung.

## Suchen, duplizieren und deaktivieren

Drücken Sie **Strg+F** unter Windows oder **⌘+F** auf dem Mac, um die aktuelle Liste nach Name, Zeichenfolge oder Aktion zu durchsuchen. Nutzen Sie **Mausgesten** / **Bildschirmränder** und die Filter nach Taste/Position; **Aus** zeigt deaktivierte Einträge.

Wählen Sie eine Geste und klicken Sie auf **Duplizieren**, um eine Kopie zu erstellen (Strg+D / ⌘+D). Geben Sie der Kopie eine andere Zeichenfolge: Identische Zeichnungen im selben Bereich kollidieren; verlassen Sie sich bei der Auswahl einer Aktion also nicht auf die Reihenfolge in der Liste. Verwenden Sie den Schalter **Ein** / **Aus** des Editors oder die Leertaste in einer ausgewählten Listenzeile, um eine Bindung zu deaktivieren, ohne sie zu löschen. Deaktivierte Bindungen reservieren ihre Auslösertaste nicht mehr von selbst.

Weiter: [Bildschirmränder](screen-edges.md), [App-spezifische Regeln](per-app.md), [Syntax für Tasten senden](key-sequences.md).
