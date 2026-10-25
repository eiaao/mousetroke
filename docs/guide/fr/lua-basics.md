---
title: "Bases de Lua"
description: "Écrivez votre premier script Lua de geste de souris, apprenez les arguments de ms_action, déboguez avec les journaux et comprenez les limites d’exécution."
order: 8
---

# Bases de Lua

## Quand utiliser Lua

Utilisez une [action simple](actions.md) pour un raccourci, une opération de fenêtre, une URL, une application ou une commande. Choisissez Lua quand un geste doit se brancher selon l’application, transformer du texte ou combiner des opérations. Mousetroke intègre **Lua 5.4**.

## Votre premier script

1. Créez ou sélectionnez un geste dans les paramètres.
2. Sous **Action**, choisissez **Script Lua**. Remplacez toute la zone de texte par le code suivant.
3. Attendez l’indication d’enregistrement, puis déclenchez-le en dehors des paramètres.

```lua
function ms_action(pos, wid, pid, exe, title, mode)
  ms.log("Target: %s, mode: %s", exe, mode)
  ms.show_toast("My first Lua gesture", 1500)
end
```

`function` commence une fonction et `end` la termine. Mousetroke charge le code, puis appelle `ms_action` une fois. Placez votre travail dans cette fonction. Les noms entre parenthèses reçoivent le contexte ; vous pouvez omettre les arguments inutilisés. `ms.log` écrit dans le journal d’exécution et `ms.show_toast` affiche un message pendant 1500 millisecondes.

Le bouton **Premiers pas en Lua** de l’éditeur ouvre cette page. Suivez les [recettes Lua](lua-examples.md) pour faire saisir du texte à un geste de souris ou coller via le presse-papiers.

## Arguments d’entrée

L’entrée complète est `ms_action(pos, wid, pid, exe, title, mode)` :

| Argument | Type Lua | Signification |
|---|---|---|
| `pos` | table `{x=integer, y=integer}` | Dernière position du curseur enregistrée au moment où l’action est distribuée. Les coordonnées peuvent être négatives sur un autre écran. |
| `wid` | integer | Identifiant de la fenêtre cible ; HWND sous Windows, ID de fenêtre sur Mac. Traitez-le comme un identifiant opaque ; il peut valoir 0 sans cible. |
| `pid` | integer | ID de processus de la cible, ou 0. |
| `exe` | string | Identité de l’exécutable sous Windows, en général un chemin complet ; identifiant de bundle ou chemin d’exécutable sur Mac. Vide si indisponible. |
| `title` | string | Titre de la fenêtre cible, ou une chaîne vide. Mac récupère le titre via l’Accessibilité sur le thread de travail. |
| `mode` | string | Exactement `UnderCursor` ou `Focused`, issu de **Préférences → Les gestes agissent sur**. Ce n’est ni le bouton déclencheur ni le nom du geste. |

La cible est capturée pour le geste. `pos` peut différer des coordonnées de départ du geste dans les variables d’action. La saisie au clavier va toujours à l’application active, sauf si vous activez la cible. Sur Mac, `ms.focus_win(wid)` peut activer uniquement l’application cible de ce geste. Le texte d’espace réservé comme `%MS_PID%` n’est pas développé en Lua ; utilisez directement `pid`.

## Une introduction à Lua en cinq minutes

`local` crée une variable. Les chaînes utilisent des guillemets simples ou doubles ; `..` concatène les chaînes. `--` commence un commentaire. `if`, `elseif`, `else`, `end` choisissent une branche. `nil` signifie absence de valeur ; seuls `false` et `nil` sont faux dans les conditions (0 et `""` sont vrais).

```lua
function ms_action(pos, wid, pid, exe)
  local message = "Hello"
  if string.find(string.lower(exe), "firefox", 1, true) then
    message = "Hello, Firefox"
  else
    message = message .. " from Mousetroke"
  end
  ms.show_toast(message)
end
```

Ici, `string.find(s, needle, 1, true)` cherche littéralement et renvoie une position ou `nil`. Le `true` évite d’interpréter la ponctuation comme un motif. `string.match` utilise les motifs Lua, qui diffèrent des expressions régulières. Une table peut stocker des champs nommés ou une liste, dont le premier indice est 1. Une fonction auxiliaire peut renvoyer une valeur :

```lua
local function app_name(identity)
  return string.match(identity, "([^/\\]+)$") or identity
end

function ms_action(pos, wid, pid, exe)
  local target = {name = app_name(exe), process = pid}
  local lines = {target.name, tostring(target.process)}
  ms.show_toast(table.concat(lines, " / "))
end
```

Les chaînes Lua interprètent les échappements avant l’analyse de la séquence de touches. `"\\"` représente une barre oblique inverse et `"\n"` une nouvelle ligne. Les chaînes à crochets longs comme `[[text]]` préservent les barres obliques inverses. Le [guide d’envoi de touches](key-sequences.md) explique la seconde couche de syntaxe.

## Débogage

Utilisez `ms.log("pid=%d title=%s", pid, title)` et consultez **À propos → Ouvrir le journal** :

- Windows : `%APPDATA%\mousetroke\mousetroke.log`.
- Mac : `~/Library/Logs/mousetroke/mousetroke.log`.

Les erreurs de chargement et d’exécution y apparaissent avec `gesture_script` comme nom de chunk. `ms.show_toast("Reached this step")` est une vérification visuelle rapide. Les résultats optionnels des API peuvent être `nil` ; testez-les avant une opération arithmétique ou un accès à une table. Voir la [référence de l’API](lua-api.md) pour les appels non pris en charge sur Mac.

## Exécution et limites

Chaque exécution de script crée une machine virtuelle Lua neuve sur un thread de travail, avec une **limite d’allocation Lua de 8 Mio**. Les globales ne persistent pas entre les gestes. Le moteur d’exécution n’impose pas de limite de nombre d’instructions ni de délai. Évitez les boucles infinies. `ms.sleep(ms)` bloque le thread de travail de ce script, pas le hook d’entrée ni le thread principal de l’interface. `ms.keys` attend la fin de sa séquence ; sur Mac, l’entrée est avancée par le minuteur du thread principal pendant que le thread Lua attend.

Les bibliothèques standard disponibles sont les fonctions de base, `coroutine`, `table`, `io`, `os`, `string`, `utf8`, `math` et `package`. `debug` n’est pas chargé ; le chargement de modules natifs en C est désactivé par le mode sûr de mlua. `os.date` est disponible. Ce n’est pas un bac à sable de sécurité du système d’exploitation : `io`, `os` et les API de processus peuvent accéder à des fichiers ou exécuter des commandes avec vos droits d’utilisateur. N’exécutez que des scripts et des configurations de confiance.

Suivant : [référence de l’API](lua-api.md), [recettes copier-coller](lua-examples.md), [dépannage](troubleshooting.md).
