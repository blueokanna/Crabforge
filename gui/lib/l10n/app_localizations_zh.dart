// ignore: unused_import
import 'package:intl/intl.dart' as intl;

import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for Chinese (`zh`).
class AppLocalizationsZh extends AppLocalizations {
  AppLocalizationsZh([String locale = 'zh']) : super(locale);

  @override
  String get shareGroup => '复制路由组以分享';

  @override
  String get groupCopied => '路由组已复制到剪贴板';

  @override
  String get importGroup => '导入路由组';

  @override
  String get sharedGroupText => '路由组分享文本';

  @override
  String get replaceGroup => '替换 ID 相同的已有路由组';

  @override
  String get missingMembers => '本机缺少的成员';

  @override
  String get replacedMembers => '已替换的成员';

  @override
  String routeSession(String id) {
    return '会话 $id';
  }

  @override
  String get enableGroup => '启用路由组';

  @override
  String get disableGroup => '停用路由组';

  @override
  String get listGroupModels => '在客户端模型目录中显示成员模型';

  @override
  String get copyGroup => '复制路由组';

  @override
  String get groupNote => '路由组备注（可选）';

  @override
  String get enableMember => '启用成员';

  @override
  String get disableMember => '停用成员';

  @override
  String get manualRouting => '手动选择成员';

  @override
  String get selectMember => '使用此成员';

  @override
  String get providerCatalogs => 'models.dev 目录 ID（可选）';

  @override
  String get anthropicAuthHeader => 'Anthropic API 密钥请求头';

  @override
  String get automaticAuthHeader => '自动选择';

  @override
  String get autoGroups => '将不同供应商的同一模型自动分组';

  @override
  String get restoreGroup => '恢复路由组';

  @override
  String get vertexProject => 'Vertex AI 项目设置';

  @override
  String get cloudProjectId => 'Google Cloud 项目 ID';

  @override
  String get vertexLocation => 'Vertex AI 区域';

  @override
  String get googleCredentials => 'ADC 凭证文件（可选）';

  @override
  String get impersonateAccount => '要模拟的服务账号（可选）';

  @override
  String get models => '模型';

  @override
  String get modelCopied => '已复制模型 ID';

  @override
  String get noModelsFound => '没有匹配的模型';

  @override
  String get routingStrategy => '路由策略';

  @override
  String get listedOrder => '按列表顺序';

  @override
  String get weightedOrder => '加权轮询';

  @override
  String get memberWeight => '成员权重';

  @override
  String get invalidWeight => '请输入 1 到 1000 之间的整数。';

  @override
  String get providers => '供应商';

  @override
  String get routes => '路由';

  @override
  String get groups => '路由组';

  @override
  String get gateway => '网关';

  @override
  String get language => '语言';

  @override
  String get systemLanguage => '跟随系统';

  @override
  String confirmRemove(String value) {
    return '移除 $value？';
  }

  @override
  String get cancel => '取消';

  @override
  String get remove => '移除';

  @override
  String get save => '保存';

  @override
  String get webLogin => '网页登录';

  @override
  String get loginHelp => '在新打开的 Chrome 窗口中完成登录，然后返回此处关闭窗口。';

  @override
  String get closeLogin => '关闭登录窗口';

  @override
  String get duplicateGroup => '路由组 ID 已存在';

  @override
  String get addProvider => '添加供应商';

  @override
  String get newGroup => '新建组';

  @override
  String get stopToEdit => '网关运行中，停止后可修改配置。';

  @override
  String get reload => '重新加载';

  @override
  String get reloadConfig => '重新读取配置';

  @override
  String providerCount(int count) {
    return '$count 个供应商';
  }

  @override
  String presetCount(int count) {
    return '可选择 $count 个 API 预设，或填写自定义基址';
  }

  @override
  String get providerHelp => '添加上游地址和模型 ID 后，客户端即可通过网关调用。';

  @override
  String get removeProvider => '移除供应商';

  @override
  String get webProviders => '网页供应商';

  @override
  String get webHelp => '网页中转使用 Chrome 和匹配的 ChromeDriver。添加后，点击供应商登录其独立浏览器配置。';

  @override
  String webSite(String value) {
    return '$value 网页';
  }

  @override
  String get groupNeedsProvider => '先添加供应商和模型，再建立路由组。';

  @override
  String get groupHelp => '按优先级添加成员。上游限流或服务故障时，网关尝试下一个成员。';

  @override
  String get removeGroup => '移除路由组';

  @override
  String recentAttempts(int count) {
    return '最近 $count 次上游尝试';
  }

  @override
  String get historyLimit => '当前网关进程 · 最多保留 128 条';

  @override
  String get waiting => '等待客户端发出模型请求。';

  @override
  String get historyHelp => '启动网关后可查看实际路由记录。';

  @override
  String get running => '网关运行中';

  @override
  String get stopped => '网关已停止';

  @override
  String get stop => '停止网关';

  @override
  String get start => '启动网关';

  @override
  String get keyCopied => '已复制网关密钥';

  @override
  String get copyKey => '复制客户端密钥';

  @override
  String get networkSettings => '网络设置';

  @override
  String get browserOrigins => '允许的浏览器来源（每行一个）';

  @override
  String get localPeer => '本机 Peer ID';

  @override
  String get copyPeer => '复制 Peer ID';

  @override
  String get peerCopied => '已复制 Peer ID';

  @override
  String get listenAddress => '监听地址';

  @override
  String get disabled => '已关闭';

  @override
  String get trustedPeers => '受信任节点';

  @override
  String get clients => '客户端连接';

  @override
  String get agentRoot => 'Agent 专属 API 根地址';

  @override
  String get copyRoot => '复制根地址';

  @override
  String get rootCopied => '已复制根地址';

  @override
  String get agentHelp =>
      'Chat Completions 和 Responses 客户端使用此地址加 /v1。路由记录会显示所选 Agent。';

  @override
  String get tunnel => '远端网关隧道';

  @override
  String get disconnected => '未连接';

  @override
  String get connect => '连接';

  @override
  String get disconnect => '断开';

  @override
  String get apiBase => 'API 基址';

  @override
  String get copyBase => '复制基址';

  @override
  String get baseCopied => '已复制 API 基址';

  @override
  String get changed => '供应商设置已改变，请重新获取模型。';

  @override
  String modelsRead(int count) {
    return '已读取 $count 个模型。可在保存前编辑列表。';
  }

  @override
  String get presetLoaded => '已带入预设模型。可按账户权限编辑。';

  @override
  String get editProvider => '编辑供应商';

  @override
  String get searchPresets => '按名称搜索预设';

  @override
  String get providerId => '供应商 ID';

  @override
  String get keyEnv => 'API 密钥环境变量';

  @override
  String get keyHint => '本地免密服务或使用 AWS IAM 的 Bedrock 可留空';

  @override
  String get resourceRoot => '资源根地址';

  @override
  String get modelIds => '上游模型 ID，每行一个';

  @override
  String protocolBase(String value) {
    return '$value 基址';
  }

  @override
  String get catalogUrl => '模型目录完整地址（可选）';

  @override
  String get catalogHint => '留空时使用供应商默认目录';

  @override
  String get readingModels => '正在读取模型';

  @override
  String get readModels => '读取模型目录';

  @override
  String get requiredField => '请填写此项';

  @override
  String get createGroup => '新建路由组';

  @override
  String get editGroup => '编辑路由组';

  @override
  String get groupId => '路由组 ID';

  @override
  String get raisePriority => '提高优先级';

  @override
  String get removeMember => '移除成员';

  @override
  String get addCandidate => '添加候选模型';

  @override
  String get connectRemote => '连接远端网关';

  @override
  String get remoteQuic => '远端 QUIC 地址（含 /p2p/PeerID）';

  @override
  String get localTunnel => '本地隧道监听地址';

  @override
  String get tunnelHelp => '在远端信任本机 Peer ID。客户端连接本地隧道时使用远端的网关密钥。';

  @override
  String get loopback => 'HTTP 回环地址';

  @override
  String get peerListen => 'P2P 监听地址，留空关闭';

  @override
  String get allowedPeers => '允许连接的 Peer ID，每行一个';

  @override
  String get rotateMembers => '轮流选择首选成员';
}
