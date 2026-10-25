---
title: "Lua 使用例"
description: "テキスト入力、クリップボードの貼り付け、日付、ウェブ検索、ウィンドウ、明るさ、一時停止など、すぐに使えるマウスジェスチャー用 Lua スクリプト集。"
order: 10
---

# Lua 使用例

完全なブロックを 1 つ、ジェスチャーの**Lua スクリプト**の動作に貼り付けます。各ブロックは `ms_action` を定義します。テキストの例は、まず使い捨てのテキストドキュメントで試してください。エディターの説明は [Lua 入門](lua-basics.md)、プラットフォーム対応の一覧は [Lua API](lua-api.md)にあります。

クロスプラットフォームの例は、`package.config` で `^`（Windows の Control）か `#`（Mac の Command）を選択します。クリップボードの例が保持するのは**テキストだけ**で、画像などの他の形式は対象外です。200 ms の待機はあくまで例で、アプリケーションによる保証はありません。貼り付け先が遅い場合は延ばしてください。これらの例はクリップボードの内容を一時的に置き換え、保存しておいたテキストを復元します。その間に別のものをコピーしても復元されます。

## 固定テキストを入力する

目的：よく使う文を、マウスジェスチャー 1 回で挿入します。

```lua
function ms_action()
  ms.keys("Hello from Mousetroke.", true)
end
```

テキスト優先モードは、両プラットフォームで Unicode に対応しています。ただし構文記号は引き続き解析されるため、`%`、`+`、波括弧などの任意の記号にはクリップボードの例を使ってください。

## クリップボードのテキストを貼り付けて戻す

目的：テキストをそのまま貼り付け、その後で元のクリップボードのテキストを復元します。

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

この方法なら、挿入するテキストのキー列エスケープを回避できます。もともとクリップボードにテキストがなかった場合、最後のクリップボードには空のテキストが入ります。他の元の形式は、これらの API では復元できません。

## アプリごとに違うテキスト

目的：ジェスチャー対象の実行ファイル識別情報で、挿入するフレーズを選びます。

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

Windows の識別情報は通常 実行ファイルの完全パス、Mac は通常バンドル ID です。判定は取得済みの対象に対して行われ、入力はアクティブなアプリに送られます。対象のアプリをアクティブにしたい場合は、先に `ms.focus_win(wid)` を使ってください。

## 今日の日付を挿入する

目的：ローカルの日付を年-月-日の形式で挿入します。

```lua
function ms_action()
  ms.keys(os.date("%Y-%m-%d"), true)
end
```

`os.date` はランタイムで使えます。`%` は Lua の日付書式の構文で、結果の数字とハイフンはテキスト優先入力でも安全です。

## 選択テキストをサイトで検索する

目的：選択範囲をコピーし、UTF-8 でエンコードした Google 検索を開きます。

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

トリガーする前に、アクティブなアプリでテキストを選択してください。先にクリップボードを空にしておくと、コピーが失敗したときに古いテキストを検索してしまいません。Lua には選択テキストの自動置換がないため、もっと簡単な [URL を開くの動作](actions.md#url-を開く)で選択範囲を収集できます。

## 選択テキストを引用符や括弧で囲む

目的：編集可能な欄で、選択テキストを引用符で囲んだものに置き換えます。

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

括弧にするには、囲む式を `"[" .. text .. "]"` に置き換えます。コピーの後もテキストが選択されたままになっている必要があり、そうしてはじめて貼り付けで置き換えられます。読み取り専用のページはこの方法では編集できません。

## 常に手前を切り替える

目的：Windows のウィンドウを最前面固定したり解除したりします。この用途には**ウィンドウ操作 → 常に手前 / 解除**を選んでください。その動作は実際の状態を照会して切り替えられます。Lua API は要求した状態を設定するだけで、最前面の取得関数はなく、ジェスチャーごとに新しい VM が作られます。Mac はアプリをまたぐ最前面制御に対応していません。

Windows で「このウィンドウをピン留め」専用の Lua ジェスチャーを別に作りたい場合は、次を使います。

```lua
function ms_action(pos, wid)
  if package.config:sub(1, 1) ~= "\\" then
    ms.show_toast("Always-on-top is unavailable on Mac")
    return
  end
  if wid ~= 0 then ms.set_win_top(wid, true) end
end
```

ピン解除専用のジェスチャーには `true` の代わりに `false` を使ってください。Lua のグローバルなブール値は実行のたびにリセットされるため、切り替えにはなりません。

## 最大化と元に戻す

目的：Windows では対象ウィンドウの最大化を、Mac では作業領域ズームを切り替えます。

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

Mac は、内蔵のウィンドウ動作と同じ AX のズームボタンを使います。状態の照会は作業領域サイズのウィンドウを認識しますが、コンテンツサイズへズームするアプリは normal と報告されることがあるため、そのようなアプリには**ウィンドウ操作 → 最大化 / 元に戻す**を使ってください。アクセシビリティ権限がない場合やウィンドウを利用できない場合は nil になります。ウィンドウを別のモニターへ移動させる Lua の位置／サイズ API はありません。

## カーソル下の画面の明るさ

目的：Windows で、カーソル下の画面の明るさを 5 パーセントポイント上げます。

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

2 つ目の暗くするジェスチャーには、`5` を `-5` に変えてください。Windows は対応する DDC/CI か WMI のハードウェアが必要です。WMI は最初の WMI 明るさデバイスへフォールバックし、それが別の画面のこともあります。成功時は 0～100 のパーセント、失敗時は両プラットフォームで nil です。Mac では nil になります。既存のバックエンドは明るさキーを送るだけで、パーセントの取得もパーセントポイントの適用もできません。内蔵画面の明るさキーには、カーソル画面の選択や外付け DDC/CI なしで、**メディアとシステム → 明るさを上げる / 明るさを下げる**を使ってください。

## フォルダーを開く

目的：現在のユーザーのダウンロードフォルダーを、プラットフォームのファイルブラウザーで開きます。

```lua
function ms_action()
  if package.config:sub(1, 1) == "\\" then
    ms.cmd('explorer.exe "%USERPROFILE%\\Downloads"')
  else
    ms.cmd('/usr/bin/open "$HOME/Downloads"')
  end
end
```

これらの環境変数は、Mousetroke の動作変数ではなくオペレーティングシステムのシェルが展開します。ダウンロードフォルダーが通常の場所にあることを前提としています。

## 引数付きでプログラムを実行する

目的：引数文字列をプログラムに渡します。

```lua
function ms_action()
  if package.config:sub(1, 1) == "\\" then
    ms.run("cmd.exe", "/K echo Hello from Mousetroke")
  else
    ms.run("/usr/bin/say", '"Hello from Mousetroke"')
  end
end
```

Windows ではコマンドウィンドウが開いてメッセージを表示します。Mac では `say` がフレーズを読み上げます。`ms.run` が受け付けるのは 1 つの引数文字列で、テーブルは渡せず、プロセスハンドルも返しません。

## タスク マネージャーかアクティビティモニタを開く

目的：プラットフォームのプロセスビューアーを開きます。

```lua
function ms_action()
  if package.config:sub(1, 1) == "\\" then
    ms.keys("^+{ESC}")
  else
    ms.run("com.apple.ActivityMonitor")
  end
end
```

Windows は Ctrl+Shift+Esc を使います。Mac はバンドル ID でアクティビティモニタを起動します。

## 画面をロックする

目的：システムのショートカットで現在のセッションをロックします。

```lua
function ms_action()
  if package.config:sub(1, 1) == "\\" then
    ms.keys("#l")
  else
    ms.keys("^#q")
  end
end
```

Mac は Control+Command+Q、Windows は Win+L を使います。Ctrl+Alt+Del は通常のキー列として合成できません。

## 10 分間ジェスチャーを一時停止する

目的：エンジンを一時的に停止し、Mousetroke を起動したまま再開します。

```lua
function ms_action()
  ms.set_paused(true)
  ms.show_toast("Gestures paused for ten minutes", 2000)
  ms.sleep(10 * 60 * 1000)
  ms.set_paused(false)
  ms.show_toast("Gestures resumed", 1500)
end
```

このスクリプトは両プラットフォームで動き、1 つの Lua ワーカーを 10 分間休眠させます。永続的なタイマーではないため、終了や再起動をすると止まります。待機中に手動で再開しても、待機が終わるとスクリプトはもう一度再開を発行します。複数のコピーを起動しないでください。[実行環境と制限](lua-basics.md#実行環境と制限)も参照してください。
