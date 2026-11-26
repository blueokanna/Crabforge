// ignore: unused_import
import 'package:intl/intl.dart' as intl;

import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for Spanish Castilian (`es`).
class AppLocalizationsEs extends AppLocalizations {
  AppLocalizationsEs([String locale = 'es']) : super(locale);

  @override
  String get agentHooks => 'Hooks';

  @override
  String get addHook => 'Añadir hook';

  @override
  String get editHook => 'Editar hook';

  @override
  String get adoptHook => 'Adoptar hook';

  @override
  String get noHooks => 'No hay hooks guardados';

  @override
  String get installedHooks => 'Hooks de la configuración de clientes';

  @override
  String get hookName => 'Nombre del hook';

  @override
  String get hookDescription => 'Descripción (opcional)';

  @override
  String get hookEvent => 'Evento';

  @override
  String get hookMatcher => 'Filtro';

  @override
  String get hookAllTools => 'Todas las herramientas';

  @override
  String get hookShellTool => 'Comandos de shell';

  @override
  String get hookEditTool => 'Edición de archivos';

  @override
  String get hookCommand => 'Comando que ejecuta el cliente';

  @override
  String get hookTimeout => 'Tiempo límite en segundos (opcional)';

  @override
  String get hookTurnEnd => 'Fin del turno';

  @override
  String get hookAfterTool => 'Después de usar la herramienta';

  @override
  String get hookBeforeTool => 'Antes de usar la herramienta';

  @override
  String get hookPrompt => 'Envío del mensaje';

  @override
  String get hookSessionStart => 'Inicio de sesión';

  @override
  String get hookExisting => 'Ya existe un hook idéntico';

  @override
  String get hookConflict => 'Se conservaron los hooks existentes';

  @override
  String get agentEnvironment => 'Variables de entorno';

  @override
  String get noEnvironmentVariables => 'No hay variables guardadas';

  @override
  String get addVariable => 'Añadir variable';

  @override
  String get editVariable => 'Editar variable';

  @override
  String get variableName => 'Nombre de la variable';

  @override
  String get variableValue => 'Valor';

  @override
  String get showVariableValue => 'Mostrar valor';

  @override
  String get hideVariableValue => 'Ocultar valor';

  @override
  String get nativeVariableExists => 'Ya configurada';

  @override
  String get nativeVariableConflict => 'Se conservó el valor existente';

  @override
  String get memoryLibrary => 'Memoria';

  @override
  String get agentRules => 'Reglas';

  @override
  String get agentSkills => 'Habilidades';

  @override
  String get importSkill => 'Importar habilidad';

  @override
  String get applySkills => 'Aplicar habilidades';

  @override
  String get noSkills => 'No hay habilidades importadas';

  @override
  String get editSkill => 'Editar asignaciones de la habilidad';

  @override
  String get skillSourceFolder => 'Carpeta local o URL de Git';

  @override
  String get checkSkillUpdate => 'Buscar actualizaciones';

  @override
  String get browseRepositorySkills => 'Explorar habilidades del origen';

  @override
  String get noRepositorySkills =>
      'No se encontraron más habilidades en este origen.';

  @override
  String get dismissSkillCandidate => 'Ocultar habilidad';

  @override
  String get skillUpToDate => 'Los archivos de la habilidad están al día.';

  @override
  String get skillUpdateAvailable =>
      'Hay archivos nuevos. Vuelve a importarlos para aplicarlos.';

  @override
  String get skillRemovedUpstream =>
      'La carpeta seleccionada ya no contiene SKILL.md.';

  @override
  String get installedSkills => 'Habilidades instaladas';

  @override
  String get adoptSkill => 'Gestionar en Crabforge';

  @override
  String get adoptSkillHint =>
      'Crabforge guardará una copia de los archivos y gestionará esta carpeta. Al quitar la asignación, se podrá eliminar la carpeta si sigue intacta.';

  @override
  String get mcpLibrary => 'Biblioteca de MCP';

  @override
  String get assignMcpServer => 'Asignar a este proyecto';

  @override
  String get unassignMcpServer => 'Quitar la asignación de este proyecto';

  @override
  String get syncMcpLibrary => 'Sincronizar asignaciones';

  @override
  String get refreshSkill => 'Volver a importar archivos';

  @override
  String get projectMcp => 'MCP del proyecto';

  @override
  String get addMcpServer => 'Añadir servidor MCP';

  @override
  String get editMcpServer => 'Editar servidor MCP';

  @override
  String get noMcpServers => 'No hay servidores MCP configurados';

  @override
  String get mcpServerName => 'Nombre del servidor';

  @override
  String get addRule => 'Añadir regla';

  @override
  String get editRule => 'Editar regla';

  @override
  String get syncRules => 'Aplicar reglas';

  @override
  String get noRules => 'No hay reglas guardadas';

  @override
  String get ruleName => 'Nombre de la regla';

  @override
  String get ruleBody => 'Instrucciones en Markdown';

  @override
  String get rulePaths => 'Patrones de ruta (opcionales, uno por línea)';

  @override
  String get ruleRequired => 'Este campo es obligatorio.';

  @override
  String ruleStatus(String value) {
    String _temp0 = intl.Intl.selectLogic(value, {
      'applied': 'Aplicada',
      'pending': 'Pendiente',
      'removed': 'Eliminada',
      'conflict': 'Archivo existente conservado',
      'other': 'No se pudo aplicar',
    });
    return '$_temp0';
  }

  @override
  String get agentSessions => 'Sesiones locales';

  @override
  String get allAgents => 'Todos los agentes';

  @override
  String get noAgentSessions => 'No se encontraron sesiones locales';

  @override
  String unreadableSessions(int count) {
    return 'Archivos de sesión ilegibles ($count)';
  }

  @override
  String get copySessionId => 'Copiar ID de sesión';

  @override
  String get sessionWorkingFolder => 'Carpeta de trabajo';

  @override
  String get sessionProfileFolder => 'Carpeta del perfil';

  @override
  String get distillTitle => 'Extraer recuerdos';

  @override
  String get distillPreview =>
      'Revisa los mensajes del usuario antes de enviarlos a la ruta del modelo seleccionado. Los nuevos recuerdos se guardarán para su revisión.';

  @override
  String get distillSend => 'Enviar al modelo';

  @override
  String get distillEmpty => 'No se encontraron nuevos recuerdos candidatos.';

  @override
  String get memorySourceQuote => 'Cita de la sesión';

  @override
  String get agentInstructions => 'Instrucciones del usuario';

  @override
  String get agentMemoryDigest => 'Memoria del usuario';

  @override
  String get agentMemoryDigestHelp =>
      'Al publicar, la memoria global y la de este agente se guardan en sus instrucciones de usuario.';

  @override
  String get agentMemoryDigestEmpty =>
      'No hay recuerdos activos globales ni de este agente para publicar.';

  @override
  String get projectMemory => 'Memoria del proyecto';

  @override
  String get publishMemory => 'Publicar resumen';

  @override
  String get projectMemoryHelp =>
      'El resumen se guarda en las instrucciones del proyecto. Cualquiera con acceso al archivo puede leerlo.';

  @override
  String get projectMemoryEmpty =>
      'Este proyecto no tiene memorias activas que publicar.';

  @override
  String get searchMemories => 'Buscar en la memoria';

  @override
  String get unreadableMemories => 'Archivos de memoria ilegibles';

  @override
  String get noMemoriesFound => 'No hay resultados en la memoria';

  @override
  String get addMemory => 'Añadir a la memoria';

  @override
  String get editMemory => 'Editar memoria';

  @override
  String get memoryScope => 'Ámbito';

  @override
  String get memoryTags => 'Etiquetas, una por línea';

  @override
  String get memoryWhy => 'Motivo (opcional)';

  @override
  String get memoryKindLabel => 'Tipo';

  @override
  String get memoryStateLabel => 'Estado';

  @override
  String get pinMemory => 'Fijar entrada';

  @override
  String memoryKind(String value) {
    String _temp0 = intl.Intl.selectLogic(value, {
      'profile': 'Perfil',
      'preference': 'Preferencia',
      'rule': 'Regla',
      'reference': 'Referencia',
      'fact': 'Hecho',
      'procedure': 'Procedimiento',
      'episode': 'Suceso',
      'other': 'Tipo',
    });
    return '$_temp0';
  }

  @override
  String memoryState(String value) {
    String _temp0 = intl.Intl.selectLogic(value, {
      'active': 'Activo',
      'candidate': 'Pendiente',
      'archived': 'Archivado',
      'superseded': 'Sustituido',
      'other': 'Todos los estados',
    });
    return '$_temp0';
  }

  @override
  String get projectInstructions => 'Instrucciones del proyecto';

  @override
  String get projectFolder => 'Carpeta del proyecto';

  @override
  String get openProject => 'Abrir proyecto';

  @override
  String get restoreInstructions => 'Restaurar la versión anterior';

  @override
  String get relayFallback =>
      'Dirección del circuito de retransmisión alternativo (opcional)';

  @override
  String get listView => 'Vista de lista';

  @override
  String get gridView => 'Vista de cuadrícula';

  @override
  String get gatewayProtocol => 'Protocolo de la pasarela';

  @override
  String get nativeModelProtocol => 'Usar el protocolo nativo de cada modelo';

  @override
  String get roleModels => 'Modelos por rol';

  @override
  String get roleModelId => 'ID del modelo';

  @override
  String get resetRole => 'Restablecer rol';

  @override
  String get syncModelCatalog => 'Sincronizar catálogo de modelos';

  @override
  String get thinkingLevels => 'Niveles de razonamiento';

  @override
  String get sessionThinking => 'Nivel de razonamiento de la sesión';

  @override
  String get thinkingAuto => 'Automático';

  @override
  String get thinkingMinimal => 'Mínimo';

  @override
  String get thinkingLow => 'Bajo';

  @override
  String get thinkingMedium => 'Medio';

  @override
  String get thinkingHigh => 'Alto';

  @override
  String get thinkingExtraHigh => 'Muy alto';

  @override
  String get thinkingMax => 'Máximo';

  @override
  String get ompProfile => 'Perfil de Oh My Pi';

  @override
  String get advisorSettings => 'Ajustes del asesor';

  @override
  String get advisorReview => 'Cuándo revisar';

  @override
  String get advisorWait => 'Revisiones pendientes antes de esperar';

  @override
  String get advisorWake => 'Gravedad mínima para reanudar';

  @override
  String get inheritSetting => 'Usar el valor predeterminado';

  @override
  String get everyTurn => 'En cada turno';

  @override
  String get runEnd => 'Al finalizar la ejecución';

  @override
  String get allReviews => 'Esperar todas las revisiones';

  @override
  String get minorNote => 'Detalle menor';

  @override
  String get concernNote => 'Motivo de preocupación';

  @override
  String get blockerNote => 'Problema bloqueante';

  @override
  String get memberPatterns => 'Patrones de modelos, uno por línea';

  @override
  String get shareGroup => 'Copiar grupo para compartir';

  @override
  String get groupCopied => 'Grupo copiado al portapapeles';

  @override
  String get importGroup => 'Importar grupo';

  @override
  String get sharedGroupText => 'Texto del grupo compartido';

  @override
  String get replaceGroup => 'Reemplazar el grupo existente con el mismo ID';

  @override
  String get missingMembers => 'Miembros no disponibles';

  @override
  String get replacedMembers => 'Miembros sustituidos';

  @override
  String routeSession(String id) {
    return 'Sesión $id';
  }

  @override
  String get enableGroup => 'Activar grupo';

  @override
  String get disableGroup => 'Desactivar grupo';

  @override
  String get listGroupModels =>
      'Mostrar los modelos del grupo en los catálogos de los clientes';

  @override
  String get copyGroup => 'Duplicar grupo';

  @override
  String get groupNote => 'Nota del grupo (opcional)';

  @override
  String get enableMember => 'Activar miembro';

  @override
  String get disableMember => 'Desactivar miembro';

  @override
  String get manualRouting => 'Selección manual';

  @override
  String get selectMember => 'Seleccionar este miembro';

  @override
  String get providerCatalogs =>
      'Identificadores de catálogo de models.dev (opcional)';

  @override
  String get anthropicAuthHeader => 'Cabecera de la clave de API de Anthropic';

  @override
  String get automaticAuthHeader => 'Automática';

  @override
  String get autoGroups =>
      'Agrupar modelos equivalentes de distintos proveedores';

  @override
  String get restoreGroup => 'Restaurar grupo';

  @override
  String get vertexProject => 'Configuración del proyecto de Vertex AI';

  @override
  String get cloudProjectId => 'ID del proyecto de Google Cloud';

  @override
  String get vertexLocation => 'Ubicación de Vertex AI';

  @override
  String get googleCredentials => 'Archivo ADC (opcional)';

  @override
  String get impersonateAccount =>
      'Cuenta de servicio que se va a suplantar (opcional)';

  @override
  String get models => 'Modelos';

  @override
  String get modelCopied => 'ID del modelo copiado';

  @override
  String get noModelsFound => 'No hay modelos coincidentes';

  @override
  String get routingStrategy => 'Estrategia de enrutamiento';

  @override
  String get listedOrder => 'Orden de la lista';

  @override
  String get weightedOrder => 'Rotación ponderada';

  @override
  String get memberWeight => 'Peso del miembro';

  @override
  String get invalidWeight => 'Introduce un entero entre 1 y 1000.';

  @override
  String get providers => 'Proveedores';

  @override
  String get routes => 'Rutas';

  @override
  String get groups => 'Grupos de rutas';

  @override
  String get gateway => 'Puerta de enlace';

  @override
  String get language => 'Idioma';

  @override
  String get systemLanguage => 'Idioma del sistema';

  @override
  String confirmRemove(String value) {
    return '¿Quitar $value?';
  }

  @override
  String get cancel => 'Cancelar';

  @override
  String get remove => 'Quitar';

  @override
  String get save => 'Guardar';

  @override
  String get webLogin => 'Acceso web';

  @override
  String get loginHelp =>
      'Inicia sesión en la nueva ventana de Chrome y vuelve aquí para cerrarla.';

  @override
  String get closeLogin => 'Cerrar ventana de acceso';

  @override
  String get duplicateGroup => 'Este ID de grupo ya existe.';

  @override
  String get addProvider => 'Añadir proveedor';

  @override
  String get newGroup => 'Nuevo grupo';

  @override
  String get stopToEdit =>
      'Detén la puerta de enlace para editar su configuración.';

  @override
  String get reload => 'Volver a cargar';

  @override
  String get reloadConfig => 'Recargar configuración';

  @override
  String providerCount(int count) {
    return 'Proveedores: $count';
  }

  @override
  String presetCount(int count) {
    return 'Elige entre $count ajustes de API o introduce una URL base.';
  }

  @override
  String get providerHelp =>
      'Añade una URL de origen y un ID de modelo para admitir solicitudes.';

  @override
  String get removeProvider => 'Quitar proveedor';

  @override
  String get webProviders => 'Proveedores web';

  @override
  String get webHelp =>
      'Usa Chrome con un ChromeDriver compatible. Selecciona el proveedor añadido para iniciar sesión en su perfil propio.';

  @override
  String webSite(String value) {
    return '$value web';
  }

  @override
  String get groupNeedsProvider =>
      'Añade un proveedor y un modelo antes de crear un grupo de rutas.';

  @override
  String get groupHelp =>
      'Añade miembros por prioridad. Si hay límites de uso o fallos del servidor, se prueba el siguiente.';

  @override
  String get removeGroup => 'Quitar grupo';

  @override
  String recentAttempts(int count) {
    return 'Intentos recientes de conexión: $count';
  }

  @override
  String get historyLimit => 'Proceso actual · últimos 128 intentos';

  @override
  String get waiting => 'Esperando una solicitud del cliente.';

  @override
  String get historyHelp => 'Inicia la puerta de enlace para ver el historial.';

  @override
  String get running => 'Puerta de enlace activa';

  @override
  String get stopped => 'Puerta de enlace detenida';

  @override
  String get stop => 'Detener puerta de enlace';

  @override
  String get start => 'Iniciar puerta de enlace';

  @override
  String get keyCopied => 'Clave copiada';

  @override
  String get copyKey => 'Copiar clave de cliente';

  @override
  String get networkSettings => 'Configuración de red';

  @override
  String get browserOrigins =>
      'Orígenes de navegador permitidos (uno por línea)';

  @override
  String get localPeer => 'ID del par local';

  @override
  String get copyPeer => 'Copiar ID del par';

  @override
  String get peerCopied => 'ID del par copiado';

  @override
  String get listenAddress => 'Dirección de escucha';

  @override
  String get disabled => 'Desactivado';

  @override
  String get trustedPeers => 'Pares de confianza';

  @override
  String get clients => 'Conexiones de clientes';

  @override
  String get agentRoot => 'URL raíz de la API del agente';

  @override
  String get copyRoot => 'Copiar URL raíz';

  @override
  String get rootCopied => 'URL raíz copiada';

  @override
  String get agentHelp =>
      'Añade /v1 para clientes Chat Completions y Responses. El historial muestra el agente seleccionado.';

  @override
  String get tunnel => 'Túnel a la puerta de enlace remota';

  @override
  String get disconnected => 'Desconectado';

  @override
  String get connect => 'Conectar';

  @override
  String get disconnect => 'Desconectar';

  @override
  String get apiBase => 'URL base de la API';

  @override
  String get copyBase => 'Copiar URL base';

  @override
  String get baseCopied => 'URL base de la API copiada';

  @override
  String get changed => 'Cambió la configuración. Vuelve a cargar los modelos.';

  @override
  String modelsRead(int count) {
    return 'Modelos encontrados: $count. Puedes editar la lista antes de guardarla.';
  }

  @override
  String get presetLoaded =>
      'Modelos predefinidos cargados. Ajusta la lista a tu cuenta.';

  @override
  String get editProvider => 'Editar proveedor';

  @override
  String get searchPresets => 'Buscar ajustes por nombre';

  @override
  String get providerId => 'ID del proveedor';

  @override
  String get renameProvider => 'Cambiar el nombre del proveedor';

  @override
  String get keyEnv => 'Variable de entorno de la clave API';

  @override
  String get keyHint => 'Opcional para servicios locales o Bedrock con AWS IAM';

  @override
  String get resourceRoot => 'URL raíz del recurso';

  @override
  String get modelIds => 'ID de modelos de origen, uno por línea';

  @override
  String get providerFallback =>
      'Modelos o grupos de respaldo, en orden, uno por línea';

  @override
  String get allowProviderFallback =>
      'Aceptar solicitudes de respaldo de otros proveedores';

  @override
  String protocolBase(String value) {
    return 'URL base de $value';
  }

  @override
  String get catalogUrl => 'URL del catálogo de modelos (opcional)';

  @override
  String get catalogHint =>
      'Deja en blanco para usar el catálogo predeterminado';

  @override
  String get readingModels => 'Cargando modelos';

  @override
  String get readModels => 'Cargar catálogo de modelos';

  @override
  String get requiredField => 'Este campo es obligatorio.';

  @override
  String get createGroup => 'Crear grupo de rutas';

  @override
  String get editGroup => 'Editar grupo de rutas';

  @override
  String get groupId => 'ID del grupo de rutas';

  @override
  String get raisePriority => 'Aumentar prioridad';

  @override
  String get removeMember => 'Quitar miembro';

  @override
  String get addCandidate => 'Añadir modelo candidato';

  @override
  String get connectRemote => 'Conectar a una puerta de enlace remota';

  @override
  String get remoteQuic => 'Dirección QUIC remota (con /p2p/PeerID)';

  @override
  String get localTunnel => 'Dirección de escucha local del túnel';

  @override
  String get tunnelHelp =>
      'Autoriza este ID de par en la puerta remota. Los clientes del túnel local usan la clave remota.';

  @override
  String get loopback => 'Dirección de bucle local HTTP';

  @override
  String get peerListen => 'Dirección de escucha P2P (vacía para desactivar)';

  @override
  String get allowedPeers => 'ID de pares permitidos, uno por línea';

  @override
  String get rotateMembers => 'Alternar el primer miembro';
}
