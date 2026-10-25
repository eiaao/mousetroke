---
title: "Coins et bords de l’écran"
description: "Configurez les coins actifs, le défilement des bords pour le volume et la luminosité, les clics sur les bords et les gestes de frottement sous Windows et sur Mac."
order: 4
---

# Coins et bords de l’écran

## Essayer les réglages par défaut

Sous **Toutes les applications → Bords de l’écran**, chaque liaison de coin ou de bord est modifiable.

| Déclencheur | Valeur par défaut sous Windows | Valeur par défaut sur Mac |
|---|---|---|
| Pousser dans le coin inférieur gauche | Menu Démarrer | Spotlight |
| Pousser dans le coin inférieur droit | Afficher le bureau | Afficher le bureau (F11) |
| Pousser dans le coin supérieur gauche | Changer d’application (Alt+Tab) | Mission Control (Control+Up) |
| Pousser dans le coin supérieur droit | Affichage des tâches (Win+Tab) | Liaison désactivée au départ |
| Défiler sur le bord supérieur | Volume + / Volume − | Volume + / Volume − |
| Clic du milieu sur le bord supérieur | Muet | Muet |
| Clic droit sur le bord supérieur | Lecture / pause | Lecture / pause |
| Défiler sur le bord inférieur | Luminosité + / Luminosité − | Les liaisons de luminosité sont désactivées au départ |
| Frotter le bord droit deux fois vers le haut/bas | Liaison Gestionnaire des tâches ; activez d’abord Exécuter dès la reconnaissance | Activity Monitor |

Les actions de coin n’ont besoin d’aucun bouton : poussez le curseur dans le coin. Le défilement et les clics de bord exigent que le curseur touche le bord choisi. Frotter signifie se déplacer le long du bord dans les directions enregistrées, plutôt que de faire tourner la molette.

## Configurer un déclencheur de bord

1. Choisissez **Bords de l’écran → Nouveau déclencheur de bord** dans la portée d’application voulue ou **Toutes les applications**.
2. Sous **Comment ça commence**, choisissez **Bord d’écran** et sélectionnez un côté ou un coin dans le schéma.
3. Laissez **Ensuite** vide pour une action de coin, ou utilisez **+ Ajouter une étape** pour ajouter un défilement molette haut/bas, un clic de souris ou un tracé le long du bord.
4. Choisissez une [action](actions.md). Pour le volume, choisissez **Média et système → Volume +** ou **Volume −**. Activez **Exécuter dès la reconnaissance** pour un défilement répété ou un frottement sans bouton.

Pour le frottement, ajoutez **Tracer un geste** et enregistrez les directions alternées. Un exemple utile est `↑↓↑↓` contre le bord droit. Essayez avec une action anodine comme un toast Lua avant d’assigner quelque chose de perturbateur.

## Luminosité et différences entre plateformes

Sous Windows, **Luminosité +** / **Luminosité −** commence par l’écran sous le curseur, ajuste de 10 points de pourcentage et affiche le résultat. Il essaie DDC/CI, puis WMI. Les écrans externes doivent prendre en charge et autoriser DDC/CI. Le recours à WMI utilise le premier périphérique de luminosité WMI, en général un écran intégré ; il n’est pas apparié à l’écran du curseur. Si les deux voies échouent, un message à l’écran apparaît. Voir les [recettes Lua](lua-examples.md#luminosité-sous-le-curseur) pour un pas plus petit.

Sur Mac, **Média et système → Luminosité +** / **Luminosité −** envoie les événements des touches système de luminosité pour l’écran intégré. Il ne sélectionne pas l’écran sous le curseur et ne prend pas en charge les écrans externes DDC/CI. Les valeurs par défaut du bord inférieur sont désactivées. Les API Lua de luminosité ne sont pas prises en charge sur Mac non plus.

Sur Mac, Mousetroke avertit si un coin actif du système chevauche un coin configuré. Modifiez l’une des affectations pour éviter des actions concurrentes. Les raccourcis système comme F11 ou Control+Up dépendent des réglages de raccourcis de macOS.

Les exclusions d’applications et la détection du plein écran suspendent aussi les déclencheurs de bord. Consultez les [règles par application](per-app.md) lorsqu’un bord fonctionne dans une application mais pas dans une autre.
