---
title: "Ações"
description: "Escolha envio de teclas, ações de janela, aplicativos, URLs, comandos, teclas de mídia, menus ou Lua, e use as variáveis de gesto."
order: 5
---

# Ações

Selecione um gesto e escolha a ação dele em **Ação**. O editor oferece oito tipos. A maioria das tarefas precisa apenas de uma ação simples; use [Lua](lua-basics.md) quando precisar de condições ou de várias operações.

## Enviar teclas

Digite uma [sequência de teclas](key-sequences.md), ou clique em **Gravar**, pressione o atalho e clique em **Concluir**. Por exemplo, `^c` copia no Windows; `#c` copia no Mac. A **Ajuda de sintaxe** abre o guia de sintaxe.

**Ativar antes a janela sob o cursor** ativa primeiro o alvo do gesto. Sem ela, os atalhos vão para o aplicativo que estiver recebendo a entrada do teclado. O alvo também segue **Preferências → Os gestos agem sobre**. Não há opção de texto puro neste editor; a [entrada em modo texto](key-sequences.md#texto-e-escape) está disponível por meio do Lua.

## Ações de janela

Escolha uma operação, aplicada ao alvo do gesto:

| Operação | Comportamento |
|---|---|
| Minimizar | Minimiza a janela. |
| Maximizar / restaurar | Alterna entre maximizada e o tamanho restaurado. |
| Fechar | Pede para a janela fechar. |
| Sempre visível / desativar | Alterna o sempre no topo no Windows; sem suporte e desativado no editor do Mac. |

O controle de janelas do Mac usa a Acessibilidade. **Maximizar / restaurar** pressiona o botão de zoom do aplicativo, então o tamanho resultante segue o comportamento dele; não pede tela cheia nativa e ignora janelas já minimizadas. Os aplicativos podem diferir em quais operações de janela expõem.

## Abrir um aplicativo

**Aplicativo** aceita um executável no Windows; **Procurar…** o seleciona. No Mac, use um ID de bundle como `com.apple.Safari`, um caminho `.app` ou um caminho de executável. **Argumentos** é opcional. No Mac, os argumentos são divididos com tratamento de aspas; use caminhos completos para executáveis de linha de comando.

**Se já estiver aberto** oferece **Trocar para ele** ou **Abrir outro**. O Windows também oferece **Executar como administrador**. O Mac não suporta elevação nem opções de inicialização com filtro de janela. Não há campos de classe/título/regex no editor de ações atual.

## Abrir uma URL

Preencha a **URL** e, opcionalmente, o **Navegador**. Deixe o Navegador vazio para o navegador padrão. O Windows aceita um executável de navegador; o Mac aceita um ID de bundle ou caminho de aplicativo do navegador. Os botões **Inserir** adicionam o texto selecionado, o texto da área de transferência, o nome do aplicativo, o ID do processo ou espaços reservados do título da janela.

Para a pesquisa na web com o texto selecionado, defina a URL como:

```text
https://www.google.com/search?q=%MS_SELECTED_TEXT%
```

Selecione um texto em um aplicativo e execute o gesto. O texto selecionado é codificado para URL. Se ele já começar com `http://`, `https://` ou `www.`, o Mousetroke abre esse endereço diretamente; `www.` recebe o prefixo `https://`.

## Executar um comando

Digite o comando na área de texto. **Inserir** adiciona variáveis. O Windows o executa por um `cmd.exe /C` oculto; o Mac usa `/bin/zsh -c`. Esta é uma sintaxe de shell, então cite os argumentos corretamente. O texto substituído não recebe escape de shell; evite alimentar comandos com texto selecionado não confiável.

Por exemplo, o comando do Windows a seguir abre o Gerenciador de Tarefas:

```text
taskmgr.exe
```

No Mac, use **Abrir um aplicativo** com `com.apple.ActivityMonitor`.

## Mídia e sistema

O editor oferece **Reproduzir / pausar**, **Faixa seguinte**, **Faixa anterior**, **Aumentar volume**, **Diminuir volume**, **Sem som**, **Aumentar brilho** e **Diminuir brilho**.

O brilho no Windows ajusta em 10 pontos percentuais usando DDC/CI com fallback WMI. O brilho no Mac envia as teclas de brilho da tela integrada; DDC/CI externo não é suportado. Essas ações de sistema do Mac estão disponíveis ainda que as funções de Lua de brilho não tenham suporte. Veja [bordas da tela](screen-edges.md).

## Mostrar um menu

Clique em **+ Adicionar item de menu**, digite o **Nome do item de menu** e escolha uma ação dentro de cada item. Use **Mover para cima**, **Mover para baixo** e **Excluir o item de menu** para organizá-lo. O menu aparece na posição do gesto. O editor não expõe um campo de número de colunas e não oferece outro tipo de menu dentro de um item. O Mac suporta uma única coluna; configurações multicoluna são marcadas como sem suporte.

## Script Lua

Cole um script completo que defina `ms_action`. O editor aponta para as [noções básicas de Lua](lua-basics.md), a [referência da API](lua-api.md) e os [exemplos](lua-examples.md). A área de texto aceita Tab como dois espaços.

<a id="variables"></a>

## Variáveis

Use estes espaços reservados em **Abrir uma URL → URL**, **Executar um comando** e **Abrir um aplicativo → Aplicativo / Argumentos**. Eles não são expandidos em strings de envio de teclas, código-fonte Lua, nomes de menu ou Navegador.

| Variável | Valor |
|---|---|
| `%MS_SELECTED_TEXT%` | Seleção capturada quando a ação precisa dela; vazia se indisponível. |
| `%MS_CLIPBOARD_TEXT%` | Texto atual da área de transferência, capturado sob demanda. |
| `%MS_PID%` | ID do processo alvo. |
| `%MS_EXE%` | Identidade do executável alvo: caminho no Windows; ID de bundle ou caminho de executável no Mac. |
| `%MS_APP_NAME%` | Parte do nome de arquivo da identidade, que pode ser um ID de bundle no Mac. |
| `%MS_WID%` | Identificador de janela da plataforma como texto decimal. |
| `%MS_WIN_NAME%` | Título da janela alvo; o Mac o obtém pela Acessibilidade quando necessário. |
| `%MS_MOUSE_X%`, `%MS_MOUSE_Y%` | Coordenadas de tela no início do gesto. |
| `%MS_MOUSE_Y_FLIP%` | Coordenada inferior da tela virtual menos a coordenada Y do início do gesto. |
| `%MS_TARGET_MODE%` | `UnderCursor` ou `Focused`, conforme a preferência de alvo. |

Variáveis de alvo indisponíveis podem estar ausentes; espaços reservados desconhecidos permanecem inalterados. Somente o texto selecionado recebe codificação automática de URL em um modelo de URL. As demais substituições são inseridas como estão. O Lua usa os [argumentos de entrada](lua-basics.md#argumentos-de-entrada) e as funções de `ms` em vez da expansão de espaços reservados.
