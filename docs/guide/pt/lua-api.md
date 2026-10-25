---
title: "API Lua"
description: "Referência completa da API ms do Lua 5.4, com assinaturas, valores de retorno, exemplos e observações de suporte no Windows/macOS."
order: 9
---

# API Lua

Estas são todas as funções registradas em `ms` pelo tempo de execução compartilhado. Os exemplos são instruções para colocar dentro de `ms_action`, onde `pos`, `wid`, `pid`, `exe`, `title` e `mode` estão disponíveis. Veja [Noções básicas de Lua](lua-basics.md) para um script completo e as [receitas](lua-examples.md) para ações prontas para usar.

## Convenções

Versão do Lua: **5.4** (`mlua` com `lua54` e `vendored`). `nil` significa ausência de valor; `?` marca um parâmetro opcional. Funções sem valores de retorno são mostradas como `-> nil`. Um argumento opcional omitido equivale a nil. Passe um booleano real para argumentos booleanos. Parâmetros inteiros truncam frações finitas em direção a zero e rejeitam valores fora do intervalo inteiro subjacente; use inteiros nos scripts.

As tables de posição usam campos nomeados `{x=..., y=...}`, não itens de array. Chamadas sem suporte no Mac em geral registram no log uma vez por função por processo; getters devolvem nil e setters não fazem nada. Um setter sem valor devolvido não é uma verificação de sucesso. As strings da API Lua não recebem substituição de variáveis de ação.

## Fluxo e feedback

### `ms.log`

`ms.log(...) -> nil`

Aceita quaisquer valores Lua. Com uma string de formato e mais argumentos, suporta `%s`, `%d`, `%f`, `%q` e `%%`; caso contrário, une os valores com espaços. `%q` não adiciona aspas no estilo Lua. Escreve uma linha `Lua:` no log do tempo de execução nas duas plataformas. Não devolve valor.

```lua
ms.log("pid=%d app=%s", pid, exe)
```

### `ms.sleep`

`ms.sleep(milliseconds: integer) -> nil`

Espera uma contagem não negativa de milissegundos (sem sinal, 64 bits). Bloqueia o worker deste script nas duas plataformas; o motor de gestos/thread principal continua. Sem valor de retorno e sem tempo limite de script instalado.

```lua
ms.sleep(100)
```

### `ms.is_paused`

`ms.is_paused() -> boolean`

Sem parâmetros. Devolve o sinalizador de pausa manual do motor nas duas plataformas. Ele não informa a supressão automática em tela cheia nem uma exclusão de aplicativo.

```lua
ms.log(ms.is_paused())
```

### `ms.set_paused`

`ms.set_paused(paused: boolean) -> nil`

`true` pausa; `false` retoma. Sem valor de retorno. Suportado nas duas plataformas. O Windows envia um comando à thread da bandeja, então um `is_paused()` imediato ainda pode ver o valor antigo. O Mac aplica pela ponte da thread principal.

```lua
ms.set_paused(true)
```

### `ms.show_settings`

`ms.show_settings() -> nil`

Sem parâmetros nem valor de retorno. Abre ou traz para frente as configurações nas duas plataformas.

```lua
ms.show_settings()
```

### `ms.show_toast`

`ms.show_toast(message: string, duration_ms?: integer) -> nil`

`message` é o texto exibido. A duração opcional, não negativa, é em milissegundos, padrão **1200**. Exibe uma mensagem sobreposta nas duas plataformas; não devolve valor.

```lua
ms.show_toast("Done", 1500)
```

## Entrada de teclado

### `ms.keys`

`ms.keys(sequence: string, text_by_default?: boolean) -> nil`

Analisa e envia a [sequência de teclas](key-sequences.md); o modo texto-primeiro tem padrão `false`. Retorna após o envio; sintaxe malformada gera um erro Lua. As duas plataformas suportam entrada comum de teclado/texto. O Mac ignora teclas sem suporte, ações de mouse e roda, e impõe seus limites de fila/texto. Ela espera na sua fila de entrada da thread principal, que também pode conter outras sequências.

```lua
ms.keys("Hello", true)
```

### `ms.keys_down`

`ms.keys_down(sequence: string) -> nil`

Somente Windows: analisa em modo de teclas e pressiona teclas/modificadores sem soltar. As repetições não se repetem nesta fase; as ações de roda disparam uma vez. Sintaxe inválida gera um erro Lua. Combine com `keys_up` para soltar. O Mac registra no log que não há suporte e não faz nada. Sem valor de retorno.

```lua
ms.keys_down("^a")
ms.sleep(50)
ms.keys_up("^a")
```

### `ms.keys_up`

`ms.keys_up(sequence: string) -> nil`

Somente Windows: solta a sequência analisada na ordem inversa das ações, com os modificadores soltos por último. As etapas de sleep ainda esperam. Erros de sintaxe geram um erro Lua. O Mac registra no log que não há suporte e não faz nada. Sem valor de retorno.

```lua
ms.keys_down("{SHIFT}")
ms.keys_up("{SHIFT}")
```

### `ms.is_keys_pressed`

`ms.is_keys_pressed(sequence: string) -> boolean | nil`

As duas plataformas verificam todas as teclas do teclado **e seus modificadores de prefixo**: `"^a"` exige Control e A. Qualquer lado satisfaz um prefixo; nomes explícitos como `{RCTRL}` exigem aquele lado. No Mac, `^` = Control, `%` = Option, `+` = Shift e `#` = Command. Várias etapas significam que todas as teclas/modificadores listados precisam estar pressionados juntos, não que foram pressionados em ordem; repetições não adicionam exigência. Ações de mouse/roda, sleep ou texto tornam o teste falso; uma sequência vazia devolve verdadeiro. Sintaxe inválida devolve falso. O Mac lê o estado combinado da sessão e devolve falso sem a permissão de Acessibilidade, registrando no log uma vez.

```lua
if ms.is_keys_pressed("^a") then
  ms.show_toast("Ctrl and A are down")
end
```

## Área de transferência e arquivos

### `ms.get_clipboard_text`

`ms.get_clipboard_text() -> string | nil`

Sem parâmetros. Devolve o texto da área de transferência, ou `nil` se não houver texto. Suportado nas duas plataformas; isto não busca a seleção automaticamente.

```lua
local text = ms.get_clipboard_text()
ms.log(text or "No text")
```

### `ms.set_clipboard_text`

`ms.set_clipboard_text(text: string) -> nil`

Substitui o conteúdo da área de transferência pelo texto nas duas plataformas; sem valor de retorno ou sinalizador de sucesso. Os demais formatos da área de transferência não são preservados. Veja as [receitas com a área de transferência](lua-examples.md).

```lua
ms.set_clipboard_text("Hello from Mousetroke")
```

### `ms.read_text`

`ms.read_text(path: string) -> string | nil`

Lê o arquivo inteiro como UTF-8 nas duas plataformas. Devolve `nil` em erro de leitura ou de decodificação UTF-8. Caminhos relativos usam o diretório de trabalho do processo, não uma pasta de scripts documentada. As leituras contam para a memória disponível.

```lua
local text = ms.read_text("config.json")
ms.log(text and #text or 0)
```

## Windows

### `ms.find_win_by_class`

`ms.find_win_by_class(pattern: string) -> table | nil`

Pesquisa janelas de nível superior visíveis com uma **expressão regular do Rust**, no Windows por nomes de classe ou no Mac por **IDs de bundle** de aplicativos (por exemplo `^com\\.apple\\.Safari$` numa string Lua). Devolve até 32 entradas em um vetor indexado a partir de 1: `{text=title, hwnd=window_id}`; sem correspondência ou com regex inválida, devolve uma table vazia. A correspondência diferencia maiúsculas de minúsculas, a menos que o padrão use uma flag como `(?i)`; estes não são padrões do Lua. O Mac lê os títulos pela Acessibilidade, devolvendo nil sem a permissão e registrando no log uma vez. Seu `hwnd` é um CGWindowID aceito por `focus_win` e pelas APIs de estado de janela. No Mac, apenas as janelas atualmente na tela são pesquisadas.

```lua
local wins = ms.find_win_by_class("^Notepad$")
if wins and wins[1] then ms.focus_win(wins[1].hwnd) end
```

### `ms.find_win_by_title`

`ms.find_win_by_title(pattern: string) -> table | nil`

Pesquisa títulos de janelas nas duas plataformas com uma expressão regular do Rust, devolvendo as mesmas entradas `{text, hwnd}` indexadas a partir de 1 de `find_win_by_class`, no máximo 32. Uma regex inválida devolve uma table vazia. O Mac usa títulos de janela AX, exige a permissão de Acessibilidade e pesquisa apenas janelas atualmente na tela.

```lua
local wins = ms.find_win_by_title("(?i)notepad")
if wins then ms.log(#wins) end
```

### `ms.focus_win`

`ms.focus_win(wid: integer) -> nil`

Ativa uma janela/aplicativo; sem valor de retorno ou sinalizador de sucesso. O Windows recebe um HWND. O Mac aceita o alvo do gesto ou um CGWindowID descoberto, desfaz a minimização, eleva sua janela AX e então ativa o aplicativo dono pela ponte da thread principal. As chamadas AX do Mac rodam no worker do script, reutilizam a busca integrada de janela por PID/limites e exigem a permissão de Acessibilidade; IDs obsoletos ou janelas AX indisponíveis são ignorados. Aplicativos com limites de janela idênticos e sobrepostos podem ficar ambíguos.

```lua
if wid ~= 0 then ms.focus_win(wid) end
```

### `ms.set_win_state`

`ms.set_win_state(wid: integer, state: string) -> nil`

Windows: `"min"` minimiza, `"max"` maximiza; **qualquer outra string restaura**, incluindo `"normal"` e `"norm"`. Mac: `"min"` usa AXMinimized; `"max"` desfaz a minimização/sai da tela cheia e pressiona o mesmo AXZoomButton da ação integrada quando a janela não preenche a área útil da tela dela. `"normal"`, `"norm"` e `"restore"` desfazem a minimização/saem da tela cheia e desativam o zoom na área útil. O Mac também aceita `"fullscreen"` por meio de AXFullScreen quando o aplicativo suporta; todo outro nome é ignorado e registrado no log uma vez. Os valores diferenciam maiúsculas de minúsculas. Sem valor de retorno. O Mac exige a permissão de Acessibilidade; a AX roda no worker e a geometria da tela vem pela ponte da thread principal. As transições podem ser assíncronas.

O Zoom do Mac é definido pelo aplicativo: um zoom no tamanho do conteúdo ainda pode informar `"normal"` e não pode ser restaurado de forma confiável com esta API; solicitações `"max"` repetidas podem alterná-lo. Para esses aplicativos, use a alternância integrada **Maximizar / restaurar**. Não existe uma flag pública universal de estado de zoom AX.

```lua
if wid ~= 0 then ms.set_win_state(wid, "max") end
```

### `ms.get_win_state`

`ms.get_win_state(wid: integer) -> string | nil`

O Windows devolve `"min"`, `"max"` ou `"normal"`. Isto não é uma consulta de sempre no topo e não valida um identificador obsoleto; um identificador inválido pode parecer normal. O Mac devolve `"min"`, `"fullscreen"`, `"max"` (a moldura coincide com a área útil de uma tela em até 2 pontos lógicos) ou `"normal"`. O Mac não tem um getter público universal de estado de zoom AX, então um zoom no tamanho do conteúdo informa normal. Ele devolve nil para permissão ausente, janela/estado AX indisponível ou ID obsoleto; a negação de permissão registra no log uma vez.

```lua
if wid ~= 0 then ms.log(ms.get_win_state(wid)) end
```

### `ms.set_win_top`

`ms.set_win_top(wid: integer, top: boolean) -> nil`

O Windows define (`true`) ou limpa (`false`) o sempre no topo sem mover/redimensionar a janela; sem valor de retorno. Isto define um estado, não alterna nem devolve o estado anterior. O Mac não faz nada e registra no log uma vez: o macOS não tem API pública para fixar a janela de outro aplicativo, e a ação integrada de sempre no topo também não tem suporte. Para alternar, escolha **Ações de janela → Sempre visível / desativar** no editor do Windows.

```lua
if wid ~= 0 then ms.set_win_top(wid, true) end
```

## Mouse e tela

### `ms.get_mouse_pos`

`ms.get_mouse_pos() -> table`

Sem parâmetros. As duas plataformas devolvem `{x=integer, y=integer}` para a posição atual do cursor, que pode diferir do `pos` de entrada. O Windows usa pixels de tela; o Mac usa pontos lógicos globais do Quartz, com Y aumentando para baixo. O Mac devolve `{x=0,y=0}` se a consulta do cursor falhar.

```lua
local p = ms.get_mouse_pos()
ms.log("%d,%d", p.x, p.y)
```

### `ms.set_mouse_pos`

`ms.set_mouse_pos(x: integer, y: integer) -> nil`

Move o cursor para coordenadas globais de tela nas duas plataformas; sem valor de retorno. As coordenadas podem ser negativas. O Mac envia um evento de movimento marcado, de modo que não inicia outro gesto.

```lua
local p = ms.get_mouse_pos()
ms.set_mouse_pos(p.x + 20, p.y)
```

### `ms.get_screen`

`ms.get_screen(x: integer, y: integer) -> table | nil`

Devolve o retângulo completo da tela `{left, top, right, bottom}` como coordenadas globais inteiras, não a área útil. A largura é `right-left`; a altura é `bottom-top`. O Windows escolhe o monitor mais próximo se o ponto estiver fora de todos os monitores. O Mac escolhe a tela contenedora ou recorre à primeira tela em cache; pode devolver `nil` se não houver resultado de tela/ponte disponível. As duas plataformas suportam esta chamada.

```lua
local p = ms.get_mouse_pos()
local s = ms.get_screen(p.x, p.y)
if s then ms.log(s.right - s.left, s.bottom - s.top) end
```

### `ms.get_brightness`

`ms.get_brightness(pos?: table) -> integer | nil`

Uma `{x=integer,y=integer}` opcional escolhe a tela; omitida/nil ou uma posição incompleta recorre ao cursor atual. Devolve **0–100 por cento em caso de sucesso, nil em caso de falha** em ambas as plataformas. O Windows usa DDC/CI com fallback WMI. O WMI usa o primeiro dispositivo de brilho dele, não um monitor correspondente ao cursor. O Mac atualmente devolve nil e registra no log uma vez: o back-end de brilho existente dele só envia eventos das teclas de brilho da tela integrada, que não conseguem ler uma porcentagem. DDC externo não tem suporte. Apenas o wrapper legado do Windows `WG.GetBrightness` mantém -1 na falha.

```lua
local value = ms.get_brightness()
if value ~= nil then ms.show_toast(tostring(value)) end
```

### `ms.set_brightness`

`ms.set_brightness(pos: table | nil, delta: integer) -> integer | nil`

Apesar do nome, isto ajusta em `delta` pontos percentuais; não define uma porcentagem absoluta. Passe uma table nomeada `{x=...,y=...}` ou **nil como primeiro argumento** para usar o cursor. O sucesso devolve a nova **porcentagem de 0–100**, limitada a esse intervalo; a falha devolve **nil nas duas plataformas**. O Windows usa a mesma seleção DDC/CI e WMI de `get_brightness`. Apenas o wrapper legado do Windows `WG.SetBrightness` mantém -1 na falha. O Mac devolve nil e registra no log uma vez: o back-end Mídia e sistema existente só envia teclas de brilho da tela integrada, sem leitura numérica nem controle por pontos percentuais. Nenhuma alteração de brilho é feita por esta chamada Lua; use essas ações integradas para as teclas de brilho. DDC externo continua sem suporte.

```lua
local value = ms.set_brightness(nil, 5)
ms.log(value)
```

## URLs e processos

### `ms.open`

`ms.open(url: string) -> nil`

Abre uma URL com o manipulador padrão; sem valor de retorno ou sinalizador de sucesso. O Windows usa shell open e também aceita um caminho de arquivo/pasta. O Mac espera uma URL; use uma URL `file://` devidamente codificada para um arquivo, ou `ms.cmd` para invocar `/usr/bin/open` num caminho. Nenhuma substituição de variáveis é feita.

```lua
ms.open("https://mousetroke.com/")
```

### `ms.run`

`ms.run(file: string, args?: string) -> nil`

Inicia com uma **string** de argumento opcional, não um array Lua; não devolve valor nem PID. O Windows usa uma inicialização comum, sem elevação, com comportamento RunAgain. O Mac aceita um ID de bundle, caminho de aplicativo ou caminho absoluto de executável; os argumentos são divididos com tratamento de aspas. A inicialização de executável pode recorrer a um processo em segundo plano se o LaunchServices a rejeitar. Ela pede uma nova instância. Sem expansão de variáveis; use concatenação Lua. A saída do processo não é devolvida.

```lua
if package.config:sub(1, 1) == "\\" then
  ms.run("cmd.exe", "/K echo Hello")
else
  ms.run("/usr/bin/say", '"Hello from Mousetroke"')
end
```

### `ms.cmd`

`ms.cmd(command: string) -> nil`

Executa código de shell por um `cmd.exe /C` oculto no Windows ou `/bin/zsh -c` no Mac. Não devolve valor, saída nem código de saída, e não espera a conclusão do processo. O Mac limita os processos em segundo plano gerenciados simultaneamente a 16 e registra a conclusão. As strings são executadas como fornecidas; os espaços reservados `%MS_*%` não são expandidos por esta API Lua. Cite você mesmo os argumentos de shell.

```lua
if package.config:sub(1, 1) == "\\" then
  ms.cmd("taskmgr.exe")
else
  ms.cmd('/usr/bin/open -a "Activity Monitor"')
end
```

Continue com a [sintaxe de envio de teclas](key-sequences.md), as [receitas](lua-examples.md) ou os [locais do log](config.md#locais-dos-arquivos).
