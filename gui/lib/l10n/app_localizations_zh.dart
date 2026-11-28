// ignore: unused_import
import 'package:intl/intl.dart' as intl;

import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for Chinese (`zh`).
class AppLocalizationsZh extends AppLocalizations {
  AppLocalizationsZh([String locale = 'zh']) : super(locale);

  @override
  String get subscriptionUsage => '订阅用量';

  @override
  String get noSubscriptionUsage => '供应商未返回此密钥的用量窗口。';

  @override
  String quotaUsed(String amount) {
    return '已用 $amount';
  }

  @override
  String quotaBalance(String amount) {
    return '剩余余额 $amount';
  }

  @override
  String quotaReset(String time) {
    return '下次重置于 $time';
  }

  @override
  String get quotaSeparate => '独立额度或超额计费';

  @override
  String quotaWindow(String kind) {
    String _temp0 = intl.Intl.selectLogic(kind, {
      'allowance': '额度',
      'month': '月度',
      'session': '会话额度',
      'extra': '额外用量',
      'monthly': '月度点数',
      'purchased': '已购点数',
      'free': '免费点数',
      'credits': '点',
      'videos': '个视频',
      'other': '额度',
    });
    return '$_temp0';
  }

  @override
  String quotaDuration(String unit, String amount) {
    String _temp0 = intl.Intl.selectLogic(unit, {
      'days': '$amount 天',
      'hours': '$amount 小时',
      'other': '$amount 分钟',
    });
    return '$_temp0';
  }

  @override
  String get memorySync => 'WebDAV 记忆同步';

  @override
  String get memorySyncUrl => 'WebDAV 目录或备份地址';

  @override
  String get memorySyncUser => 'WebDAV 用户名（可选）';

  @override
  String get memorySyncPassword => 'WebDAV 密码';

  @override
  String get memorySyncPassphrase => '备份加密口令';

  @override
  String get memorySyncPreview => '预览';

  @override
  String get memorySyncApply => '应用同步';

  @override
  String get memorySyncComplete => '同步完成';

  @override
  String memorySyncChanges(
    String downloaded,
    String updated,
    String removed,
    String uploaded,
  ) {
    return '下载 $downloaded 项 · 更新 $updated 项 · 移除 $removed 项 · 上传 $uploaded 项';
  }

  @override
  String get memoryTidy => '夜间整理记忆';

  @override
  String get memoryTidyHelp =>
      '启用后，网关会在本地时间凌晨 3 点后开始整理。已保存的记忆会发送到所选模型路由，以识别重复和过时的条目。手写及置顶记忆受到保护。';

  @override
  String get memoryTidyRun => '立即整理';

  @override
  String get memoryTidyUndo => '撤销上次整理';

  @override
  String memoryTidyChanged(String count) {
    return '已更改的记忆（$count）';
  }

  @override
  String memoryTidyRestored(String count) {
    return '已撤销的更改（$count）';
  }

  @override
  String get interfaceFont => '界面字体';

  @override
  String get importSlashCommands => '导入斜杠命令';

  @override
  String get commandImportSource => 'Markdown 或 TOML 文件、本地文件夹或 Git URL';

  @override
  String get commandChooseFile => '选择命令文件';

  @override
  String get projectHookAssignment => '编辑项目钩子分配';

  @override
  String get sharedSubagents => '共享子代理';

  @override
  String get importSubagents => '导入子代理';

  @override
  String get subagentImportSource => 'Markdown 文件、本地文件夹或 Git 地址';

  @override
  String get subagentChooseFile => '选择 Markdown 文件';

  @override
  String get subagentChooseFolder => '选择来源文件夹';

  @override
  String get subagentScanSource => '扫描来源';

  @override
  String get subagentImportTruncated => '只扫描了部分来源。选择范围更小的文件夹可查看其余文件。';

  @override
  String subagentImportStatus(String status) {
    String _temp0 = intl.Intl.selectLogic(status, {
      'update': '有可用更新',
      'unchanged': '已导入',
      'edited': '已保留本地编辑',
      'other': '',
    });
    return '$_temp0';
  }

  @override
  String get findNativeResources => '查找已有文件';

  @override
  String get noNativeResources => '未找到已有文件';

  @override
  String get nativeResourceScanLimit => '显示扫描到的前 512 个文件。';

  @override
  String get adoptResourceDetails =>
      'Crabforge 会在原位置管理此文件。取消分配给该客户端时，可能会移除未被修改的文件。';

  @override
  String get agentSubagents => '子代理';

  @override
  String get agentCommands => '斜杠命令';

  @override
  String get addSubagent => '添加子代理';

  @override
  String get addSlashCommand => '添加斜杠命令';

  @override
  String get resourceUserScope => '用户配置';

  @override
  String get noAgentResources => '暂无条目';

  @override
  String get resourcePlacements => '发布详情';

  @override
  String get agentResourceName => '名称';

  @override
  String get agentResourceDescription => '说明';

  @override
  String get agentResourcePrompt => '提示词';

  @override
  String get resourceSkillNames => '技能名称，每行一个';

  @override
  String get resourceMcpNames => 'MCP 服务器名称，每行一个';

  @override
  String get commandArgumentHelp =>
      '用 \$ARGUMENTS 表示全部参数。在兼容客户端中，可以用 \$1、\$2 等表示单个参数。';

  @override
  String get agentHooks => '钩子';

  @override
  String get addHook => '添加钩子';

  @override
  String get editHook => '编辑钩子';

  @override
  String get adoptHook => '接管钩子';

  @override
  String get noHooks => '尚未保存钩子';

  @override
  String get installedHooks => '客户端配置中的钩子';

  @override
  String get removeInstalledHook => '移除已安装的钩子';

  @override
  String get hookBackups => '钩子备份';

  @override
  String get restoreHookBackup => '恢复钩子备份';

  @override
  String get hookBackupRestored => '已恢复';

  @override
  String get hookBackupChanged => '客户端配置已更改';

  @override
  String get hookBackupUnavailable => '原配置或文件不可用';

  @override
  String get hookName => '钩子名称';

  @override
  String get hookDescription => '说明（可选）';

  @override
  String get hookEvent => '触发事件';

  @override
  String get hookMatcher => '匹配条件';

  @override
  String get hookAllTools => '所有工具';

  @override
  String get hookShellTool => 'Shell 命令';

  @override
  String get hookEditTool => '文件编辑';

  @override
  String get hookCommand => '客户端执行的命令';

  @override
  String get hookTimeout => '超时秒数（可选）';

  @override
  String get hookTurnEnd => '回合结束';

  @override
  String get hookAfterTool => '工具运行后';

  @override
  String get hookBeforeTool => '工具运行前';

  @override
  String get hookPrompt => '提交提示词时';

  @override
  String get hookSessionStart => '会话开始';

  @override
  String get hookExisting => '已有相同钩子';

  @override
  String get hookConflict => '已保留现有钩子';

  @override
  String get agentEnvironment => '环境变量';

  @override
  String get noEnvironmentVariables => '尚未保存环境变量';

  @override
  String get addVariable => '添加变量';

  @override
  String get editVariable => '编辑变量';

  @override
  String get variableName => '变量名称';

  @override
  String get variableValue => '值';

  @override
  String get showVariableValue => '显示值';

  @override
  String get hideVariableValue => '隐藏值';

  @override
  String get nativeVariableExists => '已设置';

  @override
  String get nativeVariableConflict => '已保留现有值';

  @override
  String get memoryLibrary => '记忆';

  @override
  String get agentRules => '规则';

  @override
  String get agentSkills => '技能';

  @override
  String get importSkill => '导入技能';

  @override
  String get applySkills => '应用技能';

  @override
  String get noSkills => '尚未导入技能';

  @override
  String get editSkill => '编辑技能分配';

  @override
  String get skillSourceFolder => '本地文件夹或 Git URL';

  @override
  String get checkSkillUpdate => '检查更新';

  @override
  String get browseRepositorySkills => '浏览来源技能';

  @override
  String get noRepositorySkills => '此来源没有其他可导入的技能。';

  @override
  String get dismissSkillCandidate => '忽略此技能';

  @override
  String get skillUpToDate => '技能文件已是最新版本。';

  @override
  String get skillUpdateAvailable => '有新的技能文件。选择重新导入文件即可应用。';

  @override
  String get skillRemovedUpstream => '所选目录已不包含 SKILL.md。';

  @override
  String get installedSkills => '已安装的技能';

  @override
  String get adoptSkill => '交由 Crabforge 管理';

  @override
  String get adoptSkillHint => 'Crabforge 会保存文件副本并管理此文件夹。取消分配时，内容未变的文件夹可能会被移除。';

  @override
  String get mcpLibrary => 'MCP 库';

  @override
  String get assignMcpServer => '分配到此项目';

  @override
  String get unassignMcpServer => '取消此项目的分配';

  @override
  String get syncMcpLibrary => '同步分配';

  @override
  String get refreshSkill => '重新导入文件';

  @override
  String get projectMcp => '项目 MCP';

  @override
  String get addMcpServer => '添加 MCP 服务器';

  @override
  String get editMcpServer => '编辑 MCP 服务器';

  @override
  String get noMcpServers => '尚未配置 MCP 服务器';

  @override
  String get mcpServerName => '服务器名称';

  @override
  String get addRule => '添加规则';

  @override
  String get editRule => '编辑规则';

  @override
  String get syncRules => '应用规则';

  @override
  String get noRules => '暂无已保存的规则';

  @override
  String get ruleName => '规则名称';

  @override
  String get ruleBody => 'Markdown 指令';

  @override
  String get rulePaths => '路径模式（可选，每行一个）';

  @override
  String get ruleRequired => '此项为必填项。';

  @override
  String ruleStatus(String value) {
    String _temp0 = intl.Intl.selectLogic(value, {
      'applied': '已应用',
      'pending': '待应用',
      'removed': '已移除',
      'conflict': '已保留现有文件',
      'other': '无法应用',
    });
    return '$_temp0';
  }

  @override
  String get agentSessions => '本地会话';

  @override
  String get allAgents => '所有 Agent';

  @override
  String get noAgentSessions => '未找到本地会话';

  @override
  String unreadableSessions(int count) {
    return '无法读取的会话文件（$count）';
  }

  @override
  String get copySessionId => '复制会话 ID';

  @override
  String get sessionWorkingFolder => '工作目录';

  @override
  String get sessionProfileFolder => '配置目录';

  @override
  String get distillTitle => '提炼记忆';

  @override
  String get distillPreview => '发送到所选模型路由前，请检查下方的用户消息。新记忆将保存为待审核状态。';

  @override
  String get distillSend => '发送给模型';

  @override
  String get distillEmpty => '没有发现新的候选记忆。';

  @override
  String get memorySourceQuote => '会话原文引用';

  @override
  String get agentInstructions => '用户指令';

  @override
  String get agentMemoryDigest => '用户记忆';

  @override
  String get agentMemoryDigestHelp => '发布后，全局记忆和此 Agent 的记忆将写入该 Agent 的用户指令。';

  @override
  String get autoPublishAgentMemory => '每日发布记忆';

  @override
  String get autoPublishAgentMemoryHelp => '网关运行时，每日将生效的记忆更新到此文件的摘要中。';

  @override
  String get agentMemoryDigestEmpty => '没有可发布的有效全局记忆或 Agent 记忆。';

  @override
  String get projectMemory => '项目记忆';

  @override
  String get publishMemory => '发布摘要';

  @override
  String get projectMemoryHelp => '发布后，能访问此指令文件的人都能读取摘要。';

  @override
  String get projectMemoryEmpty => '此项目没有可发布的有效记忆。';

  @override
  String get searchMemories => '搜索记忆';

  @override
  String get unreadableMemories => '无法读取的记忆文件';

  @override
  String get noMemoriesFound => '没有符合条件的记忆';

  @override
  String get addMemory => '添加记忆';

  @override
  String get editMemory => '编辑记忆';

  @override
  String get memoryScope => '适用范围';

  @override
  String get memoryTags => '标签，每行一个';

  @override
  String get memoryWhy => '原因（可选）';

  @override
  String get memoryKindLabel => '类型';

  @override
  String get memoryStateLabel => '状态';

  @override
  String get pinMemory => '置顶记忆';

  @override
  String memoryKind(String value) {
    String _temp0 = intl.Intl.selectLogic(value, {
      'profile': '个人信息',
      'preference': '偏好',
      'rule': '规则',
      'reference': '参考',
      'fact': '事实',
      'procedure': '流程',
      'episode': '事件',
      'other': '类型',
    });
    return '$_temp0';
  }

  @override
  String memoryState(String value) {
    String _temp0 = intl.Intl.selectLogic(value, {
      'active': '有效',
      'candidate': '待确认',
      'archived': '已归档',
      'superseded': '已替代',
      'other': '全部状态',
    });
    return '$_temp0';
  }

  @override
  String get projectInstructions => '项目指令';

  @override
  String get projectFolder => '项目文件夹';

  @override
  String get openProject => '打开项目';

  @override
  String get restoreInstructions => '恢复上一版本';

  @override
  String get relayFallback => '直连失败时使用的中继电路地址（可选）';

  @override
  String get listView => '列表视图';

  @override
  String get gridView => '网格视图';

  @override
  String get gatewayProtocol => '网关协议';

  @override
  String get nativeModelProtocol => '使用各模型的原生协议';

  @override
  String get roleModels => '角色模型';

  @override
  String get roleModelId => '模型 ID';

  @override
  String get resetRole => '重置角色';

  @override
  String get syncModelCatalog => '同步模型目录';

  @override
  String get thinkingLevels => '思考级别';

  @override
  String get sessionThinking => '会话思考级别';

  @override
  String get thinkingAuto => '自动';

  @override
  String get thinkingMinimal => '最低';

  @override
  String get thinkingLow => '低';

  @override
  String get thinkingMedium => '中';

  @override
  String get thinkingHigh => '高';

  @override
  String get thinkingExtraHigh => '很高';

  @override
  String get thinkingMax => '最高';

  @override
  String get ompProfile => 'Oh My Pi 配置档案';

  @override
  String get advisorSettings => '顾问设置';

  @override
  String get advisorReview => '审阅时机';

  @override
  String get advisorWait => '开始等待的待审阅数量';

  @override
  String get advisorWake => '恢复运行所需的最低问题级别';

  @override
  String get inheritSetting => '使用默认设置';

  @override
  String get everyTurn => '每一轮';

  @override
  String get runEnd => '运行结束时';

  @override
  String get allReviews => '等待所有审阅完成';

  @override
  String get minorNote => '小问题';

  @override
  String get concernNote => '需关注的问题';

  @override
  String get blockerNote => '阻塞问题';

  @override
  String get memberPatterns => '模型匹配规则，每行一条';

  @override
  String get automaticDistillation => '自动提炼记忆';

  @override
  String get automaticDistillationHelp =>
      '网关运行时，Crabforge 会将近期无活动会话的用户消息发送给所选模型路由。生成的候选记忆可在记忆库中审核。';

  @override
  String get distillationEnabled => '已启用';

  @override
  String get distillationLookback => '回溯天数';

  @override
  String get allSessions => '所有已列出的会话';

  @override
  String get distillationRun => '立即读取';

  @override
  String get pluginBundles => '共享插件库';

  @override
  String get piProjectPackages => 'Pi 项目包';

  @override
  String get claudeProjectPackages => 'Claude Code 项目插件';

  @override
  String get codexProjectPackages => 'Codex 项目插件';

  @override
  String get antigravityProjectPackages => 'Antigravity 项目插件';

  @override
  String get workspacePluginConfirm => '打开这个受信任的项目时，客户端可以运行已审核插件中的代码。';

  @override
  String get claudePackageConfirm =>
      'Claude Code 将为此项目安装已审核的插件。打开项目时，Claude Code 可以运行插件钩子。';

  @override
  String get piPackageConfirm => '打开受信任的项目时，Pi 可以运行包中的扩展。';

  @override
  String get piExportAssignments => '导出分配';

  @override
  String get piApplyAssignments => '应用分配';

  @override
  String get piAssignmentsTooLarge => '分配文件超过 1 MiB。';

  @override
  String piPackageStatus(String status) {
    String _temp0 = intl.Intl.selectLogic(status, {
      'available': '可分配',
      'applied': '已分配',
      'pending': '可同步',
      'detached': '包已从库中移除',
      'conflict': '已保留现有条目',
      'other': '',
    });
    return '$_temp0';
  }

  @override
  String get importPluginBundle => '导入插件包';

  @override
  String get previewPluginBundle => '预览插件包';

  @override
  String get pluginBundleSource => '插件目录、清单文件或 Git URL';

  @override
  String get noPluginBundles => '尚未保存插件包。';

  @override
  String get otherPluginEntries => '其他清单条目';

  @override
  String get noBundleResources => '此文件中没有资源。';

  @override
  String get copyBundleResource => '复制到资源库';

  @override
  String get bundleResourceCopied => '已复制到资源库';

  @override
  String get openPluginArchive => '打开 TAR 归档';

  @override
  String get exportPluginArchive => '导出插件归档';

  @override
  String get pluginArchiveSaved => '插件归档已保存';

  @override
  String get pluginCatalog => '浏览插件';

  @override
  String get noAvailablePlugins => '这些市场中没有可安装的插件。';

  @override
  String get exportPluginSettings => '导出插件设置';

  @override
  String get importPluginSettings => '导入插件设置';

  @override
  String get claudePlugins => 'Claude Code 插件';

  @override
  String get pluginMarketplaces => '插件市场';

  @override
  String get addPluginMarketplace => '添加插件市场';

  @override
  String get pluginSource => '插件市场路径或网址';

  @override
  String get installPlugin => '安装插件';

  @override
  String get updatePlugin => '更新插件';

  @override
  String get enablePlugin => '启用插件';

  @override
  String get disablePlugin => '禁用插件';

  @override
  String get pluginEnabled => '已启用';

  @override
  String get noClaudePlugins => '当前作用域没有已安装的 Claude Code 插件。';

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
  String get firstContentSeconds => '首段内容超时（秒）';

  @override
  String get groupFirstContentHelp => '0 继承设置，-1 等待当前成员。';

  @override
  String get defaultFirstContentHelp => '路由组的默认设置。0 或 -1 表示等待每个成员。';

  @override
  String get invalidFirstContent => '请输入 -1 到 2147483647 之间的整数。';

  @override
  String get groupPatienceSeconds => '繁忙重试时限（秒）';

  @override
  String get groupPatienceHelp => '0 表示继承，-1 表示关闭重试。';

  @override
  String get groupRoutingRules => '路由规则（JSON）';

  @override
  String get invalidGroupRoutingRules => '请输入规则 JSON 数组，或包含 rules 数组的 JSON 对象。';

  @override
  String get classifyTimeoutSeconds => '分类器超时（秒）';

  @override
  String get classifyTimeoutHelp => '请输入 1 至 60 秒，0 表示默认的 15 秒。';

  @override
  String get defaultGroupPatienceHelp => '0 表示 60 秒，-1 表示关闭重试。';

  @override
  String get invalidGroupPatience => '请输入 -1 到 240 之间的整数。';

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
  String get catalogRefreshInterval => '模型目录刷新间隔';

  @override
  String get catalogAutoInclude => '自动纳入匹配这些规则的模型';

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
  String get renameProvider => '重命名供应商';

  @override
  String get keyEnv => 'API 密钥环境变量';

  @override
  String get keyHint => '本地免密服务或使用 AWS IAM 的 Bedrock 可留空';

  @override
  String get resourceRoot => '资源根地址';

  @override
  String get modelIds => '上游模型 ID，每行一个';

  @override
  String get providerFallback => '备用模型或路由组，按顺序每行一个';

  @override
  String get allowProviderFallback => '允许为其他供应商接替请求';

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
