---
title: "Biblioteca de ações"
description: "Escolha ações prontas, defina seus parâmetros e consulte o catálogo completo para Windows e Mac."
order: 5.5
---

# Biblioteca de ações

As ações predefinidas descrevem o que você quer fazer, como fechar uma aba ou centralizar uma janela. O Mousetroke guarda o ID e os parâmetros da ação e escolhe a implementação para Windows ou Mac ao executá-la. Os atalhos de navegadores e editores continuam dependendo do suporte do aplicativo que os recebe.

## Escolher uma ação

1. Crie ou selecione um gesto e clique em **Escolher ação**, ou em **Trocar** em uma ação existente.
2. Pesquise por nome, descrição, apelido ou escrita do atalho, como `ctrl+w` no Windows ou `cmd+w` / `⌘w` no Mac. A pesquisa abrange todas as categorias. Sem pesquisar, explore **Frequentes**, **Navegador**, **Edição**, **Texto**, **Janelas**, **Área de trabalho e sistema**, **Mídia**, **Abrir** ou **Mousetroke**.
3. Escolha um resultado. Ações com parâmetros abrem um formulário: digite texto, escolha um formato de data ou serviço de pesquisa, ou informe uma URL, um arquivo, uma pasta ou um aplicativo. Clique em **Usar esta ação** para confirmar. Ações indisponíveis ficam desativadas e informam a limitação da plataforma ou a implementação prevista para a segunda fase.
4. Experimente o gesto fora das configurações. Se o nome do gesto estiver vazio, ele acompanha o nome da ação selecionada. Os itens de menu usam o mesmo seletor.

Ao abrir o seletor, o campo de pesquisa recebe o foco. Use as setas para cima e para baixo para percorrer os resultados disponíveis e Enter para selecionar. Esc fecha primeiro o formulário de parâmetros e depois o seletor. Em campos de texto com várias linhas, Ctrl+Enter ou Command+Enter confirma o formulário.

**Converter em ação avançada** substitui uma ação predefinida pelas teclas, operação de janela, URL, aplicativo, comando ou tecla de sistema editáveis da plataforma atual. A confirmação explica que a conversão é irreversível: o resultado deixa de acompanhar os padrões de plataforma ou idioma da ação predefinida. Ações nativas de disposição de janelas, energia, texto literal, data e hora, cópia sem formatação, pausa e configurações mantêm o formato predefinido; seus cartões explicam por que a conversão não está disponível. A categoria **Avançadas**, no fim da lista, oferece envio de teclas, comandos, Lua e menus. A [referência de ações avançadas](actions.md) também descreve os editores existentes.

## Texto, janelas e energia

- **Inserir texto** escreve o texto literalmente, incluindo espaços, quebras de linha, caracteres chineses, emojis e caracteres como `^+#{ENTER}`. **Colar e restaurar área de transferência** é o método padrão: usa temporariamente a área de transferência e restaura seu **texto** anterior após uma breve espera. Se ela contiver imagens, HTML, listas de arquivos ou outros dados que não sejam texto, ou não puder ser verificada, o texto será digitado diretamente, sem alterar a área de transferência. `paste_delay_ms` só aparece com `paste`: a interface permite 50–5000 ms, com padrão de 150 ms. **Digitar caracteres** usa entrada Unicode sem a área de transferência. Os dois métodos enviam a entrada ao aplicativo que recebe o teclado.
- **Inserir data e hora** usa o relógio local do sistema e o método de saída escolhido. Os formatos incluem data ISO, data chinesa, data dos EUA, hora e data com hora. Um padrão personalizado aceita `YYYY`, `MM`, `DD`, `HH`, `mm` e `ss`, diferenciando maiúsculas e minúsculas. Os demais caracteres são literais; aspas simples protegem texto literal e `''` insere uma aspa. Por exemplo, `YYYY-MM-DD HH:mm:ss` produz uma marca de data e hora. Aspas não fechadas são rejeitadas.
- **Copiar seleção sem formatação** envia Ctrl+C / Command+C, espera até 500 ms por uma mudança na versão da área de transferência e então substitui o conteúdo copiado apenas pelo texto. Se a cópia não terminar ou não contiver texto, informa a falha e mantém o conteúdo da área de transferência. Selecione texto e mantenha o foco nesse aplicativo antes de usar a ação.
- As ações de disposição usam o mesmo alvo do gesto que as demais ações de janela, seguindo **Preferências → Os gestos agem sobre**. O Windows usa o posicionamento nativo de janelas; o Mac usa Acessibilidade e a área útil da tela. Os tamanhos mínimos de janela e as restrições dos aplicativos podem impedir uma divisão exata em meia tela. No Mac, saia primeiro da tela cheia nativa. **Mover para a próxima tela** percorre as telas em uma ordem estável, mantém a posição relativa e a maximização, e não faz nada se houver apenas uma tela.
- Depois de **Desligar tela**, uma entrada pode reativar as telas. **Suspender** solicita a suspensão normal do sistema, que pode ser negada pela conta ou pelas políticas do sistema. Se uma ação nativa falhar, uma linha será escrita no log e uma mensagem breve aparecerá na tela. **Pausar / retomar gestos** usa o mesmo estado de pausa da bandeja e do atalho; use um desses meios para retomar os gestos pausados. **Abrir configurações** abre a janela habitual.

## Catálogo

As tabelas abaixo são geradas a partir do registro e incluem todas as 87 entradas. ✓ indica que a ação está implementada naquela plataforma; a disponibilidade ainda depende do aplicativo, do hardware e das permissões necessárias. A consulta rápida de gestos está prevista para a segunda fase.

<!-- action-library:generated:start -->

### Navegador

| Ação / ID | Função e onde usar | Parâmetros | Windows | Mac |
|---|---|---|---|---|
| Voltar<br>`browser.back` | Funciona em navegadores web. | — | ✓ | ✓ |
| Avançar<br>`browser.forward` | Funciona em navegadores web. | — | ✓ | ✓ |
| Atualizar<br>`browser.reload` | Funciona em navegadores web. | — | ✓ | ✓ |
| Recarregar sem cache<br>`browser.hard_reload` | Funciona em navegadores web. | — | ✓ | ✓ |
| Nova aba<br>`browser.new_tab` | Navegadores, editores e outros apps compatíveis com este atalho. | — | ✓ | ✓ |
| Fechar aba<br>`browser.close_tab` | Navegadores, editores e outros apps compatíveis com este atalho. | — | ✓ | ✓ |
| Reabrir aba fechada<br>`browser.reopen_tab` | Funciona em navegadores web. | — | ✓ | ✓ |
| Aba anterior<br>`browser.prev_tab` | Navegadores, editores e outros apps compatíveis com este atalho. | — | ✓ | ✓ |
| Aba seguinte<br>`browser.next_tab` | Navegadores, editores e outros apps compatíveis com este atalho. | — | ✓ | ✓ |
| Nova janela<br>`browser.new_window` | Navegadores, editores e outros apps compatíveis com este atalho. | — | ✓ | ✓ |
| Nova janela privativa<br>`browser.private_window` | Chrome, Edge e Safari; o Firefox usa outro atalho. | — | ✓ | ✓ |
| Focar barra de endereços<br>`browser.address_bar` | Funciona em navegadores web. | — | ✓ | ✓ |
| Localizar na página<br>`browser.find` | Navegadores, editores e outros apps compatíveis com este atalho. | — | ✓ | ✓ |
| Adicionar página aos favoritos<br>`browser.bookmark` | Funciona em navegadores web. | — | ✓ | ✓ |
| Ampliar<br>`view.zoom_in` | Navegadores, editores e outros apps compatíveis com este atalho. | — | ✓ | ✓ |
| Reduzir<br>`view.zoom_out` | Navegadores, editores e outros apps compatíveis com este atalho. | — | ✓ | ✓ |
| Redefinir zoom<br>`view.zoom_reset` | Navegadores, editores e outros apps compatíveis com este atalho. | — | ✓ | ✓ |

### Edição

| Ação / ID | Função e onde usar | Parâmetros | Windows | Mac |
|---|---|---|---|---|
| Ir ao topo<br>`view.top` | Navegadores, editores e outros apps compatíveis com este atalho. | — | ✓ | ✓ |
| Ir ao fim<br>`view.bottom` | Navegadores, editores e outros apps compatíveis com este atalho. | — | ✓ | ✓ |
| Subir uma página<br>`view.page_up` | Navegadores, editores e outros apps compatíveis com este atalho. | — | ✓ | ✓ |
| Descer uma página<br>`view.page_down` | Navegadores, editores e outros apps compatíveis com este atalho. | — | ✓ | ✓ |
| Copiar<br>`edit.copy` | Navegadores, editores e outros apps compatíveis com este atalho. | — | ✓ | ✓ |
| Recortar<br>`edit.cut` | Navegadores, editores e outros apps compatíveis com este atalho. | — | ✓ | ✓ |
| Colar<br>`edit.paste` | Navegadores, editores e outros apps compatíveis com este atalho. | — | ✓ | ✓ |
| Colar e Enter<br>`edit.paste_enter` | Navegadores, editores e outros apps compatíveis com este atalho. | — | ✓ | ✓ |
| Colar como texto sem formatação<br>`edit.paste_plain` | Apps com colagem sem formatação; o atalho varia por app. | — | ✓ | ✓ |
| Selecionar tudo<br>`edit.select_all` | Navegadores, editores e outros apps compatíveis com este atalho. | — | ✓ | ✓ |
| Desfazer<br>`edit.undo` | Navegadores, editores e outros apps compatíveis com este atalho. | — | ✓ | ✓ |
| Refazer<br>`edit.redo` | Navegadores, editores e outros apps compatíveis com este atalho. | — | ✓ | ✓ |
| Salvar<br>`edit.save` | Navegadores, editores e outros apps compatíveis com este atalho. | — | ✓ | ✓ |
| Enter<br>`key.enter` | Navegadores, editores e outros apps compatíveis com este atalho. | — | ✓ | ✓ |
| Backspace<br>`key.backspace` | Navegadores, editores e outros apps compatíveis com este atalho. | — | ✓ | ✓ |
| Delete<br>`key.delete` | Navegadores, editores e outros apps compatíveis com este atalho. | — | ✓ | ✓ |
| Cancelar (Esc)<br>`key.escape` | Navegadores, editores e outros apps compatíveis com este atalho. | — | ✓ | ✓ |
| Tela cheia<br>`view.fullscreen` | Navegadores, editores e outros apps compatíveis com este atalho. | — | ✓ | ✓ |

### Texto

| Ação / ID | Função e onde usar | Parâmetros | Windows | Mac |
|---|---|---|---|---|
| Inserir texto<br>`text.type` | Campos de texto; por padrão cola e restaura a área de transferência. | Texto (`text`): obrigatório<br>Método de entrada (`method`): Colar e restaurar área de transferência (`paste`) / Digitar caracteres (`type`); padrão: Colar e restaurar área de transferência<br>Espera após colar (ms) (`paste_delay_ms`): padrão: `150`; somente se Método de entrada for Colar e restaurar área de transferência | ✓ | ✓ |
| Inserir data e hora<br>`text.datetime` | Campos de texto; usa data e hora locais. | Formato de data e hora (`format`): 2026-10-24 (`date_iso`) / 2026年10月24日 (`date_local`) / 10/24/2026 (`date_us`) / 14:30 (`time`) / 2026-10-24 14:30 (`datetime`) / Personalizado (`custom`); padrão: 2026-10-24<br>Formato personalizado (`custom_format`): padrão: `YYYY-MM-DD`; somente se Formato de data e hora for Personalizado<br>Método de entrada (`method`): Colar e restaurar área de transferência (`paste`) / Digitar caracteres (`type`); padrão: Colar e restaurar área de transferência<br>Espera após colar (ms) (`paste_delay_ms`): padrão: `150`; somente se Método de entrada for Colar e restaurar área de transferência | ✓ | ✓ |
| Pesquisar na web<br>`text.search` | Usa o navegador padrão; selecione texto no app atual primeiro. | Mecanismo de busca (`engine`): Bing (`bing`) / Google (`google`) / Baidu (`baidu`) / DuckDuckGo (`duckduckgo`) / Personalizado (`custom`); Baidu na interface em chinês simplificado; Google nos demais idiomas<br>URL personalizada (`url`): padrão: `https://www.google.com/search?q=%MS_SELECTED_TEXT%`; somente se Mecanismo de busca for Personalizado | ✓ | ✓ |
| Traduzir texto selecionado<br>`text.translate` | Usa o navegador padrão; selecione texto no app atual primeiro. | Serviço de tradução (`service`): Google Tradutor (`google`) / DeepL (`deepl`) / Tradutor do Bing (`bing`); padrão: Google Tradutor<br>Idioma de destino (`target_language`): Seguir idioma da interface (`ui`) / 简体中文 (`zh-CN`) / English (`en-US`) / español (`es-ES`) / Français (`fr-FR`) / Deutsch (`de-DE`) / 日本語 (`ja-JP`) / Português (Brasil) (`pt-BR`) / Русский (`ru-RU`) / Italiano (`it-IT`) / 한국어 (`ko-KR`); padrão: Seguir idioma da interface | ✓ | ✓ |
| Abrir URL selecionada<br>`text.open_selected_url` | Usa o navegador padrão; selecione texto no app atual primeiro. | — | ✓ | ✓ |
| Copiar seleção sem formatação<br>`text.copy_plain` | Apps que copiam texto; remove a formatação da área de transferência. | — | ✓ | ✓ |

### Janelas

| Ação / ID | Função e onde usar | Parâmetros | Windows | Mac |
|---|---|---|---|---|
| Minimizar<br>`window.minimize` | Aplica-se à janela alvo do gesto. | — | ✓ | ✓<br>Requer permissão de Acessibilidade. |
| Maximizar / restaurar<br>`window.toggle_maximize` | Aplica-se à janela alvo do gesto. | — | ✓ | ✓<br>Requer permissão de Acessibilidade. |
| Fechar janela<br>`window.close` | Aplica-se à janela alvo do gesto. | — | ✓ | ✓<br>Requer permissão de Acessibilidade. |
| Sair do aplicativo<br>`app.quit` | Aplica-se à janela alvo do gesto. | — | ✓ | ✓ |
| Encerrar processo<br>`app.kill` | Encerra o app alvo à força; dados não salvos serão perdidos. | — | ✓ | ✓ |
| Sempre visível<br>`window.toggle_topmost` | Alternar sempre no topo para a janela alvo. | — | ✓ | Esta ação só é compatível com Windows. |
| Ajustar à metade esquerda<br>`window.snap_left` | Aplica-se à janela alvo do gesto. | — | ✓ | ✓<br>Requer permissão de Acessibilidade. |
| Ajustar à metade direita<br>`window.snap_right` | Aplica-se à janela alvo do gesto. | — | ✓ | ✓<br>Requer permissão de Acessibilidade. |
| Centralizar janela<br>`window.center` | Aplica-se à janela alvo do gesto. | — | ✓ | ✓<br>Requer permissão de Acessibilidade. |
| Mover para a próxima tela<br>`window.next_monitor` | Aplica-se à janela alvo do gesto. | — | ✓ | ✓<br>Requer permissão de Acessibilidade. |
| Ocultar aplicativo atual<br>`app.hide` | Ocultar o aplicativo atual. | — | Esta ação só é compatível com macOS. | ✓ |

### Área de trabalho e sistema

| Ação / ID | Função e onde usar | Parâmetros | Windows | Mac |
|---|---|---|---|---|
| Trocar de aplicativo<br>`desktop.switch_app` | Ação do sistema; parte do comportamento depende dos atalhos configurados. | — | ✓ | ✓ |
| Visão de Tarefas / Mission Control<br>`desktop.task_view` | Ação do sistema; parte do comportamento depende dos atalhos configurados. | — | ✓ | ✓ |
| Mostrar a área de trabalho<br>`desktop.show` | Ação do sistema; parte do comportamento depende dos atalhos configurados. | — | ✓ | ✓ |
| Área de trabalho virtual anterior<br>`desktop.prev` | Ação do sistema; parte do comportamento depende dos atalhos configurados. | — | ✓ | ✓ |
| Área de trabalho virtual seguinte<br>`desktop.next` | Ação do sistema; parte do comportamento depende dos atalhos configurados. | — | ✓ | ✓ |
| Menu Iniciar / Spotlight<br>`desktop.launcher` | Ação do sistema; parte do comportamento depende dos atalhos configurados. | — | ✓ | ✓ |
| Abrir gerenciador de arquivos<br>`desktop.files` | Ação do sistema; parte do comportamento depende dos atalhos configurados. | — | ✓ | ✓ |
| Recorte<br>`capture.region` | Ação do sistema; parte do comportamento depende dos atalhos configurados. | — | ✓ | ✓ |
| Capturar e salvar tela<br>`capture.screen` | Ação do sistema; parte do comportamento depende dos atalhos configurados. | — | ✓ | ✓ |
| Ferramentas de captura e gravação<br>`capture.tool` | Ação do sistema; parte do comportamento depende dos atalhos configurados. | — | ✓ | ✓ |
| Histórico da área de transferência<br>`desktop.clipboard_history` | Abrir o histórico da área de transferência. | — | ✓ | Esta ação só é compatível com Windows. |
| Emojis e símbolos<br>`desktop.emoji` | Ação do sistema; parte do comportamento depende dos atalhos configurados. | — | ✓ | ✓ |
| Central de notificações<br>`desktop.notifications` | Abrir a Central de Notificações. | — | ✓ | Esta ação só é compatível com Windows. |
| Bloquear tela<br>`system.lock` | Ação do sistema; parte do comportamento depende dos atalhos configurados. | — | ✓ | ✓ |
| Gerenciador de Tarefas / Monitor de Atividade<br>`system.task_manager` | Ação do sistema; parte do comportamento depende dos atalhos configurados. | — | ✓ | ✓ |
| Janela Forçar Encerrar<br>`system.force_quit_dialog` | Abrir o diálogo Forçar Encerrar. | — | Esta ação só é compatível com macOS. | ✓ |
| Configurações do sistema<br>`system.settings` | Ação do sistema; parte do comportamento depende dos atalhos configurados. | — | ✓ | ✓ |
| Calculadora<br>`system.calculator` | Ação do sistema; parte do comportamento depende dos atalhos configurados. | — | ✓ | ✓ |
| Desligar tela<br>`system.display_off` | Ação do sistema; parte do comportamento depende dos atalhos configurados. | — | ✓ | ✓ |
| Suspender<br>`system.sleep` | Ação do sistema; parte do comportamento depende dos atalhos configurados. | — | ✓ | ✓ |

### Mídia

| Ação / ID | Função e onde usar | Parâmetros | Windows | Mac |
|---|---|---|---|---|
| Reproduzir / pausar<br>`media.play_pause` | Teclas de mídia do sistema; exige suporte do player. | — | ✓ | ✓ |
| Faixa anterior<br>`media.previous` | Teclas de mídia do sistema; exige suporte do player. | — | ✓ | ✓ |
| Faixa seguinte<br>`media.next` | Teclas de mídia do sistema; exige suporte do player. | — | ✓ | ✓ |
| Aumentar volume<br>`media.volume_up` | Teclas de mídia do sistema; exige suporte do player. | — | ✓ | ✓ |
| Diminuir volume<br>`media.volume_down` | Teclas de mídia do sistema; exige suporte do player. | — | ✓ | ✓ |
| Sem som<br>`media.mute` | Teclas de mídia do sistema; exige suporte do player. | — | ✓ | ✓ |
| Aumentar brilho<br>`display.brightness_up` | Brilho do sistema; depende da tela e do driver. | — | ✓ | ✓ |
| Diminuir brilho<br>`display.brightness_down` | Brilho do sistema; depende da tela e do driver. | — | ✓ | ✓ |

### Abrir

| Ação / ID | Função e onde usar | Parâmetros | Windows | Mac |
|---|---|---|---|---|
| Abrir site<br>`open.website` | Abre a URL indicada no navegador padrão. | URL (`url`): padrão: `https://mousetroke.com/` | ✓ | ✓ |
| Abrir pasta<br>`open.folder` | Abre o caminho indicado com o app padrão do sistema. | Caminho (`path`): obrigatório | ✓ | ✓ |
| Abrir arquivo<br>`open.file` | Abre o caminho indicado com o app padrão do sistema. | Caminho (`path`): obrigatório | ✓ | ✓ |
| Abrir ou alternar para aplicativo<br>`open.app` | Abre o app ou ativa sua janela existente. | Aplicativo (`app`): obrigatório | ✓ | ✓ |

### Mousetroke

| Ação / ID | Função e onde usar | Parâmetros | Windows | Mac |
|---|---|---|---|---|
| Pausar / retomar gestos<br>`mousetroke.toggle_pause` | Pausa ou retoma o reconhecimento de gestos no Mousetroke. | — | ✓ | ✓ |
| Abrir configurações<br>`mousetroke.settings` | Abre as configurações do Mousetroke. | — | ✓ | ✓ |
| Mostrar lista de gestos<br>`mousetroke.cheatsheet` | Lista gestos do app atual; previsto para a fase 2. | — | Esta ação está prevista para a fase 2. | Esta ação está prevista para a fase 2. |

<!-- action-library:generated:end -->

Depois de alterar o registro ou as traduções, defina `UPDATE_ACTION_DOCS=1` e execute `cargo test -p mousetroke-core --test action_docs` para atualizar os catálogos dos dez idiomas. Execute o mesmo comando sem essa variável para verificar se as tabelas salvas no repositório estão atualizadas.
