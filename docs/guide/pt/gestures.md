---
title: "Gestos de mouse"
description: "Grave traçados e formas de mouse, combine botões, roda e teclas do teclado, e ajuste o reconhecimento de gestos."
order: 3
---

# Gestos de mouse

## Gravar um traçado direcional

Abra **Todos os aplicativos → Gestos de mouse → Novo gesto**. Dê um nome a ele e escolha **Como começa**: **Segurar o botão direito**, **Segurar o botão do meio**, **Segurar o botão lateral 1** ou **Segurar o botão lateral 2**. Os botões laterais correspondem a X1 e X2.

Na área de desenho, segure o botão esquerdo ou direito, desenhe e solte. A área de gravação aceita qualquer um dos dois botões, independentemente do gatilho escolhido. Segmentos retos tornam-se um traçado direcional, como `←` ou `↓→`; desenhe de novo para substituí-lo. Escolha uma [ação](actions.md) e experimente o gesto fora das configurações com o botão de disparo real.

## Gravar uma forma

Curvas como C, O ou S são gestos de forma. O gravador detecta automaticamente um traçado ou uma forma. Se necessário, clique em **Usar como forma** ou **Usar como traçado** depois de desenhar, ou escolha **+ Adicionar etapa → Desenhar uma forma**.

Desenhe a mesma forma 2–3 vezes, mantendo o mesmo ponto de partida e a mesma direção. Solte o botão após cada amostra. Pelo menos uma amostra válida é necessária para salvar; o gravador guarda até cinco amostras e permite remover amostras individuais. Acompanhe as mensagens de consistência e de conflito. Os gestos de forma são executados quando você solta o botão de disparo.

## Adicionar botões, roda ou teclas

Em **Depois**, clique em **+ Adicionar etapa**. Você pode adicionar **Desenhar um traçado**, **Desenhar uma forma**, um botão do mouse em **Clicar com um botão do mouse**, **Para cima** / **Para baixo** em **Girar a roda**, ou **Segurar uma tecla**. A última opção grava a tecla que você pressionar.

Por exemplo, comece com o botão direito e adicione um clique esquerdo, ou adicione rolar para cima para executar uma ação de volume repetidamente enquanto segura o gatilho. As etapas são ordenadas: execute-as na ordem exibida mantendo o botão de disparo pressionado. Remova uma etapa com o botão × dela.

Para gestos direcionais, **Executar assim que reconhecer** executa sem esperar a soltura. Isso serve para ações repetidas com a roda. Formas sempre esperam a soltura. **Não mostrar o nome do gesto ao desenhar** oculta o nome desse vínculo.

## Reconhecimento e aparência

Em **Preferências → Reconhecimento**:

| Configuração | Efeito |
|---|---|
| Tempo parado até tratar como clique | Padrão de 500 ms; controle de 150–1500 ms. Fique parado sem mover por esse tempo para passar um clique/arraste normal. |
| Sensibilidade do traçado | Padrão de 30 px; controle de 10–80 px. Valores menores reconhecem traçados mais curtos, mas facilitam gestos acidentais. |
| Os gestos agem sobre | Escolha **a janela sob o cursor** (padrão) ou **a janela ativa**. |

Em **Aparência**, **Mostrar o rastro**, **Espessura do rastro**, **Cor do rastro**, **Ao reconhecer**, **Desvanecer ao terminar**, **Mostrar o nome do gesto ao desenhar** e **Cor do nome do gesto** controlam o retorno visual. Não há aqui um controle de limite para o reconhecimento de formas.

## Encontrar, copiar e desativar

Pressione **Ctrl+F** no Windows ou **⌘+F** no Mac para pesquisar a lista atual por nome, traçado ou ação. Use **Gestos de mouse** / **Bordas da tela** e os filtros de botão/posição; **Desativado** mostra as entradas desativadas.

Selecione um gesto e clique em **Duplicar** para fazer uma cópia (Ctrl+D / ⌘+D). Dê à cópia um traçado diferente: desenhos idênticos em um mesmo escopo entram em conflito, então não conte com a ordem da lista para escolher a ação. Use o interruptor **Ativado** / **Desativado** do editor, ou a tecla Espaço em uma linha selecionada da lista, para desativar um gesto sem excluí-lo. Vínculos desativados deixam de reservar o gatilho por conta própria.

Próximo: [bordas da tela](screen-edges.md), [substituições por aplicativo](per-app.md), [sintaxe de envio de teclas](key-sequences.md).
