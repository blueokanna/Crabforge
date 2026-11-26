import 'dart:convert';
import 'dart:io';
import 'dart:ui' as ui;

import 'package:crabforge/l10n/app_localizations.dart';
import 'package:crabforge/main.dart';
import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

import 'desktop_widgets.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();
  testDesktopWidgets('Windows console pages adapt to both navigation layouts', (
    tester,
  ) async {
    final home = Directory(Platform.environment['CRABFORGE_HOME'] ?? '');
    if (!home.uri.pathSegments.any(
      (s) => s.startsWith('crabforge-gui-test-'),
    )) {
      throw StateError('Use an isolated crabforge-gui-test-* data directory');
    }
    final settings = File('${home.path}/settings.json');
    expect(settings.existsSync(), isFalse);
    home.createSync(recursive: true);
    settings.writeAsStringSync(
      jsonEncode({
        'listen': '127.0.0.1:0',
        'providers': [
          for (final (id, model) in [
            ('openai', 'gpt-4.1'),
            ('anthropic', 'claude-sonnet-4'),
          ])
            {
              'id': id,
              'models': [model],
              'endpoints': {'chat': 'http://127.0.0.1:1/v1'},
            },
        ],
        'groups': {
          'review': ['openai/gpt-4.1', 'anthropic/claude-sonnet-4'],
        },
      }),
    );
    final output = Platform.environment['CRABFORGE_LAYOUT_CAPTURES'];
    final capture = GlobalKey();
    tester.binding.platformDispatcher.localesTestValue = [const Locale('en')];
    addTearDown(tester.binding.platformDispatcher.clearLocalesTestValue);
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);
    tester.view.devicePixelRatio = 1;
    tester.view.physicalSize = const Size(1200, 800);
    await tester.pumpWidget(
      RepaintBoundary(key: capture, child: const CrabforgeApp()),
    );
    final deadline = DateTime.now().add(const Duration(seconds: 20));
    while (find.text('Add provider').evaluate().isEmpty &&
        DateTime.now().isBefore(deadline)) {
      await tester.pump(const Duration(milliseconds: 100));
    }
    expect(find.text('Add provider'), findsOneWidget);
    final strings = lookupAppLocalizations(const Locale('en'));
    final pages = [
      strings.providers,
      strings.routes,
      strings.groups,
      strings.gateway,
      strings.models,
      strings.clients,
    ];
    for (final width in [1200, 560]) {
      tester.view.physicalSize = Size(width.toDouble(), 800);
      await tester.pumpAndSettle();
      final nav = find.byType(width >= 820 ? NavigationRail : NavigationBar);
      expect(nav, findsOneWidget);
      for (final label in pages) {
        await tester.tap(
          find.descendant(of: nav, matching: find.text(label)).first,
        );
        await tester.pumpAndSettle();
        for (final grid in [false, true]) {
          final toggle = find.byTooltip(
            grid ? strings.listView : strings.gridView,
          );
          expect(toggle, findsOneWidget);
          expect(tester.takeException(), isNull);
          if (output != null) {
            final boundary =
                capture.currentContext!.findRenderObject()!
                    as RenderRepaintBoundary;
            final image = await boundary.toImage();
            try {
              expect(image.width, width);
              expect(image.height, 800);
              final bytes = (await image.toByteData(
                format: ui.ImageByteFormat.png,
              ))!;
              final file = File(
                '$output/$width-${pages.indexOf(label)}-${grid ? 'grid' : 'list'}.png',
              );
              await file.parent.create(recursive: true);
              await file.writeAsBytes(
                bytes.buffer.asUint8List(
                  bytes.offsetInBytes,
                  bytes.lengthInBytes,
                ),
              );
            } finally {
              image.dispose();
            }
          }
          await tester.tap(toggle);
          await tester.pumpAndSettle();
        }
      }
    }
  });
}
