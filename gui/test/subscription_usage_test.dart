import 'dart:async';

import 'package:crabforge/collection_view.dart';
import 'package:crabforge/l10n/app_localizations.dart';
import 'package:crabforge/subscription_usage.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  for (final width in [560.0, 1200.0]) {
    testWidgets('displays usage in both layouts at $width', (tester) async {
      tester.view.devicePixelRatio = 1;
      tester.view.physicalSize = Size(width, 640);
      addTearDown(tester.view.resetDevicePixelRatio);
      addTearDown(tester.view.resetPhysicalSize);
      final grid = ValueNotifier(false);
      addTearDown(grid.dispose);
      final read = Completer<String>();
      var reads = 0;
      await tester.pumpWidget(
        MaterialApp(
          localizationsDelegates: AppLocalizations.localizationsDelegates,
          supportedLocales: AppLocalizations.supportedLocales,
          locale: const Locale('en'),
          builder: (_, child) => ValueListenableBuilder(
            valueListenable: grid,
            builder: (_, value, _) => CollectionLayout(
              grid: value,
              toggle: () => grid.value = !value,
              child: child!,
            ),
          ),
          home: Builder(
            builder: (context) => TextButton(
              onPressed: () => showDialog<void>(
                context: context,
                builder: (_) => SubscriptionUsageDialog(
                  provider: 'fixture',
                  read: () {
                    reads++;
                    return read.future;
                  },
                ),
              ),
              child: const Text('Open usage'),
            ),
          ),
        ),
      );
      await tester.tap(find.text('Open usage'));
      await tester.pump();
      expect(find.byType(CircularProgressIndicator), findsOneWidget);
      read.complete('''{"plan":"Fixture Plan","windows":[
        {"name":"5 hours","used_percent":42,"resets_at":"2026-11-29T10:00:00Z"},
        {"name":"Purchased credits","balance":4.5,"unit":"USD","aside":true},
        {"name":"Separate allowance","aside":true}
      ]}''');
      await tester.pumpAndSettle();
      expect(find.byType(ListView), findsOneWidget);
      expect(find.text('fixture · Fixture Plan'), findsOneWidget);
      expect(find.text('42% used'), findsOneWidget);
      expect(find.text('Remaining balance: 4.5 USD'), findsOneWidget);
      expect(find.byType(LinearProgressIndicator), findsOneWidget);
      await tester.tap(find.byTooltip('Grid view'));
      await tester.pumpAndSettle();
      expect(find.byType(GridView), findsOneWidget);
      expect(find.text('42% used'), findsOneWidget);
      expect(reads, 1);
      expect(tester.takeException(), isNull);
      await tester.tap(find.text('Close'));
      await tester.pumpAndSettle();
      expect(find.byType(SubscriptionUsageDialog), findsNothing);
    });
  }
}
