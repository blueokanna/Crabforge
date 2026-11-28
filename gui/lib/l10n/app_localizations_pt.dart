// ignore: unused_import
import 'package:intl/intl.dart' as intl;

import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for Portuguese (`pt`).
class AppLocalizationsPt extends AppLocalizations {
  AppLocalizationsPt([String locale = 'pt']) : super(locale);

  @override
  String get memoryTidy => 'Organização noturna da memória';

  @override
  String get memoryTidyHelp =>
      'Quando ativada, a organização começa depois das 3:00 da madrugada, hora local. O gateway envia as memórias guardadas para a rota de modelo escolhida para detetar notas duplicadas e desatualizadas. As memórias escritas manualmente ou fixadas estão protegidas.';

  @override
  String get memoryTidyRun => 'Organizar agora';

  @override
  String get memoryTidyUndo => 'Desfazer a última organização';

  @override
  String memoryTidyChanged(String count) {
    return 'Memórias alteradas ($count)';
  }

  @override
  String memoryTidyRestored(String count) {
    return 'Alterações desfeitas ($count)';
  }

  @override
  String get interfaceFont => 'Fonte da interface';

  @override
  String get importSlashCommands => 'Importar comandos de barra';

  @override
  String get commandImportSource =>
      'Arquivo Markdown ou TOML, pasta local ou URL Git';

  @override
  String get commandChooseFile => 'Escolher arquivo de comando';

  @override
  String get projectHookAssignment => 'Editar atribuição do hook ao projeto';

  @override
  String get sharedSubagents => 'Subagentes compartilhados';

  @override
  String get importSubagents => 'Importar subagentes';

  @override
  String get subagentImportSource =>
      'Arquivo Markdown, pasta local ou URL do Git';

  @override
  String get subagentChooseFile => 'Escolher arquivo Markdown';

  @override
  String get subagentChooseFolder => 'Escolher pasta de origem';

  @override
  String get subagentScanSource => 'Examinar a origem';

  @override
  String get subagentImportTruncated =>
      'Apenas parte da origem foi examinada. Escolha uma pasta menor para ver mais arquivos.';

  @override
  String subagentImportStatus(String status) {
    String _temp0 = intl.Intl.selectLogic(status, {
      'update': 'Atualização disponível',
      'unchanged': 'Já importado',
      'edited': 'Alterações locais preservadas',
      'other': '',
    });
    return '$_temp0';
  }

  @override
  String get findNativeResources => 'Buscar arquivos existentes';

  @override
  String get noNativeResources => 'Nenhum arquivo existente encontrado';

  @override
  String get nativeResourceScanLimit =>
      'Os primeiros 512 arquivos encontrados são exibidos.';

  @override
  String get adoptResourceDetails =>
      'O Crabforge gerenciará este arquivo no local atual. Remover a atribuição ao cliente pode excluir o arquivo se ele estiver intacto.';

  @override
  String get agentSubagents => 'Subagentes';

  @override
  String get agentCommands => 'Comandos de barra';

  @override
  String get addSubagent => 'Adicionar subagente';

  @override
  String get addSlashCommand => 'Adicionar comando de barra';

  @override
  String get resourceUserScope => 'Perfis de usuário';

  @override
  String get noAgentResources => 'Nenhuma entrada salva';

  @override
  String get resourcePlacements => 'Detalhes da publicação';

  @override
  String get agentResourceName => 'Nome';

  @override
  String get agentResourceDescription => 'Descrição';

  @override
  String get agentResourcePrompt => 'Instrução';

  @override
  String get resourceSkillNames => 'Nomes das habilidades, um por linha';

  @override
  String get resourceMcpNames => 'Nomes dos servidores MCP, um por linha';

  @override
  String get commandArgumentHelp =>
      'Use \$ARGUMENTS para todos os argumentos e \$1, \$2, etc. para argumentos individuais em clientes compatíveis.';

  @override
  String get agentHooks => 'Hooks';

  @override
  String get addHook => 'Adicionar hook';

  @override
  String get editHook => 'Editar hook';

  @override
  String get adoptHook => 'Adotar hook';

  @override
  String get noHooks => 'Nenhum hook salvo';

  @override
  String get installedHooks => 'Hooks nas configurações dos clientes';

  @override
  String get removeInstalledHook => 'Remover hook instalado';

  @override
  String get hookBackups => 'Backups de hooks';

  @override
  String get restoreHookBackup => 'Restaurar backup do hook';

  @override
  String get hookBackupRestored => 'Restaurado';

  @override
  String get hookBackupChanged => 'As configurações do cliente mudaram';

  @override
  String get hookBackupUnavailable => 'Perfil ou arquivo original indisponível';

  @override
  String get hookName => 'Nome do hook';

  @override
  String get hookDescription => 'Descrição (opcional)';

  @override
  String get hookEvent => 'Evento';

  @override
  String get hookMatcher => 'Filtro';

  @override
  String get hookAllTools => 'Todas as ferramentas';

  @override
  String get hookShellTool => 'Comandos do shell';

  @override
  String get hookEditTool => 'Edições de arquivos';

  @override
  String get hookCommand => 'Comando executado pelo cliente';

  @override
  String get hookTimeout => 'Tempo limite em segundos (opcional)';

  @override
  String get hookTurnEnd => 'Fim do turno';

  @override
  String get hookAfterTool => 'Após usar a ferramenta';

  @override
  String get hookBeforeTool => 'Antes de usar a ferramenta';

  @override
  String get hookPrompt => 'Envio da mensagem';

  @override
  String get hookSessionStart => 'Início da sessão';

  @override
  String get hookExisting => 'Já existe um hook idêntico';

  @override
  String get hookConflict => 'Hooks existentes preservados';

  @override
  String get agentEnvironment => 'Variáveis de ambiente';

  @override
  String get noEnvironmentVariables => 'Nenhuma variável salva';

  @override
  String get addVariable => 'Adicionar variável';

  @override
  String get editVariable => 'Editar variável';

  @override
  String get variableName => 'Nome da variável';

  @override
  String get variableValue => 'Valor';

  @override
  String get showVariableValue => 'Mostrar valor';

  @override
  String get hideVariableValue => 'Ocultar valor';

  @override
  String get nativeVariableExists => 'Já definida';

  @override
  String get nativeVariableConflict => 'Valor existente preservado';

  @override
  String get memoryLibrary => 'Memória';

  @override
  String get agentRules => 'Regras';

  @override
  String get agentSkills => 'Habilidades';

  @override
  String get importSkill => 'Importar habilidade';

  @override
  String get applySkills => 'Aplicar habilidades';

  @override
  String get noSkills => 'Nenhuma habilidade importada';

  @override
  String get editSkill => 'Editar atribuições da habilidade';

  @override
  String get skillSourceFolder => 'Pasta local ou URL do Git';

  @override
  String get checkSkillUpdate => 'Buscar atualizações';

  @override
  String get browseRepositorySkills => 'Explorar habilidades da origem';

  @override
  String get noRepositorySkills =>
      'Nenhuma outra habilidade disponível nesta origem.';

  @override
  String get dismissSkillCandidate => 'Ocultar habilidade';

  @override
  String get skillUpToDate => 'Os arquivos da habilidade estão atualizados.';

  @override
  String get skillUpdateAvailable =>
      'Há novos arquivos da habilidade. Importe-os novamente para aplicá-los.';

  @override
  String get skillRemovedUpstream =>
      'A pasta selecionada não contém mais SKILL.md.';

  @override
  String get installedSkills => 'Habilidades instaladas';

  @override
  String get adoptSkill => 'Gerenciar no Crabforge';

  @override
  String get adoptSkillHint =>
      'O Crabforge salvará uma cópia dos arquivos e gerenciará esta pasta. Remover a atribuição pode excluir a pasta se ela estiver intacta.';

  @override
  String get mcpLibrary => 'Biblioteca de MCP';

  @override
  String get assignMcpServer => 'Atribuir a este projeto';

  @override
  String get unassignMcpServer => 'Remover atribuição deste projeto';

  @override
  String get syncMcpLibrary => 'Sincronizar atribuições';

  @override
  String get refreshSkill => 'Reimportar arquivos';

  @override
  String get projectMcp => 'MCP do projeto';

  @override
  String get addMcpServer => 'Adicionar servidor MCP';

  @override
  String get editMcpServer => 'Editar servidor MCP';

  @override
  String get noMcpServers => 'Nenhum servidor MCP configurado';

  @override
  String get mcpServerName => 'Nome do servidor';

  @override
  String get addRule => 'Adicionar regra';

  @override
  String get editRule => 'Editar regra';

  @override
  String get syncRules => 'Aplicar regras';

  @override
  String get noRules => 'Nenhuma regra salva';

  @override
  String get ruleName => 'Nome da regra';

  @override
  String get ruleBody => 'Instruções em Markdown';

  @override
  String get rulePaths => 'Padrões de caminho (opcionais, um por linha)';

  @override
  String get ruleRequired => 'Este campo é obrigatório.';

  @override
  String ruleStatus(String value) {
    String _temp0 = intl.Intl.selectLogic(value, {
      'applied': 'Aplicada',
      'pending': 'Pendente',
      'removed': 'Removida',
      'conflict': 'Arquivo existente mantido',
      'other': 'Falha ao aplicar',
    });
    return '$_temp0';
  }

  @override
  String get agentSessions => 'Sessões locais';

  @override
  String get allAgents => 'Todos os agentes';

  @override
  String get noAgentSessions => 'Nenhuma sessão local encontrada';

  @override
  String unreadableSessions(int count) {
    return 'Ficheiros de sessão ilegíveis ($count)';
  }

  @override
  String get copySessionId => 'Copiar ID da sessão';

  @override
  String get sessionWorkingFolder => 'Pasta de trabalho';

  @override
  String get sessionProfileFolder => 'Pasta do perfil';

  @override
  String get distillTitle => 'Extrair memórias';

  @override
  String get distillPreview =>
      'Revise as mensagens do usuário abaixo antes de enviá-las à rota do modelo selecionado. As novas memórias serão salvas para revisão.';

  @override
  String get distillSend => 'Enviar ao modelo';

  @override
  String get distillEmpty => 'Nenhuma nova memória para revisar.';

  @override
  String get memorySourceQuote => 'Citação da sessão';

  @override
  String get agentInstructions => 'Instruções do utilizador';

  @override
  String get agentMemoryDigest => 'Memória do utilizador';

  @override
  String get agentMemoryDigestHelp =>
      'A publicação grava a memória global e a deste agente nas suas instruções de utilizador.';

  @override
  String get autoPublishAgentMemory => 'Publicar a memória diariamente';

  @override
  String get autoPublishAgentMemoryHelp =>
      'Atualiza diariamente o resumo das memórias ativas neste ficheiro enquanto o gateway está em execução.';

  @override
  String get agentMemoryDigestEmpty =>
      'Não há memórias globais ou deste agente ativas para publicar.';

  @override
  String get projectMemory => 'Memória do projeto';

  @override
  String get publishMemory => 'Publicar resumo';

  @override
  String get projectMemoryHelp =>
      'O resumo é gravado nas instruções do projeto. Qualquer pessoa com acesso ao arquivo pode lê-lo.';

  @override
  String get projectMemoryEmpty =>
      'Este projeto não tem memórias ativas para publicar.';

  @override
  String get searchMemories => 'Pesquisar na memória';

  @override
  String get unreadableMemories => 'Arquivos de memória ilegíveis';

  @override
  String get noMemoriesFound => 'Nenhum resultado na memória';

  @override
  String get addMemory => 'Adicionar à memória';

  @override
  String get editMemory => 'Editar memória';

  @override
  String get memoryScope => 'Escopo';

  @override
  String get memoryTags => 'Etiquetas, uma por linha';

  @override
  String get memoryWhy => 'Motivo (opcional)';

  @override
  String get memoryKindLabel => 'Tipo';

  @override
  String get memoryStateLabel => 'Estado';

  @override
  String get pinMemory => 'Fixar entrada';

  @override
  String memoryKind(String value) {
    String _temp0 = intl.Intl.selectLogic(value, {
      'profile': 'Perfil',
      'preference': 'Preferência',
      'rule': 'Regra',
      'reference': 'Referência',
      'fact': 'Fato',
      'procedure': 'Procedimento',
      'episode': 'Evento',
      'other': 'Tipo',
    });
    return '$_temp0';
  }

  @override
  String memoryState(String value) {
    String _temp0 = intl.Intl.selectLogic(value, {
      'active': 'Ativo',
      'candidate': 'Pendente',
      'archived': 'Arquivado',
      'superseded': 'Substituído',
      'other': 'Todos os estados',
    });
    return '$_temp0';
  }

  @override
  String get projectInstructions => 'Instruções do projeto';

  @override
  String get projectFolder => 'Pasta do projeto';

  @override
  String get openProject => 'Abrir projeto';

  @override
  String get restoreInstructions => 'Restaurar a versão anterior';

  @override
  String get relayFallback =>
      'Endereço do circuito de retransmissão alternativo (opcional)';

  @override
  String get listView => 'Visualização em lista';

  @override
  String get gridView => 'Visualização em grade';

  @override
  String get gatewayProtocol => 'Protocolo do gateway';

  @override
  String get nativeModelProtocol => 'Usar o protocolo nativo de cada modelo';

  @override
  String get roleModels => 'Modelos por função';

  @override
  String get roleModelId => 'ID do modelo';

  @override
  String get resetRole => 'Restaurar função';

  @override
  String get syncModelCatalog => 'Sincronizar catálogo de modelos';

  @override
  String get thinkingLevels => 'Níveis de raciocínio';

  @override
  String get sessionThinking => 'Nível de raciocínio da sessão';

  @override
  String get thinkingAuto => 'Automático';

  @override
  String get thinkingMinimal => 'Mínimo';

  @override
  String get thinkingLow => 'Baixo';

  @override
  String get thinkingMedium => 'Médio';

  @override
  String get thinkingHigh => 'Alto';

  @override
  String get thinkingExtraHigh => 'Muito alto';

  @override
  String get thinkingMax => 'Máximo';

  @override
  String get ompProfile => 'Perfil do Oh My Pi';

  @override
  String get advisorSettings => 'Configurações do consultor';

  @override
  String get advisorReview => 'Quando revisar';

  @override
  String get advisorWait => 'Revisões pendentes antes de aguardar';

  @override
  String get advisorWake => 'Gravidade mínima para retomar';

  @override
  String get inheritSetting => 'Usar o valor padrão';

  @override
  String get everyTurn => 'A cada turno';

  @override
  String get runEnd => 'Ao final da execução';

  @override
  String get allReviews => 'Aguardar todas as revisões';

  @override
  String get minorNote => 'Questão menor';

  @override
  String get concernNote => 'Motivo de preocupação';

  @override
  String get blockerNote => 'Problema impeditivo';

  @override
  String get memberPatterns => 'Padrões de modelos, um por linha';

  @override
  String get automaticDistillation => 'Extração automática de memórias';

  @override
  String get automaticDistillationHelp =>
      'Enquanto o gateway estiver em execução, o Crabforge envia mensagens do usuário de sessões inativas à rota de modelo selecionada. Revise as sugestões na biblioteca de memórias.';

  @override
  String get distillationEnabled => 'Ativada';

  @override
  String get distillationLookback => 'Período em dias';

  @override
  String get allSessions => 'Todas as sessões listadas';

  @override
  String get distillationRun => 'Ler agora';

  @override
  String get pluginBundles => 'Biblioteca de plugins compartilhada';

  @override
  String get piProjectPackages => 'Pacotes de projeto do Pi';

  @override
  String get claudeProjectPackages => 'Plugins de projeto do Claude Code';

  @override
  String get codexProjectPackages => 'Plugins de projeto do Codex';

  @override
  String get antigravityProjectPackages => 'Plugins de projeto do Antigravity';

  @override
  String get workspacePluginConfirm =>
      'O cliente pode executar código do plugin revisado ao abrir este projeto confiável.';

  @override
  String get claudePackageConfirm =>
      'O Claude Code instalará o plugin revisado para este projeto. Ao abrir o projeto, o Claude Code pode executar os hooks do plugin.';

  @override
  String get piPackageConfirm =>
      'O Pi pode executar extensões de pacotes ao abrir um projeto confiável.';

  @override
  String get piExportAssignments => 'Exportar atribuições';

  @override
  String get piApplyAssignments => 'Aplicar atribuições';

  @override
  String get piAssignmentsTooLarge => 'O arquivo de atribuições excede 1 MiB.';

  @override
  String piPackageStatus(String status) {
    String _temp0 = intl.Intl.selectLogic(status, {
      'available': 'Disponível para atribuição',
      'applied': 'Atribuído',
      'pending': 'Pronto para sincronizar',
      'detached': 'Pacote removido da biblioteca',
      'conflict': 'Entrada existente mantida',
      'other': '',
    });
    return '$_temp0';
  }

  @override
  String get importPluginBundle => 'Importar pacote de plugin';

  @override
  String get previewPluginBundle => 'Visualizar pacote';

  @override
  String get pluginBundleSource => 'Pasta do plugin, manifesto ou URL Git';

  @override
  String get noPluginBundles => 'Nenhum pacote de plugin salvo.';

  @override
  String get otherPluginEntries => 'Outras entradas do manifesto';

  @override
  String get noBundleResources => 'Este arquivo não contém recursos.';

  @override
  String get copyBundleResource => 'Copiar para a biblioteca';

  @override
  String get bundleResourceCopied => 'Copiado para a biblioteca';

  @override
  String get openPluginArchive => 'Abrir arquivo TAR';

  @override
  String get exportPluginArchive => 'Exportar arquivo do plug-in';

  @override
  String get pluginArchiveSaved => 'Arquivo do plug-in salvo';

  @override
  String get pluginCatalog => 'Explorar plugins';

  @override
  String get noAvailablePlugins =>
      'Não há plugins disponíveis nestes catálogos.';

  @override
  String get exportPluginSettings => 'Exportar configurações de plugins';

  @override
  String get importPluginSettings => 'Importar configurações de plugins';

  @override
  String get claudePlugins => 'Plugins do Claude Code';

  @override
  String get pluginMarketplaces => 'Catálogos de plugins';

  @override
  String get addPluginMarketplace => 'Adicionar catálogo';

  @override
  String get pluginSource => 'Caminho ou URL do catálogo';

  @override
  String get installPlugin => 'Instalar plugin';

  @override
  String get updatePlugin => 'Atualizar plugin';

  @override
  String get enablePlugin => 'Ativar plugin';

  @override
  String get disablePlugin => 'Desativar plugin';

  @override
  String get pluginEnabled => 'Ativado';

  @override
  String get noClaudePlugins =>
      'Nenhum plugin do Claude Code está instalado neste escopo.';

  @override
  String get shareGroup => 'Copiar grupo para compartilhar';

  @override
  String get groupCopied => 'Grupo copiado para a área de transferência';

  @override
  String get importGroup => 'Importar grupo';

  @override
  String get sharedGroupText => 'Texto do grupo compartilhado';

  @override
  String get replaceGroup => 'Substituir o grupo existente com o mesmo ID';

  @override
  String get missingMembers => 'Membros indisponíveis';

  @override
  String get replacedMembers => 'Membros substituídos';

  @override
  String routeSession(String id) {
    return 'Sessão $id';
  }

  @override
  String get enableGroup => 'Ativar grupo';

  @override
  String get disableGroup => 'Desativar grupo';

  @override
  String get listGroupModels =>
      'Mostrar os modelos do grupo nos catálogos dos clientes';

  @override
  String get copyGroup => 'Duplicar grupo';

  @override
  String get groupNote => 'Nota do grupo (opcional)';

  @override
  String get firstContentSeconds =>
      'Tempo limite para o primeiro conteúdo (segundos)';

  @override
  String get groupFirstContentHelp =>
      'Use 0 para herdar o valor ou -1 para aguardar o membro.';

  @override
  String get defaultFirstContentHelp =>
      'Padrão dos grupos de roteamento. Use 0 ou -1 para aguardar cada membro.';

  @override
  String get invalidFirstContent =>
      'Digite um número inteiro entre -1 e 2147483647.';

  @override
  String get groupPatienceSeconds => 'Prazo para novas tentativas (segundos)';

  @override
  String get groupPatienceHelp =>
      'Use 0 para herdar o prazo ou -1 para desativar novas tentativas.';

  @override
  String get groupRoutingRules => 'Regras de roteamento (JSON)';

  @override
  String get invalidGroupRoutingRules =>
      'Digite uma lista JSON de regras ou um objeto JSON com uma lista rules.';

  @override
  String get classifyTimeoutSeconds =>
      'Tempo limite do classificador (segundos)';

  @override
  String get classifyTimeoutHelp =>
      'Use de 1 a 60 segundos ou 0 para o padrão de 15 segundos.';

  @override
  String get defaultGroupPatienceHelp =>
      'Use 0 para 60 segundos ou -1 para desativar novas tentativas.';

  @override
  String get invalidGroupPatience => 'Digite um número inteiro entre -1 e 240.';

  @override
  String get enableMember => 'Ativar membro';

  @override
  String get disableMember => 'Desativar membro';

  @override
  String get manualRouting => 'Seleção manual';

  @override
  String get selectMember => 'Selecionar este membro';

  @override
  String get providerCatalogs => 'IDs de catálogo do models.dev (opcional)';

  @override
  String get anthropicAuthHeader => 'Cabeçalho da chave de API da Anthropic';

  @override
  String get automaticAuthHeader => 'Automático';

  @override
  String get autoGroups => 'Agrupar modelos equivalentes de vários provedores';

  @override
  String get restoreGroup => 'Restaurar grupo';

  @override
  String get vertexProject => 'Configurações do projeto do Vertex AI';

  @override
  String get cloudProjectId => 'ID do projeto do Google Cloud';

  @override
  String get vertexLocation => 'Região do Vertex AI';

  @override
  String get googleCredentials => 'Arquivo ADC (opcional)';

  @override
  String get impersonateAccount => 'Conta de serviço a representar (opcional)';

  @override
  String get models => 'Modelos';

  @override
  String get modelCopied => 'ID do modelo copiado';

  @override
  String get noModelsFound => 'Nenhum modelo correspondente';

  @override
  String get routingStrategy => 'Estratégia de encaminhamento';

  @override
  String get listedOrder => 'Ordem da lista';

  @override
  String get weightedOrder => 'Rotação ponderada';

  @override
  String get memberWeight => 'Peso do membro';

  @override
  String get invalidWeight => 'Insira um inteiro entre 1 e 1000.';

  @override
  String get providers => 'Provedores';

  @override
  String get routes => 'Rotas';

  @override
  String get groups => 'Grupos de rotas';

  @override
  String get gateway => 'Gateway';

  @override
  String get language => 'Idioma';

  @override
  String get systemLanguage => 'Idioma do sistema';

  @override
  String confirmRemove(String value) {
    return 'Remover $value?';
  }

  @override
  String get cancel => 'Cancelar';

  @override
  String get remove => 'Remover';

  @override
  String get save => 'Salvar';

  @override
  String get webLogin => 'Login na web';

  @override
  String get loginHelp =>
      'Entre na nova janela do Chrome e volte aqui para fechá-la.';

  @override
  String get closeLogin => 'Fechar janela de login';

  @override
  String get duplicateGroup => 'Este ID de grupo já existe.';

  @override
  String get addProvider => 'Adicionar provedor';

  @override
  String get newGroup => 'Novo grupo';

  @override
  String get stopToEdit => 'Pare o gateway para editar a configuração.';

  @override
  String get reload => 'Recarregar';

  @override
  String get reloadConfig => 'Recarregar configuração';

  @override
  String providerCount(int count) {
    return 'Provedores: $count';
  }

  @override
  String presetCount(int count) {
    return 'Escolha entre $count predefinições de API ou insira uma URL base.';
  }

  @override
  String get providerHelp =>
      'Adicione uma URL de origem e um ID de modelo para aceitar solicitações.';

  @override
  String get removeProvider => 'Remover provedor';

  @override
  String get webProviders => 'Provedores web';

  @override
  String get webHelp =>
      'Use o Chrome com um ChromeDriver compatível. Selecione o provedor adicionado para entrar no perfil dedicado.';

  @override
  String webSite(String value) {
    return '$value web';
  }

  @override
  String get groupNeedsProvider =>
      'Adicione um provedor e um modelo antes de criar um grupo de rotas.';

  @override
  String get groupHelp =>
      'Adicione membros por prioridade. Em caso de limite de uso ou falha do servidor, o gateway tenta o próximo.';

  @override
  String get removeGroup => 'Remover grupo';

  @override
  String recentAttempts(int count) {
    return 'Tentativas recentes de conexão: $count';
  }

  @override
  String get historyLimit => 'Processo atual · últimas 128 tentativas';

  @override
  String get waiting => 'Aguardando uma solicitação do cliente.';

  @override
  String get historyHelp => 'Inicie o gateway para ver o histórico.';

  @override
  String get running => 'Gateway em execução';

  @override
  String get stopped => 'Gateway parado';

  @override
  String get stop => 'Parar gateway';

  @override
  String get start => 'Iniciar gateway';

  @override
  String get keyCopied => 'Chave do gateway copiada';

  @override
  String get copyKey => 'Copiar chave do cliente';

  @override
  String get networkSettings => 'Configurações de rede';

  @override
  String get browserOrigins =>
      'Origens de navegador permitidas (uma por linha)';

  @override
  String get localPeer => 'ID do par local';

  @override
  String get copyPeer => 'Copiar ID do par';

  @override
  String get peerCopied => 'ID do par copiado';

  @override
  String get listenAddress => 'Endereço de escuta';

  @override
  String get disabled => 'Desativado';

  @override
  String get trustedPeers => 'Pares confiáveis';

  @override
  String get clients => 'Conexões de clientes';

  @override
  String get agentRoot => 'URL raiz da API do agente';

  @override
  String get copyRoot => 'Copiar URL raiz';

  @override
  String get rootCopied => 'URL raiz copiada';

  @override
  String get agentHelp =>
      'Acrescente /v1 para clientes Chat Completions e Responses. O histórico mostra o agente selecionado.';

  @override
  String get tunnel => 'Túnel para o gateway remoto';

  @override
  String get disconnected => 'Desconectado';

  @override
  String get connect => 'Conectar';

  @override
  String get disconnect => 'Desconectar';

  @override
  String get apiBase => 'URL base da API';

  @override
  String get copyBase => 'Copiar URL base';

  @override
  String get baseCopied => 'URL base da API copiada';

  @override
  String get changed => 'A configuração mudou. Carregue os modelos novamente.';

  @override
  String get catalogRefreshInterval =>
      'Intervalo de atualização do catálogo de modelos';

  @override
  String get catalogAutoInclude =>
      'Incluir automaticamente os modelos que correspondam a estas regras';

  @override
  String modelsRead(int count) {
    return 'Modelos encontrados: $count. Edite a lista antes de salvar.';
  }

  @override
  String get presetLoaded =>
      'Modelos predefinidos carregados. Ajuste a lista à sua conta.';

  @override
  String get editProvider => 'Editar provedor';

  @override
  String get searchPresets => 'Buscar predefinições por nome';

  @override
  String get providerId => 'ID do provedor';

  @override
  String get renameProvider => 'Renomear provedor';

  @override
  String get keyEnv => 'Variável de ambiente da chave da API';

  @override
  String get keyHint => 'Opcional para serviços locais ou Bedrock com AWS IAM';

  @override
  String get resourceRoot => 'URL raiz do recurso';

  @override
  String get modelIds => 'IDs dos modelos de origem, um por linha';

  @override
  String get providerFallback =>
      'Modelos ou grupos de reserva, em ordem, um por linha';

  @override
  String get allowProviderFallback =>
      'Aceitar solicitações de reserva de outros provedores';

  @override
  String protocolBase(String value) {
    return 'URL base de $value';
  }

  @override
  String get catalogUrl => 'URL do catálogo de modelos (opcional)';

  @override
  String get catalogHint => 'Deixe em branco para usar o catálogo padrão';

  @override
  String get readingModels => 'Carregando modelos';

  @override
  String get readModels => 'Carregar catálogo de modelos';

  @override
  String get requiredField => 'Este campo é obrigatório.';

  @override
  String get createGroup => 'Criar grupo de rotas';

  @override
  String get editGroup => 'Editar grupo de rotas';

  @override
  String get groupId => 'ID do grupo de rotas';

  @override
  String get raisePriority => 'Aumentar prioridade';

  @override
  String get removeMember => 'Remover membro';

  @override
  String get addCandidate => 'Adicionar modelo candidato';

  @override
  String get connectRemote => 'Conectar a um gateway remoto';

  @override
  String get remoteQuic => 'Endereço QUIC remoto (com /p2p/PeerID)';

  @override
  String get localTunnel => 'Endereço de escuta local do túnel';

  @override
  String get tunnelHelp =>
      'Autorize este ID de par no gateway remoto. Os clientes do túnel local usam a chave remota.';

  @override
  String get loopback => 'Endereço de loopback HTTP';

  @override
  String get peerListen => 'Endereço de escuta P2P (vazio para desativar)';

  @override
  String get allowedPeers => 'IDs dos pares permitidos, um por linha';

  @override
  String get rotateMembers => 'Alternar o primeiro membro';
}
