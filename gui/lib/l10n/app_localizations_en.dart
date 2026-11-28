// ignore: unused_import
import 'package:intl/intl.dart' as intl;

import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for English (`en`).
class AppLocalizationsEn extends AppLocalizations {
  AppLocalizationsEn([String locale = 'en']) : super(locale);

  @override
  String get memoryTidy => 'Nightly memory tidy';

  @override
  String get memoryTidyHelp =>
      'When enabled, the gateway starts the tidy after 3:00 AM local time. It sends saved memories to the selected model route to find duplicates and stale notes. Handwritten and pinned memories are protected.';

  @override
  String get memoryTidyRun => 'Tidy now';

  @override
  String get memoryTidyUndo => 'Undo last tidy';

  @override
  String memoryTidyChanged(String count) {
    return 'Memories changed ($count)';
  }

  @override
  String memoryTidyRestored(String count) {
    return 'Changes undone ($count)';
  }

  @override
  String get interfaceFont => 'Interface font';

  @override
  String get importSlashCommands => 'Import slash commands';

  @override
  String get commandImportSource =>
      'Markdown or TOML file, local folder, or Git URL';

  @override
  String get commandChooseFile => 'Choose command file';

  @override
  String get projectHookAssignment => 'Edit project hook assignment';

  @override
  String get sharedSubagents => 'Shared subagents';

  @override
  String get importSubagents => 'Import subagents';

  @override
  String get subagentImportSource => 'Markdown file, local folder, or Git URL';

  @override
  String get subagentChooseFile => 'Choose Markdown file';

  @override
  String get subagentChooseFolder => 'Choose source folder';

  @override
  String get subagentScanSource => 'Scan source';

  @override
  String get subagentImportTruncated =>
      'Only part of this source was scanned. Choose a smaller folder to see more files.';

  @override
  String subagentImportStatus(String status) {
    String _temp0 = intl.Intl.selectLogic(status, {
      'update': 'Update available',
      'unchanged': 'Already imported',
      'edited': 'Local edits kept',
      'other': '',
    });
    return '$_temp0';
  }

  @override
  String get findNativeResources => 'Find existing files';

  @override
  String get noNativeResources => 'No existing files found';

  @override
  String get nativeResourceScanLimit => 'Showing the first 512 files found.';

  @override
  String get adoptResourceDetails =>
      'Crabforge will manage this file at its current location. Unassigning the client can remove the unchanged file.';

  @override
  String get agentSubagents => 'Subagents';

  @override
  String get agentCommands => 'Slash commands';

  @override
  String get addSubagent => 'Add subagent';

  @override
  String get addSlashCommand => 'Add slash command';

  @override
  String get resourceUserScope => 'User profiles';

  @override
  String get noAgentResources => 'No entries saved';

  @override
  String get resourcePlacements => 'Publication details';

  @override
  String get agentResourceName => 'Name';

  @override
  String get agentResourceDescription => 'Description';

  @override
  String get agentResourcePrompt => 'Prompt';

  @override
  String get resourceSkillNames => 'Skill names, one per line';

  @override
  String get resourceMcpNames => 'MCP server names, one per line';

  @override
  String get commandArgumentHelp =>
      'Use \$ARGUMENTS for all arguments and \$1, \$2, … for individual arguments in compatible clients.';

  @override
  String get agentHooks => 'Hooks';

  @override
  String get addHook => 'Add hook';

  @override
  String get editHook => 'Edit hook';

  @override
  String get adoptHook => 'Adopt hook';

  @override
  String get noHooks => 'No hooks saved';

  @override
  String get installedHooks => 'Hooks in client settings';

  @override
  String get removeInstalledHook => 'Remove installed hook';

  @override
  String get hookBackups => 'Hook backups';

  @override
  String get restoreHookBackup => 'Restore hook backup';

  @override
  String get hookBackupRestored => 'Restored';

  @override
  String get hookBackupChanged => 'Client settings have changed';

  @override
  String get hookBackupUnavailable => 'Original profile or file unavailable';

  @override
  String get hookName => 'Hook name';

  @override
  String get hookDescription => 'Description (optional)';

  @override
  String get hookEvent => 'Event';

  @override
  String get hookMatcher => 'Matcher';

  @override
  String get hookAllTools => 'All tools';

  @override
  String get hookShellTool => 'Shell commands';

  @override
  String get hookEditTool => 'File edits';

  @override
  String get hookCommand => 'Command run by the client';

  @override
  String get hookTimeout => 'Timeout in seconds (optional)';

  @override
  String get hookTurnEnd => 'End of turn';

  @override
  String get hookAfterTool => 'After tool use';

  @override
  String get hookBeforeTool => 'Before tool use';

  @override
  String get hookPrompt => 'Prompt submitted';

  @override
  String get hookSessionStart => 'Session start';

  @override
  String get hookExisting => 'Matching hook already exists';

  @override
  String get hookConflict => 'Existing hooks kept';

  @override
  String get agentEnvironment => 'Environment variables';

  @override
  String get noEnvironmentVariables => 'No variables saved';

  @override
  String get addVariable => 'Add variable';

  @override
  String get editVariable => 'Edit variable';

  @override
  String get variableName => 'Variable name';

  @override
  String get variableValue => 'Value';

  @override
  String get showVariableValue => 'Show value';

  @override
  String get hideVariableValue => 'Hide value';

  @override
  String get nativeVariableExists => 'Already set';

  @override
  String get nativeVariableConflict => 'Existing value kept';

  @override
  String get memoryLibrary => 'Memory';

  @override
  String get agentRules => 'Rules';

  @override
  String get agentSkills => 'Skills';

  @override
  String get importSkill => 'Import skill';

  @override
  String get applySkills => 'Apply skills';

  @override
  String get noSkills => 'No skills imported';

  @override
  String get editSkill => 'Edit skill assignments';

  @override
  String get skillSourceFolder => 'Local folder or Git URL';

  @override
  String get checkSkillUpdate => 'Check for updates';

  @override
  String get browseRepositorySkills => 'Browse source skills';

  @override
  String get noRepositorySkills => 'No additional skills found in this source.';

  @override
  String get dismissSkillCandidate => 'Set skill aside';

  @override
  String get skillUpToDate => 'The skill files are current.';

  @override
  String get skillUpdateAvailable =>
      'New skill files are available. Use Reimport files to apply them.';

  @override
  String get skillRemovedUpstream =>
      'The selected folder no longer contains SKILL.md.';

  @override
  String get installedSkills => 'Installed skills';

  @override
  String get adoptSkill => 'Manage in Crabforge';

  @override
  String get adoptSkillHint =>
      'Crabforge will save a copy of these files and manage this folder. Unassigning the skill can remove the unchanged folder.';

  @override
  String get mcpLibrary => 'MCP library';

  @override
  String get assignMcpServer => 'Assign to this project';

  @override
  String get unassignMcpServer => 'Unassign from this project';

  @override
  String get syncMcpLibrary => 'Sync assignments';

  @override
  String get refreshSkill => 'Reimport files';

  @override
  String get projectMcp => 'Project MCP';

  @override
  String get addMcpServer => 'Add MCP server';

  @override
  String get editMcpServer => 'Edit MCP server';

  @override
  String get noMcpServers => 'No MCP servers configured';

  @override
  String get mcpServerName => 'Server name';

  @override
  String get addRule => 'Add rule';

  @override
  String get editRule => 'Edit rule';

  @override
  String get syncRules => 'Apply rules';

  @override
  String get noRules => 'No rules saved';

  @override
  String get ruleName => 'Rule name';

  @override
  String get ruleBody => 'Markdown instructions';

  @override
  String get rulePaths => 'Path patterns (optional, one per line)';

  @override
  String get ruleRequired => 'This field is required.';

  @override
  String ruleStatus(String value) {
    String _temp0 = intl.Intl.selectLogic(value, {
      'applied': 'Applied',
      'pending': 'Pending',
      'removed': 'Removed',
      'conflict': 'Existing file kept',
      'other': 'Could not apply',
    });
    return '$_temp0';
  }

  @override
  String get agentSessions => 'Local sessions';

  @override
  String get allAgents => 'All agents';

  @override
  String get noAgentSessions => 'No local sessions found';

  @override
  String unreadableSessions(int count) {
    return 'Unreadable session files ($count)';
  }

  @override
  String get copySessionId => 'Copy session ID';

  @override
  String get sessionWorkingFolder => 'Working folder';

  @override
  String get sessionProfileFolder => 'Profile folder';

  @override
  String get distillTitle => 'Extract memories';

  @override
  String get distillPreview =>
      'Review the user messages below before sending them to the selected model route. New memories will be saved for review.';

  @override
  String get distillSend => 'Send to model';

  @override
  String get distillEmpty => 'No new memory candidates were found.';

  @override
  String get memorySourceQuote => 'Quote from the session';

  @override
  String get agentInstructions => 'User instructions';

  @override
  String get agentMemoryDigest => 'User memory';

  @override
  String get agentMemoryDigestHelp =>
      'Publishing writes global memory and this agent’s memory to its user instructions.';

  @override
  String get autoPublishAgentMemory => 'Publish memory daily';

  @override
  String get autoPublishAgentMemoryHelp =>
      'Refresh the digest of active memories in this file while the gateway runs.';

  @override
  String get agentMemoryDigestEmpty =>
      'There are no active global or agent memories to publish.';

  @override
  String get projectMemory => 'Project memory';

  @override
  String get publishMemory => 'Publish digest';

  @override
  String get projectMemoryHelp =>
      'Publishing writes the digest to this project’s instructions. Anyone with access to the file can read it.';

  @override
  String get projectMemoryEmpty =>
      'This project has no active memories to publish.';

  @override
  String get searchMemories => 'Search memories';

  @override
  String get unreadableMemories => 'Unreadable memory files';

  @override
  String get noMemoriesFound => 'No matching memories';

  @override
  String get addMemory => 'Add memory';

  @override
  String get editMemory => 'Edit memory';

  @override
  String get memoryScope => 'Scope';

  @override
  String get memoryTags => 'Tags, one per line';

  @override
  String get memoryWhy => 'Reason (optional)';

  @override
  String get memoryKindLabel => 'Kind';

  @override
  String get memoryStateLabel => 'Status';

  @override
  String get pinMemory => 'Pin memory';

  @override
  String memoryKind(String value) {
    String _temp0 = intl.Intl.selectLogic(value, {
      'profile': 'Profile',
      'preference': 'Preference',
      'rule': 'Rule',
      'reference': 'Reference',
      'fact': 'Fact',
      'procedure': 'Procedure',
      'episode': 'Episode',
      'other': 'Kind',
    });
    return '$_temp0';
  }

  @override
  String memoryState(String value) {
    String _temp0 = intl.Intl.selectLogic(value, {
      'active': 'Active',
      'candidate': 'Candidate',
      'archived': 'Archived',
      'superseded': 'Superseded',
      'other': 'All statuses',
    });
    return '$_temp0';
  }

  @override
  String get projectInstructions => 'Project instructions';

  @override
  String get projectFolder => 'Project folder';

  @override
  String get openProject => 'Open project';

  @override
  String get restoreInstructions => 'Restore previous version';

  @override
  String get relayFallback => 'Fallback relay circuit address (optional)';

  @override
  String get listView => 'List view';

  @override
  String get gridView => 'Grid view';

  @override
  String get gatewayProtocol => 'Gateway protocol';

  @override
  String get nativeModelProtocol => 'Use each model’s native protocol';

  @override
  String get roleModels => 'Role models';

  @override
  String get roleModelId => 'Model ID';

  @override
  String get resetRole => 'Reset role';

  @override
  String get syncModelCatalog => 'Sync model catalog';

  @override
  String get thinkingLevels => 'Thinking levels';

  @override
  String get sessionThinking => 'Session thinking level';

  @override
  String get thinkingAuto => 'Automatic';

  @override
  String get thinkingMinimal => 'Minimal';

  @override
  String get thinkingLow => 'Low';

  @override
  String get thinkingMedium => 'Medium';

  @override
  String get thinkingHigh => 'High';

  @override
  String get thinkingExtraHigh => 'Extra high';

  @override
  String get thinkingMax => 'Maximum';

  @override
  String get ompProfile => 'Oh My Pi profile';

  @override
  String get advisorSettings => 'Advisor settings';

  @override
  String get advisorReview => 'When to review';

  @override
  String get advisorWait => 'Pending reviews before waiting';

  @override
  String get advisorWake => 'Minimum note severity to resume';

  @override
  String get inheritSetting => 'Use profile default';

  @override
  String get everyTurn => 'Every turn';

  @override
  String get runEnd => 'At the end of the run';

  @override
  String get allReviews => 'Wait for all reviews';

  @override
  String get minorNote => 'Minor issue';

  @override
  String get concernNote => 'Concern';

  @override
  String get blockerNote => 'Blocker';

  @override
  String get memberPatterns => 'Model patterns, one per line';

  @override
  String get automaticDistillation => 'Automatic memory extraction';

  @override
  String get automaticDistillationHelp =>
      'While the gateway is running, Crabforge sends user messages from quiet sessions to the selected model route. Review the resulting candidates in the memory library.';

  @override
  String get distillationEnabled => 'Enabled';

  @override
  String get distillationLookback => 'Lookback in days';

  @override
  String get allSessions => 'All listed sessions';

  @override
  String get distillationRun => 'Read now';

  @override
  String get pluginBundles => 'Shared plugin library';

  @override
  String get piProjectPackages => 'Pi project packages';

  @override
  String get claudeProjectPackages => 'Claude Code project plugins';

  @override
  String get codexProjectPackages => 'Codex project plugins';

  @override
  String get antigravityProjectPackages => 'Antigravity project plugins';

  @override
  String get workspacePluginConfirm =>
      'The client can run code from the reviewed plugin when you open this trusted project.';

  @override
  String get claudePackageConfirm =>
      'Claude Code will install the reviewed plugin for this project. It can run plugin hooks when you open the project.';

  @override
  String get piPackageConfirm =>
      'Pi can run package extensions when you open a trusted project.';

  @override
  String get piExportAssignments => 'Export assignments';

  @override
  String get piApplyAssignments => 'Apply assignments';

  @override
  String get piAssignmentsTooLarge => 'The assignment file exceeds 1 MiB.';

  @override
  String piPackageStatus(String status) {
    String _temp0 = intl.Intl.selectLogic(status, {
      'available': 'Available to assign',
      'applied': 'Assigned',
      'pending': 'Ready to sync',
      'detached': 'Bundle removed from library',
      'conflict': 'Existing entry kept',
      'other': '',
    });
    return '$_temp0';
  }

  @override
  String get importPluginBundle => 'Import plugin bundle';

  @override
  String get previewPluginBundle => 'Preview bundle';

  @override
  String get pluginBundleSource => 'Plugin folder, manifest, or Git URL';

  @override
  String get noPluginBundles => 'No plugin bundles saved.';

  @override
  String get otherPluginEntries => 'Other manifest entries';

  @override
  String get noBundleResources => 'No resources in this file.';

  @override
  String get copyBundleResource => 'Copy to library';

  @override
  String get bundleResourceCopied => 'Copied to library';

  @override
  String get openPluginArchive => 'Open TAR archive';

  @override
  String get exportPluginArchive => 'Export plugin archive';

  @override
  String get pluginArchiveSaved => 'Plugin archive saved';

  @override
  String get pluginCatalog => 'Browse plugins';

  @override
  String get noAvailablePlugins =>
      'No plugins available from these marketplaces.';

  @override
  String get exportPluginSettings => 'Export plugin settings';

  @override
  String get importPluginSettings => 'Import plugin settings';

  @override
  String get claudePlugins => 'Claude Code plugins';

  @override
  String get pluginMarketplaces => 'Marketplaces';

  @override
  String get addPluginMarketplace => 'Add marketplace';

  @override
  String get pluginSource => 'Marketplace path or URL';

  @override
  String get installPlugin => 'Install plugin';

  @override
  String get updatePlugin => 'Update plugin';

  @override
  String get enablePlugin => 'Enable plugin';

  @override
  String get disablePlugin => 'Disable plugin';

  @override
  String get pluginEnabled => 'Enabled';

  @override
  String get noClaudePlugins =>
      'No Claude Code plugins installed for this scope.';

  @override
  String get shareGroup => 'Copy group to share';

  @override
  String get groupCopied => 'Group copied to clipboard';

  @override
  String get importGroup => 'Import group';

  @override
  String get sharedGroupText => 'Shared group text';

  @override
  String get replaceGroup => 'Replace an existing group with the same ID';

  @override
  String get missingMembers => 'Unavailable members';

  @override
  String get replacedMembers => 'Member substitutions';

  @override
  String routeSession(String id) {
    return 'Session $id';
  }

  @override
  String get enableGroup => 'Enable group';

  @override
  String get disableGroup => 'Disable group';

  @override
  String get listGroupModels => 'Show member models in client catalogs';

  @override
  String get copyGroup => 'Duplicate group';

  @override
  String get groupNote => 'Group note (optional)';

  @override
  String get firstContentSeconds => 'First content timeout (seconds)';

  @override
  String get groupFirstContentHelp =>
      'Use 0 to inherit or -1 to wait for the member.';

  @override
  String get defaultFirstContentHelp =>
      'Default for route groups. Use 0 or -1 to wait for each member.';

  @override
  String get invalidFirstContent =>
      'Enter a whole number from -1 to 2147483647.';

  @override
  String get groupPatienceSeconds => 'Retry patience (seconds)';

  @override
  String get groupPatienceHelp => 'Use 0 to inherit, or -1 to disable retries.';

  @override
  String get groupRoutingRules => 'Routing rules (JSON)';

  @override
  String get invalidGroupRoutingRules =>
      'Enter a JSON array of rules or a JSON object with a rules array.';

  @override
  String get classifyTimeoutSeconds => 'Classifier timeout (seconds)';

  @override
  String get classifyTimeoutHelp =>
      'Use 1 to 60 seconds, or 0 for the 15-second default.';

  @override
  String get defaultGroupPatienceHelp =>
      'Use 0 for 60 seconds, or -1 to disable retries.';

  @override
  String get invalidGroupPatience => 'Enter a whole number from -1 to 240.';

  @override
  String get enableMember => 'Enable member';

  @override
  String get disableMember => 'Disable member';

  @override
  String get manualRouting => 'Manual selection';

  @override
  String get selectMember => 'Select this member';

  @override
  String get providerCatalogs => 'models.dev catalog IDs (optional)';

  @override
  String get anthropicAuthHeader => 'Anthropic API key header';

  @override
  String get automaticAuthHeader => 'Automatic';

  @override
  String get autoGroups => 'Group matching models across providers';

  @override
  String get restoreGroup => 'Restore group';

  @override
  String get vertexProject => 'Vertex AI project settings';

  @override
  String get cloudProjectId => 'Google Cloud project ID';

  @override
  String get vertexLocation => 'Vertex AI location';

  @override
  String get googleCredentials => 'ADC file (optional)';

  @override
  String get impersonateAccount => 'Service account to impersonate (optional)';

  @override
  String get models => 'Models';

  @override
  String get modelCopied => 'Model ID copied';

  @override
  String get noModelsFound => 'No matching models';

  @override
  String get routingStrategy => 'Routing strategy';

  @override
  String get listedOrder => 'Listed order';

  @override
  String get weightedOrder => 'Weighted round robin';

  @override
  String get memberWeight => 'Member weight';

  @override
  String get invalidWeight => 'Enter a whole number from 1 to 1000.';

  @override
  String get providers => 'Providers';

  @override
  String get routes => 'Routes';

  @override
  String get groups => 'Route groups';

  @override
  String get gateway => 'Gateway';

  @override
  String get language => 'Language';

  @override
  String get systemLanguage => 'System language';

  @override
  String confirmRemove(String value) {
    return 'Remove $value?';
  }

  @override
  String get cancel => 'Cancel';

  @override
  String get remove => 'Remove';

  @override
  String get save => 'Save';

  @override
  String get webLogin => 'Web sign-in';

  @override
  String get loginHelp =>
      'Sign in in the new Chrome window, then return here to close it.';

  @override
  String get closeLogin => 'Close sign-in window';

  @override
  String get duplicateGroup => 'This group ID already exists.';

  @override
  String get addProvider => 'Add provider';

  @override
  String get newGroup => 'New group';

  @override
  String get stopToEdit => 'Stop the gateway to edit its configuration.';

  @override
  String get reload => 'Reload';

  @override
  String get reloadConfig => 'Reload configuration';

  @override
  String providerCount(int count) {
    return 'Providers: $count';
  }

  @override
  String presetCount(int count) {
    return 'Choose from $count API presets or enter a custom base URL.';
  }

  @override
  String get providerHelp =>
      'Add an upstream URL and model ID to enable client requests.';

  @override
  String get removeProvider => 'Remove provider';

  @override
  String get webProviders => 'Web providers';

  @override
  String get webHelp =>
      'Use Chrome and a matching ChromeDriver. Select an added provider to sign in with its own browser profile.';

  @override
  String webSite(String value) {
    return '$value Web';
  }

  @override
  String get groupNeedsProvider =>
      'Add a provider and model before creating a route group.';

  @override
  String get groupHelp =>
      'Add members in priority order. On rate limits or server errors, the gateway tries the next member.';

  @override
  String get removeGroup => 'Remove group';

  @override
  String recentAttempts(int count) {
    return 'Recent upstream attempts: $count';
  }

  @override
  String get historyLimit =>
      'Current gateway process · 128 most recent attempts';

  @override
  String get waiting => 'Waiting for a client request.';

  @override
  String get historyHelp => 'Start the gateway to see route history.';

  @override
  String get running => 'Gateway running';

  @override
  String get stopped => 'Gateway stopped';

  @override
  String get stop => 'Stop gateway';

  @override
  String get start => 'Start gateway';

  @override
  String get keyCopied => 'Gateway key copied';

  @override
  String get copyKey => 'Copy client key';

  @override
  String get networkSettings => 'Network settings';

  @override
  String get browserOrigins => 'Allowed browser origins (one per line)';

  @override
  String get localPeer => 'Local peer ID';

  @override
  String get copyPeer => 'Copy peer ID';

  @override
  String get peerCopied => 'Peer ID copied';

  @override
  String get listenAddress => 'Listen address';

  @override
  String get disabled => 'Disabled';

  @override
  String get trustedPeers => 'Trusted peers';

  @override
  String get clients => 'Client connections';

  @override
  String get agentRoot => 'Agent API root URL';

  @override
  String get copyRoot => 'Copy root URL';

  @override
  String get rootCopied => 'Root URL copied';

  @override
  String get agentHelp =>
      'Append /v1 for Chat Completions and Responses clients. Route history shows the selected agent.';

  @override
  String get tunnel => 'Remote gateway tunnel';

  @override
  String get disconnected => 'Disconnected';

  @override
  String get connect => 'Connect';

  @override
  String get disconnect => 'Disconnect';

  @override
  String get apiBase => 'API base URL';

  @override
  String get copyBase => 'Copy base URL';

  @override
  String get baseCopied => 'API base URL copied';

  @override
  String get changed => 'Provider settings changed. Fetch models again.';

  @override
  String get catalogRefreshInterval => 'Model directory refresh interval';

  @override
  String get catalogAutoInclude =>
      'Automatically include models matching these rules';

  @override
  String modelsRead(int count) {
    return 'Models found: $count. Edit the list before saving.';
  }

  @override
  String get presetLoaded =>
      'Preset models loaded. Adjust the list for your account.';

  @override
  String get editProvider => 'Edit provider';

  @override
  String get searchPresets => 'Search presets by name';

  @override
  String get providerId => 'Provider ID';

  @override
  String get renameProvider => 'Rename provider';

  @override
  String get keyEnv => 'API key environment variable';

  @override
  String get keyHint => 'Optional for local services or Bedrock with AWS IAM';

  @override
  String get resourceRoot => 'Resource root URL';

  @override
  String get modelIds => 'Upstream model IDs, one per line';

  @override
  String get providerFallback =>
      'Fallback models or groups, in order, one per line';

  @override
  String get allowProviderFallback =>
      'Accept fallback requests from other providers';

  @override
  String protocolBase(String value) {
    return '$value base URL';
  }

  @override
  String get catalogUrl => 'Model catalog URL (optional)';

  @override
  String get catalogHint => 'Leave blank for the provider’s default catalog';

  @override
  String get readingModels => 'Loading models';

  @override
  String get readModels => 'Load model catalog';

  @override
  String get requiredField => 'This field is required.';

  @override
  String get createGroup => 'Create route group';

  @override
  String get editGroup => 'Edit route group';

  @override
  String get groupId => 'Route group ID';

  @override
  String get raisePriority => 'Increase priority';

  @override
  String get removeMember => 'Remove member';

  @override
  String get addCandidate => 'Add candidate model';

  @override
  String get connectRemote => 'Connect to remote gateway';

  @override
  String get remoteQuic => 'Remote QUIC address (include /p2p/PeerID)';

  @override
  String get localTunnel => 'Local tunnel listen address';

  @override
  String get tunnelHelp =>
      'Trust this peer ID on the remote gateway. Clients connecting to the local tunnel use the remote gateway key.';

  @override
  String get loopback => 'HTTP loopback address';

  @override
  String get peerListen => 'P2P listen address (blank to disable)';

  @override
  String get allowedPeers => 'Allowed peer IDs, one per line';

  @override
  String get rotateMembers => 'Rotate the first member';
}
