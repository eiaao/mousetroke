---
title: "Regras por aplicativo"
description: "Adicione aplicativos, substitua os gestos globais de mouse, exclua aplicativos que precisam de arrastar com o mouse e entenda a pausa automática em tela cheia."
order: 6
---

# Regras por aplicativo

## Adicionar um aplicativo

Clique em **Adicionar aplicativo…** na barra lateral. Escolha **Definir gestos só para ele** ou **Desativar gestos nele** e então:

- Selecione uma entrada em **Em execução**.
- Clique em **Escolher uma janela** e depois na janela do aplicativo; Esc ou clique direito cancela.
- Use **Procurar…** para escolher o aplicativo.
- Arraste um aplicativo ou seu atalho para a janela de configurações. O Mac aceita pacotes `.app` e arquivos executáveis; seus escopos usam IDs de bundle quando disponíveis.

Regras do Windows podem corresponder a um nome de executável ou caminho completo, sem diferenciar maiúsculas de minúsculas. As identidades de aplicativo do Mac normalmente são como `com.apple.Safari`. Use o seletor em vez de adivinhar a identidade. O escopo do aplicativo aparece na barra lateral e pode ter seus próprios gestos de mouse e bordas da tela.

## Gestos por aplicativo e globais

**Todos os aplicativos** contém os gestos globais. Um gesto específico de um aplicativo tem prioridade sobre o gesto global correspondente. Com **Usar também os gestos globais** marcado, os vínculos globais continuam disponíveis quando os vínculos do aplicativo não corresponderem. Desmarque para usar apenas os gestos daquele aplicativo.

Por exemplo, adicione um navegador, crie um gesto `↓→` com o botão direito e atribua lá uma ação diferente. Os outros aplicativos mantêm a ação global de `↓→`. Para copiar uma configuração existente, use [Duplicar](gestures.md#encontrar-copiar-e-desativar) dentro do escopo dela e edite o vínculo.

## Desativar gestos em um aplicativo

Selecione o aplicativo na barra lateral e ative **Desativar gestos neste aplicativo**. Isso desativa os gestos dele, os gestos globais e os gatilhos de borda da tela. Os botões direito, do meio e laterais passam direto normalmente. Os vínculos armazenados permanecem; **Reativar gestos** os restaura. **Remover este aplicativo** remove o escopo específico do aplicativo.

Isso é útil para área de trabalho remota, máquinas virtuais, modelagem 3D, ferramentas de design e editores de jogos que usam arrastar com o botão direito/do meio. A lista **Sugestão de desativação** do seletor inclui aplicativos conhecidos dessas categorias. Aplicativos instalados detectados podem ser adicionados em grupo.

## Tela cheia e primeira execução

A pausa automática em tela cheia é automática: o motor evita iniciar gestos e gatilhos de borda enquanto o aplicativo em primeiro plano for detectado como tela cheia. Ela cobre jogos em tela cheia, vídeos e apresentações; é separada do estado de pausa manual. No Windows, a detecção verifica uma janela sem bordas cobrindo o monitor dela e exclui janelas da área de trabalho/shell. O Mac compara a janela do aplicativo em primeiro plano com os limites da tela. Por isso, um aplicativo com aparência de tela cheia pode se comportar diferente nas duas plataformas.

Para um jogo em janela ou área de trabalho remota, adicione uma exclusão explícita. Para pausar em todo lugar, use o [comando da área de notificação/barra de menus ou o atalho de pausa](getting-started.md#pausar).

O guia de primeiros passos mostra os aplicativos detectados que normalmente precisam do próprio arrastar do mouse. As entradas marcadas são desativadas quando você o conclui. Revise-as antes de continuar. A configuração integrada do Mac também inclui escopos desativados para seus aplicativos de exclusão sugeridos; eles não têm vínculos próprios. Você pode alterar as exclusões depois pela barra lateral.

Próximo: [ações](actions.md), [backup da configuração](config.md), [solução de problemas](troubleshooting.md).
