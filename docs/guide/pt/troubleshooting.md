---
title: "Solução de problemas"
description: "Corrija gestos de mouse que não disparam, permissões do macOS, atalhos ausentes, entrada de texto com IME e erros de Lua."
order: 12
---

# Solução de problemas

## Os gestos não disparam

Primeiro teste um gesto integrado simples em uma janela de navegador normal, fora de tela cheia.

- Verifique o menu da área de notificação/barra de menus. No Windows, selecione **Retomar gestos** se oferecido; no Mac, clique em **Pausar gestos** de novo se estiver marcado. Confira **Preferências → Atalho de pausar / retomar** se um atalho pausou o motor.
- Saia da tela cheia. A supressão automática de tela cheia pode parar tanto os gestos de mouse quanto as ações de borda, mesmo com a pausa manual desligada.
- Selecione o aplicativo nas configurações e confira **Desativar gestos neste aplicativo**. Verifique também o interruptor **Ativado** / **Desativado** do vínculo individual, **Usar também os gestos globais**, o botão de disparo e quaisquer etapas extras.
- Mova-se logo após pressionar o botão e desenhe um traçado mais longo. Ajuste o **Tempo parado até tratar como clique** e a **Sensibilidade do traçado** nas Preferências.
- Se faltar retorno visual, confira **Mostrar o rastro** e **Mostrar o nome do gesto ao desenhar**. Um rastro oculto, por si só, não significa que o reconhecimento esteja desligado.

No Windows, a entrada usa `SendInput`. Uma instância iniciada normalmente pode não conseguir enviar teclas/cliques para um aplicativo elevado por causa da UIPI do Windows. Teste primeiro em um aplicativo sem elevação; se precisar controlar uma janela de administrador, execute o Mousetroke com a elevação correspondente. Isso não permite o envio para a área de trabalho segura.

No Mac, confira a **Acessibilidade** para a cópia instalada. Se a configuração inicial ainda pedir **Monitoramento de Entrada**, conceda também. As mudanças de permissão podem levar alguns segundos para serem detectadas. Use **Reiniciar Mousetroke** quando oferecido, se a entrada ainda não iniciar. A Gravação de Tela não é necessária. Mover ou substituir uma cópia pelo navegador pode exigir nova aprovação de primeira abertura.

## SmartScreen ou Gatekeeper bloqueia a primeira abertura

Windows: obtenha o executável oficial na página de versões, verifique a soma de verificação e use **Mais informações → Executar assim mesmo**. Mac: instale em Aplicativos e use **Ajustes do Sistema → Privacidade e Segurança → Abrir Mesmo Assim**; versões mais antigas do macOS também suportam **clique direito → Abrir**. Veja a [instalação](getting-started.md) para a sequência completa.

## Um aplicativo não recebe as teclas

Experimente o atalho no teclado físico. Verifique se o aplicativo está ativo, ou ative **Ativar antes a janela sob o cursor**. Confirme a preferência de alvo do gesto e use a [sintaxe correta](key-sequences.md): `^` é Control, enquanto `#` é Command no Mac. Separe etapas dependentes com um breve `{SLEEP 100}` se o aplicativo precisar de tempo para abrir um menu ou diálogo.

As sequências de teclas no Mac ignoram tokens de mouse/roda, tokens de teclas de mídia/volume, Insert, Print Screen, Break e F21–F24. Use **Mídia e sistema** para ações de mídia/volume. As opções sem suporte estão marcadas no editor; inspecione também o log.

Ctrl+Alt+Del é uma Sequência de Atenção Segura do Windows e não pode ser enviado como entrada sintética comum. Use Ctrl+Shift+Esc para o Gerenciador de Tarefas, Win+L para bloquear, ou `shutdown /l` para encerrar a sessão. Os detalhes estão na [sintaxe de envio de teclas](key-sequences.md#por-que-ctrlaltdel-não-pode-ser-enviado).

## IME ou saída de texto incorreta

Os caracteres comuns do envio de teclas representam posições do teclado americano (US), não texto Unicode arbitrário. Letras maiúsculas sozinhas não adicionam Shift. O `ms.keys(text, true)` do Lua emite texto Unicode, mas ainda interpreta caracteres de sintaxe como `^` e `{`.

Para texto com muita pontuação, texto não latino ou aplicativos com tratamento de IME incomum, use a [receita de colar e restaurar pela área de transferência](lua-examples.md#colar-e-restaurar-o-texto-da-área-de-transferência). A restauração com estas APIs preserva somente texto, não imagens nem outros formatos da área de transferência. Alguns aplicativos precisam de um atraso maior antes da restauração da área de transferência.

## Erros de Lua

Confirme que o código define `function ms_action(...) ... end`. O tempo de execução grava erros de carregamento e de execução no [log do tempo de execução](config.md#locais-dos-arquivos). Abra-o por **Sobre → Abrir log**. Use `ms.log` para valores e `ms.show_toast` para um ponto de verificação breve na tela. Uma função sem suporte no Mac pode apenas registrar um aviso e devolver `nil`; não trate um valor ausente como sucesso.

As chamadas de descoberta/estado de janela do Mac exigem a permissão de Acessibilidade; as consultas de estado de teclas devolvem falso sem ela. Confira primeiro essa permissão se faltarem resultados. Sempre no topo entre aplicativos e brilho numérico continuam sem suporte; as ações integradas de teclas de brilho não informam uma porcentagem. Nas duas plataformas, `ms.get_brightness` / `ms.set_brightness` devolvem nil na falha.

O limite de alocação do Lua é 8 MiB. Não há tempo limite de execução de script; evite loops infinitos e veja [Noções básicas de Lua](lua-basics.md#tempo-de-execução-e-limites) para o comportamento de bloqueio.

## Área de trabalho remota e jogos

Área de trabalho remota, máquinas virtuais e aplicativos 3D/jogos podem precisar dos mesmos botões do mouse para arrastar ou tratar a entrada sintética de forma diferente. Adicione uma [exclusão por aplicativo](per-app.md) ou pause enquanto os usa. Jogos em janela podem precisar de uma exclusão explícita porque a detecção de tela cheia não se aplica. Teste atalhos em um aplicativo local comum para separar um problema de sequência de teclas do tratamento de entrada do aplicativo.

## Relatar um bug

Use **Sobre → Comentários e sugestões → Abrir** ou as [Issues do GitHub](https://github.com/eiaao/mousetroke/issues). Inclua a versão do aplicativo, a versão do sistema operacional, o aplicativo afetado, o gesto e a ação, o resultado esperado e os passos para reproduzir. Anexe o log relevante de **Sobre → Abrir log** e uma configuração exportada mínima, se útil. Revise os anexos quanto a texto privado, títulos de janela, caminhos ou argumentos de comando antes de compartilhá-los.
