import 'dart:typed_data';

import 'package:crabforge/appearance.dart';
import 'package:crabforge/material_symbols.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  test('Font choices apply across Material text roles', () {
    expect(WorkspaceFont.fromPreference(''), WorkspaceFont.googleSans);
    expect(
      WorkspaceFont.fromPreference('harmonyos-sans-sc'),
      WorkspaceFont.harmonyOs,
    );
    for (final font in WorkspaceFont.values) {
      for (final brightness in Brightness.values) {
        final theme = workspaceTheme(brightness, font);
        expect(theme.textTheme.bodyMedium!.fontFamily, font.family);
        expect(theme.textTheme.headlineLarge!.fontFamily, font.family);
        expect(theme.textTheme.labelLarge!.fontFamilyFallback, font.fallback);
      }
    }
    expect(
      WorkspaceFont.googleSans.fallback.first,
      WorkspaceFont.harmonyOs.family,
    );
  });

  test('Opaque Windows accent generates Expressive light and dark schemes', () {
    final seed = ByteData(4)..setUint32(0, 0xff0067c0, Endian.little);
    final accent = decodeSystemAccent(seed);
    expect(accent, const Color(0xff0067c0));
    expect(decodeSystemAccent(ByteData(3)), isNull);
    expect(decodeSystemAccent(ByteData(4)), isNull);
    for (final brightness in Brightness.values) {
      for (final highContrast in [false, true]) {
        final theme = workspaceTheme(
          brightness,
          WorkspaceFont.googleSans,
          accent: accent,
          highContrast: highContrast,
        );
        expect(theme.useMaterial3, isTrue);
        expect(
          theme.colorScheme,
          ColorScheme.fromSeed(
            seedColor: accent!,
            brightness: brightness,
            dynamicSchemeVariant: DynamicSchemeVariant.expressive,
            contrastLevel: highContrast ? 1 : 0,
          ),
        );
      }
    }
    expect(CrabforgeSymbols.formatSize.fontFamily, 'Material Symbols Rounded');
    expect(CrabforgeSymbols.formatSize.fontPackage, 'crabforge_assets');
  });

  test('Expressive buttons and titles use the small component tokens', () {
    final theme = workspaceTheme(Brightness.light, WorkspaceFont.googleSans);
    for (final style in [
      theme.filledButtonTheme.style!,
      theme.outlinedButtonTheme.style!,
    ]) {
      final rest = style.shape!.resolve({}) as RoundedRectangleBorder;
      final pressed =
          style.shape!.resolve({WidgetState.pressed}) as RoundedRectangleBorder;
      expect(rest.borderRadius, BorderRadius.circular(12));
      expect(pressed.borderRadius, BorderRadius.circular(8));
    }
    expect(theme.appBarTheme.titleTextStyle!.fontWeight, FontWeight.w500);
    expect(theme.dialogTheme.titleTextStyle!.fontWeight, FontWeight.w500);
  });
}
