import 'dart:async';
import 'dart:convert';

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import 'material_symbols.dart';
import 'appearance.dart';
import 'agent_environment.dart';
import 'agent_files.dart';
import 'agent_hooks.dart';
import 'claude_plugins.dart';
import 'plugin_bundles.dart';
import 'agent_rules.dart';
import 'agent_sessions.dart';
import 'agent_skills.dart';
import 'collection_view.dart';
import 'editors.dart';
import 'integration_icon.dart';
import 'language.dart';
import 'omp_settings.dart';
import 'project_instructions.dart';
import 'project_mcp.dart';
import 'l10n/app_localizations.dart';
import 'memory.dart';
import 'src/rust/api.dart' as api;
import 'src/rust/frb_generated.dart';
import 'subscription_usage.dart';
import 'workspace.dart';

void main() {
  WidgetsFlutterBinding.ensureInitialized();
  runApp(const CrabforgeApp());
}

Iterable<String> availableModels(JsonMap provider) {
  final delisted = Set<String>.from(
    provider['model_directory']?['delisted'] ?? [],
  );
  return (provider['models'] as List).cast<String>().where(
    (model) => !delisted.contains(model),
  );
}

bool managesModelDirectory(JsonMap provider) =>
    provider['remote_gateway'] == true ||
    provider['refresh_every'] != null ||
    provider['auto_include'] != null ||
    provider['model_directory'] != null;

class CrabforgeApp extends StatefulWidget {
  const CrabforgeApp({super.key});
  @override
  State<CrabforgeApp> createState() => _CrabforgeAppState();
}

class _CrabforgeAppState extends State<CrabforgeApp> {
  String language = '';
  bool grid = false, writingLayout = false;
  WorkspaceFont font = WorkspaceFont.googleSans;
  Color? accent;
  final messenger = GlobalKey<ScaffoldMessengerState>();
  bool get hasSystemAccent =>
      !kIsWeb && defaultTargetPlatform == TargetPlatform.windows;
  BinaryMessenger get platformMessenger =>
      ServicesBinding.instance.defaultBinaryMessenger;

  late ThemeData lightTheme, darkTheme, highContrastLight, highContrastDark;

  @override
  void initState() {
    super.initState();
    refreshThemes();
    if (hasSystemAccent) {
      platformMessenger.setMessageHandler(systemAccentChannel, (message) async {
        restoreAccent(message);
        return null;
      });
      unawaited(loadAccent());
    }
    for (final (family, file) in const [
      ('Google Sans', 'GoogleSans-OFL.txt'),
      ('HarmonyOS Sans SC', 'HarmonyOS-Sans-LICENSE.txt'),
      ('Noto Sans KR', 'NotoSansKR-OFL.txt'),
      ('Material Symbols Rounded', 'MaterialSymbols-LICENSE.txt'),
    ]) {
      LicenseRegistry.addLicense(() async* {
        yield LicenseEntryWithLineBreaks(
          [family],
          await rootBundle.loadString('packages/crabforge_assets/fonts/$file'),
        );
      });
    }
  }

  @override
  void dispose() {
    if (hasSystemAccent) {
      platformMessenger.setMessageHandler(systemAccentChannel, null);
    }
    super.dispose();
  }

  Future<void> loadAccent() async {
    restoreAccent(await platformMessenger.send(systemAccentChannel, null));
  }

  void restoreAccent(ByteData? bytes) {
    final next = decodeSystemAccent(bytes);
    if (!mounted || accent == next) return;
    setState(() {
      accent = next;
      refreshThemes();
    });
  }

  void restoreFont(WorkspaceFont next) {
    if (font == next) return;
    setState(() {
      font = next;
      refreshThemes();
    });
  }

  void refreshThemes() {
    lightTheme = workspaceTheme(Brightness.light, font, accent: accent);
    darkTheme = workspaceTheme(Brightness.dark, font, accent: accent);
    highContrastLight = workspaceTheme(
      Brightness.light,
      font,
      accent: accent,
      highContrast: true,
    );
    highContrastDark = workspaceTheme(
      Brightness.dark,
      font,
      accent: accent,
      highContrast: true,
    );
  }

  void restoreLayout(bool value) {
    if (grid != value) setState(() => grid = value);
  }

  Future<void> toggleLayout() async {
    if (writingLayout) return;
    final next = !grid;
    setState(() => writingLayout = true);
    try {
      await api.setUiLayout(grid: next);
      if (mounted) restoreLayout(next);
    } catch (e) {
      messenger.currentState?.showSnackBar(SnackBar(content: Text('$e')));
    } finally {
      if (mounted) setState(() => writingLayout = false);
    }
  }

  void selectLanguage(String code) {
    final selected = languageNames.containsKey(code) ? code : '';
    if (selected != language) setState(() => language = selected);
  }

  @override
  Widget build(BuildContext context) => MaterialApp(
    debugShowCheckedModeBanner: false,
    title: 'Crabforge',
    scaffoldMessengerKey: messenger,
    builder: (context, child) => CollectionLayout(
      grid: grid,
      toggle: writingLayout ? null : toggleLayout,
      child: child!,
    ),
    theme: lightTheme,
    darkTheme: darkTheme,
    highContrastTheme: highContrastLight,
    highContrastDarkTheme: highContrastDark,
    themeAnimationDuration: Duration.zero,
    locale: language.isEmpty ? null : Locale(language),
    localizationsDelegates: AppLocalizations.localizationsDelegates,
    supportedLocales: AppLocalizations.supportedLocales,
    localeListResolutionCallback: (preferred, supported) {
      for (final locale in preferred ?? const <Locale>[]) {
        if (languageNames.containsKey(locale.languageCode)) {
          return Locale(locale.languageCode);
        }
      }
      return const Locale('en');
    },
    home: Console(
      language: language,
      onLanguage: selectLanguage,
      onLayoutLoaded: restoreLayout,
      font: font,
      onFont: restoreFont,
    ),
  );
}

class Console extends StatefulWidget {
  const Console({
    super.key,
    required this.language,
    required this.onLanguage,
    required this.onLayoutLoaded,
    required this.font,
    required this.onFont,
  });
  final String language;
  final ValueChanged<String> onLanguage;
  final ValueChanged<bool> onLayoutLoaded;
  final WorkspaceFont font;
  final ValueChanged<WorkspaceFont> onFont;
  @override
  State<Console> createState() => _ConsoleState();
}

class _ConsoleState extends State<Console> with WidgetsBindingObserver {
  int page = 0;
  String? snapshot, error;
  JsonMap settings = {}, status = {};
  String? statusSnapshot;
  List<JsonMap> presets = [];
  List<(String, String)> agents = [];
  Set<String> projectAgents = const {};
  String selectedAgent = 'claude';
  final routeSearch = TextEditingController();
  bool busy = false, reading = false, initialized = false;
  bool get grid => CollectionLayout.of(context).grid;
  Timer? timer;
  late String peerId;
  bool get running => status['running'] == true;
  List<JsonMap> providers = const [];
  Set<String> subscriptionProviders = const {};
  Map<String, String> catalogGroupRefs = const {};
  JsonMap groups = {};
  List<MapEntry<String, dynamic>> groupRows = const [];
  JsonMap expandedGroups = {};
  List<dynamic> displayMembers(String id) =>
      expandedGroups[id] ?? savedGroups[id] ?? groups[id] ?? [];
  Map<String, String> publicGroupRefs = {};
  JsonMap get savedGroups => settings['groups'];
  Set<String> get hiddenGroups =>
      Set<String>.from(settings['hidden_auto_groups'] ?? []);
  Set<String> get disabledGroups =>
      Set<String>.from(settings['disabled_groups'] ?? []);
  Set<String> get rotatingGroups =>
      Set<String>.from(settings['rotating_groups'] ?? []);
  Map<String, dynamic> get groupWeights => settings['group_weights'] ?? {};
  Map<String, dynamic> get groupPicks => settings['group_picks'] ?? {};
  ColorScheme get colors => Theme.of(context).colorScheme;

  Uri? get clientOrigin {
    final address = (running ? status['address'] : null) ?? settings['listen'];
    final origin = Uri.parse('http://$address');
    if (origin.port == 0) return null;
    return switch (origin.host) {
      '0.0.0.0' => origin.replace(host: '127.0.0.1'),
      '::' => origin.replace(host: '::1'),
      _ => origin,
    };
  }

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
    load();
  }

  @override
  void dispose() {
    WidgetsBinding.instance.removeObserver(this);
    timer?.cancel();
    routeSearch.dispose();
    super.dispose();
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    schedule();
  }

  void schedule() {
    timer?.cancel();
    final lifecycle = WidgetsBinding.instance.lifecycleState;
    final visible =
        lifecycle == null ||
        lifecycle == AppLifecycleState.resumed ||
        lifecycle == AppLifecycleState.inactive;
    final manualGroups = running && page == 2 && groupPicks.isNotEmpty;
    final liveCatalog =
        running && catalogPage && providers.any(managesModelDirectory);
    if ((manualGroups && visible) ||
        (lifecycle == AppLifecycleState.resumed &&
            (liveCatalog ||
                (running && (page == 1 || page == 3)) ||
                (page == 3 && status['tunnel'] != null)))) {
      timer = Timer.periodic(const Duration(seconds: 3), (_) => refresh());
    }
  }

  bool get catalogPage => page == 0 || page == 2 || page == 4;

  Future<void> refresh() async {
    if (reading || busy) return;
    reading = true;
    try {
      final previous = snapshot;
      final previousStatus = statusSnapshot;
      final updatedStatus = await api.status();
      if (running && catalogPage) {
        final saved = await api.configuration();
        if (saved != previous) {
          final details = await configurationDetails(saved);
          if (mounted && catalogPage && !busy && snapshot == previous) {
            setState(() => applyConfiguration(saved, details));
          }
        }
      }
      if (mounted &&
          !busy &&
          previousStatus == statusSnapshot &&
          updatedStatus != statusSnapshot) {
        setState(() => applyStatus(updatedStatus));
        if (!running && status['tunnel'] == null) schedule();
      }
    } catch (e) {
      if (mounted) {
        timer?.cancel();
        notice('$e');
      }
    } finally {
      reading = false;
    }
  }

  Future<void> load() async {
    setState(() {
      busy = true;
      error = null;
    });
    try {
      if (!initialized) {
        await RustLib.init();
        initialized = true;
      }
      final values = await Future.wait([
        api.configuration(),
        api.presets(),
        api.status(),
        api.nodeId(),
        api.agents(),
        api.uiLanguage(),
        api.uiLayout(),
        api.uiFont(),
      ]);
      final details = await configurationDetails(values[0]);
      final instructionAgents = await api.projectInstructionAgents();
      if (!mounted) return;
      widget.onLanguage(values[5]);
      widget.onLayoutLoaded(values[6] == 'grid');
      widget.onFont(WorkspaceFont.fromPreference(values[7]));
      setState(() {
        applyConfiguration(values[0], details);
        presets = (jsonDecode(values[1]) as List).cast<JsonMap>();
        applyStatus(values[2]);
        peerId = values[3];
        agents = [
          for (final item in jsonDecode(values[4]))
            (item[0] as String, item[1] as String),
        ];
        projectAgents = instructionAgents.toSet();
      });
      schedule();
    } catch (e) {
      if (mounted) {
        setState(() => error = '$e');
      }
    } finally {
      if (mounted) {
        setState(() => busy = false);
      }
    }
  }

  void notice(String s) {
    ScaffoldMessenger.of(context)
      ..hideCurrentSnackBar()
      ..showSnackBar(SnackBar(content: Text(s)));
  }

  Future<void> act(Future<void> Function() job) async {
    if (busy) return;
    setState(() => busy = true);
    try {
      await job();
    } catch (e) {
      if (mounted) notice('$e');
    } finally {
      if (mounted) {
        setState(() => busy = false);
        schedule();
      }
    }
  }

  Future<void> save(JsonMap data) =>
      act(() => persist(snapshot!, jsonEncode(data)));

  Future<void> persist(String previous, String text) async {
    await api.saveConfiguration(previous: previous, text: text);
    await reloadConfiguration();
  }

  Future<void> reloadConfiguration() async {
    final s = await api.configuration();
    final details = await configurationDetails(s);
    if (mounted) {
      setState(() => applyConfiguration(s, details));
    }
  }

  Future<(JsonMap, Set<String>)> configurationDetails(String saved) async {
    final subscriptions = api.subscriptionUsageProviders(configuration: saved);
    final resolved = api.routeGroups(configuration: saved);
    final (routes, supported) = await (resolved, subscriptions).wait;
    return (jsonDecode(routes) as JsonMap, supported.toSet());
  }

  void applyConfiguration(String saved, (JsonMap, Set<String>) details) {
    final (resolved, subscriptions) = details;
    snapshot = saved;
    settings = jsonDecode(saved);
    providers = (settings['providers'] as List).cast<JsonMap>();
    subscriptionProviders = subscriptions;
    expandedGroups = resolved['expanded'] ?? {};
    groups = {
      for (final name in resolved['order']) name: resolved['members'][name],
    };
    groupRows = groups.entries.toList(growable: false);
    publicGroupRefs = Map<String, String>.from(resolved['public_refs']);
    catalogGroupRefs = {
      for (final name in groups.keys)
        name: publicGroupRefs['group/$name'] ?? 'group/$name',
    };
  }

  void applyStatus(String value) {
    statusSnapshot = value;
    status = jsonDecode(value) as JsonMap;
  }

  Future<void> copy(String s, String message) async {
    await Clipboard.setData(ClipboardData(text: s));
    if (mounted) notice(message);
  }

  Future<void> shareGroup(String id) async {
    final text = await api.sharedGroup(configuration: snapshot!, id: id);
    if (!mounted) return;
    try {
      await copy(text, context.strings.groupCopied);
    } on PlatformException {
      if (!mounted) return;
      await showDialog<void>(
        context: context,
        builder: (c) => AlertDialog(
          title: Text(c.strings.sharedGroupText),
          scrollable: true,
          content: SelectableText(text),
          actions: [
            TextButton(
              onPressed: () => Navigator.pop(c),
              child: Text(MaterialLocalizations.of(c).closeButtonLabel),
            ),
          ],
        ),
      );
    }
  }

  void navigate(int index) {
    if (index == page) return;
    setState(() => page = index);
    schedule();
    if (running) refresh();
  }

  Future<bool> confirm(String label) async =>
      await showDialog<bool>(
        context: context,
        builder: (c) => AlertDialog(
          title: Text(context.strings.confirmRemove(label)),
          actions: [
            TextButton(
              onPressed: () => Navigator.pop(c, false),
              child: Text(context.strings.cancel),
            ),
            FilledButton(
              onPressed: () => Navigator.pop(c, true),
              child: Text(context.strings.remove),
            ),
          ],
        ),
      ) ??
      false;

  Future<void> provider([JsonMap? old]) async {
    final p = await showDialog<JsonMap>(
      context: context,
      builder: (_) => ProviderEditor(presets: presets, provider: old),
    );
    if (p == null || !mounted) return;
    final items = [...providers];
    if (old == null) {
      items.add(p);
    } else {
      items[items.indexOf(old)] = p;
    }
    await save({...settings, 'providers': items});
  }

  Future<void> renameProvider(String id) async {
    final previous = snapshot!;
    var name = id;
    final picked = await showDialog<String>(
      context: context,
      builder: (c) => AlertDialog(
        title: Text(c.strings.renameProvider),
        content: TextFormField(
          initialValue: id,
          autofocus: true,
          decoration: InputDecoration(labelText: c.strings.providerId),
          onChanged: (value) => name = value.trim(),
          onFieldSubmitted: (value) => Navigator.pop(c, value.trim()),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(c),
            child: Text(c.strings.cancel),
          ),
          FilledButton(
            onPressed: () => Navigator.pop(c, name),
            child: Text(c.strings.save),
          ),
        ],
      ),
    );
    if (picked == null || picked == id || !mounted) return;
    await act(() async {
      try {
        await api.renameProvider(previous: previous, from: id, to: picked);
      } finally {
        await reloadConfiguration();
      }
    });
  }

  Future<void> login(String site) => act(() async {
    await api.webLogin(site: site);
    try {
      if (!mounted) return;
      await showDialog<void>(
        context: context,
        barrierDismissible: false,
        builder: (ctx) => AlertDialog(
          title: Text(context.strings.webLogin),
          content: Text(context.strings.loginHelp),
          actions: [
            FilledButton(
              onPressed: () => Navigator.pop(ctx),
              child: Text(context.strings.closeLogin),
            ),
          ],
        ),
      );
    } finally {
      await api.finishWebLogin();
    }
  });

  Future<void> addWeb(String site) async {
    final id = '$site-web';
    await save({
      ...settings,
      'providers': [
        ...providers,
        {
          'id': id,
          'web_site': site,
          'models': [id],
        },
      ],
    });
  }

  Future<void> group([String? id]) async {
    final value = await showDialog<GroupSelection>(
      context: context,
      builder: (_) => GroupEditor(
        configuration: snapshot!,
        patterns: List<String>.from(settings['group_match']?[id] ?? []),
        placed: List<String>.from(settings['group_placed']?[id] ?? []),
        name: id,
        note: settings['group_notes']?[id],
        firstContent: settings['group_first_content']?[id] ?? 0,
        patience: settings['group_patience']?[id] ?? 0,
        routingRules: {
          ...?settings['group_classifiers']?[id] as JsonMap?,
          'rules': settings['group_rules']?[id] ?? [],
        },
        listMembers: settings['group_unlists']?.contains(id) != true,
        off: List<String>.from(settings['group_off']?[id] ?? []),
        pick: groupPicks[id] ?? settings['group_last_picks']?[id],
        strategy: groupPicks.containsKey(id)
            ? GroupStrategy.manual
            : groupWeights.containsKey(id)
            ? GroupStrategy.weight
            : rotatingGroups.contains(id)
            ? GroupStrategy.rotate
            : GroupStrategy.order,
        weights: Map<String, int>.from(
          groupWeights[id] ?? settings['group_last_weights']?[id] ?? {},
        ),
        members: List<String>.from(savedGroups[id] ?? groups[id] ?? []),
        choices: [
          for (final p in providers)
            for (final m in p['models']) '${p['id']}/$m',
          for (final g in groups.keys)
            if (g != id) publicGroupRefs['group/$g'] ?? 'group/$g',
        ],
      ),
    );
    if (value == null || !mounted) return;
    final name = value.name;
    if (id == null && groups.containsKey(name)) {
      notice(context.strings.duplicateGroup);
      return;
    }
    JsonMap base = settings;
    if (id != null && id != name) {
      try {
        base = jsonDecode(
          await api.renamedGroup(configuration: snapshot!, from: id, to: name),
        ) as JsonMap;
      } catch (e) {
        if (mounted) notice('$e');
        return;
      }
      if (!mounted) return;
    }
    final routingRules = value.routingRules['rules'] as List;
    final classifiers = {...value.routingRules}..remove('rules');
    await save({
      ...base,
      'groups': {...base['groups'] as JsonMap, name: value.members},
      'group_match': {
        ...?base['group_match'] as JsonMap?,
        name: value.patterns,
      },
      'group_placed': {
        ...?base['group_placed'] as JsonMap?,
        name: value.placed,
      },
      'group_notes': {...?base['group_notes'] as JsonMap?, name: value.note},
      'group_first_content': {
        for (final e in (base['group_first_content'] as JsonMap? ?? {}).entries)
          if (e.key != name) e.key: e.value,
        if (value.firstContent != 0) name: value.firstContent,
      },
      'group_patience': {
        for (final e in (base['group_patience'] as JsonMap? ?? {}).entries)
          if (e.key != name) e.key: e.value,
        if (value.patience != 0) name: value.patience,
      },
      'group_rules': {
        for (final e in (base['group_rules'] as JsonMap? ?? {}).entries)
          if (e.key != name) e.key: e.value,
        if (routingRules.isNotEmpty) name: routingRules,
      },
      'group_classifiers': {
        for (final e in (base['group_classifiers'] as JsonMap? ?? {}).entries)
          if (e.key != name) e.key: e.value,
        if (classifiers.isNotEmpty) name: classifiers,
      },
      'group_off': {...?base['group_off'] as JsonMap?, name: value.off},
      'group_unlists': [
        for (final g in base['group_unlists'] ?? [])
          if (g != name) g,
        if (!value.listMembers) name,
      ],
      'former_groups': {...?base['former_groups'] as JsonMap?}..remove(name),
      'hidden_auto_groups': [
        for (final g in base['hidden_auto_groups'] ?? [])
          if (g != name) g,
      ],
      'group_weights': {
        for (final e in (base['group_weights'] as JsonMap? ?? {}).entries)
          if (e.key != name) e.key: e.value,
        if (value.strategy == GroupStrategy.weight) name: value.weights,
      },
      'group_last_weights': {
        for (final e in (base['group_last_weights'] as JsonMap? ?? {}).entries)
          if (e.key != name) e.key: e.value,
        if (value.strategy != GroupStrategy.weight &&
            value.weights.values.any((w) => w > 1))
          name: {
            for (final e in value.weights.entries)
              if (e.value > 1) e.key: e.value,
          },
      },
      'group_picks': {
        for (final e in (base['group_picks'] as JsonMap? ?? {}).entries)
          if (e.key != name) e.key: e.value,
        if (value.strategy == GroupStrategy.manual) name: value.pick ?? '',
      },
      'group_last_picks': {
        for (final e in (base['group_last_picks'] as JsonMap? ?? {}).entries)
          if (e.key != name) e.key: e.value,
        if (value.strategy != GroupStrategy.manual && value.pick != null)
          name: value.pick,
      },
      'rotating_groups': [
        for (final g in base['rotating_groups'] ?? [])
          if (g != name) g,
        if (value.strategy == GroupStrategy.rotate) name,
      ],
    });
  }

  Future<void> network() async {
    final data = await showDialog<JsonMap>(
      context: context,
      builder: (_) => NetworkEditor(config: settings),
    );
    if (data != null && mounted) await save({...settings, ...data});
  }

  @override
  Widget build(BuildContext context) {
    final pages = [
      (context.strings.providers, CrabforgeSymbols.vpnKey),
      (context.strings.routes, CrabforgeSymbols.altRoute),
      (context.strings.groups, CrabforgeSymbols.accountTree),
      (context.strings.gateway, CrabforgeSymbols.dns),
      (context.strings.models, CrabforgeSymbols.viewList),
      (context.strings.clients, CrabforgeSymbols.devices),
    ];
    final wide = MediaQuery.sizeOf(context).width >= workspaceWideBreakpoint;
    final ready = snapshot != null && error == null;
    final body = ready && page == 4
        ? ModelCatalog(
            grid: grid,
            onToggle: busy ? null : CollectionLayout.of(context).toggle,
            providers: providers,
            groupRefs: catalogGroupRefs,
            canEdit: !running && !busy,
            onEdit: provider,
            onGroup: group,
          )
        : ready
        ? CustomScrollView(
            key: PageStorageKey(page),
            slivers: [
              SliverPadding(
                padding: EdgeInsets.symmetric(
                  horizontal: wide ? 38 : 18,
                  vertical: 28,
                ),
                sliver: SliverMainAxisGroup(
                  slivers: [
                    SliverList.list(
                      children: [
                        WorkspaceHeader(
                          title: pages[page].$1,
                          detail: switch (page) {
                            0 => [
                              context.strings.providerCount(providers.length),
                              context.strings.presetCount(presets.length),
                            ].join('  ·  '),
                            1 => context.strings.historyLimit,
                            _ => null,
                          },
                          actions: [
                            LayoutToggle(
                              grid: grid,
                              onPressed: busy
                                  ? null
                                  : CollectionLayout.of(context).toggle,
                            ),
                            if (page == 0 || page == 2) ...[
                              FilledButton.icon(
                                onPressed:
                                    busy ||
                                        running ||
                                        (page == 2 && providers.isEmpty)
                                    ? null
                                    : () {
                                        if (page == 0) {
                                          provider();
                                        } else {
                                          group();
                                        }
                                      },
                                icon: const Icon(CrabforgeSymbols.add),
                                label: Text(
                                  page == 0
                                      ? context.strings.addProvider
                                      : context.strings.newGroup,
                                ),
                              ),
                            ],
                          ],
                        ),
                        if (running && (page == 0 || page == 2)) ...[
                          note(
                            context.strings.stopToEdit,
                            CrabforgeSymbols.lock,
                          ),
                          const SizedBox(height: 16),
                        ],
                      ],
                    ),
                    ...switch (page) {
                      0 => providerPage(),
                      1 => routesPage(),
                      2 => groupsPage(),
                      3 => gatewayPage(),
                      _ => agentPage(),
                    },
                  ],
                ),
              ),
            ],
          )
        : Center(
            child: error == null
                ? const CircularProgressIndicator()
                : Padding(
                    padding: const EdgeInsets.all(32),
                    child: Column(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        SelectableText(error!),
                        TextButton(
                          onPressed: busy ? null : load,
                          child: Text(context.strings.reload),
                        ),
                      ],
                    ),
                  ),
          );
    final languageMenu = PopupMenuButton<String>(
      key: const ValueKey('language-picker'),
      enabled: initialized && !busy,
      tooltip: context.strings.language,
      initialValue: widget.language,
      icon: const Icon(CrabforgeSymbols.language),
      onSelected: (code) => act(() async {
        await api.setUiLanguage(language: code);
        if (mounted) widget.onLanguage(code);
      }),
      itemBuilder: (_) => [
        workspaceMenuChoice(
          value: '',
          selected: widget.language.isEmpty,
          label: Text(context.strings.systemLanguage),
        ),
        for (final e in languageNames.entries)
          workspaceMenuChoice(
            value: e.key,
            selected: widget.language == e.key,
            label: Text(e.value),
          ),
      ],
    );
    return Scaffold(
      appBar: AppBar(
        toolbarHeight: wide ? 64 : 56,
        titleSpacing: wide ? 14 : 8,
        title: TextButton(
          style: TextButton.styleFrom(
            foregroundColor: colors.onSurface,
            padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
            shape: RoundedRectangleBorder(
              borderRadius: BorderRadius.circular(14),
            ),
          ),
          onPressed: () => showAboutDialog(
            context: context,
            applicationName: 'Crabforge',
            applicationVersion: status['version'] as String?,
            applicationIcon: const ImageIcon(
              AssetImage('crabforge-mark.png', package: 'crabforge_assets'),
              size: 36,
            ),
            children: const [
              Text('Google Sans · HarmonyOS Sans SC · Noto Sans KR'),
            ],
          ),
          child: const WorkspaceBrand(),
        ),
        actions: [
          if (ready &&
              MediaQuery.sizeOf(context).width >= 560 &&
              MediaQuery.textScalerOf(context).scale(16) < 22 &&
              page != 3)
            WorkspaceStatus(
              running: running,
              label: running
                  ? context.strings.running
                  : context.strings.stopped,
              onPressed: () => navigate(3),
            ),
          languageMenu,
          PopupMenuButton<WorkspaceFont>(
            key: const ValueKey('font-picker'),
            enabled: initialized && !busy,
            tooltip: context.strings.interfaceFont,
            initialValue: widget.font,
            icon: const Icon(CrabforgeSymbols.formatSize),
            onSelected: (font) => act(() async {
              await api.setUiFont(font: font.preference);
              if (mounted) widget.onFont(font);
            }),
            itemBuilder: (_) => [
              for (final font in WorkspaceFont.values)
                workspaceMenuChoice(
                  value: font,
                  selected: widget.font == font,
                  label: Text(
                    font.label,
                    style: TextStyle(
                      fontFamily: font.family,
                      fontFamilyFallback: font.fallback,
                    ),
                  ),
                ),
            ],
          ),
          if (ready)
            IconButton(
              tooltip: context.strings.reloadConfig,
              onPressed: busy ? null : load,
              icon: const Icon(CrabforgeSymbols.refresh),
            ),
          const SizedBox(width: 12),
        ],
      ),
      bottomNavigationBar: wide
          ? null
          : NavigationBar(
              selectedIndex: page,
              onDestinationSelected: navigate,
              destinations: [
                for (final p in pages)
                  NavigationDestination(
                    icon: Icon(p.$2),
                    selectedIcon: Icon(p.$2, fill: 1),
                    label: p.$1,
                  ),
              ],
            ),
      body: SafeArea(
        child: Padding(
          padding: wide
              ? const EdgeInsetsDirectional.only(end: 12, bottom: 12)
              : EdgeInsets.zero,
          child: Row(
            children: [
              if (wide)
                SizedBox(
                  width: 224,
                  child: NavigationDrawer(
                    selectedIndex: page,
                    onDestinationSelected: navigate,
                    children: [
                      const SizedBox(height: 16),
                      for (final p in pages)
                        NavigationDrawerDestination(
                          icon: Icon(p.$2),
                          selectedIcon: Icon(p.$2, fill: 1),
                          label: SizedBox(
                            width: 136,
                            child: Text(
                              p.$1,
                              maxLines: 2,
                              overflow: TextOverflow.ellipsis,
                            ),
                          ),
                        ),
                    ],
                  ),
                ),
              Expanded(
                child: Material(
                  color: colors.surface,
                  borderRadius: wide
                      ? BorderRadius.circular(28)
                      : BorderRadius.zero,
                  clipBehavior: Clip.antiAlias,
                  child: Center(
                    child: ConstrainedBox(
                      constraints: const BoxConstraints(maxWidth: 1360),
                      child: WorkspacePageTransition(
                        page: ready ? page : -1,
                        child: body,
                      ),
                    ),
                  ),
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }

  Widget note(String text, IconData icon) => Card.filled(
    child: Padding(
      padding: const EdgeInsets.symmetric(horizontal: 18, vertical: 16),
      child: Row(
        children: [
          Icon(icon, color: colors.primary),
          const SizedBox(width: 16),
          Expanded(child: Text(text)),
        ],
      ),
    ),
  );
  Widget heading(String title) => Padding(
    padding: const EdgeInsets.only(top: 24, bottom: 12),
    child: Semantics(
      header: true,
      child: Text(
        title,
        style: Theme.of(context).textTheme.titleMedium
            ?.copyWith(fontWeight: FontWeight.w600),
      ),
    ),
  );
  List<Widget> providerPage() => [
    if (providers.isEmpty)
      SliverToBoxAdapter(
        child: note(context.strings.providerHelp, CrabforgeSymbols.link),
      ),
    CollectionSliver.builder(
      grid: grid,
      height: 118,
      itemCount: providers.length,
      itemBuilder: (context, index) {
        final p = providers[index];
        final models = availableModels(p).join(' · ');
        return CollectionTile(
          grid: grid,
          title: collectionTitle(p['id']),
          detail: Text(
            models.isEmpty ? context.strings.noProviderModels : models,
            maxLines: 2,
            overflow: TextOverflow.ellipsis,
          ),
          leading: IntegrationIcon.provider(
            p['web_site'] ?? p['id'],
            web: p['web_site'] != null,
            remote: p['remote_gateway'] == true,
          ),
          onTap: busy || running
              ? null
              : () =>
                    p['web_site'] == null ? provider(p) : login(p['web_site']),
          actions: [
            if (subscriptionProviders.contains(p['id']))
              IconButton(
                tooltip: context.strings.subscriptionUsage,
                icon: const Icon(CrabforgeSymbols.timeline),
                onPressed: busy
                    ? null
                    : () => showDialog<void>(
                        context: context,
                        builder: (_) => SubscriptionUsageDialog(
                          provider: p['id'],
                          read: () =>
                              api.providerSubscriptionUsage(id: p['id']),
                        ),
                      ),
              ),
            if (managesModelDirectory(p))
              IconButton(
                tooltip: context.strings.readModels,
                icon: const Icon(CrabforgeSymbols.sync),
                onPressed: busy
                    ? null
                    : () => act(() async {
                        await api.refreshProviderCatalog(id: p['id']);
                        await reloadConfiguration();
                      }),
              ),
            IconButton(
              tooltip: context.strings.renameProvider,
              icon: const Icon(CrabforgeSymbols.editSquare),
              onPressed: busy || running ? null : () => renameProvider(p['id']),
            ),
            IconButton(
              tooltip: context.strings.removeProvider,
              onPressed: busy || running
                  ? null
                  : () async {
                      if (await confirm(p['id']) && mounted) {
                        await save({
                          ...settings,
                          'providers': providers.where((i) => i != p).toList(),
                        });
                      }
                    },
              icon: const Icon(CrabforgeSymbols.delete),
            ),
          ],
        );
      },
    ),
    SliverList.list(
      children: [
        heading(context.strings.webProviders),
        Padding(
          padding: const EdgeInsets.only(bottom: 12),
          child: Text(context.strings.webHelp),
        ),
        Wrap(
          spacing: 8,
          runSpacing: 8,
          children: [
            for (final s in [('DeepSeek', 'deepseek'), ('ChatGLM', 'chatglm')])
              OutlinedButton.icon(
                onPressed:
                    busy ||
                        running ||
                        providers.any((p) => p['id'] == '${s.$2}-web')
                    ? null
                    : () => addWeb(s.$2),
                icon: IntegrationIcon.provider(s.$2, web: true, size: 18),
                label: Text(context.strings.webSite(s.$1)),
              ),
          ],
        ),
      ],
    ),
  ];
  Widget collectionTitle(String value) => Text(
    value,
    maxLines: grid ? 2 : null,
    overflow: grid ? TextOverflow.ellipsis : null,
  );

  List<Widget> groupsPage() => [
    SliverList.list(
      children: [
        Align(
          alignment: AlignmentDirectional.centerStart,
          child: TextButton.icon(
            onPressed: busy || running ? null : importGroup,
            icon: const Icon(CrabforgeSymbols.fileDownload),
            label: Text(context.strings.importGroup),
          ),
        ),
        SwitchListTile(
          title: Text(context.strings.autoGroups),
          value: settings['no_auto_groups'] != true,
          onChanged: busy || running
              ? null
              : (enabled) => save({...settings, 'no_auto_groups': !enabled}),
        ),
      ],
    ),
    CollectionSliver(
      grid: grid,
      height: 60,
      children: [
        for (final name in hiddenGroups)
          ListTile(
            title: Text('group/$name'),
            trailing: TextButton(
              onPressed: busy || running
                  ? null
                  : () => save({
                      ...settings,
                      'hidden_auto_groups': [
                        ...hiddenGroups.where((g) => g != name),
                      ],
                    }),
              child: Text(context.strings.restoreGroup),
            ),
          ),
      ],
    ),
    if (groups.isEmpty)
      SliverToBoxAdapter(
        child: note(
          providers.isEmpty
              ? context.strings.groupNeedsProvider
              : context.strings.groupHelp,
          CrabforgeSymbols.accountTree,
        ),
      ),
    CollectionSliver.builder(
      grid: grid,
      height: 166,
      itemCount: groupRows.length,
      itemBuilder: (context, index) {
        final e = groupRows[index];
        return CollectionTile(
          grid: grid,
          title: collectionTitle(
            [
              if (settings['group_labels']?[e.key] case final String label)
                label,
              publicGroupRefs['group/${e.key}'] ?? 'group/${e.key}',
            ].join(' · '),
          ),
          detail: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              if (settings['group_notes']?[e.key] case final String note)
                Text(note),
              if (groupPicks.containsKey(e.key))
                DropdownButton<String>(
                  icon: const Icon(CrabforgeSymbols.arrowDropDown),
                  isExpanded: true,
                  hint: Text(context.strings.selectMember),
                  value: displayMembers(e.key).contains(groupPicks[e.key])
                      ? groupPicks[e.key]
                      : displayMembers(e.key).firstOrNull,
                  items: [
                    for (final member in displayMembers(e.key))
                      DropdownMenuItem<String>(
                        value: member,
                        child: Text(member, overflow: TextOverflow.ellipsis),
                      ),
                  ],
                  onChanged: busy
                      ? null
                      : (value) {
                          if (value != null) {
                            save({
                              ...settings,
                              'group_picks': {...groupPicks, e.key: value},
                            });
                          }
                        },
                )
              else
                Text(
                  displayMembers(e.key)
                      .where(
                        (m) =>
                            settings['group_off']?[e.key]?.contains(m) != true,
                      )
                      .join(' → '),
                ),
            ],
          ),
          leading: savedGroups.containsKey(e.key)
              ? IconButton(
                  tooltip: disabledGroups.contains(e.key)
                      ? context.strings.enableGroup
                      : context.strings.disableGroup,
                  icon: Icon(
                    disabledGroups.contains(e.key)
                        ? CrabforgeSymbols.toggleOff
                        : CrabforgeSymbols.toggleOn,
                  ),
                  onPressed: busy || running
                      ? null
                      : () => save({
                          ...settings,
                          'disabled_groups': [
                            ...disabledGroups.where((id) => id != e.key),
                            if (!disabledGroups.contains(e.key)) e.key,
                          ],
                        }),
                )
              : const Icon(CrabforgeSymbols.accountTree),
          onTap: busy || running ? null : () => group(e.key),
          actions: [
            IconButton(
              tooltip: WidgetsLocalizations.of(context).reorderItemUp,
              onPressed: busy || running || e.key == groups.keys.first
                  ? null
                  : () => moveGroup(e.key, -1),
              icon: const Icon(CrabforgeSymbols.arrowUpward),
            ),
            IconButton(
              tooltip: WidgetsLocalizations.of(context).reorderItemDown,
              onPressed: busy || running || e.key == groups.keys.last
                  ? null
                  : () => moveGroup(e.key, 1),
              icon: const Icon(CrabforgeSymbols.arrowDownward),
            ),
            IconButton(
              tooltip: context.strings.copyGroup,
              onPressed: busy || running
                  ? null
                  : () => act(() async {
                      final previous = snapshot!;
                      final text = await api.copiedGroup(
                        configuration: previous,
                        from: e.key,
                      );
                      await persist(previous, text);
                    }),
              icon: const Icon(CrabforgeSymbols.contentCopy),
            ),
            IconButton(
              tooltip: context.strings.shareGroup,
              onPressed: busy ? null : () => act(() => shareGroup(e.key)),
              icon: const Icon(CrabforgeSymbols.share),
            ),
            IconButton(
              tooltip: context.strings.removeGroup,
              onPressed: busy || running
                  ? null
                  : () async {
                      if (await confirm(e.key) && mounted) {
                        await save({
                          ...settings,
                          'groups': {...savedGroups}..remove(e.key),
                          'group_match': {
                            ...?settings['group_match'] as JsonMap?,
                          }..remove(e.key),
                          'group_placed': {
                            ...?settings['group_placed'] as JsonMap?,
                          }..remove(e.key),
                          'disabled_groups': disabledGroups
                              .where((id) => id != e.key)
                              .toList(),
                          'group_unlists': [
                            for (final g in settings['group_unlists'] ?? [])
                              if (g != e.key) g,
                          ],
                          'group_off': {...?settings['group_off'] as JsonMap?}
                            ..remove(e.key),
                          'group_notes': {
                            ...?settings['group_notes'] as JsonMap?,
                          }..remove(e.key),
                          'group_first_content': {
                            ...?settings['group_first_content'] as JsonMap?,
                          }..remove(e.key),
                          'group_patience': {
                            ...?settings['group_patience'] as JsonMap?,
                          }..remove(e.key),
                          'group_rules': {
                            ...?settings['group_rules'] as JsonMap?,
                          }..remove(e.key),
                          'group_classifiers': {
                            ...?settings['group_classifiers'] as JsonMap?,
                          }..remove(e.key),
                          'group_labels': {
                            ...?settings['group_labels'] as JsonMap?,
                          }..remove(e.key),
                          'group_display_order': [
                            for (final name
                                in settings['group_display_order'] ?? [])
                              if (name != e.key) name,
                          ],
                          'former_groups': {
                            ...?settings['former_groups'] as JsonMap?,
                          }..removeWhere((_, v) => v == e.key),
                          'hidden_auto_groups': {
                            ...hiddenGroups,
                            if (e.key.startsWith('auto-')) e.key,
                          }.toList(),
                          'group_weights': {...groupWeights}..remove(e.key),
                          'group_last_weights': {
                            ...?settings['group_last_weights'] as JsonMap?,
                          }..remove(e.key),
                          'group_picks': {...groupPicks}..remove(e.key),
                          'group_last_picks': {
                            ...?settings['group_last_picks'] as JsonMap?,
                          }..remove(e.key),
                          'rotating_groups': rotatingGroups
                              .where((g) => g != e.key)
                              .toList(),
                        });
                      }
                    },
              icon: const Icon(CrabforgeSymbols.delete),
            ),
          ],
        );
      },
    ),
  ];
  Future<void> moveGroup(String name, int step) {
    final order = groups.keys.toList();
    final index = order.indexOf(name);
    order.removeAt(index);
    order.insert(index + step, name);
    return save({...settings, 'group_display_order': order});
  }

  Future<void> importGroup() async {
    final previous = snapshot!;
    final result = await showDialog<JsonMap>(
      context: context,
      builder: (_) => SharedGroupImport(configuration: previous),
    );
    if (!mounted || result == null) return;
    await act(() async {
      await persist(previous, jsonEncode(result['configuration']));
      if (!mounted) return;
      final report = result['report'] as JsonMap;
      final missing = List<String>.from(report['missing']);
      final instead = List<String>.from(report['instead']);
      final ref = 'group/${report['id']}';
      if (missing.isEmpty && instead.isEmpty) {
        notice(ref);
        return;
      }
      await showDialog<void>(
        context: context,
        builder: (c) => AlertDialog(
          scrollable: true,
          title: Text(ref),
          content: SelectableText(
            [
              if (missing.isNotEmpty)
                '${context.strings.missingMembers}\n${missing.join('\n')}',
              if (instead.isNotEmpty)
                '${context.strings.replacedMembers}\n${instead.join('\n')}',
            ].join('\n\n'),
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.pop(c),
              child: Text(MaterialLocalizations.of(c).closeButtonLabel),
            ),
          ],
        ),
      );
    });
  }

  List<Widget> routesPage() {
    final attempts = (status['attempts'] as List).cast<JsonMap>();
    final routeQuery = routeSearch.text.trim().toLowerCase();
    final visible = routeQuery.isEmpty
        ? attempts
        : attempts
              .where(
                (attempt) =>
                    [
                      attempt['model'],
                      attempt['provider'],
                      attempt['agent'],
                      attempt['session'],
                      attempt['status'],
                      attempt['reason'],
                      ...?attempt['via'] as List?,
                    ].any(
                      (value) =>
                          value?.toString().toLowerCase().contains(
                            routeQuery,
                          ) ??
                          false,
                    ),
              )
              .toList();
    return [
      SliverList.list(
        children: [
          if (attempts.isEmpty)
            note(
              running ? context.strings.waiting : context.strings.historyHelp,
              CrabforgeSymbols.timeline,
            )
          else ...[
            SearchBar(
              controller: routeSearch,
              hintText: MaterialLocalizations.of(context).searchFieldLabel,
              leading: const Icon(CrabforgeSymbols.search),
              trailing: [
                if (routeSearch.text.isNotEmpty)
                  IconButton(
                    tooltip: MaterialLocalizations.of(context)
                        .clearButtonTooltip,
                    onPressed: () => setState(routeSearch.clear),
                    icon: const Icon(CrabforgeSymbols.close),
                  ),
              ],
              onChanged: (_) => setState(() {}),
            ),
            if (visible.isEmpty)
              Padding(
                padding: const EdgeInsets.only(top: 16),
                child: note(
                  context.strings.noMatchingAttempts,
                  CrabforgeSymbols.searchOff,
                ),
              )
            else
              heading(context.strings.recentAttempts(visible.length)),
          ],
        ],
      ),
      CollectionSliver.builder(
        grid: grid,
        height: 110,
        itemCount: visible.length,
        itemBuilder: (context, index) {
          final a = visible[index];
          return CollectionTile(
            grid: grid,
            leading: Icon(
              a['status'] < 400
                  ? CrabforgeSymbols.checkCircle
                  : CrabforgeSymbols.error,
              color: a['status'] < 400 ? colors.primary : colors.error,
            ),
            title: collectionTitle(
              [a['model'], ...?a['via'] as List?, a['provider']].join(' → '),
            ),
            detail: Text(
              [
                a['agent'],
                if (a['session'] case final String id)
                  context.strings.routeSession(id),
                '${a['status']} ${a['reason']}',
              ].whereType<String>().join(' · '),
            ),
            actions: [
              Text(
                TimeOfDay.fromDateTime(
                  DateTime.fromMillisecondsSinceEpoch(a['at'] * 1000),
                ).format(context),
              ),
            ],
          );
        },
      ),
    ];
  }

  List<Widget> gatewayPage() {
    final base = clientOrigin?.replace(path: '/v1').toString();
    final foreground = colors.onPrimaryContainer;
    return [
      SliverList.list(
        children: [
          Card.filled(
            color: colors.primaryContainer,
            shape: RoundedRectangleBorder(
              borderRadius: BorderRadius.circular(28),
            ),
            child: Padding(
              padding: const EdgeInsets.all(24),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Row(
                    children: [
                      CollectionGlyph(
                        Icon(CrabforgeSymbols.dns, fill: running ? 1 : 0),
                      ),
                      const SizedBox(width: 14),
                      Expanded(
                        child: Text(
                          running
                              ? context.strings.running
                              : context.strings.stopped,
                          style: Theme.of(context).textTheme.headlineSmall
                              ?.copyWith(
                                color: foreground,
                                fontWeight: FontWeight.w600,
                              ),
                        ),
                      ),
                    ],
                  ),
                  if (base != null) ...[
                    const SizedBox(height: 12),
                    ListTile(
                      contentPadding: EdgeInsets.zero,
                      title: Text(context.strings.apiBase),
                      subtitle: SelectableText(base),
                      textColor: foreground,
                      iconColor: foreground,
                      trailing: IconButton(
                        tooltip: context.strings.copyBase,
                        onPressed: () => copy(base, context.strings.baseCopied),
                        icon: const Icon(CrabforgeSymbols.contentCopy),
                      ),
                    ),
                  ],
                  const SizedBox(height: 16),
                  Wrap(
                    spacing: 8,
                    runSpacing: 8,
                    children: [
                      FilledButton.icon(
                        onPressed: busy
                            ? null
                            : () => act(() async {
                                await api.setRunning(on_: !running);
                                final s = await api.status();
                                if (mounted) setState(() => applyStatus(s));
                              }),
                        icon: Icon(
                          running
                              ? CrabforgeSymbols.stop
                              : CrabforgeSymbols.playArrow,
                        ),
                        label: Text(
                          running
                              ? context.strings.stop
                              : context.strings.start,
                        ),
                      ),
                      OutlinedButton.icon(
                        style: OutlinedButton.styleFrom(
                          foregroundColor: foreground,
                        ),
                        onPressed: busy
                            ? null
                            : () => act(() async {
                                final key = await api.gatewayKey();
                                if (mounted) {
                                  await copy(key, context.strings.keyCopied);
                                }
                              }),
                        icon: const Icon(CrabforgeSymbols.key),
                        label: Text(context.strings.copyKey),
                      ),
                      TextButton.icon(
                        style: TextButton.styleFrom(
                          foregroundColor: foreground,
                        ),
                        onPressed: busy || running ? null : network,
                        icon: const Icon(CrabforgeSymbols.tune),
                        label: Text(context.strings.networkSettings),
                      ),
                    ],
                  ),
                ],
              ),
            ),
          ),
          heading('P2P · QUIC'),
        ],
      ),
      CollectionSliver(
        grid: grid,
        height: 105,
        children: [
          CollectionTile(
            grid: grid,
            leading: const Icon(CrabforgeSymbols.vpnKey),
            title: collectionTitle(context.strings.localPeer),
            detail: SelectableText(peerId),
            actions: [
              IconButton(
                tooltip: context.strings.copyPeer,
                onPressed: () => copy(peerId, context.strings.peerCopied),
                icon: const Icon(CrabforgeSymbols.contentCopy),
              ),
            ],
          ),
          CollectionTile(
            grid: grid,
            leading: const Icon(CrabforgeSymbols.settingsEthernet),
            title: collectionTitle(context.strings.listenAddress),
            detail: SelectableText(
              status['peer'] ??
                  (settings['peer_listen'].isEmpty
                      ? context.strings.disabled
                      : settings['peer_listen']),
            ),
          ),
          CollectionTile(
            grid: grid,
            leading: const Icon(CrabforgeSymbols.groupWork),
            title: collectionTitle(context.strings.trustedPeers),
            detail: Text('${(settings['allowed_peers'] as List).length}'),
          ),
          Card.filled(
            child: ListTile(
              leading: const Icon(CrabforgeSymbols.devices),
              title: Text(context.strings.clients),
              trailing: const Icon(CrabforgeSymbols.chevronRight),
              onTap: () => navigate(5),
            ),
          ),
          gatewayTunnel(),
        ],
      ),
      SliverToBoxAdapter(
        child: Padding(
          padding: const EdgeInsets.all(12),
          child: Text(
            'Crabforge ${status['version']} · com.blue.crabforge',
            style: Theme.of(context).textTheme.bodySmall,
          ),
        ),
      ),
    ];
  }

  List<Widget> agentPage() {
    final strings = context.strings;
    final root = clientOrigin
        ?.replace(pathSegments: ['agent', selectedAgent])
        .toString();
    final connection = Card.filled(
      color: colors.surfaceContainerLow,
      child: Padding(
        padding: const EdgeInsets.all(20),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            DropdownMenu<String>(
              trailingIcon: const Icon(CrabforgeSymbols.arrowDropDown),
              selectedTrailingIcon: const Icon(CrabforgeSymbols.arrowDropUp),
              label: Text(strings.clientPicker),
              expandedInsets: EdgeInsets.zero,
              enableFilter: true,
              requestFocusOnTap: true,
              menuHeight: 320,
              initialSelection: selectedAgent,
              leadingIcon: Center(
                widthFactor: 1,
                heightFactor: 1,
                child: IntegrationIcon.client(selectedAgent),
              ),
              dropdownMenuEntries: [
                for (final (id, name) in agents)
                  DropdownMenuEntry(
                    value: id,
                    label: name,
                    labelWidget: Text(name, softWrap: true),
                    leadingIcon: IntegrationIcon.client(id),
                  ),
              ],
              onSelected: (id) {
                if (id != null) setState(() => selectedAgent = id);
              },
            ),
            const SizedBox(height: 8),
            ListTile(
              contentPadding: EdgeInsets.zero,
              title: Text(strings.agentRoot),
              subtitle: root == null
                  ? Text(strings.clientAddressPending)
                  : SelectableText(root),
              trailing: IconButton(
                tooltip: strings.copyRoot,
                icon: const Icon(CrabforgeSymbols.contentCopy),
                onPressed: root == null
                    ? null
                    : () => copy(root, strings.rootCopied),
              ),
            ),
            if (root != null)
              Text(
                strings.agentHelp,
                style: Theme.of(context).textTheme.bodySmall,
              ),
          ],
        ),
      ),
    );
    final selectedActions = selectedClientActions();
    return [
      SliverToBoxAdapter(
        child: LayoutBuilder(
          builder: (context, bounds) {
            if (selectedActions.isEmpty) return connection;
            final tools = clientActionGroup(
              strings.clientSettings,
              selectedActions,
            );
            if (bounds.maxWidth < 740 ||
                MediaQuery.textScalerOf(context).scale(16) >= 22) {
              return Column(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [connection, const SizedBox(height: 12), tools],
              );
            }
            return IntrinsicHeight(
              child: Row(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  Expanded(child: connection),
                  const SizedBox(width: 16),
                  Expanded(child: tools),
                ],
              ),
            );
          },
        ),
      ),
      SliverToBoxAdapter(
        child: Padding(
          padding: const EdgeInsets.only(top: 16),
          child: clientActionGroup(strings.clientResources, [
            TextButton.icon(
              onPressed: () => showDialog<void>(
                context: context,
                builder: (_) => const MemoryLibrary(),
              ),
              icon: const Icon(CrabforgeSymbols.bookmarks),
              label: Text(context.strings.memoryLibrary),
            ),
            TextButton.icon(
              onPressed: () => showDialog<void>(
                context: context,
                builder: (_) => const AgentRules(),
              ),
              icon: const Icon(CrabforgeSymbols.rule),
              label: Text(context.strings.agentRules),
            ),
            TextButton.icon(
              onPressed: () => showDialog<void>(
                context: context,
                builder: (_) => const AgentSkills(),
              ),
              icon: const Icon(CrabforgeSymbols.extension),
              label: Text(context.strings.agentSkills),
            ),
            TextButton.icon(
              onPressed: () => showDialog<void>(
                context: context,
                builder: (_) => const AgentEnvironment(),
              ),
              icon: const Icon(CrabforgeSymbols.terminal),
              label: Text(context.strings.agentEnvironment),
            ),
            TextButton.icon(
              onPressed: () => showDialog<void>(
                context: context,
                builder: (_) => const AgentHooks(),
              ),
              icon: const Icon(CrabforgeSymbols.bolt),
              label: Text(context.strings.agentHooks),
            ),
            TextButton.icon(
              onPressed: () => showDialog<void>(
                context: context,
                builder: (_) => const ClaudePlugins(),
              ),
              icon: const IntegrationIcon.client('claude', size: 18),
              label: Text(context.strings.claudePlugins),
            ),
            TextButton.icon(
              onPressed: () => showDialog<void>(
                context: context,
                builder: (_) => const PluginBundles(),
              ),
              icon: const Icon(CrabforgeSymbols.inventory2),
              label: Text(context.strings.pluginBundles),
            ),
            TextButton.icon(
              onPressed: () => showDialog<void>(
                context: context,
                builder: (_) => const AgentFiles(subagents: true),
              ),
              icon: const Icon(CrabforgeSymbols.groupWork),
              label: Text(context.strings.agentSubagents),
            ),
            TextButton.icon(
              onPressed: () => showDialog<void>(
                context: context,
                builder: (_) => const AgentFiles(subagents: false),
              ),
              icon: const Icon(CrabforgeSymbols.shortcut),
              label: Text(context.strings.agentCommands),
            ),
            TextButton.icon(
              onPressed: () => showDialog<void>(
                context: context,
                builder: (_) => AgentSessions(
                  models: {
                    for (final provider in providers)
                      for (final model in provider['models'] as List)
                        '${provider['id']}/$model',
                    ...catalogGroupRefs.values,
                  }.toList()..sort(),
                ),
              ),
              icon: const Icon(CrabforgeSymbols.history),
              label: Text(context.strings.agentSessions),
            ),
          ]),
        ),
      ),
      SliverToBoxAdapter(child: heading(strings.supportedClients)),
      CollectionSliver.builder(
        grid: grid,
        height: 34,
        minWidth: 270,
        itemCount: agents.length,
        itemBuilder: (tileContext, index) {
          final (id, name) = agents[index];
          final selected = selectedAgent == id;
          return Card.filled(
            color: selected
                ? colors.secondaryContainer
                : colors.surfaceContainerLow,
            shape: RoundedRectangleBorder(
              borderRadius: BorderRadius.circular(18),
            ),
            clipBehavior: Clip.antiAlias,
            child: ListTile(
              key: ValueKey('client-$id'),
              selected: selected,
              selectedColor: colors.onSecondaryContainer,
              horizontalTitleGap: 12,
              contentPadding: const EdgeInsets.symmetric(
                horizontal: 14,
                vertical: 6,
              ),
              leading: CollectionGlyph(
                IntegrationIcon.client(id),
                highlighted: selected,
              ),
              title: Text(name, maxLines: 1, overflow: TextOverflow.ellipsis),
              subtitle: name == id
                  ? null
                  : Text(id, maxLines: 1, overflow: TextOverflow.ellipsis),
              trailing: selected
                  ? const Icon(CrabforgeSymbols.check, size: 20)
                  : null,
              onTap: () {
                setState(() => selectedAgent = id);
                Scrollable.of(tileContext).position.jumpTo(0);
              },
            ),
          );
        },
      ),
    ];
  }

  Widget clientActionGroup(String title, List<Widget> actions) => Card.filled(
    color: colors.surfaceContainerLow,
    child: Padding(
      padding: const EdgeInsets.fromLTRB(20, 16, 20, 16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Semantics(
            header: true,
            child: Text(title, style: Theme.of(context).textTheme.titleMedium),
          ),
          const SizedBox(height: 8),
          Wrap(spacing: 8, runSpacing: 8, children: actions),
        ],
      ),
    ),
  );

  List<Widget> selectedClientActions() => [
    if (projectAgents.contains(selectedAgent))
      OutlinedButton.icon(
        onPressed: () => showDialog<void>(
          context: context,
          barrierDismissible: false,
          builder: (_) => ProjectInstructionsEditor(agent: selectedAgent),
        ),
        icon: const Icon(CrabforgeSymbols.description),
        label: Text(context.strings.projectInstructions),
      ),
    if (projectMcpAgents.contains(selectedAgent))
      OutlinedButton.icon(
        onPressed: () => showDialog<void>(
          context: context,
          builder: (_) => ProjectMcp(agent: selectedAgent),
        ),
        icon: const Icon(CrabforgeSymbols.hub),
        label: Text(context.strings.projectMcp),
      ),
    if (selectedAgent == 'claude' || selectedAgent == 'codex')
      OutlinedButton.icon(
        onPressed: () => showDialog<void>(
          context: context,
          barrierDismissible: false,
          builder: (_) => ProjectInstructionsEditor(
            agent: selectedAgent,
            userInstructions: true,
          ),
        ),
        icon: const Icon(CrabforgeSymbols.person),
        label: Text(context.strings.agentInstructions),
      ),
    if (selectedAgent == 'omp') ...[
      OutlinedButton.icon(
        onPressed: busy ? null : () => act(() => chooseOmpProtocol(context)),
        icon: const Icon(CrabforgeSymbols.settingsEthernet),
        label: Text(context.strings.gatewayProtocol),
      ),
      OutlinedButton.icon(
        onPressed: busy ? null : () => act(() => editOmpRole(context)),
        icon: const Icon(CrabforgeSymbols.assignmentInd),
        label: Text(context.strings.roleModels),
      ),
      OutlinedButton.icon(
        onPressed: busy
            ? null
            : () => act(() async {
                final profile = await chooseOmpProfile(context);
                if (profile == null) return;
                final count = await api.syncOmpCatalog(profile: profile);
                if (mounted) {
                  notice('$profile · ${context.strings.models} ($count)');
                }
              }),
        icon: const Icon(CrabforgeSymbols.sync),
        label: Text(context.strings.syncModelCatalog),
      ),
      OutlinedButton.icon(
        onPressed: busy
            ? null
            : () => act(() => editOmpSettings(context, OmpEditor.advisor)),
        icon: const Icon(CrabforgeSymbols.settings),
        label: Text(context.strings.advisorSettings),
      ),
      OutlinedButton.icon(
        onPressed: busy
            ? null
            : () => act(() => editOmpSettings(context, OmpEditor.thinking)),
        icon: const Icon(CrabforgeSymbols.psychology),
        label: Text(context.strings.thinkingLevels),
      ),
    ],
  ];

  Widget gatewayTunnel() => Card.filled(
    child: ListTile(
      leading: Icon(
        status['tunnel'] == null
            ? CrabforgeSymbols.linkOff
            : CrabforgeSymbols.link,
      ),
      title: Text(context.strings.tunnel),
      subtitle: Text(
        status['tunnel'] == null
            ? context.strings.disconnected
            : 'http://${status['tunnel']}/v1',
      ),
      trailing: TextButton(
        onPressed: busy
            ? null
            : () async {
                if (status['tunnel'] != null) {
                  await act(() async {
                    await api.disconnectPeer();
                    final updated = await api.status();
                    if (mounted) setState(() => applyStatus(updated));
                  });
                  return;
                }
                final address = await showDialog<(String, String, String)>(
                  context: context,
                  builder: (_) => const TunnelEditor(),
                );
                if (address != null && mounted) {
                  await act(() async {
                    await api.connectPeer(
                      remote: address.$1,
                      listen: address.$2,
                      relayFallback: address.$3.isEmpty ? null : address.$3,
                    );
                    final updated = await api.status();
                    if (mounted) setState(() => applyStatus(updated));
                  });
                }
              },
        child: Text(
          status['tunnel'] == null
              ? context.strings.connect
              : context.strings.disconnect,
        ),
      ),
    ),
  );
}

class ModelCatalog extends StatefulWidget {
  const ModelCatalog({
    super.key,
    required this.providers,
    required this.groupRefs,
    required this.canEdit,
    required this.onEdit,
    required this.onGroup,
    required this.grid,
    required this.onToggle,
  });
  final List<JsonMap> providers;
  final Map<String, String> groupRefs;
  final bool canEdit;
  final bool grid;
  final VoidCallback? onToggle;
  final ValueChanged<JsonMap> onEdit;
  final ValueChanged<String> onGroup;
  @override
  State<ModelCatalog> createState() => _ModelCatalogState();
}

typedef _CatalogEntry = ({
  String id,
  String search,
  JsonMap? provider,
  String? group,
});

class _ModelCatalogState extends State<ModelCatalog> {
  String filter = '';
  late List<_CatalogEntry> entries;
  late List<_CatalogEntry> rows;

  @override
  void initState() {
    super.initState();
    indexModels();
  }

  @override
  void didUpdateWidget(ModelCatalog oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (!identical(oldWidget.providers, widget.providers) ||
        !identical(oldWidget.groupRefs, widget.groupRefs)) {
      indexModels();
    }
  }

  void indexModels() {
    entries = [
      for (final provider in widget.providers)
        for (final model in availableModels(provider))
          catalogEntry('${provider['id']}/$model', provider: provider),
      for (final group in widget.groupRefs.entries)
        catalogEntry(group.value, group: group.key),
    ];
    filterModels();
  }

  _CatalogEntry catalogEntry(String id, {JsonMap? provider, String? group}) =>
      (id: id, search: id.toLowerCase(), provider: provider, group: group);

  void filterModels() {
    rows = filter.isEmpty
        ? entries
        : entries.where((entry) => entry.search.contains(filter)).toList();
  }

  @override
  Widget build(BuildContext context) {
    final strings = context.strings;
    final grid = widget.grid;
    final material = MaterialLocalizations.of(context);
    return Padding(
      padding: EdgeInsets.symmetric(
        horizontal: MediaQuery.sizeOf(context).width >= workspaceWideBreakpoint
            ? 38
            : 18,
        vertical: 28,
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          WorkspaceHeader(
            title: strings.models,
            actions: [LayoutToggle(grid: grid, onPressed: widget.onToggle)],
          ),
          SearchBar(
            hintText: material.searchFieldLabel,
            leading: const Icon(CrabforgeSymbols.search),
            onChanged: (value) {
              final query = value.trim().toLowerCase();
              if (query == filter) return;
              setState(() {
                filter = query;
                filterModels();
              });
            },
          ),
          const SizedBox(height: 12),
          if (rows.isEmpty)
            _EmptyCatalog(
              widget.providers.isEmpty
                  ? strings.providerHelp
                  : strings.noModelsFound,
            ),
          Expanded(
            child: LayoutBuilder(
              builder: (context, bounds) {
                Widget item(BuildContext context, int i) {
                  final row = rows[i];
                  final group = row.group;
                  final provider = row.provider;
                  final icon = group != null
                      ? const Icon(CrabforgeSymbols.accountTree)
                      : provider == null
                      ? const Icon(CrabforgeSymbols.memory)
                      : IntegrationIcon.provider(
                          provider['web_site'] ?? provider['id'],
                          web: provider['web_site'] != null,
                          remote: provider['remote_gateway'] == true,
                        );
                  final actions = [
                    if (group != null ||
                        (provider != null && provider['web_site'] == null))
                      IconButton(
                        tooltip: group != null
                            ? strings.editGroup
                            : strings.editProvider,
                        onPressed: widget.canEdit
                            ? () => group != null
                                  ? widget.onGroup(group)
                                  : widget.onEdit(provider!)
                            : null,
                        icon: const Icon(CrabforgeSymbols.edit),
                      ),
                    IconButton(
                      tooltip: material.copyButtonLabel,
                      onPressed: () async {
                        await Clipboard.setData(ClipboardData(text: row.id));
                        if (context.mounted) {
                          ScaffoldMessenger.of(context).showSnackBar(
                            SnackBar(content: Text(strings.modelCopied)),
                          );
                        }
                      },
                      icon: const Icon(CrabforgeSymbols.contentCopy),
                    ),
                  ];
                  return CollectionTile(
                    grid: grid,
                    leading: icon,
                    title: grid
                        ? Tooltip(
                            message: row.id,
                            child: Text(
                              row.id,
                              maxLines: 2,
                              overflow: TextOverflow.ellipsis,
                            ),
                          )
                        : SelectableText(row.id),
                    actions: actions,
                  );
                }

                if (!grid) {
                  return ListView.separated(
                    key: const PageStorageKey('model-list'),
                    itemCount: rows.length,
                    itemBuilder: item,
                    separatorBuilder: (_, _) => const SizedBox(height: 6),
                  );
                }
                return GridView.builder(
                  key: const PageStorageKey('model-grid'),
                  itemCount: rows.length,
                  itemBuilder: item,
                  gridDelegate: SliverGridDelegateWithFixedCrossAxisCount(
                    crossAxisCount: (bounds.maxWidth / 240).floor().clamp(1, 4),
                    mainAxisSpacing: 12,
                    crossAxisSpacing: 12,
                    mainAxisExtent:
                        90 + MediaQuery.textScalerOf(context).scale(38),
                  ),
                );
              },
            ),
          ),
        ],
      ),
    );
  }
}

class _EmptyCatalog extends StatelessWidget {
  const _EmptyCatalog(this.message);
  final String message;

  @override
  Widget build(BuildContext context) => Card.filled(
    child: ListTile(
      leading: const Icon(CrabforgeSymbols.searchOff),
      title: Text(message),
    ),
  );
}
