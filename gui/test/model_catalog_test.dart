import 'package:crabforge/material_symbols.dart';
import 'package:crabforge/l10n/app_localizations.dart';
import 'package:crabforge/main.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  testWidgets('Model search and actions survive layout changes', (
    tester,
  ) async {
    await tester.binding.setSurfaceSize(const Size(380, 600));
    addTearDown(() => tester.binding.setSurfaceSize(null));
    String? edited;
    bool grid = false;
    await tester.pumpWidget(
      MaterialApp(
        localizationsDelegates: AppLocalizations.localizationsDelegates,
        home: Scaffold(
          body: StatefulBuilder(
            builder: (context, setState) => ModelCatalog(
              grid: grid,
              onToggle: () => setState(() => grid = !grid),
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
      ),
    );
    await tester.pumpAndSettle();
    await tester.enterText(find.byType(TextField), 'GPT');
    await tester.tap(find.byTooltip('Grid view'));
    await tester.pumpAndSettle();
    expect(find.byType(GridView), findsOneWidget);
    expect(find.text('relay/gpt-example'), findsOneWidget);
    expect(find.byIcon(CrabforgeSymbols.dns), findsOneWidget);
    await tester.enterText(find.byType(TextField), 'GROUP/PARENT');
    await tester.pumpAndSettle();
    expect(find.text('group/parent'), findsOneWidget);
    expect(find.byIcon(CrabforgeSymbols.accountTree), findsOneWidget);
    await tester.tap(find.byIcon(CrabforgeSymbols.edit));
    expect(edited, 'parent');
    await tester.binding.setSurfaceSize(const Size(1200, 750));
    await tester.tap(find.byTooltip('List view'));
    await tester.pumpAndSettle();
    expect(find.byType(ListView), findsOneWidget);
    expect(find.text('group/parent'), findsOneWidget);
  });
}
