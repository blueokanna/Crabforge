// ignore: unused_import
import 'package:intl/intl.dart' as intl;

import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for Japanese (`ja`).
class AppLocalizationsJa extends AppLocalizations {
  AppLocalizationsJa([String locale = 'ja']) : super(locale);

  @override
  String get importSlashCommands => 'スラッシュコマンドをインポート';

  @override
  String get commandImportSource => 'Markdown または TOML ファイル、ローカルフォルダー、Git URL';

  @override
  String get commandChooseFile => 'コマンドファイルを選択';

  @override
  String get projectHookAssignment => 'プロジェクトへのフックの割り当てを編集';

  @override
  String get sharedSubagents => '共有サブエージェント';

  @override
  String get importSubagents => 'サブエージェントをインポート';

  @override
  String get subagentImportSource => 'Markdown ファイル、ローカルフォルダ、または Git URL';

  @override
  String get subagentChooseFile => 'Markdown ファイルを選択';

  @override
  String get subagentChooseFolder => 'ソースフォルダを選択';

  @override
  String get subagentScanSource => 'ソースをスキャン';

  @override
  String get subagentImportTruncated =>
      'ソースの一部だけをスキャンしました。範囲の狭いフォルダを選ぶと、残りのファイルを確認できます。';

  @override
  String subagentImportStatus(String status) {
    String _temp0 = intl.Intl.selectLogic(status, {
      'update': '更新あり',
      'unchanged': 'インポート済み',
      'edited': 'ローカルの編集を保持',
      'other': '',
    });
    return '$_temp0';
  }

  @override
  String get findNativeResources => '既存のファイルを探す';

  @override
  String get noNativeResources => '既存のファイルが見つかりませんでした';

  @override
  String get nativeResourceScanLimit => '見つかった最初の 512 件を表示しています。';

  @override
  String get adoptResourceDetails =>
      'Crabforge はこのファイルを現在の場所で管理します。クライアントへの割り当てを解除すると、変更されていないファイルが削除されることがあります。';

  @override
  String get agentSubagents => 'サブエージェント';

  @override
  String get agentCommands => 'スラッシュコマンド';

  @override
  String get addSubagent => 'サブエージェントを追加';

  @override
  String get addSlashCommand => 'スラッシュコマンドを追加';

  @override
  String get resourceUserScope => 'ユーザープロファイル';

  @override
  String get noAgentResources => '保存された項目がありません';

  @override
  String get resourcePlacements => '公開の詳細';

  @override
  String get agentResourceName => '名前';

  @override
  String get agentResourceDescription => '説明';

  @override
  String get agentResourcePrompt => 'プロンプト';

  @override
  String get resourceSkillNames => 'スキル名（1 行に 1 件）';

  @override
  String get resourceMcpNames => 'MCP サーバー名（1 行に 1 件）';

  @override
  String get commandArgumentHelp =>
      'すべての引数には \$ARGUMENTS、対応クライアントで個別の引数には \$1、\$2 などを使用します。';

  @override
  String get agentHooks => 'フック';

  @override
  String get addHook => 'フックを追加';

  @override
  String get editHook => 'フックを編集';

  @override
  String get adoptHook => 'フックを引き継ぐ';

  @override
  String get noHooks => '保存済みのフックはありません';

  @override
  String get installedHooks => 'クライアント設定のフック';

  @override
  String get removeInstalledHook => 'インストール済みフックを削除';

  @override
  String get hookBackups => 'フックのバックアップ';

  @override
  String get restoreHookBackup => 'フックのバックアップを復元';

  @override
  String get hookBackupRestored => '復元済み';

  @override
  String get hookBackupChanged => 'クライアント設定が変更されています';

  @override
  String get hookBackupUnavailable => '元のプロファイルまたはファイルを利用できません';

  @override
  String get hookName => 'フック名';

  @override
  String get hookDescription => '説明（任意）';

  @override
  String get hookEvent => 'イベント';

  @override
  String get hookMatcher => '一致条件';

  @override
  String get hookAllTools => 'すべてのツール';

  @override
  String get hookShellTool => 'シェルコマンド';

  @override
  String get hookEditTool => 'ファイル編集';

  @override
  String get hookCommand => 'クライアントが実行するコマンド';

  @override
  String get hookTimeout => 'タイムアウト秒数（任意）';

  @override
  String get hookTurnEnd => 'ターン終了';

  @override
  String get hookAfterTool => 'ツール使用後';

  @override
  String get hookBeforeTool => 'ツール使用前';

  @override
  String get hookPrompt => 'プロンプト送信時';

  @override
  String get hookSessionStart => 'セッション開始';

  @override
  String get hookExisting => '同じフックが既にあります';

  @override
  String get hookConflict => '既存のフックを保持しました';

  @override
  String get agentEnvironment => '環境変数';

  @override
  String get noEnvironmentVariables => '保存済みの変数がありません';

  @override
  String get addVariable => '変数を追加';

  @override
  String get editVariable => '変数を編集';

  @override
  String get variableName => '変数名';

  @override
  String get variableValue => '値';

  @override
  String get showVariableValue => '値を表示';

  @override
  String get hideVariableValue => '値を非表示';

  @override
  String get nativeVariableExists => '設定済み';

  @override
  String get nativeVariableConflict => '既存の値を保持しました';

  @override
  String get memoryLibrary => 'メモリ';

  @override
  String get agentRules => 'ルール';

  @override
  String get agentSkills => 'スキル';

  @override
  String get importSkill => 'スキルをインポート';

  @override
  String get applySkills => 'スキルを適用';

  @override
  String get noSkills => 'インポート済みのスキルはありません';

  @override
  String get editSkill => 'スキルの割り当てを編集';

  @override
  String get skillSourceFolder => 'ローカルフォルダーまたは Git URL';

  @override
  String get checkSkillUpdate => '更新を確認';

  @override
  String get browseRepositorySkills => 'ソース内のスキルを表示';

  @override
  String get noRepositorySkills => 'このソースにはほかにインポートできるスキルがありません。';

  @override
  String get dismissSkillCandidate => 'スキルを非表示';

  @override
  String get skillUpToDate => 'スキルファイルは最新です。';

  @override
  String get skillUpdateAvailable => '新しいスキルファイルがあります。ファイルを再インポートして適用できます。';

  @override
  String get skillRemovedUpstream => '選択したフォルダーに SKILL.md がありません。';

  @override
  String get installedSkills => 'インストール済みのスキル';

  @override
  String get adoptSkill => 'Crabforge で管理する';

  @override
  String get adoptSkillHint =>
      'Crabforge はファイルのコピーを保存し、このフォルダーを管理します。割り当てを解除すると、変更のないフォルダーが削除されることがあります。';

  @override
  String get mcpLibrary => 'MCP ライブラリ';

  @override
  String get assignMcpServer => 'このプロジェクトに割り当てる';

  @override
  String get unassignMcpServer => 'このプロジェクトへの割り当てを解除';

  @override
  String get syncMcpLibrary => '割り当てを同期';

  @override
  String get refreshSkill => 'ファイルを再インポート';

  @override
  String get projectMcp => 'プロジェクト MCP';

  @override
  String get addMcpServer => 'MCP サーバーを追加';

  @override
  String get editMcpServer => 'MCP サーバーを編集';

  @override
  String get noMcpServers => 'MCP サーバーが設定されていません';

  @override
  String get mcpServerName => 'サーバー名';

  @override
  String get addRule => 'ルールを追加';

  @override
  String get editRule => 'ルールを編集';

  @override
  String get syncRules => 'ルールを適用';

  @override
  String get noRules => '保存済みのルールはありません';

  @override
  String get ruleName => 'ルール名';

  @override
  String get ruleBody => 'Markdown 形式の指示';

  @override
  String get rulePaths => 'パスのパターン（任意、1 行に 1 つ）';

  @override
  String get ruleRequired => 'この項目は必須です。';

  @override
  String ruleStatus(String value) {
    String _temp0 = intl.Intl.selectLogic(value, {
      'applied': '適用済み',
      'pending': '保留中',
      'removed': '削除済み',
      'conflict': '既存のファイルを保持',
      'other': '適用できませんでした',
    });
    return '$_temp0';
  }

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
  String get sessionWorkingFolder => '作業フォルダー';

  @override
  String get sessionProfileFolder => 'プロファイルフォルダー';

  @override
  String get distillTitle => '記憶を抽出';

  @override
  String get distillPreview =>
      '選択したモデルルートに送信する前に、下のユーザーメッセージを確認してください。新しい記憶は確認待ちとして保存されます。';

  @override
  String get distillSend => 'モデルに送信';

  @override
  String get distillEmpty => '新しい記憶の候補はありません。';

  @override
  String get memorySourceQuote => 'セッションからの引用';

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
  String get pluginBundles => '共有プラグインライブラリ';

  @override
  String get piProjectPackages => 'Pi のプロジェクトパッケージ';

  @override
  String get piPackageConfirm => '信頼済みのプロジェクトを開くと、Pi がパッケージの拡張機能を実行できます。';

  @override
  String get piExportAssignments => '割り当てをエクスポート';

  @override
  String get piApplyAssignments => '割り当てを適用';

  @override
  String get piAssignmentsTooLarge => '割り当てファイルが 1 MiB を超えています。';

  @override
  String piPackageStatus(String status) {
    String _temp0 = intl.Intl.selectLogic(status, {
      'available': '割り当て可能',
      'applied': '割り当て済み',
      'pending': '同期可能',
      'detached': 'ライブラリから削除済み',
      'conflict': '既存の項目を保持',
      'other': '',
    });
    return '$_temp0';
  }

  @override
  String get importPluginBundle => 'プラグインバンドルをインポート';

  @override
  String get previewPluginBundle => 'バンドルのプレビュー';

  @override
  String get pluginBundleSource => 'フォルダー、マニフェスト、または Git URL';

  @override
  String get noPluginBundles => '保存されたプラグインバンドルはありません。';

  @override
  String get otherPluginEntries => 'その他のマニフェスト項目';

  @override
  String get noBundleResources => 'このファイルにリソースはありません。';

  @override
  String get copyBundleResource => 'ライブラリにコピー';

  @override
  String get bundleResourceCopied => 'ライブラリにコピー済み';

  @override
  String get openPluginArchive => 'TAR アーカイブを開く';

  @override
  String get exportPluginArchive => 'プラグインアーカイブをエクスポート';

  @override
  String get pluginArchiveSaved => 'プラグインアーカイブを保存しました';

  @override
  String get pluginCatalog => 'プラグインを探す';

  @override
  String get noAvailablePlugins => 'これらのマーケットプレイスに利用可能なプラグインはありません。';

  @override
  String get exportPluginSettings => 'プラグイン設定をエクスポート';

  @override
  String get importPluginSettings => 'プラグイン設定をインポート';

  @override
  String get claudePlugins => 'Claude Code プラグイン';

  @override
  String get pluginMarketplaces => 'マーケットプレイス';

  @override
  String get addPluginMarketplace => 'マーケットプレイスを追加';

  @override
  String get pluginSource => 'マーケットプレイスのパスまたは URL';

  @override
  String get installPlugin => 'プラグインをインストール';

  @override
  String get updatePlugin => 'プラグインを更新';

  @override
  String get enablePlugin => 'プラグインを有効化';

  @override
  String get disablePlugin => 'プラグインを無効化';

  @override
  String get pluginEnabled => '有効';

  @override
  String get noClaudePlugins => 'このスコープには Claude Code プラグインがインストールされていません。';

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
