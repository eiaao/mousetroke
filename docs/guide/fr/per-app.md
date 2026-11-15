---
title: "Règles par application"
description: "Ajoutez des applications, personnalisez les gestes, excluez des applications et réglez la pause en plein écran avec des exceptions par application."
order: 6
---

# Règles par application

## Ajouter une application

Cliquez sur **Ajouter une appli…** dans la barre latérale. Choisissez **Gestes propres à cette appli** ou **Désactiver les gestes dedans**, puis :

- Sélectionnez une entrée sous **En cours d’exécution**.
- Cliquez sur **Choisir une fenêtre**, puis cliquez sur la fenêtre de l’application ; Échap ou un clic droit annule.
- Utilisez **Parcourir…** pour choisir l’application.
- Déposez une application ou son raccourci dans la fenêtre des paramètres. Mac accepte les bundles `.app` et les fichiers exécutables ; ses portées utilisent les identifiants de bundle quand ils sont disponibles.

Les règles Windows peuvent correspondre à un nom d’exécutable ou à un chemin complet, sans tenir compte de la casse. Les identités d’application Mac ressemblent en général à `com.apple.Safari`. Utilisez le sélecteur plutôt que de deviner l’identité. La portée de l’application apparaît dans la barre latérale et peut avoir ses propres gestes de souris et bords d’écran.

## Gestes propres à une appli et gestes globaux

**Toutes les applications** contient les gestes globaux. Un geste propre à une application est prioritaire sur le geste global correspondant. Avec **Utiliser aussi les gestes globaux** activé, les liaisons globales restent disponibles quand les liaisons de l’application ne correspondent pas. Désactivez-le pour n’utiliser que les gestes de cette application.

Par exemple, ajoutez un navigateur, créez un geste `↓→` au bouton droit et assignez-y une action différente. Les autres applications conservent l’action globale de `↓→`. Pour copier une configuration existante, utilisez [Dupliquer](gestures.md#rechercher-copier-et-désactiver) dans sa portée et modifiez la liaison. Pour enregistrer un tracé une seule fois et lui donner une action différente dans chaque application, utilisez la [bibliothèque de gestes](gesture-library.md).

## Désactiver les gestes dans une appli

Sélectionnez l’application dans la barre latérale et désactivez **Utiliser les gestes**. Cela désactive ses propres gestes, les gestes globaux et les déclencheurs de bords d’écran. Les boutons droit, du milieu et latéraux passent normalement. Les liaisons enregistrées sont conservées ; réactiver **Utiliser les gestes** les rétablit. Le bouton corbeille **Retirer cette application**, en haut à droite de la page de l’application, supprime la portée propre à l’application.

C’est utile pour le bureau à distance, les machines virtuelles, la modélisation 3D, les outils de conception et les éditeurs de jeux qui utilisent le glisser au bouton droit ou du milieu. La liste **Désactivation conseillée** du sélecteur comprend des applications connues de ces catégories. Les applications installées détectées peuvent être ajoutées ensemble.

## Plein écran et premier démarrage

La pause automatique en plein écran est activée par défaut : le moteur évite de démarrer des gestes et des déclencheurs de bord tant que l’application au premier plan est détectée en plein écran. Cela couvre les jeux, la vidéo et les présentations en plein écran ; c’est distinct de l’état de pause manuel. Sous Windows, la détection vérifie une fenêtre sans bord couvrant son écran et exclut les fenêtres du bureau et du shell. Mac compare la fenêtre de l’application au premier plan aux limites de l’écran. Une application qui semble en plein écran peut donc se comporter différemment sur les deux plateformes.

Désactivez « Mettre les gestes en pause en plein écran » dans les préférences pour modifier le réglage global. Chaque application propose « Suivre le réglage global », « Garder actifs » et « Mettre en pause » sous « En plein écran » ; un choix explicite par application est prioritaire. Pour utiliser les gestes globaux dans Chrome en plein écran, ajoutez Chrome, gardez l’héritage des gestes globaux et choisissez « Garder actifs ». Aucun geste propre à Chrome n’est nécessaire. La pause manuelle et les exclusions restent applicables.

Pour un jeu en fenêtre ou un bureau à distance, ajoutez une exclusion explicite. Pour tout mettre en pause, utilisez la [commande de la zone de notification/barre des menus ou le raccourci de pause](getting-started.md#pause).

Le guide de premier lancement montre les applications détectées qui ont souvent besoin de leur propre glisser à la souris. Les entrées cochées sont désactivées quand vous le terminez. Vérifiez-les avant de continuer. La configuration intégrée du Mac comprend aussi des portées désactivées pour ses applications d’exclusion suggérées ; celles-ci n’ont pas de liaisons propres. Vous pouvez modifier les exclusions plus tard depuis la barre latérale.

Suivant : [bibliothèque de gestes](gesture-library.md), [actions](actions.md), [sauvegarde de la configuration](config.md), [dépannage](troubleshooting.md).
