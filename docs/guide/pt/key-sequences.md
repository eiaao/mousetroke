---
title: "Sintaxe de envio de teclas"
description: "Sintaxe completa de envio de teclas do Mousetroke: modificadores, grupos, teclas nomeadas, repetições, atrasos, entrada de mouse e atalhos do Windows/Mac."
order: 7
---

# Sintaxe de envio de teclas

## Comece com um atalho

Em **Enviar teclas**, cole `^c` para enviar Ctrl+C no Windows, ou `#c` para enviar Command+C no Mac. Você também pode clicar em **Gravar**, pressionar a combinação e clicar em **Concluir**. O Lua usa a mesma sintaxe por meio de `ms.keys(sequence)`.

Os caracteres normalmente significam teclas nas posições do teclado americano (US). `abc` pressiona A, B, C em sequência; maiúsculas `ABC` não adicionam Shift. Use `+a` para Shift+A. Para Unicode ou texto fixo, veja [texto e escape](#texto-e-escape) e as [receitas com a área de transferência](lua-examples.md).

## Modificadores e grupos

| Prefixo | Windows | Mac |
|---|---|---|
| `^` | Ctrl | Control ⌃ |
| `%` | Alt | Option ⌥ |
| `+` | Shift | Shift ⇧ |
| `#` | Win | Command ⌘ |

Um prefixo aplica-se à tecla seguinte ou a um grupo entre parênteses. `^+t` envia Ctrl+Shift+T. `^(ac)` aplica Ctrl tanto a A quanto a C. `^ac` aplica Ctrl somente a A. Os grupos podem ser aninhados; use parênteses balanceados. Os modificadores de grupo valem para cada ação de tecla, então não conte com o modificador permanecendo continuamente pressionado entre as teclas. Um modificador isolado no final não é um pressionamento de tecla: use `{CTRL}`, por exemplo, quando precisar da própria tecla modificadora.

O `ms.is_keys_pressed` do Lua verifica a união de todas as teclas e modificadores no momento da consulta nas duas plataformas: `^ac` exige Control, A e C pressionados juntos. Qualquer lado satisfaz um prefixo; os nomes explícitos `{LCTRL}` / `{RCTRL}` mantêm o lado. As repetições são ignoradas; etapas de sleep, texto, mouse ou roda devolvem falso. Ele não reconhece uma sequência de pressionamentos passados.

## Teclas nomeadas, repetições e atrasos

Use `{ENTER}` para Enter. Os nomes não diferenciam maiúsculas de minúsculas. `{LEFT 3}` pressiona Left três vezes. Uma contagem positiva repete a tecla; contagem zero ou negativa volta a uma. `{SLEEP 100}` espera 100 ms; exige um inteiro não negativo. Separe o nome e o argumento com um espaço.

Exemplo: `^c{SLEEP 100}^v` copia, espera um pouco e depois cola no Windows. O equivalente no Mac é `#c{SLEEP 100}#v`.

A tabela abaixo enumera os nomes aceitos pelo parser, incluindo apelidos e todos os nomes numerados de F/teclado numérico. Uma única letra ASCII, dígito ou caractere de pontuação americano suportado também pode aparecer entre chaves, como `{a}` ou `{+}`.

| Nomes | Significado |
|---|---|
| `{ENTER}`, `{RETURN}` | Enter |
| `{ESC}` | Escape |
| `{TAB}` | Tab |
| `{BACKSPACE}` | Backspace |
| `{SPACE}` | Espaço |
| `{CAPSLOCK}` | Caps Lock |
| `{INSERT}`, `{INS}` | Insert (somente Windows) |
| `{DELETE}`, `{DEL}` | Delete / delete para frente |
| `{HOME}`, `{END}` | Home / End |
| `{PGUP}`, `{PGDN}` | Page Up / Page Down |
| `{UP}`, `{DOWN}`, `{LEFT}`, `{RIGHT}` | Teclas de seta |
| `{PRTSC}` | Print Screen (somente Windows) |
| `{BREAK}` | Break (somente Windows) |
| `{MEDIA_NEXT}`, `{MEDIA_PREV}`, `{MEDIA_STOP}`, `{MEDIA_PLAY}`, `{MEDIA_PLAY_PAUSE}` | Teclas de mídia (somente Windows em sequências de teclas) |
| `{VOLUME_MUTE}`, `{VOLUME_DOWN}`, `{VOLUME_UP}` | Teclas de volume (somente Windows em sequências de teclas) |
| `{CTRL}`, `{ALT}`, `{SHIFT}` | Control / Alt (Option no Mac) / Shift |
| `{LCTRL}`, `{RCTRL}`, `{LSHIFT}`, `{RSHIFT}`, `{LALT}`, `{RALT}` | Teclas modificadoras esquerda/direita; Alt é Option no Mac |
| `{LWIN}`, `{WIN}`, `{RWIN}` | Tecla Windows; Command no Mac |
| `{F1}`, `{F2}`, `{F3}`, `{F4}`, `{F5}`, `{F6}`, `{F7}`, `{F8}`, `{F9}`, `{F10}`, `{F11}`, `{F12}`, `{F13}`, `{F14}`, `{F15}`, `{F16}`, `{F17}`, `{F18}`, `{F19}`, `{F20}`, `{F21}`, `{F22}`, `{F23}`, `{F24}` | Teclas de função; o Mac suporta F1–F20 |
| `{N0}`, `{N1}`, `{N2}`, `{N3}`, `{N4}`, `{N5}`, `{N6}`, `{N7}`, `{N8}`, `{N9}` | Dígitos do teclado numérico 0–9 |
| `{MOUSE_LEFT}`, `{MOUSE_RIGHT}`, `{MOUSE_MIDDLE}`, `{MOUSE_X1}`, `{MOUSE_X2}` | Cliques do mouse (somente Windows) |
| `{MOUSE_WHEEL}`, `{MOUSE_HWHEEL}` | Roda: ex. {MOUSE_WHEEL 120}, {MOUSE_HWHEEL -120} (somente Windows) |
| `{SLEEP}` | Atraso: ex. {SLEEP 100}, em milissegundos |

## Botões do mouse e roda

O Windows suporta `{MOUSE_LEFT}`, `{MOUSE_RIGHT}`, `{MOUSE_MIDDLE}`, `{MOUSE_X1}` e `{MOUSE_X2}` como cliques. `{MOUSE_LEFT 2}` envia dois cliques; ele não reposiciona o cursor.

`{MOUSE_WHEEL 120}` rola um incremento para cima; `-120` rola para baixo. `{MOUSE_HWHEEL 120}` rola para a direita; `-120` rola para a esquerda. O argumento é um delta da roda com sinal, não uma contagem de repetições. Esta é a convenção de entrada injetada; ela difere do sinal usado pelas etapas de roda gravadas nos gestos.

O Mac interpreta esses tokens, mas ignora ações de mouse e roda durante a execução da entrada. Elas não podem ser usadas para sintetizar cliques ou rolagens do Mac por `ms.keys`.

## Texto e escape

O modo texto-primeiro está disponível apenas com o Lua `ms.keys(s, true)`; o editor Enviar teclas sempre usa o modo de teclas. Caracteres comuns tornam-se texto Unicode, mas `^`, `%`, `+`, `#`, os parênteses e a chave de abertura ainda iniciam a sintaxe. Caracteres únicos entre chaves continuam sendo ações de tecla no modo texto-primeiro.

Para atalhos dentro do modo texto-primeiro, coloque a tecla entre chaves: `^{c}` envia Ctrl+C e `#{c}` envia Command+C. Um `^c` sem chaves nesse modo emite o texto `c` e descarta o modificador. Você também pode fazer uma chamada separada `ms.keys("^c")` usando o modo normal de teclas.

| Caractere literal | Grafia da sequência |
|---|---|
| `^` | `{^}` |
| `%` | `{%}` |
| `+` | `{+}` |
| `#` | `{#}` |
| `(` | `{(}` |
| `)` | `{)}` |
| `{` | `{{}` |
| `}` | `}` (sem chaves) |

A barra invertida não é um escape de sequência de teclas. Em uma string Lua entre aspas, escreva `"\\"` para uma barra invertida, ou use uma string com colchetes longos. Para textos arbitrários com símbolos da sintaxe, o [método da área de transferência](lua-examples.md#colar-e-restaurar-o-texto-da-área-de-transferência) evita as duas camadas de escape.

## Atalhos comuns

Estes dependem das configurações de atalho do aplicativo de destino e do sistema operacional.

| Finalidade | Sequência no Windows | Sequência no Mac |
|---|---|---|
| Copiar | `^c` | `#c` |
| Colar | `^v` | `#v` |
| Recortar | `^x` | `#x` |
| Selecionar tudo | `^a` | `#a` |
| Desfazer | `^z` | `#z` |
| Nova aba | `^t` | `#t` |
| Fechar aba | `^w` | `#w` |
| Reabrir aba fechada | `^+t` | `#+t` |
| Aba seguinte | `^{TAB}` | `^{TAB}` |
| Aba anterior | `^+{TAB}` | `^+{TAB}` |
| Trocar de aplicativo | `%{TAB}` | `#{TAB}` |
| Área de trabalho virtual anterior/seguinte | `#^{LEFT}` / `#^{RIGHT}` | `^{LEFT}` / `^{RIGHT}` |
| Recortar região da tela | `#+s` | `#+4` |
| Mostrar a área de trabalho | `#d` | `{F11}` |
| Gerenciador de Tarefas / Monitor de Atividade | `^+{ESC}` | Usar Abrir um aplicativo: `com.apple.ActivityMonitor` |
| Bloquear a tela | `#l` | `^#q` |

## Por que Ctrl+Alt+Del não pode ser enviado

Ctrl+Alt+Del é a Sequência de Atenção Segura do Windows. O Windows ignora entrada sintética comum para essa sequência, então `^%{DEL}` não consegue abrir essa tela de segurança. Use `^+{ESC}` para o Gerenciador de Tarefas, `#l` para bloquear, ou uma ação **Executar um comando** contendo `shutdown /l` para encerrar a sessão (o que encerra a sessão).

## Limites e observações por plataforma

O parser compartilhado não tem limite geral de tamanho de string nem de contagem de repetições. As contagens de repetição e os atrasos são armazenados como inteiros sem sinal de 32 bits; os deltas da roda, como inteiros com sinal de 32 bits. Valores fora do intervalo podem sofrer wrap na conversão, então use contagens e atrasos pequenos e práticos. Um nome de tecla inválido ou um argumento obrigatório ausente gera um erro de análise; o editor valida enquanto você digita.

O Windows executa as sequências em threads de trabalho e faz sleep para `{SLEEP ms}`. O Mac envia a entrada em lotes controlados por temporizador. Strings de ação do Mac maiores que 16.384 bytes são rejeitadas; para entrada Lua já analisada, os limites são 16.384 ações e 16.384 bytes por ação de texto. Sua fila de espera comporta até 64 entradas; o excesso de entrada é descartado e registrado no log.

No Mac, F1–F20, os dígitos do teclado numérico e as teclas de navegação mapeadas estão disponíveis. Insert/INS, PRTSC, BREAK, F21–F24, tokens de mídia/volume e tokens de mouse/roda não têm suporte e são ignorados. Use **Mídia e sistema** para mídia e volume. `ms.keys_down` e `ms.keys_up` são exclusivos do Windows; veja a [API Lua](lua-api.md).
