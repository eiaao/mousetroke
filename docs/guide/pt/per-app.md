---
title: "Regras por aplicativo"
description: "Adicione aplicativos, personalize gestos, exclua aplicativos e configure a pausa em tela cheia com exceções por aplicativo."
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

**Todos os aplicativos** contém os gestos globais. Um gesto específico de um aplicativo tem prioridade sobre o gesto global correspondente. Com **Usar também os gestos globais** ativado, os vínculos globais continuam disponíveis quando os vínculos do aplicativo não corresponderem. Desative para usar apenas os gestos daquele aplicativo.

Por exemplo, adicione um navegador, crie um gesto `↓→` com o botão direito e atribua lá uma ação diferente. Os outros aplicativos mantêm a ação global de `↓→`. Para copiar uma configuração existente, use [Duplicar](gestures.md#encontrar-copiar-e-desativar) dentro do escopo dela e edite o vínculo. Para gravar um traçado uma única vez e dar a ele uma ação diferente em cada aplicativo, use a [biblioteca de gestos](gesture-library.md).

O editor de gestos mostra onde o vínculo está armazenado. Clique com o botão direito no gesto da lista e escolha **Copiar para / Mover para…**, selecione os aplicativos de destino e clique em **Copiar** ou **Mover**. Você pode escolher vários aplicativos já adicionados de uma vez ou adicionar um aplicativo pela caixa de diálogo. As cópias têm configurações independentes; mover remove o vínculo do escopo original. Se um destino já tiver o mesmo desenho, escolha se deseja ignorá-lo ou substituí-lo. A operação inteira pode ser desfeita pelo aviso. Um vínculo copiado para um aplicativo com os gestos desativados é mantido, mas só será executado depois que os gestos forem ativados nele.

## Desativar gestos em um aplicativo

Selecione o aplicativo na barra lateral e desative **Usar gestos**. Isso desativa os gestos dele, os gestos globais e os gatilhos de borda da tela. Os botões direito, do meio e laterais passam direto normalmente. Os vínculos armazenados permanecem; ao reativar **Usar gestos**, eles são restaurados. O botão de lixeira **Remover este aplicativo**, no canto superior direito da página do aplicativo, remove o escopo específico do aplicativo.

Isso é útil para área de trabalho remota, máquinas virtuais, modelagem 3D, ferramentas de design e editores de jogos que usam arrastar com o botão direito/do meio. A lista **Sugestão de desativação** do seletor inclui aplicativos conhecidos dessas categorias. Aplicativos instalados detectados podem ser adicionados em grupo.

## Tela cheia e primeira execução

A pausa automática em tela cheia está ativada por padrão: o motor evita iniciar gestos e gatilhos de borda enquanto o aplicativo em primeiro plano for detectado como tela cheia. Ela cobre jogos em tela cheia, vídeos e apresentações; é separada do estado de pausa manual. No Windows, a detecção verifica uma janela sem bordas cobrindo o monitor dela e exclui janelas da área de trabalho/shell. O Mac compara a janela do aplicativo em primeiro plano com os limites da tela. Por isso, um aplicativo com aparência de tela cheia pode se comportar diferente nas duas plataformas.

Desative “Pausar gestos em tela cheia” nas preferências para mudar o ajuste global. Em “Em tela cheia”, cada aplicativo oferece “Seguir ajuste global”, “Manter ativos” e “Pausar gestos”; a escolha explícita do aplicativo tem prioridade. Para usar gestos globais no Chrome em tela cheia, adicione o Chrome, mantenha a herança dos gestos globais e escolha “Manter ativos”. Não é preciso criar gestos próprios. A pausa manual e as exclusões continuam valendo.

Para um jogo em janela ou área de trabalho remota, adicione uma exclusão explícita. Para pausar em todo lugar, use o [comando da área de notificação/barra de menus ou o atalho de pausa](getting-started.md#pausar).

O guia de primeiros passos mostra os aplicativos detectados que normalmente precisam do próprio arrastar do mouse. As entradas marcadas são desativadas quando você o conclui. Revise-as antes de continuar. A configuração integrada do Mac também inclui escopos desativados para seus aplicativos de exclusão sugeridos; eles não têm vínculos próprios. Você pode alterar as exclusões depois pela barra lateral.

Próximo: [biblioteca de gestos](gesture-library.md), [ações](actions.md), [backup da configuração](config.md), [solução de problemas](troubleshooting.md).
