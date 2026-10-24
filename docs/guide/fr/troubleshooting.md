---
title: "Dépannage"
description: "Corrigez les gestes de souris qui ne se déclenchent pas, les autorisations macOS, les raccourcis manquants, la saisie IME et les erreurs Lua."
order: 12
---

# Dépannage

## Les gestes ne se déclenchent pas

Testez d’abord un geste intégré simple dans une fenêtre de navigateur normale, non plein écran.

- Vérifiez le menu de la zone de notification/barre des menus. Sous Windows, sélectionnez **Reprendre les gestes** si c’est proposé ; sur Mac, cliquez à nouveau sur **Mettre les gestes en pause** si c’est coché. Vérifiez **Préférences → Raccourci pause / reprise** si un raccourci a mis le moteur en pause.
- Quittez le plein écran. La suspension automatique en plein écran peut arrêter aussi bien les gestes de souris que les actions de bord, même quand la pause manuelle est désactivée.
- Sélectionnez l’application dans les paramètres et vérifiez **Désactiver les gestes dans cette appli**. Vérifiez aussi l’interrupteur **Activé** / **Désactivé** de la liaison concernée, **Utiliser aussi les gestes globaux**, le bouton déclencheur et les éventuelles étapes supplémentaires.
- Bougez rapidement après avoir enfoncé le bouton et tracez un tracé plus long. Réglez **Durée d’immobilité avant de traiter en clic** et **Sensibilité du tracé** dans les Préférences.
- Si le retour visuel manque, vérifiez **Afficher la traînée** et **Afficher le nom du geste pendant le tracé**. Une traînée masquée ne signifie pas en soi que la reconnaissance est désactivée.

Sous Windows, l’entrée utilise `SendInput`. Une instance lancée normalement peut être incapable d’envoyer des touches/clics à une application élevée à cause de l’UIPI de Windows. Testez d’abord sur une application non élevée ; si vous devez contrôler une fenêtre d’administrateur, lancez Mousetroke avec l’élévation correspondante. Cela n’active pas l’envoi vers le bureau sécurisé.

Sur Mac, vérifiez l’**Accessibilité** pour la copie installée. Si l’assistant demande encore la **Surveillance de l’entrée**, accordez-la aussi. Les changements d’autorisation peuvent prendre quelques secondes à être détectés. Utilisez **Redémarrer Mousetroke** quand c’est proposé si l’entrée ne démarre toujours pas. L’Enregistrement de l’écran n’est pas nécessaire. Déplacer ou remplacer une copie via le navigateur peut de nouveau exiger une approbation de première ouverture.

## SmartScreen ou Gatekeeper bloque la première ouverture

Sous Windows : récupérez l’exécutable officiel depuis la page des versions, vérifiez la somme de contrôle, puis utilisez **Informations supplémentaires → Exécuter quand même**. Sur Mac : installez dans Applications, puis utilisez **Réglages Système → Confidentialité et sécurité → Ouvrir quand même** ; les versions antérieures de macOS prennent aussi en charge **clic droit → Ouvrir**. Voir l’[installation](getting-started.md) pour la séquence complète.

## Une application ne reçoit pas les touches

Essayez le raccourci sur le clavier physique. Vérifiez si l’application est active, ou activez **Activer d’abord la fenêtre sous le curseur**. Confirmez la préférence de cible des gestes et utilisez la [syntaxe correcte](key-sequences.md) : `^` est Control, tandis que `#` est Command sur Mac. Séparez les étapes dépendantes par un court `{SLEEP 100}` si l’application a besoin de temps pour ouvrir un menu ou une boîte de dialogue.

Les séquences de touches Mac sautent les jetons souris/molette, les jetons de touches média/volume, Insert, Print Screen, Break et F21–F24. Utilisez **Média et système** pour les actions média/volume. Les options non prises en charge sont marquées dans l’éditeur ; consultez aussi le journal.

Ctrl+Alt+Suppr est une séquence d’attention sécurisée de Windows et ne peut pas être envoyé comme entrée synthétique ordinaire. Utilisez Ctrl+Shift+Esc pour le Gestionnaire des tâches, Win+L pour verrouiller, ou `shutdown /l` pour se déconnecter. Les détails sont dans la [syntaxe d’envoi de touches](key-sequences.md#pourquoi-ctrlaltsuppr-ne-peut-pas-être-envoyé).

## La saisie IME ou la sortie de texte est incorrecte

Les caractères ordinaires d’envoi de touches représentent les positions du clavier américain, pas du texte Unicode arbitraire. Les majuscules seules n’ajoutent pas Shift. En Lua, `ms.keys(text, true)` émet du texte Unicode mais interprète encore les caractères de syntaxe comme `^` et `{`.

Pour du texte riche en ponctuation, du texte non latin ou des applications à la gestion IME inhabituelle, utilisez la [recette de collage-restauration du presse-papiers](lua-examples.md#coller-puis-restaurer-le-texte-du-presse-papiers). La restauration avec ces API ne conserve que le texte, pas les images ni les autres formats du presse-papiers. Certaines applications ont besoin d’un délai plus long avant la restauration du presse-papiers.

## Erreurs Lua

Assurez-vous que le code définit `function ms_action(...) ... end`. Le moteur d’exécution écrit les erreurs de chargement et d’exécution dans le [journal d’exécution](config.md#emplacements-des-fichiers). Ouvrez-le via **À propos → Ouvrir le journal**. Utilisez `ms.log` pour les valeurs et `ms.show_toast` pour un point de contrôle bref à l’écran. Une fonction non prise en charge sur Mac peut seulement journaliser un avertissement et renvoyer `nil` ; ne prenez pas une valeur manquante pour un succès.

Les appels de découverte/d’état de fenêtre sur Mac exigent l’autorisation d’Accessibilité ; les requêtes d’état des touches renvoient false sans elle. Vérifiez d’abord cette autorisation si des résultats manquent. Le premier plan entre applications et la luminosité numérique restent non pris en charge ; les actions intégrées de touches de luminosité ne fournissent pas de pourcentage. Sur les deux plateformes, `ms.get_brightness` / `ms.set_brightness` renvoient nil en cas d’échec.

La limite d’allocation Lua est de 8 Mio. Il n’y a pas de délai d’exécution de script ; évitez les boucles infinies et voir les [bases de Lua](lua-basics.md#exécution-et-limites) pour le comportement bloquant.

## Bureau à distance et jeux

Le bureau à distance, les machines virtuelles et les applications 3D/jeux peuvent avoir besoin des mêmes boutons de la souris pour glisser ou gérer l’entrée synthétique différemment. Ajoutez une [exclusion par application](per-app.md) ou mettez en pause pendant leur utilisation. Les jeux en fenêtre peuvent nécessiter une exclusion explicite parce que la détection du plein écran ne s’applique pas. Testez les raccourcis dans une application locale ordinaire pour distinguer un problème de séquence de touches de la gestion de l’entrée par l’application.

## Signaler un bug

Utilisez **À propos → Retours et suggestions → Ouvrir** ou les [GitHub Issues](https://github.com/eiaao/mousetroke/issues). Incluez la version de l’application, la version de l’OS, l’application concernée, le geste et l’action, le résultat attendu et les étapes de reproduction. Joignez le journal pertinent depuis **À propos → Ouvrir le journal** et une configuration exportée minimale si utile. Vérifiez les pièces jointes pour du texte privé, des titres de fenêtres, des chemins ou des arguments de commande avant de les partager.
