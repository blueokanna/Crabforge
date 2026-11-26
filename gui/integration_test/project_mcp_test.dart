import 'dart:convert';
import 'dart:io';

import 'package:crabforge/main.dart';
import 'package:crabforge/project_mcp.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

import 'desktop_widgets.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testDesktopWidgets('Manage native project MCP entries in both layouts', (
    tester,
  ) async {
    final home = Directory(Platform.environment['CRABFORGE_HOME'] ?? '');
    if (!home.uri.pathSegments.any(
      (s) => s.startsWith('crabforge-gui-test-'),
    )) {
      throw StateError('Use an isolated Crabforge test home');
    }
    final project = Directory('${home.path}/project')
      ..createSync(recursive: true);
    final config = File('${project.path}/.mcp.json');
    config.writeAsStringSync('{"mcpServers":{"existing":{"command":"other"}}}');

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

    await tap(
      find.descendant(
        of: find.byType(NavigationBar),
        matching: find.text('Client connections'),
      ),
    );
    await tap(find.text('Project MCP'));
    final page = find.byType(ProjectMcp);
    await waitFor(page);
    await tester.enterText(
      find.widgetWithText(TextField, 'Project folder'),
      project.path,
    );
    await tap(find.text('Open project'));
    await tap(find.text('Add MCP server'));
    await tester.enterText(
      find.widgetWithText(TextField, 'Server name'),
      'diagnostics',
    );
    await tester.enterText(
      find.widgetWithText(TextField, 'JSON'),
      '{"type":"stdio","command":"fixture"}',
    );
    await tap(find.widgetWithText(FilledButton, 'Save'));
    await waitFor(find.text('diagnostics'));
    Map<String, dynamic> servers() =>
        jsonDecode(config.readAsStringSync())['mcpServers']
            as Map<String, dynamic>;
    expect(servers()['diagnostics']['command'], 'fixture');
    expect(servers()['existing']['command'], 'other');

    Finder action(String label, {Finder? inTile}) =>
        find.descendant(of: inTile ?? page, matching: find.byTooltip(label));
    final tile = find.byKey(const ValueKey('diagnostics'));
    await tap(action('Grid view'));
    expect(
      find.descendant(of: page, matching: find.byType(SliverGrid)),
      findsOneWidget,
    );
    tester.view.physicalSize = const Size(1280, 900);
    await tester.pumpAndSettle();
    await tap(action('Edit MCP server', inTile: tile));
    final external = config.readAsStringSync().replaceFirst(
      '"other"',
      '"external"',
    );
    config.writeAsStringSync(external);
    await tap(find.widgetWithText(FilledButton, 'Save'));
    await waitFor(find.textContaining('project MCP settings changed'));
    expect(config.readAsStringSync(), external);
    await tap(find.widgetWithText(TextButton, 'Cancel'));
    await tap(find.text('Reload'));
    await tap(action('List view'));
    await tap(action('Remove', inTile: tile));
    await tap(find.widgetWithText(TextButton, 'Remove'));
    expect(servers().keys, ['existing']);
    expect(servers()['existing']['command'], 'external');

    await tap(find.text('MCP library'));
    final library = find.byKey(const ValueKey('project-mcp-library'));
    Finder inLibrary(Finder target) =>
        find.descendant(of: library, matching: target);
    await tap(inLibrary(find.text('Add MCP server')));
    await tester.enterText(
      find.widgetWithText(TextField, 'Server name'),
      'shared-diagnostics',
    );
    await tester.enterText(
      find.widgetWithText(TextField, 'JSON'),
      '{"transport":"stdio","command":"shared-fixture"}',
    );
    await tap(find.widgetWithText(FilledButton, 'Save'));
    await tap(find.byTooltip('Assign to this project'));
    await waitFor(find.byTooltip('Unassign from this project'));
    expect(servers()['shared-diagnostics']['command'], 'shared-fixture');
    await tap(inLibrary(find.byTooltip('Grid view')));
    expect(inLibrary(find.byType(SliverGrid)), findsOneWidget);
    tester.view.physicalSize = const Size(560, 800);
    await tester.pumpAndSettle();
    await tap(find.byTooltip('Unassign from this project'));
    await waitFor(find.byTooltip('Assign to this project'));
    expect(servers().keys, ['existing']);
    expect(servers()['existing']['command'], 'external');
  });
}
