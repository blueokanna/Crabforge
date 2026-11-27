import 'dart:convert';
import 'dart:io';

import 'package:crabforge/main.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

import 'desktop_widgets.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testDesktopWidgets('Manage Claude plugins in both layouts', (tester) async {
    final root = Platform.environment['CRABFORGE_HOME'] ?? '';
    final profile = Directory('$root/claude');
    if (!Directory(root).uri.pathSegments
            .any((part) => part.startsWith('crabforge-gui-test-')) ||
        Directory(Platform.environment['CLAUDE_CONFIG_DIR'] ?? '').uri !=
            profile.uri) {
      throw StateError('Use an isolated Claude Code profile');
    }

    void write(String path, Object data) {
      final file = File(path);
      file.parent.createSync(recursive: true);
      file.writeAsStringSync(jsonEncode(data));
    }

    const marketplaceName = 'crabforge-desktop-fixture';
    const id = 'probe@$marketplaceName';
    final market = Directory('$root/marketplace');
    final project = Directory('$root/project')..createSync(recursive: true);
    final local = File('${project.path}/.claude/settings.local.json');
    write('${market.path}/.claude-plugin/marketplace.json', {
      'name': marketplaceName,
      'owner': {'name': 'Crabforge fixture'},
      'plugins': [
        {'name': 'probe', 'source': './probe', 'description': 'Local fixture'},
      ],
    });
    write('${market.path}/probe/.claude-plugin/plugin.json', {
      'name': 'probe',
      'version': '1.0.0',
    });
    write(local.path, {
      'env': {'KEEP': 'yes'},
    });

    addTearDown(tester.binding.platformDispatcher.clearLocalesTestValue);
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);
    tester.binding.platformDispatcher.localesTestValue = [const Locale('en')];
    tester.view.devicePixelRatio = 1;
    tester.view.physicalSize = const Size(560, 800);
    await tester.pumpWidget(const CrabforgeApp());

    Future<void> waitFor(Finder target, {bool enabled = false}) async {
      final deadline = DateTime.now().add(const Duration(seconds: 30));
      bool ready() =>
          target.evaluate().isNotEmpty &&
          (!enabled ||
              target.evaluate().first.widget is! IconButton ||
              (target.evaluate().first.widget as IconButton).onPressed != null);
      while (!ready() && DateTime.now().isBefore(deadline)) {
        await tester.pump(const Duration(milliseconds: 50));
      }
      expect(target, findsOneWidget);
      expect(ready(), isTrue);
      await tester.pumpAndSettle();
    }

    Future<void> tap(Finder target) async {
      await waitFor(target, enabled: true);
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
    await tap(find.text('Claude Code plugins'));
    final page = find.byKey(const ValueKey('claude-plugins'));
    await waitFor(
      find.text('No Claude Code plugins installed for this scope.'),
    );
    final toList = below(page, find.byTooltip('List view'));
    if (toList.evaluate().isNotEmpty) await tap(toList);
    await tap(below(page, find.text('Add marketplace')));
    await tester.enterText(
      find.widgetWithText(TextField, 'Marketplace path or URL'),
      market.path,
    );
    await tap(find.widgetWithText(FilledButton, 'Add marketplace'));
    await waitFor(below(page, find.text(marketplaceName)));
    await tap(below(page, find.text('Browse plugins')));
    final catalog = find.byType(AlertDialog);
    await waitFor(below(catalog, find.text(id)));
    await tap(below(catalog, find.byTooltip('Install plugin')));
    final plugin = find.byKey(const ValueKey(id));
    await waitFor(plugin);
    expect(below(page, find.byType(SliverGrid)), findsNothing);
    await tap(below(plugin, find.byTooltip('Disable plugin')));
    await waitFor(below(plugin, find.byTooltip('Enable plugin')));

    tester.view.physicalSize = const Size(1280, 900);
    await tester.pumpAndSettle();
    await tap(below(page, find.byTooltip('Grid view')));
    expect(below(page, find.byType(SliverGrid)), findsNWidgets(2));
    await tap(below(page, find.text('Open project')));
    await tester.enterText(
      find.widgetWithText(TextFormField, 'Project folder'),
      project.path,
    );
    await tap(find.widgetWithText(FilledButton, 'Open project'));
    await tap(below(plugin, find.byTooltip('Enable plugin')));
    await waitFor(below(plugin, find.byTooltip('Remove')));
    expect(jsonDecode(local.readAsStringSync())['enabledPlugins'][id], true);

    tester.view.physicalSize = const Size(560, 800);
    await tester.pumpAndSettle();
    await tap(below(page, find.byTooltip('List view')));
    await tap(below(plugin, find.byTooltip('Remove')));
    await waitFor(below(plugin, find.byTooltip('Enable plugin')));
    final settings = jsonDecode(local.readAsStringSync());
    expect(settings['enabledPlugins'][id], isNull);
    expect(settings['env']['KEEP'], 'yes');

    await tap(below(page, find.text('User profiles')));
    final menu = below(page, find.byType(PopupMenuButton<String>));
    await tap(menu);
    await tap(find.text('Import plugin settings'));
    await tester.enterText(
      find.widgetWithText(TextField, 'JSON'),
      '{"enabled":["example@claude-plugins-official"]}',
    );
    await tap(find.widgetWithText(FilledButton, 'Import plugin settings'));
    final user = File('${profile.path}/settings.json');
    await waitFor(
      below(plugin, find.byTooltip('Enable plugin')),
      enabled: true,
    );
    expect(jsonDecode(user.readAsStringSync())['enabledPlugins'], {
      id: false,
      'example@claude-plugins-official': true,
    });
    await tap(menu);
    await tap(find.text('Export plugin settings'));
    final payload = below(
      find.byType(AlertDialog),
      find.byType(SelectableText),
    );
    await waitFor(payload);
    expect(jsonDecode(tester.widget<SelectableText>(payload).data!), {
      'marketplaces': {},
      'enabled': ['example@claude-plugins-official'],
    });
  });
}
