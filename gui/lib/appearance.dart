import 'dart:typed_data';

import 'package:flutter/material.dart';

import 'material_symbols.dart';

enum WorkspaceFont {
  googleSans('google-sans', 'Google Sans'),
  harmonyOs('harmonyos-sans-sc', 'HarmonyOS Sans SC');

  const WorkspaceFont(this.preference, this.label);

  final String preference;
  final String label;

  static WorkspaceFont fromPreference(String preference) =>
      preference == harmonyOs.preference ? harmonyOs : googleSans;

  String get family => 'packages/crabforge_assets/$label';

  List<String> get fallback => switch (this) {
    googleSans => [harmonyOs.family, _korean],
    harmonyOs => [googleSans.family, _korean],
  };

  static const _korean = 'packages/crabforge_assets/Noto Sans KR';
}

const systemAccentChannel = 'com.blue.crabforge/system-accent';

Color? decodeSystemAccent(ByteData? bytes) {
  if (bytes == null || bytes.lengthInBytes != 4) return null;
  final argb = bytes.getUint32(0, Endian.little);
  return argb >> 24 == 0xff ? Color(argb) : null;
}

ThemeData workspaceTheme(
  Brightness brightness,
  WorkspaceFont font, {
  Color? accent,
  bool highContrast = false,
}) {
  final colors = ColorScheme.fromSeed(
    seedColor: accent ?? const Color(0xffb45032),
    brightness: brightness,
    dynamicSchemeVariant: DynamicSchemeVariant.expressive,
    contrastLevel: highContrast ? 1 : 0,
  );
  final border = OutlineInputBorder(borderRadius: BorderRadius.circular(12));
  return ThemeData(
    useMaterial3: true,
    colorScheme: colors,
    typography: Typography.material2021(colorScheme: colors),
    fontFamily: font.family,
    fontFamilyFallback: font.fallback,
    splashFactory: InkRipple.splashFactory,
    actionIconTheme: ActionIconThemeData(
      backButtonIconBuilder: (_) => const Icon(CrabforgeSymbols.arrowBack),
      closeButtonIconBuilder: (_) => const Icon(CrabforgeSymbols.close),
      drawerButtonIconBuilder: (_) => const Icon(CrabforgeSymbols.menu),
      endDrawerButtonIconBuilder: (_) => const Icon(CrabforgeSymbols.menu),
    ),
    cardTheme: CardThemeData(
      margin: EdgeInsets.zero,
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(16)),
    ),
    inputDecorationTheme: InputDecorationThemeData(
      filled: true,
      fillColor: colors.surfaceContainerLowest,
      border: border,
    ),
    tooltipTheme: const TooltipThemeData(
      waitDuration: Duration(milliseconds: 400),
      preferBelow: false,
    ),
  );
}
