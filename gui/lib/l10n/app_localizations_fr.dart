// ignore: unused_import
import 'package:intl/intl.dart' as intl;

import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for French (`fr`).
class AppLocalizationsFr extends AppLocalizations {
  AppLocalizationsFr([String locale = 'fr']) : super(locale);

  @override
  String get importSlashCommands => 'Importer des commandes slash';

  @override
  String get commandImportSource =>
      'Fichier Markdown ou TOML, dossier local ou URL Git';

  @override
  String get commandChooseFile => 'Choisir un fichier de commande';

  @override
  String get projectHookAssignment =>
      'Modifier l’affectation du hook au projet';

  @override
  String get sharedSubagents => 'Sous-agents partagés';

  @override
  String get importSubagents => 'Importer des sous-agents';

  @override
  String get subagentImportSource =>
      'Fichier Markdown, dossier local ou URL Git';

  @override
  String get subagentChooseFile => 'Choisir un fichier Markdown';

  @override
  String get subagentChooseFolder => 'Choisir un dossier source';

  @override
  String get subagentScanSource => 'Analyser la source';

  @override
  String get subagentImportTruncated =>
      'Seule une partie de la source a été analysée. Choisissez un dossier plus restreint pour voir plus de fichiers.';

  @override
  String subagentImportStatus(String status) {
    String _temp0 = intl.Intl.selectLogic(status, {
      'update': 'Mise à jour disponible',
      'unchanged': 'Déjà importé',
      'edited': 'Modifications locales conservées',
      'other': '',
    });
    return '$_temp0';
  }

  @override
  String get findNativeResources => 'Rechercher des fichiers existants';

  @override
  String get noNativeResources => 'Aucun fichier existant trouvé';

  @override
  String get nativeResourceScanLimit =>
      'Les 512 premiers fichiers trouvés sont affichés.';

  @override
  String get adoptResourceDetails =>
      'Crabforge gérera ce fichier à son emplacement actuel. Retirer l’affectation au client peut supprimer le fichier s’il est intact.';

  @override
  String get agentSubagents => 'Sous-agents';

  @override
  String get agentCommands => 'Commandes obliques';

  @override
  String get addSubagent => 'Ajouter un sous-agent';

  @override
  String get addSlashCommand => 'Ajouter une commande oblique';

  @override
  String get resourceUserScope => 'Profils utilisateur';

  @override
  String get noAgentResources => 'Aucune entrée enregistrée';

  @override
  String get resourcePlacements => 'Détails de publication';

  @override
  String get agentResourceName => 'Nom';

  @override
  String get agentResourceDescription => 'Description';

  @override
  String get agentResourcePrompt => 'Invite';

  @override
  String get resourceSkillNames => 'Noms des compétences, un par ligne';

  @override
  String get resourceMcpNames => 'Noms des serveurs MCP, un par ligne';

  @override
  String get commandArgumentHelp =>
      'Utilisez \$ARGUMENTS pour tous les arguments et \$1, \$2, etc. pour les arguments individuels dans les clients compatibles.';

  @override
  String get agentHooks => 'Hooks';

  @override
  String get addHook => 'Ajouter un hook';

  @override
  String get editHook => 'Modifier le hook';

  @override
  String get adoptHook => 'Adopter le hook';

  @override
  String get noHooks => 'Aucun hook enregistré';

  @override
  String get installedHooks => 'Hooks des paramètres clients';

  @override
  String get removeInstalledHook => 'Supprimer le hook installé';

  @override
  String get hookBackups => 'Sauvegardes des hooks';

  @override
  String get restoreHookBackup => 'Restaurer la sauvegarde du hook';

  @override
  String get hookBackupRestored => 'Restauré';

  @override
  String get hookBackupChanged => 'Les paramètres du client ont changé';

  @override
  String get hookBackupUnavailable =>
      'Profil ou fichier d’origine indisponible';

  @override
  String get hookName => 'Nom du hook';

  @override
  String get hookDescription => 'Description (facultative)';

  @override
  String get hookEvent => 'Événement';

  @override
  String get hookMatcher => 'Filtre';

  @override
  String get hookAllTools => 'Tous les outils';

  @override
  String get hookShellTool => 'Commandes shell';

  @override
  String get hookEditTool => 'Modifications de fichiers';

  @override
  String get hookCommand => 'Commande exécutée par le client';

  @override
  String get hookTimeout => 'Délai en secondes (facultatif)';

  @override
  String get hookTurnEnd => 'Fin du tour';

  @override
  String get hookAfterTool => 'Après l’appel d’un outil';

  @override
  String get hookBeforeTool => 'Avant l’appel d’un outil';

  @override
  String get hookPrompt => 'Envoi de la requête';

  @override
  String get hookSessionStart => 'Début de session';

  @override
  String get hookExisting => 'Un hook identique existe déjà';

  @override
  String get hookConflict => 'Hooks existants conservés';

  @override
  String get agentEnvironment => 'Variables d’environnement';

  @override
  String get noEnvironmentVariables => 'Aucune variable enregistrée';

  @override
  String get addVariable => 'Ajouter une variable';

  @override
  String get editVariable => 'Modifier la variable';

  @override
  String get variableName => 'Nom de la variable';

  @override
  String get variableValue => 'Valeur';

  @override
  String get showVariableValue => 'Afficher la valeur';

  @override
  String get hideVariableValue => 'Masquer la valeur';

  @override
  String get nativeVariableExists => 'Déjà définie';

  @override
  String get nativeVariableConflict => 'Valeur existante conservée';

  @override
  String get memoryLibrary => 'Mémoire';

  @override
  String get agentRules => 'Règles';

  @override
  String get agentSkills => 'Compétences';

  @override
  String get importSkill => 'Importer une compétence';

  @override
  String get applySkills => 'Appliquer les compétences';

  @override
  String get noSkills => 'Aucune compétence importée';

  @override
  String get editSkill => 'Modifier les agents associés';

  @override
  String get skillSourceFolder => 'Dossier local ou URL Git';

  @override
  String get checkSkillUpdate => 'Rechercher des mises à jour';

  @override
  String get browseRepositorySkills => 'Parcourir les compétences de la source';

  @override
  String get noRepositorySkills =>
      'Aucune autre compétence à importer depuis cette source.';

  @override
  String get dismissSkillCandidate => 'Mettre la compétence de côté';

  @override
  String get skillUpToDate => 'Les fichiers de la compétence sont à jour.';

  @override
  String get skillUpdateAvailable =>
      'De nouveaux fichiers sont disponibles. Réimportez-les pour les appliquer.';

  @override
  String get skillRemovedUpstream =>
      'Le dossier sélectionné ne contient plus de fichier SKILL.md.';

  @override
  String get installedSkills => 'Compétences installées';

  @override
  String get adoptSkill => 'Gérer dans Crabforge';

  @override
  String get adoptSkillHint =>
      'Crabforge conservera une copie des fichiers et gérera ce dossier. Le retrait de l’affectation peut supprimer le dossier s’il est intact.';

  @override
  String get mcpLibrary => 'Bibliothèque MCP';

  @override
  String get assignMcpServer => 'Assigner à ce projet';

  @override
  String get unassignMcpServer => 'Retirer l’affectation à ce projet';

  @override
  String get syncMcpLibrary => 'Synchroniser les affectations';

  @override
  String get refreshSkill => 'Réimporter les fichiers';

  @override
  String get projectMcp => 'MCP du projet';

  @override
  String get addMcpServer => 'Ajouter un serveur MCP';

  @override
  String get editMcpServer => 'Modifier le serveur MCP';

  @override
  String get noMcpServers => 'Aucun serveur MCP configuré';

  @override
  String get mcpServerName => 'Nom du serveur';

  @override
  String get addRule => 'Ajouter une règle';

  @override
  String get editRule => 'Modifier la règle';

  @override
  String get syncRules => 'Appliquer les règles';

  @override
  String get noRules => 'Aucune règle enregistrée';

  @override
  String get ruleName => 'Nom de la règle';

  @override
  String get ruleBody => 'Instructions Markdown';

  @override
  String get rulePaths => 'Motifs de chemin (facultatifs, un par ligne)';

  @override
  String get ruleRequired => 'Ce champ est obligatoire.';

  @override
  String ruleStatus(String value) {
    String _temp0 = intl.Intl.selectLogic(value, {
      'applied': 'Appliquée',
      'pending': 'En attente',
      'removed': 'Supprimée',
      'conflict': 'Fichier existant conservé',
      'other': 'Échec de l’application',
    });
    return '$_temp0';
  }

  @override
  String get agentSessions => 'Sessions locales';

  @override
  String get allAgents => 'Tous les agents';

  @override
  String get noAgentSessions => 'Aucune session locale trouvée';

  @override
  String unreadableSessions(int count) {
    return 'Fichiers de session illisibles ($count)';
  }

  @override
  String get copySessionId => 'Copier l’identifiant de session';

  @override
  String get sessionWorkingFolder => 'Dossier de travail';

  @override
  String get sessionProfileFolder => 'Dossier du profil';

  @override
  String get distillTitle => 'Extraire des souvenirs';

  @override
  String get distillPreview =>
      'Vérifiez les messages ci-dessous avant de les envoyer à la route du modèle sélectionné. Les nouveaux souvenirs seront enregistrés pour examen.';

  @override
  String get distillSend => 'Envoyer au modèle';

  @override
  String get distillEmpty => 'Aucun nouveau souvenir à examiner.';

  @override
  String get memorySourceQuote => 'Citation de la session';

  @override
  String get agentInstructions => 'Instructions utilisateur';

  @override
  String get agentMemoryDigest => 'Mémoire utilisateur';

  @override
  String get agentMemoryDigestHelp =>
      'La publication inscrit la mémoire globale et celle de cet agent dans ses instructions utilisateur.';

  @override
  String get agentMemoryDigestEmpty =>
      'Aucune entrée active de la mémoire globale ou de celle de cet agent à publier.';

  @override
  String get projectMemory => 'Mémoire du projet';

  @override
  String get publishMemory => 'Publier le résumé';

  @override
  String get projectMemoryHelp =>
      'Le résumé est écrit dans les instructions du projet. Toute personne ayant accès au fichier peut le lire.';

  @override
  String get projectMemoryEmpty =>
      'Ce projet n’a aucune mémoire active à publier.';

  @override
  String get searchMemories => 'Rechercher dans la mémoire';

  @override
  String get unreadableMemories => 'Fichiers de mémoire illisibles';

  @override
  String get noMemoriesFound => 'Aucun élément correspondant';

  @override
  String get addMemory => 'Ajouter à la mémoire';

  @override
  String get editMemory => 'Modifier la mémoire';

  @override
  String get memoryScope => 'Portée';

  @override
  String get memoryTags => 'Étiquettes, une par ligne';

  @override
  String get memoryWhy => 'Raison (facultatif)';

  @override
  String get memoryKindLabel => 'Type';

  @override
  String get memoryStateLabel => 'État';

  @override
  String get pinMemory => 'Épingler';

  @override
  String memoryKind(String value) {
    String _temp0 = intl.Intl.selectLogic(value, {
      'profile': 'Profil',
      'preference': 'Préférence',
      'rule': 'Règle',
      'reference': 'Référence',
      'fact': 'Fait',
      'procedure': 'Procédure',
      'episode': 'Événement',
      'other': 'Type',
    });
    return '$_temp0';
  }

  @override
  String memoryState(String value) {
    String _temp0 = intl.Intl.selectLogic(value, {
      'active': 'Actif',
      'candidate': 'À examiner',
      'archived': 'Archivé',
      'superseded': 'Remplacé',
      'other': 'Tous les états',
    });
    return '$_temp0';
  }

  @override
  String get projectInstructions => 'Instructions du projet';

  @override
  String get projectFolder => 'Dossier du projet';

  @override
  String get openProject => 'Ouvrir le projet';

  @override
  String get restoreInstructions => 'Restaurer la version précédente';

  @override
  String get relayFallback =>
      'Adresse du circuit relais de secours (facultatif)';

  @override
  String get listView => 'Vue en liste';

  @override
  String get gridView => 'Vue en grille';

  @override
  String get gatewayProtocol => 'Protocole de la passerelle';

  @override
  String get nativeModelProtocol =>
      'Utiliser le protocole natif de chaque modèle';

  @override
  String get roleModels => 'Modèles par rôle';

  @override
  String get roleModelId => 'Identifiant du modèle';

  @override
  String get resetRole => 'Réinitialiser le rôle';

  @override
  String get syncModelCatalog => 'Synchroniser le catalogue de modèles';

  @override
  String get thinkingLevels => 'Niveaux de réflexion';

  @override
  String get sessionThinking => 'Niveau de réflexion de la session';

  @override
  String get thinkingAuto => 'Automatique';

  @override
  String get thinkingMinimal => 'Minimal';

  @override
  String get thinkingLow => 'Faible';

  @override
  String get thinkingMedium => 'Moyen';

  @override
  String get thinkingHigh => 'Élevé';

  @override
  String get thinkingExtraHigh => 'Très élevé';

  @override
  String get thinkingMax => 'Maximal';

  @override
  String get ompProfile => 'Profil Oh My Pi';

  @override
  String get advisorSettings => 'Paramètres du conseiller';

  @override
  String get advisorReview => 'Moment de la révision';

  @override
  String get advisorWait => 'Révisions en attente avant la pause';

  @override
  String get advisorWake => 'Gravité minimale pour reprendre';

  @override
  String get inheritSetting => 'Utiliser la valeur par défaut';

  @override
  String get everyTurn => 'À chaque tour';

  @override
  String get runEnd => 'En fin d’exécution';

  @override
  String get allReviews => 'Attendre toutes les révisions';

  @override
  String get minorNote => 'Détail mineur';

  @override
  String get concernNote => 'Point préoccupant';

  @override
  String get blockerNote => 'Problème bloquant';

  @override
  String get memberPatterns => 'Motifs de modèles, un par ligne';

  @override
  String get pluginBundles => 'Bibliothèque d’extensions partagée';

  @override
  String get importPluginBundle => 'Importer un paquet d’extension';

  @override
  String get previewPluginBundle => 'Prévisualiser le paquet';

  @override
  String get pluginBundleSource => 'Dossier, manifeste ou URL Git';

  @override
  String get noPluginBundles => 'Aucun paquet d’extension enregistré.';

  @override
  String get otherPluginEntries => 'Autres entrées du manifeste';

  @override
  String get noBundleResources => 'Ce fichier ne contient aucune ressource.';

  @override
  String get copyBundleResource => 'Copier dans la bibliothèque';

  @override
  String get bundleResourceCopied => 'Copié dans la bibliothèque';

  @override
  String get openPluginArchive => 'Ouvrir une archive TAR';

  @override
  String get exportPluginArchive => 'Exporter l’archive de l’extension';

  @override
  String get pluginArchiveSaved => 'Archive de l’extension enregistrée';

  @override
  String get pluginCatalog => 'Parcourir les extensions';

  @override
  String get noAvailablePlugins =>
      'Aucune extension disponible dans ces catalogues.';

  @override
  String get exportPluginSettings => 'Exporter les paramètres des extensions';

  @override
  String get importPluginSettings => 'Importer les paramètres des extensions';

  @override
  String get claudePlugins => 'Extensions Claude Code';

  @override
  String get pluginMarketplaces => 'Catalogues d’extensions';

  @override
  String get addPluginMarketplace => 'Ajouter un catalogue';

  @override
  String get pluginSource => 'Chemin ou URL du catalogue';

  @override
  String get installPlugin => 'Installer l’extension';

  @override
  String get updatePlugin => 'Mettre à jour l’extension';

  @override
  String get enablePlugin => 'Activer l’extension';

  @override
  String get disablePlugin => 'Désactiver l’extension';

  @override
  String get pluginEnabled => 'Activée';

  @override
  String get noClaudePlugins =>
      'Aucune extension Claude Code installée pour cette portée.';

  @override
  String get shareGroup => 'Copier le groupe pour le partager';

  @override
  String get groupCopied => 'Groupe copié dans le presse-papiers';

  @override
  String get importGroup => 'Importer un groupe';

  @override
  String get sharedGroupText => 'Texte du groupe partagé';

  @override
  String get replaceGroup =>
      'Remplacer le groupe existant ayant le même identifiant';

  @override
  String get missingMembers => 'Membres indisponibles';

  @override
  String get replacedMembers => 'Membres remplacés';

  @override
  String routeSession(String id) {
    return 'Session $id';
  }

  @override
  String get enableGroup => 'Activer le groupe';

  @override
  String get disableGroup => 'Désactiver le groupe';

  @override
  String get listGroupModels =>
      'Afficher les modèles du groupe dans les catalogues des clients';

  @override
  String get copyGroup => 'Dupliquer le groupe';

  @override
  String get groupNote => 'Note du groupe (facultatif)';

  @override
  String get enableMember => 'Activer le membre';

  @override
  String get disableMember => 'Désactiver le membre';

  @override
  String get manualRouting => 'Sélection manuelle';

  @override
  String get selectMember => 'Sélectionner ce membre';

  @override
  String get providerCatalogs =>
      'Identifiants de catalogue models.dev (facultatif)';

  @override
  String get anthropicAuthHeader => 'En-tête de clé API Anthropic';

  @override
  String get automaticAuthHeader => 'Automatique';

  @override
  String get autoGroups =>
      'Regrouper les modèles identiques de plusieurs fournisseurs';

  @override
  String get restoreGroup => 'Restaurer le groupe';

  @override
  String get vertexProject => 'Paramètres du projet Vertex AI';

  @override
  String get cloudProjectId => 'ID du projet Google Cloud';

  @override
  String get vertexLocation => 'Région Vertex AI';

  @override
  String get googleCredentials => 'Fichier ADC (facultatif)';

  @override
  String get impersonateAccount => 'Compte de service à emprunter (facultatif)';

  @override
  String get models => 'Modèles';

  @override
  String get modelCopied => 'Identifiant du modèle copié';

  @override
  String get noModelsFound => 'Aucun modèle correspondant';

  @override
  String get routingStrategy => 'Stratégie de routage';

  @override
  String get listedOrder => 'Ordre de la liste';

  @override
  String get weightedOrder => 'Rotation pondérée';

  @override
  String get memberWeight => 'Poids du membre';

  @override
  String get invalidWeight => 'Saisissez un entier de 1 à 1000.';

  @override
  String get providers => 'Fournisseurs';

  @override
  String get routes => 'Routes';

  @override
  String get groups => 'Groupes de routes';

  @override
  String get gateway => 'Passerelle';

  @override
  String get language => 'Langue';

  @override
  String get systemLanguage => 'Langue du système';

  @override
  String confirmRemove(String value) {
    return 'Supprimer $value ?';
  }

  @override
  String get cancel => 'Annuler';

  @override
  String get remove => 'Supprimer';

  @override
  String get save => 'Enregistrer';

  @override
  String get webLogin => 'Connexion web';

  @override
  String get loginHelp =>
      'Connectez-vous dans la nouvelle fenêtre Chrome, puis revenez ici pour la fermer.';

  @override
  String get closeLogin => 'Fermer la fenêtre de connexion';

  @override
  String get duplicateGroup => 'Cet identifiant de groupe existe déjà.';

  @override
  String get addProvider => 'Ajouter un fournisseur';

  @override
  String get newGroup => 'Nouveau groupe';

  @override
  String get stopToEdit =>
      'Arrêtez la passerelle pour modifier sa configuration.';

  @override
  String get reload => 'Recharger';

  @override
  String get reloadConfig => 'Recharger la configuration';

  @override
  String providerCount(int count) {
    return 'Fournisseurs : $count';
  }

  @override
  String presetCount(int count) {
    return 'Choisissez parmi $count préréglages API ou indiquez une URL de base.';
  }

  @override
  String get providerHelp =>
      'Ajoutez une URL en amont et un ID de modèle pour accepter les requêtes.';

  @override
  String get removeProvider => 'Supprimer le fournisseur';

  @override
  String get webProviders => 'Fournisseurs web';

  @override
  String get webHelp =>
      'Utilisez Chrome avec la version correspondante de ChromeDriver. Sélectionnez le fournisseur ajouté pour vous connecter dans son profil dédié.';

  @override
  String webSite(String value) {
    return '$value Web';
  }

  @override
  String get groupNeedsProvider =>
      'Ajoutez un fournisseur et un modèle avant de créer un groupe de routes.';

  @override
  String get groupHelp =>
      'Ajoutez les membres par priorité. En cas de limitation ou de panne, la passerelle essaie le suivant.';

  @override
  String get removeGroup => 'Supprimer le groupe';

  @override
  String recentAttempts(int count) {
    return 'Tentatives récentes en amont : $count';
  }

  @override
  String get historyLimit => 'Processus actuel · 128 dernières tentatives';

  @override
  String get waiting => 'En attente d’une requête client.';

  @override
  String get historyHelp =>
      'Démarrez la passerelle pour afficher l’historique.';

  @override
  String get running => 'Passerelle en service';

  @override
  String get stopped => 'Passerelle arrêtée';

  @override
  String get stop => 'Arrêter la passerelle';

  @override
  String get start => 'Démarrer la passerelle';

  @override
  String get keyCopied => 'Clé de la passerelle copiée';

  @override
  String get copyKey => 'Copier la clé client';

  @override
  String get networkSettings => 'Paramètres réseau';

  @override
  String get browserOrigins => 'Origines web autorisées (une par ligne)';

  @override
  String get localPeer => 'ID du pair local';

  @override
  String get copyPeer => 'Copier l’ID du pair';

  @override
  String get peerCopied => 'ID du pair copié';

  @override
  String get listenAddress => 'Adresse d’écoute';

  @override
  String get disabled => 'Désactivé';

  @override
  String get trustedPeers => 'Pairs de confiance';

  @override
  String get clients => 'Connexions client';

  @override
  String get agentRoot => 'URL racine de l’API de l’agent';

  @override
  String get copyRoot => 'Copier l’URL racine';

  @override
  String get rootCopied => 'URL racine copiée';

  @override
  String get agentHelp =>
      'Ajoutez /v1 pour les clients Chat Completions et Responses. L’historique affiche l’agent choisi.';

  @override
  String get tunnel => 'Tunnel vers la passerelle distante';

  @override
  String get disconnected => 'Déconnecté';

  @override
  String get connect => 'Connecter';

  @override
  String get disconnect => 'Déconnecter';

  @override
  String get apiBase => 'URL de base de l’API';

  @override
  String get copyBase => 'Copier l’URL de base';

  @override
  String get baseCopied => 'URL de base de l’API copiée';

  @override
  String get changed => 'La configuration a changé. Rechargez les modèles.';

  @override
  String modelsRead(int count) {
    return 'Modèles trouvés : $count. Vous pouvez modifier la liste avant de l’enregistrer.';
  }

  @override
  String get presetLoaded =>
      'Modèles du préréglage chargés. Adaptez la liste à votre compte.';

  @override
  String get editProvider => 'Modifier le fournisseur';

  @override
  String get searchPresets => 'Rechercher un préréglage par nom';

  @override
  String get providerId => 'ID du fournisseur';

  @override
  String get renameProvider => 'Renommer le fournisseur';

  @override
  String get keyEnv => 'Variable d’environnement de la clé API';

  @override
  String get keyHint =>
      'Facultatif pour les services locaux ou Bedrock avec AWS IAM';

  @override
  String get resourceRoot => 'URL racine de la ressource';

  @override
  String get modelIds => 'ID des modèles en amont, un par ligne';

  @override
  String get providerFallback =>
      'Modèles ou groupes de secours, dans l’ordre, un par ligne';

  @override
  String get allowProviderFallback =>
      'Accepter les requêtes de secours d’autres fournisseurs';

  @override
  String protocolBase(String value) {
    return 'URL de base de $value';
  }

  @override
  String get catalogUrl => 'URL du catalogue de modèles (facultative)';

  @override
  String get catalogHint =>
      'Laissez vide pour utiliser le catalogue par défaut';

  @override
  String get readingModels => 'Chargement des modèles';

  @override
  String get readModels => 'Charger le catalogue';

  @override
  String get requiredField => 'Ce champ est obligatoire.';

  @override
  String get createGroup => 'Créer un groupe de routes';

  @override
  String get editGroup => 'Modifier le groupe de routes';

  @override
  String get groupId => 'ID du groupe de routes';

  @override
  String get raisePriority => 'Augmenter la priorité';

  @override
  String get removeMember => 'Supprimer le membre';

  @override
  String get addCandidate => 'Ajouter un modèle candidat';

  @override
  String get connectRemote => 'Se connecter à une passerelle distante';

  @override
  String get remoteQuic => 'Adresse QUIC distante (avec /p2p/PeerID)';

  @override
  String get localTunnel => 'Adresse d’écoute locale du tunnel';

  @override
  String get tunnelHelp =>
      'Autorisez cet ID de pair sur la passerelle distante. Les clients du tunnel local utilisent la clé distante.';

  @override
  String get loopback => 'Adresse de bouclage HTTP';

  @override
  String get peerListen => 'Adresse d’écoute P2P (vide pour désactiver)';

  @override
  String get allowedPeers => 'ID des pairs autorisés, un par ligne';

  @override
  String get rotateMembers => 'Alterner le premier membre';
}
