---
title: "Noções básicas de Lua"
description: "Escreva seu primeiro script Lua de gesto de mouse, conheça os argumentos de ms_action, depure com logs e entenda os limites do tempo de execução."
order: 8
---

# Noções básicas de Lua

## Quando usar Lua

Use uma [ação simples](actions.md) para um atalho, operação de janela, URL, aplicativo ou comando. Escolha Lua quando um gesto precisar ramificar por aplicativo, transformar texto ou combinar operações. O Mousetroke embute o **Lua 5.4**.

## Seu primeiro script

1. Crie ou selecione um gesto nas configurações.
2. Em **Ação**, escolha **Script Lua**. Substitua toda a área de texto pelo código a seguir.
3. Aguarde a indicação de salvo e então dispare-o fora das configurações.

```lua
function ms_action(pos, wid, pid, exe, title, mode)
  ms.log("Target: %s, mode: %s", exe, mode)
  ms.show_toast("My first Lua gesture", 1500)
end
```

`function` inicia uma função e `end` a encerra. O Mousetroke carrega o código e depois chama `ms_action` uma vez. Coloque seu trabalho dentro dessa função. Os nomes entre parênteses recebem o contexto; você pode omitir argumentos não usados. `ms.log` escreve no log do tempo de execução e `ms.show_toast` exibe uma mensagem por 1500 milissegundos.

A **Introdução ao Lua** do editor abre esta página. Siga para as [receitas de Lua](lua-examples.md) para fazer um gesto de mouse digitar texto ou colar pela área de transferência.

## Argumentos de entrada

A entrada completa é `ms_action(pos, wid, pid, exe, title, mode)`:

| Argumento | Tipo Lua | Significado |
|---|---|---|
| `pos` | table `{x=integer, y=integer}` | Última posição do cursor registrada quando a ação é disparada. As coordenadas podem ser negativas em outro monitor. |
| `wid` | integer | Identificador da janela alvo; HWND no Windows, window ID no Mac. Trate-o como um identificador opaco; pode ser 0 sem um alvo. |
| `pid` | integer | ID do processo alvo, ou 0. |
| `exe` | string | Identidade do executável no Windows, normalmente um caminho completo; ID de bundle ou caminho de executável no Mac. Vazia se indisponível. |
| `title` | string | Título da janela alvo, ou uma string vazia. O Mac obtém o título pela Acessibilidade no worker. |
| `mode` | string | Exatamente `UnderCursor` ou `Focused`, de **Preferências → Os gestos agem sobre**. Não é o botão de disparo nem o nome do gesto. |

O alvo é capturado para o gesto. `pos` pode diferir das coordenadas do início do gesto nas variáveis de ação. A entrada do teclado continua indo para o aplicativo ativo, a menos que você ative o alvo. No Mac, `ms.focus_win(wid)` pode ativar apenas o aplicativo alvo deste gesto. Textos de espaços reservados como `%MS_PID%` não são expandidos no Lua; use `pid` diretamente.

## Uma introdução de cinco minutos ao Lua

`local` cria uma variável. Strings usam aspas simples ou duplas; `..` concatena strings. `--` inicia um comentário. `if`, `elseif`, `else`, `end` escolhem um ramo. `nil` significa ausência de valor; apenas `false` e `nil` são falsos em condições (0 e `""` são verdadeiros).

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

Aqui `string.find(s, needle, 1, true)` pesquisa literalmente e devolve uma posição ou `nil`. O `true` evita interpretar pontuação como padrão. `string.match` usa padrões do Lua, que diferem de expressões regulares. Uma table pode guardar campos nomeados ou uma lista, cujo primeiro índice é 1. Uma função auxiliar pode devolver um valor:

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

As strings do Lua interpretam escapes antes da análise da sequência de teclas. `"\\"` representa uma barra invertida e `"\n"` uma quebra de linha. Strings com colchetes longos, como `[[text]]`, preservam barras invertidas. O [guia de envio de teclas](key-sequences.md) explica a segunda camada de sintaxe.

## Depuração

Use `ms.log("pid=%d title=%s", pid, title)` e consulte **Sobre → Abrir log**:

- Windows: `%APPDATA%\mousetroke\mousetroke.log`.
- Mac: `~/Library/Logs/mousetroke/mousetroke.log`.

Erros de carregamento e de execução aparecem lá com `gesture_script` como nome do chunk. `ms.show_toast("Reached this step")` é uma verificação visual rápida. Resultados opcionais de API podem ser `nil`; teste-os antes de aritmética ou acesso a table. Veja a [referência da API](lua-api.md) para chamadas sem suporte no Mac.

## Tempo de execução e limites

Cada execução de script cria uma VM Lua nova em uma thread de trabalho, com um **limite de alocação Lua de 8 MiB**. As globais não persistem entre gestos. O tempo de execução não define limite de instruções nem de tempo de relógio. Evite loops infinitos. `ms.sleep(ms)` bloqueia o worker daquele script, não o gancho de entrada nem a thread principal da interface. `ms.keys` espera a sequência terminar; no Mac, a entrada é avançada pelo temporizador da thread principal enquanto o worker do Lua espera.

As bibliotecas padrão disponíveis são as funções base, `coroutine`, `table`, `io`, `os`, `string`, `utf8`, `math` e `package`. `debug` não é carregado; o carregamento de módulos nativos em C está desativado pelo modo seguro do mlua. `os.date` está disponível. Isto não é um sandbox de segurança do sistema operacional: `io`, `os` e as APIs de processo podem acessar arquivos ou executar comandos com os seus direitos de usuário. Execute apenas scripts e configurações em que você confia.

Próximo: [referência da API](lua-api.md), [receitas de copiar e colar](lua-examples.md), [solução de problemas](troubleshooting.md).
