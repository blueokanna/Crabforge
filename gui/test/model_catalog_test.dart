import 'package:crabforge/brand_assets.dart';
import 'package:crabforge/l10n/app_localizations.dart';
import 'package:crabforge/main.dart';
import 'package:flutter/material.dart';
import 'package:flutter_svg/flutter_svg.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  testWidgets('Model search and actions survive layout changes', (
    tester,
  ) async {
    await tester.binding.setSurfaceSize(const Size(380, 600));
    addTearDown(() => tester.binding.setSurfaceSize(null));
    String? edited;
    await tester.pumpWidget(
      MaterialApp(
        localizationsDelegates: AppLocalizations.localizationsDelegates,
        home: Scaffold(
          body: ModelCatalog(
            providers: const [
              {
                'id': 'relay',
                'models': ['gpt-example', 'other'],
              },
            ],
            groupRefs: const {'parent': 'group/parent'},
            canEdit: true,
            onEdit: (_) {},
            onGroup: (id) => edited = id,
          ),
        ),
      ),
    );
    await tester.pumpAndSettle();
    await tester.enterText(find.byType(TextField), 'GPT');
    await tester.tap(find.byTooltip('Grid view'));
    await tester.pumpAndSettle();
    expect(find.byType(GridView), findsOneWidget);
    expect(find.text('relay/gpt-example'), findsOneWidget);
    await tester.enterText(find.byType(TextField), 'GROUP/PARENT');
    await tester.pumpAndSettle();
    expect(find.text('group/parent'), findsOneWidget);
    await tester.tap(find.byIcon(Icons.edit_outlined));
    expect(edited, 'parent');
    await tester.binding.setSurfaceSize(const Size(1200, 750));
    await tester.tap(find.byTooltip('List view'));
    await tester.pumpAndSettle();
    expect(find.byType(ListView), findsOneWidget);
    expect(find.text('group/parent'), findsOneWidget);
  });

  testWidgets('Bundled provider and agent vectors decode', (tester) async {
    for (final asset in {
      ...providerBrandAssets.values,
      ...agentBrandAssets.values,
    }.where((s) => s.endsWith('.svg'))) {
      final data = await SvgAssetLoader(
        'brands/$asset',
        packageName: 'crabforge_assets',
      ).loadBytes(null);
      expect(data.lengthInBytes, greaterThan(0), reason: asset);
    }
  });
}
