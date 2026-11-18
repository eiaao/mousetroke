---
title: "Lua 入門"
description: "從零撰寫滑鼠手勢 Lua 指令碼，理解 ms_action 參數、常用語法、日誌除錯和執行限制。"
order: 8
---

# Lua 入門

## 什麼時候用 Lua

單一快速鍵、視窗操作、網址、程式或命令，優先用[簡單動作](actions.md)。需要按程式判斷、處理文字或組合多步時，選擇 Lua。Mousetroke 內建 **Lua 5.4**。

## 第一個指令碼

1. 在設定中新增或選取手勢。
2. 在「執行」中選擇「Lua 指令碼」，將整個文字方塊替換成下列程式碼。
3. 等待已儲存提示，再到設定視窗外觸發。

```lua
function ms_action(pos, wid, pid, exe, title, mode)
  ms.log("Target: %s, mode: %s", exe, mode)
  ms.show_toast("我的第一个 Lua 手势", 1500)
end
```

`function` 開始一個函式，`end` 結束函式。Mousetroke 載入程式碼後呼叫一次 `ms_action`。把要執行的操作放在函式裡。括號中的名稱接收手勢的執行環境，不用的參數可以省略。`ms.log` 寫執行日誌，`ms.show_toast` 在螢幕上顯示 1500 毫秒的提示。

編輯器的「Lua 入門」開啟本頁。想讓滑鼠手勢輸出文字、透過剪貼簿貼上，可繼續看 [Lua 實用範例](lua-examples.md)。

## 入口參數

完整入口是 `ms_action(pos, wid, pid, exe, title, mode)`：

| 參數 | Lua 型別 | 說明 |
|---|---|---|
| `pos` | 資料表 `{x=整数, y=整数}` | 分派動作時最後記錄的滑鼠座標，其他顯示器上可能為負數。 |
| `wid` | 整數 | 目標視窗識別碼，Windows 是 HWND，Mac 是視窗 ID。原樣傳給相應 API；沒有目標時可能為 0。 |
| `pid` | 整數 | 目標程序識別碼，沒有時為 0。 |
| `exe` | 字串 | Windows 程式身分，通常為完整路徑；Mac 是 bundle id 或可執行檔路徑，無法取得時為空。 |
| `title` | 字串 | 目標視窗標題，無法取得時為空。Mac 在工作執行緒透過輔助功能讀取。 |
| `mode` | 字串 | 根據「偏好設定 → 手勢作用於」，取固定值 `UnderCursor` 或 `Focused`，不代表滑鼠按鈕或手勢名稱。 |

目標在手勢的執行環境中確定，`pos` 可能不同於動作變數中的手勢開始座標。鍵盤輸入仍由使用中的程式接收，除非先啟用目標。Mac 的 `ms.focus_win(wid)` 只能啟用本次手勢的目標程式。Lua 不展開 `%MS_PID%` 之類預留位置，請直接使用 `pid`。

## 五分鐘語法

`local` 宣告變數，字串用單引號或雙引號，`..` 串接字串。`--` 開始註解。`if`、`elseif`、`else`、`end` 用於分支。`nil` 表示沒有值，條件中只有 `false` 和 `nil` 為假，0 和空字串都為真。

```lua
function ms_action(pos, wid, pid, exe)
  local message = "你好"
  if string.find(string.lower(exe), "firefox", 1, true) then
    message = "你好，Firefox"
  else
    message = message .. "，来自 Mousetroke"
  end
  ms.show_toast(message)
end
```

`string.find(s, needle, 1, true)` 按字面查詢，傳回位置或 `nil`，其中 `true` 表示不把標點當模式。`string.match` 使用 Lua 模式，規則與正規表示式不同。資料表可以存放命名欄位，也可以存放從 1 開始編號的清單。輔助函式可以傳回結果：

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

Lua 字串會先處理轉義，再交給按鍵解析器。`"\\"` 表示一個反斜線，`"\n"` 表示換行；`[[文字]]` 這樣的長括號字串會保留反斜線。[按鍵語法](key-sequences.md)介紹第二層語法。

## 除錯

用 `ms.log("pid=%d title=%s", pid, title)` 記錄值，再從「關於 → 開啟日誌」檢視：

- Windows：`%APPDATA%\mousetroke\mousetroke.log`。
- Mac：`~/Library/Logs/mousetroke/mousetroke.log`。

載入錯誤和執行錯誤會寫入日誌，程式碼區塊名稱為 `gesture_script`。`ms.show_toast("执行到这里")` 可以快速觀察進度。可選傳回值可能為 `nil`，先判斷再做運算或讀取欄位。不支援的 Mac 呼叫見 [API 參考](lua-api.md)。

## 執行環境與限制

每次執行都會在工作執行緒建立獨立 Lua VM，**Lua 分配記憶體上限 8 MiB**。全域變數不會跨手勢保留。執行時沒有指令數量限制或執行逾時，請避免無限迴圈。`ms.sleep(ms)` 阻塞本次指令碼的工作執行緒，不阻塞輸入鉤子或主介面執行緒。`ms.keys` 會等序列傳送完再傳回；Mac 由主執行緒計時器推進輸入，Lua 工作執行緒等待。

可用標準庫包括基礎函式、`coroutine`、`table`、`io`、`os`、`string`、`utf8`、`math` 和 `package`。沒有載入 `debug`，mlua 安全模式停用原生 C 模組載入。可以用 `os.date` 取日期。這並不是作業系統安全沙箱：`io`、`os` 和程序 API 可使用目前使用者權限存取檔案、執行命令。請只執行可信指令碼和設定。

繼續閱讀：[API 參考](lua-api.md)、[可複製範例](lua-examples.md)、[常見問題](troubleshooting.md)。
