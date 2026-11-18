---
title: "Lua 實用範例"
description: "直接複製的滑鼠手勢 Lua 指令碼：輸出文字、剪貼簿貼上、日期、選取範圍搜尋、視窗、亮度、程式與定時暫停。"
order: 10
---

# Lua 實用範例

將一個完整程式碼區塊貼上到手勢的「Lua 指令碼」動作中，每塊都定義了 `ms_action`。文字範例先在暫存文件中試用。編輯方法見 [Lua 入門](lua-basics.md)，平台支援見 [Lua API](lua-api.md)。

跨平台範例用 `package.config` 選擇 Windows 的 `^`（Control）或 Mac 的 `#`（Command）。剪貼簿範例只還原**文字**，不保留圖片等格式。200 ms 等待只是範例，不保證每個程式都足夠，較慢時請增加。範例會暫時替換剪貼簿，之後還原儲存的文字；期間新複製的內容也會在還原時被覆寫。

## 輸出固定文字

用途：用一次滑鼠手勢輸入常用句子。

```lua
function ms_action()
  ms.keys("你好，来自 Mousetroke 的问候。", true)
end
```

兩平台的文字優先模式都支援 Unicode，但仍會解析語法字元。任意包含 `%`、`+`、花括號的文字，建議用下面的剪貼簿方法。

## 貼上後還原剪貼簿文字

用途：按原樣貼上文字，再還原剪貼簿之前的文字。

```lua
function ms_action()
  local mod = package.config:sub(1, 1) == "\\" and "^" or "#"
  local saved = ms.get_clipboard_text()
  ms.set_clipboard_text("价格：50% + 税费 {示例}")
  ms.keys(mod .. "v")
  ms.sleep(200)
  ms.set_clipboard_text(saved or "")
end
```

這樣貼上的內容不經過按鍵序列轉義。原來沒有文字時，最終剪貼簿變為空文字；這些 API 無法還原原來的其他格式。

## 按程式輸出不同文字

用途：根據手勢目標的程式身分選擇句子。

```lua
function ms_action(pos, wid, pid, exe)
  local name = string.lower(exe)
  local text = "来自 Mousetroke 的问候"
  if string.find(name, "firefox", 1, true) then
    text = "你好 Firefox"
  elseif name == "com.apple.safari" then
    text = "你好 Safari"
  end
  ms.keys(text, true)
end
```

Windows 通常傳完整可執行檔路徑，Mac 通常傳 bundle id。這裡檢查的是手勢目標，輸入仍由使用中的程式接收；需要先啟用目標程式時，可先呼叫 `ms.focus_win(wid)`。

## 插入今天日期

用途：按年-月-日格式插入本機日期。

```lua
function ms_action()
  ms.keys(os.date("%Y-%m-%d"), true)
end
```

執行時可以使用 `os.date`。這裡 `%` 是 Lua 日期格式語法，最終輸出的數字和連字元適用於文字優先輸入。

## 在網站搜尋選取文字

用途：複製選取範圍，按 UTF-8 編碼後開啟 Google 搜尋。

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
    ms.show_toast("没有复制到选中文字")
  end
end
```

先在使用中的程式裡選取文字再觸發。複製前清空剪貼簿，避免複製無效時搜尋舊內容。Lua 不自動替換選取範圍變數；更簡單的[開啟網址動作](actions.md#開啟網址)可以自動擷取選取範圍。

## 給選取文字加引號或括號

用途：在可編輯文字方塊中，將選取範圍替換為帶引號的文字。

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

改成方括號時，將加上引號的表示式替換為 `"[" .. text .. "]"`。複製後需要保留選取狀態，貼上才能覆蓋選取範圍；唯讀網頁不能這樣編輯。

## 切換視窗置頂

用途：切換 Windows 視窗是否置頂。此需求請選「視窗操作 → 置頂 / 取消置頂」，該動作會查詢實際狀態並切換。Lua API 只能設定指定狀態，沒有置頂查詢函式，每次執行手勢都會建立新的 VM。Mac 不支援跨程式置頂。

Windows 如需單獨的「固定此視窗」Lua 手勢，可以用：

```lua
function ms_action(pos, wid)
  if package.config:sub(1, 1) ~= "\\" then
    ms.show_toast("Mac 暂不支持窗口置顶")
    return
  end
  if wid ~= 0 then ms.set_win_top(wid, true) end
end
```

另建取消置頂手勢時將 `true` 改為 `false`。使用 Lua 全域布林變數會在每次執行時重設，不能實作切換。

## 最大化或還原

用途：在 Windows 切換目標視窗的最大化狀態，在 Mac 切換填滿工作區的縮放。

```lua
function ms_action(pos, wid)
  if wid == 0 then return end
  local state = ms.get_win_state(wid)
  if state == nil then
    ms.show_toast("窗口不可用；Mac 请检查辅助功能权限")
    return
  end
  ms.set_win_state(wid, state == "max" and "normal" or "max")
end
```

Mac 使用與內建視窗動作相同的 AX 縮放按鈕。狀態查詢識別填滿工作區的視窗；按內容大小縮放的應用程式可能仍報告 normal，這類應用程式請用「視窗操作 → 最大化 / 還原」。缺少輔助功能權限或視窗不可用時傳回 nil。目前沒有可將視窗移動到另一顯示器的 Lua 位置／大小 API。

## 調整滑鼠所在螢幕亮度

用途：Windows 將滑鼠所在螢幕調亮 5 個百分點。

```lua
function ms_action()
  local p = ms.get_mouse_pos()
  local value = ms.set_brightness(p, 5)
  if value ~= nil then
    ms.show_toast("亮度：" .. value)
  else
    ms.show_toast("当前屏幕或平台无法使用亮度 API")
  end
end
```

將 `5` 改成 `-5` 可另建調暗手勢。Windows 需要可用的 DDC/CI 或 WMI 硬體支援，改用 WMI 時會選擇第一個亮度裝置，可能是另一塊螢幕。成功傳回 0–100 的百分比，兩平台失敗均為 nil。Mac 此處傳回 nil：現有後端僅傳送亮度鍵，不能傳回百分比或以百分點為單位調整。可用「媒體與系統 → 亮度 +」／「亮度 −」傳送內建螢幕亮度鍵，不支援依滑鼠位置選擇螢幕或外接 DDC/CI。

## 開啟資料夾

用途：用系統檔案管理器開啟目前使用者的下載資料夾。

```lua
function ms_action()
  if package.config:sub(1, 1) == "\\" then
    ms.cmd('explorer.exe "%USERPROFILE%\\Downloads"')
  else
    ms.cmd('/usr/bin/open "$HOME/Downloads"')
  end
end
```

這些環境變數由系統 Shell 展開，不是 Mousetroke 動作變數。範例假設下載資料夾位於預設位置。

## 帶參數執行程式

用途：向程式傳遞參數字串。

```lua
function ms_action()
  if package.config:sub(1, 1) == "\\" then
    ms.run("cmd.exe", "/K echo Hello from Mousetroke")
  else
    ms.run("/usr/bin/say", '"Hello from Mousetroke"')
  end
end
```

Windows 開啟命令視窗顯示文字；Mac 用 `say` 朗讀。`ms.run` 的參數是一段字串，不是表，也不傳回程序控制碼。

## 開啟工作管理員或活動監視器

用途：開啟目前平台的程序檢視工具。

```lua
function ms_action()
  if package.config:sub(1, 1) == "\\" then
    ms.keys("^+{ESC}")
  else
    ms.run("com.apple.ActivityMonitor")
  end
end
```

Windows 使用 Ctrl+Shift+Esc；Mac 按 bundle id 啟動活動監視器。

## 鎖定螢幕

用途：使用系統快速鍵鎖定目前工作階段。

```lua
function ms_action()
  if package.config:sub(1, 1) == "\\" then
    ms.keys("#l")
  else
    ms.keys("^#q")
  end
end
```

Mac 是 Control+Command+Q，Windows 是 Win+L。Ctrl+Alt+Del 不能作為普通模擬按鍵序列傳送。

## 暫停十分鐘

用途：暫時暫停引擎，Mousetroke 持續執行時到時恢復。

```lua
function ms_action()
  ms.set_paused(true)
  ms.show_toast("手势暂停十分钟", 2000)
  ms.sleep(10 * 60 * 1000)
  ms.set_paused(false)
  ms.show_toast("手势已恢复", 1500)
end
```

兩平台均可用，會保留一個休眠十分鐘的 Lua 工作執行緒。這不是持久計時器，結束或重新啟動程式後，計時也會結束。期間可以手動恢復，但等待結束時指令碼仍會再送出恢復命令。避免啟動多個副本。詳見[執行限制](lua-basics.md#執行環境與限制)。
