// ignore: unused_import
import 'package:intl/intl.dart' as intl;

import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for Japanese (`ja`).
class AppLocalizationsJa extends AppLocalizations {
  AppLocalizationsJa([String locale = 'ja']) : super(locale);

  @override
  String get memoryLibrary => 'メモリ';

  @override
  String get agentSessions => 'ローカルセッション';

  @override
  String get allAgents => 'すべてのエージェント';

  @override
  String get noAgentSessions => 'ローカルセッションが見つかりません';

  @override
  String unreadableSessions(int count) {
    return '読み取れないセッションファイル（$count）';
  }

  @override
  String get copySessionId => 'セッション ID をコピー';

  @override
  String get agentInstructions => 'ユーザー指示';

  @override
  String get agentMemoryDigest => 'ユーザーメモリ';

  @override
  String get agentMemoryDigestHelp =>
      '公開すると、グローバルメモリとこのエージェントのメモリが、エージェントのユーザー指示に書き込まれます。';

  @override
  String get agentMemoryDigestEmpty => '公開できる有効なグローバルメモリまたはエージェントメモリはありません。';

  @override
  String get projectMemory => 'プロジェクトのメモリ';

  @override
  String get publishMemory => '概要を公開';

  @override
  String get projectMemoryHelp =>
      '概要がプロジェクトの指示ファイルに書き込まれます。ファイルにアクセスできる人は誰でも読めます。';

  @override
  String get projectMemoryEmpty => 'このプロジェクトには公開できる有効なメモリがありません。';

  @override
  String get searchMemories => 'メモリを検索';

  @override
  String get unreadableMemories => '読み取れないメモリファイル';

  @override
  String get noMemoriesFound => '該当するメモリがありません';

  @override
  String get addMemory => 'メモリを追加';

  @override
  String get editMemory => 'メモリを編集';

  @override
  String get memoryScope => '適用範囲';

  @override
  String get memoryTags => 'タグ（1 行に 1 つ）';

  @override
  String get memoryWhy => '理由（任意）';

  @override
  String get memoryKindLabel => '種類';

  @override
  String get memoryStateLabel => '状態';

  @override
  String get pinMemory => 'メモリを固定';

  @override
  String memoryKind(String value) {
    String _temp0 = intl.Intl.selectLogic(value, {
      'profile': 'プロフィール',
      'preference': '設定の好み',
      'rule': 'ルール',
      'reference': '参照',
      'fact': '事実',
      'procedure': '手順',
      'episode': '出来事',
      'other': '種類',
    });
    return '$_temp0';
  }

  @override
  String memoryState(String value) {
    String _temp0 = intl.Intl.selectLogic(value, {
      'active': '有効',
      'candidate': '確認待ち',
      'archived': 'アーカイブ済み',
      'superseded': '置き換え済み',
      'other': 'すべての状態',
    });
    return '$_temp0';
  }

  @override
  String get projectInstructions => 'プロジェクトの指示';

  @override
  String get projectFolder => 'プロジェクトフォルダー';

  @override
  String get openProject => 'プロジェクトを開く';

  @override
  String get restoreInstructions => '前のバージョンを復元';

  @override
  String get relayFallback => '直接接続失敗時のリレー回線アドレス（任意）';

  @override
  String get listView => 'リスト表示';

  @override
  String get gridView => 'グリッド表示';

  @override
  String get gatewayProtocol => 'ゲートウェイのプロトコル';

  @override
  String get nativeModelProtocol => '各モデルのネイティブプロトコルを使う';

  @override
  String get roleModels => '役割別のモデル';

  @override
  String get roleModelId => 'モデル ID';

  @override
  String get resetRole => '役割をリセット';

  @override
  String get syncModelCatalog => 'モデル一覧を同期';

  @override
  String get thinkingLevels => '思考レベル';

  @override
  String get sessionThinking => 'セッションの思考レベル';

  @override
  String get thinkingAuto => '自動';

  @override
  String get thinkingMinimal => '最小';

  @override
  String get thinkingLow => '低';

  @override
  String get thinkingMedium => '中';

  @override
  String get thinkingHigh => '高';

  @override
  String get thinkingExtraHigh => '非常に高い';

  @override
  String get thinkingMax => '最大';

  @override
  String get ompProfile => 'Oh My Pi プロファイル';

  @override
  String get advisorSettings => 'アドバイザー設定';

  @override
  String get advisorReview => 'レビューのタイミング';

  @override
  String get advisorWait => '待機を開始する未完了レビュー数';

  @override
  String get advisorWake => '実行再開に必要な指摘の重要度';

  @override
  String get inheritSetting => '既定値を使用';

  @override
  String get everyTurn => '毎ターン';

  @override
  String get runEnd => '実行終了時';

  @override
  String get allReviews => 'すべてのレビューを待つ';

  @override
  String get minorNote => '軽微な指摘';

  @override
  String get concernNote => '懸念事項';

  @override
  String get blockerNote => '進行を妨げる問題';

  @override
  String get memberPatterns => 'モデルのパターン（1 行に 1 つ）';

  @override
  String get shareGroup => '共有用にグループをコピー';

  @override
  String get groupCopied => 'グループをクリップボードにコピーしました';

  @override
  String get importGroup => 'グループをインポート';

  @override
  String get sharedGroupText => '共有されたグループのテキスト';

  @override
  String get replaceGroup => '同じ ID の既存グループを置き換える';

  @override
  String get missingMembers => '利用できないメンバー';

  @override
  String get replacedMembers => '置き換えたメンバー';

  @override
  String routeSession(String id) {
    return 'セッション $id';
  }

  @override
  String get enableGroup => 'グループを有効にする';

  @override
  String get disableGroup => 'グループを無効にする';

  @override
  String get listGroupModels => 'クライアントのモデル一覧にグループ内のモデルを表示';

  @override
  String get copyGroup => 'グループを複製';

  @override
  String get groupNote => 'グループのメモ（任意）';

  @override
  String get enableMember => 'メンバーを有効にする';

  @override
  String get disableMember => 'メンバーを無効にする';

  @override
  String get manualRouting => '手動選択';

  @override
  String get selectMember => 'このメンバーを選択';

  @override
  String get providerCatalogs => 'models.dev カタログ ID（任意）';

  @override
  String get anthropicAuthHeader => 'Anthropic API キーのヘッダー';

  @override
  String get automaticAuthHeader => '自動選択';

  @override
  String get autoGroups => '複数のプロバイダーの同じモデルをまとめる';

  @override
  String get restoreGroup => 'グループを復元';

  @override
  String get vertexProject => 'Vertex AI プロジェクトの設定';

  @override
  String get cloudProjectId => 'Google Cloud プロジェクト ID';

  @override
  String get vertexLocation => 'Vertex AI のロケーション';

  @override
  String get googleCredentials => 'ADC ファイル（任意）';

  @override
  String get impersonateAccount => '権限を借用するサービス アカウント（任意）';

  @override
  String get models => 'モデル';

  @override
  String get modelCopied => 'モデル ID をコピーしました';

  @override
  String get noModelsFound => '一致するモデルがありません';

  @override
  String get routingStrategy => 'ルーティング方式';

  @override
  String get listedOrder => 'リスト順';

  @override
  String get weightedOrder => '重み付きラウンドロビン';

  @override
  String get memberWeight => 'メンバーの重み';

  @override
  String get invalidWeight => '1 から 1000 までの整数を入力してください。';

  @override
  String get providers => 'プロバイダー';

  @override
  String get routes => 'ルート';

  @override
  String get groups => 'ルートグループ';

  @override
  String get gateway => 'ゲートウェイ';

  @override
  String get language => '言語';

  @override
  String get systemLanguage => 'システムの言語';

  @override
  String confirmRemove(String value) {
    return '$value を削除しますか？';
  }

  @override
  String get cancel => 'キャンセル';

  @override
  String get remove => '削除';

  @override
  String get save => '保存';

  @override
  String get webLogin => 'ウェブログイン';

  @override
  String get loginHelp => '新しい Chrome ウィンドウでログインし、ここに戻って閉じてください。';

  @override
  String get closeLogin => 'ログイン画面を閉じる';

  @override
  String get duplicateGroup => 'このグループ ID は既に使われています。';

  @override
  String get addProvider => 'プロバイダーを追加';

  @override
  String get newGroup => '新規グループ';

  @override
  String get stopToEdit => '設定を編集するにはゲートウェイを停止してください。';

  @override
  String get reload => '再読み込み';

  @override
  String get reloadConfig => '設定を再読み込み';

  @override
  String providerCount(int count) {
    return 'プロバイダー数：$count';
  }

  @override
  String presetCount(int count) {
    return '$count 件の API プリセットから選ぶか、ベース URL を入力してください。';
  }

  @override
  String get providerHelp => '接続先の URL とモデル ID を追加すると、クライアントから利用できます。';

  @override
  String get removeProvider => 'プロバイダーを削除';

  @override
  String get webProviders => 'ウェブプロバイダー';

  @override
  String get webHelp =>
      'Chrome と対応する ChromeDriver を使用します。追加したプロバイダーを選択し、専用プロファイルでログインしてください。';

  @override
  String webSite(String value) {
    return '$value ウェブ版';
  }

  @override
  String get groupNeedsProvider => 'ルートグループを作成する前にプロバイダーとモデルを追加してください。';

  @override
  String get groupHelp => '優先順に追加します。レート制限やサーバー障害が発生すると次のメンバーを試します。';

  @override
  String get removeGroup => 'グループを削除';

  @override
  String recentAttempts(int count) {
    return '最近の接続試行：$count 件';
  }

  @override
  String get historyLimit => '現在のプロセスの直近 128 件';

  @override
  String get waiting => 'クライアントのリクエストを待っています。';

  @override
  String get historyHelp => 'ゲートウェイを起動するとルート履歴が表示されます。';

  @override
  String get running => 'ゲートウェイ稼働中';

  @override
  String get stopped => 'ゲートウェイ停止中';

  @override
  String get stop => 'ゲートウェイを停止';

  @override
  String get start => 'ゲートウェイを起動';

  @override
  String get keyCopied => 'ゲートウェイキーをコピーしました';

  @override
  String get copyKey => 'クライアントキーをコピー';

  @override
  String get networkSettings => 'ネットワーク設定';

  @override
  String get browserOrigins => '許可するブラウザーのオリジン（1 行につき 1 件）';

  @override
  String get localPeer => 'ローカルのピア ID';

  @override
  String get copyPeer => 'ピア ID をコピー';

  @override
  String get peerCopied => 'ピア ID をコピーしました';

  @override
  String get listenAddress => '待受アドレス';

  @override
  String get disabled => '無効';

  @override
  String get trustedPeers => '信頼するピア';

  @override
  String get clients => 'クライアント接続';

  @override
  String get agentRoot => 'エージェントの API ルート URL';

  @override
  String get copyRoot => 'ルート URL をコピー';

  @override
  String get rootCopied => 'ルート URL をコピーしました';

  @override
  String get agentHelp =>
      'Chat Completions と Responses では末尾に /v1 を追加します。ルート履歴には選択したエージェントが表示されます。';

  @override
  String get tunnel => 'リモートゲートウェイのトンネル';

  @override
  String get disconnected => '未接続';

  @override
  String get connect => '接続';

  @override
  String get disconnect => '切断';

  @override
  String get apiBase => 'API ベース URL';

  @override
  String get copyBase => 'ベース URL をコピー';

  @override
  String get baseCopied => 'API ベース URL をコピーしました';

  @override
  String get changed => '設定が変更されました。モデルを再取得してください。';

  @override
  String modelsRead(int count) {
    return 'モデルを $count 件取得しました。保存前に編集できます。';
  }

  @override
  String get presetLoaded => 'プリセットのモデルを読み込みました。アカウントに合わせて編集できます。';

  @override
  String get editProvider => 'プロバイダーを編集';

  @override
  String get searchPresets => '名前でプリセットを検索';

  @override
  String get providerId => 'プロバイダー ID';

  @override
  String get renameProvider => 'プロバイダー名を変更';

  @override
  String get keyEnv => 'API キーの環境変数';

  @override
  String get keyHint => 'ローカルサービスや AWS IAM を使う Bedrock では省略可';

  @override
  String get resourceRoot => 'リソースのルート URL';

  @override
  String get modelIds => '接続先のモデル ID（1 行に 1 件）';

  @override
  String get providerFallback => '予備のモデルまたはグループ（優先順に 1 行に 1 件）';

  @override
  String get allowProviderFallback => '他のプロバイダーからの代替リクエストを受け付ける';

  @override
  String protocolBase(String value) {
    return '$value のベース URL';
  }

  @override
  String get catalogUrl => 'モデルカタログの URL（任意）';

  @override
  String get catalogHint => '空欄の場合は既定のカタログを使用';

  @override
  String get readingModels => 'モデルを読み込み中';

  @override
  String get readModels => 'モデルカタログを取得';

  @override
  String get requiredField => 'この項目は必須です。';

  @override
  String get createGroup => 'ルートグループを作成';

  @override
  String get editGroup => 'ルートグループを編集';

  @override
  String get groupId => 'ルートグループ ID';

  @override
  String get raisePriority => '優先度を上げる';

  @override
  String get removeMember => 'メンバーを削除';

  @override
  String get addCandidate => '候補モデルを追加';

  @override
  String get connectRemote => 'リモートゲートウェイに接続';

  @override
  String get remoteQuic => 'リモート QUIC アドレス（/p2p/PeerID を含む）';

  @override
  String get localTunnel => 'トンネルのローカル待受アドレス';

  @override
  String get tunnelHelp => 'リモート側でこのピア ID を許可してください。ローカルトンネルにはリモートのキーを使用します。';

  @override
  String get loopback => 'HTTP ループバックアドレス';

  @override
  String get peerListen => 'P2P 待受アドレス（空欄で無効）';

  @override
  String get allowedPeers => '許可するピア ID（1 行に 1 件）';

  @override
  String get rotateMembers => '最初のメンバーを順に切り替える';
}
