---
title: "Syntaxe d’envoi de touches"
description: "Syntaxe complète d’envoi de touches de Mousetroke : modificateurs, groupes, touches nommées, répétitions, délais, entrée souris et raccourcis Windows/Mac."
order: 7
---

# Syntaxe d’envoi de touches

## Commencer par un raccourci

Dans **Envoyer des touches**, collez `^c` pour envoyer Ctrl+C sous Windows, ou `#c` pour envoyer Command+C sur Mac. Vous pouvez aussi cliquer sur **Enregistrer**, appuyer sur la combinaison puis cliquer sur **Terminé**. Lua utilise la même syntaxe via `ms.keys(sequence)`.

Les caractères désignent normalement des touches aux positions du clavier américain. `abc` appuie sur A, B, C dans l’ordre ; les majuscules `ABC` n’ajoutent pas Shift. Utilisez `+a` pour Shift+A. Pour de l’Unicode ou du texte fixe, voir [texte et échappement](#texte-et-échappement) et les [recettes presse-papiers](lua-examples.md).

## Modificateurs et groupes

| Préfixe | Windows | Mac |
|---|---|---|
| `^` | Ctrl | Control ⌃ |
| `%` | Alt | Option ⌥ |
| `+` | Shift | Shift ⇧ |
| `#` | Win | Command ⌘ |

Un préfixe s’applique à la touche suivante ou au groupe entre parenthèses. `^+t` envoie Ctrl+Shift+T. `^(ac)` applique Ctrl à la fois à A et à C. `^ac` n’applique Ctrl qu’à A. Les groupes peuvent s’imbriquer ; utilisez des parenthèses équilibrées. Les modificateurs de groupe s’attachent à chaque action de touche, ne comptez donc pas sur le maintien continu du modificateur entre les touches. Un modificateur final isolé n’est pas une pression de touche : utilisez `{CTRL}`, par exemple, quand vous avez besoin de la touche modificateur elle-même.

En Lua, `ms.is_keys_pressed` vérifie l’union de toutes les touches et de tous les modificateurs au moment de la requête sur les deux plateformes : `^ac` exige Control, A et C enfoncés ensemble. Chaque côté satisfait un préfixe ; les noms explicites `{LCTRL}` / `{RCTRL}` conservent leur côté. Les répétitions sont ignorées ; les étapes d’attente, de texte, de souris ou de molette renvoient false. Elle ne reconnaît pas une séquence de pressions passées.

## Touches nommées, répétitions et délais

Utilisez `{ENTER}` pour Entrée. Les noms ne tiennent pas compte de la casse. `{LEFT 3}` appuie trois fois sur Gauche. Un nombre positif répète une touche ; un nombre nul ou négatif retombe à un. `{SLEEP 100}` attend 100 ms ; il exige un entier non négatif. Séparez un nom et son argument par un espace.

Exemple : `^c{SLEEP 100}^v` copie, attend brièvement, puis colle sous Windows. L’équivalent Mac est `#c{SLEEP 100}#v`.

Le tableau ci-dessous énumère les noms acceptés par l’analyseur, y compris les alias et tous les noms numérotés F/pavé numérique. Une seule lettre ASCII, un chiffre ou un caractère de ponctuation américaine pris en charge peut aussi apparaître entre accolades, comme `{a}` ou `{+}`.

| Noms | Signification |
|---|---|
| `{ENTER}`, `{RETURN}` | Entrée |
| `{ESC}` | Échap |
| `{TAB}` | Tab |
| `{BACKSPACE}` | Retour arrière |
| `{SPACE}` | Espace |
| `{CAPSLOCK}` | Verr. Maj |
| `{INSERT}`, `{INS}` | Inser (Windows uniquement) |
| `{DELETE}`, `{DEL}` | Suppr / suppression vers l’avant |
| `{HOME}`, `{END}` | Début / Fin |
| `{PGUP}`, `{PGDN}` | Page préc. / Page suiv. |
| `{UP}`, `{DOWN}`, `{LEFT}`, `{RIGHT}` | Touches fléchées |
| `{PRTSC}` | Impr. écran (Windows uniquement) |
| `{BREAK}` | Break (Windows uniquement) |
| `{MEDIA_NEXT}`, `{MEDIA_PREV}`, `{MEDIA_STOP}`, `{MEDIA_PLAY}`, `{MEDIA_PLAY_PAUSE}` | Touches média (Windows uniquement dans les séquences de touches) |
| `{VOLUME_MUTE}`, `{VOLUME_DOWN}`, `{VOLUME_UP}` | Touches de volume (Windows uniquement dans les séquences de touches) |
| `{CTRL}`, `{ALT}`, `{SHIFT}` | Control / Alt (Option sur Mac) / Shift |
| `{LCTRL}`, `{RCTRL}`, `{LSHIFT}`, `{RSHIFT}`, `{LALT}`, `{RALT}` | Touches modificateurs gauche/droite ; Alt est Option sur Mac |
| `{LWIN}`, `{WIN}`, `{RWIN}` | Touche Windows ; Command sur Mac |
| `{F1}`, `{F2}`, `{F3}`, `{F4}`, `{F5}`, `{F6}`, `{F7}`, `{F8}`, `{F9}`, `{F10}`, `{F11}`, `{F12}`, `{F13}`, `{F14}`, `{F15}`, `{F16}`, `{F17}`, `{F18}`, `{F19}`, `{F20}`, `{F21}`, `{F22}`, `{F23}`, `{F24}` | Touches de fonction ; Mac prend en charge F1–F20 |
| `{N0}`, `{N1}`, `{N2}`, `{N3}`, `{N4}`, `{N5}`, `{N6}`, `{N7}`, `{N8}`, `{N9}` | Chiffres du pavé numérique 0–9 |
| `{MOUSE_LEFT}`, `{MOUSE_RIGHT}`, `{MOUSE_MIDDLE}`, `{MOUSE_X1}`, `{MOUSE_X2}` | Clics de souris (Windows uniquement) |
| `{MOUSE_WHEEL}`, `{MOUSE_HWHEEL}` | Molette : p. ex. {MOUSE_WHEEL 120}, {MOUSE_HWHEEL -120} (Windows uniquement) |
| `{SLEEP}` | Délai : p. ex. {SLEEP 100}, en millisecondes |

## Boutons de la souris et molette

Windows prend en charge `{MOUSE_LEFT}`, `{MOUSE_RIGHT}`, `{MOUSE_MIDDLE}`, `{MOUSE_X1}` et `{MOUSE_X2}` comme clics. `{MOUSE_LEFT 2}` envoie deux clics ; il ne repositionne pas le curseur.

`{MOUSE_WHEEL 120}` fait défiler d’un cran vers le haut ; `-120` vers le bas. `{MOUSE_HWHEEL 120}` fait défiler vers la droite ; `-120` vers la gauche. L’argument est un delta de molette signé, pas un nombre de répétitions. C’est la convention de l’entrée injectée ; elle diffère du signe utilisé par les étapes de molette enregistrées des gestes.

Mac analyse ces jetons mais saute les actions de souris et de molette pendant l’exécution de l’entrée. Ils ne peuvent pas servir à synthétiser des clics ou un défilement Mac via `ms.keys`.

## Texte et échappement

Le mode texte prioritaire n’est disponible qu’avec `ms.keys(s, true)` en Lua ; l’éditeur Envoyer des touches utilise toujours le mode touches. Les caractères ordinaires deviennent du texte Unicode, mais `^`, `%`, `+`, `#`, les parenthèses et l’accolade ouvrante déclenchent toujours la syntaxe. Un caractère unique entre accolades reste une action de touche en mode texte prioritaire.

Pour des raccourcis en mode texte prioritaire, mettez la touche entre accolades : `^{c}` envoie Ctrl+C et `#{c}` envoie Command+C. Un `^c` sans accolade dans ce mode émet le texte `c` et abandonne le modificateur. Vous pouvez aussi faire un appel séparé `ms.keys("^c")` en mode touches normal.

| Caractère littéral | Orthographe dans la séquence |
|---|---|
| `^` | `{^}` |
| `%` | `{%}` |
| `+` | `{+}` |
| `#` | `{#}` |
| `(` | `{(}` |
| `)` | `{)}` |
| `{` | `{{}` |
| `}` | `}` (sans accolade) |

La barre oblique inverse n’est pas un caractère d’échappement de séquence de touches. Dans une chaîne Lua entre guillemets, écrivez `"\\"` pour une seule barre oblique inverse, ou utilisez une chaîne à crochets longs. Pour du texte arbitraire contenant des symboles de syntaxe, la [méthode du presse-papiers](lua-examples.md#coller-puis-restaurer-le-texte-du-presse-papiers) évite les deux couches d’échappement.

## Raccourcis usuels

Ils dépendent des réglages de raccourcis de l’application de destination et du système d’exploitation.

| Objectif | Séquence Windows | Séquence Mac |
|---|---|---|
| Copier | `^c` | `#c` |
| Coller | `^v` | `#v` |
| Couper | `^x` | `#x` |
| Tout sélectionner | `^a` | `#a` |
| Annuler | `^z` | `#z` |
| Nouvel onglet | `^t` | `#t` |
| Fermer l’onglet | `^w` | `#w` |
| Rouvrir l’onglet fermé | `^+t` | `#+t` |
| Onglet suivant | `^{TAB}` | `^{TAB}` |
| Onglet précédent | `^+{TAB}` | `^+{TAB}` |
| Changer d’application | `%{TAB}` | `#{TAB}` |
| Bureau précédent/suivant | `#^{LEFT}` / `#^{RIGHT}` | `^{LEFT}` / `^{RIGHT}` |
| Capture d’une zone | `#+s` | `#+4` |
| Afficher le bureau | `#d` | `{F11}` |
| Gestionnaire des tâches / Activity Monitor | `^+{ESC}` | Utiliser Ouvrir une appli : `com.apple.ActivityMonitor` |
| Verrouiller l’écran | `#l` | `^#q` |

## Pourquoi Ctrl+Alt+Suppr ne peut pas être envoyé

Ctrl+Alt+Suppr est la séquence d’attention sécurisée de Windows. Windows ignore l’entrée synthétique ordinaire pour cette séquence, donc `^%{DEL}` ne peut pas ouvrir cet écran de sécurité. Utilisez `^+{ESC}` pour le Gestionnaire des tâches, `#l` pour verrouiller, ou une action **Exécuter une commande** contenant `shutdown /l` pour se déconnecter (ce qui met fin à la session).

## Limites et remarques par plateforme

L’analyseur commun n’a pas de limite globale de longueur de chaîne ni de nombre de répétitions. Les nombres de répétitions et les délais sont stockés comme des valeurs non signées de 32 bits ; les deltas de molette comme des valeurs signées de 32 bits. Les valeurs hors limites peuvent boucler lors de la conversion, utilisez donc des nombres et des délais petits et raisonnables. Un mauvais nom de touche ou un argument requis manquant produit une erreur d’analyse ; l’éditeur valide pendant la saisie.

Windows exécute les séquences sur des threads de travail et s’endort pendant `{SLEEP ms}`. Mac envoie l’entrée par lots pilotés par un minuteur. Les chaînes d’actions Mac de plus de 16 384 octets sont rejetées ; pour l’entrée Lua déjà analysée, les limites sont de 16 384 actions et 16 384 octets par action de texte. Sa file d’attente contient jusqu’à 64 entrées ; l’excédent d’entrée est abandonné et journalisé.

Sur Mac, F1–F20, les chiffres du pavé numérique et les touches de navigation mappées sont disponibles. Insert/INS, PRTSC, BREAK, F21–F24, les jetons média/volume et les jetons souris/molette ne sont pas pris en charge et sont ignorés. Utilisez **Média et système** pour le média et le volume. `ms.keys_down` et `ms.keys_up` sont réservés à Windows ; voir l’[API Lua](lua-api.md).
