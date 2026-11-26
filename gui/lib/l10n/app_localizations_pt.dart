// ignore: unused_import
import 'package:intl/intl.dart' as intl;

import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for Portuguese (`pt`).
class AppLocalizationsPt extends AppLocalizations {
  AppLocalizationsPt([String locale = 'pt']) : super(locale);

  @override
  String get memoryLibrary => 'Memória';

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
  String get agentInstructions => 'Instruções do utilizador';

  @override
  String get agentMemoryDigest => 'Memória do utilizador';

  @override
  String get agentMemoryDigestHelp =>
      'A publicação grava a memória global e a deste agente nas suas instruções de utilizador.';

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
