---
title: "Premiers pas"
description: "Installez Mousetroke sous Windows ou sur Mac, tracez votre premier geste de souris, ouvrez les paramètres et mettez les gestes en pause."
order: 2
---

# Premiers pas

## Installer et ouvrir

Sous Windows, téléchargez `mousetroke.exe` depuis la [dernière version](https://github.com/eiaao/mousetroke/releases/latest). C’est un exécutable portable : enregistrez-le à un emplacement où vous souhaitez le conserver et double-cliquez dessus. Comparez le SHA-256 du téléchargement avec `SHA256SUMS.txt` sur la page de cette version.

Windows 10 1809 ou ultérieur et Windows 11, 64 bits, sont pris en charge. Les paramètres utilisent Microsoft Edge WebView2, inclus dans Windows 11. L’exécutable n’est pas encore signé numériquement. Si SmartScreen bloque le premier lancement, choisissez **Informations supplémentaires → Exécuter quand même** après avoir vérifié que vous avez téléchargé la version officielle.

Sur Mac, téléchargez `Mousetroke.dmg`, ouvrez-le et glissez **Mousetroke** dans **Applications**. L’application universelle prend en charge Apple silicon et Intel sur macOS 11 ou ultérieur. Si elle est ouverte depuis le DMG, l’application propose **Déplacer dans Applications** et rouvre la copie installée.

La version bêta Mac est auto-signée et non notarisée. Si elle est bloquée, utilisez **Réglages Système → Confidentialité et sécurité → Ouvrir quand même** ; sur les versions antérieures de macOS, utilisez **clic droit → Ouvrir**. Dans l’assistant de configuration, cliquez sur **Autoriser…** et activez Mousetroke sous **Accessibilité**. Attendez **Tout est prêt — vous pouvez utiliser les gestes**. L’autorisation d’**Enregistrement de l’écran** n’est pas nécessaire. L’assistant propose la **Surveillance de l’entrée** si l’écoute échoue encore sur les systèmes qui en ont besoin ; activez-la quand elle est demandée.

## Ouvrir les paramètres et terminer le guide

Sous Windows, cliquez avec le bouton gauche sur l’icône de la zone de notification pour ouvrir les paramètres. Cliquez avec le bouton droit pour **Paramètres…**, **Mettre les gestes en pause** / **Reprendre les gestes**, **Lancer automatiquement**, **Ouvrir le dossier de configuration** et **Quitter**. Un élément de nouvelle version peut aussi apparaître lorsqu’une mise à jour est disponible.

Sur Mac, utilisez l’icône de la barre des menus pour ouvrir **Paramètres…**, mettre en pause / reprendre ou **Quitter**. Les éléments d’autorisation et de mise à jour apparaissent le cas échéant. **À propos de Mousetroke** ouvre la page À propos.

Le guide de premier lancement explique le tracé, les bords de l’écran, les exclusions d’applications et la pause. Vérifiez les applications cochées avant de terminer : les gestes y seront désactivés. Vous pouvez le rouvrir avec **À propos → Revoir le guide de démarrage**. Les modifications dans les paramètres s’enregistrent automatiquement ; un tracé incomplet doit être terminé avant de pouvoir enregistrer.

## Tracer votre premier geste

1. Ouvrez une page de navigateur avec un historique de navigation.
2. Maintenez le bouton droit de la souris, déplacez vers la gauche pour tracer `←`, puis relâchez. Cela revient en arrière.
3. Essayez `↓→` sur un onglet jetable pour le fermer, ou `↙` pour réduire la fenêtre.

Les actions intégrées s’adaptent à la plateforme : les raccourcis du navigateur utilisent Ctrl sous Windows et Command sur Mac quand c’est approprié. La liste complète modifiable se trouve sous **Toutes les applications → Gestes de souris**. Certaines actions intégrées sont désactivées, en particulier celles indisponibles sur Mac.

Un simple clic droit ouvre toujours le menu contextuel de l’application. Rester immobile assez longtemps transmet aussi le bouton pour un clic ou un glisser normaux ; ajustez cette durée dans les [Préférences](gestures.md#reconnaissance-et-apparence).

## Pause

Sous Windows, choisissez **Mettre les gestes en pause**, puis **Reprendre les gestes** pour les rétablir. Sur Mac, l’élément garde le même libellé : sa coche indique une pause manuelle, et cliquer à nouveau dessus reprend les gestes. Le raccourci de pause/reprise par défaut est **Win+Esc** sous Windows et **Control+Option+Command+P** sur Mac. Modifiez-le sous **Préférences → Raccourci pause / reprise**.

Les jeux, vidéos et présentations en plein écran suspendent automatiquement les gestes lorsque la fenêtre au premier plan est détectée en plein écran. Consultez les [règles par application](per-app.md) pour les exclusions et le [dépannage](troubleshooting.md) si rien ne se déclenche.
