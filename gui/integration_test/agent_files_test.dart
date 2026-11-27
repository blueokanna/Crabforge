import 'dart:convert';
import 'dart:io';

import 'package:crabforge/collection_view.dart';
import 'package:crabforge/main.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

import 'desktop_widgets.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testDesktopWidgets('Publish commands and subagents in both layouts', (
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
    ]) {
      if (Directory(Platform.environment[variable] ?? '').uri !=
          Directory('$root/$profile').uri) {
        throw StateError('Use isolated client profiles');
      }
    }

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

    Future<void> enter(String label, String value) async {
      final field = find.widgetWithText(TextFormField, label);
      await tap(field);
      await tester.enterText(field, value);
      await tester.pumpAndSettle();
    }

    Finder below(Finder parent, Finder child) =>
        find.descendant(of: parent, matching: child);
    Future<void> verifyLayout(Finder page, bool grid) async {
      final toggle = below(page, find.byType(LayoutToggle));
      await waitFor(toggle);
      if (tester.widget<LayoutToggle>(toggle).grid != grid) {
        await tap(below(toggle, find.byType(IconButton)));
      }
      await waitFor(
        below(page, grid ? find.byType(SliverGrid) : find.byType(SliverList)),
      );
    }

    await tap(
      below(find.byType(NavigationBar), find.text('Client connections')),
    );
    await tap(find.text('Slash commands'));
    final commands = find.byKey(const ValueKey('agent-commands'));
    await tap(below(commands, find.text('Add slash command')));
    await enter('Name', 'check');
    await enter('Prompt', r'Check $ARGUMENTS.');
    await tap(find.widgetWithText(FilterChip, 'Gemini CLI'));
    await tap(find.widgetWithText(FilledButton, 'Save'));
    final command = find.byKey(const ValueKey('check'));
    await waitFor(command);
    await verifyLayout(commands, false);
    for (final profile in ['gemini', 'agents/gemini']) {
      expect(
        File('$root/$profile/.gemini/commands/check.toml').readAsStringSync(),
        contains('{{args}}'),
      );
    }
    tester.view.physicalSize = const Size(1280, 900);
    await tester.pumpAndSettle();
    await verifyLayout(commands, true);
    final native = File('$root/claude/commands/native-check.md');
    native.parent.createSync(recursive: true);
    const original =
        '\ufeff---\r\ndescription: Existing command\r\n---\r\n\r\nReview the file.\r\n';
    native.writeAsStringSync(original);
    await tap(below(commands, find.text('Find existing files')));
    final discovery = find.byKey(const ValueKey('agent-file-discovery'));
    await waitFor(discovery);
    await verifyLayout(discovery, false);
    await verifyLayout(discovery, true);
    await tap(
      below(
        find.ancestor(
          of: find.text('native-check'),
          matching: find.byType(CollectionTile),
        ),
        find.byTooltip('Manage in Crabforge'),
      ),
    );
    await tap(find.widgetWithText(FilledButton, 'Manage in Crabforge'));
    await waitFor(find.byKey(const ValueKey('native-check')));
    expect(native.readAsBytesSync(), utf8.encode(original));
    final external = File('$root/compare.toml')
      ..writeAsStringSync(
        "description = 'Compare files'\nprompt = 'Compare {{args}}.'\n",
      );
    await tap(below(commands, find.text('Import slash commands')));
    final commandImporter = find.byKey(const ValueKey('import-commands'));
    await tester.enterText(
      below(commandImporter, find.byType(TextField)),
      external.path,
    );
    await tap(below(commandImporter, find.text('Scan source')));
    await verifyLayout(commandImporter, false);
    await verifyLayout(commandImporter, true);
    await tap(below(commandImporter, find.byType(Checkbox)));
    await tap(
      below(commandImporter, find.widgetWithText(FilterChip, 'Claude Code')),
    );
    await tap(
      below(
        commandImporter,
        find.widgetWithText(FilledButton, 'Import slash commands'),
      ),
    );
    await waitFor(find.byKey(const ValueKey('compare')));
    expect(
      File('$root/claude/commands/compare.md').readAsStringSync(),
      contains(r'Compare $ARGUMENTS.'),
    );
    await tap(below(commands, find.widgetWithText(TextButton, 'Close')));
    expect(commands, findsNothing);

    final rail = find.byType(NavigationRail);
    await tap(below(rail, find.text('Providers')));
    await tap(below(rail, find.text('Client connections')));
    final clientPage = find.byKey(const PageStorageKey(5));
    await waitFor(clientPage);
    final scrollable = tester.state<ScrollableState>(
      below(clientPage, find.byType(Scrollable)),
    );
    scrollable.position.jumpTo(scrollable.position.minScrollExtent);
    await tester.pumpAndSettle();
    await tap(below(clientPage, find.text('Subagents')));
    final subagents = find.byKey(const ValueKey('agent-subagents'));
    await tap(below(subagents, find.text('Add subagent')));
    await enter('Name', 'reviewer');
    await enter('Description', 'Reviews changes');
    await enter('Prompt', 'Report findings.');
    await tap(find.widgetWithText(FilterChip, 'Claude Code'));
    await tap(find.widgetWithText(FilterChip, 'Codex'));
    await tap(find.widgetWithText(FilledButton, 'Save'));
    final reviewer = find.byKey(const ValueKey('reviewer'));
    await waitFor(reviewer);
    await verifyLayout(subagents, true);
    final claude = File('$root/claude/agents/reviewer.md');
    final codex = File('$root/codex/agents/reviewer.toml');
    expect(claude.readAsStringSync(), contains('Report findings.'));
    expect(codex.readAsStringSync(), contains('developer_instructions'));
    claude.writeAsStringSync('External edit');

    tester.view.physicalSize = const Size(560, 800);
    await tester.pumpAndSettle();
    await verifyLayout(subagents, false);
    await tap(below(reviewer, find.byTooltip('Remove')));
    await tap(find.widgetWithText(TextButton, 'Remove'));
    await waitFor(find.text('Claude Code · Existing file kept'));
    expect(claude.readAsStringSync(), 'External edit');
    expect(codex.existsSync(), isFalse);

    final source = File('$root/external/review.md');
    source.parent.createSync(recursive: true);
    void writeSource(String prompt) => source.writeAsStringSync(
      '---\nname: imported-review\ndescription: Checks a change\n---\n$prompt\n',
    );

    final importer = find.byKey(const ValueKey('import-subagents'));
    final candidate = below(importer, find.text('imported-review'));
    Future<void> scanSource({Finder? saved}) async {
      if (saved == null) {
        await tap(below(subagents, find.text('Import subagents')));
        await tester.enterText(
          below(importer, find.byType(TextField)),
          source.path,
        );
        await tester.pumpAndSettle();
      } else {
        await tap(below(saved, find.byTooltip('Check for updates')));
      }
      await tap(below(importer, find.text('Scan source')));
      await waitFor(candidate);
    }

    Future<void> selectSource() => tap(below(importer, find.byType(Checkbox)));
    Future<void> importSource() => tap(
      below(importer, find.widgetWithText(FilledButton, 'Import subagents')),
    );

    writeSource('First prompt.');
    await scanSource();
    await verifyLayout(importer, false);
    await selectSource();
    await tap(below(importer, find.widgetWithText(FilterChip, 'Claude Code')));
    await importSource();
    final imported = find.byKey(const ValueKey('imported-review'));
    await waitFor(imported);
    final nativeImport = File('$root/claude/agents/imported-review.md');
    expect(nativeImport.readAsStringSync(), contains('First prompt.'));

    writeSource('Revised prompt.');
    tester.view.physicalSize = const Size(1280, 900);
    await tester.pumpAndSettle();
    await scanSource(saved: imported);
    await verifyLayout(importer, true);
    await waitFor(below(importer, find.text('Update available')));
    await selectSource();
    await importSource();
    await waitFor(imported);
    expect(nativeImport.readAsStringSync(), contains('Revised prompt.'));

    await scanSource(saved: imported);
    await waitFor(below(importer, find.text('Already imported')));
    expect(
      tester.widget<Checkbox>(below(importer, find.byType(Checkbox))).onChanged,
      isNull,
    );
    await tap(below(importer, find.text('Cancel')));
  });
}
