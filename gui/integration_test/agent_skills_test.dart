import 'dart:io';

import 'package:crabforge/agent_skills.dart';
import 'package:crabforge/main.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

import 'desktop_widgets.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testDesktopWidgets('Import a skill folder and manage its native copy', (
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
    final source = Directory('${home.path}/source');
    final references = Directory('${source.path}/references')
      ..createSync(recursive: true);
    const doc =
        '---\nname: diagnostics\ndescription: Format diagnostic tables.\n---\n\nShow units.\n';
    File('${source.path}/SKILL.md').writeAsStringSync(doc);
    File('${references.path}/units.bin').writeAsBytesSync([0, 125, 255]);
    final installed = Directory('${claude.path}/skills/diagnostics');
    final attachment = File('${installed.path}/references/units.bin');

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
      await tester.ensureVisible(target);
      await tester.pumpAndSettle();
      await tester.tap(target);
      await tester.pumpAndSettle();
    }

    await waitFor(find.byType(NavigationBar));
    await tap(
      find.descendant(
        of: find.byType(NavigationBar),
        matching: find.text('Client connections'),
      ),
    );
    await waitFor(find.text('Skills'));
    await tap(find.text('Skills'));
    final page = find.byType(AgentSkills);
    Finder action(String label) =>
        find.descendant(of: page, matching: find.byTooltip(label));
    await waitFor(find.text('No skills imported'));
    await tap(find.text('Import skill'));
    await waitFor(find.byType(AlertDialog));
    await tester.enterText(
      find.widgetWithText(TextField, 'Local folder or GitHub URL'),
      source.path,
    );
    await tap(find.widgetWithText(FilterChip, 'Claude Code'));
    await tap(find.widgetWithText(FilledButton, 'Import skill'));
    await waitFor(find.text('Claude Code · Applied'));
    expect(File('${installed.path}/SKILL.md').readAsStringSync(), doc);
    expect(attachment.readAsBytesSync(), [0, 125, 255]);

    await tap(action('Grid view'));
    expect(
      find.descendant(of: page, matching: find.byType(SliverGrid)),
      findsOneWidget,
    );
    tester.view.physicalSize = const Size(1280, 900);
    await tester.pumpAndSettle();
    await tap(action('Edit skill assignments'));
    await waitFor(find.byType(AlertDialog));
    File('${references.path}/units.bin').writeAsBytesSync([1, 2, 3]);
    await tap(find.text('Reimport files'));
    await waitFor(find.text('Claude Code · Applied'));
    expect(attachment.readAsBytesSync(), [1, 2, 3]);

    attachment.writeAsBytesSync([4, 5, 6]);
    await tap(find.text('Apply skills'));
    await waitFor(find.text('Claude Code · Existing file kept'));
    await tap(action('List view'));
    await tap(action('Remove'));
    await waitFor(find.text('Remove diagnostics?'));
    await tap(find.widgetWithText(TextButton, 'Remove'));
    await waitFor(find.text('No skills imported'));
    expect(attachment.readAsBytesSync(), [4, 5, 6]);
  });
}
