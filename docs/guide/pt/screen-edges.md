---
title: "Cantos e bordas da tela"
description: "Configure cantos de acesso rápido, rolagem na borda para volume e brilho, cliques na borda e gestos de esfregar no Windows e no Mac."
order: 4
---

# Cantos e bordas da tela

## Experimente os padrões

Em **Todos os aplicativos → Bordas da tela**, todos os vínculos de canto ou borda são editáveis.

| Gatilho | Padrão no Windows | Padrão no Mac |
|---|---|---|
| Empurrar para o canto inferior esquerdo | Menu Iniciar | Spotlight |
| Empurrar para o canto inferior direito | Mostrar a área de trabalho | Mostrar a Mesa (F11) |
| Empurrar para o canto superior esquerdo | Trocar de aplicativo (Alt+Tab) | Mission Control (Control+Up) |
| Empurrar para o canto superior direito | Visão de tarefas (Win+Tab) | Vínculo inicialmente desativado |
| Rolar na borda superior | Aumentar / diminuir volume | Aumentar / diminuir volume |
| Clique com o botão do meio na borda superior | Sem som | Sem som |
| Clique com o botão direito na borda superior | Reproduzir / pausar | Reproduzir / pausar |
| Rolar na borda inferior | Aumentar / diminuir brilho | Vínculos de brilho inicialmente desativados |
| Esfregar a borda direita para cima/baixo duas vezes | Vínculo do Gerenciador de Tarefas; ative antes Executar assim que reconhecer | Monitor de Atividade |

As ações de canto não precisam de botão: empurre o cursor contra o canto. A rolagem e os cliques na borda exigem o cursor encostado na borda escolhida. Esfregar significa mover ao longo da borda nas direções gravadas, em vez de girar a roda.

## Configurar um gatilho de borda

1. Escolha **Bordas da tela → Nova borda** no escopo do aplicativo desejado ou em **Todos os aplicativos**.
2. Em **Como começa**, escolha **Borda da tela** e selecione um lado ou canto no diagrama.
3. Deixe **Depois** vazio para uma ação de canto, ou use **+ Adicionar etapa** para adicionar rolar para cima/baixo, um clique do mouse ou um traçado ao longo da borda.
4. Escolha uma [ação](actions.md). Para volume, escolha **Mídia e sistema → Aumentar volume** ou **Diminuir volume**. Ative **Executar assim que reconhecer** para rolagem repetida ou para esfregar sem botão.

Para esfregar, adicione **Desenhar um traçado** e grave as direções alternadas. Um exemplo útil é `↑↓↑↓` contra a borda direita. Experimente com uma ação inofensiva, como um toast de Lua, antes de atribuir algo disruptivo.

## Brilho e diferenças entre plataformas

O **Aumentar brilho** / **Diminuir brilho** do Windows começa pelo monitor sob o cursor, ajusta em 10 pontos percentuais e mostra o resultado. Ele tenta DDC/CI e depois WMI. Monitores externos precisam suportar e permitir DDC/CI. O fallback WMI usa o primeiro dispositivo de brilho do WMI, normalmente um painel integrado; ele não corresponde ao monitor do cursor. Se ambas as rotas falharem, aparece uma mensagem na tela. Veja [receitas de Lua](lua-examples.md#brilho-sob-o-cursor) para um passo menor.

O **Mídia e sistema → Aumentar brilho** / **Diminuir brilho** do Mac envia eventos das teclas de brilho do sistema para a tela integrada. Ele não seleciona a tela sob o cursor e não suporta monitores DDC/CI externos. Os padrões da borda inferior vêm desativados. As APIs de Lua para brilho também não têm suporte no Mac.

No Mac, o Mousetroke avisa se um canto de acesso rápido do sistema coincidir com um canto configurado. Altere uma das atribuições para evitar ações concorrentes. Atalhos do sistema como F11 ou Control+Up dependem das configurações de atalhos do macOS.

Exclusões de aplicativos e a detecção de tela cheia também suprimem os gatilhos de borda. Veja as [regras por aplicativo](per-app.md) quando uma borda funciona em um aplicativo e não em outro.
