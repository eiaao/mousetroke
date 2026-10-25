---
title: "Receitas de Lua"
description: "Copie scripts Lua prontos para usar em gestos de mouse: texto, colar da área de transferência, datas, pesquisa na web, janelas, brilho e pausa."
order: 10
---

# Receitas de Lua

Cole um bloco completo na ação **Script Lua** de um gesto. Cada bloco define `ms_action`. Experimente primeiro as receitas de texto em um documento descartável. [Noções básicas de Lua](lua-basics.md) explica o editor e a [API Lua](lua-api.md) lista o suporte por plataforma.

As receitas multiplataforma usam `package.config` para escolher `^` (Control no Windows) ou `#` (Command no Mac). As receitas com a área de transferência preservam **somente texto**, não imagens nem outros formatos. Suas esperas de 200 ms são exemplos, não uma garantia do aplicativo; aumente-as se o destino for lento. Elas substituem o conteúdo da área de transferência temporariamente e restauram o texto salvo, mesmo que você copie outra coisa durante esse intervalo.

## Digitar um texto fixo

Finalidade: inserir uma frase reutilizável com um gesto de mouse.

```lua
function ms_action()
  ms.keys("Hello from Mousetroke.", true)
end
```

O modo texto-primeiro suporta Unicode nas duas plataformas. Ele ainda analisa os símbolos da sintaxe; use a receita da área de transferência para pontuação arbitrária como `%`, `+` ou chaves.

## Colar e restaurar o texto da área de transferência

Finalidade: colar um texto literal e depois restaurar o texto que estava na área de transferência.

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

Isto contorna o escape da sequência de teclas para o texto inserido. Se não havia texto na área de transferência, ela ficará com texto vazio no final; os outros formatos originais não podem ser restaurados com estas APIs.

## Texto diferente por aplicativo

Finalidade: usar a identidade do executável alvo do gesto para escolher uma frase.

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

No Windows, as identidades são normalmente caminhos completos de executáveis; no Mac, costuma vir um ID de bundle. Isto verifica o alvo capturado, enquanto a digitação vai para o aplicativo ativo. Use `ms.focus_win(wid)` antes se quiser ativar o aplicativo alvo.

## Inserir a data de hoje

Finalidade: inserir a data local como ano-mês-dia.

```lua
function ms_action()
  ms.keys(os.date("%Y-%m-%d"), true)
end
```

`os.date` está disponível no tempo de execução. Os caracteres `%` são a sintaxe de formato de data do Lua; os dígitos e hífens resultantes são seguros para a entrada em modo texto-primeiro.

## Pesquisar o texto selecionado em um site

Finalidade: copiar a seleção e abrir uma consulta do Google codificada em UTF-8.

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

Selecione um texto no aplicativo ativo antes de disparar. Limpar a área de transferência primeiro evita pesquisar o texto antigo dela caso Copiar não faça nada. O Lua não tem substituição automática do texto selecionado; a [ação Abrir uma URL](actions.md#abrir-uma-url), mais simples, pode coletar a seleção para você.

## Colocar o texto selecionado entre aspas ou colchetes

Finalidade: substituir o texto selecionado por uma versão entre aspas em um campo editável.

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

Para colchetes, substitua a expressão de envelopamento por `"[" .. text .. "]"`. Copiar precisa deixar o texto selecionado para que Colar o substitua. Páginas somente leitura não podem ser editadas assim.

## Alternar sempre visível

Finalidade: alternar o estado de sempre no topo de uma janela do Windows. Escolha **Ações de janela → Sempre visível / desativar** para esta receita: essa ação consegue consultar e alternar o estado real. A API Lua apenas define um estado solicitado; não há getter de sempre no topo e cada gesto recebe uma VM nova. O Mac não suporta o controle de sempre no topo entre aplicativos.

Se você quiser um gesto Lua separado de “fixar esta janela” no Windows, use:

```lua
function ms_action(pos, wid)
  if package.config:sub(1, 1) ~= "\\" then
    ms.show_toast("Always-on-top is unavailable on Mac")
    return
  end
  if wid ~= 0 then ms.set_win_top(wid, true) end
end
```

Use `false` em vez de `true` para um gesto separado de desafixar. Um booleano global em Lua reiniciaria a cada execução e não implementaria uma alternância.

## Maximizar ou restaurar

Finalidade: alternar o estado maximizado da janela alvo no Windows ou o zoom na área útil no Mac.

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

O Mac usa o mesmo botão de zoom AX da ação de janela integrada. A consulta de estado reconhece janelas no tamanho da área útil; aplicativos que dão zoom no tamanho do conteúdo ainda podem informar normal, então use **Ações de janela → Maximizar / restaurar** nesses aplicativos. Permissão de Acessibilidade ausente ou janela indisponível devolve nil. Não há API Lua de posição/tamanho de janela para mover uma janela a outro monitor.

## Brilho sob o cursor

Finalidade: aumentar o brilho da tela do cursor em 5 pontos percentuais no Windows.

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

Troque `5` por `-5` para um segundo gesto, de diminuir. O Windows precisa de hardware compatível com DDC/CI ou WMI; o WMI recorre ao primeiro dispositivo de brilho dele, que pode ser outra tela. O sucesso é uma porcentagem de 0–100; a falha é nil nas duas plataformas. O Mac devolve nil aqui: o back-end existente dele só envia teclas de brilho e não consegue devolver uma porcentagem nem aplicar um delta em pontos percentuais. Use **Mídia e sistema → Aumentar brilho / Diminuir brilho** para as teclas de brilho da tela integrada, sem seleção da tela do cursor nem DDC/CI externo.

## Abrir uma pasta

Finalidade: abrir a pasta Downloads do usuário atual com o navegador de arquivos da plataforma.

```lua
function ms_action()
  if package.config:sub(1, 1) == "\\" then
    ms.cmd('explorer.exe "%USERPROFILE%\\Downloads"')
  else
    ms.cmd('/usr/bin/open "$HOME/Downloads"')
  end
end
```

Estas variáveis de ambiente são expandidas pelo shell do sistema operacional, não pelas variáveis de ação do Mousetroke. Isto assume o local usual de Downloads.

## Executar um programa com argumentos

Finalidade: passar uma string de argumento a um programa.

```lua
function ms_action()
  if package.config:sub(1, 1) == "\\" then
    ms.run("cmd.exe", "/K echo Hello from Mousetroke")
  else
    ms.run("/usr/bin/say", '"Hello from Mousetroke"')
  end
end
```

O Windows abre uma janela de comando exibindo uma mensagem. O Mac fala a frase com `say`. `ms.run` aceita uma string de argumento única, não uma table, e não devolve um identificador de processo.

## Abrir o Gerenciador de Tarefas ou o Monitor de Atividade

Finalidade: abrir o visualizador de processos da plataforma.

```lua
function ms_action()
  if package.config:sub(1, 1) == "\\" then
    ms.keys("^+{ESC}")
  else
    ms.run("com.apple.ActivityMonitor")
  end
end
```

O Windows usa Ctrl+Shift+Esc. O Mac abre o Monitor de Atividade por ID de bundle.

## Bloquear a tela

Finalidade: bloquear a sessão atual com o atalho do sistema dela.

```lua
function ms_action()
  if package.config:sub(1, 1) == "\\" then
    ms.keys("#l")
  else
    ms.keys("^#q")
  end
end
```

O Mac usa Control+Command+Q. O Windows usa Win+L; Ctrl+Alt+Del não pode ser sintetizado como uma sequência de teclas comum.

## Pausar os gestos por dez minutos

Finalidade: pausar o motor temporariamente e retomá-lo enquanto o Mousetroke continua em execução.

```lua
function ms_action()
  ms.set_paused(true)
  ms.show_toast("Gestures paused for ten minutes", 2000)
  ms.sleep(10 * 60 * 1000)
  ms.set_paused(false)
  ms.show_toast("Gestures resumed", 1500)
end
```

Isto funciona nas duas plataformas e mantém um worker do Lua dormindo por dez minutos. Não é um temporizador persistente: sair ou reiniciar encerra. Você pode retomar manualmente durante a espera, mas o script ainda executa a retomada quando a espera termina. Evite iniciar várias cópias. Veja os [limites do tempo de execução](lua-basics.md#tempo-de-execução-e-limites).
