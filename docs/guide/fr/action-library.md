---
title: "Bibliothèque d’actions"
description: "Choisissez des actions prêtes à l’emploi, réglez leurs paramètres et consultez le catalogue complet pour Windows et Mac."
order: 5.5
---

# Bibliothèque d’actions

Les actions prédéfinies décrivent ce que vous voulez faire, par exemple fermer un onglet ou centrer une fenêtre. Mousetroke enregistre leur ID et leurs paramètres, puis choisit l’implémentation Windows ou Mac à l’exécution. Les raccourcis des navigateurs et des éditeurs dépendent toujours de leur prise en charge par l’application qui les reçoit.

## Choisir une action

1. Créez ou sélectionnez un geste, puis cliquez sur **Choisir une action**, ou sur **Remplacer** pour une action existante.
2. Recherchez un nom, une description, un alias ou la notation d’un raccourci, comme `ctrl+w` sous Windows ou `cmd+w` / `⌘w` sur Mac. La recherche couvre toutes les catégories. Sans recherche, parcourez **Fréquentes**, **Navigateur**, **Édition**, **Texte**, **Fenêtres**, **Bureau et système**, **Médias**, **Ouvrir** ou **Mousetroke**.
3. Choisissez un résultat. Les actions avec paramètres ouvrent un formulaire : saisissez du texte, choisissez un format de date ou un service de recherche, ou indiquez une URL, un fichier, un dossier ou une application. Cliquez sur **Utiliser cette action** pour confirmer. Les actions indisponibles sont désactivées et précisent la limite de la plateforme ou leur mise en œuvre prévue dans une deuxième phase.
4. Essayez le geste en dehors des paramètres. Un nom de geste vide reprend le nom de l’action choisie. Les éléments de menu utilisent le même sélecteur.

À l’ouverture du sélecteur, le champ de recherche reçoit le focus. Les flèches haut et bas parcourent les résultats disponibles et Entrée en sélectionne un. Échap ferme d’abord le formulaire de paramètres, puis le sélecteur. Dans un champ de texte multiligne, Ctrl+Enter ou Command+Enter confirme le formulaire.

**Convertir en action avancée** remplace une action prédéfinie par ses touches, opération de fenêtre, URL, application, commande ou touche système modifiables pour la plateforme actuelle. La confirmation précise que la conversion est irréversible : le résultat ne suit plus les valeurs par défaut de plateforme ou de langue de l’action prédéfinie. Les actions natives de disposition des fenêtres, d’alimentation, de texte littéral, de date et heure, de copie en texte brut, de pause et de paramètres restent prédéfinies ; leurs cartes expliquent pourquoi la conversion est indisponible. La catégorie **Avancées**, en fin de liste, propose l’envoi de touches, les commandes, Lua et les menus. La [référence des actions avancées](actions.md) décrit aussi les éditeurs existants.

## Texte, fenêtres et alimentation

- **Insérer du texte** restitue le texte tel quel, y compris les espaces, les sauts de ligne, les caractères chinois, les emoji et les caractères tels que `^+#{ENTER}`. **Coller et restaurer le presse-papiers** est la méthode par défaut : elle utilise temporairement le presse-papiers, puis restaure son **texte** précédent après un bref délai. S’il contient des images, du HTML, des listes de fichiers ou d’autres données non textuelles, ou si son contenu ne peut pas être vérifié, le texte est saisi directement sans modifier le presse-papiers. `paste_delay_ms` n’apparaît qu’avec `paste` : l’interface accepte 50–5000 ms, avec 150 ms par défaut. **Saisir les caractères** utilise Unicode sans passer par le presse-papiers. Les deux méthodes envoient la saisie à l’application qui reçoit le clavier.
- **Insérer la date et l’heure** utilise l’horloge locale du système et la méthode de sortie choisie. Les formats comprennent la date ISO, la date chinoise, la date américaine, l’heure et la date avec l’heure. Un motif personnalisé accepte `YYYY`, `MM`, `DD`, `HH`, `mm` et `ss`, en respectant la casse. Les autres caractères sont littéraux ; les apostrophes protègent le texte littéral et `''` insère une apostrophe. Par exemple, `YYYY-MM-DD HH:mm:ss` produit un horodatage. Les apostrophes non fermées sont refusées.
- **Copier la sélection en texte brut** envoie Ctrl+C / Command+C, attend jusqu’à 500 ms un changement de version du presse-papiers, puis remplace le contenu copié par son texte seul. Si la copie n’aboutit pas ou ne contient pas de texte, un échec est signalé et le contenu du presse-papiers est conservé. Sélectionnez du texte et gardez cette application au premier plan avant d’utiliser l’action.
- Les actions de disposition utilisent la même cible de geste que les autres actions de fenêtre, selon **Préférences → Les gestes agissent sur**. Windows utilise le positionnement natif des fenêtres ; Mac utilise Accessibilité et la zone utile de l’écran. Les tailles minimales et les restrictions des applications peuvent empêcher un partage exact de l’écran en deux. Sur Mac, quittez d’abord le plein écran natif. **Déplacer vers l’écran suivant** parcourt les écrans dans un ordre stable, conserve la position relative et l’état agrandi, et ne fait rien avec un seul écran.
- Après **Éteindre l’écran**, une entrée permet de réveiller les écrans. **Mettre en veille** demande la veille normale du système, qui peut être refusée par le compte ou les règles du système. En cas d’échec d’une action native, une ligne est écrite dans le journal et un bref message apparaît à l’écran. **Suspendre / reprendre les gestes** utilise le même état de pause que l’icône de notification et le raccourci ; utilisez l’un de ces moyens pour reprendre les gestes. **Ouvrir les réglages** ouvre la fenêtre habituelle.

## Catalogue

Les tableaux ci-dessous sont générés à partir du registre et comprennent les 87 entrées. ✓ signifie que l’action est implémentée sur cette plateforme ; sa disponibilité dépend toujours de l’application, du matériel et des autorisations nécessaires. L’aide-mémoire des gestes est prévu pour la deuxième phase.

<!-- action-library:generated:start -->

### Navigateur

| Action / ID | Fonction et contexte d’utilisation | Paramètres | Windows | Mac |
|---|---|---|---|---|
| Précédent<br>`browser.back` | Disponible dans les navigateurs web. | — | ✓ | ✓ |
| Avancer<br>`browser.forward` | Disponible dans les navigateurs web. | — | ✓ | ✓ |
| Actualiser<br>`browser.reload` | Disponible dans les navigateurs web. | — | ✓ | ✓ |
| Actualiser sans cache<br>`browser.hard_reload` | Disponible dans les navigateurs web. | — | ✓ | ✓ |
| Nouvel onglet<br>`browser.new_tab` | Navigateurs, éditeurs et autres apps compatibles avec ce raccourci. | — | ✓ | ✓ |
| Fermer l’onglet<br>`browser.close_tab` | Navigateurs, éditeurs et autres apps compatibles avec ce raccourci. | — | ✓ | ✓ |
| Rouvrir l’onglet fermé<br>`browser.reopen_tab` | Disponible dans les navigateurs web. | — | ✓ | ✓ |
| Onglet précédent<br>`browser.prev_tab` | Navigateurs, éditeurs et autres apps compatibles avec ce raccourci. | — | ✓ | ✓ |
| Onglet suivant<br>`browser.next_tab` | Navigateurs, éditeurs et autres apps compatibles avec ce raccourci. | — | ✓ | ✓ |
| Nouvelle fenêtre<br>`browser.new_window` | Navigateurs, éditeurs et autres apps compatibles avec ce raccourci. | — | ✓ | ✓ |
| Nouvelle fenêtre privée<br>`browser.private_window` | Chrome, Edge et Safari ; Firefox utilise un autre raccourci. | — | ✓ | ✓ |
| Activer la barre d’adresse<br>`browser.address_bar` | Disponible dans les navigateurs web. | — | ✓ | ✓ |
| Rechercher dans la page<br>`browser.find` | Navigateurs, éditeurs et autres apps compatibles avec ce raccourci. | — | ✓ | ✓ |
| Ajouter cette page aux favoris<br>`browser.bookmark` | Disponible dans les navigateurs web. | — | ✓ | ✓ |
| Zoom avant<br>`view.zoom_in` | Navigateurs, éditeurs et autres apps compatibles avec ce raccourci. | — | ✓ | ✓ |
| Zoom arrière<br>`view.zoom_out` | Navigateurs, éditeurs et autres apps compatibles avec ce raccourci. | — | ✓ | ✓ |
| Réinitialiser le zoom<br>`view.zoom_reset` | Navigateurs, éditeurs et autres apps compatibles avec ce raccourci. | — | ✓ | ✓ |

### Édition

| Action / ID | Fonction et contexte d’utilisation | Paramètres | Windows | Mac |
|---|---|---|---|---|
| Haut de page<br>`view.top` | Navigateurs, éditeurs et autres apps compatibles avec ce raccourci. | — | ✓ | ✓ |
| Bas de page<br>`view.bottom` | Navigateurs, éditeurs et autres apps compatibles avec ce raccourci. | — | ✓ | ✓ |
| Page précédente<br>`view.page_up` | Navigateurs, éditeurs et autres apps compatibles avec ce raccourci. | — | ✓ | ✓ |
| Page suivante<br>`view.page_down` | Navigateurs, éditeurs et autres apps compatibles avec ce raccourci. | — | ✓ | ✓ |
| Copier<br>`edit.copy` | Navigateurs, éditeurs et autres apps compatibles avec ce raccourci. | — | ✓ | ✓ |
| Couper<br>`edit.cut` | Navigateurs, éditeurs et autres apps compatibles avec ce raccourci. | — | ✓ | ✓ |
| Coller<br>`edit.paste` | Navigateurs, éditeurs et autres apps compatibles avec ce raccourci. | — | ✓ | ✓ |
| Coller et Entrée<br>`edit.paste_enter` | Navigateurs, éditeurs et autres apps compatibles avec ce raccourci. | — | ✓ | ✓ |
| Coller en texte brut<br>`edit.paste_plain` | Apps avec collage en texte brut ; le raccourci varie. | — | ✓ | ✓ |
| Tout sélectionner<br>`edit.select_all` | Navigateurs, éditeurs et autres apps compatibles avec ce raccourci. | — | ✓ | ✓ |
| Annuler<br>`edit.undo` | Navigateurs, éditeurs et autres apps compatibles avec ce raccourci. | — | ✓ | ✓ |
| Rétablir<br>`edit.redo` | Navigateurs, éditeurs et autres apps compatibles avec ce raccourci. | — | ✓ | ✓ |
| Enregistrer<br>`edit.save` | Navigateurs, éditeurs et autres apps compatibles avec ce raccourci. | — | ✓ | ✓ |
| Entrée<br>`key.enter` | Navigateurs, éditeurs et autres apps compatibles avec ce raccourci. | — | ✓ | ✓ |
| Retour arrière<br>`key.backspace` | Navigateurs, éditeurs et autres apps compatibles avec ce raccourci. | — | ✓ | ✓ |
| Supprimer<br>`key.delete` | Navigateurs, éditeurs et autres apps compatibles avec ce raccourci. | — | ✓ | ✓ |
| Annuler (Échap)<br>`key.escape` | Navigateurs, éditeurs et autres apps compatibles avec ce raccourci. | — | ✓ | ✓ |
| Plein écran<br>`view.fullscreen` | Navigateurs, éditeurs et autres apps compatibles avec ce raccourci. | — | ✓ | ✓ |

### Texte

| Action / ID | Fonction et contexte d’utilisation | Paramètres | Windows | Mac |
|---|---|---|---|---|
| Insérer du texte<br>`text.type` | Champs de texte ; colle puis restaure le presse-papiers par défaut. | Texte (`text`): obligatoire<br>Méthode de saisie (`method`): Coller et restaurer le presse-papiers (`paste`) / Saisir les caractères (`type`); par défaut : Coller et restaurer le presse-papiers<br>Attente après collage (ms) (`paste_delay_ms`): par défaut : `150` ; uniquement si Méthode de saisie vaut Coller et restaurer le presse-papiers | ✓ | ✓ |
| Insérer la date et l’heure<br>`text.datetime` | Champs de texte ; utilise la date et l’heure locales. | Format de date et d’heure (`format`): 2026-10-24 (`date_iso`) / 2026年10月24日 (`date_local`) / 10/24/2026 (`date_us`) / 14:30 (`time`) / 2026-10-24 14:30 (`datetime`) / Personnalisé (`custom`); par défaut : 2026-10-24<br>Format personnalisé (`custom_format`): par défaut : `YYYY-MM-DD` ; uniquement si Format de date et d’heure vaut Personnalisé<br>Méthode de saisie (`method`): Coller et restaurer le presse-papiers (`paste`) / Saisir les caractères (`type`); par défaut : Coller et restaurer le presse-papiers<br>Attente après collage (ms) (`paste_delay_ms`): par défaut : `150` ; uniquement si Méthode de saisie vaut Coller et restaurer le presse-papiers | ✓ | ✓ |
| Recherche web<br>`text.search` | Utilise le navigateur par défaut ; sélectionnez d’abord du texte. | Moteur de recherche (`engine`): Bing (`bing`) / Google (`google`) / Baidu (`baidu`) / DuckDuckGo (`duckduckgo`) / Personnalisé (`custom`); Baidu avec l’interface en chinois simplifié ; Google sinon<br>URL personnalisée (`url`): par défaut : `https://www.google.com/search?q=%MS_SELECTED_TEXT%` ; uniquement si Moteur de recherche vaut Personnalisé | ✓ | ✓ |
| Traduire le texte sélectionné<br>`text.translate` | Utilise le navigateur par défaut ; sélectionnez d’abord du texte. | Service de traduction (`service`): Google Traduction (`google`) / DeepL (`deepl`) / Bing Traducteur (`bing`); par défaut : Google Traduction<br>Langue cible (`target_language`): Suivre la langue de l’interface (`ui`) / 简体中文 (`zh-CN`) / English (`en-US`) / español (`es-ES`) / Français (`fr-FR`) / Deutsch (`de-DE`) / 日本語 (`ja-JP`) / Português (Brasil) (`pt-BR`) / Русский (`ru-RU`) / Italiano (`it-IT`) / 한국어 (`ko-KR`); par défaut : Suivre la langue de l’interface | ✓ | ✓ |
| Ouvrir l’URL sélectionnée<br>`text.open_selected_url` | Utilise le navigateur par défaut ; sélectionnez d’abord du texte. | — | ✓ | ✓ |
| Copier la sélection en texte brut<br>`text.copy_plain` | Apps pouvant copier du texte ; supprime la mise en forme du presse-papiers. | — | ✓ | ✓ |

### Fenêtres

| Action / ID | Fonction et contexte d’utilisation | Paramètres | Windows | Mac |
|---|---|---|---|---|
| Réduire<br>`window.minimize` | S’applique à la fenêtre ciblée par le geste. | — | ✓ | ✓<br>Nécessite l’autorisation Accessibilité. |
| Agrandir / restaurer<br>`window.toggle_maximize` | S’applique à la fenêtre ciblée par le geste. | — | ✓ | ✓<br>Nécessite l’autorisation Accessibilité. |
| Fermer la fenêtre<br>`window.close` | S’applique à la fenêtre ciblée par le geste. | — | ✓ | ✓<br>Nécessite l’autorisation Accessibilité. |
| Quitter l’application<br>`app.quit` | S’applique à la fenêtre ciblée par le geste. | — | ✓ | ✓ |
| Terminer le processus<br>`app.kill` | Force l’arrêt de l’app cible ; les données non enregistrées seront perdues. | — | ✓ | ✓ |
| Toujours au premier plan<br>`window.toggle_topmost` | Basculer le maintien au premier plan de la fenêtre ciblée. | — | ✓ | Cette action est uniquement disponible sur Windows. |
| Placer sur la moitié gauche<br>`window.snap_left` | S’applique à la fenêtre ciblée par le geste. | — | ✓ | ✓<br>Nécessite l’autorisation Accessibilité. |
| Placer sur la moitié droite<br>`window.snap_right` | S’applique à la fenêtre ciblée par le geste. | — | ✓ | ✓<br>Nécessite l’autorisation Accessibilité. |
| Centrer la fenêtre<br>`window.center` | S’applique à la fenêtre ciblée par le geste. | — | ✓ | ✓<br>Nécessite l’autorisation Accessibilité. |
| Déplacer vers l’écran suivant<br>`window.next_monitor` | S’applique à la fenêtre ciblée par le geste. | — | ✓ | ✓<br>Nécessite l’autorisation Accessibilité. |
| Masquer l’application actuelle<br>`app.hide` | Masquer l’application actuelle. | — | Cette action est uniquement disponible sur macOS. | ✓ |

### Bureau et système

| Action / ID | Fonction et contexte d’utilisation | Paramètres | Windows | Mac |
|---|---|---|---|---|
| Changer d’application<br>`desktop.switch_app` | Action système ; certains comportements dépendent des raccourcis configurés. | — | ✓ | ✓ |
| Affichage des tâches / Mission Control<br>`desktop.task_view` | Action système ; certains comportements dépendent des raccourcis configurés. | — | ✓ | ✓ |
| Afficher le bureau<br>`desktop.show` | Action système ; certains comportements dépendent des raccourcis configurés. | — | ✓ | ✓ |
| Bureau virtuel précédent<br>`desktop.prev` | Action système ; certains comportements dépendent des raccourcis configurés. | — | ✓ | ✓ |
| Bureau virtuel suivant<br>`desktop.next` | Action système ; certains comportements dépendent des raccourcis configurés. | — | ✓ | ✓ |
| Menu Démarrer / Spotlight<br>`desktop.launcher` | Action système ; certains comportements dépendent des raccourcis configurés. | — | ✓ | ✓ |
| Ouvrir le gestionnaire de fichiers<br>`desktop.files` | Action système ; certains comportements dépendent des raccourcis configurés. | — | ✓ | ✓ |
| Capture<br>`capture.region` | Action système ; certains comportements dépendent des raccourcis configurés. | — | ✓ | ✓ |
| Capturer et enregistrer l’écran<br>`capture.screen` | Action système ; certains comportements dépendent des raccourcis configurés. | — | ✓ | ✓ |
| Outils de capture et d’enregistrement<br>`capture.tool` | Action système ; certains comportements dépendent des raccourcis configurés. | — | ✓ | ✓ |
| Historique du presse-papiers<br>`desktop.clipboard_history` | Ouvrir l’historique du presse-papiers. | — | ✓ | Cette action est uniquement disponible sur Windows. |
| Emoji et symboles<br>`desktop.emoji` | Action système ; certains comportements dépendent des raccourcis configurés. | — | ✓ | ✓ |
| Centre de notifications<br>`desktop.notifications` | Ouvrir le centre de notifications. | — | ✓ | Cette action est uniquement disponible sur Windows. |
| Verrouiller l’écran<br>`system.lock` | Action système ; certains comportements dépendent des raccourcis configurés. | — | ✓ | ✓ |
| Gestionnaire des tâches / Moniteur d’activité<br>`system.task_manager` | Action système ; certains comportements dépendent des raccourcis configurés. | — | ✓ | ✓ |
| Fenêtre Forcer à quitter<br>`system.force_quit_dialog` | Ouvrir la boîte de dialogue Forcer à quitter. | — | Cette action est uniquement disponible sur macOS. | ✓ |
| Paramètres système<br>`system.settings` | Action système ; certains comportements dépendent des raccourcis configurés. | — | ✓ | ✓ |
| Calculatrice<br>`system.calculator` | Action système ; certains comportements dépendent des raccourcis configurés. | — | ✓ | ✓ |
| Éteindre l’écran<br>`system.display_off` | Action système ; certains comportements dépendent des raccourcis configurés. | — | ✓ | ✓ |
| Mettre en veille<br>`system.sleep` | Action système ; certains comportements dépendent des raccourcis configurés. | — | ✓ | ✓ |

### Médias

| Action / ID | Fonction et contexte d’utilisation | Paramètres | Windows | Mac |
|---|---|---|---|---|
| Lecture / pause<br>`media.play_pause` | Touches multimédias système ; lecteur compatible requis. | — | ✓ | ✓ |
| Piste précédente<br>`media.previous` | Touches multimédias système ; lecteur compatible requis. | — | ✓ | ✓ |
| Piste suivante<br>`media.next` | Touches multimédias système ; lecteur compatible requis. | — | ✓ | ✓ |
| Volume +<br>`media.volume_up` | Touches multimédias système ; lecteur compatible requis. | — | ✓ | ✓ |
| Volume −<br>`media.volume_down` | Touches multimédias système ; lecteur compatible requis. | — | ✓ | ✓ |
| Muet<br>`media.mute` | Touches multimédias système ; lecteur compatible requis. | — | ✓ | ✓ |
| Plus lumineux<br>`display.brightness_up` | Luminosité système ; dépend de l’écran et des pilotes. | — | ✓ | ✓ |
| Moins lumineux<br>`display.brightness_down` | Luminosité système ; dépend de l’écran et des pilotes. | — | ✓ | ✓ |

### Ouvrir

| Action / ID | Fonction et contexte d’utilisation | Paramètres | Windows | Mac |
|---|---|---|---|---|
| Ouvrir un site<br>`open.website` | Ouvre l’URL indiquée dans le navigateur par défaut. | URL (`url`): par défaut : `https://mousetroke.com/` | ✓ | ✓ |
| Ouvrir un dossier<br>`open.folder` | Ouvre le chemin indiqué avec l’app système par défaut. | Chemin (`path`): obligatoire | ✓ | ✓ |
| Ouvrir un fichier<br>`open.file` | Ouvre le chemin indiqué avec l’app système par défaut. | Chemin (`path`): obligatoire | ✓ | ✓ |
| Ouvrir ou activer une application<br>`open.app` | Ouvre l’app ou active sa fenêtre existante. | Application (`app`): obligatoire | ✓ | ✓ |

### Mousetroke

| Action / ID | Fonction et contexte d’utilisation | Paramètres | Windows | Mac |
|---|---|---|---|---|
| Suspendre / reprendre les gestes<br>`mousetroke.toggle_pause` | Suspend ou reprend la reconnaissance des gestes de Mousetroke. | — | ✓ | ✓ |
| Ouvrir les réglages<br>`mousetroke.settings` | Ouvre les réglages de Mousetroke. | — | ✓ | ✓ |
| Afficher la liste des gestes<br>`mousetroke.cheatsheet` | Liste les gestes de l’app actuelle ; prévu pour la phase 2. | — | Cette action est prévue pour la phase 2. | Cette action est prévue pour la phase 2. |

<!-- action-library:generated:end -->

Après une modification du registre ou des traductions, définissez `UPDATE_ACTION_DOCS=1` et exécutez `cargo test -p mousetroke-core --test action_docs` pour actualiser les catalogues des dix langues. Exécutez la même commande sans cette variable pour vérifier que les tableaux enregistrés dans le dépôt sont à jour.
