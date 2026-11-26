// ignore: unused_import
import 'package:intl/intl.dart' as intl;

import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for German (`de`).
class AppLocalizationsDe extends AppLocalizations {
  AppLocalizationsDe([String locale = 'de']) : super(locale);

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
