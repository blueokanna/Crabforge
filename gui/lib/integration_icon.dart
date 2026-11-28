import 'package:flutter/material.dart';
import 'package:flutter_svg/flutter_svg.dart';

import 'material_symbols.dart';

/// Artwork for known integrations, with Material symbols for custom entries.
class IntegrationIcon extends StatelessWidget {
  const IntegrationIcon.client(this.id, {super.key, this.size = 22})
    : _client = true,
      web = false,
      remote = false;

  const IntegrationIcon.provider(
    this.id, {
    super.key,
    this.size = 22,
    this.web = false,
    this.remote = false,
  }) : _client = false;

  final String id;
  final double size;
  final bool _client;
  final bool web, remote;

  static const _clients = {
    'agy': 'antigravity-color.svg',
    'aionui': 'aionui.svg',
    'air': 'air.svg',
    'air-ide': 'air.svg',
    'alma': 'alma.png',
    'antigravity': 'antigravity-color.svg',
    'aside': 'aside.svg',
    'astrbot': 'astrbot.svg',
    'atomcode': 'atomcode.png',
    'avibe': 'avibe.png',
    'cherrystudio': 'cherrystudio-color.svg',
    'cindy': 'cindy.png',
    'claude': 'claudecode-color.svg',
    'claude-desktop': 'claude-color.svg',
    'claude-science': 'claude-color.svg',
    'cline': 'cline.svg',
    'codebuddy': 'codebuddy-color.svg',
    'codex': 'codex-color.svg',
    'commandcode': 'commandcode.svg',
    'copilot': 'githubcopilot.svg',
    'copilot-jetbrains': 'githubcopilot.svg',
    'craft-agents': 'craft-agents-color.svg',
    'crush': 'crush.png',
    'cursor': 'cursor.svg',
    'cursor-local': 'cursor.svg',
    'deveco': 'deveco-color.svg',
    'devin': 'devin.svg',
    'dim': 'dim-color.svg',
    'droid': 'factory.svg',
    'ds-bot': 'deepseek-color.svg',
    'dsh': 'deepseek-color.svg',
    'empryo': 'empryo-color.svg',
    'fx': 'fx.svg',
    'goose': 'goose.svg',
    'grok': 'grok.svg',
    'grok-app': 'grok.svg',
    'hanako': 'hanako.png',
    'hermes': 'hermes.svg',
    'jcode': 'jcode.svg',
    'kigi': 'kigi.svg',
    'kimi': 'kimi.svg',
    'kun': 'kun.png',
    'liveagent': 'liveagent-color.svg',
    'lody': 'lody.png',
    'mimocode': 'mimocode.svg',
    'minimax-code': 'minimax-color.svg',
    'miniq': 'miniq-color.svg',
    'morph': 'mistermorph-color.svg',
    'mothx': 'mothx.png',
    'mu': 'mu.svg',
    'nanobot': 'nanobot-color.svg',
    'nowledge-mem': 'nowledge-mem.png',
    'obsidian': 'obsidian.svg',
    'ocr': 'open-code-review.svg',
    'omo': 'omo.svg',
    'omp': 'omp.svg',
    'openbot': 'openbot.png',
    'openchamber': 'openchamber.svg',
    'openclaw': 'openclaw.svg',
    'opencode': 'opencode.svg',
    'openmausbot': 'openmausbot-color.svg',
    'paimon': 'paimon.svg',
    'penguin': 'penguin-color.svg',
    'pi': 'pi.svg',
    'prime-agent': 'prime-agent.svg',
    'qoder': 'qoder.svg',
    'qoder-cn': 'qoder.svg',
    'qwen': 'qwen-color.svg',
    'qwenpaw': 'qwenpaw.svg',
    'raycast': 'raycast.svg',
    'reasonix': 'reasonix-color.svg',
    'shelley': 'shelley-color.svg',
    'snow': 'snow.png',
    'stepcode': 'stepfun-color.svg',
    't3code': 't3code.svg',
    'tuanjie-cowork': 'tuanjie-cowork-color.svg',
    'vibe': 'mistral-color.svg',
    'vscode': 'vscode.svg',
    'vscode-insiders': 'vscode.svg',
    'workbuddy': 'workbuddy-color.svg',
    'workbuddy-ai': 'workbuddy-color.svg',
    'zcode': 'zcode.png',
    'zed': 'zed.svg',
    'zoo-code': 'zoo-code.png',
  };

  static const _providerFamilies = {
    '302ai': 'ai302-color.svg',
    'aigocode': 'aigocode.png',
    'aihubmix': 'aihubmix-color.svg',
    'alibaba': 'bailian-color.svg',
    'anthropic': 'claude-color.svg',
    'azure': 'azure-color.svg',
    'baidu': 'baiducloud-color.svg',
    'bailian': 'bailian-color.svg',
    'bedrock': 'bedrock-color.svg',
    'chatglm': 'zhipu-color.svg',
    'cherryin': 'cherryin-color.svg',
    'clinepass': 'cline.svg',
    'commandcode': 'commandcode.svg',
    'deepseek': 'deepseek-color.svg',
    'fireworks': 'fireworks-color.svg',
    'gmicloud': 'gmicloud.svg',
    'google': 'gemini-color.svg',
    'gpustack': 'gpustack.png',
    'groq': 'groq.svg',
    'huaweicloud': 'huaweicloud-color.svg',
    'kilo': 'kilo.svg',
    'kimi': 'kimi.svg',
    'lmstudio': 'lmstudio.svg',
    'longcat': 'longcat.svg',
    'minimax': 'minimax-color.svg',
    'mistral': 'mistral-color.svg',
    'mlx': 'mlx-serve.png',
    'modelscope': 'modelscope-color.svg',
    'moonshot': 'kimi.svg',
    'nous': 'nousresearch.svg',
    'nvidia': 'nvidia-color.svg',
    'ollama': 'ollama.svg',
    'omlx': 'omlx.svg',
    'openai': 'openai.svg',
    'opencode': 'opencode.svg',
    'openrouter': 'openrouter.svg',
    'pipellm': 'pipellm-color.svg',
    'qwen': 'qwen-color.svg',
    'qwencloud': 'qwen-color.svg',
    'siliconflow': 'siliconcloud-color.svg',
    'stepfun': 'stepfun-color.svg',
    'tencent': 'tencentcloud-color.svg',
    'together': 'together-color.svg',
    'vercel': 'vercel.svg',
    'volcengine': 'volcengine-color.svg',
    'voyage': 'voyage-color.svg',
    'xai': 'xai.svg',
    'xiaomi': 'xiaomimimo.svg',
    'yylx': 'yylx.svg',
    'zai': 'zai.svg',
    'zhipu': 'zhipu-color.svg',
  };

  static const _coloredSvg = {
    'astrbot.svg',
    'open-code-review.svg',
    'qwenpaw.svg',
  };

  String? get _artwork {
    if (_client) return _clients[id];
    if (remote) return null;
    if (id == 'google-vertex') return 'vertexai-color.svg';
    return _providerFamilies[id.split('-').first];
  }

  Widget _fallback() {
    if (_client) {
      return Icon(switch (id) {
        'ante' || 'kirocrew' => CrabforgeSymbols.groupWork,
        'muse' || 'pencil' || 'openpencil' => CrabforgeSymbols.editSquare,
        'newmax' => CrabforgeSymbols.bolt,
        'paseo' => CrabforgeSymbols.altRoute,
        'vscodium' => CrabforgeSymbols.code,
        _ => CrabforgeSymbols.terminal,
      }, size: size);
    }
    return Icon(
      remote || id == 'remote-gateway'
          ? CrabforgeSymbols.settingsEthernet
          : web
          ? CrabforgeSymbols.language
          : CrabforgeSymbols.dns,
      size: size,
    );
  }

  @override
  Widget build(BuildContext context) {
    final file = _artwork;
    if (file == null) return _fallback();
    final path = 'brands/$file';
    if (file.endsWith('.png')) {
      return Image.asset(
        path,
        package: 'crabforge_assets',
        width: size,
        height: size,
        excludeFromSemantics: true,
        errorBuilder: (_, _, _) => _fallback(),
      );
    }
    final colored = file.endsWith('-color.svg') || _coloredSvg.contains(file);
    return SvgPicture.asset(
      path,
      package: 'crabforge_assets',
      width: size,
      height: size,
      colorFilter: colored
          ? null
          : ColorFilter.mode(
              IconTheme.of(context).color ??
                  Theme.of(context).colorScheme.onSurface,
              BlendMode.srcIn,
            ),
      excludeFromSemantics: true,
      errorBuilder: (_, _, _) => _fallback(),
    );
  }
}
