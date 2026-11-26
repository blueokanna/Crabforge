import 'package:flutter/material.dart';
import 'package:flutter_svg/flutter_svg.dart';

import 'brand_assets.dart';

class BrandIcon extends StatelessWidget {
  const BrandIcon._(this.asset, this.fallback, this.size);

  factory BrandIcon.provider(String id, {double size = 24}) =>
      BrandIcon._(providerBrandAssets[id], Icons.hub_outlined, size);

  factory BrandIcon.agent(String id) =>
      BrandIcon._(agentBrandAssets[id], Icons.terminal, 24);

  factory BrandIcon.model(String id) {
    final name = id.split('/').last.toLowerCase();
    for (final (pattern, asset) in _labs) {
      if (pattern.hasMatch(name)) {
        return BrandIcon._('$asset.svg', Icons.memory, 24);
      }
    }
    return BrandIcon._(null, Icons.memory_outlined, 24);
  }

  static final _labs = [
    (RegExp(r'claude'), 'claude-color'),
    (RegExp(r'gpt|^o\d|codex'), 'openai'),
    (RegExp(r'gemma'), 'gemma-color'),
    (RegExp(r'gemini'), 'gemini-color'),
    (RegExp(r'deepseek'), 'deepseek-color'),
    (RegExp(r'qwen|qwq'), 'qwen-color'),
    (RegExp(r'kimi|moonshot'), 'kimi'),
    (RegExp(r'glm'), 'zhipu-color'),
    (RegExp(r'grok'), 'grok'),
    (RegExp(r'minimax|abab'), 'minimax-color'),
    (RegExp(r'doubao|seed'), 'doubao-color'),
    (RegExp(r'stral'), 'mistral-color'),
    (RegExp(r'llama'), 'meta-color'),
    (RegExp(r'mimo'), 'xiaomimimo'),
    (RegExp(r'^step'), 'stepfun-color'),
    (RegExp(r'hunyuan'), 'hunyuan-color'),
  ];
  final String? asset;
  final IconData fallback;
  final double size;

  @override
  Widget build(BuildContext context) {
    final name = asset;
    if (name == null) return Icon(fallback, size: size);
    const package = 'crabforge_assets';
    if (name.endsWith('.png')) {
      return Image.asset(
        'brands/$name',
        package: package,
        width: size,
        height: size,
        excludeFromSemantics: true,
      );
    }
    return SvgPicture.asset(
      'brands/$name',
      package: package,
      width: size,
      height: size,
      excludeFromSemantics: true,
      theme: SvgTheme(currentColor: Theme.of(context).colorScheme.onSurface),
    );
  }
}
