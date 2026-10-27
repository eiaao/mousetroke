---
title: "Primeiros passos"
description: "Instale o Mousetroke no Windows ou no Mac, desenhe seu primeiro gesto de mouse, abra as configurações e pause os gestos."
order: 2
---

# Primeiros passos

## Instalar e abrir

No Windows, baixe `mousetroke.exe` da [versão mais recente](https://github.com/eiaao/mousetroke/releases/latest). É um executável portátil: salve-o em um local onde pretende mantê-lo e clique duas vezes para executar. Compare o SHA-256 do download com o `SHA256SUMS.txt` da página dessa versão.

São suportados Windows 10 1809 ou posterior e Windows 11, 64 bits. As configurações usam o Microsoft Edge WebView2, incluído no Windows 11. O executável ainda não tem assinatura de código. Se o SmartScreen bloquear a primeira execução, escolha **Mais informações → Executar assim mesmo** depois de conferir que você baixou a versão oficial.

No Mac, baixe `Mousetroke.dmg`, abra-o e arraste **Mousetroke** para **Aplicativos**. O aplicativo universal suporta Apple silicon e Intel no macOS 11 ou posterior. Se for aberto dentro do DMG, o aplicativo oferece **Mover para Aplicativos** e reabre a cópia instalada.

O beta do Mac é autoassinado e não é notarizado. Se for bloqueado, use **Ajustes do Sistema → Privacidade e Segurança → Abrir Mesmo Assim**; em versões mais antigas do macOS, use **clique direito → Abrir**. Na configuração inicial, clique em **Permitir…** e ative o Mousetroke em **Acessibilidade**. Aguarde **Tudo pronto — você já pode usar gestos**. A permissão de Gravação de Tela não é necessária. A configuração inicial oferece o **Monitoramento de Entrada** se a escuta ainda falhar em sistemas que precisam dele; ative-o quando solicitado.

## Abrir as configurações e concluir o guia

No Windows, clique com o botão esquerdo no ícone da área de notificação para abrir as configurações. Clique com o botão direito para **Configurações…**, **Pausar gestos** / **Retomar gestos**, **Iniciar com o Windows**, **Abrir pasta de configuração** e **Sair**. Um item de nova versão também pode aparecer quando houver uma atualização disponível.

No Mac, use o ícone da barra de menus para abrir **Configurações…**, pausar/retomar ou **Sair**. Itens de permissão e de atualização aparecem quando relevante. **Sobre o Mousetroke** abre a página Sobre.

O guia de primeiros passos explica o desenho, as bordas da tela, as exclusões de aplicativos e a pausa. Revise os aplicativos marcados antes de concluir: eles ficarão com os gestos desativados. Você pode reabri-lo com **Sobre → Ver o guia de primeiros passos**. As alterações nas configurações são salvas automaticamente; um traçado incompleto precisa ser gravado até o fim antes de poder salvar.

## Desenhe seu primeiro gesto

1. Abra uma página do navegador com histórico de navegação.
2. Segure o botão direito do mouse, mova para a esquerda para desenhar `←` e solte. Isso volta uma página.
3. Experimente `↓→` em uma aba descartável para fechá-la, ou `↙` para minimizar a janela.

As ações integradas seguem a plataforma: os atalhos do navegador usam Ctrl no Windows e Command no Mac, quando apropriado. A lista completa e editável fica em **Todos os aplicativos → Gestos de mouse**. Alguns itens integrados vêm desativados, especialmente ações indisponíveis no Mac.

Um clique direito simples continua abrindo o menu de contexto do aplicativo. Segurar o mouse parado por tempo suficiente também deixa o botão passar direto para cliques ou arrastes normais; ajuste esse tempo em [Preferências](gestures.md#reconhecimento-e-aparência).

## Criar um gesto

Abra **Todos os aplicativos → Gestos de mouse → Novo gesto**, escolha o gatilho e grave o desenho. Clique em **Escolher ação** para pesquisar ou explorar a [biblioteca de ações](action-library.md), selecione uma ação predefinida e preencha seus parâmetros. As alterações são salvas automaticamente quando o gesto é válido; experimente fora das configurações. Veja [Gestos de mouse](gestures.md) para os detalhes de gravação.

## Pausar

No Windows, escolha **Pausar gestos** e depois **Retomar gestos** para restaurá-los. No Mac, **Pausar gestos** mantém sempre o mesmo nome: a marcação indica pausa manual, e clicar novamente retoma. O atalho padrão de pausa/retomada é **Win+Esc** no Windows e **Control+Option+Command+P** no Mac. Altere-o em **Preferências → Atalho de pausar / retomar**.

Jogos, vídeos e apresentações em tela cheia suprimem os gestos automaticamente enquanto a janela em primeiro plano for detectada como tela cheia. Veja as [regras por aplicativo](per-app.md) para exclusões e a [solução de problemas](troubleshooting.md) se nada disparar.
