// ignore: unused_import
import 'package:intl/intl.dart' as intl;

import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for German (`de`).
class AppLocalizationsDe extends AppLocalizations {
  AppLocalizationsDe([String locale = 'de']) : super(locale);

  @override
  String get interfaceFont => 'Schriftart';

  @override
  String get importSlashCommands => 'Slash-Befehle importieren';

  @override
  String get commandImportSource =>
      'Markdown- oder TOML-Datei, lokaler Ordner oder Git-URL';

  @override
  String get commandChooseFile => 'Befehlsdatei auswählen';

  @override
  String get projectHookAssignment => 'Hook-Zuweisung im Projekt bearbeiten';

  @override
  String get sharedSubagents => 'Freigegebene Subagenten';

  @override
  String get importSubagents => 'Subagenten importieren';

  @override
  String get subagentImportSource =>
      'Markdown-Datei, lokaler Ordner oder Git-URL';

  @override
  String get subagentChooseFile => 'Markdown-Datei auswählen';

  @override
  String get subagentChooseFolder => 'Quellordner auswählen';

  @override
  String get subagentScanSource => 'Quelle durchsuchen';

  @override
  String get subagentImportTruncated =>
      'Nur ein Teil der Quelle wurde durchsucht. Wählen Sie einen kleineren Ordner, um weitere Dateien zu sehen.';

  @override
  String subagentImportStatus(String status) {
    String _temp0 = intl.Intl.selectLogic(status, {
      'update': 'Aktualisierung verfügbar',
      'unchanged': 'Bereits importiert',
      'edited': 'Lokale Änderungen beibehalten',
      'other': '',
    });
    return '$_temp0';
  }

  @override
  String get findNativeResources => 'Vorhandene Dateien suchen';

  @override
  String get noNativeResources => 'Keine vorhandenen Dateien gefunden';

  @override
  String get nativeResourceScanLimit =>
      'Die ersten 512 gefundenen Dateien werden angezeigt.';

  @override
  String get adoptResourceDetails =>
      'Crabforge verwaltet diese Datei an ihrem aktuellen Speicherort. Wird die Client-Zuweisung aufgehoben, kann die unveränderte Datei entfernt werden.';

  @override
  String get agentSubagents => 'Subagenten';

  @override
  String get agentCommands => 'Schrägstrichbefehle';

  @override
  String get addSubagent => 'Subagent hinzufügen';

  @override
  String get addSlashCommand => 'Schrägstrichbefehl hinzufügen';

  @override
  String get resourceUserScope => 'Benutzerprofile';

  @override
  String get noAgentResources => 'Keine Einträge gespeichert';

  @override
  String get resourcePlacements => 'Veröffentlichungsdetails';

  @override
  String get agentResourceName => 'Name';

  @override
  String get agentResourceDescription => 'Beschreibung';

  @override
  String get agentResourcePrompt => 'Prompt';

  @override
  String get resourceSkillNames => 'Skill-Namen, einer pro Zeile';

  @override
  String get resourceMcpNames => 'MCP-Servernamen, einer pro Zeile';

  @override
  String get commandArgumentHelp =>
      'Verwende \$ARGUMENTS für alle Argumente und \$1, \$2 usw. für einzelne Argumente in kompatiblen Clients.';

  @override
  String get agentHooks => 'Hooks';

  @override
  String get addHook => 'Hook hinzufügen';

  @override
  String get editHook => 'Hook bearbeiten';

  @override
  String get adoptHook => 'Hook übernehmen';

  @override
  String get noHooks => 'Keine Hooks gespeichert';

  @override
  String get installedHooks => 'Hooks in den Client-Einstellungen';

  @override
  String get removeInstalledHook => 'Installierten Hook entfernen';

  @override
  String get hookBackups => 'Hook-Sicherungen';

  @override
  String get restoreHookBackup => 'Hook-Sicherung wiederherstellen';

  @override
  String get hookBackupRestored => 'Wiederhergestellt';

  @override
  String get hookBackupChanged => 'Client-Einstellungen wurden geändert';

  @override
  String get hookBackupUnavailable =>
      'Ursprüngliches Profil oder Datei nicht verfügbar';

  @override
  String get hookName => 'Name des Hooks';

  @override
  String get hookDescription => 'Beschreibung (optional)';

  @override
  String get hookEvent => 'Ereignis';

  @override
  String get hookMatcher => 'Filter';

  @override
  String get hookAllTools => 'Alle Werkzeuge';

  @override
  String get hookShellTool => 'Shell-Befehle';

  @override
  String get hookEditTool => 'Dateibearbeitungen';

  @override
  String get hookCommand => 'Vom Client ausgeführter Befehl';

  @override
  String get hookTimeout => 'Zeitlimit in Sekunden (optional)';

  @override
  String get hookTurnEnd => 'Ende des Gesprächsschritts';

  @override
  String get hookAfterTool => 'Nach dem Werkzeugaufruf';

  @override
  String get hookBeforeTool => 'Vor dem Werkzeugaufruf';

  @override
  String get hookPrompt => 'Nach Absenden der Eingabe';

  @override
  String get hookSessionStart => 'Sitzungsbeginn';

  @override
  String get hookExisting => 'Passender Hook bereits vorhanden';

  @override
  String get hookConflict => 'Vorhandene Hooks beibehalten';

  @override
  String get agentEnvironment => 'Umgebungsvariablen';

  @override
  String get noEnvironmentVariables => 'Keine Variablen gespeichert';

  @override
  String get addVariable => 'Variable hinzufügen';

  @override
  String get editVariable => 'Variable bearbeiten';

  @override
  String get variableName => 'Variablenname';

  @override
  String get variableValue => 'Wert';

  @override
  String get showVariableValue => 'Wert anzeigen';

  @override
  String get hideVariableValue => 'Wert ausblenden';

  @override
  String get nativeVariableExists => 'Bereits gesetzt';

  @override
  String get nativeVariableConflict => 'Vorhandener Wert beibehalten';

  @override
  String get memoryLibrary => 'Gedächtnis';

  @override
  String get agentRules => 'Regeln';

  @override
  String get agentSkills => 'Skills';

  @override
  String get importSkill => 'Skill importieren';

  @override
  String get applySkills => 'Skills anwenden';

  @override
  String get noSkills => 'Keine Skills importiert';

  @override
  String get editSkill => 'Skill-Zuweisungen bearbeiten';

  @override
  String get skillSourceFolder => 'Lokaler Ordner oder Git-URL';

  @override
  String get checkSkillUpdate => 'Nach Updates suchen';

  @override
  String get browseRepositorySkills => 'Skills in der Quelle durchsuchen';

  @override
  String get noRepositorySkills =>
      'In dieser Quelle sind keine weiteren Skills verfügbar.';

  @override
  String get dismissSkillCandidate => 'Skill ausblenden';

  @override
  String get skillUpToDate => 'Die Skill-Dateien sind aktuell.';

  @override
  String get skillUpdateAvailable =>
      'Neue Skill-Dateien sind verfügbar. Importieren Sie die Dateien erneut.';

  @override
  String get skillRemovedUpstream =>
      'Der ausgewählte Ordner enthält keine SKILL.md mehr.';

  @override
  String get installedSkills => 'Installierte Skills';

  @override
  String get adoptSkill => 'In Crabforge verwalten';

  @override
  String get adoptSkillHint =>
      'Crabforge speichert eine Kopie der Dateien und verwaltet diesen Ordner. Beim Aufheben der Zuweisung kann der unveränderte Ordner entfernt werden.';

  @override
  String get mcpLibrary => 'MCP-Bibliothek';

  @override
  String get assignMcpServer => 'Diesem Projekt zuweisen';

  @override
  String get unassignMcpServer => 'Projektzuweisung aufheben';

  @override
  String get syncMcpLibrary => 'Zuweisungen synchronisieren';

  @override
  String get refreshSkill => 'Dateien neu importieren';

  @override
  String get projectMcp => 'Projekt-MCP';

  @override
  String get addMcpServer => 'MCP-Server hinzufügen';

  @override
  String get editMcpServer => 'MCP-Server bearbeiten';

  @override
  String get noMcpServers => 'Keine MCP-Server konfiguriert';

  @override
  String get mcpServerName => 'Servername';

  @override
  String get addRule => 'Regel hinzufügen';

  @override
  String get editRule => 'Regel bearbeiten';

  @override
  String get syncRules => 'Regeln anwenden';

  @override
  String get noRules => 'Keine Regeln gespeichert';

  @override
  String get ruleName => 'Regelname';

  @override
  String get ruleBody => 'Markdown-Anweisungen';

  @override
  String get rulePaths => 'Pfadmuster (optional, eines pro Zeile)';

  @override
  String get ruleRequired => 'Dies ist ein Pflichtfeld.';

  @override
  String ruleStatus(String value) {
    String _temp0 = intl.Intl.selectLogic(value, {
      'applied': 'Angewendet',
      'pending': 'Ausstehend',
      'removed': 'Entfernt',
      'conflict': 'Vorhandene Datei beibehalten',
      'other': 'Anwendung fehlgeschlagen',
    });
    return '$_temp0';
  }

  @override
  String get agentSessions => 'Lokale Sitzungen';

  @override
  String get allAgents => 'Alle Agenten';

  @override
  String get noAgentSessions => 'Keine lokalen Sitzungen gefunden';

  @override
  String unreadableSessions(int count) {
    return 'Nicht lesbare Sitzungsdateien ($count)';
  }

  @override
  String get copySessionId => 'Sitzungs-ID kopieren';

  @override
  String get sessionWorkingFolder => 'Arbeitsordner';

  @override
  String get sessionProfileFolder => 'Profilordner';

  @override
  String get distillTitle => 'Erinnerungen ableiten';

  @override
  String get distillPreview =>
      'Prüfe die Benutzernachrichten unten, bevor du sie an die gewählte Modellroute sendest. Neue Erinnerungen werden zur Prüfung gespeichert.';

  @override
  String get distillSend => 'An Modell senden';

  @override
  String get distillEmpty =>
      'Es wurden keine neuen Erinnerungsvorschläge gefunden.';

  @override
  String get memorySourceQuote => 'Zitat aus der Sitzung';

  @override
  String get agentInstructions => 'Benutzeranweisungen';

  @override
  String get agentMemoryDigest => 'Benutzergedächtnis';

  @override
  String get agentMemoryDigestHelp =>
      'Beim Veröffentlichen werden Einträge aus dem globalen Gedächtnis und dem Gedächtnis dieses Agenten in seine Benutzeranweisungen geschrieben.';

  @override
  String get agentMemoryDigestEmpty =>
      'Es gibt keine aktiven globalen oder agentenspezifischen Einträge zum Veröffentlichen.';

  @override
  String get projectMemory => 'Projektgedächtnis';

  @override
  String get publishMemory => 'Zusammenfassung veröffentlichen';

  @override
  String get projectMemoryHelp =>
      'Die Zusammenfassung wird in die Projektanweisungen geschrieben. Alle mit Zugriff auf die Datei können sie lesen.';

  @override
  String get projectMemoryEmpty =>
      'Dieses Projekt hat keine aktiven Erinnerungen zum Veröffentlichen.';

  @override
  String get searchMemories => 'Gedächtnis durchsuchen';

  @override
  String get unreadableMemories => 'Nicht lesbare Gedächtnisdateien';

  @override
  String get noMemoriesFound => 'Keine passenden Einträge';

  @override
  String get addMemory => 'Eintrag hinzufügen';

  @override
  String get editMemory => 'Eintrag bearbeiten';

  @override
  String get memoryScope => 'Geltungsbereich';

  @override
  String get memoryTags => 'Schlagwörter, eines pro Zeile';

  @override
  String get memoryWhy => 'Begründung (optional)';

  @override
  String get memoryKindLabel => 'Typ';

  @override
  String get memoryStateLabel => 'Status';

  @override
  String get pinMemory => 'Eintrag anheften';

  @override
  String memoryKind(String value) {
    String _temp0 = intl.Intl.selectLogic(value, {
      'profile': 'Profil',
      'preference': 'Vorliebe',
      'rule': 'Regel',
      'reference': 'Referenz',
      'fact': 'Fakt',
      'procedure': 'Vorgehen',
      'episode': 'Ereignis',
      'other': 'Typ',
    });
    return '$_temp0';
  }

  @override
  String memoryState(String value) {
    String _temp0 = intl.Intl.selectLogic(value, {
      'active': 'Aktiv',
      'candidate': 'Zu prüfen',
      'archived': 'Archiviert',
      'superseded': 'Ersetzt',
      'other': 'Alle Status',
    });
    return '$_temp0';
  }

  @override
  String get projectInstructions => 'Projektanweisungen';

  @override
  String get projectFolder => 'Projektordner';

  @override
  String get openProject => 'Projekt öffnen';

  @override
  String get restoreInstructions => 'Vorherige Version wiederherstellen';

  @override
  String get relayFallback => 'Relay-Adresse bei Verbindungsfehlern (optional)';

  @override
  String get listView => 'Listenansicht';

  @override
  String get gridView => 'Rasteransicht';

  @override
  String get gatewayProtocol => 'Gateway-Protokoll';

  @override
  String get nativeModelProtocol =>
      'Natives Protokoll des jeweiligen Modells verwenden';

  @override
  String get roleModels => 'Modelle für Rollen';

  @override
  String get roleModelId => 'Modell-ID';

  @override
  String get resetRole => 'Rolle zurücksetzen';

  @override
  String get syncModelCatalog => 'Modellkatalog synchronisieren';

  @override
  String get thinkingLevels => 'Denkstufen';

  @override
  String get sessionThinking => 'Denkstufe der Sitzung';

  @override
  String get thinkingAuto => 'Automatisch';

  @override
  String get thinkingMinimal => 'Minimal';

  @override
  String get thinkingLow => 'Niedrig';

  @override
  String get thinkingMedium => 'Mittel';

  @override
  String get thinkingHigh => 'Hoch';

  @override
  String get thinkingExtraHigh => 'Sehr hoch';

  @override
  String get thinkingMax => 'Maximal';

  @override
  String get ompProfile => 'Oh My Pi-Profil';

  @override
  String get advisorSettings => 'Beratereinstellungen';

  @override
  String get advisorReview => 'Zeitpunkt der Prüfung';

  @override
  String get advisorWait => 'Ausstehende Prüfungen bis zur Pause';

  @override
  String get advisorWake => 'Mindestschweregrad zum Fortsetzen';

  @override
  String get inheritSetting => 'Profilstandard verwenden';

  @override
  String get everyTurn => 'Nach jeder Runde';

  @override
  String get runEnd => 'Am Ende des Laufs';

  @override
  String get allReviews => 'Alle Prüfungen abwarten';

  @override
  String get minorNote => 'Kleinigkeit';

  @override
  String get concernNote => 'Bedenken';

  @override
  String get blockerNote => 'Blockierendes Problem';

  @override
  String get memberPatterns => 'Modellmuster, eines pro Zeile';

  @override
  String get pluginBundles => 'Gemeinsame Plug-in-Bibliothek';

  @override
  String get piProjectPackages => 'Pi-Projektpakete';

  @override
  String get claudeProjectPackages => 'Claude Code-Projektplugins';

  @override
  String get codexProjectPackages => 'Codex-Projektplugins';

  @override
  String get antigravityProjectPackages => 'Antigravity-Projektplugins';

  @override
  String get workspacePluginConfirm =>
      'Der Client kann beim Öffnen dieses vertrauenswürdigen Projekts Code aus dem geprüften Plugin ausführen.';

  @override
  String get claudePackageConfirm =>
      'Claude Code installiert das geprüfte Plugin für dieses Projekt. Beim Öffnen des Projekts kann Claude Code die Hooks des Plugins ausführen.';

  @override
  String get piPackageConfirm =>
      'Pi kann beim Öffnen eines vertrauenswürdigen Projekts Paketerweiterungen ausführen.';

  @override
  String get piExportAssignments => 'Zuweisungen exportieren';

  @override
  String get piApplyAssignments => 'Zuweisungen anwenden';

  @override
  String get piAssignmentsTooLarge =>
      'Die Zuweisungsdatei ist größer als 1 MiB.';

  @override
  String piPackageStatus(String status) {
    String _temp0 = intl.Intl.selectLogic(status, {
      'available': 'Zur Zuweisung verfügbar',
      'applied': 'Zugewiesen',
      'pending': 'Bereit zur Synchronisierung',
      'detached': 'Paket aus der Bibliothek entfernt',
      'conflict': 'Vorhandenen Eintrag beibehalten',
      'other': '',
    });
    return '$_temp0';
  }

  @override
  String get importPluginBundle => 'Plug-in-Paket importieren';

  @override
  String get previewPluginBundle => 'Paketvorschau';

  @override
  String get pluginBundleSource => 'Plug-in-Ordner, Manifest oder Git-URL';

  @override
  String get noPluginBundles => 'Keine Plug-in-Pakete gespeichert.';

  @override
  String get otherPluginEntries => 'Weitere Manifesteinträge';

  @override
  String get noBundleResources => 'Diese Datei enthält keine Ressourcen.';

  @override
  String get copyBundleResource => 'In Bibliothek kopieren';

  @override
  String get bundleResourceCopied => 'In Bibliothek kopiert';

  @override
  String get openPluginArchive => 'TAR-Archiv öffnen';

  @override
  String get exportPluginArchive => 'Plugin-Archiv exportieren';

  @override
  String get pluginArchiveSaved => 'Plugin-Archiv gespeichert';

  @override
  String get pluginCatalog => 'Plug-ins durchsuchen';

  @override
  String get noAvailablePlugins =>
      'Auf diesen Marktplätzen sind keine Plug-ins verfügbar.';

  @override
  String get exportPluginSettings => 'Plug-in-Einstellungen exportieren';

  @override
  String get importPluginSettings => 'Plug-in-Einstellungen importieren';

  @override
  String get claudePlugins => 'Claude Code-Plug-ins';

  @override
  String get pluginMarketplaces => 'Marktplätze';

  @override
  String get addPluginMarketplace => 'Marktplatz hinzufügen';

  @override
  String get pluginSource => 'Marktplatzpfad oder URL';

  @override
  String get installPlugin => 'Plug-in installieren';

  @override
  String get updatePlugin => 'Plug-in aktualisieren';

  @override
  String get enablePlugin => 'Plug-in aktivieren';

  @override
  String get disablePlugin => 'Plug-in deaktivieren';

  @override
  String get pluginEnabled => 'Aktiviert';

  @override
  String get noClaudePlugins =>
      'Für diesen Bereich sind keine Claude Code-Plug-ins installiert.';

  @override
  String get shareGroup => 'Gruppe zum Teilen kopieren';

  @override
  String get groupCopied => 'Gruppe in die Zwischenablage kopiert';

  @override
  String get importGroup => 'Gruppe importieren';

  @override
  String get sharedGroupText => 'Text der geteilten Gruppe';

  @override
  String get replaceGroup => 'Vorhandene Gruppe mit derselben ID ersetzen';

  @override
  String get missingMembers => 'Nicht verfügbare Mitglieder';

  @override
  String get replacedMembers => 'Ersetzte Mitglieder';

  @override
  String routeSession(String id) {
    return 'Sitzung $id';
  }

  @override
  String get enableGroup => 'Gruppe aktivieren';

  @override
  String get disableGroup => 'Gruppe deaktivieren';

  @override
  String get listGroupModels =>
      'Modelle der Gruppe in Client-Katalogen anzeigen';

  @override
  String get copyGroup => 'Gruppe duplizieren';

  @override
  String get groupNote => 'Gruppennotiz (optional)';

  @override
  String get firstContentSeconds =>
      'Zeitlimit bis zum ersten Inhalt (Sekunden)';

  @override
  String get groupFirstContentHelp =>
      '0 übernimmt die übergeordnete Einstellung. Bei -1 wird auf das Mitglied gewartet.';

  @override
  String get defaultFirstContentHelp =>
      'Vorgabe für Routinggruppen. Bei 0 oder -1 wird auf jedes Mitglied gewartet.';

  @override
  String get invalidFirstContent =>
      'Geben Sie eine ganze Zahl von -1 bis 2147483647 ein.';

  @override
  String get groupPatienceSeconds => 'Wiederholungszeit (Sekunden)';

  @override
  String get groupPatienceHelp =>
      '0 übernimmt die Vorgabe, -1 deaktiviert Wiederholungen.';

  @override
  String get defaultGroupPatienceHelp =>
      '0 entspricht 60 Sekunden, -1 deaktiviert Wiederholungen.';

  @override
  String get invalidGroupPatience =>
      'Geben Sie eine ganze Zahl von -1 bis 240 ein.';

  @override
  String get enableMember => 'Mitglied aktivieren';

  @override
  String get disableMember => 'Mitglied deaktivieren';

  @override
  String get manualRouting => 'Manuelle Auswahl';

  @override
  String get selectMember => 'Dieses Mitglied auswählen';

  @override
  String get providerCatalogs => 'models.dev-Katalog-IDs (optional)';

  @override
  String get anthropicAuthHeader => 'Header für den Anthropic-API-Schlüssel';

  @override
  String get automaticAuthHeader => 'Automatisch';

  @override
  String get autoGroups => 'Gleiche Modelle verschiedener Anbieter gruppieren';

  @override
  String get restoreGroup => 'Gruppe wiederherstellen';

  @override
  String get vertexProject => 'Vertex-AI-Projekteinstellungen';

  @override
  String get cloudProjectId => 'Google-Cloud-Projekt-ID';

  @override
  String get vertexLocation => 'Vertex-AI-Region';

  @override
  String get googleCredentials => 'ADC-Datei (optional)';

  @override
  String get impersonateAccount => 'Zu verwendendes Dienstkonto (optional)';

  @override
  String get models => 'Modelle';

  @override
  String get modelCopied => 'Modell-ID kopiert';

  @override
  String get noModelsFound => 'Keine passenden Modelle';

  @override
  String get routingStrategy => 'Routingstrategie';

  @override
  String get listedOrder => 'Listenreihenfolge';

  @override
  String get weightedOrder => 'Gewichtetes Round-Robin';

  @override
  String get memberWeight => 'Gewicht des Mitglieds';

  @override
  String get invalidWeight => 'Ganze Zahl von 1 bis 1000 eingeben.';

  @override
  String get providers => 'Anbieter';

  @override
  String get routes => 'Routen';

  @override
  String get groups => 'Routengruppen';

  @override
  String get gateway => 'Gateway';

  @override
  String get language => 'Sprache';

  @override
  String get systemLanguage => 'Systemsprache';

  @override
  String confirmRemove(String value) {
    return '$value entfernen?';
  }

  @override
  String get cancel => 'Abbrechen';

  @override
  String get remove => 'Entfernen';

  @override
  String get save => 'Speichern';

  @override
  String get webLogin => 'Web-Anmeldung';

  @override
  String get loginHelp =>
      'Im neuen Chrome-Fenster anmelden und anschließend hier das Fenster schließen.';

  @override
  String get closeLogin => 'Anmeldefenster schließen';

  @override
  String get duplicateGroup => 'Diese Gruppen-ID ist bereits vergeben.';

  @override
  String get addProvider => 'Anbieter hinzufügen';

  @override
  String get newGroup => 'Neue Gruppe';

  @override
  String get stopToEdit =>
      'Zum Bearbeiten der Konfiguration das Gateway anhalten.';

  @override
  String get reload => 'Neu laden';

  @override
  String get reloadConfig => 'Konfiguration neu laden';

  @override
  String providerCount(int count) {
    return 'Anbieter: $count';
  }

  @override
  String presetCount(int count) {
    return 'Aus $count API-Vorlagen wählen oder eine eigene Basis-URL eingeben.';
  }

  @override
  String get providerHelp =>
      'Eine Upstream-URL und Modell-ID hinzufügen, um Anfragen zu ermöglichen.';

  @override
  String get removeProvider => 'Anbieter entfernen';

  @override
  String get webProviders => 'Webanbieter';

  @override
  String get webHelp =>
      'Chrome mit passendem ChromeDriver verwenden. Den hinzugefügten Anbieter auswählen, um sich in seinem Browserprofil anzumelden.';

  @override
  String webSite(String value) {
    return '$value Web';
  }

  @override
  String get groupNeedsProvider =>
      'Vor dem Erstellen einer Routengruppe einen Anbieter und ein Modell hinzufügen.';

  @override
  String get groupHelp =>
      'Mitglieder nach Priorität hinzufügen. Bei Ratenbegrenzung oder Serverfehlern versucht das Gateway das nächste Mitglied.';

  @override
  String get removeGroup => 'Gruppe entfernen';

  @override
  String recentAttempts(int count) {
    return 'Letzte Upstream-Versuche: $count';
  }

  @override
  String get historyLimit => 'Aktueller Gateway-Prozess · letzte 128 Versuche';

  @override
  String get waiting => 'Warten auf eine Clientanfrage.';

  @override
  String get historyHelp => 'Gateway starten, um den Routenverlauf zu sehen.';

  @override
  String get running => 'Gateway aktiv';

  @override
  String get stopped => 'Gateway angehalten';

  @override
  String get stop => 'Gateway anhalten';

  @override
  String get start => 'Gateway starten';

  @override
  String get keyCopied => 'Gateway-Schlüssel kopiert';

  @override
  String get copyKey => 'Clientschlüssel kopieren';

  @override
  String get networkSettings => 'Netzwerkeinstellungen';

  @override
  String get browserOrigins => 'Erlaubte Browser-Ursprünge (einer pro Zeile)';

  @override
  String get localPeer => 'Lokale Peer-ID';

  @override
  String get copyPeer => 'Peer-ID kopieren';

  @override
  String get peerCopied => 'Peer-ID kopiert';

  @override
  String get listenAddress => 'Lauschadresse';

  @override
  String get disabled => 'Deaktiviert';

  @override
  String get trustedPeers => 'Vertrauenswürdige Peers';

  @override
  String get clients => 'Clientverbindungen';

  @override
  String get agentRoot => 'API-Stammadresse des Agenten';

  @override
  String get copyRoot => 'Stammadresse kopieren';

  @override
  String get rootCopied => 'Stammadresse kopiert';

  @override
  String get agentHelp =>
      'Für Chat Completions und Responses /v1 anhängen. Im Routenverlauf erscheint der gewählte Agent.';

  @override
  String get tunnel => 'Tunnel zum entfernten Gateway';

  @override
  String get disconnected => 'Getrennt';

  @override
  String get connect => 'Verbinden';

  @override
  String get disconnect => 'Trennen';

  @override
  String get apiBase => 'API-Basis-URL';

  @override
  String get copyBase => 'Basis-URL kopieren';

  @override
  String get baseCopied => 'API-Basis-URL kopiert';

  @override
  String get changed =>
      'Anbietereinstellungen geändert. Modelle erneut abrufen.';

  @override
  String modelsRead(int count) {
    return 'Gefundene Modelle: $count. Die Liste lässt sich vor dem Speichern bearbeiten.';
  }

  @override
  String get presetLoaded =>
      'Vorlagenmodelle geladen. Liste an die Kontoberechtigungen anpassen.';

  @override
  String get editProvider => 'Anbieter bearbeiten';

  @override
  String get searchPresets => 'Vorlagen nach Namen suchen';

  @override
  String get providerId => 'Anbieter-ID';

  @override
  String get renameProvider => 'Anbieter umbenennen';

  @override
  String get keyEnv => 'Umgebungsvariable für den API-Schlüssel';

  @override
  String get keyHint => 'Optional für lokale Dienste oder Bedrock mit AWS IAM';

  @override
  String get resourceRoot => 'Stammadresse der Ressource';

  @override
  String get modelIds => 'Upstream-Modell-IDs, eine pro Zeile';

  @override
  String get providerFallback =>
      'Ersatzmodelle oder Gruppen, in Reihenfolge, eines pro Zeile';

  @override
  String get allowProviderFallback =>
      'Ersatzanfragen anderer Anbieter annehmen';

  @override
  String protocolBase(String value) {
    return 'Basis-URL für $value';
  }

  @override
  String get catalogUrl => 'Modellkatalog-URL (optional)';

  @override
  String get catalogHint => 'Leer lassen für den Standardkatalog des Anbieters';

  @override
  String get readingModels => 'Modelle werden geladen';

  @override
  String get readModels => 'Modellkatalog laden';

  @override
  String get requiredField => 'Dieses Feld ist erforderlich.';

  @override
  String get createGroup => 'Routengruppe erstellen';

  @override
  String get editGroup => 'Routengruppe bearbeiten';

  @override
  String get groupId => 'Routengruppen-ID';

  @override
  String get raisePriority => 'Priorität erhöhen';

  @override
  String get removeMember => 'Mitglied entfernen';

  @override
  String get addCandidate => 'Kandidatenmodell hinzufügen';

  @override
  String get connectRemote => 'Mit entferntem Gateway verbinden';

  @override
  String get remoteQuic => 'Entfernte QUIC-Adresse (mit /p2p/PeerID)';

  @override
  String get localTunnel => 'Lokale Tunnel-Lauschadresse';

  @override
  String get tunnelHelp =>
      'Diese Peer-ID am entfernten Gateway zulassen. Clients am lokalen Tunnel verwenden den entfernten Gateway-Schlüssel.';

  @override
  String get loopback => 'HTTP-Loopback-Adresse';

  @override
  String get peerListen => 'P2P-Lauschadresse (leer zum Deaktivieren)';

  @override
  String get allowedPeers => 'Erlaubte Peer-IDs, eine pro Zeile';

  @override
  String get rotateMembers => 'Erstes Mitglied abwechseln';
}
