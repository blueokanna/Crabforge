import 'dart:convert';
import 'dart:io';

import 'package:crabforge/l10n/app_localizations.dart';
import 'package:crabforge/main.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

import 'desktop_widgets.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();
  testDesktopWidgets('Nightly tidy can run and undo across layouts', (
    tester,
  ) async {
    final home = Directory(Platform.environment['CRABFORGE_HOME'] ?? '');
    if (!home.uri.pathSegments.any(
      (s) => s.startsWith('crabforge-gui-test-'),
    )) {
      throw StateError('Use an isolated crabforge-gui-test-* data directory');
    }
    final note = File('${home.path}/memory/fixture.md');
    note.parent.createSync(recursive: true);
    const original =
        '\ufeff---\r\norigin: inferred\r\nstatus: candidate\r\n'
        'updated: 2024-01-01T00:00:00Z\r\n---\r\n\r\nReview before release.\r\n';
    note.writeAsStringSync(original);
    File('${home.path}/settings.json').writeAsStringSync(
      jsonEncode({
        'providers': [
          {
            'id': 'fixture',
            'models': ['tidy'],
            'endpoints': {'chat': 'http://127.0.0.1:1/v1'},
          },
        ],
      }),
    );
    final dispatcher = tester.binding.platformDispatcher;
    addTearDown(dispatcher.clearLocalesTestValue);
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);
    const locale = Locale('en');
    dispatcher.localesTestValue = [locale];
    final strings = lookupAppLocalizations(locale);
    tester.view.devicePixelRatio = 1;
    tester.view.physicalSize = const Size(1200, 800);
    await tester.pumpWidget(const CrabforgeApp());

    Future<void> waitFor(Finder finder) async {
      final deadline = DateTime.now().add(const Duration(seconds: 15));
      do {
        await tester.pump(const Duration(milliseconds: 50));
      } while (finder.evaluate().isEmpty && DateTime.now().isBefore(deadline));
      expect(finder, findsOneWidget);
      await tester.ensureVisible(finder);
      await tester.pumpAndSettle();
    }

    Future<void> tap(Finder finder) async {
      await waitFor(finder);
      await tester.tap(finder.hitTestable());
    }

    final clients = find.descendant(
      of: find.byWidgetPredicate(
        (widget) =>
            widget is NavigationRail ||
            widget is NavigationBar ||
            widget is NavigationDrawer,
      ),
      matching: find.text(strings.clients),
    );
    await tap(clients);
    await tap(find.widgetWithText(TextButton, strings.agentSessions));
    final open = find.widgetWithText(TextButton, strings.memoryTidy);
    await tap(open);
    final toggle = find.byType(SwitchListTile);
    await waitFor(toggle);
    expect(tester.widget<SwitchListTile>(toggle).value, isFalse);
    await tap(find.byType(DropdownButtonFormField<String>));
    await tap(find.text('fixture/tidy').hitTestable());
    await tap(toggle);
    final save = find.widgetWithText(FilledButton, strings.save);
    await tap(save);
    final state = File('${home.path}/memory-tidy.json');
    final run = find.byWidgetPredicate(
      (widget) =>
          widget is TextButton &&
          widget.onPressed != null &&
          widget.child is Text &&
          (widget.child as Text).data == strings.memoryTidyRun,
    );
    await waitFor(run);
    expect(
      jsonDecode(state.readAsStringSync())['schedule']['model'],
      'fixture/tidy',
    );
    await tap(run);
    final undo = find.widgetWithText(TextButton, strings.memoryTidyUndo);
    await waitFor(undo);
    expect(note.readAsStringSync(), contains('status: archived'));
    final close = find.descendant(
      of: find.byType(AlertDialog),
      matching: find.widgetWithText(TextButton, 'Close'),
    );
    await tap(close);
    tester.view.physicalSize = const Size(560, 800);
    await tester.pumpAndSettle();
    await tap(open);
    await waitFor(toggle);
    expect(tester.widget<SwitchListTile>(toggle).value, isTrue);
    await tap(undo);
    await waitFor(find.text(strings.memoryTidyRestored('1')));
    expect(note.readAsBytesSync(), utf8.encode(original));
    await tap(toggle);
    await tap(save);
    await waitFor(run);
    expect(jsonDecode(state.readAsStringSync())['schedule'], isNull);
  });
}
