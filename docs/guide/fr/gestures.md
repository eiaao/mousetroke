---
title: "Gestes de souris"
description: "Enregistrez des tracés et des formes à la souris, combinez boutons, molette et touches du clavier, et réglez la reconnaissance des gestes."
order: 3
---

# Gestes de souris

## Enregistrer un tracé directionnel

Ouvrez **Toutes les applications → Gestes de souris → Nouveau geste**. Nommez-le, puis choisissez **Comment ça commence** : **Maintenir le bouton droit enfoncé**, **Maintenir le bouton du milieu enfoncé**, **Maintenir le bouton latéral 1 enfoncé** ou **Maintenir le bouton latéral 2 enfoncé**. Les boutons latéraux correspondent à X1 et X2.

Dans la zone de dessin, maintenez le bouton gauche ou droit, dessinez et relâchez. La zone d’enregistrement accepte l’un ou l’autre bouton, indépendamment du déclencheur choisi. Les segments droits deviennent un tracé directionnel, comme `←` ou `↓→` ; retracez pour le remplacer. Cliquez sur **Choisir une action**, recherchez ou parcourez la [bibliothèque d’actions](action-library.md), puis sélectionnez une action prédéfinie et renseignez les paramètres nécessaires. Utilisez **Avancées** pour les touches, commandes, scripts Lua ou menus personnalisés ; voir [Actions](actions.md). Essayez le geste en dehors des paramètres avec son vrai bouton déclencheur.

## Enregistrer une forme

Les courbes comme C, O ou S sont des gestes de forme. L’enregistreur détecte automatiquement un tracé ou une forme. Si nécessaire, cliquez sur **Utiliser comme forme** ou **Utiliser comme tracé** après avoir dessiné, ou choisissez **+ Ajouter une étape → Dessiner une forme**.

Dessinez la même forme 2 à 3 fois, en gardant le même point de départ et le même sens. Relâchez après chaque exemple. Au moins un exemple valide est nécessaire pour enregistrer ; l’enregistreur conserve jusqu’à cinq exemples et peut retirer des exemples individuels. Suivez ses messages de cohérence et de conflit. Les gestes de forme s’exécutent au relâchement du bouton déclencheur.

## Ajouter des boutons, la molette ou des touches

Sous **Ensuite**, cliquez sur **+ Ajouter une étape**. Vous pouvez ajouter **Tracer un geste**, **Dessiner une forme**, un bouton de la souris sous **Cliquer avec un bouton**, **Haut** / **Bas** sous **Faire tourner la molette**, ou **Maintenir une touche**. Cette dernière option enregistre la touche du clavier sur laquelle vous appuyez.

Par exemple, commencez avec le bouton droit et ajoutez un clic gauche, ou ajoutez un coup de molette vers le haut pour répéter une action de volume en maintenant le déclencheur. Les étapes sont ordonnées : effectuez-les dans l’ordre affiché tout en gardant le bouton déclencheur enfoncé. Retirez une étape avec son bouton ×.

Pour les gestes directionnels, **Exécuter dès la reconnaissance** s’exécute sans attendre le relâchement. Cela convient aux actions de molette répétées. Les formes attendent toujours le relâchement. **Ne pas afficher le nom du geste pendant le tracé** masque le nom pour cette liaison.

## Reconnaissance et apparence

Sous **Préférences → Reconnaissance** :

| Réglage | Effet |
|---|---|
| Durée d’immobilité avant de traiter en clic | 500 ms par défaut ; curseur de 150 à 1500 ms. Restez sans bouger pendant cette durée pour laisser passer un clic ou un glisser normal. |
| Sensibilité du tracé | 30 px par défaut ; curseur de 10 à 80 px. Des valeurs plus faibles reconnaissent les tracés plus courts mais rendent les gestes accidentels plus faciles à déclencher. |
| Les gestes agissent sur | Choisissez **la fenêtre sous le curseur** (par défaut) ou **la fenêtre active**. |

Sous **Apparence**, **Afficher la traînée**, **Épaisseur de la traînée**, **Couleur de la traînée**, **En cas de succès**, **Estomper à la fin**, **Afficher le nom du geste pendant le tracé** et **Couleur du nom** contrôlent le retour visuel. Il n’y a pas de curseur de seuil de reconnaissance des formes ici.

## Rechercher, copier et désactiver

Appuyez sur **Ctrl+F** sous Windows ou **⌘+F** sur Mac pour rechercher dans la liste actuelle par nom, tracé ou action. Utilisez **Gestes de souris** / **Bords de l’écran** et les filtres par bouton/position ; **Désactivé** affiche les entrées désactivées.

Sélectionnez un geste et cliquez sur **Dupliquer** pour en faire une copie (Ctrl+D / ⌘+D). Donnez à la copie un tracé différent : des dessins identiques dans une même portée entrent en conflit, ne comptez donc pas sur leur ordre dans la liste pour choisir une action. Utilisez l’interrupteur **Activé** / **Désactivé** de l’éditeur, ou la barre d’espace sur une ligne sélectionnée de la liste, pour désactiver un geste sans le supprimer. Les liaisons désactivées ne réservent plus leur déclencheur par elles-mêmes.

Suivant : [bords de l’écran](screen-edges.md), [règles par application](per-app.md), [syntaxe d’envoi de touches](key-sequences.md).
