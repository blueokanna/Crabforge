import 'dart:convert';
import 'dart:io';

import 'package:crabforge/l10n/app_localizations.dart';
import 'package:crabforge/main.dart';
import 'package:crabforge/src/rust/api.dart' as native;
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

import 'desktop_widgets.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();
  testDesktopWidgets('Local sessions and scoped user instructions', (
    tester,
  ) async {
    final home = Directory(Platform.environment['CRABFORGE_HOME'] ?? '');
    final claude = Directory('${home.path}/claude');
    final codex = Directory('${home.path}/codex');
    if (!home.uri.pathSegments.any(
          (s) => s.startsWith('crabforge-gui-test-'),
        ) ||
        Directory(Platform.environment['CLAUDE_CONFIG_DIR'] ?? '').uri !=
            claude.uri ||
        Directory(Platform.environment['CODEX_HOME'] ?? '').uri != codex.uri) {
      throw StateError('Use a fresh test home with isolated native profiles');
    }
    final instructions = File('${codex.path}/AGENTS.override.md');
    instructions.parent.createSync(recursive: true);
    instructions.writeAsStringSync('# Existing user instructions\r\n');
    final claudeLog = File('${claude.path}/projects/fixture/session.jsonl');
    claudeLog.parent.createSync(recursive: true);
    claudeLog.writeAsStringSync(
      '${jsonEncode({
        'type': 'user',
        'sessionId': 'claude-fixture-id',
        'cwd': home.path,
        'message': {'content': 'Claude session fixture'},
      })}\n',
    );
    final codexLog = File(
      '${codex.path}/sessions/2026/11/rollout-fixture.jsonl',
    );
    codexLog.parent.createSync(recursive: true);
    codexLog.writeAsStringSync(
      '${jsonEncode({
        'type': 'session_meta',
        'payload': {'id': 'codex-fixture-id', 'cwd': home.path},
      })}\n${jsonEncode({
        'type': 'event_msg',
        'payload': {'type': 'user_message', 'message': 'Codex session fixture'},
      })}\n',
    );
    File('${home.path}/settings.json')
        .writeAsStringSync('{"listen":"127.0.0.1:0"}');
    tester.binding.platformDispatcher.localesTestValue = [const Locale('en')];
    addTearDown(tester.binding.platformDispatcher.clearLocalesTestValue);
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);
    tester.view.devicePixelRatio = 1;
    tester.view.physicalSize = const Size(560, 800);
    await tester.pumpWidget(const CrabforgeApp());

    Future<void> waitFor(Finder target) async {
      final deadline = DateTime.now().add(const Duration(seconds: 10));
      while (target.evaluate().isEmpty && DateTime.now().isBefore(deadline)) {
        await tester.pump(const Duration(milliseconds: 50));
      }
      expect(target, findsOneWidget);
      await tester.pumpAndSettle(
        const Duration(milliseconds: 50),
        EnginePhase.sendSemanticsUpdate,
        const Duration(seconds: 5),
      );
    }

    await waitFor(find.byType(NavigationBar));
    final strings = lookupAppLocalizations(const Locale('en'));
    await tester.tap(
      find.descendant(
        of: find.byType(NavigationBar),
        matching: find.text(strings.clients),
      ),
    );
    final sessions = find.text('Local sessions');
    await waitFor(sessions);
    await tester.ensureVisible(sessions);
    await tester.tap(sessions);
    final claudeTitle = find.text('Claude session fixture');
    final codexTitle = find.text('Codex session fixture');
    await waitFor(codexTitle);
    expect(claudeTitle, findsOneWidget);
    final library = find.byType(Dialog);
    await tester.tap(
      find.descendant(of: library, matching: find.byTooltip('Grid view')),
    );
    await tester.pumpAndSettle();
    expect(
      find.descendant(of: library, matching: find.byType(GridView)),
      findsOneWidget,
    );
    await tester.tap(find.byType(DropdownButton<String>));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Codex').last);
    await waitFor(codexTitle);
    expect(claudeTitle, findsNothing);
    await tester.tap(codexTitle);
    await waitFor(find.text('codex-fixture-id'));
    expect(find.text(strings.sessionProfileFolder), findsOneWidget);
    expect(find.text(Platform.environment['CODEX_HOME']!), findsOneWidget);
    await tester.tap(
      find.descendant(
        of: find.byType(AlertDialog),
        matching: find.text('Close'),
      ),
    );
    await tester.pumpAndSettle();
    await tester.tap(find.text('Close'));
    await tester.pumpAndSettle();

    await tester.runAsync(() async {
      for (final (scope, text) in [
        ('global', 'Global test memory.'),
        ('agent:codex', 'Codex test memory.'),
        ('agent:claude', 'Claude test memory.'),
        ('project:fixture', 'Project test memory.'),
      ]) {
        await native.saveMemory(
          changes: jsonEncode({'scope': scope, 'text': text}),
        );
      }
    });
    tester
        .widget<DropdownMenu<String>>(find.byType(DropdownMenu<String>))
        .onSelected!('codex');
    final userInstructions = find.widgetWithText(
      TextButton,
      strings.agentInstructions,
    );
    await waitFor(userInstructions);
    await Scrollable.ensureVisible(
      tester.element(userInstructions),
      alignment: 0.45,
    );
    await tester.pumpAndSettle();
    expect(userInstructions.hitTestable(), findsOneWidget);
    await tester.tap(userInstructions);
    final memory = find.text(strings.agentMemoryDigest).hitTestable();
    await waitFor(memory);
    await tester.tap(memory);
    final publish = find.text('Publish digest');
    await waitFor(publish);
    expect(find.textContaining('Global test memory.'), findsOneWidget);
    expect(find.textContaining('Codex test memory.'), findsOneWidget);
    expect(find.textContaining('Claude test memory.'), findsNothing);
    expect(find.textContaining('Project test memory.'), findsNothing);
    await tester.tap(publish);
    final restore = find.text('Restore previous version');
    await waitFor(restore);
    expect(
      instructions.readAsStringSync(),
      startsWith('# Existing user instructions\r\n'),
    );
    expect(
      instructions.readAsStringSync(),
      contains('- Codex test memory.\r\n'),
    );
    await tester.tap(restore);
    await tester.pumpAndSettle();
    await tester.tap(
      find.widgetWithText(FilledButton, 'Restore previous version'),
    );
    final deadline = DateTime.now().add(const Duration(seconds: 10));
    while (restore.evaluate().isNotEmpty && DateTime.now().isBefore(deadline)) {
      await tester.pump(const Duration(milliseconds: 50));
    }
    expect(restore, findsNothing);
    expect(instructions.readAsStringSync(), '# Existing user instructions\r\n');
  });
}
