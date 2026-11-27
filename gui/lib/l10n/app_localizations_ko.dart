// ignore: unused_import
import 'package:intl/intl.dart' as intl;

import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for Korean (`ko`).
class AppLocalizationsKo extends AppLocalizations {
  AppLocalizationsKo([String locale = 'ko']) : super(locale);

  @override
  String get interfaceFont => '????? ??';

  @override
  String get importSlashCommands => '슬래시 명령 가져오기';

  @override
  String get commandImportSource => 'Markdown 또는 TOML 파일, 로컬 폴더 또는 Git URL';

  @override
  String get commandChooseFile => '명령 파일 선택';

  @override
  String get projectHookAssignment => '프로젝트 훅 할당 편집';

  @override
  String get sharedSubagents => '공유 하위 에이전트';

  @override
  String get importSubagents => '하위 에이전트 가져오기';

  @override
  String get subagentImportSource => 'Markdown 파일, 로컬 폴더 또는 Git URL';

  @override
  String get subagentChooseFile => 'Markdown 파일 선택';

  @override
  String get subagentChooseFolder => '원본 폴더 선택';

  @override
  String get subagentScanSource => '원본 검색';

  @override
  String get subagentImportTruncated =>
      '원본의 일부만 검색했습니다. 더 많은 파일을 보려면 범위가 좁은 폴더를 선택하세요.';

  @override
  String subagentImportStatus(String status) {
    String _temp0 = intl.Intl.selectLogic(status, {
      'update': '업데이트 가능',
      'unchanged': '이미 가져옴',
      'edited': '로컬 편집 내용 유지',
      'other': '',
    });
    return '$_temp0';
  }

  @override
  String get findNativeResources => '기존 파일 찾기';

  @override
  String get noNativeResources => '기존 파일을 찾을 수 없습니다';

  @override
  String get nativeResourceScanLimit => '찾은 파일 중 처음 512개를 표시합니다.';

  @override
  String get adoptResourceDetails =>
      'Crabforge가 현재 위치에서 이 파일을 관리합니다. 클라이언트 할당을 해제하면 변경되지 않은 파일이 삭제될 수 있습니다.';

  @override
  String get agentSubagents => '하위 에이전트';

  @override
  String get agentCommands => '슬래시 명령';

  @override
  String get addSubagent => '하위 에이전트 추가';

  @override
  String get addSlashCommand => '슬래시 명령 추가';

  @override
  String get resourceUserScope => '사용자 프로필';

  @override
  String get noAgentResources => '저장된 항목이 없습니다';

  @override
  String get resourcePlacements => '게시 세부정보';

  @override
  String get agentResourceName => '이름';

  @override
  String get agentResourceDescription => '설명';

  @override
  String get agentResourcePrompt => '프롬프트';

  @override
  String get resourceSkillNames => '스킬 이름, 한 줄에 하나';

  @override
  String get resourceMcpNames => 'MCP 서버 이름, 한 줄에 하나';

  @override
  String get commandArgumentHelp =>
      '전체 인수에는 \$ARGUMENTS를 사용하고, 호환 클라이언트에서 개별 인수에는 \$1, \$2 등을 사용하세요.';

  @override
  String get agentHooks => '훅';

  @override
  String get addHook => '훅 추가';

  @override
  String get editHook => '훅 편집';

  @override
  String get adoptHook => '훅 가져오기';

  @override
  String get noHooks => '저장된 훅 없음';

  @override
  String get installedHooks => '클라이언트 설정의 훅';

  @override
  String get removeInstalledHook => '설치된 훅 제거';

  @override
  String get hookBackups => '훅 백업';

  @override
  String get restoreHookBackup => '훅 백업 복원';

  @override
  String get hookBackupRestored => '복원됨';

  @override
  String get hookBackupChanged => '클라이언트 설정이 변경됨';

  @override
  String get hookBackupUnavailable => '원래 프로필 또는 파일을 사용할 수 없음';

  @override
  String get hookName => '훅 이름';

  @override
  String get hookDescription => '설명(선택 사항)';

  @override
  String get hookEvent => '이벤트';

  @override
  String get hookMatcher => '일치 조건';

  @override
  String get hookAllTools => '모든 도구';

  @override
  String get hookShellTool => '셸 명령';

  @override
  String get hookEditTool => '파일 편집';

  @override
  String get hookCommand => '클라이언트가 실행하는 명령';

  @override
  String get hookTimeout => '제한 시간(초, 선택 사항)';

  @override
  String get hookTurnEnd => '턴 종료';

  @override
  String get hookAfterTool => '도구 사용 후';

  @override
  String get hookBeforeTool => '도구 사용 전';

  @override
  String get hookPrompt => '프롬프트 제출';

  @override
  String get hookSessionStart => '세션 시작';

  @override
  String get hookExisting => '일치하는 훅이 이미 있음';

  @override
  String get hookConflict => '기존 훅을 유지함';

  @override
  String get agentEnvironment => '환경 변수';

  @override
  String get noEnvironmentVariables => '저장된 변수가 없습니다';

  @override
  String get addVariable => '변수 추가';

  @override
  String get editVariable => '변수 편집';

  @override
  String get variableName => '변수 이름';

  @override
  String get variableValue => '값';

  @override
  String get showVariableValue => '값 표시';

  @override
  String get hideVariableValue => '값 숨기기';

  @override
  String get nativeVariableExists => '이미 설정됨';

  @override
  String get nativeVariableConflict => '기존 값 유지';

  @override
  String get memoryLibrary => '메모리';

  @override
  String get agentRules => '규칙';

  @override
  String get agentSkills => '스킬';

  @override
  String get importSkill => '스킬 가져오기';

  @override
  String get applySkills => '스킬 적용';

  @override
  String get noSkills => '가져온 스킬이 없습니다';

  @override
  String get editSkill => '스킬 할당 편집';

  @override
  String get skillSourceFolder => '로컬 폴더 또는 Git URL';

  @override
  String get checkSkillUpdate => '업데이트 확인';

  @override
  String get browseRepositorySkills => '소스에서 스킬 찾아보기';

  @override
  String get noRepositorySkills => '이 소스에서 추가로 가져올 스킬이 없습니다.';

  @override
  String get dismissSkillCandidate => '스킬 숨기기';

  @override
  String get skillUpToDate => '스킬 파일이 최신 상태입니다.';

  @override
  String get skillUpdateAvailable => '새 스킬 파일이 있습니다. 파일을 다시 가져와 적용하세요.';

  @override
  String get skillRemovedUpstream => '선택한 폴더에 SKILL.md가 더 이상 없습니다.';

  @override
  String get installedSkills => '설치된 스킬';

  @override
  String get adoptSkill => 'Crabforge에서 관리';

  @override
  String get adoptSkillHint =>
      'Crabforge가 파일의 복사본을 저장하고 이 폴더를 관리합니다. 할당을 해제하면 변경되지 않은 폴더가 제거될 수 있습니다.';

  @override
  String get mcpLibrary => 'MCP 라이브러리';

  @override
  String get assignMcpServer => '이 프로젝트에 할당';

  @override
  String get unassignMcpServer => '이 프로젝트에서 할당 해제';

  @override
  String get syncMcpLibrary => '할당 동기화';

  @override
  String get refreshSkill => '파일 다시 가져오기';

  @override
  String get projectMcp => '프로젝트 MCP';

  @override
  String get addMcpServer => 'MCP 서버 추가';

  @override
  String get editMcpServer => 'MCP 서버 편집';

  @override
  String get noMcpServers => '구성된 MCP 서버가 없습니다';

  @override
  String get mcpServerName => '서버 이름';

  @override
  String get addRule => '규칙 추가';

  @override
  String get editRule => '규칙 편집';

  @override
  String get syncRules => '규칙 적용';

  @override
  String get noRules => '저장된 규칙이 없습니다';

  @override
  String get ruleName => '규칙 이름';

  @override
  String get ruleBody => 'Markdown 지침';

  @override
  String get rulePaths => '경로 패턴 (선택 사항, 줄마다 하나씩)';

  @override
  String get ruleRequired => '필수 입력 항목입니다.';

  @override
  String ruleStatus(String value) {
    String _temp0 = intl.Intl.selectLogic(value, {
      'applied': '적용됨',
      'pending': '대기 중',
      'removed': '삭제됨',
      'conflict': '기존 파일 유지됨',
      'other': '적용 실패',
    });
    return '$_temp0';
  }

  @override
  String get agentSessions => '로컬 세션';

  @override
  String get allAgents => '모든 에이전트';

  @override
  String get noAgentSessions => '로컬 세션을 찾을 수 없습니다';

  @override
  String unreadableSessions(int count) {
    return '읽을 수 없는 세션 파일($count개)';
  }

  @override
  String get copySessionId => '세션 ID 복사';

  @override
  String get sessionWorkingFolder => '작업 폴더';

  @override
  String get sessionProfileFolder => '프로필 폴더';

  @override
  String get distillTitle => '기억 추출';

  @override
  String get distillPreview =>
      '선택한 모델 경로로 보내기 전에 아래 사용자 메시지를 검토하세요. 새 기억은 검토 대기 상태로 저장됩니다.';

  @override
  String get distillSend => '모델에 보내기';

  @override
  String get distillEmpty => '새로운 기억 후보가 없습니다.';

  @override
  String get memorySourceQuote => '세션 인용문';

  @override
  String get agentInstructions => '사용자 지침';

  @override
  String get agentMemoryDigest => '사용자 메모리';

  @override
  String get agentMemoryDigestHelp =>
      '게시하면 전역 메모리와 이 에이전트의 메모리를 해당 에이전트의 사용자 지침에 기록합니다.';

  @override
  String get agentMemoryDigestEmpty => '게시할 수 있는 활성 전역 또는 에이전트 메모리가 없습니다.';

  @override
  String get projectMemory => '프로젝트 메모리';

  @override
  String get publishMemory => '요약 게시';

  @override
  String get projectMemoryHelp =>
      '요약은 프로젝트 지침 파일에 저장됩니다. 파일에 접근할 수 있는 사람은 누구나 읽을 수 있습니다.';

  @override
  String get projectMemoryEmpty => '이 프로젝트에는 게시할 활성 메모리가 없습니다.';

  @override
  String get searchMemories => '메모리 검색';

  @override
  String get unreadableMemories => '읽을 수 없는 메모리 파일';

  @override
  String get noMemoriesFound => '일치하는 메모리가 없습니다';

  @override
  String get addMemory => '메모리 추가';

  @override
  String get editMemory => '메모리 편집';

  @override
  String get memoryScope => '적용 범위';

  @override
  String get memoryTags => '태그, 한 줄에 하나씩';

  @override
  String get memoryWhy => '이유(선택 사항)';

  @override
  String get memoryKindLabel => '유형';

  @override
  String get memoryStateLabel => '상태';

  @override
  String get pinMemory => '메모리 고정';

  @override
  String memoryKind(String value) {
    String _temp0 = intl.Intl.selectLogic(value, {
      'profile': '프로필',
      'preference': '선호',
      'rule': '규칙',
      'reference': '참조',
      'fact': '사실',
      'procedure': '절차',
      'episode': '사건',
      'other': '유형',
    });
    return '$_temp0';
  }

  @override
  String memoryState(String value) {
    String _temp0 = intl.Intl.selectLogic(value, {
      'active': '활성',
      'candidate': '검토 대기',
      'archived': '보관됨',
      'superseded': '대체됨',
      'other': '모든 상태',
    });
    return '$_temp0';
  }

  @override
  String get projectInstructions => '프로젝트 지침';

  @override
  String get projectFolder => '프로젝트 폴더';

  @override
  String get openProject => '프로젝트 열기';

  @override
  String get restoreInstructions => '이전 버전 복원';

  @override
  String get relayFallback => '직접 연결 실패 시 사용할 릴레이 회로 주소(선택 사항)';

  @override
  String get listView => '목록 보기';

  @override
  String get gridView => '격자 보기';

  @override
  String get gatewayProtocol => '게이트웨이 프로토콜';

  @override
  String get nativeModelProtocol => '각 모델의 기본 프로토콜 사용';

  @override
  String get roleModels => '역할별 모델';

  @override
  String get roleModelId => '모델 ID';

  @override
  String get resetRole => '역할 초기화';

  @override
  String get syncModelCatalog => '모델 목록 동기화';

  @override
  String get thinkingLevels => '추론 수준';

  @override
  String get sessionThinking => '세션 추론 수준';

  @override
  String get thinkingAuto => '자동';

  @override
  String get thinkingMinimal => '최소';

  @override
  String get thinkingLow => '낮음';

  @override
  String get thinkingMedium => '중간';

  @override
  String get thinkingHigh => '높음';

  @override
  String get thinkingExtraHigh => '매우 높음';

  @override
  String get thinkingMax => '최대';

  @override
  String get ompProfile => 'Oh My Pi 프로필';

  @override
  String get advisorSettings => '자문 설정';

  @override
  String get advisorReview => '검토 시점';

  @override
  String get advisorWait => '대기를 시작할 미완료 검토 수';

  @override
  String get advisorWake => '실행을 재개할 최소 심각도';

  @override
  String get inheritSetting => '기본값 사용';

  @override
  String get everyTurn => '매 턴마다';

  @override
  String get runEnd => '실행 종료 시';

  @override
  String get allReviews => '모든 검토가 끝날 때까지 대기';

  @override
  String get minorNote => '사소한 문제';

  @override
  String get concernNote => '우려 사항';

  @override
  String get blockerNote => '진행을 막는 문제';

  @override
  String get memberPatterns => '모델 패턴 (한 줄에 하나씩)';

  @override
  String get pluginBundles => '공유 플러그인 라이브러리';

  @override
  String get piProjectPackages => 'Pi 프로젝트 패키지';

  @override
  String get claudeProjectPackages => 'Claude Code 프로젝트 플러그인';

  @override
  String get codexProjectPackages => 'Codex 프로젝트 플러그인';

  @override
  String get antigravityProjectPackages => 'Antigravity 프로젝트 플러그인';

  @override
  String get workspacePluginConfirm =>
      '이 신뢰할 수 있는 프로젝트를 열면 클라이언트가 검토한 플러그인의 코드를 실행할 수 있습니다.';

  @override
  String get claudePackageConfirm =>
      'Claude Code가 이 프로젝트에 검토한 플러그인을 설치합니다. 프로젝트를 열면 Claude Code가 플러그인 훅을 실행할 수 있습니다.';

  @override
  String get piPackageConfirm => '신뢰하는 프로젝트를 열면 Pi가 패키지의 확장을 실행할 수 있습니다.';

  @override
  String get piExportAssignments => '할당 내보내기';

  @override
  String get piApplyAssignments => '할당 적용';

  @override
  String get piAssignmentsTooLarge => '할당 파일이 1 MiB를 초과합니다.';

  @override
  String piPackageStatus(String status) {
    String _temp0 = intl.Intl.selectLogic(status, {
      'available': '할당 가능',
      'applied': '할당됨',
      'pending': '동기화 가능',
      'detached': '라이브러리에서 제거된 패키지',
      'conflict': '기존 항목 유지',
      'other': '',
    });
    return '$_temp0';
  }

  @override
  String get importPluginBundle => '플러그인 번들 가져오기';

  @override
  String get previewPluginBundle => '번들 미리 보기';

  @override
  String get pluginBundleSource => '플러그인 폴더, 매니페스트 또는 Git URL';

  @override
  String get noPluginBundles => '저장된 플러그인 번들이 없습니다.';

  @override
  String get otherPluginEntries => '기타 매니페스트 항목';

  @override
  String get noBundleResources => '이 파일에 리소스가 없습니다.';

  @override
  String get copyBundleResource => '라이브러리에 복사';

  @override
  String get bundleResourceCopied => '라이브러리에 복사됨';

  @override
  String get openPluginArchive => 'TAR 아카이브 열기';

  @override
  String get exportPluginArchive => '플러그인 아카이브 내보내기';

  @override
  String get pluginArchiveSaved => '플러그인 아카이브 저장됨';

  @override
  String get pluginCatalog => '플러그인 찾아보기';

  @override
  String get noAvailablePlugins => '이 마켓플레이스에 설치할 수 있는 플러그인이 없습니다.';

  @override
  String get exportPluginSettings => '플러그인 설정 내보내기';

  @override
  String get importPluginSettings => '플러그인 설정 가져오기';

  @override
  String get claudePlugins => 'Claude Code 플러그인';

  @override
  String get pluginMarketplaces => '마켓플레이스';

  @override
  String get addPluginMarketplace => '마켓플레이스 추가';

  @override
  String get pluginSource => '마켓플레이스 경로 또는 URL';

  @override
  String get installPlugin => '플러그인 설치';

  @override
  String get updatePlugin => '플러그인 업데이트';

  @override
  String get enablePlugin => '플러그인 사용';

  @override
  String get disablePlugin => '플러그인 사용 중지';

  @override
  String get pluginEnabled => '사용 중';

  @override
  String get noClaudePlugins => '이 범위에 설치된 Claude Code 플러그인이 없습니다.';

  @override
  String get shareGroup => '공유할 그룹 복사';

  @override
  String get groupCopied => '그룹을 클립보드에 복사했습니다';

  @override
  String get importGroup => '그룹 가져오기';

  @override
  String get sharedGroupText => '공유된 그룹 텍스트';

  @override
  String get replaceGroup => '같은 ID의 기존 그룹 교체';

  @override
  String get missingMembers => '사용할 수 없는 구성원';

  @override
  String get replacedMembers => '교체된 구성원';

  @override
  String routeSession(String id) {
    return '세션 $id';
  }

  @override
  String get enableGroup => '그룹 사용';

  @override
  String get disableGroup => '그룹 사용 중지';

  @override
  String get listGroupModels => '클라이언트 모델 목록에 그룹 구성원 모델 표시';

  @override
  String get copyGroup => '그룹 복제';

  @override
  String get groupNote => '그룹 메모(선택 사항)';

  @override
  String get firstContentSeconds => '첫 콘텐츠 대기 시간(초)';

  @override
  String get groupFirstContentHelp => '0은 상위 설정을 따르고, -1은 멤버의 응답을 기다립니다.';

  @override
  String get defaultFirstContentHelp =>
      '라우팅 그룹의 기본값입니다. 0 또는 -1이면 각 멤버의 응답을 기다립니다.';

  @override
  String get invalidFirstContent => '-1에서 2147483647 사이의 정수를 입력하세요.';

  @override
  String get groupPatienceSeconds => '재시도 제한 시간(초)';

  @override
  String get groupPatienceHelp => '0은 설정을 상속하고, -1은 재시도를 끕니다.';

  @override
  String get groupRoutingRules => '라우팅 규칙(JSON)';

  @override
  String get invalidGroupRoutingRules =>
      '규칙의 JSON 배열 또는 rules 배열이 있는 JSON 객체를 입력하세요.';

  @override
  String get classifyTimeoutSeconds => '분류기 제한 시간(초)';

  @override
  String get classifyTimeoutHelp => '1~60초를 입력하세요. 0은 기본값인 15초를 적용합니다.';

  @override
  String get defaultGroupPatienceHelp => '0은 60초를 적용하고, -1은 재시도를 끕니다.';

  @override
  String get invalidGroupPatience => '-1에서 240 사이의 정수를 입력하세요.';

  @override
  String get enableMember => '멤버 활성화';

  @override
  String get disableMember => '멤버 비활성화';

  @override
  String get manualRouting => '수동 선택';

  @override
  String get selectMember => '이 멤버 선택';

  @override
  String get providerCatalogs => 'models.dev 카탈로그 ID(선택 사항)';

  @override
  String get anthropicAuthHeader => 'Anthropic API 키 헤더';

  @override
  String get automaticAuthHeader => '자동 선택';

  @override
  String get autoGroups => '여러 공급자의 동일한 모델을 그룹으로 묶기';

  @override
  String get restoreGroup => '그룹 복원';

  @override
  String get vertexProject => 'Vertex AI 프로젝트 설정';

  @override
  String get cloudProjectId => 'Google Cloud 프로젝트 ID';

  @override
  String get vertexLocation => 'Vertex AI 리전';

  @override
  String get googleCredentials => 'ADC 파일(선택 사항)';

  @override
  String get impersonateAccount => '가장할 서비스 계정(선택 사항)';

  @override
  String get models => '모델';

  @override
  String get modelCopied => '모델 ID를 복사했습니다';

  @override
  String get noModelsFound => '일치하는 모델이 없습니다';

  @override
  String get routingStrategy => '라우팅 방식';

  @override
  String get listedOrder => '목록 순서';

  @override
  String get weightedOrder => '가중 라운드 로빈';

  @override
  String get memberWeight => '구성원 가중치';

  @override
  String get invalidWeight => '1에서 1000 사이의 정수를 입력하세요.';

  @override
  String get providers => '공급자';

  @override
  String get routes => '라우트';

  @override
  String get groups => '라우트 그룹';

  @override
  String get gateway => '게이트웨이';

  @override
  String get language => '언어';

  @override
  String get systemLanguage => '시스템 언어';

  @override
  String confirmRemove(String value) {
    return '$value을(를) 삭제할까요?';
  }

  @override
  String get cancel => '취소';

  @override
  String get remove => '삭제';

  @override
  String get save => '저장';

  @override
  String get webLogin => '웹 로그인';

  @override
  String get loginHelp => '새 Chrome 창에서 로그인한 뒤 여기로 돌아와 창을 닫으세요.';

  @override
  String get closeLogin => '로그인 창 닫기';

  @override
  String get duplicateGroup => '이미 사용 중인 그룹 ID입니다.';

  @override
  String get addProvider => '공급자 추가';

  @override
  String get newGroup => '새 그룹';

  @override
  String get stopToEdit => '설정을 편집하려면 게이트웨이를 중지하세요.';

  @override
  String get reload => '다시 불러오기';

  @override
  String get reloadConfig => '설정 다시 불러오기';

  @override
  String providerCount(int count) {
    return '공급자 $count개';
  }

  @override
  String presetCount(int count) {
    return 'API 프리셋 $count개 중에서 선택하거나 기본 URL을 입력하세요.';
  }

  @override
  String get providerHelp => '클라이언트 요청을 받으려면 업스트림 URL과 모델 ID를 추가하세요.';

  @override
  String get removeProvider => '공급자 삭제';

  @override
  String get webProviders => '웹 공급자';

  @override
  String get webHelp =>
      'Chrome과 버전이 맞는 ChromeDriver를 사용하세요. 추가한 공급자를 선택하면 전용 프로필로 로그인할 수 있습니다.';

  @override
  String webSite(String value) {
    return '$value 웹';
  }

  @override
  String get groupNeedsProvider => '라우트 그룹을 만들려면 먼저 공급자와 모델을 추가하세요.';

  @override
  String get groupHelp => '우선순위에 따라 추가하세요. 요청 제한이나 서버 오류가 발생하면 다음 구성원을 시도합니다.';

  @override
  String get removeGroup => '그룹 삭제';

  @override
  String recentAttempts(int count) {
    return '최근 업스트림 시도 $count회';
  }

  @override
  String get historyLimit => '현재 게이트웨이 프로세스 · 최근 128회';

  @override
  String get waiting => '클라이언트 요청을 기다리는 중입니다.';

  @override
  String get historyHelp => '라우트 기록을 보려면 게이트웨이를 시작하세요.';

  @override
  String get running => '게이트웨이 실행 중';

  @override
  String get stopped => '게이트웨이 중지됨';

  @override
  String get stop => '게이트웨이 중지';

  @override
  String get start => '게이트웨이 시작';

  @override
  String get keyCopied => '게이트웨이 키 복사됨';

  @override
  String get copyKey => '클라이언트 키 복사';

  @override
  String get networkSettings => '네트워크 설정';

  @override
  String get browserOrigins => '허용할 브라우저 출처 (한 줄에 하나씩)';

  @override
  String get localPeer => '로컬 피어 ID';

  @override
  String get copyPeer => '피어 ID 복사';

  @override
  String get peerCopied => '피어 ID 복사됨';

  @override
  String get listenAddress => '수신 주소';

  @override
  String get disabled => '비활성화';

  @override
  String get trustedPeers => '신뢰하는 피어';

  @override
  String get clients => '클라이언트 연결';

  @override
  String get agentRoot => '에이전트 API 루트 URL';

  @override
  String get copyRoot => '루트 URL 복사';

  @override
  String get rootCopied => '루트 URL 복사됨';

  @override
  String get agentHelp =>
      'Chat Completions 및 Responses 클라이언트는 뒤에 /v1을 붙이세요. 선택한 에이전트가 라우트 기록에 표시됩니다.';

  @override
  String get tunnel => '원격 게이트웨이 터널';

  @override
  String get disconnected => '연결 끊김';

  @override
  String get connect => '연결';

  @override
  String get disconnect => '연결 해제';

  @override
  String get apiBase => 'API 기본 URL';

  @override
  String get copyBase => '기본 URL 복사';

  @override
  String get baseCopied => 'API 기본 URL 복사됨';

  @override
  String get changed => '공급자 설정이 변경되었습니다. 모델을 다시 불러오세요.';

  @override
  String modelsRead(int count) {
    return '모델 $count개를 불러왔습니다. 저장하기 전에 편집할 수 있습니다.';
  }

  @override
  String get presetLoaded => '프리셋 모델을 불러왔습니다. 계정 권한에 맞게 수정하세요.';

  @override
  String get editProvider => '공급자 편집';

  @override
  String get searchPresets => '이름으로 프리셋 검색';

  @override
  String get providerId => '공급자 ID';

  @override
  String get renameProvider => '공급자 이름 변경';

  @override
  String get keyEnv => 'API 키 환경 변수';

  @override
  String get keyHint => '로컬 서비스 또는 AWS IAM을 쓰는 Bedrock은 생략 가능';

  @override
  String get resourceRoot => '리소스 루트 URL';

  @override
  String get modelIds => '업스트림 모델 ID, 한 줄에 하나씩';

  @override
  String get providerFallback => '대체 모델 또는 그룹을 우선순위에 따라 한 줄에 하나씩 입력';

  @override
  String get allowProviderFallback => '다른 공급자의 대체 요청 허용';

  @override
  String protocolBase(String value) {
    return '$value 기본 URL';
  }

  @override
  String get catalogUrl => '모델 카탈로그 URL(선택)';

  @override
  String get catalogHint => '비워 두면 기본 카탈로그를 사용합니다';

  @override
  String get readingModels => '모델 불러오는 중';

  @override
  String get readModels => '모델 카탈로그 불러오기';

  @override
  String get requiredField => '필수 항목입니다.';

  @override
  String get createGroup => '라우트 그룹 만들기';

  @override
  String get editGroup => '라우트 그룹 편집';

  @override
  String get groupId => '라우트 그룹 ID';

  @override
  String get raisePriority => '우선순위 높이기';

  @override
  String get removeMember => '구성원 삭제';

  @override
  String get addCandidate => '후보 모델 추가';

  @override
  String get connectRemote => '원격 게이트웨이 연결';

  @override
  String get remoteQuic => '원격 QUIC 주소(/p2p/PeerID 포함)';

  @override
  String get localTunnel => '로컬 터널 수신 주소';

  @override
  String get tunnelHelp =>
      '원격 게이트웨이에서 이 피어 ID를 허용하세요. 로컬 터널에 연결할 때 원격 게이트웨이 키를 사용합니다.';

  @override
  String get loopback => 'HTTP 루프백 주소';

  @override
  String get peerListen => 'P2P 수신 주소(비우면 비활성화)';

  @override
  String get allowedPeers => '허용할 피어 ID, 한 줄에 하나씩';

  @override
  String get rotateMembers => '첫 번째 구성원 순환 선택';
}
