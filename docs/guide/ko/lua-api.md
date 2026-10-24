---
title: "Lua API"
description: "시그니처, 반환 값, 예제와 Windows/macOS 지원 참고 사항을 담은 Lua 5.4 ms API 전체 참조."
order: 9
---

# Lua API

이것은 공유 런타임이 `ms`에 등록하는 모든 함수입니다. 예제는 `ms_action` 안에 넣는 구문이며, 거기서 `pos`, `wid`, `pid`, `exe`, `title`, `mode`를 사용할 수 있습니다. 완전한 스크립트는 [Lua 기초](lua-basics.md)를, 바로 쓸 수 있는 동작은 [레시피](lua-examples.md)를 참조하세요.

## 규칙

Lua 버전: **5.4**(`lua54`와 `vendored`를 사용한 `mlua`). `nil`은 값이 없음을 의미하고, `?`는 선택적 매개변수를 표시합니다. 값을 반환하지 않는 함수는 `-> nil`로 표시합니다. 생략된 선택적 인수는 nil과 동일합니다. 불리언 인수에는 실제 불리언을 전달하세요. 정수 매개변수는 유한한 소수를 0 방향으로 버리고 기반 정수 범위를 벗어난 값은 거부합니다. 스크립트에서는 정수를 사용하세요.

위치 테이블은 배열 항목이 아니라 이름 있는 필드 `{x=..., y=...}`를 사용합니다. 지원되지 않는 Mac 호출은 일반적으로 프로세스당 함수당 한 번 기록합니다. getter는 nil을 반환하고 setter는 아무것도 하지 않습니다. 값이 없는 setter는 성공 확인이 아닙니다. Lua API 문자열은 동작 변수 치환을 받지 않습니다.

## 흐름과 피드백

### `ms.log`

`ms.log(...) -> nil`

모든 Lua 값을 받습니다. 서식 문자열과 추가 인수가 있으면 `%s`, `%d`, `%f`, `%q`, `%%`를 지원하고, 그렇지 않으면 값을 공백으로 잇습니다. `%q`는 Lua 인용을 추가하지 않습니다. 두 플랫폼에서 런타임 로그에 `Lua:` 줄을 씁니다. 값을 반환하지 않습니다.

```lua
ms.log("pid=%d app=%s", pid, exe)
```

### `ms.sleep`

`ms.sleep(milliseconds: integer) -> nil`

음수가 아닌 밀리초 수(부호 없는 64비트)만큼 기다립니다. 두 플랫폼에서 이 스크립트 워커를 막고, 제스처 엔진/메인 스레드는 계속됩니다. 반환 값이 없고 스크립트 시간 초과도 설치되어 있지 않습니다.

```lua
ms.sleep(100)
```

### `ms.is_paused`

`ms.is_paused() -> boolean`

매개변수가 없습니다. 두 플랫폼에서 수동 엔진 일시 정지 플래그를 반환합니다. 자동 전체 화면 억제나 앱 제외는 보고하지 않습니다.

```lua
ms.log(ms.is_paused())
```

### `ms.set_paused`

`ms.set_paused(paused: boolean) -> nil`

`true`는 일시 정지하고 `false`는 재개합니다. 반환 값이 없습니다. 두 플랫폼에서 지원됩니다. Windows는 트레이 스레드에 명령을 보내므로, 바로 이어진 `is_paused()`는 아직 이전 값을 볼 수 있습니다. Mac은 메인 스레드 브리지를 통해 적용합니다.

```lua
ms.set_paused(true)
```

### `ms.show_settings`

`ms.show_settings() -> nil`

매개변수와 반환 값이 없습니다. 두 플랫폼에서 설정을 열거나 앞으로 가져옵니다.

```lua
ms.show_settings()
```

### `ms.show_toast`

`ms.show_toast(message: string, duration_ms?: integer) -> nil`

`message`는 표시할 텍스트입니다. 선택적인 음수 아닌 지속 시간은 밀리초이며 기본값은 **1200**입니다. 두 플랫폼에서 오버레이 메시지를 표시하고, 값을 반환하지 않습니다.

```lua
ms.show_toast("Done", 1500)
```

## 키보드 입력

### `ms.keys`

`ms.keys(sequence: string, text_by_default?: boolean) -> nil`

[키 시퀀스](key-sequences.md)를 파싱해 보냅니다. 텍스트 우선 모드는 기본값이 `false`입니다. 보낸 뒤 반환하며, 잘못된 문법은 Lua 오류를 발생시킵니다. 두 플랫폼 모두 일반 키보드/텍스트 입력을 지원합니다. Mac은 지원되지 않는 키, 마우스와 휠 동작을 건너뛰고 큐/텍스트 제한을 적용합니다. 다른 시퀀스가 들어 있을 수도 있는 메인 스레드 입력 큐를 기다립니다.

```lua
ms.keys("Hello", true)
```

### `ms.keys_down`

`ms.keys_down(sequence: string) -> nil`

Windows 전용: 키 모드로 파싱해 키/수식 키를 놓지 않고 누릅니다. 반복은 이 단계에서 반복되지 않고, 휠 동작은 한 번 발동합니다. 잘못된 문법은 Lua 오류를 발생시킵니다. 놓으려면 `keys_up`과 짝을 맞추세요. Mac은 지원되지 않음을 기록하고 아무것도 하지 않습니다. 반환 값이 없습니다.

```lua
ms.keys_down("^a")
ms.sleep(50)
ms.keys_up("^a")
```

### `ms.keys_up`

`ms.keys_up(sequence: string) -> nil`

Windows 전용: 파싱된 시퀀스를 역순으로 놓되 수식 키는 마지막에 놓습니다. 대기 단계는 여전히 기다립니다. 문법 오류는 Lua 오류를 발생시킵니다. Mac은 지원되지 않음을 기록하고 아무것도 하지 않습니다. 반환 값이 없습니다.

```lua
ms.keys_down("{SHIFT}")
ms.keys_up("{SHIFT}")
```

### `ms.is_keys_pressed`

`ms.is_keys_pressed(sequence: string) -> boolean | nil`

두 플랫폼 모두 모든 키보드 키 **와 그 접두사 수식 키**를 검사합니다. `"^a"`는 Control과 A가 필요합니다. 어느 쪽이든 접두사를 만족하며, `{RCTRL}` 같은 명시적 이름은 해당 쪽을 요구합니다. Mac에서 `^` = Control, `%` = Option, `+` = Shift, `#` = Command입니다. 여러 단계는 나열된 키/수식 키가 순서대로 눌렸다는 뜻이 아니라 함께 눌려 있어야 한다는 뜻이며, 반복은 요구 사항을 추가하지 않습니다. 마우스/휠, 대기 또는 텍스트 동작은 검사를 false로 만들고, 빈 시퀀스는 true를 반환합니다. 잘못된 문법은 false를 반환합니다. Mac은 결합된 세션 상태를 읽으며, 손쉬운 사용 권한이 없으면 한 번 기록하고 false를 반환합니다.

```lua
if ms.is_keys_pressed("^a") then
  ms.show_toast("Ctrl and A are down")
end
```

## 클립보드와 파일

### `ms.get_clipboard_text`

`ms.get_clipboard_text() -> string | nil`

매개변수가 없습니다. 클립보드 텍스트를 반환하거나, 텍스트를 사용할 수 없으면 `nil`을 반환합니다. 두 플랫폼에서 지원되며, 선택 영역을 자동으로 가져오지는 않습니다.

```lua
local text = ms.get_clipboard_text()
ms.log(text or "No text")
```

### `ms.set_clipboard_text`

`ms.set_clipboard_text(text: string) -> nil`

두 플랫폼에서 클립보드 내용을 text로 바꿉니다. 반환 값이나 성공 플래그가 없습니다. 다른 클립보드 형식은 보존되지 않습니다. [클립보드 레시피](lua-examples.md)를 참조하세요.

```lua
ms.set_clipboard_text("Hello from Mousetroke")
```

### `ms.read_text`

`ms.read_text(path: string) -> string | nil`

두 플랫폼에서 파일 전체를 UTF-8로 읽습니다. 읽기 또는 UTF-8 디코딩 오류 시 `nil`을 반환합니다. 상대 경로는 문서화된 스크립트 폴더가 아니라 프로세스 작업 디렉터리를 사용합니다. 읽은 내용은 사용 가능한 메모리에 포함됩니다.

```lua
local text = ms.read_text("config.json")
ms.log(text and #text or 0)
```

## 창

### `ms.find_win_by_class`

`ms.find_win_by_class(pattern: string) -> table | nil`

**Rust 정규 표현식**으로 보이는 최상위 창을 검색합니다. Windows에서는 클래스 이름, Mac에서는 애플리케이션 **번들 ID**를 대상으로 합니다(Lua 문자열로는 예: `^com\\.apple\\.Safari$`). 1부터 시작하는 배열로 최대 32개 항목을 반환합니다: `{text=title, hwnd=window_id}`. 일치하는 것이 없거나 잘못된 정규식이면 빈 테이블입니다. 패턴이 `(?i)` 같은 플래그를 쓰지 않으면 대소문자를 구분하며, Lua 패턴이 아닙니다. Mac은 손쉬운 사용을 통해 제목을 읽고, 권한이 없으면 한 번 기록하고 nil을 반환합니다. 여기서 `hwnd`는 `focus_win`과 창 상태 API가 받는 CGWindowID입니다. Mac에서는 현재 화면에 보이는 창만 검색합니다.

```lua
local wins = ms.find_win_by_class("^Notepad$")
if wins and wins[1] then ms.focus_win(wins[1].hwnd) end
```

### `ms.find_win_by_title`

`ms.find_win_by_title(pattern: string) -> table | nil`

두 플랫폼에서 Rust 정규 표현식으로 창 제목을 검색해 `find_win_by_class`와 같은 1부터 시작하는 `{text, hwnd}` 항목을 최대 32개 반환합니다. 잘못된 정규식이면 빈 테이블입니다. Mac은 AX 창 제목을 사용하고, 손쉬운 사용 권한이 필요하며, 현재 화면에 보이는 창만 검색합니다.

```lua
local wins = ms.find_win_by_title("(?i)notepad")
if wins then ms.log(#wins) end
```

### `ms.focus_win`

`ms.focus_win(wid: integer) -> nil`

창/앱을 활성화합니다. 반환 값이나 성공 플래그가 없습니다. Windows는 HWND를 받습니다. Mac은 제스처 대상이나 발견된 CGWindowID를 받고, 최소화를 해제하고 AX 창을 앞으로 올린 다음, 메인 스레드 브리지를 통해 소유 앱을 활성화합니다. Mac AX 호출은 스크립트 워커에서 실행되고, PID/경계 기반 내장 창 조회를 재사용하며, 손쉬운 사용 권한이 필요합니다. 오래된 ID나 사용할 수 없는 AX 창은 무시됩니다. 겹치는 창 경계가 같은 앱은 모호할 수 있습니다.

```lua
if wid ~= 0 then ms.focus_win(wid) end
```

### `ms.set_win_state`

`ms.set_win_state(wid: integer, state: string) -> nil`

Windows: `"min"`은 최소화, `"max"`는 최대화이며, `"normal"`과 `"norm"`을 포함한 **다른 모든 문자열은 복원**입니다. Mac: `"min"`은 AXMinimized를 사용하고, `"max"`는 최소화 해제/전체 화면 종료 후 창이 화면 작업 영역을 채우지 않을 때 내장 동작과 같은 AXZoomButton을 누릅니다. `"normal"`, `"norm"`, `"restore"`는 최소화 해제/전체 화면 종료 후 작업 영역 확대를 되돌립니다. Mac은 앱이 지원하는 경우 AXFullScreen을 통해 `"fullscreen"`도 받으며, 다른 이름은 모두 무시되고 한 번 기록됩니다. 값은 대소문자를 구분합니다. 반환 값이 없습니다. Mac은 손쉬운 사용 권한이 필요하고, AX는 워커에서 실행되며, 화면 지오메트리는 메인 스레드 브리지를 통해 옵니다. 전환은 비동기일 수 있습니다.

Mac Zoom은 앱이 정의합니다. 콘텐츠 크기로 확대하는 경우에도 `"normal"`로 보고될 수 있으며 이 API로 확실히 복원할 수 없습니다. 반복된 `"max"` 요청은 토글처럼 동작할 수 있습니다. 그런 앱에는 내장 **최대화 / 복원** 토글을 사용하세요. 범용적인 공개 AX 확대 상태 플래그는 없습니다.

```lua
if wid ~= 0 then ms.set_win_state(wid, "max") end
```

### `ms.get_win_state`

`ms.get_win_state(wid: integer) -> string | nil`

Windows는 `"min"`, `"max"` 또는 `"normal"`을 반환합니다. 이것은 최상위 조회가 아니며 오래된 핸들을 검증하지도 않습니다. 잘못된 핸들이 정상으로 보일 수 있습니다. Mac은 `"min"`, `"fullscreen"`, `"max"`(프레임이 화면 작업 영역과 2 논리 포인트 이내로 일치) 또는 `"normal"`을 반환합니다. Mac에는 범용 공개 AX 확대 상태 getter가 없으므로 콘텐츠 크기 확대는 normal로 보고됩니다. 권한 부족, 사용할 수 없는 AX 창/상태 또는 오래된 ID에는 nil을 반환하고, 권한 거부는 한 번 기록됩니다.

```lua
if wid ~= 0 then ms.log(ms.get_win_state(wid)) end
```

### `ms.set_win_top`

`ms.set_win_top(wid: integer, top: boolean) -> nil`

Windows는 창을 이동하거나 크기를 바꾸지 않고 항상 위를 설정(`true`)하거나 해제(`false`)합니다. 반환 값이 없습니다. 상태를 설정하는 것이며 토글하거나 이전 상태를 반환하지 않습니다. Mac은 아무것도 하지 않고 한 번 기록합니다. macOS에는 다른 앱의 창을 고정하는 공개 API가 없으며, 내장 최상위 동작도 지원되지 않습니다. 토글하려면 Windows 편집기에서 **창 동작 → 항상 위 / 해제**를 선택하세요.

```lua
if wid ~= 0 then ms.set_win_top(wid, true) end
```

## 마우스와 화면

### `ms.get_mouse_pos`

`ms.get_mouse_pos() -> table`

매개변수가 없습니다. 두 플랫폼 모두 현재 커서 위치를 `{x=integer, y=integer}`로 반환하며, 진입 인수 `pos`와 다를 수 있습니다. Windows는 화면 픽셀을 사용하고, Mac은 Y가 아래로 증가하는 Quartz 전역 논리 포인트를 사용합니다. Mac은 커서 조회가 실패하면 `{x=0,y=0}`을 반환합니다.

```lua
local p = ms.get_mouse_pos()
ms.log("%d,%d", p.x, p.y)
```

### `ms.set_mouse_pos`

`ms.set_mouse_pos(x: integer, y: integer) -> nil`

두 플랫폼에서 포인터를 전역 화면 좌표로 옮깁니다. 반환 값이 없습니다. 좌표는 음수일 수 있습니다. Mac은 다른 제스처를 시작하지 않도록 태그가 붙은 이동 이벤트를 보냅니다.

```lua
local p = ms.get_mouse_pos()
ms.set_mouse_pos(p.x + 20, p.y)
```

### `ms.get_screen`

`ms.get_screen(x: integer, y: integer) -> table | nil`

사용 가능한 작업 영역이 아니라 전체 화면 사각형 `{left, top, right, bottom}`을 정수 전역 좌표로 반환합니다. 너비는 `right-left`, 높이는 `bottom-top`입니다. Windows는 점이 모든 모니터 밖에 있으면 가장 가까운 모니터를 고릅니다. Mac은 점을 포함하는 디스플레이를 고르거나 첫 번째 캐시된 디스플레이로 대체합니다. 화면/브리지 결과가 없으면 `nil`을 반환할 수 있습니다. 두 플랫폼 모두 이 호출을 지원합니다.

```lua
local p = ms.get_mouse_pos()
local s = ms.get_screen(p.x, p.y)
if s then ms.log(s.right - s.left, s.bottom - s.top) end
```

### `ms.get_brightness`

`ms.get_brightness(pos?: table) -> integer | nil`

선택적 `{x=integer,y=integer}`이 화면을 고릅니다. 생략/nil이거나 불완전한 위치면 현재 커서로 대체합니다. 어느 플랫폼에서든 **성공 시 0–100 퍼센트, 실패 시 nil**을 반환합니다. Windows는 DDC/CI와 WMI 대체 경로를 사용합니다. WMI는 커서와 대응된 모니터가 아니라 첫 번째 밝기 장치를 사용합니다. Mac은 현재 nil을 반환하고 한 번 기록합니다. 기존 밝기 백엔드는 내장 디스플레이 밝기 키 이벤트만 보낼 뿐 퍼센트를 읽을 수 없습니다. 외장 DDC는 지원되지 않습니다. Windows 레거시 `WG.GetBrightness` 래퍼만 실패 시 -1을 유지합니다.

```lua
local value = ms.get_brightness()
if value ~= nil then ms.show_toast(tostring(value)) end
```

### `ms.set_brightness`

`ms.set_brightness(pos: table | nil, delta: integer) -> integer | nil`

이름과 달리 절대 퍼센트를 설정하는 것이 아니라 `delta` 퍼센트 포인트만큼 조정합니다. 커서를 사용하려면 이름 있는 `{x=...,y=...}` 테이블이나 **nil을 첫 번째 인수로** 전달하세요. 성공하면 그 범위로 고정된 새 **0–100 퍼센트**를 반환하고, 실패하면 **두 플랫폼 모두 nil**을 반환합니다. Windows는 `get_brightness`와 같은 DDC/CI와 WMI 선택을 사용합니다. Windows 레거시 `WG.SetBrightness` 래퍼만 실패 시 -1을 유지합니다. Mac은 nil을 반환하고 한 번 기록합니다. 기존 미디어와 시스템 백엔드는 내장 디스플레이 밝기 키만 보낼 뿐 수치 읽기나 퍼센트 포인트 제어가 없습니다. 이 Lua 호출로는 밝기가 바뀌지 않으므로 밝기 키에는 내장 동작을 사용하세요. 외장 DDC는 여전히 지원되지 않습니다.

```lua
local value = ms.set_brightness(nil, 5)
ms.log(value)
```

## URL과 프로세스

### `ms.open`

`ms.open(url: string) -> nil`

기본 처리기로 URL을 엽니다. 반환 값이나 성공 플래그가 없습니다. Windows는 셸 열기를 사용하며 파일/폴더 경로도 받습니다. Mac은 URL을 기대하므로 파일에는 올바르게 인코딩된 `file://` URL을 사용하거나, `ms.cmd`로 경로에 `/usr/bin/open`을 호출하세요. 변수 치환은 수행하지 않습니다.

```lua
ms.open("https://mousetroke.com/")
```

### `ms.run`

`ms.run(file: string, args?: string) -> nil`

Lua 배열이 아니라 선택적 인수 **문자열**로 실행합니다. 값이나 PID를 반환하지 않습니다. Windows는 RunAgain 동작의 일반적인 비승격 실행을 사용합니다. Mac은 번들 ID, 앱 경로 또는 절대 실행 파일 경로를 받고, 인수는 따옴표 처리와 함께 나눕니다. 실행 파일 실행은 LaunchServices가 거부하면 백그라운드 프로세스로 대체할 수 있습니다. 새 인스턴스를 요청합니다. 변수 확장이 없으므로 Lua 연결을 사용하세요. 프로세스 출력은 반환되지 않습니다.

```lua
if package.config:sub(1, 1) == "\\" then
  ms.run("cmd.exe", "/K echo Hello")
else
  ms.run("/usr/bin/say", '"Hello from Mousetroke"')
end
```

### `ms.cmd`

`ms.cmd(command: string) -> nil`

Windows에서는 숨겨진 `cmd.exe /C`로, Mac에서는 `/bin/zsh -c`로 셸 코드를 실행합니다. 값, 출력 또는 종료 코드를 반환하지 않고 프로세스 완료를 기다리지도 않습니다. Mac은 동시에 관리하는 백그라운드 프로세스를 16개로 제한하고 완료를 기록합니다. 문자열은 제공된 그대로 실행되며, `%MS_*%` 자리 표시자는 이 Lua API에서 확장되지 않습니다. 셸 인수는 직접 따옴표로 묶으세요.

```lua
if package.config:sub(1, 1) == "\\" then
  ms.cmd("taskmgr.exe")
else
  ms.cmd('/usr/bin/open -a "Activity Monitor"')
end
```

다음: [키 보내기 문법](key-sequences.md), [레시피](lua-examples.md) 또는 [로그 위치](config.md#파일-위치).
