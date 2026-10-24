---
title: "Recettes Lua"
description: "Copiez des scripts Lua de gestes de souris prêts à l’emploi pour le texte, le collage presse-papiers, les dates, la recherche web, les fenêtres, la luminosité et la pause."
order: 10
---

# Recettes Lua

Collez un bloc complet dans une action **Script Lua** d’un geste. Chaque bloc définit `ms_action`. Essayez d’abord les recettes de texte dans un document texte jetable. Les [bases de Lua](lua-basics.md) expliquent l’éditeur et l’[API Lua](lua-api.md) liste la prise en charge par plateforme.

Les recettes multiplateformes utilisent `package.config` pour choisir `^` (Control sous Windows) ou `#` (Command sur Mac). Les recettes presse-papiers ne conservent que le **texte**, pas les images ni les autres formats. Leurs attentes de 200 ms sont des exemples, pas une garantie des applications ; augmentez-les si la destination est lente. Elles remplacent temporairement le contenu du presse-papiers et restaurent le texte enregistré, même si vous copiez autre chose pendant cet intervalle.

## Saisir un texte fixe

Objectif : insérer une phrase réutilisable avec un seul geste de souris.

```lua
function ms_action()
  ms.keys("Hello from Mousetroke.", true)
end
```

Le mode texte prioritaire prend en charge l’Unicode sur les deux plateformes. Il analyse toujours les symboles de syntaxe ; utilisez la recette presse-papiers pour la ponctuation arbitraire comme `%`, `+` ou les accolades.

## Coller puis restaurer le texte du presse-papiers

Objectif : coller du texte littéral, puis restaurer le texte qui se trouvait dans le presse-papiers.

```lua
function ms_action()
  local mod = package.config:sub(1, 1) == "\\" and "^" or "#"
  local saved = ms.get_clipboard_text()
  ms.set_clipboard_text("Price: 50% + tax {example}")
  ms.keys(mod .. "v")
  ms.sleep(200)
  ms.set_clipboard_text(saved or "")
end
```

Cela contourne l’échappement de séquence de touches pour le texte inséré. S’il n’y avait pas de texte dans le presse-papiers, le presse-papiers final contient un texte vide ; les autres formats d’origine ne peuvent pas être restaurés avec ces API.

## Un texte différent selon l’appli

Objectif : utiliser l’identité de l’exécutable cible du geste pour choisir une phrase.

```lua
function ms_action(pos, wid, pid, exe)
  local name = string.lower(exe)
  local text = "Hello from Mousetroke"
  if string.find(name, "firefox", 1, true) then
    text = "Hello Firefox"
  elseif name == "com.apple.safari" then
    text = "Hello Safari"
  end
  ms.keys(text, true)
end
```

Sous Windows, les identités sont en général des chemins d’exécutable complets ; Mac fournit en général un identifiant de bundle. Cela vérifie la cible capturée, tandis que la saisie va à l’application active. Utilisez d’abord `ms.focus_win(wid)` si vous voulez activer l’application cible.

## Insérer la date du jour

Objectif : insérer la date locale au format année-mois-jour.

```lua
function ms_action()
  ms.keys(os.date("%Y-%m-%d"), true)
end
```

`os.date` est disponible dans le moteur d’exécution. Les caractères `%` relèvent de la syntaxe de format de date Lua ; les chiffres et tirets obtenus sont sûrs pour la saisie en mode texte prioritaire.

## Rechercher le texte sélectionné sur un site

Objectif : copier la sélection et ouvrir une requête Google encodée en UTF-8.

```lua
local function url_encode(text)
  return (text:gsub("([^A-Za-z0-9%-._~])", function(c)
    return string.format("%%%02X", string.byte(c))
  end))
end

function ms_action()
  local mod = package.config:sub(1, 1) == "\\" and "^" or "#"
  local saved = ms.get_clipboard_text()
  ms.set_clipboard_text("")
  ms.keys(mod .. "c")
  ms.sleep(200)
  local text = ms.get_clipboard_text() or ""
  ms.set_clipboard_text(saved or "")
  if text ~= "" then
    ms.open("https://www.google.com/search?q=" .. url_encode(text))
  else
    ms.show_toast("No selected text copied")
  end
end
```

Sélectionnez du texte dans l’application active avant de déclencher. Vider d’abord le presse-papiers évite de rechercher l’ancien texte du presse-papiers si Copier ne fait rien. Lua n’a pas de substitution automatique du texte sélectionné ; l’action plus simple [Ouvrir une URL](actions.md#ouvrir-une-url) peut collecter la sélection pour vous.

## Encadrer le texte sélectionné de guillemets ou de crochets

Objectif : remplacer le texte sélectionné par une version entre guillemets dans un champ modifiable.

```lua
function ms_action()
  local mod = package.config:sub(1, 1) == "\\" and "^" or "#"
  local saved = ms.get_clipboard_text()
  ms.set_clipboard_text("")
  ms.keys(mod .. "c")
  ms.sleep(200)
  local text = ms.get_clipboard_text() or ""
  if text ~= "" then
    ms.set_clipboard_text('"' .. text .. '"')
    ms.keys(mod .. "v")
    ms.sleep(200)
  end
  ms.set_clipboard_text(saved or "")
end
```

Pour des crochets, remplacez l’expression d’encadrement par `"[" .. text .. "]"`. Copier doit laisser le texte sélectionné pour que Coller le remplace. Les pages en lecture seule ne peuvent pas être modifiées ainsi.

## Basculer l’affichage au premier plan

Objectif : basculer l’état de premier plan d’une fenêtre Windows. Choisissez **Actions de fenêtre → Toujours au premier plan / non** pour cette recette : cette action peut interroger et basculer l’état réel. L’API Lua ne définit qu’un état demandé ; elle n’a pas de getter de premier plan et chaque geste obtient une machine virtuelle neuve. Mac ne prend pas en charge le contrôle du premier plan entre applications.

Si vous voulez un geste Lua séparé « épingler cette fenêtre » sous Windows, utilisez :

```lua
function ms_action(pos, wid)
  if package.config:sub(1, 1) ~= "\\" then
    ms.show_toast("Always-on-top is unavailable on Mac")
    return
  end
  if wid ~= 0 then ms.set_win_top(wid, true) end
end
```

Utilisez `false` au lieu de `true` pour un geste de désépinglage séparé. Une variable globale Lua serait réinitialisée à chaque exécution et n’implémenterait pas une bascule.

## Agrandir ou restaurer

Objectif : basculer l’état agrandi de la fenêtre cible sous Windows ou le zoom sur la zone de travail sur Mac.

```lua
function ms_action(pos, wid)
  if wid == 0 then return end
  local state = ms.get_win_state(wid)
  if state == nil then
    ms.show_toast("Window unavailable; check Accessibility permission on Mac")
    return
  end
  ms.set_win_state(wid, state == "max" and "normal" or "max")
end
```

Mac utilise le même bouton de zoom AX que l’action de fenêtre intégrée. La requête d’état reconnaît les fenêtres à la taille de la zone de travail ; les applications qui zooment à la taille de leur contenu peuvent toujours rapporter normal, utilisez donc **Actions de fenêtre → Agrandir / restaurer** pour ces applications. Une autorisation d’Accessibilité manquante ou une fenêtre indisponible donne nil. Il n’existe pas d’API Lua de position/taille de fenêtre pour déplacer une fenêtre vers un autre écran.

## Luminosité sous le curseur

Objectif : augmenter de 5 points de pourcentage la luminosité de l’écran sous le curseur sous Windows.

```lua
function ms_action()
  local p = ms.get_mouse_pos()
  local value = ms.set_brightness(p, 5)
  if value ~= nil then
    ms.show_toast("Brightness: " .. value)
  else
    ms.show_toast("Brightness API unavailable for this screen/platform")
  end
end
```

Changez `5` en `-5` pour un second geste, plus sombre. Windows a besoin de matériel DDC/CI ou WMI compatible ; WMI retombe sur son premier périphérique de luminosité, qui peut être un autre écran. Le succès est un pourcentage 0–100 ; l’échec est nil sur les deux plateformes. Mac renvoie nil ici : son backend existant n’envoie que des touches de luminosité et ne peut ni renvoyer un pourcentage ni appliquer un delta en points de pourcentage. Utilisez **Média et système → Luminosité + / Luminosité −** pour les touches de luminosité de l’écran intégré, sans sélection de l’écran sous le curseur ni DDC/CI externe.

## Ouvrir un dossier

Objectif : ouvrir le dossier Téléchargements de l’utilisateur actuel avec le navigateur de fichiers de la plateforme.

```lua
function ms_action()
  if package.config:sub(1, 1) == "\\" then
    ms.cmd('explorer.exe "%USERPROFILE%\\Downloads"')
  else
    ms.cmd('/usr/bin/open "$HOME/Downloads"')
  end
end
```

Ces variables d’environnement sont développées par le shell du système d’exploitation, pas par les variables d’action de Mousetroke. Cela suppose l’emplacement habituel du dossier Téléchargements.

## Lancer un programme avec des arguments

Objectif : passer une chaîne d’arguments à un programme.

```lua
function ms_action()
  if package.config:sub(1, 1) == "\\" then
    ms.run("cmd.exe", "/K echo Hello from Mousetroke")
  else
    ms.run("/usr/bin/say", '"Hello from Mousetroke"')
  end
end
```

Windows ouvre une fenêtre de commande affichant un message. Mac prononce la phrase avec `say`. `ms.run` accepte une seule chaîne d’arguments, pas une table, et ne renvoie aucun handle de processus.

## Ouvrir le Gestionnaire des tâches ou Activity Monitor

Objectif : ouvrir la visionneuse de processus de la plateforme.

```lua
function ms_action()
  if package.config:sub(1, 1) == "\\" then
    ms.keys("^+{ESC}")
  else
    ms.run("com.apple.ActivityMonitor")
  end
end
```

Windows utilise Ctrl+Shift+Esc. Mac lance Activity Monitor par identifiant de bundle.

## Verrouiller l’écran

Objectif : verrouiller la session actuelle avec son raccourci système.

```lua
function ms_action()
  if package.config:sub(1, 1) == "\\" then
    ms.keys("#l")
  else
    ms.keys("^#q")
  end
end
```

Mac utilise Control+Command+Q. Windows utilise Win+L ; Ctrl+Alt+Suppr ne peut pas être synthétisé comme une séquence de touches ordinaire.

## Mettre les gestes en pause pendant dix minutes

Objectif : suspendre temporairement le moteur, puis le reprendre pendant que Mousetroke continue de tourner.

```lua
function ms_action()
  ms.set_paused(true)
  ms.show_toast("Gestures paused for ten minutes", 2000)
  ms.sleep(10 * 60 * 1000)
  ms.set_paused(false)
  ms.show_toast("Gestures resumed", 1500)
end
```

Cela fonctionne sur les deux plateformes et maintient un thread de travail Lua endormi pendant dix minutes. Ce n’est pas un minuteur persistant : quitter ou redémarrer y met fin. Vous pouvez reprendre manuellement pendant l’attente, mais le script émet quand même une reprise quand son attente se termine. Évitez de lancer plusieurs copies. Voir les [limites d’exécution](lua-basics.md#exécution-et-limites).
