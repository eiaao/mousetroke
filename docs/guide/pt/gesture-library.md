---
title: "Biblioteca de gestos"
description: "Defina um gesto uma vez e use-o em vários aplicativos com ações diferentes: organize e importe seus gestos, atribua-os a outros aplicativos, edite gestos compartilhados e notas de compatibilidade."
order: 3.5
---

# Biblioteca de gestos

A biblioteca de gestos separa como você desenha do que acontece. Defina um gesto uma vez e dê a ele uma ação diferente em cada aplicativo. Regravar um gesto compartilhado atualiza todos os lugares que o usam, e um gesto de forma mantém um único conjunto de amostras.

## Navegar pela biblioteca

Clique em **Gestos** na barra lateral. A lista é dividida em **Gestos de mouse** e **Bordas da tela**. Filtre pelo botão de acionamento, use **Não usados** para encontrar gestos que nada usa ou pesquise por nome, traçado ou uso. Cada cartão mostra em quantos lugares o gesto é usado.

Clique em um cartão para abrir os detalhes. No topo você pode redesenhar; **Usado em** lista cada aplicativo, nome de gesto e ação que o usa, e clicar em uma linha leva até lá. Em uma instalação nova, os gestos integrados já estão na biblioteca. Ao atualizar de uma versão anterior, seus gestos continuam espalhados pelos aplicativos: organize-os primeiro como descrito abaixo.

## Organizar e importar

Com a biblioteca vazia, clique em **Organizar e importar gestos…**; depois, **Organizar e importar…** fica sempre no canto superior direito. Primeiro escolha o alcance:

- **Só gestos comuns (recomendado)**: adiciona os gestos de **Todos os aplicativos** e vincula os idênticos ou parecidos de outros aplicativos. Gestos configurados só para um aplicativo continuam específicos.
- **Todos os gestos**: adiciona os gestos compartilhados e todos os específicos de cada aplicativo.

Clique em **Avançar** para revisar o resultado. Nada muda até você confirmar:

- Traçados idênticos são combinados automaticamente. Traçados com as mesmas direções que diferem só no comprimento dos trechos são combinados por padrão.
- **Formas parecidas: confirmar**: em cada grupo, escolha o que manter (**Usar este**) ou **Combinar amostras**. Grupos abaixo da semelhança recomendada não são combinados por padrão, e uma forma combinada pode ser reconhecida de modo um pouco diferente.
- **Adicionados diretamente**: expanda para desmarcar gestos. Os desmarcados não são importados e continuam específicos.

Clique em **Importar** para concluir; o aviso permite desfazer. Dois gestos parecidos no mesmo aplicativo que fazem coisas diferentes nunca são combinados.

## Usar um gesto em outro aplicativo

Nos detalhes do gesto, clique em **Usar em outro aplicativo…**, escolha um aplicativo e depois uma ação. Aplicativos que já usam o gesto não podem ser escolhidos; se um aplicativo já tem o mesmo traçado, aparece um aviso de conflito; é possível atribuir a um aplicativo com gestos desativados, mas lá ele não funcionará. **Adicionar aplicativo…**, no fim da lista, adiciona primeiro um aplicativo. Depois você continua nos detalhes do gesto com a nova linha destacada, e **Configurar** no aviso abre essa atribuição.

Você também pode começar pelo aplicativo. No editor, um gesto que pertence só ao aplicativo atual mostra **Somente neste aplicativo**:

- Clique em **Escolher da biblioteca** para usar um gesto da biblioteca. O seletor permite pesquisa; escolher um dos **Gestos em outros aplicativos** primeiro o adiciona à biblioteca e o compartilha com o original; **Gravar um gesto novo (só neste aplicativo)** não entra na biblioteca.
- O menu **Mais ações** tem **Adicionar à biblioteca** e **Usar em outro aplicativo…**.

## Editar um gesto compartilhado

No editor, um gesto da biblioteca mostra **Biblioteca** e onde é usado. Ao redesenhá-lo, a alteração vale para todos os lugares que o usam, e os novos conflitos são listados:

- **Alterar só aqui**: a alteração fica só no aplicativo atual; o gesto passa a ser específico, e a biblioteca e os outros aplicativos mantêm o traçado anterior.
- **Concluído**: mantém a alteração compartilhada e fecha o aviso.

**Trocar** troca por outro gesto da biblioteca; o botão ↗ (**Ver na biblioteca**) abre os detalhes do gesto.

## Criar e excluir

**Novo gesto** grava um gesto diretamente na biblioteca. Ele é adicionado quando você termina de desenhar; o nome é opcional. Atribua-o depois com **Usar em outro aplicativo…**.

Se escolher **Excluir gesto** e o gesto ainda estiver em uso, escolha:

- **Excluir; cada uso mantém a própria cópia**
- **Excluir junto com todos os usos**

As duas opções podem ser desfeitas pelo aviso.

## Configuração e compatibilidade

A biblioteca fica em `gestures` no arquivo de configuração, e cada gesto que a usa guarda também uma cópia completa do traçado. Por isso, versões anteriores do Mousetroke conseguem ler a nova configuração e os gestos continuam funcionando. Porém, salvar as configurações em uma versão anterior descarta a biblioteca: cada gesto volta a ser específico, e você pode organizar de novo depois de atualizar. Se editar o arquivo à mão fizer um traçado divergir da biblioteca, esse lugar passa a ser específico e você é avisado uma vez. Local dos arquivos e backups: [Configuração, backup e atualizações](config.md).

Próximo: [Gestos de mouse](gestures.md), [Regras por aplicativo](per-app.md), [Biblioteca de ações](action-library.md).
