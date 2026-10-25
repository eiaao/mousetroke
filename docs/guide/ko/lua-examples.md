---
title: "Lua 레시피"
description: "텍스트, 클립보드 붙여넣기, 날짜, 웹 검색, 창, 밝기와 일시 정지를 위한 바로 쓸 수 있는 마우스 제스처 Lua 스크립트."
order: 10
---

# Lua 레시피

완전한 블록 하나를 제스처의 **Lua 스크립트** 동작에 붙여 넣으세요. 각 블록은 `ms_action`을 정의합니다. 텍스트 레시피는 먼저 버려도 되는 텍스트 문서에서 시험해 보세요. [Lua 기초](lua-basics.md)는 편집기를 설명하고 [Lua API](lua-api.md)는 플랫폼 지원을 나열합니다.

크로스 플랫폼 레시피는 `package.config`로 `^`(Windows의 Control)와 `#`(Mac의 Command)를 고릅니다. 클립보드 레시피는 이미지나 다른 형식이 아닌 **텍스트만** 보존합니다. 200 ms 대기는 예시이며 애플리케이션의 보장이 아니므로, 대상이 느리면 늘리세요. 클립보드 내용을 일시적으로 바꾸고, 그 사이에 다른 것을 복사하더라도 저장해 둔 텍스트를 복원합니다.

## 고정된 텍스트 입력

목적: 마우스 제스처 하나로 재사용할 문장을 넣습니다.

```lua
function ms_action()
  ms.keys("Hello from Mousetroke.", true)
end
```

텍스트 우선 모드는 두 플랫폼에서 유니코드를 지원합니다. 여전히 문법 기호를 파싱하므로, `%`, `+` 또는 중괄호 같은 임의 구두점에는 클립보드 레시피를 사용하세요.

## 클립보드 텍스트 붙여넣고 복원하기

목적: 문자 그대로의 텍스트를 붙여넣은 다음 클립보드에 있던 텍스트를 복원합니다.

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

이 방법은 삽입할 텍스트의 키 시퀀스 이스케이프를 우회합니다. 클립보드 텍스트가 없었다면 최종 클립보드에는 빈 텍스트가 들어가며, 다른 원래 형식은 이 API로 복원할 수 없습니다.

## 앱마다 다른 텍스트

목적: 제스처 대상의 실행 파일 식별자로 문구를 고릅니다.

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

Windows 식별자는 보통 실행 파일 전체 경로이고, Mac은 보통 번들 ID를 제공합니다. 이것은 캡처된 대상을 검사하는 것이며, 입력은 활성 앱으로 갑니다. 대상 앱을 활성화하려면 먼저 `ms.focus_win(wid)`를 사용하세요.

## 오늘 날짜 넣기

목적: 로컬 날짜를 연-월-일 형식으로 넣습니다.

```lua
function ms_action()
  ms.keys(os.date("%Y-%m-%d"), true)
end
```

`os.date`는 런타임에서 사용할 수 있습니다. `%` 문자는 Lua 날짜 서식 문법이며, 결과로 나오는 숫자와 하이픈은 텍스트 우선 입력에 안전합니다.

## 선택한 텍스트를 사이트에서 검색

목적: 선택 영역을 복사하고 UTF-8로 인코딩된 Google 쿼리를 엽니다.

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

발동하기 전에 활성 앱에서 텍스트를 선택하세요. 먼저 클립보드를 비우면 복사가 아무것도 하지 않을 때 이전 클립보드 텍스트를 검색하는 것을 피할 수 있습니다. Lua에는 선택 텍스트 자동 치환이 없으며, 더 간단한 [URL 열기 동작](actions.md#url-열기)이 선택 영역을 대신 수집해 줍니다.

## 선택한 텍스트를 따옴표나 괄호로 감싸기

목적: 편집 가능한 필드에서 선택한 텍스트를 따옴표로 묶은 버전으로 바꿉니다.

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

괄호로 감싸려면 감싸는 식을 `"[" .. text .. "]"`로 바꾸세요. 붙여넣기가 대체하려면 복사 후에도 텍스트가 선택된 상태로 남아 있어야 합니다. 읽기 전용 페이지는 이 방법으로 편집할 수 없습니다.

## 항상 위 토글

목적: Windows 창의 최상위 상태를 토글합니다. 이 레시피에는 **창 동작 → 항상 위 / 해제**를 선택하세요. 그 동작은 실제 상태를 조회하고 토글할 수 있습니다. Lua API는 요청된 상태를 설정할 뿐이며, 최상위 getter가 없고 각 제스처는 새 VM을 받습니다. Mac은 앱 간 최상위 제어를 지원하지 않습니다.

Windows에서 별도의 "이 창 고정" Lua 제스처를 원하면 다음을 사용하세요:

```lua
function ms_action(pos, wid)
  if package.config:sub(1, 1) ~= "\\" then
    ms.show_toast("Always-on-top is unavailable on Mac")
    return
  end
  if wid ~= 0 then ms.set_win_top(wid, true) end
end
```

별도의 고정 해제 제스처에는 `true` 대신 `false`를 사용하세요. Lua 전역 불리언은 실행할 때마다 초기화되므로 토글을 구현할 수 없습니다.

## 최대화 또는 복원

목적: Windows에서 대상 창의 최대화 상태를, Mac에서 작업 영역 확대를 토글합니다.

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

Mac은 내장 창 동작과 같은 AX 확대 버튼을 사용합니다. 상태 조회는 작업 영역 크기의 창을 인식하며, 콘텐츠 크기로 확대하는 앱은 여전히 normal로 보고할 수 있으므로 그런 앱에는 **창 동작 → 최대화 / 복원**을 사용하세요. 손쉬운 사용 권한이 없거나 창을 사용할 수 없으면 nil이 됩니다. 창을 다른 모니터로 옮기는 Lua 창 위치/크기 API는 없습니다.

## 커서 아래 화면 밝기

목적: Windows에서 커서가 있는 화면의 밝기를 5퍼센트 포인트 올립니다.

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

더 어둡게 하는 두 번째 제스처에는 `5`를 `-5`로 바꾸세요. Windows는 호환되는 DDC/CI 또는 WMI 하드웨어가 필요합니다. WMI는 첫 번째 밝기 장치로 대체되며 다른 화면일 수 있습니다. 성공은 0–100 퍼센트이고, 실패는 두 플랫폼 모두 nil입니다. Mac은 여기서 nil을 반환합니다. 기존 백엔드는 밝기 키만 보낼 뿐 퍼센트를 반환하거나 퍼센트 포인트 델타를 적용할 수 없습니다. 커서 화면 선택이나 외장 DDC/CI 없이 내장 화면 밝기 키를 쓰려면 **미디어와 시스템 → 밝게 / 어둡게**를 사용하세요.

## 폴더 열기

목적: 현재 사용자의 Downloads 폴더를 플랫폼의 파일 브라우저로 엽니다.

```lua
function ms_action()
  if package.config:sub(1, 1) == "\\" then
    ms.cmd('explorer.exe "%USERPROFILE%\\Downloads"')
  else
    ms.cmd('/usr/bin/open "$HOME/Downloads"')
  end
end
```

이 환경 변수는 Mousetroke의 동작 변수가 아니라 운영 체제 셸이 확장합니다. 일반적인 Downloads 위치를 가정합니다.

## 인수를 지정해 프로그램 실행

목적: 프로그램에 인수 문자열을 전달합니다.

```lua
function ms_action()
  if package.config:sub(1, 1) == "\\" then
    ms.run("cmd.exe", "/K echo Hello from Mousetroke")
  else
    ms.run("/usr/bin/say", '"Hello from Mousetroke"')
  end
end
```

Windows는 메시지를 표시하는 명령 창을 엽니다. Mac은 `say`로 문구를 말합니다. `ms.run`은 테이블이 아니라 인수 문자열 하나를 받으며 프로세스 핸들을 반환하지 않습니다.

## 작업 관리자 또는 Activity Monitor 열기

목적: 플랫폼의 프로세스 뷰어를 엽니다.

```lua
function ms_action()
  if package.config:sub(1, 1) == "\\" then
    ms.keys("^+{ESC}")
  else
    ms.run("com.apple.ActivityMonitor")
  end
end
```

Windows는 Ctrl+Shift+Esc를 사용합니다. Mac은 번들 ID로 Activity Monitor를 실행합니다.

## 화면 잠금

목적: 시스템 단축키로 현재 세션을 잠급니다.

```lua
function ms_action()
  if package.config:sub(1, 1) == "\\" then
    ms.keys("#l")
  else
    ms.keys("^#q")
  end
end
```

Mac은 Control+Command+Q를 사용합니다. Windows는 Win+L을 사용하며, Ctrl+Alt+Del은 일반 키 시퀀스로 합성할 수 없습니다.

## 10분 동안 제스처 일시 정지

목적: 엔진을 일시적으로 일시 정지한 다음, Mousetroke가 실행된 상태를 유지하며 재개합니다.

```lua
function ms_action()
  ms.set_paused(true)
  ms.show_toast("Gestures paused for ten minutes", 2000)
  ms.sleep(10 * 60 * 1000)
  ms.set_paused(false)
  ms.show_toast("Gestures resumed", 1500)
end
```

두 플랫폼에서 동작하며 Lua 워커 하나를 10분 동안 잠들게 합니다. 영구적인 타이머가 아니므로 종료하거나 다시 시작하면 끝납니다. 기다리는 동안 수동으로 재개할 수 있지만, 스크립트는 대기가 끝나면 여전히 재개를 실행합니다. 여러 사본을 시작하지 마세요. [런타임 제한](lua-basics.md#런타임과-제한)을 참조하세요.
