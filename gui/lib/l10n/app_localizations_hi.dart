// ignore: unused_import
import 'package:intl/intl.dart' as intl;

import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for Hindi (`hi`).
class AppLocalizationsHi extends AppLocalizations {
  AppLocalizationsHi([String locale = 'hi']) : super(locale);

  @override
  String get importSubagents => 'सबएजेंट आयात करें';

  @override
  String get subagentImportSource =>
      'Markdown फ़ाइल, स्थानीय फ़ोल्डर या Git URL';

  @override
  String get subagentChooseFile => 'Markdown फ़ाइल चुनें';

  @override
  String get subagentChooseFolder => 'स्रोत फ़ोल्डर चुनें';

  @override
  String get subagentScanSource => 'स्रोत स्कैन करें';

  @override
  String get subagentImportTruncated =>
      'स्रोत का केवल एक भाग स्कैन किया गया। अधिक फ़ाइलें देखने के लिए छोटा फ़ोल्डर चुनें।';

  @override
  String subagentImportStatus(String status) {
    String _temp0 = intl.Intl.selectLogic(status, {
      'update': 'अपडेट उपलब्ध है',
      'unchanged': 'पहले से आयातित',
      'edited': 'स्थानीय बदलाव सुरक्षित हैं',
      'other': '',
    });
    return '$_temp0';
  }

  @override
  String get findNativeResources => 'मौजूदा फ़ाइलें ढूँढें';

  @override
  String get noNativeResources => 'कोई मौजूदा फ़ाइल नहीं मिली';

  @override
  String get nativeResourceScanLimit =>
      'मिली हुई पहली 512 फ़ाइलें दिखाई जा रही हैं।';

  @override
  String get adoptResourceDetails =>
      'Crabforge इस फ़ाइल को उसके मौजूदा स्थान पर प्रबंधित करेगा। क्लाइंट से असाइनमेंट हटाने पर बिना बदलाव वाली फ़ाइल हट सकती है।';

  @override
  String get agentSubagents => 'उप-एजेंट';

  @override
  String get agentCommands => 'स्लैश कमांड';

  @override
  String get addSubagent => 'उप-एजेंट जोड़ें';

  @override
  String get addSlashCommand => 'स्लैश कमांड जोड़ें';

  @override
  String get resourceUserScope => 'उपयोगकर्ता प्रोफ़ाइल';

  @override
  String get noAgentResources => 'कोई प्रविष्टि सहेजी नहीं गई है';

  @override
  String get resourcePlacements => 'प्रकाशन का विवरण';

  @override
  String get agentResourceName => 'नाम';

  @override
  String get agentResourceDescription => 'विवरण';

  @override
  String get agentResourcePrompt => 'प्रॉम्प्ट';

  @override
  String get resourceSkillNames => 'कौशल के नाम, प्रति पंक्ति एक';

  @override
  String get resourceMcpNames => 'MCP सर्वर के नाम, प्रति पंक्ति एक';

  @override
  String get commandArgumentHelp =>
      'सभी आर्ग्युमेंट के लिए \$ARGUMENTS और संगत क्लाइंट में अलग-अलग आर्ग्युमेंट के लिए \$1, \$2 आदि इस्तेमाल करें।';

  @override
  String get agentHooks => 'हुक';

  @override
  String get addHook => 'हुक जोड़ें';

  @override
  String get editHook => 'हुक संपादित करें';

  @override
  String get adoptHook => 'हुक का प्रबंधन करें';

  @override
  String get noHooks => 'कोई हुक सहेजा नहीं गया है';

  @override
  String get installedHooks => 'क्लाइंट सेटिंग में हुक';

  @override
  String get removeInstalledHook => 'इंस्टॉल किया गया हुक हटाएँ';

  @override
  String get hookBackups => 'हुक बैकअप';

  @override
  String get restoreHookBackup => 'हुक बैकअप बहाल करें';

  @override
  String get hookBackupRestored => 'बहाल किया गया';

  @override
  String get hookBackupChanged => 'क्लाइंट सेटिंग बदल गई हैं';

  @override
  String get hookBackupUnavailable => 'मूल प्रोफ़ाइल या फ़ाइल उपलब्ध नहीं है';

  @override
  String get hookName => 'हुक का नाम';

  @override
  String get hookDescription => 'विवरण (वैकल्पिक)';

  @override
  String get hookEvent => 'इवेंट';

  @override
  String get hookMatcher => 'मिलान की शर्त';

  @override
  String get hookAllTools => 'सभी टूल';

  @override
  String get hookShellTool => 'शेल कमांड';

  @override
  String get hookEditTool => 'फ़ाइल संपादन';

  @override
  String get hookCommand => 'क्लाइंट द्वारा चलाई जाने वाली कमांड';

  @override
  String get hookTimeout => 'सेकंड में समय सीमा (वैकल्पिक)';

  @override
  String get hookTurnEnd => 'टर्न का अंत';

  @override
  String get hookAfterTool => 'टूल के इस्तेमाल के बाद';

  @override
  String get hookBeforeTool => 'टूल के इस्तेमाल से पहले';

  @override
  String get hookPrompt => 'प्रॉम्प्ट भेजने पर';

  @override
  String get hookSessionStart => 'सेशन शुरू होने पर';

  @override
  String get hookExisting => 'ऐसा हुक पहले से मौजूद है';

  @override
  String get hookConflict => 'मौजूदा हुक बनाए रखे गए';

  @override
  String get agentEnvironment => 'एनवायरनमेंट वैरिएबल';

  @override
  String get noEnvironmentVariables => 'कोई वैरिएबल सेव नहीं है';

  @override
  String get addVariable => 'वैरिएबल जोड़ें';

  @override
  String get editVariable => 'वैरिएबल संपादित करें';

  @override
  String get variableName => 'वैरिएबल का नाम';

  @override
  String get variableValue => 'मान';

  @override
  String get showVariableValue => 'मान दिखाएँ';

  @override
  String get hideVariableValue => 'मान छिपाएँ';

  @override
  String get nativeVariableExists => 'पहले से सेट है';

  @override
  String get nativeVariableConflict => 'मौजूदा मान रखा गया';

  @override
  String get memoryLibrary => 'स्मृति';

  @override
  String get agentRules => 'नियम';

  @override
  String get agentSkills => 'कौशल';

  @override
  String get importSkill => 'कौशल आयात करें';

  @override
  String get applySkills => 'कौशल लागू करें';

  @override
  String get noSkills => 'कोई कौशल आयात नहीं हुआ';

  @override
  String get editSkill => 'कौशल असाइनमेंट संपादित करें';

  @override
  String get skillSourceFolder => 'स्थानीय फ़ोल्डर या Git URL';

  @override
  String get checkSkillUpdate => 'अपडेट देखें';

  @override
  String get browseRepositorySkills => 'स्रोत में स्किल देखें';

  @override
  String get noRepositorySkills => 'इस स्रोत में कोई और स्किल नहीं मिला।';

  @override
  String get dismissSkillCandidate => 'स्किल छिपाएँ';

  @override
  String get skillUpToDate => 'स्किल की फ़ाइलें अप टू डेट हैं।';

  @override
  String get skillUpdateAvailable =>
      'स्किल की नई फ़ाइलें उपलब्ध हैं। उन्हें लागू करने के लिए फ़ाइलें फिर से इंपोर्ट करें।';

  @override
  String get skillRemovedUpstream =>
      'चुने हुए फ़ोल्डर में अब SKILL.md नहीं है।';

  @override
  String get installedSkills => 'इंस्टॉल किए गए स्किल';

  @override
  String get adoptSkill => 'Crabforge में प्रबंधित करें';

  @override
  String get adoptSkillHint =>
      'Crabforge इन फ़ाइलों की एक कॉपी सहेजेगा और इस फ़ोल्डर को प्रबंधित करेगा। असाइनमेंट हटाने पर अपरिवर्तित फ़ोल्डर हटाया जा सकता है।';

  @override
  String get mcpLibrary => 'MCP लाइब्रेरी';

  @override
  String get assignMcpServer => 'इस प्रोजेक्ट को असाइन करें';

  @override
  String get unassignMcpServer => 'इस प्रोजेक्ट से असाइनमेंट हटाएँ';

  @override
  String get syncMcpLibrary => 'असाइनमेंट सिंक करें';

  @override
  String get refreshSkill => 'फ़ाइलें फिर से आयात करें';

  @override
  String get projectMcp => 'प्रोजेक्ट MCP';

  @override
  String get addMcpServer => 'MCP सर्वर जोड़ें';

  @override
  String get editMcpServer => 'MCP सर्वर संपादित करें';

  @override
  String get noMcpServers => 'कोई MCP सर्वर कॉन्फ़िगर नहीं है';

  @override
  String get mcpServerName => 'सर्वर का नाम';

  @override
  String get addRule => 'नियम जोड़ें';

  @override
  String get editRule => 'नियम संपादित करें';

  @override
  String get syncRules => 'नियम लागू करें';

  @override
  String get noRules => 'कोई नियम सहेजा नहीं गया है';

  @override
  String get ruleName => 'नियम का नाम';

  @override
  String get ruleBody => 'Markdown निर्देश';

  @override
  String get rulePaths => 'पथ पैटर्न (वैकल्पिक, हर पंक्ति में एक)';

  @override
  String get ruleRequired => 'यह फ़ील्ड ज़रूरी है।';

  @override
  String ruleStatus(String value) {
    String _temp0 = intl.Intl.selectLogic(value, {
      'applied': 'लागू',
      'pending': 'लंबित',
      'removed': 'हटाया गया',
      'conflict': 'मौजूदा फ़ाइल सुरक्षित रखी गई',
      'other': 'लागू नहीं हो सका',
    });
    return '$_temp0';
  }

  @override
  String get agentSessions => 'स्थानीय सत्र';

  @override
  String get allAgents => 'सभी एजेंट';

  @override
  String get noAgentSessions => 'कोई स्थानीय सत्र नहीं मिला';

  @override
  String unreadableSessions(int count) {
    return 'ऐसी सत्र फ़ाइलें जिन्हें पढ़ा नहीं जा सका ($count)';
  }

  @override
  String get copySessionId => 'सत्र आईडी कॉपी करें';

  @override
  String get sessionWorkingFolder => 'कार्य फ़ोल्डर';

  @override
  String get sessionProfileFolder => 'प्रोफ़ाइल फ़ोल्डर';

  @override
  String get distillTitle => 'स्मृतियाँ निकालें';

  @override
  String get distillPreview =>
      'चुने गए मॉडल रूट पर भेजने से पहले नीचे दिए गए उपयोगकर्ता संदेशों की समीक्षा करें। नई स्मृतियाँ समीक्षा के लिए सहेजी जाएँगी।';

  @override
  String get distillSend => 'मॉडल को भेजें';

  @override
  String get distillEmpty => 'समीक्षा के लिए कोई नई स्मृति नहीं मिली।';

  @override
  String get memorySourceQuote => 'सत्र से उद्धरण';

  @override
  String get agentInstructions => 'उपयोगकर्ता निर्देश';

  @override
  String get agentMemoryDigest => 'उपयोगकर्ता की स्मृति';

  @override
  String get agentMemoryDigestHelp =>
      'प्रकाशित करने पर वैश्विक स्मृति और इस एजेंट की स्मृति उसके उपयोगकर्ता निर्देशों में लिखी जाती हैं।';

  @override
  String get agentMemoryDigestEmpty =>
      'प्रकाशित करने के लिए कोई सक्रिय वैश्विक या एजेंट स्मृति नहीं है।';

  @override
  String get projectMemory => 'प्रोजेक्ट की स्मृति';

  @override
  String get publishMemory => 'सारांश प्रकाशित करें';

  @override
  String get projectMemoryHelp =>
      'सारांश प्रोजेक्ट के निर्देशों में लिखा जाता है। फ़ाइल तक पहुँच रखने वाला कोई भी व्यक्ति उसे पढ़ सकता है।';

  @override
  String get projectMemoryEmpty =>
      'इस प्रोजेक्ट में प्रकाशित करने के लिए कोई सक्रिय स्मृति नहीं है।';

  @override
  String get searchMemories => 'स्मृतियाँ खोजें';

  @override
  String get unreadableMemories => 'पढ़ी न जा सकने वाली स्मृति फ़ाइलें';

  @override
  String get noMemoriesFound => 'कोई मेल खाती स्मृति नहीं मिली';

  @override
  String get addMemory => 'स्मृति जोड़ें';

  @override
  String get editMemory => 'स्मृति संपादित करें';

  @override
  String get memoryScope => 'दायरा';

  @override
  String get memoryTags => 'टैग, हर पंक्ति में एक';

  @override
  String get memoryWhy => 'कारण (वैकल्पिक)';

  @override
  String get memoryKindLabel => 'प्रकार';

  @override
  String get memoryStateLabel => 'स्थिति';

  @override
  String get pinMemory => 'स्मृति पिन करें';

  @override
  String memoryKind(String value) {
    String _temp0 = intl.Intl.selectLogic(value, {
      'profile': 'प्रोफ़ाइल',
      'preference': 'पसंद',
      'rule': 'नियम',
      'reference': 'संदर्भ',
      'fact': 'तथ्य',
      'procedure': 'प्रक्रिया',
      'episode': 'घटना',
      'other': 'प्रकार',
    });
    return '$_temp0';
  }

  @override
  String memoryState(String value) {
    String _temp0 = intl.Intl.selectLogic(value, {
      'active': 'सक्रिय',
      'candidate': 'समीक्षा बाकी',
      'archived': 'संग्रहीत',
      'superseded': 'प्रतिस्थापित',
      'other': 'सभी स्थितियाँ',
    });
    return '$_temp0';
  }

  @override
  String get projectInstructions => 'प्रोजेक्ट निर्देश';

  @override
  String get projectFolder => 'प्रोजेक्ट फ़ोल्डर';

  @override
  String get openProject => 'प्रोजेक्ट खोलें';

  @override
  String get restoreInstructions => 'पिछला संस्करण बहाल करें';

  @override
  String get relayFallback =>
      'सीधा कनेक्शन विफल होने पर रिले सर्किट का पता (वैकल्पिक)';

  @override
  String get listView => 'सूची दृश्य';

  @override
  String get gridView => 'ग्रिड दृश्य';

  @override
  String get gatewayProtocol => 'गेटवे प्रोटोकॉल';

  @override
  String get nativeModelProtocol => 'हर मॉडल का मूल प्रोटोकॉल इस्तेमाल करें';

  @override
  String get roleModels => 'भूमिका के लिए मॉडल';

  @override
  String get roleModelId => 'मॉडल आईडी';

  @override
  String get resetRole => 'भूमिका रीसेट करें';

  @override
  String get syncModelCatalog => 'मॉडल सूची सिंक करें';

  @override
  String get thinkingLevels => 'तर्क के स्तर';

  @override
  String get sessionThinking => 'सत्र का तर्क स्तर';

  @override
  String get thinkingAuto => 'स्वचालित';

  @override
  String get thinkingMinimal => 'न्यूनतम';

  @override
  String get thinkingLow => 'कम';

  @override
  String get thinkingMedium => 'मध्यम';

  @override
  String get thinkingHigh => 'उच्च';

  @override
  String get thinkingExtraHigh => 'बहुत उच्च';

  @override
  String get thinkingMax => 'अधिकतम';

  @override
  String get ompProfile => 'Oh My Pi प्रोफ़ाइल';

  @override
  String get advisorSettings => 'सलाहकार सेटिंग';

  @override
  String get advisorReview => 'समीक्षा का समय';

  @override
  String get advisorWait => 'प्रतीक्षा से पहले लंबित समीक्षाओं की संख्या';

  @override
  String get advisorWake => 'फिर से शुरू करने के लिए न्यूनतम गंभीरता';

  @override
  String get inheritSetting => 'डिफ़ॉल्ट सेटिंग का उपयोग करें';

  @override
  String get everyTurn => 'हर चरण पर';

  @override
  String get runEnd => 'रन के अंत में';

  @override
  String get allReviews => 'सभी समीक्षाओं की प्रतीक्षा करें';

  @override
  String get minorNote => 'मामूली समस्या';

  @override
  String get concernNote => 'चिंता का विषय';

  @override
  String get blockerNote => 'बाधा';

  @override
  String get memberPatterns => 'मॉडल पैटर्न, हर पंक्ति में एक';

  @override
  String get shareGroup => 'साझा करने के लिए समूह कॉपी करें';

  @override
  String get groupCopied => 'समूह क्लिपबोर्ड पर कॉपी किया गया';

  @override
  String get importGroup => 'समूह आयात करें';

  @override
  String get sharedGroupText => 'साझा किए गए समूह का टेक्स्ट';

  @override
  String get replaceGroup => 'समान आईडी वाले मौजूदा समूह को बदलें';

  @override
  String get missingMembers => 'अनुपलब्ध सदस्य';

  @override
  String get replacedMembers => 'बदले गए सदस्य';

  @override
  String routeSession(String id) {
    return 'सत्र $id';
  }

  @override
  String get enableGroup => 'समूह चालू करें';

  @override
  String get disableGroup => 'समूह बंद करें';

  @override
  String get listGroupModels => 'क्लाइंट की मॉडल सूची में समूह के मॉडल दिखाएँ';

  @override
  String get copyGroup => 'समूह की प्रतिलिपि बनाएँ';

  @override
  String get groupNote => 'समूह का नोट (वैकल्पिक)';

  @override
  String get enableMember => 'सदस्य सक्रिय करें';

  @override
  String get disableMember => 'सदस्य निष्क्रिय करें';

  @override
  String get manualRouting => 'मैन्युअल चयन';

  @override
  String get selectMember => 'यह सदस्य चुनें';

  @override
  String get providerCatalogs => 'models.dev कैटलॉग आईडी (वैकल्पिक)';

  @override
  String get anthropicAuthHeader => 'Anthropic API कुंजी का हेडर';

  @override
  String get automaticAuthHeader => 'स्वचालित चयन';

  @override
  String get autoGroups => 'अलग-अलग प्रदाताओं के समान मॉडल का समूह बनाएँ';

  @override
  String get restoreGroup => 'समूह बहाल करें';

  @override
  String get vertexProject => 'Vertex AI प्रोजेक्ट सेटिंग';

  @override
  String get cloudProjectId => 'Google Cloud प्रोजेक्ट आईडी';

  @override
  String get vertexLocation => 'Vertex AI क्षेत्र';

  @override
  String get googleCredentials => 'ADC फ़ाइल (वैकल्पिक)';

  @override
  String get impersonateAccount => 'जिस सेवा खाते का उपयोग करना है (वैकल्पिक)';

  @override
  String get models => 'मॉडल';

  @override
  String get modelCopied => 'मॉडल आईडी कॉपी की गई';

  @override
  String get noModelsFound => 'कोई मेल खाता मॉडल नहीं मिला';

  @override
  String get routingStrategy => 'रूटिंग रणनीति';

  @override
  String get listedOrder => 'सूची का क्रम';

  @override
  String get weightedOrder => 'भारित राउंड रॉबिन';

  @override
  String get memberWeight => 'सदस्य का भार';

  @override
  String get invalidWeight => '1 से 1000 तक की पूर्ण संख्या दर्ज करें।';

  @override
  String get providers => 'प्रदाता';

  @override
  String get routes => 'रूट';

  @override
  String get groups => 'रूट समूह';

  @override
  String get gateway => 'गेटवे';

  @override
  String get language => 'भाषा';

  @override
  String get systemLanguage => 'सिस्टम की भाषा';

  @override
  String confirmRemove(String value) {
    return '$value हटाएँ?';
  }

  @override
  String get cancel => 'रद्द करें';

  @override
  String get remove => 'हटाएँ';

  @override
  String get save => 'सहेजें';

  @override
  String get webLogin => 'वेब साइन-इन';

  @override
  String get loginHelp =>
      'नई Chrome विंडो में साइन इन करें, फिर उसे बंद करने के लिए यहाँ लौटें।';

  @override
  String get closeLogin => 'साइन-इन विंडो बंद करें';

  @override
  String get duplicateGroup => 'यह समूह आईडी पहले से मौजूद है।';

  @override
  String get addProvider => 'प्रदाता जोड़ें';

  @override
  String get newGroup => 'नया समूह';

  @override
  String get stopToEdit => 'सेटिंग बदलने के लिए गेटवे रोकें।';

  @override
  String get reload => 'फिर लोड करें';

  @override
  String get reloadConfig => 'सेटिंग फिर लोड करें';

  @override
  String providerCount(int count) {
    return 'प्रदाता: $count';
  }

  @override
  String presetCount(int count) {
    return '$count API प्रीसेट में से चुनें या अपना बेस URL दर्ज करें।';
  }

  @override
  String get providerHelp =>
      'क्लाइंट अनुरोध चालू करने के लिए अपस्ट्रीम URL और मॉडल आईडी जोड़ें।';

  @override
  String get removeProvider => 'प्रदाता हटाएँ';

  @override
  String get webProviders => 'वेब प्रदाता';

  @override
  String get webHelp =>
      'Chrome और उससे मेल खाता ChromeDriver इस्तेमाल करें। प्रदाता जोड़कर उसकी अलग ब्राउज़र प्रोफ़ाइल में साइन इन करें।';

  @override
  String webSite(String value) {
    return '$value वेब';
  }

  @override
  String get groupNeedsProvider =>
      'रूट समूह बनाने से पहले प्रदाता और मॉडल जोड़ें।';

  @override
  String get groupHelp =>
      'सदस्य प्राथमिकता क्रम में जोड़ें। दर सीमा या सर्वर त्रुटि पर गेटवे अगला सदस्य आज़माता है।';

  @override
  String get removeGroup => 'समूह हटाएँ';

  @override
  String recentAttempts(int count) {
    return 'हाल के अपस्ट्रीम प्रयास: $count';
  }

  @override
  String get historyLimit => 'वर्तमान गेटवे प्रक्रिया · नवीनतम 128 प्रयास';

  @override
  String get waiting => 'क्लाइंट अनुरोध की प्रतीक्षा है।';

  @override
  String get historyHelp => 'रूट इतिहास देखने के लिए गेटवे शुरू करें।';

  @override
  String get running => 'गेटवे चालू है';

  @override
  String get stopped => 'गेटवे रुका हुआ है';

  @override
  String get stop => 'गेटवे रोकें';

  @override
  String get start => 'गेटवे शुरू करें';

  @override
  String get keyCopied => 'गेटवे कुंजी कॉपी हुई';

  @override
  String get copyKey => 'क्लाइंट कुंजी कॉपी करें';

  @override
  String get networkSettings => 'नेटवर्क सेटिंग';

  @override
  String get browserOrigins => 'अनुमत ब्राउज़र ऑरिजिन (हर पंक्ति में एक)';

  @override
  String get localPeer => 'स्थानीय पीयर आईडी';

  @override
  String get copyPeer => 'पीयर आईडी कॉपी करें';

  @override
  String get peerCopied => 'पीयर आईडी कॉपी हुई';

  @override
  String get listenAddress => 'लिसन पता';

  @override
  String get disabled => 'बंद';

  @override
  String get trustedPeers => 'विश्वसनीय पीयर';

  @override
  String get clients => 'क्लाइंट कनेक्शन';

  @override
  String get agentRoot => 'एजेंट API का रूट URL';

  @override
  String get copyRoot => 'रूट URL कॉपी करें';

  @override
  String get rootCopied => 'रूट URL कॉपी हुआ';

  @override
  String get agentHelp =>
      'Chat Completions और Responses क्लाइंट के लिए /v1 जोड़ें। रूट इतिहास में चुना हुआ एजेंट दिखेगा।';

  @override
  String get tunnel => 'दूरस्थ गेटवे टनल';

  @override
  String get disconnected => 'कनेक्शन बंद';

  @override
  String get connect => 'कनेक्ट करें';

  @override
  String get disconnect => 'कनेक्शन काटें';

  @override
  String get apiBase => 'API बेस URL';

  @override
  String get copyBase => 'बेस URL कॉपी करें';

  @override
  String get baseCopied => 'API बेस URL कॉपी हुआ';

  @override
  String get changed => 'प्रदाता सेटिंग बदल गई हैं। मॉडल फिर लोड करें।';

  @override
  String modelsRead(int count) {
    return '$count मॉडल मिले। सहेजने से पहले सूची बदल सकते हैं।';
  }

  @override
  String get presetLoaded =>
      'प्रीसेट मॉडल लोड हो गए। खाते के अनुसार सूची बदलें।';

  @override
  String get editProvider => 'प्रदाता संपादित करें';

  @override
  String get searchPresets => 'नाम से प्रीसेट खोजें';

  @override
  String get providerId => 'प्रदाता आईडी';

  @override
  String get renameProvider => 'प्रदाता का नाम बदलें';

  @override
  String get keyEnv => 'API कुंजी का परिवेश चर';

  @override
  String get keyHint => 'स्थानीय सेवा या AWS IAM वाले Bedrock के लिए वैकल्पिक';

  @override
  String get resourceRoot => 'संसाधन का रूट URL';

  @override
  String get modelIds => 'अपस्ट्रीम मॉडल आईडी, हर पंक्ति में एक';

  @override
  String get providerFallback =>
      'वैकल्पिक मॉडल या समूह, क्रम से हर पंक्ति में एक';

  @override
  String get allowProviderFallback =>
      'अन्य प्रदाताओं के वैकल्पिक अनुरोध स्वीकार करें';

  @override
  String protocolBase(String value) {
    return '$value का बेस URL';
  }

  @override
  String get catalogUrl => 'मॉडल कैटलॉग URL (वैकल्पिक)';

  @override
  String get catalogHint => 'डिफ़ॉल्ट कैटलॉग के लिए खाली छोड़ें';

  @override
  String get readingModels => 'मॉडल लोड हो रहे हैं';

  @override
  String get readModels => 'मॉडल कैटलॉग लोड करें';

  @override
  String get requiredField => 'यह फ़ील्ड आवश्यक है।';

  @override
  String get createGroup => 'रूट समूह बनाएँ';

  @override
  String get editGroup => 'रूट समूह संपादित करें';

  @override
  String get groupId => 'रूट समूह आईडी';

  @override
  String get raisePriority => 'प्राथमिकता बढ़ाएँ';

  @override
  String get removeMember => 'सदस्य हटाएँ';

  @override
  String get addCandidate => 'संभावित मॉडल जोड़ें';

  @override
  String get connectRemote => 'दूरस्थ गेटवे से कनेक्ट करें';

  @override
  String get remoteQuic => 'दूरस्थ QUIC पता (/p2p/PeerID सहित)';

  @override
  String get localTunnel => 'स्थानीय टनल का लिसन पता';

  @override
  String get tunnelHelp =>
      'दूरस्थ गेटवे पर इस पीयर आईडी को अनुमति दें। स्थानीय टनल के क्लाइंट दूरस्थ गेटवे की कुंजी इस्तेमाल करते हैं।';

  @override
  String get loopback => 'HTTP लूपबैक पता';

  @override
  String get peerListen => 'P2P लिसन पता (बंद करने के लिए खाली छोड़ें)';

  @override
  String get allowedPeers => 'अनुमति प्राप्त पीयर आईडी, हर पंक्ति में एक';

  @override
  String get rotateMembers => 'पहले सदस्य को बारी-बारी से चुनें';
}
