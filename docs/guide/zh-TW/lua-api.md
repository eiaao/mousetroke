---
title: "Lua API"
description: "Lua 5.4 的完整 ms 函式參考，包含簽名、參數型別、傳回值、範例以及 Windows 和 Mac 平台差異。"
order: 9
---

# Lua API

本頁列出共用執行環境註冊到 `ms` 的全部函式。範例是放入 `ms_action` 的語句，函式中可使用 `pos`、`wid`、`pid`、`exe`、`title`、`mode`。完整指令碼見 [Lua 入門](lua-basics.md)，可直接用作動作的指令碼見[實用範例](lua-examples.md)。

## 約定

Lua 版本為 **5.4**（`mlua` 的 `lua54`、`vendored` 特性）。`nil` 表示無值，`?` 表示可選參數。無傳回值的函式寫作 `-> nil`，可選參數省略等同於 nil。布林參數請傳真正的布林值。整數參數會將有限小數向零截斷，超出底層整數範圍會回報錯誤；指令碼中建議使用整數。

位置表應使用命名欄位 `{x=..., y=...}`，不是陣列。Mac 不支援的呼叫通常每個函式、每個程序只記一次警告，查詢傳回 nil，設定函式無操作。無傳回值不表示操作成功。Lua API 字串不經過動作變數替換。

## 流程與回饋

### `ms.log`

`ms.log(...) -> nil`

接受任意 Lua 值。第一個參數是格式字串且還有其他參數時支援 `%s`、`%d`、`%f`、`%q`、`%%`；否則以空格串接各個值。`%q` 不會額外加 Lua 引號。兩平台均寫入帶 `Lua:` 字首的執行日誌，無傳回值。

```lua
ms.log("pid=%d app=%s", pid, exe)
```

### `ms.sleep`

`ms.sleep(milliseconds: integer) -> nil`

等待非負毫秒數（64 位元無號整數）。兩平台均阻塞目前指令碼工作執行緒，手勢引擎／主執行緒繼續執行。無傳回值，執行時沒有指令碼超時。

```lua
ms.sleep(100)
```

### `ms.is_paused`

`ms.is_paused() -> boolean`

無參數。兩平台均傳回引擎手動暫停旗標，不代表全螢幕自動抑制或程式排除狀態。

```lua
ms.log(ms.is_paused())
```

### `ms.set_paused`

`ms.set_paused(paused: boolean) -> nil`

`true` 暫停，`false` 恢復，無傳回值。兩平台支援。Windows 向系統匣執行緒送出命令，緊接著查詢 `is_paused()` 可能仍是舊值；Mac 透過主執行緒套用。

```lua
ms.set_paused(true)
```

### `ms.show_settings`

`ms.show_settings() -> nil`

無參數、無傳回值。兩平台均開啟設定或將現有設定視窗帶到前面。

```lua
ms.show_settings()
```

### `ms.show_toast`

`ms.show_toast(message: string, duration_ms?: integer) -> nil`

`message` 為提示文字；可指定非負的持續時間（毫秒），預設 **1200**。兩平台均顯示螢幕提示，無傳回值。

```lua
ms.show_toast("完成", 1500)
```

## 鍵盤輸入

### `ms.keys`

`ms.keys(sequence: string, text_by_default?: boolean) -> nil`

解析並傳送[按鍵序列](key-sequences.md)，文字優先模式預設為 `false`。傳送完成後即結束，沒有傳回值，語法無效會丟擲 Lua 錯誤。兩平台支援一般鍵盤／文字輸入。Mac 跳過不支援的鍵、滑鼠和滾輪動作，並受佇列／文字長度限制；它會等待主執行緒輸入佇列，該佇列也可能包含其他序列。

```lua
ms.keys("你好", true)
```

### `ms.keys_down`

`ms.keys_down(sequence: string) -> nil`

僅 Windows：按按鍵模式解析，只按下按鍵／修飾鍵，不鬆開。該階段不重複次數，滾輪動作傳送一次。語法無效會丟擲 Lua 錯誤。請配合 `keys_up` 釋放。Mac 記錄不支援並無操作，無傳回值。

```lua
ms.keys_down("^a")
ms.sleep(50)
ms.keys_up("^a")
```

### `ms.keys_up`

`ms.keys_up(sequence: string) -> nil`

僅 Windows：按動作逆序釋放解析出的序列，修飾鍵最後釋放；延遲步驟仍會等待。語法錯誤會丟擲 Lua 錯誤。Mac 記錄不支援並無操作，無傳回值。

```lua
ms.keys_down("{SHIFT}")
ms.keys_up("{SHIFT}")
```

### `ms.is_keys_pressed`

`ms.is_keys_pressed(sequence: string) -> boolean | nil`

兩平台均檢查每個鍵盤鍵**及其字首修飾鍵**，`"^a"` 要求 Control 與 A 同時按下。字首接受任意一側，`{RCTRL}` 等明確指定的名稱要求指定側。Mac 上 `^` = Control、`%` = Option、`+` = Shift、`#` = Command。多步序列要求所有列出的鍵與修飾鍵同時按下，不檢查按下順序；重複次數不增加要求。滑鼠／滾輪、延遲或文字動作使結果為假；空序列為真。無效語法傳回 false。Mac 讀取合併工作階段狀態，缺少輔助功能權限時傳回 false，並僅記錄一次日誌。

```lua
if ms.is_keys_pressed("^a") then
  ms.show_toast("Ctrl 和 A 均已按下")
end
```

## 剪貼簿與檔案

### `ms.get_clipboard_text`

`ms.get_clipboard_text() -> string | nil`

無參數。傳回剪貼簿文字，無可用文字時為 `nil`。兩平台支援，不會自動讀取選取範圍。

```lua
local text = ms.get_clipboard_text()
ms.log(text or "没有文字")
```

### `ms.set_clipboard_text`

`ms.set_clipboard_text(text: string) -> nil`

兩平台均用文字替換剪貼簿內容，無傳回值或成功旗標，不保留其他剪貼簿格式。參見[剪貼簿範例](lua-examples.md)。

```lua
ms.set_clipboard_text("来自 Mousetroke 的问候")
```

### `ms.read_text`

`ms.read_text(path: string) -> string | nil`

兩平台均完整讀取 UTF-8 檔案，讀取或 UTF-8 解碼失敗傳回 `nil`。相對路徑基於程序工作目錄，沒有約定的指令碼資料夾。讀取內容受可用記憶體約束。

```lua
local text = ms.read_text("config.json")
ms.log(text and #text or 0)
```

## 視窗

### `ms.find_win_by_class`

`ms.find_win_by_class(pattern: string) -> table | nil`

使用 **Rust 正規表示式**查詢可見的最上層視窗：Windows 符合視窗類別名稱，Mac 符合應用程式的 **bundle ID**（例如 Lua 字串 `^com\\.apple\\.Safari$`）。傳回從 1 開始的陣列，最多 32 項，每項 `{text=标题, hwnd=窗口标识}`；找不到符合項目或正規表示式無效時為空表。預設區分大小寫，可用 `(?i)` 等旗標，不使用 Lua 模式。Mac 透過輔助功能讀取標題，缺少權限傳回 nil 並僅記錄一次日誌；`hwnd` 是 `focus_win` 和視窗狀態 API 接受的 CGWindowID。Mac 僅搜尋目前螢幕上可見的視窗。

```lua
local wins = ms.find_win_by_class("^Notepad$")
if wins and wins[1] then ms.focus_win(wins[1].hwnd) end
```

### `ms.find_win_by_title`

`ms.find_win_by_title(pattern: string) -> table | nil`

兩平台按標題的 Rust 正規表示式查詢視窗，傳回與 `find_win_by_class` 相同的、從 1 開始的 `{text, hwnd}` 陣列，最多 32 項。正規表示式無效時傳回空表。Mac 使用 AX 視窗標題，需要輔助功能權限，僅搜尋目前螢幕上可見的視窗。

```lua
local wins = ms.find_win_by_title("(?i)notepad")
if wins then ms.log(#wins) end
```

### `ms.focus_win`

`ms.focus_win(wid: integer) -> nil`

啟用視窗／程式，無傳回值或成功旗標。Windows 接受 HWND。Mac 接受手勢目標或查詢得到的 CGWindowID，取消最小化並提升對應 AX 視窗，再經主執行緒橋接啟用所屬的應用程式。Mac AX 呼叫在指令碼工作執行緒執行，重複使用內建動作按 PID／邊界定位視窗的邏輯，需要輔助功能權限；忽略失效 ID 或不可用的 AX 視窗。邊界完全相同的重疊視窗可能存在定位歧義。

```lua
if wid ~= 0 then ms.focus_win(wid) end
```

### `ms.set_win_state`

`ms.set_win_state(wid: integer, state: string) -> nil`

Windows：`"min"` 最小化，`"max"` 最大化；**其他任意字串都執行還原**，包括 `"normal"` 和 `"norm"`。Mac：`"min"` 使用 AXMinimized；`"max"` 取消最小化／結束全螢幕，視窗未填滿螢幕工作區時按下與內建動作相同的 AXZoomButton。`"normal"`、`"norm"`、`"restore"` 取消最小化／結束全螢幕，並復原填滿工作區的縮放。Mac 另支援 `"fullscreen"`，透過 AXFullScreen 要求應用程式進入全螢幕（取決於應用程式是否支援）；其他名稱忽略並僅記錄一次日誌。值區分大小寫，無傳回值。Mac 需要輔助功能權限；AX 在工作執行緒執行，經主執行緒橋接讀取螢幕幾何資訊。狀態轉換可能非同步完成。

Mac 的縮放行為由應用程式決定：按內容大小縮放後可能仍回報 `"normal"`，此時本 API 無法可靠還原，重複請求 `"max"` 可能切換縮放。這類應用程式請用內建「最大化 / 還原」切換動作。公開 AX 介面沒有統一的縮放狀態旗標。

```lua
if wid ~= 0 then ms.set_win_state(wid, "max") end
```

### `ms.get_win_state`

`ms.get_win_state(wid: integer) -> string | nil`

Windows 傳回 `"min"`、`"max"` 或 `"normal"`，不查詢置頂，也不驗證失效控制碼，失效控制碼可能表現為 normal。Mac 傳回 `"min"`、`"fullscreen"`、`"max"`（邊界與某個螢幕的工作區的差不超過 2 個邏輯點）或 `"normal"`。公開 AX 介面沒有通用的縮放狀態查詢，按內容大小縮放的視窗回報 normal。Mac 缺少權限、AX 視窗／狀態不可用或 ID 失效時傳回 nil；權限拒絕僅記錄一次日誌。

```lua
if wid ~= 0 then ms.log(ms.get_win_state(wid)) end
```

### `ms.set_win_top`

`ms.set_win_top(wid: integer, top: boolean) -> nil`

Windows 用 `true` 設定、`false` 取消置頂，不移動或改變視窗大小，無傳回值。這是設定狀態，不是切換，也不會傳回原狀態。Mac 無操作並僅記錄一次日誌：系統沒有公開的跨應用程式視窗置頂 API，內建置頂動作同樣不支援。需要切換時，Windows 編輯器選擇「視窗操作 → 置頂 / 取消置頂」。

```lua
if wid ~= 0 then ms.set_win_top(wid, true) end
```

## 滑鼠與螢幕

### `ms.get_mouse_pos`

`ms.get_mouse_pos() -> table`

無參數，兩平台均傳回目前滑鼠位置 `{x=整数, y=整数}`，可能與入口 `pos` 不同。Windows 使用螢幕像素，Mac 使用 Quartz 全域邏輯點，Y 向下增加。Mac 查詢失敗時傳回 `{x=0,y=0}`。

```lua
local p = ms.get_mouse_pos()
ms.log("%d,%d", p.x, p.y)
```

### `ms.set_mouse_pos`

`ms.set_mouse_pos(x: integer, y: integer) -> nil`

兩平台均將滑鼠移動到全域螢幕座標，無傳回值，座標可以為負數。Mac 傳送帶標記的移動事件，避免再次觸發手勢。

```lua
local p = ms.get_mouse_pos()
ms.set_mouse_pos(p.x + 20, p.y)
```

### `ms.get_screen`

`ms.get_screen(x: integer, y: integer) -> table | nil`

傳回完整螢幕矩形 `{left, top, right, bottom}`，欄位為全域整數座標，不是扣除工作列等區域的工作區。寬為 `right-left`，高為 `bottom-top`。Windows 在點不屬於任何螢幕時選最近顯示器；Mac 選包含該點的顯示器，否則改用快取中的第一台顯示器，找不到螢幕或無法取得橋接結果時可能為 `nil`。兩平台支援。

```lua
local p = ms.get_mouse_pos()
local s = ms.get_screen(p.x, p.y)
if s then ms.log(s.right - s.left, s.bottom - s.top) end
```

### `ms.get_brightness`

`ms.get_brightness(pos?: table) -> integer | nil`

可選 `{x=整数,y=整数}` 指定螢幕，省略／nil 或位置不完整時使用目前滑鼠。兩平台統一為**成功傳回 0–100 的亮度百分比，失敗傳回 nil**。Windows 使用 DDC/CI，失敗時改用 WMI；WMI 使用第一個亮度裝置，並未依滑鼠所在位置選擇裝置。Mac 目前傳回 nil 並僅記錄一次日誌：現有亮度後端僅傳送內建螢幕亮度鍵，不能讀取百分比，也不支援外接 DDC。僅 Windows 舊介面 `WG.GetBrightness` 保留失敗傳回 -1。

```lua
local value = ms.get_brightness()
if value ~= nil then ms.show_toast(tostring(value)) end
```

### `ms.set_brightness`

`ms.set_brightness(pos: table | nil, delta: integer) -> integer | nil`

雖然叫 set，但它是增減 `delta` 個百分點，不是設定絕對亮度。第一個參數請傳入命名欄位表 `{x=...,y=...}`，或**明確傳 nil** 使用目前滑鼠。成功傳回限制在 **0–100 的新亮度百分比**，兩平台失敗均為 **nil**。Windows 的 DDC/CI 與 WMI 裝置選擇同 `get_brightness`；僅 Windows 舊介面 `WG.SetBrightness` 保留失敗傳回 -1。Mac 傳回 nil 並僅記錄一次日誌：現有「媒體與系統」後端只傳送內建螢幕亮度鍵，沒有數值讀取或按百分點控制能力。本 Lua 呼叫不改變亮度；亮度鍵請使用內建動作，外接 DDC 仍不支援。

```lua
local value = ms.set_brightness(nil, 5)
ms.log(value)
```

## 網址與程序

### `ms.open`

`ms.open(url: string) -> nil`

使用預設處理程式開啟 URL，無傳回值或成功旗標。Windows 使用 Shell 開啟，也接受檔案／資料夾路徑。Mac 需要 URL，檔案可用正確編碼的 `file://` URL，或透過 `ms.cmd` 用 `/usr/bin/open` 開啟路徑。不做變數替換。

```lua
ms.open("https://mousetroke.com/")
```

### `ms.run`

`ms.run(file: string, args?: string) -> nil`

可選參數為**字串**，不是 Lua 陣列，無傳回值或程序識別碼。Windows 以一般權限啟動，採用再次執行行為。Mac 接受 bundle id、應用程式路徑或可執行檔絕對路徑，按引號規則拆成個別參數；LaunchServices 拒絕命令列可執行檔時可改以背景程序啟動，並請求新執行個體。不展開變數，需要時用 Lua 串接，不傳回程序輸出。

```lua
if package.config:sub(1, 1) == "\\" then
  ms.run("cmd.exe", "/K echo Hello")
else
  ms.run("/usr/bin/say", '"Hello from Mousetroke"')
end
```

### `ms.cmd`

`ms.cmd(command: string) -> nil`

Windows 透過隱藏的 `cmd.exe /C`，Mac 透過 `/bin/zsh -c` 執行 Shell 程式碼。無傳回值、輸出或結束代碼，不等待程序結束。Mac 管理的背景程序同時執行上限為 16，結束時記日誌。按原字串執行，此 Lua API 不展開 `%MS_*%` 預留位置，Shell 參數需自行加引號。

```lua
if package.config:sub(1, 1) == "\\" then
  ms.cmd("taskmgr.exe")
else
  ms.cmd('/usr/bin/open -a "Activity Monitor"')
end
```

繼續閱讀：[按鍵語法](key-sequences.md)、[實用範例](lua-examples.md)、[日誌位置](config.md#檔案位置)。
