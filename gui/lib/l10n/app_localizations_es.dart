// ignore: unused_import
import 'package:intl/intl.dart' as intl;

import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for Spanish Castilian (`es`).
class AppLocalizationsEs extends AppLocalizations {
  AppLocalizationsEs([String locale = 'es']) : super(locale);

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
  String get keyEnv => 'Variable de entorno de la clave API';

  @override
  String get keyHint => 'Opcional para servicios locales o Bedrock con AWS IAM';

  @override
  String get resourceRoot => 'URL raíz del recurso';

  @override
  String get modelIds => 'ID de modelos de origen, uno por línea';

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
