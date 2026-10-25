---
title: "Lua 入門"
description: "最初のマウスジェスチャー Lua スクリプトを書き、ms_action の引数を学び、ログでデバッグし、実行時の制限を理解します。"
order: 8
---

# Lua 入門

## Lua を使うとき

1 つのショートカット、ウィンドウ操作、URL、アプリ、コマンドなら[単純な動作](actions.md)を使います。ジェスチャーをアプリで分岐させたい、テキストを変換したい、操作を組み合わせたいときに Lua を選んでください。Mousetroke は **Lua 5.4** を組み込んでいます。

## 最初のスクリプト

1. 設定でジェスチャーを作成または選択します。
2. **動作**で**Lua スクリプト**を選び、テキスト欄全体を次のコードに置き換えます。
3. 保存の表示を待ってから、設定の外でトリガーします。

```lua
function ms_action(pos, wid, pid, exe, title, mode)
  ms.log("Target: %s, mode: %s", exe, mode)
  ms.show_toast("My first Lua gesture", 1500)
end
```

`function` で関数を始め、`end` で終えます。Mousetroke はコードを読み込み、`ms_action` を 1 回呼び出します。処理はこの関数の中に書きます。括弧内の名前はコンテキストを受け取り、未使用の引数は省略できます。`ms.log` はランタイムログに書き込み、`ms.show_toast` は 1500 ミリ秒のメッセージを表示します。

エディターの**Lua 入門**でこのページを開けます。[Lua 使用例](lua-examples.md)に従えば、マウスジェスチャーでテキストを出力したり、クリップボード経由で貼り付けたりできます。

## エントリー引数

完全なエントリーは `ms_action(pos, wid, pid, exe, title, mode)` です。

| 引数 | Lua 型 | 意味 |
|---|---|---|
| `pos` | テーブル `{x=integer, y=integer}` | 動作のディスパッチ時に記録された最後のカーソル位置。他のモニターでは座標が負になることがあります。 |
| `wid` | integer | 対象のウィンドウ ID。Windows では HWND、Mac ではウィンドウ ID。不透明な識別子として扱ってください。対象がなければ 0 のことがあります。 |
| `pid` | integer | 対象のプロセス ID、または 0。 |
| `exe` | string | Windows の実行ファイル識別情報（通常は完全パス）、Mac のバンドル ID か実行ファイルのパス。取得できない場合は空。 |
| `title` | string | 対象のウィンドウタイトル、または空文字列。Mac はワーカー上でアクセシビリティ経由で取得します。 |
| `mode` | string | **環境設定 → ジェスチャの対象**による `UnderCursor` または `Focused`。トリガーボタンやジェスチャー名ではありません。 |

対象はジェスチャー用に取得されます。`pos` は、動作変数のジェスチャー開始座標と異なることがあります。対象をアクティブにしない限り、キーボード入力はアクティブなアプリへ送られ続けます。Mac では、`ms.focus_win(wid)` はこのジェスチャーの対象アプリだけをアクティブ化できます。`%MS_PID%` のようなプレースホルダーのテキストは Lua では展開されないため、`pid` を直接使ってください。

## 5 分でわかる Lua の基本

`local` は変数を作ります。文字列は単引用符か二重引用符で囲み、`..` で連結します。`--` はコメントです。`if`、`elseif`、`else`、`end` で分岐を選びます。`nil` は値がないことを表し、条件で偽になるのは `false` と `nil` だけです（0 と `""` は真です）。

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

ここで `string.find(s, needle, 1, true)` はリテラルとして検索し、位置か `nil` を返します。`true` は、句読点をパターンとして解釈しないためです。`string.match` は、正規表現とは異なる Lua パターンを使います。テーブルは名前付きフィールドやリストを格納でき、先頭のインデックスは 1 です。ヘルパー関数は値を返せます。

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

Lua の文字列は、キー列の解析の前にエスケープを解釈します。`"\\"` は 1 つのバックスラッシュ、`"\n"` は改行です。`[[text]]` のような長括弧文字列はバックスラッシュをそのまま保持します。第 2 層の構文は[キー送信の構文](key-sequences.md)で説明します。

## デバッグ

`ms.log("pid=%d title=%s", pid, title)` を使い、**このアプリについて → ログを開く**で確認します。

- Windows: `%APPDATA%\mousetroke\mousetroke.log`。
- Mac: `~/Library/Logs/mousetroke/mousetroke.log`。

読み込みエラーと実行エラーは、チャンク名 `gesture_script` とともにここに出力されます。`ms.show_toast("Reached this step")` は手軽な視覚確認です。省略可能な API の結果は `nil` のことがあるため、演算やテーブルアクセスの前に確認してください。Mac で対応していない呼び出しは [API リファレンス](lua-api.md)を参照してください。

## 実行環境と制限

スクリプトは実行のたびにワーカースレッド上で新しい Lua VM を作成し、**Lua の割り当て上限は 8 MiB** です。グローバル変数はジェスチャー間で保持されません。実行時に命令数や経過時間のタイムアウトはありません。無限ループは避けてください。`ms.sleep(ms)` は入力フックやメインの UI スレッドではなく、そのスクリプトのワーカーをブロックします。`ms.keys` はシーケンスの完了を待ちます。Mac では、Lua ワーカーが待機している間、メインスレッドのタイマーが入力を進めます。

利用できる標準ライブラリは、基本関数、`coroutine`、`table`、`io`、`os`、`string`、`utf8`、`math`、`package` です。`debug` は読み込まれず、ネイティブ C モジュールの読み込みは mlua のセーフモードで無効です。`os.date` は使えます。これはオペレーティングシステムのセキュリティサンドボックスではありません。`io`、`os`、プロセス API は、ユーザー権限でファイルにアクセスしたりコマンドを実行したりできます。信頼できるスクリプトと設定だけを実行してください。

次へ：[API リファレンス](lua-api.md)、[コピー＆ペーストの使用例](lua-examples.md)、[トラブルシューティング](troubleshooting.md)。
