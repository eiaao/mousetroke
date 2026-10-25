---
title: "Configuração, backup e atualizações"
description: "Encontre a configuração e os logs do Mousetroke, exporte e restaure as configurações, recarregue alterações, atualize o aplicativo ou desinstale-o."
order: 11
---

# Configuração, backup e atualizações

## Locais dos arquivos

| Arquivo | Windows | Mac |
|---|---|---|
| Configuração | `%APPDATA%\mousetroke\config.json` | `~/Library/Application Support/mousetroke/config.json` |
| Log do tempo de execução | `%APPDATA%\mousetroke\mousetroke.log` | `~/Library/Logs/mousetroke/mousetroke.log` |

Os logs são rotacionados para `mousetroke.log.old` após exceder 1 MB. **Sobre → Locais dos arquivos** exibe os caminhos reais. **Abrir pasta** abre o diretório de configuração; **Abrir log** abre o log do tempo de execução. Em **Importar e exportar**, **Mostrar no Explorador de Arquivos** no Windows ou **Mostrar no Finder** no Mac revela a configuração.

## Exportar, importar e restaurar

Em **Importar e exportar → Arquivo de configuração do Mousetroke**, clique em **Exportar…** e salve um backup JSON. **Importar de um arquivo…** lê um JSON de configuração do Mousetroke e substitui a configuração atual, incluindo as preferências. Ele não mescla aquele arquivo com os seus vínculos atuais. Não há opção de mesclagem da configuração nativa neste fluxo. Você pode desfazer a importação com Ctrl+Z no Windows ou ⌘+Z no Mac enquanto as configurações estiverem abertas.

Antes de editar o JSON manualmente, exporte um backup. Para uma restauração direta, encerre o motor e feche as configurações, substitua `config.json` pelo backup e reabra o Mousetroke. Manter as configurações fechadas evita que o salvamento automático delas sobrescreva o arquivo restaurado.

## Recarregar e redefinir

O motor observa o carimbo de data/hora do arquivo de configuração. O Windows verifica cerca de uma vez por segundo; o Mac verifica no ramo de manutenção agendado a cada dois segundos. Uma alteração recebida durante um gesto é aplicada após o fim da captura. JSON inválido mantém a configuração ativa anterior em uso e registra um erro no log.

A interface de configurações atual não tem um botão de restaurar padrões. Para começar com os padrões integrados, encerre o Mousetroke e feche as configurações, faça backup e remova ou renomeie `config.json`, então reabra. Um arquivo ausente é inicializado com padrões específicos da plataforma. Um arquivo malformado não é um pedido de redefinição.

## Atualizações

**Preferências → Verificar atualizações automaticamente** vem ativado por padrão e consulta o GitHub uma vez por dia. Desative ali para interromper as verificações automáticas. Use **Sobre → Verificar atualizações** para uma verificação manual e **Novidades** para as notas da versão. Um item de menu de nova versão também abre o Sobre.

No Windows, **Atualizar agora** baixa o executável, verifica o SHA-256 dele, substitui a cópia em execução e reinicia. **Abrir página de download** fica disponível para instalação manual.

No Mac, **Atualizar agora** baixa o DMG, confere SHA-256, identidade do bundle, versão e o requisito de assinatura, substitui o aplicativo e reinicia. O bundle antigo vai para a Lixeira após o encerramento; falha na substituição ou na transferência do reinício o restaura. **Baixar manualmente** fica disponível se a pasta de instalação não permitir gravação ou se a atualização falhar. As atualizações dentro do aplicativo evitam outro aviso de quarentena do navegador e preservam a identidade de instalação usada pelas permissões. Um download novo pelo navegador pode precisar de **Abrir Mesmo Assim** novamente.

## Desinstalar

Windows: desative **Ativar os gestos junto com o Windows** nas Preferências, use **Sair** pela área de notificação, feche as configurações e então exclua o executável e `%APPDATA%\mousetroke`, se você não precisar mais das configurações ou dos logs.

Mac: use **Sair** pela barra de menus, feche as configurações e exclua `Mousetroke.app`. Para remover os dados também, exclua `~/Library/Application Support/mousetroke` e `~/Library/Logs/mousetroke`. Remova o Mousetroke dos itens de início nos Ajustes do Sistema. Guarde um backup exportado caso você venha a reinstalar.

Veja a [solução de problemas](troubleshooting.md) para erros de configuração e logs ausentes.
