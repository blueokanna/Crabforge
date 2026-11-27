import 'dart:convert';
import 'dart:io';

import 'package:crabforge/l10n/app_localizations.dart';
import 'package:crabforge/main.dart';
import 'package:crabforge/material_symbols.dart';
import 'package:crabforge/src/rust/api.dart' as api;
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

import 'desktop_widgets.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();
  testDesktopWidgets('Edit Oh My Pi settings in an isolated profile', (
    tester,
  ) async {
    final home = Platform.environment['CRABFORGE_HOME'];
    if (home == null ||
        !Directory(home).uri.pathSegments
            .any((p) => p.startsWith('crabforge-gui-test-'))) {
      throw StateError(
        'Set CRABFORGE_HOME to a new temporary crabforge-gui-test-* directory',
      );
    }
    final dispatcher = tester.binding.platformDispatcher;
    dispatcher.localesTestValue = [const Locale('zh')];
    addTearDown(dispatcher.clearLocalesTestValue);
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);
    tester.view.devicePixelRatio = 1;
    tester.view.physicalSize = const Size(1280, 900);
    final s = lookupAppLocalizations(const Locale('zh'));
    Future<void> settle() => tester.pumpAndSettle(
      const Duration(milliseconds: 50),
      EnginePhase.sendSemanticsUpdate,
      const Duration(seconds: 8),
    );

    Future<void> waitFor(Finder finder) async {
      final until = DateTime.now().add(const Duration(seconds: 45));
      while (finder.evaluate().isEmpty && DateTime.now().isBefore(until)) {
        await tester.pump(const Duration(milliseconds: 100));
      }
      expect(finder, findsWidgets);
    }

    Future<void> tap(Finder finder) async {
      await waitFor(finder);
      await tester.ensureVisible(finder.last);
      await waitFor(finder.hitTestable());
      await tester.tap(finder.hitTestable().last);
      await settle();
    }

    final profile = Directory('$home/omp/profiles/work/agent');
    final file = File('${profile.path}/config.yml');
    final restart = Platform.environment['CRABFORGE_RESTART_CHECK'] == '1';
    if (!restart) {
      profile.createSync(recursive: true);
      File('$home/settings.json').writeAsStringSync(
        jsonEncode({
          'providers': [
            {
              'id': 'local',
              'models': ['m'],
              'endpoints': {'chat': 'http://127.0.0.1:1/v1'},
            },
          ],
        }),
      );
    }
    await tester.pumpWidget(const CrabforgeApp());
    await waitFor(find.text(s.addProvider));
    if (restart) expect(await api.uiLayout(), 'grid');
    await tap(find.text(s.clients));
    final menu = find.byType(DropdownMenu<String>);
    final input = find.descendant(
      of: menu,
      matching: find.byType(EditableText),
    );
    await tester.tap(input);
    await tester.enterText(input, 'omp');
    await settle();
    await tap(find.widgetWithText(MenuItemButton, 'omp'));
    Future<void> open({bool thinking = false}) async {
      final label = find.text(thinking ? s.thinkingLevels : s.advisorSettings);
      await tap(
        find.ancestor(
          of: label,
          matching: find.byWidgetPredicate(
            (w) => w is TextButton && w.onPressed != null,
          ),
        ),
      );
      await tap(find.widgetWithText(ListTile, 'work'));
      await waitFor(
        find.byKey(
          ValueKey(thinking ? 'omp-thinking-session' : 'omp-advisor-review'),
        ),
      );
    }

    Future<void> openTinyRole() async {
      await tap(find.widgetWithText(TextButton, s.roleModels));
      await tap(find.widgetWithText(ListTile, 'work'));
      await waitFor(find.text('${s.roleModels} · work'));
      final tiny = find.widgetWithText(ListTile, 'tiny');
      await tester.scrollUntilVisible(
        tiny,
        140,
        scrollable: find
            .descendant(
              of: find.byType(AlertDialog),
              matching: find.byType(Scrollable),
            )
            .last,
      );
      await tap(tiny);
      await waitFor(find.byKey(const ValueKey('omp-role-model')));
    }

    Future<void> protocol(String checked, String pick) async {
      final button = find.ancestor(
        of: find.text(s.gatewayProtocol),
        matching: find.byWidgetPredicate(
          (w) => w is TextButton && w.onPressed != null,
        ),
      );
      await tap(button);
      await tap(find.widgetWithText(ListTile, 'work'));
      await waitFor(
        find.descendant(
          of: find.widgetWithText(ListTile, checked),
          matching: find.byIcon(CrabforgeSymbols.check),
        ),
      );
      await tap(find.widgetWithText(ListTile, pick));
      await waitFor(button);
    }

    await open();
    if (restart) {
      await waitFor(find.text(s.runEnd));
      await waitFor(find.text(s.blockerNote));
      await tap(find.text(s.cancel));
      await open(thinking: true);
      await waitFor(find.text(s.thinkingAuto));
      await waitFor(find.text(s.thinkingHigh));
      await tap(find.text(s.cancel));
      await openTinyRole();
      await tap(find.text(s.resetRole));
      await waitFor(
        find.widgetWithText(TextButton, s.roleModels).hitTestable(),
      );
      expect(
        (jsonDecode(
          await api.ompRoles(profile: 'work'),
        )['assignments'])['tiny'],
        ['a', 'b'],
      );
      await protocol('OpenAI Responses', s.nativeModelProtocol);
      expect(await api.ompProtocol(profile: 'work'), 'auto');
      return;
    }
    await tap(find.byKey(const ValueKey('omp-advisor-review')));
    await tap(find.text(s.runEnd));
    await tap(find.byKey(const ValueKey('omp-advisor-wait')));
    await tap(find.text('3'));
    await tap(find.text(s.save));
    final until = DateTime.now().add(const Duration(seconds: 20));
    while (!file.existsSync() && DateTime.now().isBefore(until)) {
      await tester.pump(const Duration(milliseconds: 100));
    }
    expect(jsonDecode(await api.ompAdvisor(profile: 'work')), {
      'review': 'agent-end',
      'wait': '3',
      'wake': 'inherit',
    });
    await open();
    await waitFor(find.text(s.runEnd));
    final previous = await api.ompAdvisor(profile: 'work');
    await api.saveOmpAdvisor(
      profile: 'work',
      previous: previous,
      changes: '{"wake":"blocker"}',
    );
    await tap(find.byKey(const ValueKey('omp-advisor-wait')));
    await tap(find.text('5'));
    await tap(find.text(s.save));
    await waitFor(find.textContaining('changed on disk'));
    await tap(find.text(s.cancel));
    expect(jsonDecode(await api.ompAdvisor(profile: 'work')), {
      'review': 'agent-end',
      'wait': '3',
      'wake': 'blocker',
    });
    expect(File('$home/agents/omp/config.yml').existsSync(), isFalse);
    file.writeAsStringSync(
      '\nmodelRoles:\n  slow: local/qwen3:8b\n  tiny: [a, b]\n',
      mode: FileMode.append,
    );
    await open(thinking: true);
    expect(find.byKey(const ValueKey('omp-thinking-tiny')), findsNothing);
    await tap(find.byKey(const ValueKey('omp-thinking-session')));
    await tap(find.text(s.thinkingAuto));
    await tap(find.byKey(const ValueKey('omp-thinking-slow')));
    await tap(find.text(s.thinkingHigh));
    await tap(find.text(s.save));
    final deadline = DateTime.now().add(const Duration(seconds: 20));
    while (!file.readAsStringSync().contains('local/qwen3:8b:high') &&
        DateTime.now().isBefore(deadline)) {
      await tester.pump(const Duration(milliseconds: 100));
    }
    expect(jsonDecode(await api.ompThinking(profile: 'work')), {
      'session': 'auto',
      'roles': {
        'slow': {'model': 'local/qwen3:8b', 'level': 'high'},
      },
    });
    file.writeAsStringSync(
      'enabledModels: [other/m, crabforge/gone:low]\n',
      mode: FileMode.append,
    );
    await tap(find.text(s.syncModelCatalog));
    await tap(find.widgetWithText(ListTile, 'work'));
    await waitFor(find.text('work · ${s.models} (2)'));
    expect(file.readAsStringSync(), isNot(contains('crabforge/gone:low')));
    expect(
      File('${profile.path}/models.yml').readAsStringSync(),
      contains('local/m'),
    );
    await openTinyRole();
    await tap(find.byTooltip(s.gridView));
    await waitFor(find.byType(GridView));
    expect(await api.uiLayout(), 'grid');
    await tap(find.widgetWithText(ListTile, 'crabforge/local/m'));
    await tap(find.text(s.save));
    await waitFor(find.widgetWithText(TextButton, s.roleModels).hitTestable());
    expect(
      jsonDecode(await api.ompRoles(profile: 'work'))['assignments']['tiny'],
      'crabforge/local/m',
    );
    await protocol(s.nativeModelProtocol, 'OpenAI Responses');
    expect(await api.ompProtocol(profile: 'work'), 'openai-responses');
  });
}
