---
title: "Bibliothèque de gestes"
description: "Définir un geste une fois et l’utiliser dans plusieurs applications avec des actions différentes : organiser et importer vos gestes, les attribuer à d’autres applications, modifier un geste partagé, compatibilité."
order: 3.5
---

# Bibliothèque de gestes

La bibliothèque de gestes sépare la façon de dessiner de ce qui se passe. Définissez un geste une seule fois, puis donnez-lui une action différente dans chaque application. Réenregistrer un geste partagé met à jour tous les endroits qui l’utilisent, et un geste de forme ne conserve qu’un seul jeu d’échantillons.

## Parcourir la bibliothèque

Cliquez sur **Gestes** dans la barre latérale. La liste est divisée en **Gestes de souris** et **Bords de l’écran**. Filtrez par bouton de déclenchement, utilisez **Inutilisés** pour trouver les gestes inutilisés, ou recherchez par nom, tracé ou usage. Chaque carte indique à combien d’endroits le geste est utilisé.

Cliquez sur une carte pour ouvrir ses détails. En haut, vous pouvez redessiner ; **Utilisé dans** liste chaque application, nom de geste et action qui l’utilise, et un clic sur une ligne y mène. Sur une nouvelle installation, les gestes intégrés sont déjà dans la bibliothèque. Après une mise à jour depuis une version antérieure, vos gestes sont encore répartis entre les applications : organisez-les d’abord comme décrit ci-dessous.

## Organiser et importer

Quand la bibliothèque est vide, cliquez sur **Organiser et importer les gestes…** ; ensuite, **Organiser et importer…** reste disponible en haut à droite. Choisissez d’abord la portée :

- **Gestes communs uniquement (recommandé)** : ajoute les gestes de **Toutes les applications** et relie les gestes identiques ou similaires des autres applications. Les gestes propres à une application le restent.
- **Tous les gestes** : ajoute les gestes partagés et tous les gestes propres à chaque application.

Cliquez sur **Suivant** pour vérifier le résultat. Rien ne change avant votre confirmation :

- Les tracés identiques sont fusionnés automatiquement. Les tracés de mêmes directions qui ne diffèrent que par la longueur des segments sont fusionnés par défaut.
- **Formes semblables — à confirmer** : pour chaque groupe, choisissez celui à garder (**Utiliser celui-ci**) ou **Combiner les exemples**. Les groupes sous la similarité recommandée ne sont pas fusionnés par défaut, et une forme fusionnée peut être reconnue légèrement différemment.
- **Ajoutés directement** : dépliez la section pour décocher des gestes. Les gestes décochés ne sont pas importés et restent propres à leur application.

Cliquez sur **Importer** pour terminer ; la notification permet d’annuler. Deux gestes similaires d’une même application qui font des choses différentes ne sont jamais fusionnés.

## Utiliser un geste dans une autre application

Dans les détails du geste, cliquez sur **Utiliser dans une autre application…**, choisissez une application, puis une action. Les applications qui utilisent déjà le geste ne peuvent pas être choisies ; si une application a déjà le même tracé, un avertissement de conflit s’affiche ; une application dont les gestes sont désactivés peut le recevoir, mais il n’y fonctionnera pas. **Ajouter une appli…**, en fin de liste, ajoute d’abord une application. Vous restez ensuite sur les détails du geste, la nouvelle ligne mise en évidence, et **Modifier** dans la notification ouvre cette association.

Vous pouvez aussi partir de l’application. Dans l’éditeur, un geste qui n’appartient qu’à l’application courante affiche **Uniquement dans cette application** :

- Cliquez sur **Choisir dans la bibliothèque** pour utiliser un geste de la bibliothèque. Le sélecteur permet la recherche ; choisir l’un des **Gestes dans d’autres applications** l’ajoute d’abord à la bibliothèque et le partage avec l’original ; **Enregistrer un nouveau geste (uniquement dans cette application)** n’entre pas dans la bibliothèque.
- Le menu **Plus d’actions** contient **Ajouter à la bibliothèque** et **Utiliser dans une autre application…**.

## Modifier un geste partagé

Dans l’éditeur, un geste de la bibliothèque affiche **Bibliothèque** et ses lieux d’utilisation. Si vous le redessinez, la modification s’applique partout où il est utilisé, et les nouveaux conflits sont listés :

- **Modifier ici seulement** : la modification ne vaut que pour l’application courante ; le geste devient propre à celle-ci, et la bibliothèque comme les autres applications gardent l’ancien tracé.
- **Terminé** : conserve la modification partagée et ferme la notification.

**Changer** passe à un autre geste de la bibliothèque ; le bouton ↗ (**Afficher dans la bibliothèque**) ouvre les détails du geste.

## Créer et supprimer

**Nouveau geste** enregistre un geste directement dans la bibliothèque. Il est ajouté une fois le tracé terminé ; le nom est facultatif. Attribuez-le ensuite avec **Utiliser dans une autre application…**.

Si vous choisissez **Supprimer le geste** alors que le geste est encore utilisé, choisissez :

- **Supprimer ; chaque utilisation conserve sa propre copie**
- **Le supprimer avec toutes ses utilisations**

Les deux peuvent être annulés depuis la notification.

## Configuration et compatibilité

La bibliothèque est enregistrée sous `gestures` dans le fichier de configuration, et chaque geste qui l’utilise conserve aussi une copie complète de son tracé. Les versions antérieures de Mousetroke peuvent donc lire la nouvelle configuration, et les gestes continuent de fonctionner. En revanche, enregistrer les réglages dans une version antérieure supprime la bibliothèque : chaque geste redevient propre à son application, et vous pouvez réorganiser après la mise à jour. Si une modification manuelle du fichier rend un tracé différent de la bibliothèque, cet endroit devient propre à son application et vous en êtes averti une fois. Emplacements et sauvegardes : [Configuration, sauvegarde et mises à jour](config.md).

Suivant : [Gestes de souris](gestures.md), [Règles par application](per-app.md), [Bibliothèque d’actions](action-library.md).
