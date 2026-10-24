---
title: "Actions"
description: "Choisissez l’envoi de touches, des actions de fenêtre, des applications, des URL, des commandes, des touches média, des menus ou Lua, et utilisez les variables de geste."
order: 5
---

# Actions

Sélectionnez un geste et choisissez son action sous **Action**. L’éditeur propose huit types. La plupart des tâches n’exigent qu’une action simple ; utilisez [Lua](lua-basics.md) quand vous avez besoin de conditions ou de plusieurs opérations.

## Envoyer des touches

Saisissez une [séquence de touches](key-sequences.md), ou cliquez sur **Enregistrer**, appuyez sur le raccourci puis cliquez sur **Terminé**. Par exemple, `^c` copie sous Windows ; `#c` copie sur Mac. **Aide sur la syntaxe** ouvre le guide de syntaxe.

**Activer d’abord la fenêtre sous le curseur** active d’abord la cible du geste. Sans cela, les raccourcis vont à l’application qui reçoit la saisie au clavier. La cible suit aussi **Préférences → Les gestes agissent sur**. Il n’y a pas d’option de texte brut dans cet éditeur ; la [saisie texte prioritaire](key-sequences.md#texte-et-échappement) est disponible via Lua.

## Actions de fenêtre

Choisissez une opération, appliquée à la cible du geste :

| Opération | Comportement |
|---|---|
| Réduire | Réduit la fenêtre. |
| Agrandir / restaurer | Bascule entre la taille agrandie et la taille restaurée. |
| Fermer | Demande à la fenêtre de se fermer. |
| Toujours au premier plan / non | Bascule le premier plan sous Windows ; non pris en charge et désactivé dans l’éditeur Mac. |

Le contrôle des fenêtres sur Mac utilise l’Accessibilité. **Agrandir / restaurer** appuie sur le bouton Zoom de l’application, donc la taille obtenue suit le comportement de celle-ci ; il ne demande pas le plein écran natif et ignore les fenêtres déjà réduites. Les applications peuvent différer dans les opérations de fenêtre qu’elles exposent.

## Ouvrir une appli

**Application** accepte un exécutable sous Windows ; **Parcourir…** le sélectionne. Sur Mac, utilisez un identifiant de bundle comme `com.apple.Safari`, un chemin `.app` ou un chemin d’exécutable. **Arguments** est facultatif. Sur Mac, les arguments sont découpés en gérant les guillemets ; utilisez des chemins complets pour les exécutables en ligne de commande.

**Si déjà lancée** propose **Passer dessus** ou **En ouvrir une autre**. Windows propose aussi **Exécuter en tant qu’administrateur**. Mac ne prend pas en charge l’élévation de privilèges ni les options de lancement avec filtre de fenêtre. Il n’y a pas de champs classe/titre/regex dans l’éditeur d’actions actuel.

## Ouvrir une URL

Renseignez **URL** et, facultativement, **Navigateur**. Laissez Navigateur vide pour le navigateur par défaut. Windows accepte un exécutable de navigateur ; Mac accepte un identifiant de bundle ou un chemin d’application de navigateur. Les boutons **Insérer** ajoutent le texte sélectionné, le texte du presse-papiers, le nom de l’application, l’ID de processus ou des espaces réservés pour le titre de la fenêtre.

Pour une recherche web sur le texte sélectionné, définissez l’URL sur :

```text
https://www.google.com/search?q=%MS_SELECTED_TEXT%
```

Sélectionnez du texte dans une application, puis effectuez le geste. Le texte sélectionné est encodé pour les URL. S’il commence déjà par `http://`, `https://` ou `www.`, Mousetroke ouvre directement cette adresse ; `www.` reçoit un préfixe `https://`.

## Exécuter une commande

Saisissez la commande dans la zone de texte. **Insérer** ajoute des variables. Windows l’exécute via un `cmd.exe /C` masqué ; Mac utilise `/bin/zsh -c`. C’est de la syntaxe shell, donc mettez les arguments entre guillemets correctement. Le texte substitué n’est pas échappé pour le shell ; évitez d’injecter du texte sélectionné non fiable dans des commandes.

Par exemple, la commande Windows suivante ouvre le Gestionnaire des tâches :

```text
taskmgr.exe
```

Sur Mac, utilisez plutôt **Ouvrir une appli** avec `com.apple.ActivityMonitor`.

## Média et système

L’éditeur propose **Lecture / pause**, **Piste suivante**, **Piste précédente**, **Volume +**, **Volume −**, **Muet**, **Luminosité +** et **Luminosité −**.

Sous Windows, la luminosité s’ajuste de 10 points de pourcentage via DDC/CI avec un recours à WMI. Sur Mac, la luminosité envoie les touches de luminosité pour l’écran intégré ; le DDC/CI externe n’est pas pris en charge. Ces actions système Mac sont disponibles même si les fonctions Lua de luminosité ne sont pas prises en charge. Voir [bords de l’écran](screen-edges.md).

## Afficher un menu

Cliquez sur **+ Ajouter un élément**, saisissez le **Nom de l’élément**, puis choisissez une action dans chaque élément. Utilisez **Monter**, **Descendre** et **Supprimer l’élément** pour l’organiser. Le menu apparaît à la position du geste. L’éditeur n’expose pas de champ de nombre de colonnes et n’offre pas d’autre type de menu dans un élément. Mac prend en charge une seule colonne ; les réglages multi-colonnes sont signalés comme non pris en charge.

## Script Lua

Collez un script complet définissant `ms_action`. L’éditeur pointe vers les [bases de Lua](lua-basics.md), la [référence de l’API](lua-api.md) et les [exemples](lua-examples.md). La zone de texte accepte la tabulation comme deux espaces.

<a id="variables"></a>

## Variables

Utilisez ces espaces réservés dans **Ouvrir une URL → URL**, **Exécuter une commande** et **Ouvrir une appli → Application / Arguments**. Ils ne sont pas développés dans les chaînes de touches, le code Lua, les noms de menu ou Navigateur.

| Variable | Valeur |
|---|---|
| `%MS_SELECTED_TEXT%` | Sélection capturée quand l’action en a besoin ; vide si indisponible. |
| `%MS_CLIPBOARD_TEXT%` | Texte actuel du presse-papiers, capturé à la demande. |
| `%MS_PID%` | ID de processus de la cible. |
| `%MS_EXE%` | Identité de l’exécutable cible : chemin sous Windows ; identifiant de bundle ou chemin d’exécutable sur Mac. |
| `%MS_APP_NAME%` | Partie nom de fichier de l’identité, qui peut être un identifiant de bundle sur Mac. |
| `%MS_WID%` | Identifiant de fenêtre de la plateforme, en texte décimal. |
| `%MS_WIN_NAME%` | Titre de la fenêtre cible ; Mac le récupère via l’Accessibilité quand nécessaire. |
| `%MS_MOUSE_X%`, `%MS_MOUSE_Y%` | Coordonnées à l’écran au début du geste. |
| `%MS_MOUSE_Y_FLIP%` | Coordonnée basse de l’écran virtuel moins la coordonnée Y de départ du geste. |
| `%MS_TARGET_MODE%` | `UnderCursor` ou `Focused`, selon la préférence de cible. |

Les variables de cible indisponibles peuvent être absentes ; les espaces réservés inconnus restent inchangés. Seul le texte sélectionné reçoit un encodage URL automatique dans un modèle d’URL. Les autres substitutions sont insérées telles quelles. Lua utilise les [arguments d’entrée](lua-basics.md#arguments-dentrée) et les fonctions `ms` plutôt que le développement d’espaces réservés.
