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
  testDesktopWidgets(
    'Daily memory publication persists across layouts and restarts',
    (tester) async {
      final root = Directory(Platform.environment['CRABFORGE_HOME'] ?? '');
      final profile = Directory('${root.path}/claude');
      if (!root.uri.pathSegments.any(
            (s) => s.startsWith('crabforge-gui-test-'),
          ) ||
          Directory(Platform.environment['CLAUDE_CONFIG_DIR'] ?? '').uri !=
              profile.uri) {
        throw StateError('Use an isolated data directory and Claude profile');
      }
      profile.createSync(recursive: true);
      final instructions = File('${profile.path}/CLAUDE.md');
      const original = '\ufeff# Native user instructions\r\n';
      final restart = Platform.environment['CRABFORGE_RESTART_CHECK'] == '1';
      expect(instructions.existsSync(), restart);
      if (!restart) instructions.writeAsStringSync(original);

      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      final dispatcher = tester.binding.platformDispatcher;
      addTearDown(dispatcher.clearLocalesTestValue);
      const locale = Locale('en');
      dispatcher.localesTestValue = [locale];
      final strings = lookupAppLocalizations(locale);
      tester.view.devicePixelRatio = 1;
      tester.view.physicalSize = const Size(1200, 800);
      await tester.pumpWidget(const CrabforgeApp());

      Future<void> waitFor(Finder finder) async {
        final deadline = DateTime.now().add(const Duration(seconds: 12));
        do {
          await tester.pump(const Duration(milliseconds: 50));
        } while (finder.evaluate().isEmpty &&
            DateTime.now().isBefore(deadline));
        expect(finder, findsOneWidget);
        await tester.ensureVisible(finder);
        await tester.pumpAndSettle();
      }

      Finder publicationSwitch(bool enabled) => find.byWidgetPredicate(
        (widget) =>
            widget is SwitchListTile &&
            widget.title is Text &&
            (widget.title as Text).data == strings.autoPublishAgentMemory &&
            widget.value == enabled &&
            widget.onChanged != null,
      );

      for (final width in restart ? [560.0] : [1200.0, 560.0]) {
        tester.view.physicalSize = Size(width, 800);
        await tester.pumpAndSettle();
        final clients = find.descendant(
          of: find.byWidgetPredicate(
            (widget) =>
                widget is NavigationBar ||
                widget is NavigationDrawer ||
                widget is NavigationRail,
          ),
          matching: find.text(strings.clients),
        );
        await waitFor(clients);
        await tester.tap(clients.hitTestable());
        final open = find.widgetWithText(TextButton, strings.agentInstructions);
        await waitFor(open);
        await tester.tap(open.hitTestable());
        final toggle = publicationSwitch(restart || width == 560);
        await waitFor(toggle);
        if (restart || width == 1200) {
          await tester.tap(toggle.hitTestable());
        }
        await waitFor(publicationSwitch(!restart));
        final result = jsonDecode(
          await native.agentInstructions(agent: 'claude'),
        );
        expect(result['auto_publish'] == true, !restart);
        expect(instructions.readAsBytesSync(), utf8.encode(original));
        await tester.tap(
          find.widgetWithText(TextButton, 'Close').hitTestable(),
        );
        await tester.pumpAndSettle();
      }
    },
  );
}
