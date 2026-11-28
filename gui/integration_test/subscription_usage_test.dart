import 'dart:convert';
import 'dart:io';

import 'package:crabforge/main.dart';
import 'package:crabforge/subscription_usage.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

import 'desktop_widgets.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testDesktopWidgets('Subscription usage opens through the Windows bridge', (
    tester,
  ) async {
    const keyName = 'CRABFORGE_USAGE_FIXTURE_NO_KEY';
    expect(Platform.environment[keyName], isNull);
    final home = Directory(Platform.environment['CRABFORGE_HOME'] ?? '');
    if (!home.uri.pathSegments.any(
      (part) => part.startsWith('crabforge-gui-test-'),
    )) {
      throw StateError('Use an isolated crabforge-gui-test-* data directory');
    }
    final settings = File('${home.path}/settings.json');
    expect(settings.existsSync(), isFalse);
    home.createSync(recursive: true);
    settings.writeAsStringSync(
      jsonEncode({
        'providers': [
          {
            'id': 'go-fixture',
            'key_env': keyName,
            'models': ['go-model'],
            'endpoints': {'chat': 'https://opencode.ai/zen/go/v1'},
          },
          {
            'id': 'local-fixture',
            'key_env': keyName,
            'models': ['local-model'],
            'endpoints': {'chat': 'http://127.0.0.1:1/v1'},
          },
        ],
      }),
    );
    tester.binding.platformDispatcher.localesTestValue = [const Locale('en')];
    addTearDown(tester.binding.platformDispatcher.clearLocalesTestValue);
    tester.view.devicePixelRatio = 1;
    tester.view.physicalSize = const Size(1200, 800);
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);
    await tester.pumpWidget(const CrabforgeApp());

    final usage = find.byTooltip('Subscription usage');
    final missing = find.textContaining(
      'provider key environment variable is missing or empty',
    );
    Future<void> waitFor(Finder target) async {
      final until = DateTime.now().add(const Duration(seconds: 15));
      while (target.evaluate().isEmpty && DateTime.now().isBefore(until)) {
        await tester.pump(const Duration(milliseconds: 100));
      }
      expect(target, findsOneWidget);
    }

    await waitFor(usage);
    for (final width in [1200.0, 560.0]) {
      tester.view.physicalSize = Size(width, 800);
      await tester.pumpAndSettle();
      final dialog = find.byType(SubscriptionUsageDialog);
      Finder layout(String label) =>
          find.descendant(of: dialog, matching: find.byTooltip(label));
      for (final (current, next) in [
        ('Grid view', 'List view'),
        ('List view', 'Grid view'),
      ]) {
        await tester.tap(usage);
        await waitFor(missing);
        expect(dialog, findsOneWidget);
        await tester.tap(layout(current));
        await waitFor(layout(next));
        expect(missing, findsOneWidget);
        await tester.tap(find.widgetWithText(TextButton, 'Close'));
        await tester.pumpAndSettle();
        expect(dialog, findsNothing);
      }
    }
  });
}
