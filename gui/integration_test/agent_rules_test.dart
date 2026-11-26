import 'dart:io';

import 'package:crabforge/agent_rules.dart';
import 'package:crabforge/main.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

import 'desktop_widgets.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testDesktopWidgets('Edit and apply rules in both window layouts', (
    tester,
  ) async {
    final home = Directory(Platform.environment['CRABFORGE_HOME'] ?? '');
    final claude = Directory('${home.path}/claude');
    if (!home.uri.pathSegments.any(
          (s) => s.startsWith('crabforge-gui-test-'),
        ) ||
        Directory(Platform.environment['CLAUDE_CONFIG_DIR'] ?? '').uri !=
            claude.uri) {
      throw StateError('Use a fresh test home with an isolated Claude profile');
    }
    final ruleFile = File('${claude.path}/rules/diagnostic-style.md');
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

    await waitFor(find.byType(NavigationBar));
    await tester.tap(
      find.descendant(
        of: find.byType(NavigationBar),
        matching: find.text('Client connections'),
      ),
    );
    await waitFor(find.text('Rules'));
    await tester.ensureVisible(find.text('Rules'));
    await tester.tap(find.text('Rules'));
    final library = find.byType(AgentRules);
    await waitFor(find.text('No rules saved'));
    await tester.tap(
      find.descendant(of: library, matching: find.text('Add rule')),
    );
    await waitFor(find.byType(AlertDialog));
    await tester.enterText(
      find.widgetWithText(TextFormField, 'Rule name'),
      'diagnostic-style',
    );
    await tester.enterText(
      find.widgetWithText(TextFormField, 'Markdown instructions'),
      'Show units in tables.',
    );
    await tester.enterText(
      find.widgetWithText(
        TextFormField,
        'Path patterns (optional, one per line)',
      ),
      'src/**/*.rs',
    );
    final claudeChip = find.widgetWithText(FilterChip, 'Claude Code');
    await tester.ensureVisible(claudeChip);
    await tester.tap(claudeChip);
    await tester.tap(find.widgetWithText(FilledButton, 'Save'));
    await waitFor(find.text('Claude Code · Applied'));
    expect(
      ruleFile.readAsStringSync(),
      '---\npaths: ["src/**/*.rs"]\n---\n\nShow units in tables.\n',
    );
    await tester.tap(
      find.descendant(of: library, matching: find.byTooltip('Grid view')),
    );
    await tester.pumpAndSettle();
    expect(
      find.descendant(of: library, matching: find.byType(SliverGrid)),
      findsOneWidget,
    );

    tester.view.physicalSize = const Size(1280, 900);
    await tester.pumpAndSettle();
    await tester.tap(
      find.descendant(of: library, matching: find.byTooltip('Edit rule')),
    );
    await waitFor(find.byType(AlertDialog));
    await tester.enterText(
      find.widgetWithText(TextFormField, 'Markdown instructions'),
      'Show units and sources in tables.',
    );
    await tester.tap(find.widgetWithText(FilledButton, 'Save'));
    await waitFor(find.text('Show units and sources in tables.'));
    expect(
      ruleFile.readAsStringSync(),
      endsWith('Show units and sources in tables.\n'),
    );

    ruleFile.writeAsStringSync('A direct edit.\r\n');
    await tester.tap(find.text('Apply rules'));
    await waitFor(find.text('Claude Code · Existing file kept'));
    await tester.tap(
      find.descendant(of: library, matching: find.byTooltip('List view')),
    );
    await tester.pumpAndSettle();
    await tester.tap(
      find.descendant(of: library, matching: find.byTooltip('Remove')),
    );
    await waitFor(find.text('Remove diagnostic-style?'));
    await tester.tap(find.widgetWithText(TextButton, 'Remove'));
    await waitFor(find.text('No rules saved'));
    expect(ruleFile.readAsStringSync(), 'A direct edit.\r\n');
  });
}
