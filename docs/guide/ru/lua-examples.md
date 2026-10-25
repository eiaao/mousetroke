---
title: "Рецепты Lua"
description: "Готовые скрипты Lua для жестов мыши: текст, вставка из буфера обмена, даты, поиск в интернете, окна, яркость и пауза."
order: 10
---

# Рецепты Lua

Вставьте один полный блок в действие жеста **Скрипт Lua**. Каждый блок определяет `ms_action`. Текстовые рецепты сначала пробуйте в одноразовом текстовом документе. [Основы Lua](lua-basics.md) объясняют редактор, а [Lua API](lua-api.md) перечисляет поддержку платформ.

Кроссплатформенные рецепты используют `package.config`, чтобы выбрать `^` (Control в Windows) или `#` (Command на Mac). Рецепты буфера обмена сохраняют **только текст**, не изображения и не другие форматы. Их ожидания 200 мс — примеры, а не гарантия приложения; увеличьте их, если цель медленная. Они временно заменяют содержимое буфера обмена и восстанавливают сохранённый текст, даже если за этот интервал вы скопируете что-то другое.

## Ввод фиксированного текста

Назначение: вставить повторяемое предложение одним жестом мыши.

```lua
function ms_action()
  ms.keys("Hello from Mousetroke.", true)
end
```

Текстовый режим поддерживает Юникод на обеих платформах. Он по-прежнему разбирает символы синтаксиса; для произвольной пунктуации вроде `%`, `+` или фигурных скобок используйте рецепт буфера обмена.

## Вставка с восстановлением буфера обмена

Назначение: вставить буквальный текст, затем восстановить текст, который был в буфере обмена.

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

Это обходит экранирование последовательности клавиш для вставляемого текста. Если текста в буфере не было, в итоге буфер содержит пустой текст; другие исходные форматы этими API восстановить нельзя.

## Разный текст для разных программ

Назначение: по идентичности исполняемого файла цели жеста выбрать фразу.

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

Идентичности Windows обычно являются полными путями к исполняемым файлам; Mac обычно даёт ID пакета. Это проверяет зафиксированную цель, а ввод идёт в активную программу. Сначала вызовите `ms.focus_win(wid)`, если нужно активировать целевую программу.

## Вставка сегодняшней даты

Назначение: вставить локальную дату как год-месяц-день.

```lua
function ms_action()
  ms.keys(os.date("%Y-%m-%d"), true)
end
```

`os.date` доступна в среде выполнения. Символы `%` — синтаксис формата даты Lua; получившиеся цифры и дефисы безопасны для текстового ввода.

## Поиск выделенного текста на сайте

Назначение: скопировать выделение и открыть закодированный в UTF-8 запрос Google.

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

Перед запуском выделите текст в активной программе. Предварительная очистка буфера обмена не даёт искать старый текст буфера, если копирование ничего не делает. В Lua нет автоматической подстановки выделенного текста; более простое [действие «Открыть URL»](actions.md#открыть-url) может собрать выделение за вас.

## Заключить выделенный текст в кавычки или скобки

Назначение: заменить выделенный текст закавыченной версией в редактируемом поле.

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

Для скобок замените выражение обёртки на `"[" .. text .. "]"`. Копирование должно оставить текст выделенным, чтобы вставка его заменила. Страницы только для чтения так править нельзя.

## Переключение поверх всех окон

Назначение: переключить состояние поверх всех у окна Windows. Для этого рецепта выберите **Действия с окном → Поверх всех / отменить**: это действие может запросить и переключить фактическое состояние. Lua API только задаёт запрошенное состояние; геттера режима поверх всех нет, и каждый жест получает свежую VM. Mac не поддерживает межпрограммное управление режимом поверх всех.

Если нужен отдельный жест Lua «закрепить это окно» в Windows, используйте:

```lua
function ms_action(pos, wid)
  if package.config:sub(1, 1) ~= "\\" then
    ms.show_toast("Always-on-top is unavailable on Mac")
    return
  end
  if wid ~= 0 then ms.set_win_top(wid, true) end
end
```

Используйте `false` вместо `true` для отдельного жеста открепления. Глобальное логическое значение Lua сбрасывалось бы при каждом выполнении и не реализовало бы переключатель.

## Развернуть или восстановить

Назначение: переключить развёрнутое состояние целевого окна в Windows или масштаб рабочей области на Mac.

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

Mac использует ту же кнопку масштаба AX, что и встроенное действие окна. Запрос состояния распознаёт окна размером рабочей области; программы, масштабирующие по размеру содержимого, могут по-прежнему сообщать normal, поэтому для них используйте **Действия с окном → Развернуть / восстановить**. Отсутствующее разрешение универсального доступа или недоступное окно даёт nil. Lua API положения/размера окна для переноса окна на другой монитор нет.

## Яркость экрана под курсором

Назначение: поднять яркость экрана курсора на 5 процентных пунктов в Windows.

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

Замените `5` на `-5` для второго, затемняющего жеста. Windows нужна совместимая аппаратура DDC/CI или WMI; WMI откатывается к своему первому устройству яркости, которое может быть другим экраном. Успех — процент 0–100; неудача — nil на обеих платформах. Mac здесь возвращает nil: существующий модуль только посылает клавиши яркости и не может вернуть процент или применить приращение в процентных пунктах. Используйте **Медиа и система → Ярче / Темнее** для клавиш яркости встроенного экрана, без выбора экрана курсора и без внешнего DDC/CI.

## Открыть папку

Назначение: открыть папку загрузок текущего пользователя файловым браузером платформы.

```lua
function ms_action()
  if package.config:sub(1, 1) == "\\" then
    ms.cmd('explorer.exe "%USERPROFILE%\\Downloads"')
  else
    ms.cmd('/usr/bin/open "$HOME/Downloads"')
  end
end
```

Эти переменные среды раскрывает оболочка операционной системы, а не переменные действия Mousetroke. Предполагается обычное расположение загрузок.

## Запустить программу с аргументами

Назначение: передать программе строку аргументов.

```lua
function ms_action()
  if package.config:sub(1, 1) == "\\" then
    ms.run("cmd.exe", "/K echo Hello from Mousetroke")
  else
    ms.run("/usr/bin/say", '"Hello from Mousetroke"')
  end
end
```

Windows открывает окно командной строки с сообщением. Mac произносит фразу через `say`. `ms.run` принимает одну строку аргументов, а не таблицу, и не возвращает дескриптор процесса.

## Открыть диспетчер задач или Activity Monitor

Назначение: открыть средство просмотра процессов платформы.

```lua
function ms_action()
  if package.config:sub(1, 1) == "\\" then
    ms.keys("^+{ESC}")
  else
    ms.run("com.apple.ActivityMonitor")
  end
end
```

Windows использует Ctrl+Shift+Esc. Mac запускает Activity Monitor по ID пакета.

## Заблокировать экран

Назначение: заблокировать текущий сеанс системным сочетанием.

```lua
function ms_action()
  if package.config:sub(1, 1) == "\\" then
    ms.keys("#l")
  else
    ms.keys("^#q")
  end
end
```

Mac использует Control+Command+Q. Windows использует Win+L; Ctrl+Alt+Del нельзя синтезировать как обычную последовательность клавиш.

## Приостановить жесты на десять минут

Назначение: временно приостановить движок, затем возобновить его, пока Mousetroke продолжает работать.

```lua
function ms_action()
  ms.set_paused(true)
  ms.show_toast("Gestures paused for ten minutes", 2000)
  ms.sleep(10 * 60 * 1000)
  ms.set_paused(false)
  ms.show_toast("Gestures resumed", 1500)
end
```

Это работает на обеих платформах и держит один рабочий поток Lua спящим десять минут. Это не постоянный таймер: выход или перезапуск его завершает. Во время ожидания можно возобновить вручную, но скрипт всё равно выдаст возобновление, когда ожидание закончится. Не запускайте несколько копий. См. [ограничения среды выполнения](lua-basics.md#среда-выполнения-и-ограничения).
