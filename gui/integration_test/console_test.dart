import 'dart:convert';
import 'dart:io';

import 'package:crabforge/main.dart' as app;
import 'package:crabforge/language.dart';
import 'package:crabforge/l10n/app_localizations.dart';
import 'package:crabforge/material_symbols.dart';
import 'package:crabforge/src/rust/api.dart' as native;
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

import 'desktop_widgets.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();
  testDesktopWidgets('桌面配置、网关请求和路由记录通过真实 Rust 绑定工作', (tester) async {
    final path = Platform.environment['CRABFORGE_HOME'];
    if (path == null ||
        !Directory(path).uri.pathSegments
            .any((s) => s.startsWith('crabforge-gui-test-'))) {
      throw StateError(
        'Set CRABFORGE_HOME to a new temporary crabforge-gui-test-* directory',
      );
    }
    final settings = File('$path/settings.json');
    final picker = find.byKey(const ValueKey('language-picker'));
    final dispatcher = tester.binding.platformDispatcher;
    dispatcher.localesTestValue = [const Locale('it')];
    addTearDown(dispatcher.clearLocalesTestValue);
    tester.view.devicePixelRatio = 1;
    tester.view.physicalSize = const Size(1200, 800);
    addTearDown(tester.view.resetDevicePixelRatio);
    addTearDown(tester.view.resetPhysicalSize);
    Future<void> waitFor(Finder finder) async {
      final until = DateTime.now().add(const Duration(seconds: 25));
      while (finder.evaluate().isEmpty && DateTime.now().isBefore(until)) {
        await tester.pump(const Duration(milliseconds: 100));
      }
      expect(
        finder,
        findsWidgets,
        reason: 'Window state ${tester.binding.lifecycleState}',
      );
    }

    if (Platform.environment['CRABFORGE_RESTART_CHECK'] == '1') {
      app.main();
      await waitFor(find.text('renamed-local'));
      final zh = lookupAppLocalizations(const Locale('zh'));
      await waitFor(find.text(zh.addProvider));
      expect(await native.uiLayout(), 'grid');
      await waitFor(find.byType(SliverGrid));
      await tester.tap(find.byTooltip(zh.listView));
      await waitFor(find.byTooltip(zh.gridView));
      final persisted = jsonDecode(await native.configuration());
      expect(persisted['groups']['renamed-2'], ['renamed-local/desktop-model']);
      expect(persisted['group_weights']['renamed-2'], {
        'renamed-local/desktop-model': 3,
      });
      expect(persisted['group_match']['renamed-copy'], [
        'free:renamed-local/*',
      ]);
      expect(persisted['former_providers']['local'], 'renamed-local');
      expect(persisted['providers'][0]['no_stand_in'], true);
      expect(persisted['providers'][0]['fallback'], [
        'renamed-local/account-model',
        'group/renamed',
      ]);
      expect(persisted['providers'][0]['model_protocols']['account-model'], {
        'native_endpoints': ['/v1/messages'],
        'preferred_endpoint': '/v1/messages',
      });
      expect(Localizations.localeOf(tester.element(picker)).languageCode, 'zh');
      await tester.tap(find.text(zh.groups).last);
      await tester.pumpAndSettle();
      await waitFor(find.text(zh.autoGroups));
      await waitFor(find.byTooltip(zh.enableGroup));
      await tester.scrollUntilVisible(
        find.text('group/parent'),
        120,
        scrollable: find
            .descendant(
              of: find.byKey(const PageStorageKey(2)),
              matching: find.byType(Scrollable),
            )
            .first,
      );
      await waitFor(
        find.byWidgetPredicate(
          (w) => w is DropdownButton<String> && w.value == 'group/renamed',
        ),
      );
      await tester.tap(find.text(zh.providers).last);
      await tester.tap(picker);
      await tester.pumpAndSettle();
      await tester.tap(
        find.widgetWithText(PopupMenuItem<String>, zh.systemLanguage),
      );
      await tester.pumpAndSettle();
      await waitFor(find.text('Add provider'));
      expect(await native.uiLanguage(), '');
      dispatcher.localesTestValue = [
        const Locale('it'),
        const Locale('ja', 'JP'),
      ];
      await waitFor(
        find.text(lookupAppLocalizations(const Locale('ja')).addProvider),
      );
      return;
    }
    expect(settings.existsSync(), false);
    Directory(path).createSync(recursive: true);
    final upstream = await HttpServer.bind(InternetAddress.loopbackIPv4, 0);
    addTearDown(() => upstream.close(force: true));
    upstream.listen((req) async {
      if (req.method == 'GET' && req.uri.path == '/account/catalog') {
        req.response.headers.contentType = ContentType.json;
        req.response.write(
          '{"data":[{"id":"account-model","free":true,"native_endpoints":["/v1/messages"],"preferred_endpoint":"/v1/messages"}]}',
        );
        await req.response.close();
        return;
      }
      if (req.method != 'POST' || req.uri.path != '/v1/chat/completions') {
        req.response.statusCode = HttpStatus.notFound;
        await req.response.close();
        return;
      }
      final body = jsonDecode(await utf8.decoder.bind(req).join());
      if (body['model'] != 'desktop-model') {
        req.response.statusCode = HttpStatus.badRequest;
        await req.response.close();
        return;
      }
      req.response.headers.contentType = ContentType.json;
      req.response.write(
        jsonEncode({
          'choices': [
            {
              'message': {'role': 'assistant', 'content': 'desktop reply'},
            },
          ],
        }),
      );
      await req.response.close();
    });
    settings.writeAsStringSync(
      jsonEncode({
        'listen': '127.0.0.1:0',
        'groups': {
          'desktop': ['local/desktop-model'],
          'parent': ['local/desktop-model', 'group/desktop'],
        },
        'providers': [
          {
            'id': 'local',
            'endpoints': {
              'chat': 'http://127.0.0.1:${upstream.port}/v1',
              'anthropic': 'http://127.0.0.1:${upstream.port}',
            },
            'models': ['desktop-model'],
          },
          {
            'id': 'vertex-fixture',
            'endpoints': {
              'vertex': {'project': 'desktop-fixture', 'location': 'us'},
            },
            'models': ['gemini-3.8-flash', 'desktop-model'],
          },
        ],
      }),
    );
    Future<void> tapText(String text) async {
      final matches = find.text(text);
      await waitFor(matches);
      final finder = matches.last;
      await tester.ensureVisible(finder);
      await tester.tap(finder);
      await tester.pumpAndSettle(
        const Duration(milliseconds: 100),
        EnginePhase.sendSemanticsUpdate,
        const Duration(seconds: 10),
      );
    }

    app.main();
    await waitFor(find.text('local'));
    await waitFor(find.text('Add provider'));
    for (final code in [
      ...languageNames.keys.where((code) => code != 'zh'),
      'zh',
    ]) {
      await tester.tap(picker);
      await tester.pumpAndSettle();
      final choices = find.byType(PopupMenuItem<String>);
      expect(
        tester
            .widgetList<PopupMenuItem<String>>(choices)
            .map((item) => item.value),
        ['', 'zh', 'en', 'fr', 'de', 'hi', 'ja', 'ko', 'pt', 'es'],
      );
      expect(
        find.descendant(
          of: choices,
          matching: find.byIcon(CrabforgeSymbols.check),
        ),
        findsOneWidget,
      );
      final item = find.widgetWithText(
        PopupMenuItem<String>,
        languageNames[code]!,
      );
      await tester.ensureVisible(item);
      await tester.tap(item);
      await tester.pumpAndSettle();
      final strings = lookupAppLocalizations(Locale(code));
      await waitFor(find.text(strings.addProvider));
      expect(await native.uiLanguage(), code);
      expect(Localizations.localeOf(tester.element(picker)).languageCode, code);
      await tapText('local');
      await waitFor(find.text(strings.editProvider));
      expect(find.text(strings.providerId), findsWidgets);
      await tapText(strings.cancel);
    }
    await tapText('local');
    final catalogUrl = 'http://127.0.0.1:${upstream.port}/account/catalog';
    final authMenu = find.byType(DropdownMenu<String>);
    await tester.ensureVisible(authMenu);
    await tester.tap(authMenu);
    await tester.pumpAndSettle();
    await tapText('Authorization: Bearer');
    final catalogField = find.widgetWithText(TextFormField, '模型目录完整地址（可选）');
    await tester.ensureVisible(catalogField);
    await tester.enterText(catalogField, catalogUrl);
    final sourcesField = find.widgetWithText(
      TextFormField,
      'models.dev 目录 ID（可选）',
    );
    await tester.ensureVisible(sourcesField);
    await tester.enterText(sourcesField, 'kimi-code-plan-cn, vercel');
    final fallbackField = find.widgetWithText(
      TextFormField,
      lookupAppLocalizations(const Locale('zh')).providerFallback,
    );
    await tester.ensureVisible(fallbackField);
    await tester.enterText(fallbackField, 'local/account-model\ngroup/desktop');
    await tapText(
      lookupAppLocalizations(const Locale('zh')).allowProviderFallback,
    );
    await tapText('读取模型目录');
    await waitFor(find.textContaining('已读取 1 个模型'));
    await tapText('保存');
    await waitFor(find.text('desktop-model · account-model'));
    final saved = jsonDecode(settings.readAsStringSync())['providers'][0];
    expect(saved['endpoints']['models_url'], catalogUrl);
    expect(saved['endpoints']['auth_header'], 'bearer');
    expect(saved['catalog'], 'kimi-code-plan-cn, vercel');
    expect(saved['models'], ['desktop-model', 'account-model']);
    expect(saved['fallback'], ['local/account-model', 'group/desktop']);
    expect(saved['no_stand_in'], true);
    expect(saved['free_models'], ['account-model']);
    expect(saved['model_protocols']['account-model'], {
      'native_endpoints': ['/v1/messages'],
      'preferred_endpoint': '/v1/messages',
    });
    await tapText('DeepSeek 网页');
    await waitFor(find.text('deepseek-web'));
    expect(
      jsonDecode(settings.readAsStringSync())['providers'].last['web_site'],
      'deepseek',
    );
    final zh = lookupAppLocalizations(const Locale('zh'));
    await tapText('vertex-fixture');
    final region = find.widgetWithText(TextFormField, zh.vertexLocation);
    await tester.ensureVisible(region);
    await tester.enterText(region, 'us-central1');
    await tapText(zh.readModels);
    await waitFor(find.text(zh.modelsRead(3)));
    await tapText(zh.save);
    await waitFor(find.text('vertex-fixture'));
    final vertex = jsonDecode(settings.readAsStringSync())['providers'][1];
    expect(vertex['endpoints']['vertex']['location'], 'us-central1');
    expect(vertex['models'], contains('gemini-2.5-flash-lite'));
    await tapText(zh.models);
    await waitFor(find.text('local/account-model'));
    final search = find.byType(TextField);
    await tester.enterText(search, 'AUTO/DESKTOP');
    await waitFor(find.text('auto/desktop-model'));
    await tester.tap(find.byTooltip(zh.gridView));
    await waitFor(find.byKey(const PageStorageKey('model-grid')));
    await waitFor(find.text('auto/desktop-model'));
    await tester.tap(search);
    await tester.pumpAndSettle();
    await tester.enterText(search, 'GROUP/PARENT');
    await waitFor(find.text('group/parent'));
    expect(find.text('group/desktop'), findsNothing);
    await tester.tap(find.byTooltip(zh.listView));
    await waitFor(find.byKey(const PageStorageKey('model-list')));
    await tester.tap(search);
    await tester.pumpAndSettle();
    await tester.enterText(search, 'missing-model');
    await waitFor(find.text(zh.noModelsFound));
    await tapText(zh.clients);
    await waitFor(find.text(zh.agentRoot));
    await tester.tap(find.byTooltip(zh.gridView));
    await waitFor(find.byType(SliverGrid));
    await tapText(zh.gateway);
    await waitFor(find.text(zh.listenAddress));
    await waitFor(find.byType(SliverGrid));
    await tapText(zh.groups);
    await waitFor(find.byTooltip(zh.listView));
    await tester.tap(find.byTooltip(zh.listView));
    await waitFor(find.byTooltip(zh.gridView));
    final foundGroup = find.text('auto/desktop-model');
    await waitFor(foundGroup);
    await tapText(zh.autoGroups);
    await waitFor(
      find.byWidgetPredicate((w) => w is SwitchListTile && !w.value),
    );
    expect(foundGroup, findsNothing);
    expect(jsonDecode(settings.readAsStringSync())['no_auto_groups'], true);
    await tapText(zh.autoGroups);
    await waitFor(foundGroup);
    expect(
      jsonDecode(settings.readAsStringSync())['groups'],
      isNot(contains('auto-desktop-model')),
    );
    final desktopGroup = find.byWidgetPredicate(
      (w) =>
          w is ListTile &&
          w.title is Text &&
          (w.title as Text).data == 'group/desktop',
    );
    final up = find.descendant(
      of: desktopGroup,
      matching: find.byIcon(CrabforgeSymbols.arrowUpward),
    );
    await tester.ensureVisible(up);
    await tester.tap(up);
    await waitFor(
      find.ancestor(
        of: up,
        matching: find.byWidgetPredicate(
          (w) => w is IconButton && w.onPressed == null,
        ),
      ),
    );
    expect(
      jsonDecode(settings.readAsStringSync())['group_display_order'].first,
      'desktop',
    );
    await tester.ensureVisible(desktopGroup);
    await tester.tap(desktopGroup);
    await tester.pumpAndSettle();
    await tapText(zh.listedOrder);
    await tapText(zh.weightedOrder);
    final weight = find.widgetWithText(TextFormField, zh.memberWeight);
    await tester.enterText(weight, '3');
    final editor = find.byType(AlertDialog);
    for (final tooltip in [zh.gridView, zh.listView]) {
      final toggle = find.descendant(
        of: editor,
        matching: find.byTooltip(tooltip),
      );
      await tester.tap(toggle);
      await waitFor(
        find.descendant(
          of: editor,
          matching: find.byType(tooltip == zh.gridView ? GridView : ListView),
        ),
      );
      expect(tester.widget<TextFormField>(weight).controller!.text, '3');
    }
    final noteField = find.widgetWithText(TextFormField, zh.groupNote);
    await tester.ensureVisible(noteField);
    await tester.enterText(noteField, '  备用   模型  ');
    await tapText(zh.listGroupModels);
    await tester.enterText(
      find.widgetWithText(TextFormField, zh.groupId),
      'renamed',
    );
    await tapText(zh.save);
    await waitFor(find.text('group/renamed'));
    expect(
      jsonDecode(settings.readAsStringSync())['group_display_order'].first,
      'renamed',
    );
    expect(
      jsonDecode(settings.readAsStringSync())['group_weights']['renamed'],
      {'local/desktop-model': 3},
    );
    final renamedCard = find.widgetWithText(ListTile, 'group/renamed');
    final copyGroup = find.descendant(
      of: renamedCard,
      matching: find.byTooltip(zh.copyGroup),
    );
    await tester.ensureVisible(copyGroup);
    await tester.tap(copyGroup);
    await waitFor(find.text('renamed copy · group/renamed-copy'));
    final copied = jsonDecode(settings.readAsStringSync());
    expect(copied['group_unlists'], containsAll(['renamed', 'renamed-copy']));
    expect(copied['group_notes']['renamed-copy'], '备用 模型');
    expect(copied['group_weights']['renamed-copy'], {'local/desktop-model': 3});
    expect(copied['group_display_order'].take(2), ['renamed', 'renamed-copy']);
    final shareButton = find.descendant(
      of: renamedCard,
      matching: find.byTooltip(zh.shareGroup),
    );
    await tester.ensureVisible(shareButton);
    await tester.tap(shareButton);
    await waitFor(
      find.byWidgetPredicate(
        (w) =>
            w is Text &&
            (w.data == zh.groupCopied || w.data == zh.sharedGroupText),
      ),
    );
    final copiedToSystem = find.text(zh.groupCopied).evaluate().isNotEmpty;
    final String shareText;
    if (copiedToSystem) {
      shareText = (await Clipboard.getData(Clipboard.kTextPlain))!.text!;
    } else {
      final dialog = find.byType(AlertDialog);
      shareText = tester
          .widget<SelectableText>(
            find.descendant(of: dialog, matching: find.byType(SelectableText)),
          )
          .data!;
      await tapText(
        MaterialLocalizations.of(tester.element(dialog)).closeButtonLabel,
      );
    }
    debugPrint(
      copiedToSystem
          ? 'Group share read from system clipboard'
          : 'Group share read from selectable text after clipboard error',
    );
    expect(jsonDecode(shareText)['group']['routing'], 'weight');
    await tapText(zh.importGroup);
    final importText = find.widgetWithText(TextField, zh.sharedGroupText);
    await tester.tap(importText);
    await tester.pumpAndSettle();
    final editable = find.descendant(
      of: importText,
      matching: find.byType(EditableText),
    );
    if (copiedToSystem) {
      await tester
          .state<EditableTextState>(editable)
          .pasteText(SelectionChangedCause.keyboard);
    } else {
      await tester.enterText(importText, shareText);
    }
    await tester.pumpAndSettle();
    expect(tester.widget<EditableText>(editable).controller.text, shareText);
    expect(
      tester
          .widget<FilledButton>(find.widgetWithText(FilledButton, zh.save))
          .onPressed,
      isNotNull,
    );
    await tapText(zh.save);
    final importedCard = find.widgetWithText(
      ListTile,
      'renamed 2 · group/renamed-2',
    );
    await tester.scrollUntilVisible(
      importedCard,
      120,
      scrollable: find
          .descendant(
            of: find.byKey(const PageStorageKey(2)),
            matching: find.byType(Scrollable),
          )
          .first,
    );
    await waitFor(importedCard);
    final imported = jsonDecode(settings.readAsStringSync());
    expect(imported['groups']['renamed-2'], ['local/desktop-model']);
    expect(imported['group_weights']['renamed-2'], {'local/desktop-model': 3});
    final copyCard = find.widgetWithText(
      ListTile,
      'renamed copy · group/renamed-copy',
    );
    await tester.ensureVisible(copyCard);
    await tester.tap(copyCard);
    await tester.pumpAndSettle();
    final patternField = find.widgetWithText(TextField, zh.memberPatterns);
    await tester.ensureVisible(patternField);
    await tester.enterText(patternField, 'free:local/*');
    await waitFor(
      find.descendant(
        of: find.byType(AlertDialog),
        matching: find.byKey(const ValueKey('local/account-model')),
      ),
    );
    await tapText(zh.save);
    await waitFor(
      find.descendant(
        of: copyCard,
        matching: find.text('local/desktop-model → local/account-model'),
      ),
    );
    final patternConfig = jsonDecode(settings.readAsStringSync());
    expect(patternConfig['groups']['renamed-copy'], ['local/desktop-model']);
    expect(patternConfig['group_match']['renamed-copy'], ['free:local/*']);
    final disableCopy = find.descendant(
      of: copyCard,
      matching: find.byTooltip(zh.disableGroup),
    );
    await tester.ensureVisible(disableCopy);
    await tester.tap(disableCopy);
    await waitFor(find.byTooltip(zh.enableGroup));
    expect(jsonDecode(settings.readAsStringSync())['disabled_groups'], [
      'renamed-copy',
    ]);
    final parentCard = find.widgetWithText(ListTile, 'group/parent');
    await tester.ensureVisible(parentCard);
    await tester.tap(parentCard);
    await tester.pumpAndSettle();
    final disable = find.descendant(
      of: find.byKey(const ValueKey('group/renamed')),
      matching: find.byTooltip(zh.disableMember),
    );
    await tester.ensureVisible(disable);
    await tester.tap(disable);
    await tapText(zh.save);
    await waitFor(
      find.descendant(
        of: parentCard,
        matching: find.text('local/desktop-model'),
      ),
    );
    expect(jsonDecode(settings.readAsStringSync())['group_off']['parent'], [
      'group/renamed',
    ]);
    await tester.tap(parentCard);
    await tester.pumpAndSettle();
    await tapText(zh.listedOrder);
    await tapText(zh.manualRouting);
    await tapText(zh.save);
    await waitFor(find.byType(DropdownButton<String>));
    expect(
      jsonDecode(settings.readAsStringSync())['group_picks']['parent'],
      'local/desktop-model',
    );
    await tapText('网关');
    await tapText(zh.networkSettings);
    final originsField = find.widgetWithText(TextFormField, zh.browserOrigins);
    await tester.ensureVisible(originsField);
    await tester.enterText(originsField, 'https://client.example:443/');
    await tapText(zh.save);
    await waitFor(find.text(zh.start));
    expect(jsonDecode(settings.readAsStringSync())['cors_origins'], [
      'https://client.example',
    ]);
    await tapText('启动网关');
    await waitFor(find.text('停止网关'));
    await tapText(zh.groups);
    final selector = find.descendant(
      of: parentCard,
      matching: find.byType(DropdownButton<String>),
    );
    await tester.ensureVisible(selector);
    await tester.tap(selector);
    await tester.pumpAndSettle();
    await tapText('group/renamed');
    await waitFor(
      find.byWidgetPredicate(
        (w) => w is DropdownButton<String> && w.value == 'group/renamed',
      ),
    );
    expect(
      jsonDecode(settings.readAsStringSync())['group_picks']['parent'],
      'group/renamed',
    );
    final status = jsonDecode(await native.status());
    final cli = Platform.environment['CRABFORGE_TEST_CLI'];
    if (cli == null) {
      throw StateError(
        'Set CRABFORGE_TEST_CLI to the built carbforge-cli executable',
      );
    }
    final current = await native.configuration();
    final external = await Process.run(cli, [
      '--data-dir',
      path,
      'group',
      'pick',
      'parent',
      'local/desktop-model',
    ]);
    expect(external.exitCode, 0, reason: '${external.stderr}');
    await expectLater(
      native.saveConfiguration(previous: current, text: current),
      throwsA(anything),
    );
    await waitFor(
      find.byWidgetPredicate(
        (w) => w is DropdownButton<String> && w.value == 'local/desktop-model',
      ),
    );
    await tester.tap(selector);
    await tester.pumpAndSettle();
    await tapText('group/renamed');
    await waitFor(
      find.byWidgetPredicate(
        (w) => w is DropdownButton<String> && w.value == 'group/renamed',
      ),
    );
    final http = HttpClient();
    addTearDown(() => http.close(force: true));
    final catalog = await http.getUrl(
      Uri.parse('http://${status['address']}/v1/models'),
    );
    catalog.headers.set('Authorization', 'Bearer ${await native.gatewayKey()}');
    final catalogReply = await catalog.close();
    expect(catalogReply.statusCode, 200);
    final listed =
        jsonDecode(await utf8.decoder.bind(catalogReply).join())['data']
            as List;
    expect(
      listed.map((row) => row['id']),
      isNot(contains('local/desktop-model')),
    );
    expect(listed.map((row) => row['id']), contains('group/renamed'));
    expect(
      listed.map((row) => row['id']),
      isNot(contains('group/renamed-copy')),
    );
    Future<void> requestParent(String address, String session) async {
      final req = await http.postUrl(
        Uri.parse('http://$address/agent/codex/v1/chat/completions'),
      );
      req.headers.set('Authorization', 'Bearer ${await native.gatewayKey()}');
      req.headers.set('session_id', session);
      req.headers.set('Origin', 'https://client.example');
      req.write(
        jsonEncode({
          'model': 'group/parent',
          'messages': [
            {'role': 'user', 'content': 'hello'},
          ],
        }),
      );
      final resp = await req.close();
      expect(resp.statusCode, 200);
      expect(
        resp.headers.value('access-control-allow-origin'),
        'https://client.example',
      );
      expect(await utf8.decoder.bind(resp).join(), contains('desktop reply'));
    }

    await requestParent(status['address'] as String, 'desktop-test-session');
    await tapText('路由');
    await waitFor(find.text('group/parent → group/renamed → local'));
    expect(find.textContaining('Codex'), findsOneWidget);
    expect(
      find.textContaining(zh.routeSession('desktop-test-session')),
      findsOneWidget,
    );
    await tapText('网关');
    await tapText('停止网关');
    await waitFor(find.text('启动网关'));
    await tapText(zh.groups);
    await tester.ensureVisible(parentCard);
    // Tap the title; the selector in the subtitle has a separate action.
    await tester.tap(
      find.descendant(of: parentCard, matching: find.text('group/parent')),
    );
    await tester.pumpAndSettle();
    await tapText(zh.manualRouting);
    await tapText(zh.rotateMembers);
    await tapText(zh.save);
    await waitFor(
      find.descendant(
        of: parentCard,
        matching: find.text('local/desktop-model'),
      ),
    );
    await tester.tap(parentCard);
    await tester.pumpAndSettle();
    final enable = find.descendant(
      of: find.byKey(const ValueKey('group/renamed')),
      matching: find.byTooltip(zh.enableMember),
    );
    await tester.ensureVisible(enable);
    await tester.tap(enable);
    await tapText(zh.save);
    await waitFor(find.text('local/desktop-model → group/renamed'));
    expect(
      jsonDecode(settings.readAsStringSync())['group_last_picks']['parent'],
      'group/renamed',
    );
    await tester.tap(
      find.descendant(of: parentCard, matching: find.text('group/parent')),
    );
    await waitFor(find.text(zh.editGroup));
    await tapText(zh.rotateMembers);
    await tapText(zh.manualRouting);
    await tapText(zh.save);
    await waitFor(
      find.byWidgetPredicate(
        (w) => w is DropdownButton<String> && w.value == 'group/renamed',
      ),
    );
    await tapText('网关');
    await tapText('启动网关');
    await waitFor(find.text('停止网关'));
    final restarted = jsonDecode(await native.status());
    await requestParent(
      restarted['address'] as String,
      'desktop-restored-pick',
    );
    await tapText('路由');
    await waitFor(find.text('group/parent → group/renamed → local'));
    await waitFor(
      find.textContaining(zh.routeSession('desktop-restored-pick')),
    );
    await tapText('网关');
    await tapText('停止网关');
    await waitFor(find.text('启动网关'));

    final remote = Directory('$path/peer')..createSync();
    File('${remote.path}/settings.json').writeAsStringSync(
      jsonEncode({
        'listen': '127.0.0.1:0',
        'peer_listen': '/ip4/127.0.0.1/udp/0/quic-v1',
        'allowed_peers': [await native.nodeId()],
      }),
    );
    final keyResult = await Process.run(cli, [
      '--data-dir',
      remote.path,
      'key',
    ]);
    expect(keyResult.exitCode, 0);
    final remoteKey = (keyResult.stdout as String).trim();
    final server = await Process.start(cli, [
      '--data-dir',
      remote.path,
      'serve',
    ]);
    addTearDown(() async {
      server.kill();
      await server.exitCode;
    });
    final address = await server.stderr
        .transform(utf8.decoder)
        .transform(const LineSplitter())
        .firstWhere((s) => s.startsWith('P2P '))
        .timeout(const Duration(seconds: 15));
    final connect = find.widgetWithText(TextButton, zh.connect);
    await waitFor(connect);
    await Scrollable.ensureVisible(tester.element(connect), alignment: 0.5);
    await tester.pumpAndSettle();
    await tester.tap(connect.hitTestable());
    await tester.pumpAndSettle();
    final inputs = find.descendant(
      of: find.byType(AlertDialog),
      matching: find.byType(TextField),
    );
    await tester.enterText(inputs.at(0), address.substring(4));
    final remotePeer = address.split('/').last;
    await tester.enterText(
      inputs.at(1),
      '/ip4/127.0.0.1/udp/1/quic-v1/p2p/${await native.nodeId()}/p2p-circuit/p2p/$remotePeer',
    );
    await tester.enterText(inputs.at(2), '127.0.0.1:0');
    await tester.tap(
      find.descendant(
        of: find.byType(AlertDialog),
        matching: find.widgetWithText(FilledButton, zh.connect),
      ),
    );
    await tester.pumpAndSettle();
    await waitFor(find.text('断开'));
    final local = jsonDecode(await native.status())['tunnel'];
    final list = await http.getUrl(Uri.parse('http://$local/v1/models'));
    list.headers.set('Authorization', 'Bearer $remoteKey');
    final throughPeer = await list.close();
    expect(throughPeer.statusCode, 200);
    expect(
      jsonDecode(await utf8.decoder.bind(throughPeer).join())['data'],
      isEmpty,
    );
    await tapText('断开');
    await waitFor(find.text('连接'));
    expect(jsonDecode(await native.status())['tunnel'], isNull);
    await tapText(zh.providers);
    final rename = find.descendant(
      of: find.widgetWithText(ListTile, 'local'),
      matching: find.byTooltip(zh.renameProvider),
    );
    await tester.ensureVisible(rename);
    await tester.tap(rename);
    await tester.pumpAndSettle();
    await tester.enterText(
      find.widgetWithText(TextFormField, zh.providerId),
      'renamed-local',
    );
    await tapText(zh.save);
    await waitFor(find.text('renamed-local'));
    dispatcher.platformBrightnessTestValue = Brightness.dark;
    addTearDown(dispatcher.clearPlatformBrightnessTestValue);
    await tester.ensureVisible(find.byTooltip(zh.gridView));
    await tester.tap(find.byTooltip(zh.gridView));
    await waitFor(find.byType(SliverGrid));
    expect(
      Theme.of(tester.element(find.text('renamed-local'))).brightness,
      Brightness.dark,
    );
    expect(await native.uiLayout(), 'grid');
  });
}
