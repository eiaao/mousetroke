---
title: "Lua API"
description: "Lua 5.4 の完全な ms API リファレンス。シグネチャ、戻り値、使用例、Windows と macOS の対応状況をまとめています。"
order: 9
---

# Lua API

ここでは、共有ランタイムが `ms` に登録するすべての関数を説明します。使用例は `ms_action` の中に置く文で、そこでは `pos`、`wid`、`pid`、`exe`、`title`、`mode` が使えます。完全なスクリプトは [Lua 入門](lua-basics.md)を、そのまま使える動作は[使用例](lua-examples.md)を参照してください。

## 表記の慣例

Lua バージョンは **5.4**（`lua54` と `vendored` を有効化した `mlua`）です。`nil` は値がないことを、`?` は省略可能な引数を示します。戻り値のない関数は `-> nil` と表記します。省略された省略可能引数は nil と同じです。ブール引数には本物のブール値を渡してください。整数引数は有限の小数をゼロ方向へ切り捨て、基になる整数範囲外の値は拒否します。スクリプトでは整数を使ってください。

位置テーブルは配列ではなく、`{x=..., y=...}` の名前付きフィールドを使います。Mac で対応していない呼び出しは通常、関数ごと・プロセスごとに 1 回だけログに記録され、取得系は nil を返し、設定系は何もしません。戻り値のない設定は成功の確認にはなりません。Lua API の文字列は動作変数の置換を受けません。

## フローとフィードバック

### `ms.log`

`ms.log(...) -> nil`

任意の Lua 値を受け付けます。書式文字列と追加の引数がある場合は `%s`、`%d`、`%f`、`%q`、`%%` に対応し、それ以外は値をスペースで結合します。`%q` は Lua の引用符を追加しません。両プラットフォームのランタイムログに `Lua:` の行を書き込みます。戻り値はありません。

```lua
ms.log("pid=%d app=%s", pid, exe)
```

### `ms.sleep`

`ms.sleep(milliseconds: integer) -> nil`

非負のミリ秒数（符号なし 64 ビット）だけ待機します。両プラットフォームでこのスクリプトのワーカーをブロックし、ジェスチャーエンジンとメインスレッドは動き続けます。戻り値はなく、スクリプトのタイムアウトも設けられていません。

```lua
ms.sleep(100)
```

### `ms.is_paused`

`ms.is_paused() -> boolean`

引数はありません。両プラットフォームでエンジンの手動一時停止フラグを返します。フルスクリーンによる自動抑止やアプリの除外は反映しません。

```lua
ms.log(ms.is_paused())
```

### `ms.set_paused`

`ms.set_paused(paused: boolean) -> nil`

`true` で一時停止、`false` で再開します。戻り値はありません。両プラットフォームで対応しています。Windows はトレイスレッドへコマンドを投稿するため、直後の `is_paused()` が古い値を返すことがあります。Mac はメインスレッドのブリッジ経由で適用します。

```lua
ms.set_paused(true)
```

### `ms.show_settings`

`ms.show_settings() -> nil`

引数も戻り値もありません。両プラットフォームで設定を開くか、前面に出します。

```lua
ms.show_settings()
```

### `ms.show_toast`

`ms.show_toast(message: string, duration_ms?: integer) -> nil`

`message` は表示するテキストです。省略可能な非負の長さはミリ秒で、既定は **1200** です。両プラットフォームでオーバーレイのメッセージを表示し、戻り値はありません。

```lua
ms.show_toast("Done", 1500)
```

## キーボード入力

### `ms.keys`

`ms.keys(sequence: string, text_by_default?: boolean) -> nil`

[キー列](key-sequences.md)を解析して送信します。テキスト優先モードの既定は `false` です。送信後に戻り、不正な構文は Lua エラーを発生させます。通常のキーボードとテキスト入力は両プラットフォームで対応しています。Mac は対応していないキー、マウス、ホイールの動作をスキップし、キューとテキストの上限を適用します。メインスレッドの入力キューで待機するため、他のシーケンスが混ざることもあります。

```lua
ms.keys("Hello", true)
```

### `ms.keys_down`

`ms.keys_down(sequence: string) -> nil`

Windows 専用。キーモードで解析し、キーと修飾キーを離さずに押します。このフェーズでは繰り返しは実行されず、ホイール動作は 1 回だけ発生します。不正な構文は Lua エラーを発生させます。解除するには `keys_up` と組み合わせてください。Mac は未対応をログに記録し、何もしません。戻り値はありません。

```lua
ms.keys_down("^a")
ms.sleep(50)
ms.keys_up("^a")
```

### `ms.keys_up`

`ms.keys_up(sequence: string) -> nil`

Windows 専用。解析したシーケンスを動作の逆順で解放し、修飾キーは最後に解放します。SLEEP ステップは待機します。構文エラーは Lua エラーを発生させます。Mac は未対応をログに記録し、何もしません。戻り値はありません。

```lua
ms.keys_down("{SHIFT}")
ms.keys_up("{SHIFT}")
```

### `ms.is_keys_pressed`

`ms.is_keys_pressed(sequence: string) -> boolean | nil`

両プラットフォームで、すべてのキーボードのキー**とそのプレフィックスの修飾キー**を確認します。`"^a"` は Control と A が必要です。プレフィックスはどちらかの側で満たされ、`{RCTRL}` のような明示的な名前はその側を要求します。Mac では `^` = Control、`%` = Option、`+` = Shift、`#` = Command です。複数のステップは、列挙されたキーと修飾キーがすべて同時に押されていることを意味し、押した順序ではありません。繰り返しは条件を追加しません。マウス／ホイール、SLEEP、テキストの動作があると結果は偽になり、空のシーケンスは true を返します。不正な構文は false を返します。Mac は統合されたセッション状態を読み取り、アクセシビリティ権限がない場合は 1 回だけログを記録して false を返します。

```lua
if ms.is_keys_pressed("^a") then
  ms.show_toast("Ctrl and A are down")
end
```

## クリップボードとファイル

### `ms.get_clipboard_text`

`ms.get_clipboard_text() -> string | nil`

引数はありません。クリップボードのテキストを返し、テキストを取得できない場合は `nil` を返します。両プラットフォームで対応しています。選択範囲を自動的に取得することはありません。

```lua
local text = ms.get_clipboard_text()
ms.log(text or "No text")
```

### `ms.set_clipboard_text`

`ms.set_clipboard_text(text: string) -> nil`

両プラットフォームでクリップボードの内容をテキストで置き換えます。戻り値や成功フラグはありません。他のクリップボード形式は保持されません。[クリップボードの使用例](lua-examples.md)を参照してください。

```lua
ms.set_clipboard_text("Hello from Mousetroke")
```

### `ms.read_text`

`ms.read_text(path: string) -> string | nil`

両プラットフォームでファイル全体を UTF-8 として読み込みます。読み込みや UTF-8 デコードのエラーでは `nil` を返します。相対パスは、ドキュメント化されたスクリプトフォルダーではなくプロセスの作業ディレクトリを基準にします。読み込みは利用可能なメモリにカウントされます。

```lua
local text = ms.read_text("config.json")
ms.log(text and #text or 0)
```

## ウィンドウ

### `ms.find_win_by_class`

`ms.find_win_by_class(pattern: string) -> table | nil`

**Rust の正規表現**で可視の最上位ウィンドウを検索します。Windows ではクラス名、Mac ではアプリケーションの**バンドル ID** に対して検索します（Lua 文字列ではたとえば `^com\\.apple\\.Safari$`）。1 起点の配列で最大 32 件を `{text=title, hwnd=window_id}` として返します。一致がない場合や正規表現が不正な場合は空のテーブルを返します。パターンが `(?i)` のようなフラグを使わない限り、一致は大文字小文字を区別します。これらは Lua パターンではありません。Mac はアクセシビリティ経由でタイトルを読み取り、権限がない場合は 1 回だけログを記録して nil を返します。Mac の `hwnd` は、`focus_win` とウィンドウ状態 API が受け付ける CGWindowID です。Mac では、現在画面上にあるウィンドウだけが検索対象です。

```lua
local wins = ms.find_win_by_class("^Notepad$")
if wins and wins[1] then ms.focus_win(wins[1].hwnd) end
```

### `ms.find_win_by_title`

`ms.find_win_by_title(pattern: string) -> table | nil`

両プラットフォームでウィンドウタイトルを Rust の正規表現で検索し、`find_win_by_class` と同じ 1 起点の `{text, hwnd}` エントリーを最大 32 件返します。正規表現が不正な場合は空のテーブルを返します。Mac は AX のウィンドウタイトルを使い、アクセシビリティ権限が必要で、現在画面上にあるウィンドウだけを検索します。

```lua
local wins = ms.find_win_by_title("(?i)notepad")
if wins then ms.log(#wins) end
```

### `ms.focus_win`

`ms.focus_win(wid: integer) -> nil`

ウィンドウ／アプリをアクティブ化します。戻り値や成功フラグはありません。Windows は HWND を受け付けます。Mac はジェスチャーの対象か発見した CGWindowID を受け取り、最小化を解除して AX ウィンドウを前面に出し、メインスレッドのブリッジ経由で所有アプリをアクティブ化します。Mac の AX 呼び出しはスクリプトワーカー上で実行され、PID と境界による内蔵のウィンドウ検索を再利用し、アクセシビリティ権限が必要です。古い ID や利用できない AX ウィンドウは無視されます。同じ境界が重なるウィンドウでは、曖昧になることがあります。

```lua
if wid ~= 0 then ms.focus_win(wid) end
```

### `ms.set_win_state`

`ms.set_win_state(wid: integer, state: string) -> nil`

Windows：`"min"` は最小化、`"max"` は最大化します。`"normal"` と `"norm"` を含む**それ以外の文字列は元に戻します**。Mac：`"min"` は AXMinimized を使います。`"max"` は最小化の解除とフルスクリーン脱出を行い、ウィンドウが画面の作業領域を埋めていないときは、内蔵動作と同じ AXZoomButton を押します。`"normal"`、`"norm"`、`"restore"` は最小化の解除、フルスクリーン脱出、作業領域ズームの取り消しを行います。Mac は、対応するアプリでは `"fullscreen"` も AXFullScreen 経由で受け付けます。それ以外の名前は無視され、1 回だけログに記録されます。値は大文字小文字を区別します。戻り値はありません。Mac ではアクセシビリティ権限が必要で、AX はワーカー上で実行され、画面の形状はメインスレッドのブリッジ経由で取得されます。遷移は非同期の場合があります。

Mac のズームはアプリ側の定義です。コンテンツサイズのズームは `"normal"` と報告されることがあり、この API では確実に元に戻せません。`"max"` を繰り返し要求すると切り替わってしまうことがあります。そのようなアプリでは、内蔵の**最大化 / 元に戻す**トグルを使ってください。公開された汎用の AX ズーム状態フラグはありません。

```lua
if wid ~= 0 then ms.set_win_state(wid, "max") end
```

### `ms.get_win_state`

`ms.get_win_state(wid: integer) -> string | nil`

Windows は `"min"`、`"max"`、`"normal"` を返します。最前面の照会ではなく、古いハンドルの検証もしないため、無効なハンドルが normal に見えることがあります。Mac は `"min"`、`"fullscreen"`、`"max"`（フレームが論理 2 ポイント以内で画面の作業領域と一致）、または `"normal"` を返します。Mac には汎用公開の AX ズーム状態取得がないため、コンテンツサイズのズームは normal と報告されます。権限がない場合、AX ウィンドウや状態が利用できない場合、古い ID の場合は nil を返し、権限の拒否は 1 回だけログに記録されます。

```lua
if wid ~= 0 then ms.log(ms.get_win_state(wid)) end
```

### `ms.set_win_top`

`ms.set_win_top(wid: integer, top: boolean) -> nil`

Windows は、ウィンドウを移動／リサイズせずに、常に手前表示を設定 (`true`) または解除 (`false`) します。戻り値はありません。これは状態を設定するもので、切り替えも前の状態の返却も行いません。Mac は何もせず、1 回だけログに記録します。macOS には他のアプリのウィンドウをピン留めする公開 API がなく、内蔵の最前面動作も非対応です。切り替えには、Windows のエディターで**ウィンドウ操作 → 常に手前 / 解除**を選んでください。

```lua
if wid ~= 0 then ms.set_win_top(wid, true) end
```

## マウスと画面

### `ms.get_mouse_pos`

`ms.get_mouse_pos() -> table`

引数はありません。両プラットフォームで、現在のカーソル位置を `{x=integer, y=integer}` として返します。エントリーの `pos` と異なることがあります。Windows は画面ピクセルを使います。Mac は Quartz のグローバル論理ポイントを使い、Y は下向きに増えます。Mac はカーソルの照会に失敗すると `{x=0,y=0}` を返します。

```lua
local p = ms.get_mouse_pos()
ms.log("%d,%d", p.x, p.y)
```

### `ms.set_mouse_pos`

`ms.set_mouse_pos(x: integer, y: integer) -> nil`

両プラットフォームで、ポインターをグローバル画面座標へ移動します。戻り値はありません。座標は負になることがあります。Mac はタグ付きの移動イベントを投稿するため、別のジェスチャーは開始しません。

```lua
local p = ms.get_mouse_pos()
ms.set_mouse_pos(p.x + 20, p.y)
```

### `ms.get_screen`

`ms.get_screen(x: integer, y: integer) -> table | nil`

画面全体の矩形 `{left, top, right, bottom}` を、整数のグローバル座標で返します。使用可能な作業領域ではありません。幅は `right-left`、高さは `bottom-top` です。Windows は、点がどのモニターにも含まれないとき、最も近いモニターを選びます。Mac は点を含むディスプレイを選び、なければ最初にキャッシュされたディスプレイへフォールバックします。画面やブリッジの結果がない場合は `nil` を返すことがあります。両プラットフォームで利用できます。

```lua
local p = ms.get_mouse_pos()
local s = ms.get_screen(p.x, p.y)
if s then ms.log(s.right - s.left, s.bottom - s.top) end
```

### `ms.get_brightness`

`ms.get_brightness(pos?: table) -> integer | nil`

省略可能な `{x=integer,y=integer}`で画面を選びます。省略・nil・不完全な位置は、現在のカーソルへフォールバックします。成功時は **0～100 パーセント**を、失敗時にはどちらのプラットフォームでも **nil** を返します。Windows は WMI フォールバック付きの DDC/CI を使います。WMI は最初の WMI 明るさデバイスを使い、カーソルのモニターとは照合しません。Mac は現在 nil を返し、1 回だけログに記録します。既存の明るさバックエンドは内蔵ディスプレイの明るさキーイベントを送るだけで、パーセントを読み取れません。外付け DDC には対応していません。失敗時に -1 を保持するのは、Windows のレガシーな `WG.GetBrightness` ラッパーだけです。

```lua
local value = ms.get_brightness()
if value ~= nil then ms.show_toast(tostring(value)) end
```

### `ms.set_brightness`

`ms.set_brightness(pos: table | nil, delta: integer) -> integer | nil`

名前とは異なり、絶対パーセントではなく `delta` パーセントポイントで調整します。名前付きの `{x=...,y=...}` テーブルを渡すか、**第 1 引数に nil**を渡すとカーソル位置を使います。成功時は 0～100 の範囲に丸めた新しい**パーセント**を返し、失敗時は**両プラットフォームで nil** を返します。Windows は `get_brightness` と同じ DDC/CI と WMI の選択を使います。失敗時に -1 を保持するのは、Windows のレガシーな `WG.SetBrightness` ラッパーだけです。Mac は nil を返し、1 回だけログに記録します。既存のメディアとシステムのバックエンドは内蔵ディスプレイの明るさキーだけを送るため、数値の読み取りもパーセントポイントの制御もできません。この Lua 呼び出しでは明るさは変化しません。明るさキーにはそれらの内蔵動作を使ってください。外付け DDC は引き続き非対応です。

```lua
local value = ms.set_brightness(nil, 5)
ms.log(value)
```

## URL とプロセス

### `ms.open`

`ms.open(url: string) -> nil`

URL を既定のハンドラーで開きます。戻り値や成功フラグはありません。Windows はシェルの open を使い、ファイルやフォルダーのパスも受け付けます。Mac は URL を想定しています。ファイルには正しくエンコードした `file://` URL を使うか、パスに対して `/usr/bin/open` を実行する `ms.cmd` を使ってください。変数の置換は行われません。

```lua
ms.open("https://mousetroke.com/")
```

### `ms.run`

`ms.run(file: string, args?: string) -> nil`

省略可能な引数**文字列**を渡して起動します。Lua の配列ではありません。戻り値や PID はありません。Windows は RunAgain 動作による通常の非昇格起動を使います。Mac はバンドル ID、アプリのパス、実行ファイルの絶対パスを受け付け、引数は引用符を処理して分割されます。実行ファイルの起動は、LaunchServices に拒否されるとバックグラウンドプロセスへフォールバックすることがあります。新しいインスタンスを要求します。変数の展開はなく、Lua の連結を使ってください。プロセスの出力は返されません。

```lua
if package.config:sub(1, 1) == "\\" then
  ms.run("cmd.exe", "/K echo Hello")
else
  ms.run("/usr/bin/say", '"Hello from Mousetroke"')
end
```

### `ms.cmd`

`ms.cmd(command: string) -> nil`

Windows では非表示の `cmd.exe /C`、Mac では `/bin/zsh -c` でシェルコードを実行します。値、出力、終了コードを返さず、プロセスの完了も待ちません。Mac は同時に管理するバックグラウンドプロセスを 16 に制限し、完了をログに記録します。文字列はそのまま実行され、`%MS_*%` プレースホルダーはこの Lua API では展開されません。シェルの引数は自分で引用符で囲んでください。

```lua
if package.config:sub(1, 1) == "\\" then
  ms.cmd("taskmgr.exe")
else
  ms.cmd('/usr/bin/open -a "Activity Monitor"')
end
```

続けて [キー送信の構文](key-sequences.md)、[使用例](lua-examples.md)、[ログの場所](config.md#ファイルの場所)を参照してください。
