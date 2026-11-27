import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:intl/intl.dart' as intl;

import 'app_localizations_de.dart';
import 'app_localizations_en.dart';
import 'app_localizations_es.dart';
import 'app_localizations_fr.dart';
import 'app_localizations_hi.dart';
import 'app_localizations_ja.dart';
import 'app_localizations_ko.dart';
import 'app_localizations_pt.dart';
import 'app_localizations_zh.dart';

// ignore_for_file: type=lint

/// Callers can lookup localized strings with an instance of AppLocalizations
/// returned by `AppLocalizations.of(context)`.
///
/// Applications need to include `AppLocalizations.delegate()` in their app's
/// `localizationDelegates` list, and the locales they support in the app's
/// `supportedLocales` list. For example:
///
/// ```dart
/// import 'l10n/app_localizations.dart';
///
/// return MaterialApp(
///   localizationsDelegates: AppLocalizations.localizationsDelegates,
///   supportedLocales: AppLocalizations.supportedLocales,
///   home: MyApplicationHome(),
/// );
/// ```
///
/// ## Update pubspec.yaml
///
/// Please make sure to update your pubspec.yaml to include the following
/// packages:
///
/// ```yaml
/// dependencies:
///   # Internationalization support.
///   flutter_localizations:
///     sdk: flutter
///   intl: any # Use the pinned version from flutter_localizations
///
///   # Rest of dependencies
/// ```
///
/// ## iOS Applications
///
/// iOS applications define key application metadata, including supported
/// locales, in an Info.plist file that is built into the application bundle.
/// To configure the locales supported by your app, you’ll need to edit this
/// file.
///
/// First, open your project’s ios/Runner.xcworkspace Xcode workspace file.
/// Then, in the Project Navigator, open the Info.plist file under the Runner
/// project’s Runner folder.
///
/// Next, select the Information Property List item, select Add Item from the
/// Editor menu, then select Localizations from the pop-up menu.
///
/// Select and expand the newly-created Localizations item then, for each
/// locale your application supports, add a new item and select the locale
/// you wish to add from the pop-up menu in the Value field. This list should
/// be consistent with the languages listed in the AppLocalizations.supportedLocales
/// property.
abstract class AppLocalizations {
  AppLocalizations(String locale)
    : localeName = intl.Intl.canonicalizedLocale(locale.toString());

  final String localeName;

  static AppLocalizations of(BuildContext context) {
    return Localizations.of<AppLocalizations>(context, AppLocalizations)!;
  }

  static const LocalizationsDelegate<AppLocalizations> delegate =
      _AppLocalizationsDelegate();

  /// A list of this localizations delegate along with the default localizations
  /// delegates.
  ///
  /// Returns a list of localizations delegates containing this delegate along with
  /// GlobalMaterialLocalizations.delegate, GlobalCupertinoLocalizations.delegate,
  /// and GlobalWidgetsLocalizations.delegate.
  ///
  /// Additional delegates can be added by appending to this list in
  /// MaterialApp. This list does not have to be used at all if a custom list
  /// of delegates is preferred or required.
  static const List<LocalizationsDelegate<dynamic>> localizationsDelegates =
      <LocalizationsDelegate<dynamic>>[
        delegate,
        GlobalMaterialLocalizations.delegate,
        GlobalCupertinoLocalizations.delegate,
        GlobalWidgetsLocalizations.delegate,
      ];

  /// A list of this localizations delegate's supported locales.
  static const List<Locale> supportedLocales = <Locale>[
    Locale('de'),
    Locale('en'),
    Locale('es'),
    Locale('fr'),
    Locale('hi'),
    Locale('ja'),
    Locale('ko'),
    Locale('pt'),
    Locale('zh'),
  ];

  /// No description provided for @importSlashCommands.
  ///
  /// In en, this message translates to:
  /// **'Import slash commands'**
  String get importSlashCommands;

  /// No description provided for @commandImportSource.
  ///
  /// In en, this message translates to:
  /// **'Markdown or TOML file, local folder, or Git URL'**
  String get commandImportSource;

  /// No description provided for @commandChooseFile.
  ///
  /// In en, this message translates to:
  /// **'Choose command file'**
  String get commandChooseFile;

  /// No description provided for @projectHookAssignment.
  ///
  /// In en, this message translates to:
  /// **'Edit project hook assignment'**
  String get projectHookAssignment;

  /// No description provided for @sharedSubagents.
  ///
  /// In en, this message translates to:
  /// **'Shared subagents'**
  String get sharedSubagents;

  /// No description provided for @importSubagents.
  ///
  /// In en, this message translates to:
  /// **'Import subagents'**
  String get importSubagents;

  /// No description provided for @subagentImportSource.
  ///
  /// In en, this message translates to:
  /// **'Markdown file, local folder, or Git URL'**
  String get subagentImportSource;

  /// No description provided for @subagentChooseFile.
  ///
  /// In en, this message translates to:
  /// **'Choose Markdown file'**
  String get subagentChooseFile;

  /// No description provided for @subagentChooseFolder.
  ///
  /// In en, this message translates to:
  /// **'Choose source folder'**
  String get subagentChooseFolder;

  /// No description provided for @subagentScanSource.
  ///
  /// In en, this message translates to:
  /// **'Scan source'**
  String get subagentScanSource;

  /// No description provided for @subagentImportTruncated.
  ///
  /// In en, this message translates to:
  /// **'Only part of this source was scanned. Choose a smaller folder to see more files.'**
  String get subagentImportTruncated;

  /// No description provided for @subagentImportStatus.
  ///
  /// In en, this message translates to:
  /// **'{status, select, update{Update available} unchanged{Already imported} edited{Local edits kept} other{}}'**
  String subagentImportStatus(String status);

  /// No description provided for @findNativeResources.
  ///
  /// In en, this message translates to:
  /// **'Find existing files'**
  String get findNativeResources;

  /// No description provided for @noNativeResources.
  ///
  /// In en, this message translates to:
  /// **'No existing files found'**
  String get noNativeResources;

  /// No description provided for @nativeResourceScanLimit.
  ///
  /// In en, this message translates to:
  /// **'Showing the first 512 files found.'**
  String get nativeResourceScanLimit;

  /// No description provided for @adoptResourceDetails.
  ///
  /// In en, this message translates to:
  /// **'Crabforge will manage this file at its current location. Unassigning the client can remove the unchanged file.'**
  String get adoptResourceDetails;

  /// No description provided for @agentSubagents.
  ///
  /// In en, this message translates to:
  /// **'Subagents'**
  String get agentSubagents;

  /// No description provided for @agentCommands.
  ///
  /// In en, this message translates to:
  /// **'Slash commands'**
  String get agentCommands;

  /// No description provided for @addSubagent.
  ///
  /// In en, this message translates to:
  /// **'Add subagent'**
  String get addSubagent;

  /// No description provided for @addSlashCommand.
  ///
  /// In en, this message translates to:
  /// **'Add slash command'**
  String get addSlashCommand;

  /// No description provided for @resourceUserScope.
  ///
  /// In en, this message translates to:
  /// **'User profiles'**
  String get resourceUserScope;

  /// No description provided for @noAgentResources.
  ///
  /// In en, this message translates to:
  /// **'No entries saved'**
  String get noAgentResources;

  /// No description provided for @resourcePlacements.
  ///
  /// In en, this message translates to:
  /// **'Publication details'**
  String get resourcePlacements;

  /// No description provided for @agentResourceName.
  ///
  /// In en, this message translates to:
  /// **'Name'**
  String get agentResourceName;

  /// No description provided for @agentResourceDescription.
  ///
  /// In en, this message translates to:
  /// **'Description'**
  String get agentResourceDescription;

  /// No description provided for @agentResourcePrompt.
  ///
  /// In en, this message translates to:
  /// **'Prompt'**
  String get agentResourcePrompt;

  /// No description provided for @resourceSkillNames.
  ///
  /// In en, this message translates to:
  /// **'Skill names, one per line'**
  String get resourceSkillNames;

  /// No description provided for @resourceMcpNames.
  ///
  /// In en, this message translates to:
  /// **'MCP server names, one per line'**
  String get resourceMcpNames;

  /// No description provided for @commandArgumentHelp.
  ///
  /// In en, this message translates to:
  /// **'Use \$ARGUMENTS for all arguments and \$1, \$2, … for individual arguments in compatible clients.'**
  String get commandArgumentHelp;

  /// No description provided for @agentHooks.
  ///
  /// In en, this message translates to:
  /// **'Hooks'**
  String get agentHooks;

  /// No description provided for @addHook.
  ///
  /// In en, this message translates to:
  /// **'Add hook'**
  String get addHook;

  /// No description provided for @editHook.
  ///
  /// In en, this message translates to:
  /// **'Edit hook'**
  String get editHook;

  /// No description provided for @adoptHook.
  ///
  /// In en, this message translates to:
  /// **'Adopt hook'**
  String get adoptHook;

  /// No description provided for @noHooks.
  ///
  /// In en, this message translates to:
  /// **'No hooks saved'**
  String get noHooks;

  /// No description provided for @installedHooks.
  ///
  /// In en, this message translates to:
  /// **'Hooks in client settings'**
  String get installedHooks;

  /// No description provided for @removeInstalledHook.
  ///
  /// In en, this message translates to:
  /// **'Remove installed hook'**
  String get removeInstalledHook;

  /// No description provided for @hookBackups.
  ///
  /// In en, this message translates to:
  /// **'Hook backups'**
  String get hookBackups;

  /// No description provided for @restoreHookBackup.
  ///
  /// In en, this message translates to:
  /// **'Restore hook backup'**
  String get restoreHookBackup;

  /// No description provided for @hookBackupRestored.
  ///
  /// In en, this message translates to:
  /// **'Restored'**
  String get hookBackupRestored;

  /// No description provided for @hookBackupChanged.
  ///
  /// In en, this message translates to:
  /// **'Client settings have changed'**
  String get hookBackupChanged;

  /// No description provided for @hookBackupUnavailable.
  ///
  /// In en, this message translates to:
  /// **'Original profile or file unavailable'**
  String get hookBackupUnavailable;

  /// No description provided for @hookName.
  ///
  /// In en, this message translates to:
  /// **'Hook name'**
  String get hookName;

  /// No description provided for @hookDescription.
  ///
  /// In en, this message translates to:
  /// **'Description (optional)'**
  String get hookDescription;

  /// No description provided for @hookEvent.
  ///
  /// In en, this message translates to:
  /// **'Event'**
  String get hookEvent;

  /// No description provided for @hookMatcher.
  ///
  /// In en, this message translates to:
  /// **'Matcher'**
  String get hookMatcher;

  /// No description provided for @hookAllTools.
  ///
  /// In en, this message translates to:
  /// **'All tools'**
  String get hookAllTools;

  /// No description provided for @hookShellTool.
  ///
  /// In en, this message translates to:
  /// **'Shell commands'**
  String get hookShellTool;

  /// No description provided for @hookEditTool.
  ///
  /// In en, this message translates to:
  /// **'File edits'**
  String get hookEditTool;

  /// No description provided for @hookCommand.
  ///
  /// In en, this message translates to:
  /// **'Command run by the client'**
  String get hookCommand;

  /// No description provided for @hookTimeout.
  ///
  /// In en, this message translates to:
  /// **'Timeout in seconds (optional)'**
  String get hookTimeout;

  /// No description provided for @hookTurnEnd.
  ///
  /// In en, this message translates to:
  /// **'End of turn'**
  String get hookTurnEnd;

  /// No description provided for @hookAfterTool.
  ///
  /// In en, this message translates to:
  /// **'After tool use'**
  String get hookAfterTool;

  /// No description provided for @hookBeforeTool.
  ///
  /// In en, this message translates to:
  /// **'Before tool use'**
  String get hookBeforeTool;

  /// No description provided for @hookPrompt.
  ///
  /// In en, this message translates to:
  /// **'Prompt submitted'**
  String get hookPrompt;

  /// No description provided for @hookSessionStart.
  ///
  /// In en, this message translates to:
  /// **'Session start'**
  String get hookSessionStart;

  /// No description provided for @hookExisting.
  ///
  /// In en, this message translates to:
  /// **'Matching hook already exists'**
  String get hookExisting;

  /// No description provided for @hookConflict.
  ///
  /// In en, this message translates to:
  /// **'Existing hooks kept'**
  String get hookConflict;

  /// No description provided for @agentEnvironment.
  ///
  /// In en, this message translates to:
  /// **'Environment variables'**
  String get agentEnvironment;

  /// No description provided for @noEnvironmentVariables.
  ///
  /// In en, this message translates to:
  /// **'No variables saved'**
  String get noEnvironmentVariables;

  /// No description provided for @addVariable.
  ///
  /// In en, this message translates to:
  /// **'Add variable'**
  String get addVariable;

  /// No description provided for @editVariable.
  ///
  /// In en, this message translates to:
  /// **'Edit variable'**
  String get editVariable;

  /// No description provided for @variableName.
  ///
  /// In en, this message translates to:
  /// **'Variable name'**
  String get variableName;

  /// No description provided for @variableValue.
  ///
  /// In en, this message translates to:
  /// **'Value'**
  String get variableValue;

  /// No description provided for @showVariableValue.
  ///
  /// In en, this message translates to:
  /// **'Show value'**
  String get showVariableValue;

  /// No description provided for @hideVariableValue.
  ///
  /// In en, this message translates to:
  /// **'Hide value'**
  String get hideVariableValue;

  /// No description provided for @nativeVariableExists.
  ///
  /// In en, this message translates to:
  /// **'Already set'**
  String get nativeVariableExists;

  /// No description provided for @nativeVariableConflict.
  ///
  /// In en, this message translates to:
  /// **'Existing value kept'**
  String get nativeVariableConflict;

  /// No description provided for @memoryLibrary.
  ///
  /// In en, this message translates to:
  /// **'Memory'**
  String get memoryLibrary;

  /// No description provided for @agentRules.
  ///
  /// In en, this message translates to:
  /// **'Rules'**
  String get agentRules;

  /// No description provided for @agentSkills.
  ///
  /// In en, this message translates to:
  /// **'Skills'**
  String get agentSkills;

  /// No description provided for @importSkill.
  ///
  /// In en, this message translates to:
  /// **'Import skill'**
  String get importSkill;

  /// No description provided for @applySkills.
  ///
  /// In en, this message translates to:
  /// **'Apply skills'**
  String get applySkills;

  /// No description provided for @noSkills.
  ///
  /// In en, this message translates to:
  /// **'No skills imported'**
  String get noSkills;

  /// No description provided for @editSkill.
  ///
  /// In en, this message translates to:
  /// **'Edit skill assignments'**
  String get editSkill;

  /// No description provided for @skillSourceFolder.
  ///
  /// In en, this message translates to:
  /// **'Local folder or Git URL'**
  String get skillSourceFolder;

  /// No description provided for @checkSkillUpdate.
  ///
  /// In en, this message translates to:
  /// **'Check for updates'**
  String get checkSkillUpdate;

  /// No description provided for @browseRepositorySkills.
  ///
  /// In en, this message translates to:
  /// **'Browse source skills'**
  String get browseRepositorySkills;

  /// No description provided for @noRepositorySkills.
  ///
  /// In en, this message translates to:
  /// **'No additional skills found in this source.'**
  String get noRepositorySkills;

  /// No description provided for @dismissSkillCandidate.
  ///
  /// In en, this message translates to:
  /// **'Set skill aside'**
  String get dismissSkillCandidate;

  /// No description provided for @skillUpToDate.
  ///
  /// In en, this message translates to:
  /// **'The skill files are current.'**
  String get skillUpToDate;

  /// No description provided for @skillUpdateAvailable.
  ///
  /// In en, this message translates to:
  /// **'New skill files are available. Use Reimport files to apply them.'**
  String get skillUpdateAvailable;

  /// No description provided for @skillRemovedUpstream.
  ///
  /// In en, this message translates to:
  /// **'The selected folder no longer contains SKILL.md.'**
  String get skillRemovedUpstream;

  /// No description provided for @installedSkills.
  ///
  /// In en, this message translates to:
  /// **'Installed skills'**
  String get installedSkills;

  /// No description provided for @adoptSkill.
  ///
  /// In en, this message translates to:
  /// **'Manage in Crabforge'**
  String get adoptSkill;

  /// No description provided for @adoptSkillHint.
  ///
  /// In en, this message translates to:
  /// **'Crabforge will save a copy of these files and manage this folder. Unassigning the skill can remove the unchanged folder.'**
  String get adoptSkillHint;

  /// No description provided for @mcpLibrary.
  ///
  /// In en, this message translates to:
  /// **'MCP library'**
  String get mcpLibrary;

  /// No description provided for @assignMcpServer.
  ///
  /// In en, this message translates to:
  /// **'Assign to this project'**
  String get assignMcpServer;

  /// No description provided for @unassignMcpServer.
  ///
  /// In en, this message translates to:
  /// **'Unassign from this project'**
  String get unassignMcpServer;

  /// No description provided for @syncMcpLibrary.
  ///
  /// In en, this message translates to:
  /// **'Sync assignments'**
  String get syncMcpLibrary;

  /// No description provided for @refreshSkill.
  ///
  /// In en, this message translates to:
  /// **'Reimport files'**
  String get refreshSkill;

  /// No description provided for @projectMcp.
  ///
  /// In en, this message translates to:
  /// **'Project MCP'**
  String get projectMcp;

  /// No description provided for @addMcpServer.
  ///
  /// In en, this message translates to:
  /// **'Add MCP server'**
  String get addMcpServer;

  /// No description provided for @editMcpServer.
  ///
  /// In en, this message translates to:
  /// **'Edit MCP server'**
  String get editMcpServer;

  /// No description provided for @noMcpServers.
  ///
  /// In en, this message translates to:
  /// **'No MCP servers configured'**
  String get noMcpServers;

  /// No description provided for @mcpServerName.
  ///
  /// In en, this message translates to:
  /// **'Server name'**
  String get mcpServerName;

  /// No description provided for @addRule.
  ///
  /// In en, this message translates to:
  /// **'Add rule'**
  String get addRule;

  /// No description provided for @editRule.
  ///
  /// In en, this message translates to:
  /// **'Edit rule'**
  String get editRule;

  /// No description provided for @syncRules.
  ///
  /// In en, this message translates to:
  /// **'Apply rules'**
  String get syncRules;

  /// No description provided for @noRules.
  ///
  /// In en, this message translates to:
  /// **'No rules saved'**
  String get noRules;

  /// No description provided for @ruleName.
  ///
  /// In en, this message translates to:
  /// **'Rule name'**
  String get ruleName;

  /// No description provided for @ruleBody.
  ///
  /// In en, this message translates to:
  /// **'Markdown instructions'**
  String get ruleBody;

  /// No description provided for @rulePaths.
  ///
  /// In en, this message translates to:
  /// **'Path patterns (optional, one per line)'**
  String get rulePaths;

  /// No description provided for @ruleRequired.
  ///
  /// In en, this message translates to:
  /// **'This field is required.'**
  String get ruleRequired;

  /// No description provided for @ruleStatus.
  ///
  /// In en, this message translates to:
  /// **'{value, select, applied{Applied} pending{Pending} removed{Removed} conflict{Existing file kept} other{Could not apply}}'**
  String ruleStatus(String value);

  /// No description provided for @agentSessions.
  ///
  /// In en, this message translates to:
  /// **'Local sessions'**
  String get agentSessions;

  /// No description provided for @allAgents.
  ///
  /// In en, this message translates to:
  /// **'All agents'**
  String get allAgents;

  /// No description provided for @noAgentSessions.
  ///
  /// In en, this message translates to:
  /// **'No local sessions found'**
  String get noAgentSessions;

  /// No description provided for @unreadableSessions.
  ///
  /// In en, this message translates to:
  /// **'Unreadable session files ({count})'**
  String unreadableSessions(int count);

  /// No description provided for @copySessionId.
  ///
  /// In en, this message translates to:
  /// **'Copy session ID'**
  String get copySessionId;

  /// No description provided for @sessionWorkingFolder.
  ///
  /// In en, this message translates to:
  /// **'Working folder'**
  String get sessionWorkingFolder;

  /// No description provided for @sessionProfileFolder.
  ///
  /// In en, this message translates to:
  /// **'Profile folder'**
  String get sessionProfileFolder;

  /// No description provided for @distillTitle.
  ///
  /// In en, this message translates to:
  /// **'Extract memories'**
  String get distillTitle;

  /// No description provided for @distillPreview.
  ///
  /// In en, this message translates to:
  /// **'Review the user messages below before sending them to the selected model route. New memories will be saved for review.'**
  String get distillPreview;

  /// No description provided for @distillSend.
  ///
  /// In en, this message translates to:
  /// **'Send to model'**
  String get distillSend;

  /// No description provided for @distillEmpty.
  ///
  /// In en, this message translates to:
  /// **'No new memory candidates were found.'**
  String get distillEmpty;

  /// No description provided for @memorySourceQuote.
  ///
  /// In en, this message translates to:
  /// **'Quote from the session'**
  String get memorySourceQuote;

  /// No description provided for @agentInstructions.
  ///
  /// In en, this message translates to:
  /// **'User instructions'**
  String get agentInstructions;

  /// No description provided for @agentMemoryDigest.
  ///
  /// In en, this message translates to:
  /// **'User memory'**
  String get agentMemoryDigest;

  /// No description provided for @agentMemoryDigestHelp.
  ///
  /// In en, this message translates to:
  /// **'Publishing writes global memory and this agent’s memory to its user instructions.'**
  String get agentMemoryDigestHelp;

  /// No description provided for @agentMemoryDigestEmpty.
  ///
  /// In en, this message translates to:
  /// **'There are no active global or agent memories to publish.'**
  String get agentMemoryDigestEmpty;

  /// No description provided for @projectMemory.
  ///
  /// In en, this message translates to:
  /// **'Project memory'**
  String get projectMemory;

  /// No description provided for @publishMemory.
  ///
  /// In en, this message translates to:
  /// **'Publish digest'**
  String get publishMemory;

  /// No description provided for @projectMemoryHelp.
  ///
  /// In en, this message translates to:
  /// **'Publishing writes the digest to this project’s instructions. Anyone with access to the file can read it.'**
  String get projectMemoryHelp;

  /// No description provided for @projectMemoryEmpty.
  ///
  /// In en, this message translates to:
  /// **'This project has no active memories to publish.'**
  String get projectMemoryEmpty;

  /// No description provided for @searchMemories.
  ///
  /// In en, this message translates to:
  /// **'Search memories'**
  String get searchMemories;

  /// No description provided for @unreadableMemories.
  ///
  /// In en, this message translates to:
  /// **'Unreadable memory files'**
  String get unreadableMemories;

  /// No description provided for @noMemoriesFound.
  ///
  /// In en, this message translates to:
  /// **'No matching memories'**
  String get noMemoriesFound;

  /// No description provided for @addMemory.
  ///
  /// In en, this message translates to:
  /// **'Add memory'**
  String get addMemory;

  /// No description provided for @editMemory.
  ///
  /// In en, this message translates to:
  /// **'Edit memory'**
  String get editMemory;

  /// No description provided for @memoryScope.
  ///
  /// In en, this message translates to:
  /// **'Scope'**
  String get memoryScope;

  /// No description provided for @memoryTags.
  ///
  /// In en, this message translates to:
  /// **'Tags, one per line'**
  String get memoryTags;

  /// No description provided for @memoryWhy.
  ///
  /// In en, this message translates to:
  /// **'Reason (optional)'**
  String get memoryWhy;

  /// No description provided for @memoryKindLabel.
  ///
  /// In en, this message translates to:
  /// **'Kind'**
  String get memoryKindLabel;

  /// No description provided for @memoryStateLabel.
  ///
  /// In en, this message translates to:
  /// **'Status'**
  String get memoryStateLabel;

  /// No description provided for @pinMemory.
  ///
  /// In en, this message translates to:
  /// **'Pin memory'**
  String get pinMemory;

  /// No description provided for @memoryKind.
  ///
  /// In en, this message translates to:
  /// **'{value, select, profile{Profile} preference{Preference} rule{Rule} reference{Reference} fact{Fact} procedure{Procedure} episode{Episode} other{Kind}}'**
  String memoryKind(String value);

  /// No description provided for @memoryState.
  ///
  /// In en, this message translates to:
  /// **'{value, select, active{Active} candidate{Candidate} archived{Archived} superseded{Superseded} other{All statuses}}'**
  String memoryState(String value);

  /// No description provided for @projectInstructions.
  ///
  /// In en, this message translates to:
  /// **'Project instructions'**
  String get projectInstructions;

  /// No description provided for @projectFolder.
  ///
  /// In en, this message translates to:
  /// **'Project folder'**
  String get projectFolder;

  /// No description provided for @openProject.
  ///
  /// In en, this message translates to:
  /// **'Open project'**
  String get openProject;

  /// No description provided for @restoreInstructions.
  ///
  /// In en, this message translates to:
  /// **'Restore previous version'**
  String get restoreInstructions;

  /// No description provided for @relayFallback.
  ///
  /// In en, this message translates to:
  /// **'Fallback relay circuit address (optional)'**
  String get relayFallback;

  /// No description provided for @listView.
  ///
  /// In en, this message translates to:
  /// **'List view'**
  String get listView;

  /// No description provided for @gridView.
  ///
  /// In en, this message translates to:
  /// **'Grid view'**
  String get gridView;

  /// No description provided for @gatewayProtocol.
  ///
  /// In en, this message translates to:
  /// **'Gateway protocol'**
  String get gatewayProtocol;

  /// No description provided for @nativeModelProtocol.
  ///
  /// In en, this message translates to:
  /// **'Use each model’s native protocol'**
  String get nativeModelProtocol;

  /// No description provided for @roleModels.
  ///
  /// In en, this message translates to:
  /// **'Role models'**
  String get roleModels;

  /// No description provided for @roleModelId.
  ///
  /// In en, this message translates to:
  /// **'Model ID'**
  String get roleModelId;

  /// No description provided for @resetRole.
  ///
  /// In en, this message translates to:
  /// **'Reset role'**
  String get resetRole;

  /// No description provided for @syncModelCatalog.
  ///
  /// In en, this message translates to:
  /// **'Sync model catalog'**
  String get syncModelCatalog;

  /// No description provided for @thinkingLevels.
  ///
  /// In en, this message translates to:
  /// **'Thinking levels'**
  String get thinkingLevels;

  /// No description provided for @sessionThinking.
  ///
  /// In en, this message translates to:
  /// **'Session thinking level'**
  String get sessionThinking;

  /// No description provided for @thinkingAuto.
  ///
  /// In en, this message translates to:
  /// **'Automatic'**
  String get thinkingAuto;

  /// No description provided for @thinkingMinimal.
  ///
  /// In en, this message translates to:
  /// **'Minimal'**
  String get thinkingMinimal;

  /// No description provided for @thinkingLow.
  ///
  /// In en, this message translates to:
  /// **'Low'**
  String get thinkingLow;

  /// No description provided for @thinkingMedium.
  ///
  /// In en, this message translates to:
  /// **'Medium'**
  String get thinkingMedium;

  /// No description provided for @thinkingHigh.
  ///
  /// In en, this message translates to:
  /// **'High'**
  String get thinkingHigh;

  /// No description provided for @thinkingExtraHigh.
  ///
  /// In en, this message translates to:
  /// **'Extra high'**
  String get thinkingExtraHigh;

  /// No description provided for @thinkingMax.
  ///
  /// In en, this message translates to:
  /// **'Maximum'**
  String get thinkingMax;

  /// No description provided for @ompProfile.
  ///
  /// In en, this message translates to:
  /// **'Oh My Pi profile'**
  String get ompProfile;

  /// No description provided for @advisorSettings.
  ///
  /// In en, this message translates to:
  /// **'Advisor settings'**
  String get advisorSettings;

  /// No description provided for @advisorReview.
  ///
  /// In en, this message translates to:
  /// **'When to review'**
  String get advisorReview;

  /// No description provided for @advisorWait.
  ///
  /// In en, this message translates to:
  /// **'Pending reviews before waiting'**
  String get advisorWait;

  /// No description provided for @advisorWake.
  ///
  /// In en, this message translates to:
  /// **'Minimum note severity to resume'**
  String get advisorWake;

  /// No description provided for @inheritSetting.
  ///
  /// In en, this message translates to:
  /// **'Use profile default'**
  String get inheritSetting;

  /// No description provided for @everyTurn.
  ///
  /// In en, this message translates to:
  /// **'Every turn'**
  String get everyTurn;

  /// No description provided for @runEnd.
  ///
  /// In en, this message translates to:
  /// **'At the end of the run'**
  String get runEnd;

  /// No description provided for @allReviews.
  ///
  /// In en, this message translates to:
  /// **'Wait for all reviews'**
  String get allReviews;

  /// No description provided for @minorNote.
  ///
  /// In en, this message translates to:
  /// **'Minor issue'**
  String get minorNote;

  /// No description provided for @concernNote.
  ///
  /// In en, this message translates to:
  /// **'Concern'**
  String get concernNote;

  /// No description provided for @blockerNote.
  ///
  /// In en, this message translates to:
  /// **'Blocker'**
  String get blockerNote;

  /// No description provided for @memberPatterns.
  ///
  /// In en, this message translates to:
  /// **'Model patterns, one per line'**
  String get memberPatterns;

  /// No description provided for @pluginBundles.
  ///
  /// In en, this message translates to:
  /// **'Shared plugin library'**
  String get pluginBundles;

  /// No description provided for @importPluginBundle.
  ///
  /// In en, this message translates to:
  /// **'Import plugin bundle'**
  String get importPluginBundle;

  /// No description provided for @previewPluginBundle.
  ///
  /// In en, this message translates to:
  /// **'Preview bundle'**
  String get previewPluginBundle;

  /// No description provided for @pluginBundleSource.
  ///
  /// In en, this message translates to:
  /// **'Plugin folder, manifest, or Git URL'**
  String get pluginBundleSource;

  /// No description provided for @noPluginBundles.
  ///
  /// In en, this message translates to:
  /// **'No plugin bundles saved.'**
  String get noPluginBundles;

  /// No description provided for @otherPluginEntries.
  ///
  /// In en, this message translates to:
  /// **'Other manifest entries'**
  String get otherPluginEntries;

  /// No description provided for @noBundleResources.
  ///
  /// In en, this message translates to:
  /// **'No resources in this file.'**
  String get noBundleResources;

  /// No description provided for @copyBundleResource.
  ///
  /// In en, this message translates to:
  /// **'Copy to library'**
  String get copyBundleResource;

  /// No description provided for @bundleResourceCopied.
  ///
  /// In en, this message translates to:
  /// **'Copied to library'**
  String get bundleResourceCopied;

  /// No description provided for @openPluginArchive.
  ///
  /// In en, this message translates to:
  /// **'Open TAR archive'**
  String get openPluginArchive;

  /// No description provided for @exportPluginArchive.
  ///
  /// In en, this message translates to:
  /// **'Export plugin archive'**
  String get exportPluginArchive;

  /// No description provided for @pluginArchiveSaved.
  ///
  /// In en, this message translates to:
  /// **'Plugin archive saved'**
  String get pluginArchiveSaved;

  /// No description provided for @pluginCatalog.
  ///
  /// In en, this message translates to:
  /// **'Browse plugins'**
  String get pluginCatalog;

  /// No description provided for @noAvailablePlugins.
  ///
  /// In en, this message translates to:
  /// **'No plugins available from these marketplaces.'**
  String get noAvailablePlugins;

  /// No description provided for @exportPluginSettings.
  ///
  /// In en, this message translates to:
  /// **'Export plugin settings'**
  String get exportPluginSettings;

  /// No description provided for @importPluginSettings.
  ///
  /// In en, this message translates to:
  /// **'Import plugin settings'**
  String get importPluginSettings;

  /// No description provided for @claudePlugins.
  ///
  /// In en, this message translates to:
  /// **'Claude Code plugins'**
  String get claudePlugins;

  /// No description provided for @pluginMarketplaces.
  ///
  /// In en, this message translates to:
  /// **'Marketplaces'**
  String get pluginMarketplaces;

  /// No description provided for @addPluginMarketplace.
  ///
  /// In en, this message translates to:
  /// **'Add marketplace'**
  String get addPluginMarketplace;

  /// No description provided for @pluginSource.
  ///
  /// In en, this message translates to:
  /// **'Marketplace path or URL'**
  String get pluginSource;

  /// No description provided for @installPlugin.
  ///
  /// In en, this message translates to:
  /// **'Install plugin'**
  String get installPlugin;

  /// No description provided for @updatePlugin.
  ///
  /// In en, this message translates to:
  /// **'Update plugin'**
  String get updatePlugin;

  /// No description provided for @enablePlugin.
  ///
  /// In en, this message translates to:
  /// **'Enable plugin'**
  String get enablePlugin;

  /// No description provided for @disablePlugin.
  ///
  /// In en, this message translates to:
  /// **'Disable plugin'**
  String get disablePlugin;

  /// No description provided for @pluginEnabled.
  ///
  /// In en, this message translates to:
  /// **'Enabled'**
  String get pluginEnabled;

  /// No description provided for @noClaudePlugins.
  ///
  /// In en, this message translates to:
  /// **'No Claude Code plugins installed for this scope.'**
  String get noClaudePlugins;

  /// No description provided for @shareGroup.
  ///
  /// In en, this message translates to:
  /// **'Copy group to share'**
  String get shareGroup;

  /// No description provided for @groupCopied.
  ///
  /// In en, this message translates to:
  /// **'Group copied to clipboard'**
  String get groupCopied;

  /// No description provided for @importGroup.
  ///
  /// In en, this message translates to:
  /// **'Import group'**
  String get importGroup;

  /// No description provided for @sharedGroupText.
  ///
  /// In en, this message translates to:
  /// **'Shared group text'**
  String get sharedGroupText;

  /// No description provided for @replaceGroup.
  ///
  /// In en, this message translates to:
  /// **'Replace an existing group with the same ID'**
  String get replaceGroup;

  /// No description provided for @missingMembers.
  ///
  /// In en, this message translates to:
  /// **'Unavailable members'**
  String get missingMembers;

  /// No description provided for @replacedMembers.
  ///
  /// In en, this message translates to:
  /// **'Member substitutions'**
  String get replacedMembers;

  /// No description provided for @routeSession.
  ///
  /// In en, this message translates to:
  /// **'Session {id}'**
  String routeSession(String id);

  /// No description provided for @enableGroup.
  ///
  /// In en, this message translates to:
  /// **'Enable group'**
  String get enableGroup;

  /// No description provided for @disableGroup.
  ///
  /// In en, this message translates to:
  /// **'Disable group'**
  String get disableGroup;

  /// No description provided for @listGroupModels.
  ///
  /// In en, this message translates to:
  /// **'Show member models in client catalogs'**
  String get listGroupModels;

  /// No description provided for @copyGroup.
  ///
  /// In en, this message translates to:
  /// **'Duplicate group'**
  String get copyGroup;

  /// No description provided for @groupNote.
  ///
  /// In en, this message translates to:
  /// **'Group note (optional)'**
  String get groupNote;

  /// No description provided for @enableMember.
  ///
  /// In en, this message translates to:
  /// **'Enable member'**
  String get enableMember;

  /// No description provided for @disableMember.
  ///
  /// In en, this message translates to:
  /// **'Disable member'**
  String get disableMember;

  /// No description provided for @manualRouting.
  ///
  /// In en, this message translates to:
  /// **'Manual selection'**
  String get manualRouting;

  /// No description provided for @selectMember.
  ///
  /// In en, this message translates to:
  /// **'Select this member'**
  String get selectMember;

  /// No description provided for @providerCatalogs.
  ///
  /// In en, this message translates to:
  /// **'models.dev catalog IDs (optional)'**
  String get providerCatalogs;

  /// No description provided for @anthropicAuthHeader.
  ///
  /// In en, this message translates to:
  /// **'Anthropic API key header'**
  String get anthropicAuthHeader;

  /// No description provided for @automaticAuthHeader.
  ///
  /// In en, this message translates to:
  /// **'Automatic'**
  String get automaticAuthHeader;

  /// No description provided for @autoGroups.
  ///
  /// In en, this message translates to:
  /// **'Group matching models across providers'**
  String get autoGroups;

  /// No description provided for @restoreGroup.
  ///
  /// In en, this message translates to:
  /// **'Restore group'**
  String get restoreGroup;

  /// No description provided for @vertexProject.
  ///
  /// In en, this message translates to:
  /// **'Vertex AI project settings'**
  String get vertexProject;

  /// No description provided for @cloudProjectId.
  ///
  /// In en, this message translates to:
  /// **'Google Cloud project ID'**
  String get cloudProjectId;

  /// No description provided for @vertexLocation.
  ///
  /// In en, this message translates to:
  /// **'Vertex AI location'**
  String get vertexLocation;

  /// No description provided for @googleCredentials.
  ///
  /// In en, this message translates to:
  /// **'ADC file (optional)'**
  String get googleCredentials;

  /// No description provided for @impersonateAccount.
  ///
  /// In en, this message translates to:
  /// **'Service account to impersonate (optional)'**
  String get impersonateAccount;

  /// No description provided for @models.
  ///
  /// In en, this message translates to:
  /// **'Models'**
  String get models;

  /// No description provided for @modelCopied.
  ///
  /// In en, this message translates to:
  /// **'Model ID copied'**
  String get modelCopied;

  /// No description provided for @noModelsFound.
  ///
  /// In en, this message translates to:
  /// **'No matching models'**
  String get noModelsFound;

  /// No description provided for @routingStrategy.
  ///
  /// In en, this message translates to:
  /// **'Routing strategy'**
  String get routingStrategy;

  /// No description provided for @listedOrder.
  ///
  /// In en, this message translates to:
  /// **'Listed order'**
  String get listedOrder;

  /// No description provided for @weightedOrder.
  ///
  /// In en, this message translates to:
  /// **'Weighted round robin'**
  String get weightedOrder;

  /// No description provided for @memberWeight.
  ///
  /// In en, this message translates to:
  /// **'Member weight'**
  String get memberWeight;

  /// No description provided for @invalidWeight.
  ///
  /// In en, this message translates to:
  /// **'Enter a whole number from 1 to 1000.'**
  String get invalidWeight;

  /// No description provided for @providers.
  ///
  /// In en, this message translates to:
  /// **'Providers'**
  String get providers;

  /// No description provided for @routes.
  ///
  /// In en, this message translates to:
  /// **'Routes'**
  String get routes;

  /// No description provided for @groups.
  ///
  /// In en, this message translates to:
  /// **'Route groups'**
  String get groups;

  /// No description provided for @gateway.
  ///
  /// In en, this message translates to:
  /// **'Gateway'**
  String get gateway;

  /// No description provided for @language.
  ///
  /// In en, this message translates to:
  /// **'Language'**
  String get language;

  /// No description provided for @systemLanguage.
  ///
  /// In en, this message translates to:
  /// **'System language'**
  String get systemLanguage;

  /// No description provided for @confirmRemove.
  ///
  /// In en, this message translates to:
  /// **'Remove {value}?'**
  String confirmRemove(String value);

  /// No description provided for @cancel.
  ///
  /// In en, this message translates to:
  /// **'Cancel'**
  String get cancel;

  /// No description provided for @remove.
  ///
  /// In en, this message translates to:
  /// **'Remove'**
  String get remove;

  /// No description provided for @save.
  ///
  /// In en, this message translates to:
  /// **'Save'**
  String get save;

  /// No description provided for @webLogin.
  ///
  /// In en, this message translates to:
  /// **'Web sign-in'**
  String get webLogin;

  /// No description provided for @loginHelp.
  ///
  /// In en, this message translates to:
  /// **'Sign in in the new Chrome window, then return here to close it.'**
  String get loginHelp;

  /// No description provided for @closeLogin.
  ///
  /// In en, this message translates to:
  /// **'Close sign-in window'**
  String get closeLogin;

  /// No description provided for @duplicateGroup.
  ///
  /// In en, this message translates to:
  /// **'This group ID already exists.'**
  String get duplicateGroup;

  /// No description provided for @addProvider.
  ///
  /// In en, this message translates to:
  /// **'Add provider'**
  String get addProvider;

  /// No description provided for @newGroup.
  ///
  /// In en, this message translates to:
  /// **'New group'**
  String get newGroup;

  /// No description provided for @stopToEdit.
  ///
  /// In en, this message translates to:
  /// **'Stop the gateway to edit its configuration.'**
  String get stopToEdit;

  /// No description provided for @reload.
  ///
  /// In en, this message translates to:
  /// **'Reload'**
  String get reload;

  /// No description provided for @reloadConfig.
  ///
  /// In en, this message translates to:
  /// **'Reload configuration'**
  String get reloadConfig;

  /// No description provided for @providerCount.
  ///
  /// In en, this message translates to:
  /// **'Providers: {count}'**
  String providerCount(int count);

  /// No description provided for @presetCount.
  ///
  /// In en, this message translates to:
  /// **'Choose from {count} API presets or enter a custom base URL.'**
  String presetCount(int count);

  /// No description provided for @providerHelp.
  ///
  /// In en, this message translates to:
  /// **'Add an upstream URL and model ID to enable client requests.'**
  String get providerHelp;

  /// No description provided for @removeProvider.
  ///
  /// In en, this message translates to:
  /// **'Remove provider'**
  String get removeProvider;

  /// No description provided for @webProviders.
  ///
  /// In en, this message translates to:
  /// **'Web providers'**
  String get webProviders;

  /// No description provided for @webHelp.
  ///
  /// In en, this message translates to:
  /// **'Use Chrome and a matching ChromeDriver. Select an added provider to sign in with its own browser profile.'**
  String get webHelp;

  /// No description provided for @webSite.
  ///
  /// In en, this message translates to:
  /// **'{value} Web'**
  String webSite(String value);

  /// No description provided for @groupNeedsProvider.
  ///
  /// In en, this message translates to:
  /// **'Add a provider and model before creating a route group.'**
  String get groupNeedsProvider;

  /// No description provided for @groupHelp.
  ///
  /// In en, this message translates to:
  /// **'Add members in priority order. On rate limits or server errors, the gateway tries the next member.'**
  String get groupHelp;

  /// No description provided for @removeGroup.
  ///
  /// In en, this message translates to:
  /// **'Remove group'**
  String get removeGroup;

  /// No description provided for @recentAttempts.
  ///
  /// In en, this message translates to:
  /// **'Recent upstream attempts: {count}'**
  String recentAttempts(int count);

  /// No description provided for @historyLimit.
  ///
  /// In en, this message translates to:
  /// **'Current gateway process · 128 most recent attempts'**
  String get historyLimit;

  /// No description provided for @waiting.
  ///
  /// In en, this message translates to:
  /// **'Waiting for a client request.'**
  String get waiting;

  /// No description provided for @historyHelp.
  ///
  /// In en, this message translates to:
  /// **'Start the gateway to see route history.'**
  String get historyHelp;

  /// No description provided for @running.
  ///
  /// In en, this message translates to:
  /// **'Gateway running'**
  String get running;

  /// No description provided for @stopped.
  ///
  /// In en, this message translates to:
  /// **'Gateway stopped'**
  String get stopped;

  /// No description provided for @stop.
  ///
  /// In en, this message translates to:
  /// **'Stop gateway'**
  String get stop;

  /// No description provided for @start.
  ///
  /// In en, this message translates to:
  /// **'Start gateway'**
  String get start;

  /// No description provided for @keyCopied.
  ///
  /// In en, this message translates to:
  /// **'Gateway key copied'**
  String get keyCopied;

  /// No description provided for @copyKey.
  ///
  /// In en, this message translates to:
  /// **'Copy client key'**
  String get copyKey;

  /// No description provided for @networkSettings.
  ///
  /// In en, this message translates to:
  /// **'Network settings'**
  String get networkSettings;

  /// No description provided for @browserOrigins.
  ///
  /// In en, this message translates to:
  /// **'Allowed browser origins (one per line)'**
  String get browserOrigins;

  /// No description provided for @localPeer.
  ///
  /// In en, this message translates to:
  /// **'Local peer ID'**
  String get localPeer;

  /// No description provided for @copyPeer.
  ///
  /// In en, this message translates to:
  /// **'Copy peer ID'**
  String get copyPeer;

  /// No description provided for @peerCopied.
  ///
  /// In en, this message translates to:
  /// **'Peer ID copied'**
  String get peerCopied;

  /// No description provided for @listenAddress.
  ///
  /// In en, this message translates to:
  /// **'Listen address'**
  String get listenAddress;

  /// No description provided for @disabled.
  ///
  /// In en, this message translates to:
  /// **'Disabled'**
  String get disabled;

  /// No description provided for @trustedPeers.
  ///
  /// In en, this message translates to:
  /// **'Trusted peers'**
  String get trustedPeers;

  /// No description provided for @clients.
  ///
  /// In en, this message translates to:
  /// **'Client connections'**
  String get clients;

  /// No description provided for @agentRoot.
  ///
  /// In en, this message translates to:
  /// **'Agent API root URL'**
  String get agentRoot;

  /// No description provided for @copyRoot.
  ///
  /// In en, this message translates to:
  /// **'Copy root URL'**
  String get copyRoot;

  /// No description provided for @rootCopied.
  ///
  /// In en, this message translates to:
  /// **'Root URL copied'**
  String get rootCopied;

  /// No description provided for @agentHelp.
  ///
  /// In en, this message translates to:
  /// **'Append /v1 for Chat Completions and Responses clients. Route history shows the selected agent.'**
  String get agentHelp;

  /// No description provided for @tunnel.
  ///
  /// In en, this message translates to:
  /// **'Remote gateway tunnel'**
  String get tunnel;

  /// No description provided for @disconnected.
  ///
  /// In en, this message translates to:
  /// **'Disconnected'**
  String get disconnected;

  /// No description provided for @connect.
  ///
  /// In en, this message translates to:
  /// **'Connect'**
  String get connect;

  /// No description provided for @disconnect.
  ///
  /// In en, this message translates to:
  /// **'Disconnect'**
  String get disconnect;

  /// No description provided for @apiBase.
  ///
  /// In en, this message translates to:
  /// **'API base URL'**
  String get apiBase;

  /// No description provided for @copyBase.
  ///
  /// In en, this message translates to:
  /// **'Copy base URL'**
  String get copyBase;

  /// No description provided for @baseCopied.
  ///
  /// In en, this message translates to:
  /// **'API base URL copied'**
  String get baseCopied;

  /// No description provided for @changed.
  ///
  /// In en, this message translates to:
  /// **'Provider settings changed. Fetch models again.'**
  String get changed;

  /// No description provided for @modelsRead.
  ///
  /// In en, this message translates to:
  /// **'Models found: {count}. Edit the list before saving.'**
  String modelsRead(int count);

  /// No description provided for @presetLoaded.
  ///
  /// In en, this message translates to:
  /// **'Preset models loaded. Adjust the list for your account.'**
  String get presetLoaded;

  /// No description provided for @editProvider.
  ///
  /// In en, this message translates to:
  /// **'Edit provider'**
  String get editProvider;

  /// No description provided for @searchPresets.
  ///
  /// In en, this message translates to:
  /// **'Search presets by name'**
  String get searchPresets;

  /// No description provided for @providerId.
  ///
  /// In en, this message translates to:
  /// **'Provider ID'**
  String get providerId;

  /// No description provided for @renameProvider.
  ///
  /// In en, this message translates to:
  /// **'Rename provider'**
  String get renameProvider;

  /// No description provided for @keyEnv.
  ///
  /// In en, this message translates to:
  /// **'API key environment variable'**
  String get keyEnv;

  /// No description provided for @keyHint.
  ///
  /// In en, this message translates to:
  /// **'Optional for local services or Bedrock with AWS IAM'**
  String get keyHint;

  /// No description provided for @resourceRoot.
  ///
  /// In en, this message translates to:
  /// **'Resource root URL'**
  String get resourceRoot;

  /// No description provided for @modelIds.
  ///
  /// In en, this message translates to:
  /// **'Upstream model IDs, one per line'**
  String get modelIds;

  /// No description provided for @providerFallback.
  ///
  /// In en, this message translates to:
  /// **'Fallback models or groups, in order, one per line'**
  String get providerFallback;

  /// No description provided for @allowProviderFallback.
  ///
  /// In en, this message translates to:
  /// **'Accept fallback requests from other providers'**
  String get allowProviderFallback;

  /// No description provided for @protocolBase.
  ///
  /// In en, this message translates to:
  /// **'{value} base URL'**
  String protocolBase(String value);

  /// No description provided for @catalogUrl.
  ///
  /// In en, this message translates to:
  /// **'Model catalog URL (optional)'**
  String get catalogUrl;

  /// No description provided for @catalogHint.
  ///
  /// In en, this message translates to:
  /// **'Leave blank for the provider’s default catalog'**
  String get catalogHint;

  /// No description provided for @readingModels.
  ///
  /// In en, this message translates to:
  /// **'Loading models'**
  String get readingModels;

  /// No description provided for @readModels.
  ///
  /// In en, this message translates to:
  /// **'Load model catalog'**
  String get readModels;

  /// No description provided for @requiredField.
  ///
  /// In en, this message translates to:
  /// **'This field is required.'**
  String get requiredField;

  /// No description provided for @createGroup.
  ///
  /// In en, this message translates to:
  /// **'Create route group'**
  String get createGroup;

  /// No description provided for @editGroup.
  ///
  /// In en, this message translates to:
  /// **'Edit route group'**
  String get editGroup;

  /// No description provided for @groupId.
  ///
  /// In en, this message translates to:
  /// **'Route group ID'**
  String get groupId;

  /// No description provided for @raisePriority.
  ///
  /// In en, this message translates to:
  /// **'Increase priority'**
  String get raisePriority;

  /// No description provided for @removeMember.
  ///
  /// In en, this message translates to:
  /// **'Remove member'**
  String get removeMember;

  /// No description provided for @addCandidate.
  ///
  /// In en, this message translates to:
  /// **'Add candidate model'**
  String get addCandidate;

  /// No description provided for @connectRemote.
  ///
  /// In en, this message translates to:
  /// **'Connect to remote gateway'**
  String get connectRemote;

  /// No description provided for @remoteQuic.
  ///
  /// In en, this message translates to:
  /// **'Remote QUIC address (include /p2p/PeerID)'**
  String get remoteQuic;

  /// No description provided for @localTunnel.
  ///
  /// In en, this message translates to:
  /// **'Local tunnel listen address'**
  String get localTunnel;

  /// No description provided for @tunnelHelp.
  ///
  /// In en, this message translates to:
  /// **'Trust this peer ID on the remote gateway. Clients connecting to the local tunnel use the remote gateway key.'**
  String get tunnelHelp;

  /// No description provided for @loopback.
  ///
  /// In en, this message translates to:
  /// **'HTTP loopback address'**
  String get loopback;

  /// No description provided for @peerListen.
  ///
  /// In en, this message translates to:
  /// **'P2P listen address (blank to disable)'**
  String get peerListen;

  /// No description provided for @allowedPeers.
  ///
  /// In en, this message translates to:
  /// **'Allowed peer IDs, one per line'**
  String get allowedPeers;

  /// No description provided for @rotateMembers.
  ///
  /// In en, this message translates to:
  /// **'Rotate the first member'**
  String get rotateMembers;
}

class _AppLocalizationsDelegate
    extends LocalizationsDelegate<AppLocalizations> {
  const _AppLocalizationsDelegate();

  @override
  Future<AppLocalizations> load(Locale locale) {
    return SynchronousFuture<AppLocalizations>(lookupAppLocalizations(locale));
  }

  @override
  bool isSupported(Locale locale) => <String>[
    'de',
    'en',
    'es',
    'fr',
    'hi',
    'ja',
    'ko',
    'pt',
    'zh',
  ].contains(locale.languageCode);

  @override
  bool shouldReload(_AppLocalizationsDelegate old) => false;
}

AppLocalizations lookupAppLocalizations(Locale locale) {
  // Lookup logic when only language code is specified.
  switch (locale.languageCode) {
    case 'de':
      return AppLocalizationsDe();
    case 'en':
      return AppLocalizationsEn();
    case 'es':
      return AppLocalizationsEs();
    case 'fr':
      return AppLocalizationsFr();
    case 'hi':
      return AppLocalizationsHi();
    case 'ja':
      return AppLocalizationsJa();
    case 'ko':
      return AppLocalizationsKo();
    case 'pt':
      return AppLocalizationsPt();
    case 'zh':
      return AppLocalizationsZh();
  }

  throw FlutterError(
    'AppLocalizations.delegate failed to load unsupported locale "$locale". This is likely '
    'an issue with the localizations generation tool. Please file an issue '
    'on GitHub with a reproducible sample app and the gen-l10n configuration '
    'that was used.',
  );
}
