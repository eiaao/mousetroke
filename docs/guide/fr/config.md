---
title: "Configuration, sauvegarde et mises à jour"
description: "Trouvez la configuration et les journaux de Mousetroke, exportez et restaurez les paramètres, rechargez les modifications, mettez l’application à jour ou désinstallez-la."
order: 11
---

# Configuration, sauvegarde et mises à jour

## Emplacements des fichiers

| Fichier | Windows | Mac |
|---|---|---|
| Configuration | `%APPDATA%\mousetroke\config.json` | `~/Library/Application Support/mousetroke/config.json` |
| Journal d’exécution | `%APPDATA%\mousetroke\mousetroke.log` | `~/Library/Logs/mousetroke/mousetroke.log` |

Les journaux tournent vers `mousetroke.log.old` après avoir dépassé 1 Mo. **À propos → Emplacements des fichiers** affiche les chemins réels. **Ouvrir le dossier** ouvre le répertoire de configuration ; **Ouvrir le journal** ouvre le journal d’exécution. Sous **Importer et exporter**, **Afficher dans l’Explorateur de fichiers** sous Windows ou **Afficher dans le Finder** sur Mac révèle la configuration.

## Exporter, importer et restaurer

Sous **Importer et exporter → Fichier de configuration Mousetroke**, cliquez sur **Exporter…** et enregistrez une sauvegarde JSON. **Importer depuis un fichier…** lit un JSON de configuration Mousetroke et remplace la configuration actuelle, préférences comprises. Il ne fusionne pas ce fichier avec vos liaisons actuelles. Il n’y a pas d’option de fusion de configuration native dans ce flux. Vous pouvez annuler l’import avec Ctrl+Z sous Windows ou ⌘+Z sur Mac tant que les paramètres sont ouverts.

Avant de modifier le JSON à la main, exportez une sauvegarde. Pour une restauration directe, quittez le moteur et fermez les paramètres, remplacez `config.json` par la sauvegarde, puis rouvrez Mousetroke. Garder les paramètres fermés évite que son enregistrement automatique n’écrase le fichier restauré.

## Rechargement et réinitialisation

Le moteur surveille l’horodatage du fichier de configuration. Windows vérifie environ une fois par seconde ; Mac vérifie dans la branche de maintenance planifiée toutes les deux secondes. Un changement reçu pendant un geste est appliqué après la fin de la capture. Un JSON invalide laisse la configuration active précédente en vigueur et enregistre une erreur dans le journal.

L’interface des paramètres actuelle n’a pas de bouton de réinitialisation aux valeurs par défaut. Pour repartir des valeurs intégrées, quittez Mousetroke et fermez les paramètres, sauvegardez puis supprimez ou renommez `config.json`, puis rouvrez. Un fichier manquant est initialisé avec les valeurs par défaut de la plateforme. Un fichier mal formé n’est pas une demande de réinitialisation.

## Mises à jour

**Préférences → Rechercher les mises à jour automatiquement** est activé par défaut et interroge GitHub une fois par jour. Désactivez-le là pour arrêter les vérifications automatiques. Utilisez **À propos → Rechercher des mises à jour** pour une vérification manuelle et **Nouveautés** pour les notes de version. Un élément de menu de nouvelle version ouvre aussi À propos.

Sous Windows, **Mettre à jour** télécharge l’exécutable, vérifie son SHA-256, remplace la copie en cours d’exécution et redémarre. **Ouvrir la page de téléchargement** est disponible pour une installation manuelle.

Sur Mac, **Mettre à jour** télécharge le DMG, vérifie le SHA-256, l’identité du bundle, la version et l’exigence de signature, remplace l’application et redémarre. L’ancien bundle va à la Corbeille après l’arrêt ; un échec de remplacement ou de passation au redémarrage le restaure. **Télécharger manuellement** est disponible si le dossier d’installation n’est pas inscriptible ou si la mise à jour échoue. Les mises à jour dans l’application évitent une nouvelle invite de quarantaine du navigateur et préservent l’identité d’installation utilisée pour les autorisations. Un nouveau téléchargement depuis le navigateur peut de nouveau exiger **Ouvrir quand même**.

## Désinstallation

Sous Windows : désactivez **Activer les gestes au démarrage de Windows** dans les Préférences, **Quittez** depuis la zone de notification, fermez les paramètres, puis supprimez l’exécutable et `%APPDATA%\mousetroke` si vous n’avez plus besoin des paramètres ou des journaux.

Sur Mac : **Quittez** depuis la barre des menus, fermez les paramètres et supprimez `Mousetroke.app`. Pour aussi supprimer les données, supprimez `~/Library/Application Support/mousetroke` et `~/Library/Logs/mousetroke`. Retirez Mousetroke des éléments d’ouverture à la connexion dans les Réglages Système. Conservez une sauvegarde exportée si vous êtes susceptible de réinstaller.

Voir le [dépannage](troubleshooting.md) pour les erreurs de configuration et les journaux manquants.
