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
    harmonyOs => [_korean],
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
  final theme = ThemeData(
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
    cardTheme: const CardThemeData(margin: EdgeInsets.zero),
    inputDecorationTheme: InputDecorationThemeData(
      filled: true,
      fillColor: colors.surfaceContainerLowest,
      border: const OutlineInputBorder(),
    ),
    tooltipTheme: const TooltipThemeData(
      waitDuration: Duration(milliseconds: 400),
      preferBelow: false,
    ),
  );
  // Material 3 Expressive small-button tokens use medium corners at rest and
  // small corners when pressed. The emphasized title roles keep the type scale.
  final buttonShape = WidgetStateProperty.resolveWith<OutlinedBorder>(
    (states) => RoundedRectangleBorder(
      borderRadius: BorderRadius.circular(
        states.contains(WidgetState.pressed) ? 8 : 12,
      ),
    ),
  );
  return theme.copyWith(
    filledButtonTheme: FilledButtonThemeData(
      style: ButtonStyle(shape: buttonShape),
    ),
    outlinedButtonTheme: OutlinedButtonThemeData(
      style: ButtonStyle(shape: buttonShape),
    ),
    appBarTheme: theme.appBarTheme.copyWith(
      titleTextStyle: theme.textTheme.titleLarge?.copyWith(
        fontWeight: FontWeight.w500,
      ),
    ),
    dialogTheme: theme.dialogTheme.copyWith(
      titleTextStyle: theme.textTheme.headlineSmall?.copyWith(
        fontWeight: FontWeight.w500,
      ),
    ),
  );
}
