---
title: "Fehlerbehebung"
description: "Mausgesten beheben, die nicht auslösen, macOS-Berechtigungen, fehlende Kürzel, IME-Texteingabe und Lua-Fehler."
order: 12
---

# Fehlerbehebung

## Gesten lösen nicht aus

Testen Sie zuerst eine einfache integrierte Geste in einem normalen, nicht-vollbildigen Browserfenster.

- Prüfen Sie das Menü im Infobereich bzw. in der Menüleiste. Wählen Sie unter Windows **Gesten fortsetzen**, falls angeboten; klicken Sie auf dem Mac erneut auf **Gesten pausieren**, falls es abgehakt ist. Prüfen Sie **Einstellungen → Kürzel für Pause / Fortsetzen**, falls ein Kürzel die Engine pausiert hat.
- Verlassen Sie das Vollbild. Die automatische Vollbild-Unterdrückung kann Mausgesten und Randaktionen stoppen, selbst wenn die manuelle Pause aus ist.
- Wählen Sie die App in den Einstellungen und prüfen Sie **Gesten in dieser App deaktivieren**. Prüfen Sie außerdem den Schalter **Ein** / **Aus** der einzelnen Bindung, **Auch globale Gesten verwenden**, die Auslösertaste und zusätzliche Schritte.
- Bewegen Sie sich zügig, nachdem Sie die Taste gedrückt haben, und zeichnen Sie eine längere Zeichenfolge. Justieren Sie **Wartezeit bis zur Behandlung als Klick** und **Empfindlichkeit der Zeichenfolge** in den Einstellungen.
- Fehlt das Feedback, prüfen Sie **Spur anzeigen** und **Gestenname beim Zeichnen anzeigen**. Eine ausgeblendete Spur bedeutet nicht von selbst, dass die Erkennung aus ist.

Unter Windows verwendet die Eingabe `SendInput`. Eine normal gestartete Instanz kann unter Umständen wegen Windows UIPI keine Tasten/Klicks an eine App mit erhöhten Rechten senden. Testen Sie zuerst gegen eine App ohne Rechteerhöhung; wenn Sie ein Administrator-Fenster steuern müssen, starten Sie Mousetroke mit der entsprechenden Rechteerhöhung. Das ermöglicht kein Senden an den sicheren Desktop.

Auf dem Mac prüfen Sie **Bedienungshilfen** für die installierte Kopie. Bittet die Einrichtung weiterhin um die **Eingabeüberwachung**, erlauben Sie auch diese. Berechtigungsänderungen brauchen einige Sekunden, bis sie erkannt werden. Verwenden Sie **Mousetroke neu starten**, falls die Eingabe weiterhin nicht startet und das Angebot erscheint. Bildschirmaufnahme ist nicht erforderlich. Verschieben oder Ersetzen einer Kopie über den Browser kann eine erneute Freigabe beim ersten Öffnen erfordern.

## SmartScreen oder Gatekeeper blockiert den ersten Start

Windows: Holen Sie die offizielle ausführbare Datei von der Veröffentlichungsseite, prüfen Sie die Prüfsumme und verwenden Sie dann **Weitere Informationen → Trotzdem ausführen**. Mac: Installieren Sie die App in Programme und verwenden Sie dann **Systemeinstellungen → Datenschutz & Sicherheit → Trotzdem öffnen**; ältere macOS-Versionen unterstützen auch **Rechtsklick → Öffnen**. Siehe [Installation](getting-started.md) für die vollständige Abfolge.

## Eine Anwendung empfängt keine Tasten

Probieren Sie das Kürzel auf der physischen Tastatur. Prüfen Sie, ob die Anwendung aktiv ist, oder aktivieren Sie **Zuerst das Fenster unter dem Cursor aktivieren**. Bestätigen Sie die Einstellung des Gestenziels und verwenden Sie die [korrekte Syntax](key-sequences.md): `^` ist Strg, während `#` auf dem Mac Command ist. Trennen Sie voneinander abhängige Schritte mit einem kurzen `{SLEEP 100}`, wenn die App Zeit braucht, um ein Menü oder einen Dialog zu öffnen.

Mac-Tastenfolgen überspringen Maus-/Rad-Zeichen, Medien-/Lautstärketasten-Zeichen, Insert, Print Screen, Break und F21–F24. Verwenden Sie **Medien und System** für Medien-/Lautstärkeaktionen. Nicht unterstützte Optionen sind im Editor markiert; sehen Sie auch im Protokoll nach.

Strg+Alt+Entf ist eine Secure-Attention-Sequence von Windows und lässt sich nicht als gewöhnliche synthetische Eingabe senden. Verwenden Sie Strg+Umschalt+Esc für den Task-Manager, Win+L zum Sperren oder `shutdown /l` zum Abmelden. Details stehen in [Syntax für Tasten senden](key-sequences.md#warum-strgaltentf-nicht-gesendet-werden-kann).

## IME oder Textausgabe ist falsch

Gewöhnliche Zeichen in Tastenfolgen stehen für US-Tastaturpositionen, nicht für beliebigen Unicode-Text. Großbuchstaben allein fügen kein Shift hinzu. Lua `ms.keys(text, true)` gibt Unicode-Text aus, interpretiert aber weiterhin Syntaxzeichen wie `^` und `{`.

Verwenden Sie für zeichenreichen Text, nicht-lateinischen Text oder Apps mit ungewöhnlicher IME-Behandlung das [Rezept zum Einfügen und Wiederherstellen über die Zwischenablage](lua-examples.md#zwischenablagentext-einfügen-und-wiederherstellen). Das Wiederherstellen mit diesen APIs bewahrt nur Text, keine Bilder oder anderen Zwischenablagenformate. Manche Apps brauchen eine längere Verzögerung, bevor die Zwischenablage wiederhergestellt wird.

## Lua-Fehler

Stellen Sie sicher, dass der Code `function ms_action(...) ... end` definiert. Die Laufzeit schreibt Lade- und Ausführungsfehler in das [Laufzeitprotokoll](config.md#dateispeicherorte). Öffnen Sie es über **Über → Protokoll öffnen**. Verwenden Sie `ms.log` für Werte und `ms.show_toast` als kurze Sichtprüfung auf dem Bildschirm. Eine nicht unterstützte Mac-Funktion protokolliert eventuell nur eine Warnung und liefert `nil`; behandeln Sie einen fehlenden Wert nicht als Erfolg.

Mac-Aufrufe zum Auffinden/Abfragen von Fenstern benötigen die Bedienungshilfen-Berechtigung; Tastenzustandsabfragen liefern ohne sie false. Prüfen Sie diese Berechtigung zuerst, wenn Ergebnisse fehlen. Das app-übergreifende „immer im Vordergrund“ und numerische Helligkeit bleiben nicht unterstützt; die integrierten Helligkeitstasten-Aktionen liefern keinen Prozentsatz. Auf beiden Plattformen liefern `ms.get_brightness` / `ms.set_brightness` bei Fehler nil.

Das Lua-Speicherlimit beträgt 8 MiB. Es gibt kein Ausführungs-Timeout für Skripte; vermeiden Sie Endlosschleifen und siehe [Lua-Grundlagen](lua-basics.md#laufzeit-und-grenzen) zum blockierenden Verhalten.

## Remotedesktop und Spiele

Remotedesktop, virtuelle Maschinen und 3D-/Spielanwendungen können dieselben Maustasten für das Ziehen brauchen oder synthetische Eingaben anders behandeln. Fügen Sie einen [App-Ausschluss](per-app.md) hinzu oder pausieren Sie, während Sie sie verwenden. Fenster-Spiele brauchen eventuell eine explizite Ausnahme, weil die Vollbilderkennung nicht greift. Testen Sie Kürzel in einer gewöhnlichen lokalen App, um ein Tastenfolgen-Problem von der Eingabeverarbeitung der Anwendung zu trennen.

## Einen Fehler melden

Verwenden Sie **Über → Feedback → Öffnen** oder [GitHub Issues](https://github.com/eiaao/mousetroke/issues). Geben Sie die App-Version, die Betriebssystemversion, die betroffene Anwendung, Geste und Aktion, das erwartete Ergebnis und die Schritte zum Reproduzieren an. Hängen Sie bei Bedarf das einschlägige Protokoll aus **Über → Protokoll öffnen** und eine minimale exportierte Konfiguration an. Prüfen Sie Anhänge vor dem Teilen auf privaten Text, Fenstertitel, Pfade oder Befehlsargumente.
