import 'dart:convert';
import 'dart:io';

import 'package:crabforge/main.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

import 'desktop_widgets.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testDesktopWidgets('Publish, adopt, remove and restore hooks in both layouts', (
    tester,
  ) async {
    final root = Platform.environment['CRABFORGE_HOME'] ?? '';
    if (!Directory(root).uri.pathSegments
        .any((part) => part.startsWith('crabforge-gui-test-'))) {
      throw StateError('Use an isolated test home');
    }
    for (final (variable, profile) in [
      ('CLAUDE_CONFIG_DIR', 'claude'),
      ('CODEX_HOME', 'codex'),
      ('GEMINI_CLI_HOME', 'gemini'),
      ('CURSOR_HOME', 'cursor'),
      ('GROK_HOME', 'grok'),
    ]) {
      if (Directory(Platform.environment[variable] ?? '').uri !=
          Directory('$root/$profile').uri) {
        throw StateError('Use isolated client profiles');
      }
    }
    final settings = File('$root/claude/settings.json');
    settings.parent.createSync(recursive: true);
    const original =
        '{"env":{"PREEXISTING":"kept"},"hooks":{"SessionStart":[{"hooks":[{"type":"command","command":"echo existing"}]}]}}';
    settings.writeAsStringSync(original);

    addTearDown(tester.binding.platformDispatcher.clearLocalesTestValue);
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);
    tester.binding.platformDispatcher.localesTestValue = [const Locale('en')];
    tester.view.devicePixelRatio = 1;
    tester.view.physicalSize = const Size(560, 800);
    await tester.pumpWidget(const CrabforgeApp());

    Future<void> waitFor(Finder target) async {
      final deadline = DateTime.now().add(const Duration(seconds: 12));
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
    await tap(find.text('Hooks'));
    final page = find.byKey(const ValueKey('agent-hooks'));
    await tap(below(page, find.byTooltip('Adopt hook')));
    await tester.enterText(
      find.widgetWithText(TextField, 'Hook name'),
      'adopted',
    );
    await tap(find.widgetWithText(FilledButton, 'Adopt hook'));
    final adopted = find.byKey(const ValueKey('adopted'));
    await waitFor(adopted);
    expect(settings.readAsStringSync(), original);
    final toList = below(page, find.byTooltip('List view'));
    if (toList.evaluate().isNotEmpty) await tap(toList);
    expect(below(page, find.byType(SliverList)), findsOneWidget);

    tester.view.physicalSize = const Size(1280, 900);
    await tester.pumpAndSettle();
    await tap(below(page, find.byTooltip('Grid view')));
    expect(below(page, find.byType(SliverGrid)), findsOneWidget);
    await tap(find.text('Add hook'));
    await tester.enterText(
      find.widgetWithText(TextField, 'Hook name'),
      'managed',
    );
    await tester.enterText(
      find.widgetWithText(TextField, 'Command run by the client'),
      'echo managed',
    );
    await tap(find.widgetWithText(FilterChip, 'Claude Code'));
    await tap(find.widgetWithText(FilterChip, 'Gemini CLI'));
    await tap(find.widgetWithText(FilledButton, 'Save'));
    final managed = find.byKey(const ValueKey('managed'));
    await waitFor(managed);
    expect(settings.readAsStringSync(), contains('echo managed'));
    expect(
      File('$root/agents/gemini/.gemini/settings.json').readAsStringSync(),
      contains('AfterAgent'),
    );

    settings.writeAsStringSync(
      settings.readAsStringSync().replaceFirst('echo managed', 'echo external'),
    );
    await tap(find.text('Sync assignments'));
    await waitFor(find.text('Claude Code · Existing hooks kept'));
    await tap(below(managed, find.byTooltip('Remove')));
    await tap(find.widgetWithText(TextButton, 'Remove'));
    final config = jsonDecode(settings.readAsStringSync());
    expect(config['env']['PREEXISTING'], 'kept');
    expect(settings.readAsStringSync(), contains('echo existing'));
    expect(settings.readAsStringSync(), contains('echo external'));
    expect(
      File('$root/gemini/.gemini/settings.json').readAsStringSync(),
      isNot(contains('echo managed')),
    );

    final beforeRemoval = settings.readAsBytesSync();
    await tap(below(page, find.byTooltip('Remove installed hook')));
    await tap(find.widgetWithText(FilledButton, 'Remove'));
    final restore = below(page, find.byTooltip('Restore hook backup'));
    await waitFor(restore);
    expect(settings.readAsStringSync(), isNot(contains('echo external')));
    expect(settings.readAsStringSync(), contains('echo existing'));

    tester.view.physicalSize = const Size(560, 800);
    await tester.pumpAndSettle();
    await tap(below(page, find.byTooltip('List view')));
    await tap(restore);
    await tap(find.widgetWithText(FilledButton, 'Restore hook backup'));
    await waitFor(below(page, find.text('Restored')));
    expect(settings.readAsBytesSync(), beforeRemoval);

    final project = Directory('$root/project')..createSync();
    final projectSettings = File('${project.path}/.claude/settings.local.json');
    final grokHooks = File('${project.path}/.grok/hooks/crabforge.json');
    final antigravityHooks = File('${project.path}/.agents/hooks.json');
    await tap(below(page, find.text('Open project')));
    await tester.enterText(
      find.widgetWithText(TextFormField, 'Project folder'),
      project.path,
    );
    await tap(find.widgetWithText(FilledButton, 'Open project'));
    final assign = below(
      adopted,
      find.byTooltip('Edit project hook assignment'),
    );
    await tap(assign);
    await tap(find.widgetWithText(FilterChip, 'Claude Code'));
    await tap(find.widgetWithText(FilledButton, 'Save'));
    await waitFor(below(adopted, find.text('Claude Code · Applied')));
    expect(projectSettings.readAsStringSync(), contains('echo existing'));
    expect(settings.readAsBytesSync(), beforeRemoval);

    tester.view.physicalSize = const Size(1280, 900);
    await tester.pumpAndSettle();
    await tap(below(page, find.byTooltip('Grid view')));
    await tap(assign);
    await tap(find.widgetWithText(FilterChip, 'Claude Code'));
    await tap(find.widgetWithText(FilledButton, 'Save'));
    await waitFor(below(adopted, find.text('Claude Code · Removed')));
    expect(
      projectSettings.readAsStringSync(),
      isNot(contains('echo existing')),
    );
    expect(settings.readAsBytesSync(), beforeRemoval);

    await tap(below(page, find.text('User profiles')));
    await tap(find.text('Add hook'));
    await tester.enterText(
      find.widgetWithText(TextField, 'Hook name'),
      'startup',
    );
    await tester.enterText(
      find.widgetWithText(TextField, 'Command run by the client'),
      'echo project startup',
    );
    await tap(below(find.byType(AlertDialog), find.text('End of turn')));
    await tap(find.text('Session start').last);
    await tap(find.widgetWithText(FilledButton, 'Save'));
    await tap(below(page, find.text('Open project')));
    await tester.enterText(
      find.widgetWithText(TextFormField, 'Project folder'),
      project.path,
    );
    await tap(find.widgetWithText(FilledButton, 'Open project'));
    final startup = find.byKey(const ValueKey('startup'));
    await tap(below(startup, find.byTooltip('Edit project hook assignment')));
    await tap(find.widgetWithText(FilterChip, 'Grok Build'));
    await tap(find.widgetWithText(FilterChip, 'Antigravity'));
    await tap(find.widgetWithText(FilledButton, 'Save'));
    await waitFor(below(startup, find.text('Antigravity · Applied')));
    await waitFor(below(startup, find.text('Grok Build · Applied')));
    final agy = jsonDecode(antigravityHooks.readAsStringSync());
    final grok = jsonDecode(grokHooks.readAsStringSync());
    expect(
      agy['crabforge']['SessionStart'][0]['command'],
      'echo project startup',
    );
    expect(
      grok['hooks']['SessionStart'][0]['hooks'][0]['command'],
      'echo project startup',
    );
    tester.view.physicalSize = const Size(560, 800);
    await tester.pumpAndSettle();
    await tap(below(page, find.byTooltip('List view')));
    await tap(below(startup, find.byTooltip('Edit project hook assignment')));
    await tap(find.widgetWithText(FilterChip, 'Grok Build'));
    await tap(find.widgetWithText(FilterChip, 'Antigravity'));
    await tap(find.widgetWithText(FilledButton, 'Save'));
    await waitFor(below(startup, find.text('Antigravity · Removed')));
    await waitFor(below(startup, find.text('Grok Build · Removed')));
    expect(
      antigravityHooks.readAsStringSync(),
      isNot(contains('echo project startup')),
    );
    expect(
      grokHooks.readAsStringSync(),
      isNot(contains('echo project startup')),
    );
  });
}
