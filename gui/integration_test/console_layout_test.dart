import 'dart:convert';
import 'dart:io';
import 'dart:ui' as ui;

import 'package:crabforge/appearance.dart';
import 'package:crabforge/l10n/app_localizations.dart';
import 'package:crabforge/language.dart';
import 'package:crabforge/main.dart';
import 'package:crabforge/src/rust/api.dart' as native;
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

import 'desktop_widgets.dart';

void main() {
  final binding = IntegrationTestWidgetsFlutterBinding.ensureInitialized();
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
    final restart = Platform.environment['CRABFORGE_RESTART_CHECK'] == '1';
    expect(settings.existsSync(), restart);
    if (!restart) {
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
                if (id == 'openai')
                  'model_reasoning_levels': {
                    model: ['low', 'high'],
                  },
              },
          ],
          'groups': {
            'review': ['openai/gpt-4.1', 'anthropic/claude-sonnet-4'],
          },
        }),
      );
    }
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
    final fontMenu = find.byKey(const ValueKey('font-picker'));
    ThemeData activeTheme() => Theme.of(tester.element(fontMenu));
    Future<void> expectFont(WorkspaceFont font) async {
      final deadline = DateTime.now().add(const Duration(seconds: 10));
      while (activeTheme().textTheme.bodyMedium!.fontFamily != font.family &&
          DateTime.now().isBefore(deadline)) {
        await tester.pump(const Duration(milliseconds: 50));
      }
      expect(activeTheme().textTheme.bodyMedium!.fontFamily, font.family);
      expect(await native.uiFont(), font.preference);
      await tester.pumpAndSettle();
    }

    if (restart) {
      await expectFont(WorkspaceFont.harmonyOs);
      return;
    }

    final strings = lookupAppLocalizations(const Locale('en'));
    List<String> labels(AppLocalizations strings) => [
      strings.providers,
      strings.routes,
      strings.groups,
      strings.gateway,
      strings.models,
      strings.clients,
    ];
    final pages = labels(strings);
    Finder navigation(int width) =>
        find.byType(width >= 820 ? NavigationRail : NavigationBar);
    Future<void> openPage(Finder nav, String label) async {
      await tester.tap(
        find.descendant(of: nav, matching: find.text(label)).first,
      );
      await tester.pumpAndSettle(const Duration(milliseconds: 16));
    }

    if (Platform.environment['CRABFORGE_LAYOUT_PROFILE']
        case final String file) {
      expect(kProfileMode, isTrue);
      binding.framePolicy = LiveTestWidgetsFlutterBindingFramePolicy.fullyLive;
      final nav = navigation(1200);
      for (final label in pages) {
        await openPage(nav, label);
      }
      final timings = <ui.FrameTiming>[];
      final collect = timings.addAll;
      binding.addTimingsCallback(collect);
      try {
        await binding.watchPerformance(() async {
          timings.clear();
          for (var round = 0; round < 4; round++) {
            for (final label in pages) {
              await openPage(nav, label);
            }
          }
        });
      } finally {
        binding.removeTimingsCallback(collect);
      }
      await File(file).writeAsString(
        const JsonEncoder.withIndent('  ').convert({
          ...?binding.reportData,
          'frames': [
            for (final frame in timings)
              {
                'build_us': frame.buildDuration.inMicroseconds,
                'raster_us': frame.rasterDuration.inMicroseconds,
                'span_us': frame.totalSpan.inMicroseconds,
              },
          ],
        }),
      );
      return;
    }

    addTearDown(
      tester.binding.platformDispatcher.clearPlatformBrightnessTestValue,
    );
    for (final brightness in Brightness.values) {
      tester.binding.platformDispatcher.platformBrightnessTestValue =
          brightness;
      for (final width in [1200, 560]) {
        tester.view.physicalSize = Size(width.toDouble(), 800);
        await tester.pumpAndSettle();
        expect(activeTheme().brightness, brightness);
        final nav = navigation(width);
        expect(nav, findsOneWidget);
        for (final label in pages) {
          await openPage(nav, label);
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
                  '$output/${brightness.name}/$width-${pages.indexOf(label)}-${grid ? 'grid' : 'list'}.png',
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
    }

    final saveRouteLimits = find.widgetWithText(FilledButton, strings.save);
    final timeoutField = find.widgetWithText(
      TextFormField,
      strings.firstContentSeconds,
    );
    final patienceField = find.widgetWithText(
      TextFormField,
      strings.groupPatienceSeconds,
    );
    Future<void> fillField(Finder field, String text) async {
      await tester.ensureVisible(field);
      await tester.pump();
      await tester.enterText(field, text);
      await tester.pump();
      expect(tester.state<FormFieldState<String>>(field).value, text);
    }

    Future<void> saveAndRead(bool Function(Map) persisted) async {
      FocusManager.instance.primaryFocus?.unfocus();
      await tester.pump();
      await tester.tap(saveRouteLimits);
      final deadline = DateTime.now().add(const Duration(seconds: 10));
      var saved = false;
      while (!saved && DateTime.now().isBefore(deadline)) {
        await tester.pump(const Duration(milliseconds: 50));
        saved = persisted(jsonDecode(settings.readAsStringSync()) as Map);
      }
      expect(
        saved,
        isTrue,
        reason: tester
            .widgetList<Text>(
              find.descendant(
                of: find.byType(SnackBar),
                matching: find.byType(Text),
              ),
            )
            .map((text) => text.data)
            .join('\n'),
      );
      await tester.pumpAndSettle(const Duration(milliseconds: 16));
    }

    Future<void> saveLimits(
      int firstContent,
      int patience,
      bool Function(Map) persisted, {
      String? routingRules,
    }) async {
      await fillField(timeoutField, '$firstContent');
      await fillField(patienceField, '$patience');
      if (routingRules != null) {
        final field = find.widgetWithText(
          TextFormField,
          strings.groupRoutingRules,
        );
        await fillField(field, routingRules);
      }
      await saveAndRead(persisted);
    }

    for (final width in [1200, 560]) {
      tester.view.physicalSize = Size(width.toDouble(), 800);
      await tester.pumpAndSettle();
      final nav = navigation(width);
      await openPage(nav, strings.providers);
      await tester.tap(find.text('openai'));
      await tester.pumpAndSettle();
      final providerPatience = width == 1200 ? 7 : 0;
      await fillField(patienceField, '$providerPatience');
      await saveAndRead(
        (cfg) => (cfg['providers'] as List).any(
          (item) =>
              item['id'] == 'openai' &&
              item['patience'] == providerPatience &&
              listEquals(item['model_reasoning_levels']?['gpt-4.1'] as List?, [
                'low',
                'high',
              ]),
        ),
      );
      await openPage(nav, strings.groups);
      await tester.tap(find.text('group/review'));
      await tester.pumpAndSettle();
      final seconds = width == 1200 ? 2 : -1;
      final patience = width == 1200 ? 10 : -1;
      final tokens = width == 1200 ? 100 : 200;
      final intent = 'Review $width';
      final effort = width == 1200 ? 'medium' : 'high';
      final confidence = width == 1200 ? 0.6 : 0;
      await saveLimits(
        seconds,
        patience,
        (cfg) =>
            cfg['group_first_content']?['review'] == seconds &&
            cfg['group_patience']?['review'] == patience &&
            cfg['group_rules']?['review']?[0]['tokens'] == tokens &&
            cfg['group_rules']?['review']?[0]['intent'] == intent &&
            cfg['group_rules']?['review']?[0]['useEffort'] == effort &&
            cfg['group_classifiers']?['review']?['effort'] == 'auto' &&
            cfg['group_classifiers']?['review']?['intentSure'] == confidence &&
            cfg['group_classifiers']?['review']?['classifier'] ==
                'openai/gpt-4.1',
        routingRules:
            '{"classifier":"openai/gpt-4.1","effort":"auto","intentSure":$confidence,"rules":[{"use":"openai/gpt-4.1","tokens":$tokens,"intent":"$intent","useEffort":"$effort"}]}',
      );
      await openPage(nav, strings.gateway);
      await tester.tap(find.text(strings.networkSettings));
      await tester.pumpAndSettle();
      final defaultSeconds = width == 1200 ? 3 : 4;
      final defaultPatience = width == 1200 ? 12 : 0;
      final classifySeconds = width == 1200 ? 8 : 60;
      final classifyField = find.widgetWithText(
        TextFormField,
        strings.classifyTimeoutSeconds,
      );
      await fillField(classifyField, '$classifySeconds');
      await saveLimits(
        defaultSeconds,
        defaultPatience,
        (cfg) =>
            cfg['default_group_first_content'] == defaultSeconds &&
            cfg['default_group_patience'] == defaultPatience &&
            cfg['classify_timeout'] == classifySeconds,
      );
      expect(tester.takeException(), isNull);
    }

    final narrowNav = navigation(560);
    for (final code in languageNames.keys) {
      final locale = Locale(code);
      final translated = lookupAppLocalizations(locale);
      tester.binding.platformDispatcher.localesTestValue = [locale];
      await tester.pumpAndSettle();
      for (final label in labels(translated)) {
        await openPage(narrowNav, label);
        expect(tester.takeException(), isNull, reason: '$code / $label');
      }
    }
    tester.binding.platformDispatcher.localesTestValue = [const Locale('en')];
    await tester.pumpAndSettle();

    final accent = decodeSystemAccent(
      await ServicesBinding.instance.defaultBinaryMessenger.send(
        systemAccentChannel,
        null,
      ),
    );
    expect(accent, isNotNull);
    final dispatcher = tester.binding.platformDispatcher;
    addTearDown(dispatcher.clearAccessibilityFeaturesTestValue);
    dispatcher.accessibilityFeaturesTestValue = const FakeAccessibilityFeatures(
      highContrast: true,
    );
    for (final brightness in Brightness.values) {
      dispatcher.platformBrightnessTestValue = brightness;
      await tester.pumpAndSettle();
      expect(
        activeTheme().colorScheme,
        workspaceTheme(
          brightness,
          WorkspaceFont.googleSans,
          accent: accent,
          highContrast: true,
        ).colorScheme,
      );
    }
    dispatcher.clearAccessibilityFeaturesTestValue();
    await tester.pumpAndSettle();
    for (final font in [
      WorkspaceFont.harmonyOs,
      WorkspaceFont.googleSans,
      WorkspaceFont.harmonyOs,
    ]) {
      await tester.tap(fontMenu);
      await tester.pumpAndSettle();
      await tester.tap(find.text(font.label));
      await expectFont(font);
    }

    final project = Directory('${home.path}/layout-project')..createSync();
    final instructions = File('${project.path}/CLAUDE.md');
    instructions.writeAsStringSync('# Existing instructions\n');
    await tester.tap(find.text(strings.projectInstructions));
    await tester.pumpAndSettle();
    final editor = find.byType(AlertDialog);
    final fields = find.descendant(
      of: editor,
      matching: find.byType(TextField),
    );
    Future<void> waitForEdit(String text) async {
      await tester.pump();
      final ready = find.byWidgetPredicate(
        (widget) =>
            widget is TextField &&
            widget.decoration?.labelText == 'Markdown' &&
            widget.enabled == true &&
            widget.controller?.text == text,
      );
      final deadline = DateTime.now().add(const Duration(seconds: 10));
      while (ready.evaluate().isEmpty && DateTime.now().isBefore(deadline)) {
        await tester.pump(const Duration(milliseconds: 50));
      }
      expect(ready, findsOneWidget);
    }

    expect(fields, findsOneWidget);
    await tester.enterText(fields.first, project.path);
    await tester.tap(find.text(strings.openProject));
    await waitForEdit('');
    expect(fields, findsNWidgets(2));
    final save = find.widgetWithText(FilledButton, strings.save);

    instructions.writeAsStringSync('# Updated elsewhere\n');
    await tester.enterText(fields.last, 'Stale edit.');
    await tester.pump();
    expect(tester.widget<FilledButton>(save).onPressed, isNotNull);
    await tester.tap(save);
    await waitForEdit('Stale edit.');
    final messages = tester.widgetList<SelectableText>(
      find.descendant(of: editor, matching: find.byType(SelectableText)),
    );
    expect(
      messages.map((widget) => widget.data),
      contains(contains('instructions changed')),
    );
    expect(instructions.readAsStringSync(), '# Updated elsewhere\n');
    await tester.tap(find.widgetWithText(TextButton, strings.reload));
    await waitForEdit('');

    await tester.tap(fields.last);
    await tester.enterText(fields.last, 'Run the workspace tests.');
    await tester.pump();
    expect(tester.widget<FilledButton>(save).onPressed, isNotNull);
    await tester.tap(save);
    await waitForEdit('Run the workspace tests.');
    expect(
      instructions.readAsStringSync(),
      contains('Run the workspace tests.'),
    );
    await tester.tap(find.text(strings.restoreInstructions));
    await tester.pumpAndSettle();
    await tester.tap(
      find.widgetWithText(FilledButton, strings.restoreInstructions),
    );
    await waitForEdit('');
    await tester.pumpAndSettle();
    expect(instructions.readAsStringSync(), '# Updated elsewhere\n');

    await tester.tap(find.widgetWithText(TextButton, 'Close'));
    await tester.pumpAndSettle();
    final geminiSettings = File('${project.path}/.gemini/settings.json');
    geminiSettings.parent.createSync();
    geminiSettings.writeAsStringSync('{"context":{"fileName":["CUSTOM.md"]}}');
    final customInstructions = File('${project.path}/CUSTOM.md');
    customInstructions.writeAsStringSync('# Before\r\n');
    final agentSelector = find.byType(DropdownMenu<String>);
    tester.widget<DropdownMenu<String>>(agentSelector).onSelected!('gemini');
    await tester.pumpAndSettle();
    await tester.tap(find.text(strings.projectInstructions));
    await tester.pumpAndSettle();
    await tester.enterText(fields.first, project.path);
    await tester.tap(find.text(strings.openProject));
    await waitForEdit('');
    await tester.enterText(fields.last, 'Use the custom instructions file.');
    await tester.pump();
    await tester.tap(save);
    await waitForEdit('Use the custom instructions file.');
    expect(customInstructions.readAsStringSync(), contains('# Before\r\n'));
    expect(
      customInstructions.readAsStringSync(),
      contains('Use the custom instructions file.\r\n'),
    );
    expect(File('${project.path}/GEMINI.md').existsSync(), isFalse);
    await tester.tap(find.widgetWithText(TextButton, 'Close'));
    await tester.pumpAndSettle();

    final openMemory = find.widgetWithText(TextButton, strings.memoryLibrary);
    await tester.ensureVisible(openMemory);
    await tester.tap(openMemory);
    await tester.pumpAndSettle();
    await tester.tap(find.widgetWithText(FilledButton, strings.addMemory));
    await tester.pumpAndSettle();
    const memory = 'Use one isolated configuration for each run.';
    await tester.enterText(
      find.byWidgetPredicate(
        (widget) =>
            widget is TextField && widget.decoration?.labelText == 'Markdown',
      ),
      memory,
    );
    final key = await tester.runAsync(
      () => native.memoryProjectKey(project: project.path),
    );
    expect(key, isNotNull);
    await tester.enterText(
      find.widgetWithText(TextField, strings.memoryScope),
      'project:$key',
    );
    await tester.ensureVisible(save);
    await tester.tap(save);
    final memoryTile = find.widgetWithText(ListTile, memory);
    final memoryDeadline = DateTime.now().add(const Duration(seconds: 10));
    while (memoryTile.evaluate().isEmpty &&
        DateTime.now().isBefore(memoryDeadline)) {
      await tester.pump(const Duration(milliseconds: 50));
    }
    expect(memoryTile, findsOneWidget);
    expect(Directory('${home.path}/memory').listSync().single, isA<File>());
    final memoryDialog = find.byType(Dialog);
    final wasGrid = find
        .descendant(of: memoryDialog, matching: find.byType(GridView))
        .evaluate()
        .isNotEmpty;
    await tester.pumpAndSettle();
    await tester.tap(
      find.descendant(
        of: memoryDialog,
        matching: find.byTooltip(wasGrid ? strings.listView : strings.gridView),
      ),
    );
    await tester.pumpAndSettle();
    expect(
      find.descendant(
        of: memoryDialog,
        matching: wasGrid ? find.byType(ListView) : find.byType(GridView),
      ),
      findsOneWidget,
    );
    await tester.tap(find.widgetWithText(TextButton, 'Close'));
    await tester.pumpAndSettle();
    await tester.tap(find.text(strings.projectInstructions));
    await tester.pumpAndSettle();
    await tester.enterText(fields.first, project.path);
    await tester.tap(find.text(strings.openProject));
    await waitForEdit('Use the custom instructions file.');
    await tester.tap(find.text(strings.projectMemory));
    final publish = find.widgetWithText(FilledButton, strings.publishMemory);
    final publicationDeadline = DateTime.now().add(const Duration(seconds: 10));
    while (publish.evaluate().isEmpty &&
        DateTime.now().isBefore(publicationDeadline)) {
      await tester.pump(const Duration(milliseconds: 50));
    }
    expect(find.textContaining('- $memory'), findsOneWidget);
    await tester.pumpAndSettle();
    await tester.tap(publish);
    await waitForEdit('Use the custom instructions file.');
    expect(customInstructions.readAsStringSync(), contains('- $memory\r\n'));
  });
}
