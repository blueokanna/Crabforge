import 'dart:convert';
import 'dart:io';

import 'package:crabforge/agent_environment.dart';
import 'package:crabforge/main.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

import 'desktop_widgets.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testDesktopWidgets('Manage environment variables in both layouts', (
    tester,
  ) async {
    final home = Directory(Platform.environment['CRABFORGE_HOME'] ?? '');
    if (!home.uri.pathSegments.any(
      (s) => s.startsWith('crabforge-gui-test-'),
    )) {
      throw StateError('Use an isolated test home');
    }
    for (final entry in {
      'CLAUDE_CONFIG_DIR': 'claude',
      'CODEX_HOME': 'codex',
      'PI_CODING_AGENT_DIR': 'pi',
    }.entries) {
      if (Directory(Platform.environment[entry.key] ?? '').uri !=
          Directory('${home.path}/${entry.value}').uri) {
        throw StateError('Use isolated client profiles');
      }
    }
    final claude = File('${home.path}/claude/settings.json');
    final codex = File('${home.path}/codex/config.toml');
    final nativePi = File('${home.path}/pi/extensions/crabforge-env.ts');
    final gatewayPi = File(
      '${home.path}/agents/pi/extensions/crabforge-env.ts',
    );
    addTearDown(tester.binding.platformDispatcher.clearLocalesTestValue);
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);
    tester.binding.platformDispatcher.localesTestValue = [const Locale('en')];
    tester.view.devicePixelRatio = 1;
    tester.view.physicalSize = const Size(560, 800);
    await tester.pumpWidget(const CrabforgeApp());

    Future<void> waitFor(Finder target) async {
      final deadline = DateTime.now().add(const Duration(seconds: 10));
      while (target.evaluate().isEmpty && DateTime.now().isBefore(deadline)) {
        await tester.pump(const Duration(milliseconds: 50));
      }
      expect(target, findsOneWidget);
      await tester.pumpAndSettle();
    }

    Future<void> tap(Finder target) async {
      await waitFor(target);
      await tester.ensureVisible(target);
      await tester.pumpAndSettle();
      await tester.tap(target);
      await tester.pumpAndSettle();
    }

    Finder below(Finder parent, Finder child) =>
        find.descendant(of: parent, matching: child);
    await tap(
      below(find.byType(NavigationBar), find.text('Client connections')),
    );
    await tap(find.text('Environment variables'));
    final page = find.byType(AgentEnvironment);
    await waitFor(find.text('No variables saved'));
    expect(below(page, find.byType(SliverList)), findsOneWidget);
    await tap(find.text('Add variable'));
    final save = find.widgetWithText(FilledButton, 'Save');
    final editor = find.ancestor(of: save, matching: find.byType(AlertDialog));
    final name = below(editor, find.widgetWithText(TextField, 'Variable name'));
    final value = below(editor, find.widgetWithText(TextField, 'Value'));
    await tester.enterText(name, 'CRABFORGE_GUI_FIXTURE');
    await tester.enterText(value, 'initial string');
    expect(tester.widget<TextField>(value).obscureText, isTrue);
    await tap(find.byTooltip('Show value'));
    expect(tester.widget<TextField>(value).obscureText, isFalse);
    for (final agent in ['Claude Code', 'Codex', 'Pi']) {
      await tap(below(editor, find.widgetWithText(FilterChip, agent)));
    }
    await tap(save);
    final entry = find.byKey(const ValueKey('CRABFORGE_GUI_FIXTURE'));
    await waitFor(entry);
    expect(
      jsonDecode(claude.readAsStringSync())['env']['CRABFORGE_GUI_FIXTURE'],
      'initial string',
    );
    expect(codex.readAsStringSync(), contains('CRABFORGE_GUI_FIXTURE'));
    expect(nativePi.readAsStringSync(), gatewayPi.readAsStringSync());

    await tap(below(page, find.byTooltip('Grid view')));
    expect(below(page, find.byType(SliverGrid)), findsOneWidget);
    tester.view.physicalSize = const Size(1280, 900);
    await tester.pumpAndSettle();
    claude.writeAsStringSync(
      claude.readAsStringSync().replaceFirst('initial string', 'external'),
    );
    await tap(find.text('Sync assignments'));
    await waitFor(find.text('Claude Code · Existing value kept'));
    await tap(below(entry, find.byTooltip('Edit variable')));
    await tap(below(editor, find.widgetWithText(FilterChip, 'Pi')));
    await tap(save);
    await waitFor(find.text('Claude Code · Existing value kept'));
    expect(nativePi.existsSync(), isFalse);
    expect(gatewayPi.existsSync(), isFalse);

    await tap(below(entry, find.byTooltip('Remove')));
    await tap(find.widgetWithText(TextButton, 'Remove'));
    await waitFor(find.text('Codex · Removed'));
    expect(codex.readAsStringSync(), isNot(contains('CRABFORGE_GUI_FIXTURE')));
    expect(
      jsonDecode(claude.readAsStringSync())['env']['CRABFORGE_GUI_FIXTURE'],
      'external',
    );
  });
}
