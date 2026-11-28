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
  testDesktopWidgets('WebDAV memory sync previews and applies across layouts', (
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
    const original = '# Fixture sync note\n';
    note.writeAsStringSync(original);
    final server = await HttpServer.bind(InternetAddress.loopbackIPv4, 0);
    addTearDown(() => server.close(force: true));
    List<int>? encrypted;
    var writes = 0;
    server.listen((request) async {
      expectSync(request.uri.path, '/dav/crabforge.crabforge-backup');
      expectSync(request.headers.value('authorization'), 'Basic dXNlcjpwYXNz');
      if (request.method == 'PUT') {
        writes++;
        expectSync(request.headers.value('if-none-match'), '*');
        encrypted = await request.expand((chunk) => chunk).toList();
        request.response.statusCode = HttpStatus.created;
      } else if (encrypted case final List<int> stored) {
        request.response.headers.set('etag', '"fixture"');
        request.response.contentLength = stored.length;
        request.response.add(stored);
      } else {
        request.response.statusCode = HttpStatus.notFound;
      }
      await request.response.close();
    });
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

    Future<void> enter(String label, String value) async {
      final field = find.widgetWithText(TextField, label);
      await waitFor(field);
      await tester.enterText(field, value);
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
    await tap(find.widgetWithText(TextButton, strings.memoryLibrary));
    await tap(find.widgetWithText(TextButton, strings.memorySync));
    await enter(strings.memorySyncUrl, 'http://127.0.0.1:${server.port}/dav/');
    await enter(strings.memorySyncUser, 'user');
    await enter(strings.memorySyncPassword, 'pass');
    await enter(strings.memorySyncPassphrase, 'fixture backup key');
    await tap(find.widgetWithText(TextButton, strings.memorySyncPreview));
    await waitFor(find.text(strings.memorySyncChanges('0', '0', '0', '1')));
    expect(writes, 0);
    tester.view.physicalSize = const Size(560, 800);
    await tester.pumpAndSettle();
    await tap(find.widgetWithText(FilledButton, strings.memorySyncApply));
    await waitFor(find.text(strings.memorySyncComplete));
    expect(writes, 1);
    final saved = utf8.decode(encrypted!);
    expect(jsonDecode(saved)['format'], 'crabforge-backup');
    expect(saved, isNot(contains('Fixture sync note')));
    expect(note.readAsStringSync(), original);
    expect(tester.takeException(), isNull);
  });
}
