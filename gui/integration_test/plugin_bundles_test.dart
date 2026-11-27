import 'dart:convert';
import 'dart:io';

import 'package:crabforge/main.dart';
import 'package:crabforge/src/rust/api.dart' as api;
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

import 'desktop_widgets.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testDesktopWidgets('Review, update, and split a shared plugin bundle', (
    tester,
  ) async {
    final root = Directory(Platform.environment['CRABFORGE_HOME'] ?? '');
    if (!root.uri.pathSegments.any(
      (part) => part.startsWith('crabforge-gui-test-'),
    )) {
      throw StateError('Use an isolated Crabforge profile');
    }
    final source = Directory('${root.path}/bundle-source');
    final manifest = File('${source.path}/.claude-plugin/plugin.json');
    manifest.parent.createSync(recursive: true);
    void version(String value) => manifest.writeAsStringSync(
      jsonEncode({
        'name': 'desktop-reviewer',
        'description': 'Plugin fixture',
        'version': value,
      }),
    );
    version('1.0.0');
    final agent = File('${source.path}/agents/review.md');
    agent.parent.createSync(recursive: true);
    agent.writeAsStringSync(
      '---\nname: desktop-review\ndescription: Review the fixture\n---\nReview the fixture.\n',
    );
    final other = File('${root.path}/other/.codex-plugin/plugin.json');
    other.parent.createSync(recursive: true);
    other.writeAsStringSync(jsonEncode({'name': 'other-plugin'}));
    final market = File('${root.path}/.claude-plugin/marketplace.json');
    market.parent.createSync(recursive: true);
    market.writeAsStringSync(
      jsonEncode({
        'name': 'Desktop fixture marketplace',
        'plugins': [
          {'name': 'desktop-reviewer', 'source': './bundle-source'},
          {'name': 'other-plugin', 'source': './other'},
          {'name': 'outside', 'source': '../outside'},
        ],
      }),
    );

    addTearDown(tester.binding.platformDispatcher.clearLocalesTestValue);
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);
    tester.binding.platformDispatcher.localesTestValue = [const Locale('en')];
    tester.view.devicePixelRatio = 1;
    tester.view.physicalSize = const Size(560, 800);
    await tester.pumpWidget(const CrabforgeApp());

    Finder below(Finder parent, Finder child) =>
        find.descendant(of: parent, matching: child);
    Future<void> settle() => tester.pumpAndSettle(
      const Duration(milliseconds: 50),
      EnginePhase.sendSemanticsUpdate,
      const Duration(seconds: 30),
    );

    Future<void> waitFor(Finder target) async {
      final deadline = DateTime.now().add(const Duration(seconds: 30));
      while (target.evaluate().isEmpty && DateTime.now().isBefore(deadline)) {
        await tester.pump(const Duration(milliseconds: 50));
      }
      expect(target, findsOneWidget);
      await settle();
    }

    Future<void> tap(Finder target) async {
      await waitFor(target);
      await tester.ensureVisible(target);
      await settle();
      await tester.tap(target);
      await settle();
    }

    await tap(
      below(find.byType(NavigationBar), find.text('Client connections')),
    );
    await tap(find.text('Shared plugin library'));
    final page = find.byKey(const ValueKey('plugin-bundles'));
    await waitFor(below(page, find.text('No plugin bundles saved.')));
    final list = below(page, find.byTooltip('List view'));
    if (list.evaluate().isNotEmpty) await tap(list);
    await tap(below(page, find.text('Import plugin bundle')));
    await tester.enterText(
      find.widgetWithText(TextField, 'Plugin folder, manifest, or Git URL'),
      market.path,
    );
    final dialog = find.byType(AlertDialog).last;
    await tap(find.widgetWithText(TextButton, 'Browse plugins'));
    await waitFor(below(dialog, find.text('Desktop fixture marketplace')));
    expect(below(dialog, find.text('other-plugin')), findsOneWidget);
    await tap(below(dialog, find.text('desktop-reviewer')));
    await waitFor(below(dialog, find.text('agents/review.md')));
    await tap(find.widgetWithText(FilledButton, 'Save'));
    final bundle = below(page, find.byKey(const ValueKey('desktop-reviewer')));
    await waitFor(bundle);
    expect(below(page, find.byType(SliverList)), findsOneWidget);

    tester.view.physicalSize = const Size(1280, 900);
    await settle();
    await tap(below(page, find.byTooltip('Grid view')));
    expect(below(page, find.byType(SliverGrid)), findsOneWidget);
    await tap(below(bundle, find.text('desktop-reviewer')));
    await tap(below(dialog, find.text('agents/review.md')));
    await tap(below(dialog, find.text('desktop-review')));
    final reviewed = below(dialog, find.byType(SelectableText));
    await waitFor(reviewed);
    expect(
      jsonDecode(tester.widget<SelectableText>(reviewed).data!)['prompt'],
      'Review the fixture.',
    );
    await tap(find.widgetWithText(FilledButton, 'Copy to library'));
    await waitFor(below(dialog, find.text('Copied to library')));
    final library = File('${root.path}/subagents/library.json');
    final original = library.readAsStringSync();
    expect(
      jsonDecode(original)['entries']['desktop-review']['agents'],
      isEmpty,
    );
    for (var i = 0; i < 2; i++) {
      await tap(below(dialog, find.text('Close')));
    }

    version('2.0.0');
    await tap(below(bundle, find.byTooltip('Update plugin')));
    await waitFor(below(dialog, find.text('2.0.0')));
    await tap(find.widgetWithText(FilledButton, 'Save'));
    await waitFor(below(bundle, find.text('2.0.0 · Plugin fixture')));
    expect(
      below(bundle, find.byTooltip('Export plugin archive')),
      findsOneWidget,
    );
    final saved = (jsonDecode(await api.pluginBundles()) as List).single;
    final archive = '${root.path}/reviewer.tar';
    await api.exportPluginBundle(
      name: 'desktop-reviewer',
      revision: saved['revision'] as String,
      output: archive,
    );
    await tap(below(bundle, find.byTooltip('Remove')));
    await tap(find.widgetWithText(FilledButton, 'Remove'));
    await waitFor(below(page, find.text('No plugin bundles saved.')));
    await tap(below(page, find.text('Import plugin bundle')));
    await tester.enterText(
      find.widgetWithText(TextField, 'Plugin folder, manifest, or Git URL'),
      archive,
    );
    await tap(find.widgetWithText(FilledButton, 'Preview bundle'));
    await waitFor(below(dialog, find.text('2.0.0')));
    await tap(find.widgetWithText(FilledButton, 'Save'));
    await waitFor(below(bundle, find.text('2.0.0 · Plugin fixture')));
    expect(library.readAsStringSync(), original);
    expect(tester.takeException(), isNull);
  });
}
