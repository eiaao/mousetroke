---
title: "Lua 기초"
description: "첫 마우스 제스처 Lua 스크립트를 작성하고, ms_action 인수를 배우고, 로그로 디버깅하고, 런타임 제한을 이해합니다."
order: 8
---

# Lua 기초

## Lua를 사용할 때

단축키 하나, 창 동작 하나, URL, 애플리케이션 또는 명령 하나에는 [간단한 동작](actions.md)을 사용하세요. 제스처가 애플리케이션에 따라 분기하거나, 텍스트를 변환하거나, 여러 작업을 결합해야 할 때 Lua를 선택하세요. Mousetroke는 **Lua 5.4**를 내장하고 있습니다.

## 첫 스크립트

1. 설정에서 제스처를 만들거나 선택합니다.
2. **동작** 아래에서 **Lua 스크립트**를 고릅니다. 텍스트 영역 전체를 다음 코드로 바꿉니다.
3. 저장됨 표시를 기다린 다음 설정 밖에서 발동해 봅니다.

```lua
function ms_action(pos, wid, pid, exe, title, mode)
  ms.log("Target: %s, mode: %s", exe, mode)
  ms.show_toast("My first Lua gesture", 1500)
end
```

`function`은 함수를 시작하고 `end`는 끝냅니다. Mousetroke는 코드를 불러온 다음 `ms_action`을 한 번 호출합니다. 작업은 그 함수 안에 넣으세요. 괄호 안의 이름은 컨텍스트를 받으며, 쓰지 않는 인수는 생략할 수 있습니다. `ms.log`는 런타임 로그에 기록하고 `ms.show_toast`는 1500밀리초 동안 메시지를 표시합니다.

편집기의 **Lua 입문**은 이 페이지를 엽니다. [Lua 레시피](lua-examples.md)를 따라 마우스 제스처로 텍스트를 출력하거나 클립보드를 통해 붙여넣어 보세요.

## 진입 인수

전체 진입점은 `ms_action(pos, wid, pid, exe, title, mode)`입니다:

| 인수 | Lua 타입 | 의미 |
|---|---|---|
| `pos` | 테이블 `{x=integer, y=integer}` | 동작이 발송될 때 기록된 마지막 커서 위치. 다른 모니터에서는 좌표가 음수일 수 있습니다. |
| `wid` | 정수 | 대상 창 식별자. Windows HWND, Mac 창 ID. 불투명 식별자로 다루세요. 대상이 없으면 0일 수 있습니다. |
| `pid` | 정수 | 대상 프로세스 ID 또는 0. |
| `exe` | 문자열 | Windows 실행 파일 식별자, 보통 전체 경로. Mac 번들 ID 또는 실행 파일 경로. 없으면 비어 있습니다. |
| `title` | 문자열 | 대상 창 제목 또는 빈 문자열. Mac은 워커에서 손쉬운 사용을 통해 제목을 가져옵니다. |
| `mode` | 문자열 | **기본 설정 → 제스처가 작용할 대상**에 따라 정확히 `UnderCursor` 또는 `Focused`. 트리거 버튼이나 제스처 이름이 아닙니다. |

대상은 제스처에 대해 캡처됩니다. `pos`는 동작 변수의 제스처 시작 좌표와 다를 수 있습니다. 대상을 활성화하지 않으면 키보드 입력은 여전히 활성 앱으로 갑니다. Mac에서 `ms.focus_win(wid)`는 이 제스처의 대상 앱만 활성화할 수 있습니다. `%MS_PID%` 같은 자리 표시자 텍스트는 Lua에서 확장되지 않으며, `pid`를 직접 사용하세요.

## 5분 Lua 입문

`local`은 변수를 만듭니다. 문자열은 작은따옴표나 큰따옴표를 쓰고, `..`는 문자열을 잇습니다. `--`는 주석을 시작합니다. `if`, `elseif`, `else`, `end`는 분기를 고릅니다. `nil`은 값이 없음을 의미하고, 조건에서 `false`와 `nil`만 거짓입니다(0과 `""`는 참입니다).

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

여기서 `string.find(s, needle, 1, true)`는 문자 그대로 검색해 위치 또는 `nil`을 반환합니다. `true`는 구두점을 패턴으로 해석하지 않게 합니다. `string.match`는 정규 표현식과 다른 Lua 패턴을 사용합니다. 테이블은 이름 있는 필드나 첫 번째 인덱스가 1인 목록을 저장할 수 있습니다. 도우미 함수는 값을 반환할 수 있습니다:

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

Lua 문자열은 키 시퀀스 파싱 전에 이스케이프를 해석합니다. `"\\"`는 백슬래시 하나를, `"\n"`은 줄 바꿈을 나타냅니다. `[[text]]` 같은 긴 괄호 문자열은 백슬래시를 보존합니다. 두 번째 문법 계층은 [키 보내기 가이드](key-sequences.md)가 설명합니다.

## 디버깅

`ms.log("pid=%d title=%s", pid, title)`를 사용하고 **정보 → 로그 열기**에서 확인하세요:

- Windows: `%APPDATA%\mousetroke\mousetroke.log`.
- Mac: `~/Library/Logs/mousetroke/mousetroke.log`.

불러오기 오류와 실행 오류는 청크 이름 `gesture_script`와 함께 거기에 나타납니다. `ms.show_toast("Reached this step")`는 빠른 시각 확인입니다. 선택적 API 결과는 `nil`일 수 있으므로 산술 연산이나 테이블 접근 전에 검사하세요. Mac에서 지원되지 않는 호출은 [API 참조](lua-api.md)를 참조하세요.

## 런타임과 제한

스크립트는 실행할 때마다 워커 스레드에서 새 Lua VM을 만들며, **8 MiB Lua 할당 제한**이 있습니다. 전역 상태는 제스처 사이에 유지되지 않습니다. 런타임은 명령어 수나 벽시계 시간 초과를 두지 않습니다. 무한 루프를 피하세요. `ms.sleep(ms)`는 입력 훅이나 메인 UI 스레드가 아니라 그 스크립트의 워커만 막습니다. `ms.keys`는 자기 시퀀스가 끝날 때까지 기다리며, Mac에서는 Lua 워커가 기다리는 동안 메인 스레드 타이머가 입력을 진행합니다.

사용 가능한 표준 라이브러리는 기본 함수와 `coroutine`, `table`, `io`, `os`, `string`, `utf8`, `math`, `package`입니다. `debug`는 불러오지 않으며, 네이티브 C 모듈 로딩은 mlua의 안전 모드로 비활성화됩니다. `os.date`는 사용할 수 있습니다. 이것은 운영 체제 보안 샌드박스가 아닙니다. `io`, `os`와 프로세스 API는 사용자 권한으로 파일에 접근하거나 명령을 실행할 수 있습니다. 신뢰하는 스크립트와 구성만 실행하세요.

다음: [API 참조](lua-api.md), [복사해 쓰는 레시피](lua-examples.md), [문제 해결](troubleshooting.md).
