import 'dart:io';

import 'package:crabforge/agent_skills.dart';
import 'package:crabforge/collection_view.dart';
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
      find.widgetWithText(TextField, 'Local folder or Git URL'),
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
    final preserved = find.ancestor(
      of: find.text('Claude Code · Existing file kept'),
      matching: find.byType(CollectionTile),
    );
    expect(tester.widget<CollectionTile>(preserved).grid, isFalse);
    await tap(action('Grid view'));
    expect(tester.widget<CollectionTile>(preserved).grid, isTrue);
    await tap(action('List view'));

    const other = 'crabforge-installed-fixture';
    final own = Directory('${claude.path}/skills/$other')..createSync();
    File(
      '${own.path}/SKILL.md',
    ).writeAsStringSync(doc.replaceFirst('name: diagnostics', 'name: $other'));
    final ownAttachment = File('${own.path}/units.bin')
      ..writeAsBytesSync([6, 7, 8]);
    await tap(find.text('Installed skills'));
    final installedPage = find.byKey(const ValueKey('installed-skills'));
    Finder installedAction(String label) =>
        find.descendant(of: installedPage, matching: find.byTooltip(label));
    await waitFor(find.text(other));
    await tap(installedAction('Grid view'));
    expect(
      find.descendant(of: installedPage, matching: find.byType(SliverGrid)),
      findsOneWidget,
    );
    final candidate = find.ancestor(
      of: find.descendant(of: installedPage, matching: find.text(other)),
      matching: find.byType(CollectionTile),
    );
    await tap(
      find.descendant(
        of: candidate,
        matching: find.byTooltip('Manage in Crabforge'),
      ),
    );
    await tap(find.widgetWithText(FilledButton, 'Manage in Crabforge'));
    final adopted = find.byKey(const ValueKey(other));
    await waitFor(adopted);
    expect(ownAttachment.readAsBytesSync(), [6, 7, 8]);
    await tap(find.descendant(of: adopted, matching: find.byTooltip('Remove')));
    await tap(find.widgetWithText(TextButton, 'Remove'));
    await waitFor(find.text('No skills imported'));
    expect(own.existsSync(), isFalse);

    final repository = Directory('${home.path}/skill-repository')..createSync();
    Future<void> git(List<String> args) async {
      final result = await Process.run('git', [
        '-c',
        'user.name=Crabforge fixture',
        '-c',
        'user.email=fixture@example.test',
        '-c',
        'commit.gpgSign=false',
        ...args,
      ], workingDirectory: repository.path);
      expect(result.exitCode, 0, reason: '${result.stderr}');
    }

    final document = File('${repository.path}/SKILL.md')
      ..writeAsStringSync(
        doc.replaceFirst('name: diagnostics', 'name: git-fixture'),
      );
    await git(['init', '-q']);
    await git(['add', 'SKILL.md']);
    await git(['commit', '-qm', 'Add skill']);
    await tap(find.text('Import skill'));
    await tester.enterText(
      find.widgetWithText(TextField, 'Local folder or Git URL'),
      repository.uri.toString(),
    );
    await tap(find.widgetWithText(FilledButton, 'Import skill'));
    final fromGit = find.byKey(const ValueKey('git-fixture'));
    await waitFor(fromGit);
    await tap(
      find.descendant(
        of: fromGit,
        matching: find.byTooltip('Edit skill assignments'),
      ),
    );
    await tap(find.text('Check for updates'));
    await waitFor(find.text('The skill files are current.'));
    document.writeAsStringSync(
      '${document.readAsStringSync()}\nUse SI units.\n',
    );
    await git(['add', 'SKILL.md']);
    await git(['commit', '-qm', 'Update skill']);
    await tap(find.text('Check for updates'));
    await waitFor(
      find.text(
        'New skill files are available. Use Reimport files to apply them.',
      ),
    );
    await tap(find.text('Reimport files'));
    await waitFor(fromGit);
    expect(find.byType(AlertDialog), findsNothing);

    final sibling = Directory('${repository.path}/tools/sibling')
      ..createSync(recursive: true);
    File('${sibling.path}/SKILL.md').writeAsStringSync(
      doc.replaceFirst('name: diagnostics', 'name: sibling-fixture'),
    );
    await git(['add', 'tools/sibling/SKILL.md']);
    await git(['commit', '-qm', 'Add a second skill']);
    await tap(
      find.descendant(
        of: fromGit,
        matching: find.byTooltip('Edit skill assignments'),
      ),
    );
    tester.view.physicalSize = const Size(560, 800);
    await tester.pumpAndSettle();
    await tap(find.text('Browse source skills'));
    final catalog = find.byKey(const ValueKey('repository-skills'));
    await waitFor(
      find.descendant(of: catalog, matching: find.text('sibling-fixture')),
    );
    expect(
      find.descendant(of: catalog, matching: find.text('git-fixture')),
      findsNothing,
    );
    final wasGrid = find
        .descendant(of: catalog, matching: find.byType(SliverGrid))
        .evaluate()
        .isNotEmpty;
    await tap(
      find.descendant(
        of: catalog,
        matching: find.byTooltip(wasGrid ? 'List view' : 'Grid view'),
      ),
    );
    expect(
      find.descendant(
        of: catalog,
        matching: find.byType(wasGrid ? SliverList : SliverGrid),
      ),
      findsOneWidget,
    );
    await tap(
      find.descendant(of: catalog, matching: find.byTooltip('Import skill')),
    );
    final importAction = find.widgetWithText(FilledButton, 'Import skill');
    final importDialog = find.ancestor(
      of: importAction,
      matching: find.byType(AlertDialog),
    );
    await tap(
      find.descendant(
        of: importDialog,
        matching: find.widgetWithText(FilterChip, 'Codex'),
      ),
    );
    await tap(importAction);
    await waitFor(find.byKey(const ValueKey('sibling-fixture')));
    expect(
      File('${home.path}/codex/skills/sibling-fixture/SKILL.md').existsSync(),
      isTrue,
    );

    final collections = Directory('${home.path}/local-skills');
    final deferred = Directory('${collections.path}/deferred')
      ..createSync(recursive: true);
    File('${deferred.path}/SKILL.md').writeAsStringSync(
      doc.replaceFirst('name: diagnostics', 'name: deferred-fixture'),
    );
    await tap(find.descendant(of: page, matching: find.text('Import skill')));
    await tester.enterText(
      find.widgetWithText(TextField, 'Local folder or Git URL'),
      collections.path,
    );
    final browse = find.text('Browse source skills');
    await waitFor(browse);
    await tap(browse);
    await waitFor(
      find.descendant(of: catalog, matching: find.text('deferred-fixture')),
    );
    await tap(
      find.descendant(of: catalog, matching: find.byTooltip('Set skill aside')),
    );
    final empty = find.text('No additional skills found in this source.');
    await waitFor(empty);
    await tap(find.descendant(of: catalog, matching: find.text('Close')));
    await tap(browse);
    await waitFor(empty);
    await tap(find.descendant(of: catalog, matching: find.text('Close')));
    await tap(find.descendant(of: importDialog, matching: find.text('Cancel')));
    await tap(find.descendant(of: page, matching: find.text('Import skill')));
    final editorField = find.widgetWithText(
      TextField,
      'Local folder or Git URL',
    );
    await tester.enterText(editorField, deferred.path);
    await tester.pumpAndSettle();
    expect(
      tester.widget<TextField>(editorField).controller!.text,
      deferred.path,
    );
    await tap(
      find.descendant(
        of: importDialog,
        matching: find.widgetWithText(FilterChip, 'Claude Code'),
      ),
    );
    await tap(importAction);
    expect(importDialog, findsNothing);
    expect(
      File('${claude.path}/skills/deferred-fixture/SKILL.md').existsSync(),
      isTrue,
    );
  });
}
