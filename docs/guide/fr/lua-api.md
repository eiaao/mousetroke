---
title: "API Lua"
description: "Référence complète de l’API ms pour Lua 5.4 : signatures, valeurs de retour, exemples et remarques de prise en charge Windows/macOS."
order: 9
---

# API Lua

Voici toutes les fonctions enregistrées sur `ms` par le moteur d’exécution commun. Les exemples sont des instructions à placer dans `ms_action`, où `pos`, `wid`, `pid`, `exe`, `title` et `mode` sont disponibles. Voir les [bases de Lua](lua-basics.md) pour un script complet et les [recettes](lua-examples.md) pour des actions prêtes à l’emploi.

## Conventions

Version de Lua : **5.4** (`mlua` avec `lua54` et `vendored`). `nil` signifie absence de valeur ; `?` marque un paramètre facultatif. Les fonctions ne renvoyant aucune valeur sont notées `-> nil`. Un argument facultatif omis équivaut à nil. Passez un vrai booléen pour les arguments booléens. Les paramètres entiers tronquent les fractions finies vers zéro et rejettent les valeurs hors de leur plage entière sous-jacente ; utilisez des entiers dans les scripts.

Les tables de position utilisent des champs nommés `{x=..., y=...}`, pas des éléments de tableau. Les appels non pris en charge sur Mac journalisent en général une fois par fonction et par processus ; les getters renvoient nil et les setters ne font rien. Un setter sans valeur de retour n’est pas une preuve de succès. Les chaînes de l’API Lua ne reçoivent pas de substitution de variables d’action.

## Flux et retour d’information

### `ms.log`

`ms.log(...) -> nil`

Accepte n’importe quelles valeurs Lua. Avec une chaîne de format et d’autres arguments, prend en charge `%s`, `%d`, `%f`, `%q` et `%%` ; sinon, elle joint les valeurs par des espaces. `%q` n’ajoute pas de guillemets Lua. Écrit une ligne `Lua:` dans le journal d’exécution sur les deux plateformes. Ne renvoie aucune valeur.

```lua
ms.log("pid=%d app=%s", pid, exe)
```

### `ms.sleep`

`ms.sleep(milliseconds: integer) -> nil`

Attend un nombre de millisecondes non négatif (non signé 64 bits). Bloque ce thread de travail de script sur les deux plateformes ; le moteur de gestes/thread principal continue. Aucune valeur de retour et aucun délai d’exécution de script n’est installé.

```lua
ms.sleep(100)
```

### `ms.is_paused`

`ms.is_paused() -> boolean`

Sans paramètres. Renvoie l’indicateur de pause manuelle du moteur sur les deux plateformes. Il ne rapporte ni la suspension automatique en plein écran ni une exclusion d’application.

```lua
ms.log(ms.is_paused())
```

### `ms.set_paused`

`ms.set_paused(paused: boolean) -> nil`

`true` met en pause ; `false` reprend. Aucune valeur de retour. Pris en charge sur les deux plateformes. Windows publie une commande vers le thread de la zone de notification, donc un `is_paused()` immédiat peut encore voir l’ancienne valeur. Mac applique via le pont du thread principal.

```lua
ms.set_paused(true)
```

### `ms.show_settings`

`ms.show_settings() -> nil`

Sans paramètres ni valeur de retour. Ouvre ou amène les paramètres au premier plan sur les deux plateformes.

```lua
ms.show_settings()
```

### `ms.show_toast`

`ms.show_toast(message: string, duration_ms?: integer) -> nil`

`message` est le texte affiché. La durée facultative, non négative, est en millisecondes, par défaut **1200**. Affiche un message en surimpression sur les deux plateformes ; ne renvoie aucune valeur.

```lua
ms.show_toast("Done", 1500)
```

## Entrée clavier

### `ms.keys`

`ms.keys(sequence: string, text_by_default?: boolean) -> nil`

Analyse et envoie la [séquence de touches](key-sequences.md) ; le mode texte prioritaire vaut `false` par défaut. Renvoie après l’envoi ; une syntaxe mal formée lève une erreur Lua. Les deux plateformes prennent en charge l’entrée clavier/texte ordinaire. Mac saute les touches non prises en charge, les actions de souris et de molette, et applique ses limites de file/texte. Elle attend sa file d’entrée du thread principal, qui peut aussi contenir d’autres séquences.

```lua
ms.keys("Hello", true)
```

### `ms.keys_down`

`ms.keys_down(sequence: string) -> nil`

Réservé à Windows : analyse en mode touches et enfonce les touches/modificateurs sans les relâcher. Les répétitions ne sont pas répétées dans cette phase ; les actions de molette se déclenchent une fois. Une syntaxe invalide lève une erreur Lua. Associez-le à `keys_up` pour relâcher. Mac journalise comme non pris en charge et ne fait rien. Aucune valeur de retour.

```lua
ms.keys_down("^a")
ms.sleep(50)
ms.keys_up("^a")
```

### `ms.keys_up`

`ms.keys_up(sequence: string) -> nil`

Réservé à Windows : relâche la séquence analysée dans l’ordre inverse des actions, les modificateurs étant relâchés en dernier. Les étapes d’attente attendent toujours. Les erreurs de syntaxe lèvent une erreur Lua. Mac journalise comme non pris en charge et ne fait rien. Aucune valeur de retour.

```lua
ms.keys_down("{SHIFT}")
ms.keys_up("{SHIFT}")
```

### `ms.is_keys_pressed`

`ms.is_keys_pressed(sequence: string) -> boolean | nil`

Les deux plateformes vérifient chaque touche du clavier **et ses modificateurs de préfixe** : `"^a"` exige Control et A. Chaque côté satisfait un préfixe ; les noms explicites comme `{RCTRL}` exigent ce côté. Sur Mac, `^` = Control, `%` = Option, `+` = Shift et `#` = Command. Plusieurs étapes signifient que toutes les touches/modificateurs listés doivent être enfoncés ensemble, et non qu’ils ont été pressés dans l’ordre ; les répétitions n’ajoutent aucune exigence. Les actions souris/molette, attente ou texte rendent le test faux ; une séquence vide renvoie true. Une syntaxe invalide renvoie false. Mac lit l’état combiné de la session et renvoie false sans l’autorisation d’Accessibilité, avec une journalisation unique.

```lua
if ms.is_keys_pressed("^a") then
  ms.show_toast("Ctrl and A are down")
end
```

## Presse-papiers et fichiers

### `ms.get_clipboard_text`

`ms.get_clipboard_text() -> string | nil`

Sans paramètres. Renvoie le texte du presse-papiers, ou `nil` si aucun texte n’est disponible. Pris en charge sur les deux plateformes ; ne récupère pas la sélection automatiquement.

```lua
local text = ms.get_clipboard_text()
ms.log(text or "No text")
```

### `ms.set_clipboard_text`

`ms.set_clipboard_text(text: string) -> nil`

Remplace le contenu du presse-papiers par le texte sur les deux plateformes ; aucune valeur de retour ni indicateur de succès. Les autres formats du presse-papiers ne sont pas conservés. Voir les [recettes presse-papiers](lua-examples.md).

```lua
ms.set_clipboard_text("Hello from Mousetroke")
```

### `ms.read_text`

`ms.read_text(path: string) -> string | nil`

Lit tout le fichier en UTF-8 sur les deux plateformes. Renvoie `nil` en cas d’erreur de lecture ou de décodage UTF-8. Les chemins relatifs utilisent le répertoire de travail du processus, pas un dossier de scripts documenté. Les lectures comptent dans la mémoire disponible.

```lua
local text = ms.read_text("config.json")
ms.log(text and #text or 0)
```

## Windows

### `ms.find_win_by_class`

`ms.find_win_by_class(pattern: string) -> table | nil`

Recherche les fenêtres de premier niveau visibles avec une **expression régulière Rust**, sur les noms de classe sous Windows ou les **identifiants de bundle** d’application sur Mac (par exemple `^com\\.apple\\.Safari$` dans une chaîne Lua). Renvoie jusqu’à 32 entrées dans un tableau indexé à partir de 1 : `{text=title, hwnd=window_id}` ; aucune correspondance ou regex invalide donne une table vide. La correspondance tient compte de la casse, sauf si le motif utilise un indicateur comme `(?i)` ; ce ne sont pas des motifs Lua. Mac lit les titres via l’Accessibilité, renvoie nil sans autorisation et journalise une fois. Son `hwnd` est un CGWindowID accepté par `focus_win` et les API d’état de fenêtre. Seules les fenêtres actuellement à l’écran sont recherchées sur Mac.

```lua
local wins = ms.find_win_by_class("^Notepad$")
if wins and wins[1] then ms.focus_win(wins[1].hwnd) end
```

### `ms.find_win_by_title`

`ms.find_win_by_title(pattern: string) -> table | nil`

Recherche les titres de fenêtres sur les deux plateformes avec une expression régulière Rust, en renvoyant les mêmes entrées `{text, hwnd}` indexées à partir de 1 que `find_win_by_class`, 32 au maximum. Une regex invalide donne une table vide. Mac utilise les titres de fenêtres AX, exige l’autorisation d’Accessibilité et ne recherche que les fenêtres actuellement à l’écran.

```lua
local wins = ms.find_win_by_title("(?i)notepad")
if wins then ms.log(#wins) end
```

### `ms.focus_win`

`ms.focus_win(wid: integer) -> nil`

Active une fenêtre/application ; aucune valeur de retour ni indicateur de succès. Windows prend un HWND. Mac accepte la cible du geste ou un CGWindowID découvert, restaure et remonte sa fenêtre AX, puis active l’application propriétaire via le pont du thread principal. Les appels AX Mac s’exécutent sur le thread de travail du script, réutilisent la recherche intégrée de fenêtre par PID/limites et exigent l’autorisation d’Accessibilité ; les ID périmés ou les fenêtres AX indisponibles sont ignorés. Les applications dont les limites de fenêtres se chevauchent exactement peuvent être ambiguës.

```lua
if wid ~= 0 then ms.focus_win(wid) end
```

### `ms.set_win_state`

`ms.set_win_state(wid: integer, state: string) -> nil`

Sous Windows : `"min"` réduit, `"max"` agrandit ; **toute autre chaîne restaure**, y compris `"normal"` et `"norm"`. Sur Mac : `"min"` utilise AXMinimized ; `"max"` restaure/sort du plein écran et appuie sur le même AXZoomButton que l’action intégrée quand la fenêtre ne remplit pas la zone de travail de son écran. `"normal"`, `"norm"` et `"restore"` restaurent/sortent du plein écran et annulent le zoom sur la zone de travail. Mac accepte aussi `"fullscreen"` via AXFullScreen quand l’application le permet ; tout autre nom est ignoré et journalisé une fois. Les valeurs sont sensibles à la casse. Aucune valeur de retour. Mac exige l’autorisation d’Accessibilité ; AX s’exécute sur le thread de travail et la géométrie de l’écran passe par le pont du thread principal. Les transitions peuvent être asynchrones.

Le Zoom Mac est défini par l’application : un zoom à la taille du contenu peut toujours rapporter `"normal"` et ne peut pas être restauré de façon fiable avec cette API ; des demandes `"max"` répétées peuvent le basculer. Pour ces applications, utilisez la bascule intégrée **Agrandir / restaurer**. Il n’existe pas d’indicateur AX public universel d’état de zoom.

```lua
if wid ~= 0 then ms.set_win_state(wid, "max") end
```

### `ms.get_win_state`

`ms.get_win_state(wid: integer) -> string | nil`

Sous Windows, renvoie `"min"`, `"max"` ou `"normal"`. Ce n’est pas une requête de premier plan et elle ne valide pas un handle périmé ; un handle invalide peut paraître normal. Sur Mac, renvoie `"min"`, `"fullscreen"`, `"max"` (le cadre correspond à une zone de travail d’écran à moins de 2 points logiques) ou `"normal"`. Mac n’a pas de getter AX public universel d’état de zoom, donc un zoom à la taille du contenu rapporte normal. Elle renvoie nil en cas d’autorisation manquante, de fenêtre/état AX indisponible ou d’ID périmé ; le refus d’autorisation est journalisé une fois.

```lua
if wid ~= 0 then ms.log(ms.get_win_state(wid)) end
```

### `ms.set_win_top`

`ms.set_win_top(wid: integer, top: boolean) -> nil`

Sous Windows, active (`true`) ou désactive (`false`) le premier plan permanent sans déplacer ni redimensionner la fenêtre ; aucune valeur de retour. Cela définit un état, sans le basculer ni renvoyer l’état précédent. Mac ne fait rien et journalise une fois : macOS n’a pas d’API publique pour épingler la fenêtre d’une autre application, et l’action intégrée de premier plan n’est pas prise en charge non plus. Pour une bascule, choisissez **Actions de fenêtre → Toujours au premier plan / non** dans l’éditeur Windows.

```lua
if wid ~= 0 then ms.set_win_top(wid, true) end
```

## Souris et écran

### `ms.get_mouse_pos`

`ms.get_mouse_pos() -> table`

Sans paramètres. Les deux plateformes renvoient `{x=integer, y=integer}` pour la position actuelle du curseur, qui peut différer du `pos` d’entrée. Windows utilise des pixels d’écran ; Mac utilise des points logiques globaux Quartz, avec Y croissant vers le bas. Mac renvoie `{x=0,y=0}` si la requête du curseur échoue.

```lua
local p = ms.get_mouse_pos()
ms.log("%d,%d", p.x, p.y)
```

### `ms.set_mouse_pos`

`ms.set_mouse_pos(x: integer, y: integer) -> nil`

Déplace le curseur vers des coordonnées globales d’écran sur les deux plateformes ; aucune valeur de retour. Les coordonnées peuvent être négatives. Mac publie un événement de mouvement balisé pour ne pas démarrer un autre geste.

```lua
local p = ms.get_mouse_pos()
ms.set_mouse_pos(p.x + 20, p.y)
```

### `ms.get_screen`

`ms.get_screen(x: integer, y: integer) -> table | nil`

Renvoie le rectangle complet de l’écran `{left, top, right, bottom}` en coordonnées globales entières, pas la zone de travail utilisable. La largeur est `right-left` ; la hauteur est `bottom-top`. Windows choisit l’écran le plus proche si le point est en dehors de tous les écrans. Mac choisit l’affichage contenant le point, ou retombe sur le premier affichage en cache ; il peut renvoyer `nil` si aucun résultat d’écran/pont n’est disponible. Les deux plateformes prennent en charge cet appel.

```lua
local p = ms.get_mouse_pos()
local s = ms.get_screen(p.x, p.y)
if s then ms.log(s.right - s.left, s.bottom - s.top) end
```

### `ms.get_brightness`

`ms.get_brightness(pos?: table) -> integer | nil`

Le `{x=integer,y=integer}` facultatif choisit l’écran ; omis/nil ou une position incomplète retombe sur le curseur actuel. Renvoie **0 à 100 pour cent en cas de succès, nil en cas d’échec** sur chaque plateforme. Windows utilise DDC/CI avec un recours à WMI. WMI utilise son premier périphérique de luminosité, pas un écran apparié au curseur. Mac renvoie actuellement nil et journalise une fois : son backend de luminosité existant n’envoie que les événements de touches de luminosité de l’écran intégré, qui ne peuvent pas lire un pourcentage. Le DDC externe n’est pas pris en charge. Seul le wrapper historique Windows `WG.GetBrightness` conserve -1 en cas d’échec.

```lua
local value = ms.get_brightness()
if value ~= nil then ms.show_toast(tostring(value)) end
```

### `ms.set_brightness`

`ms.set_brightness(pos: table | nil, delta: integer) -> integer | nil`

Malgré son nom, cette fonction ajuste de `delta` points de pourcentage ; elle ne définit pas un pourcentage absolu. Passez une table nommée `{x=...,y=...}` ou **nil comme premier argument** pour utiliser le curseur. En cas de succès, elle renvoie le nouveau **pourcentage 0–100**, borné à cette plage ; en cas d’échec, elle renvoie **nil sur les deux plateformes**. Windows utilise la même sélection DDC/CI et WMI que `get_brightness`. Seul le wrapper historique Windows `WG.SetBrightness` conserve -1 en cas d’échec. Mac renvoie nil et journalise une fois : le backend existant Média et système n’envoie que les touches de luminosité de l’écran intégré, sans lecture numérique ni contrôle par points de pourcentage. Aucun changement de luminosité n’est effectué par cet appel Lua ; utilisez ces actions intégrées pour les touches de luminosité. Le DDC externe reste non pris en charge.

```lua
local value = ms.set_brightness(nil, 5)
ms.log(value)
```

## URL et processus

### `ms.open`

`ms.open(url: string) -> nil`

Ouvre une URL avec le gestionnaire par défaut ; aucune valeur de retour ni indicateur de succès. Windows utilise l’ouverture du shell et accepte aussi un chemin de fichier/dossier. Mac attend une URL ; utilisez une URL `file://` correctement encodée pour un fichier, ou `ms.cmd` pour appeler `/usr/bin/open` sur un chemin. Aucune substitution de variables n’est effectuée.

```lua
ms.open("https://mousetroke.com/")
```

### `ms.run`

`ms.run(file: string, args?: string) -> nil`

Lance avec une **chaîne** d’arguments facultative, pas un tableau Lua ; ne renvoie ni valeur ni PID. Windows utilise un lancement ordinaire sans élévation avec le comportement RunAgain. Mac accepte un identifiant de bundle, un chemin d’application ou un chemin d’exécutable absolu ; les arguments sont découpés en gérant les guillemets. Le lancement d’exécutable peut retomber sur un processus d’arrière-plan si LaunchServices le refuse. Il demande une nouvelle instance. Pas de développement de variables ; utilisez la concaténation Lua. La sortie du processus n’est pas renvoyée.

```lua
if package.config:sub(1, 1) == "\\" then
  ms.run("cmd.exe", "/K echo Hello")
else
  ms.run("/usr/bin/say", '"Hello from Mousetroke"')
end
```

### `ms.cmd`

`ms.cmd(command: string) -> nil`

Exécute du code shell via `cmd.exe /C` masqué sous Windows ou `/bin/zsh -c` sur Mac. Ne renvoie ni valeur, ni sortie, ni code de sortie, et n’attend pas la fin du processus. Mac limite à 16 les processus d’arrière-plan gérés simultanément et journalise leur fin. Les chaînes s’exécutent telles quelles ; les espaces réservés `%MS_*%` ne sont pas développés par cette API Lua. Mettez vous-même les arguments du shell entre guillemets.

```lua
if package.config:sub(1, 1) == "\\" then
  ms.cmd("taskmgr.exe")
else
  ms.cmd('/usr/bin/open -a "Activity Monitor"')
end
```

Poursuivez avec la [syntaxe d’envoi de touches](key-sequences.md), les [recettes](lua-examples.md) ou les [emplacements du journal](config.md#emplacements-des-fichiers).
