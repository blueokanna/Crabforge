// ignore: unused_import
import 'package:intl/intl.dart' as intl;

import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for Korean (`ko`).
class AppLocalizationsKo extends AppLocalizations {
  AppLocalizationsKo([String locale = 'ko']) : super(locale);

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
