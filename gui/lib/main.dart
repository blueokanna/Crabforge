import 'dart:async';
import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import 'brand_icon.dart';
import 'collection_view.dart';
import 'editors.dart';
import 'language.dart';
import 'omp_settings.dart';
import 'l10n/app_localizations.dart';
import 'src/rust/api.dart' as api;
import 'src/rust/frb_generated.dart';

void main() {
  WidgetsFlutterBinding.ensureInitialized();
  runApp(const CrabforgeApp());
}

class CrabforgeApp extends StatefulWidget {
  const CrabforgeApp({super.key});
  @override
  State<CrabforgeApp> createState() => _CrabforgeAppState();
}

class _CrabforgeAppState extends State<CrabforgeApp> {
  String language = '';
  bool grid = false, writingLayout = false;
  final messenger = GlobalKey<ScaffoldMessengerState>();

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

  ThemeData theme(Brightness b) {
    final palette = ColorScheme.fromSeed(
      seedColor: const Color(0xffac4624),
      brightness: b,
      dynamicSchemeVariant: DynamicSchemeVariant.expressive,
    );
    return ThemeData(
      useMaterial3: true,
      fontFamily: 'packages/crabforge_assets/Google Sans',
      fontFamilyFallback: const [
        'packages/crabforge_assets/HarmonyOS Sans SC',
        'packages/crabforge_assets/Noto Sans KR',
      ],
      colorScheme: palette,
      scaffoldBackgroundColor: palette.surface,
      cardTheme: CardThemeData(
        elevation: 0,
        color: palette.surfaceContainerLow,
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(22)),
      ),
      inputDecorationTheme: const InputDecorationTheme(
        filled: true,
        border: OutlineInputBorder(),
      ),
    );
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
    theme: theme(Brightness.light),
    darkTheme: theme(Brightness.dark),
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
    ),
  );
}

class Console extends StatefulWidget {
  const Console({
    super.key,
    required this.language,
    required this.onLanguage,
    required this.onLayoutLoaded,
  });
  final String language;
  final ValueChanged<String> onLanguage;
  final ValueChanged<bool> onLayoutLoaded;
  @override
  State<Console> createState() => _ConsoleState();
}

class _ConsoleState extends State<Console> with WidgetsBindingObserver {
  int page = 0;
  String? snapshot, error;
  JsonMap settings = {}, status = {};
  List<JsonMap> presets = [];
  List<(String, String)> agents = [];
  String selectedAgent = 'claude';
  bool busy = false, reading = false, initialized = false;
  bool get grid => CollectionLayout.of(context).grid;
  Timer? timer;
  late String peerId;
  bool get running => status['running'] == true;
  List<JsonMap> get providers =>
      (settings['providers'] as List).cast<JsonMap>();
  JsonMap groups = {};
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
    if ((manualGroups && visible) ||
        (lifecycle == AppLifecycleState.resumed &&
            ((running && (page == 1 || page == 3)) ||
                (page == 3 && status['tunnel'] != null)))) {
      timer = Timer.periodic(const Duration(seconds: 3), (_) => refresh());
    }
  }

  Future<void> refresh() async {
    if (reading || busy) return;
    reading = true;
    try {
      final previous = snapshot;
      final data = jsonDecode(await api.status());
      // Both the CLI and this window can change a manual selection while serving.
      if (running && page == 2 && groupPicks.isNotEmpty) {
        final saved = await api.configuration();
        if (saved != previous) {
          final resolved = jsonDecode(
            await api.routeGroups(configuration: saved),
          );
          if (mounted && page == 2 && !busy && snapshot == previous) {
            setState(() => applyConfiguration(saved, resolved));
          }
        }
      }
      if (mounted && !busy) {
        setState(() => status = data);
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
      ]);
      final resolved = jsonDecode(
        await api.routeGroups(configuration: values[0]),
      ) as JsonMap;
      if (!mounted) return;
      widget.onLanguage(values[5]);
      widget.onLayoutLoaded(values[6] == 'grid');
      setState(() {
        applyConfiguration(values[0], resolved);
        presets = (jsonDecode(values[1]) as List).cast<JsonMap>();
        status = jsonDecode(values[2]);
        peerId = values[3];
        agents = [
          for (final item in jsonDecode(values[4]))
            (item[0] as String, item[1] as String),
        ];
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
    final resolved =
        jsonDecode(await api.routeGroups(configuration: s)) as JsonMap;
    if (mounted) {
      setState(() => applyConfiguration(s, resolved));
    }
  }

  void applyConfiguration(String saved, JsonMap resolved) {
    snapshot = saved;
    settings = jsonDecode(saved);
    expandedGroups = resolved['expanded'] ?? {};
    groups = {
      for (final name in resolved['order']) name: resolved['members'][name],
    };
    publicGroupRefs = Map<String, String>.from(resolved['public_refs']);
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
      (context.strings.providers, Icons.vpn_key_outlined, Icons.vpn_key),
      (context.strings.routes, Icons.alt_route_outlined, Icons.alt_route),
      (context.strings.groups, Icons.account_tree_outlined, Icons.account_tree),
      (context.strings.gateway, Icons.dns_outlined, Icons.dns),
      (context.strings.models, Icons.view_list_outlined, Icons.view_list),
      (context.strings.clients, Icons.devices_outlined, Icons.devices),
    ];
    final wide = MediaQuery.sizeOf(context).width >= 820;
    final ready = snapshot != null && error == null;
    final body = ready && page == 4
        ? ModelCatalog(
            grid: grid,
            onToggle: busy ? null : CollectionLayout.of(context).toggle,
            providers: providers,
            groupRefs: {
              for (final g in groups.keys)
                g: publicGroupRefs['group/$g'] ?? 'group/$g',
            },
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
                  horizontal: wide ? 36 : 16,
                  vertical: 24,
                ),
                sliver: SliverMainAxisGroup(
                  slivers: [
                    SliverList.list(
                      children: [
                        Row(
                          children: [
                            Expanded(
                              child: Text(
                                pages[page].$1,
                                style: Theme.of(context).textTheme.headlineLarge
                                    ?.copyWith(fontWeight: FontWeight.w700),
                              ),
                            ),
                            LayoutToggle(
                              grid: grid,
                              onPressed: busy
                                  ? null
                                  : CollectionLayout.of(context).toggle,
                            ),
                            if (page == 0 || page == 2)
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
                                icon: const Icon(Icons.add),
                                label: Text(
                                  page == 0
                                      ? context.strings.addProvider
                                      : context.strings.newGroup,
                                ),
                              ),
                          ],
                        ),
                        const SizedBox(height: 18),
                        if (running && (page == 0 || page == 2))
                          note(context.strings.stopToEdit, Icons.lock_outline),
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
    return Scaffold(
      appBar: AppBar(
        leading: Padding(
          padding: const EdgeInsets.all(9),
          child: Image.asset(
            'crabforge.png',
            package: 'crabforge_assets',
            excludeFromSemantics: true,
          ),
        ),
        title: const Text(
          'Crabforge',
          style: TextStyle(fontWeight: FontWeight.bold, letterSpacing: -0.8),
        ),
        actions: [
          PopupMenuButton<String>(
            key: const ValueKey('language-picker'),
            enabled: initialized && !busy,
            tooltip: context.strings.language,
            initialValue: widget.language,
            icon: const Icon(Icons.language),
            onSelected: (code) => act(() async {
              await api.setUiLanguage(language: code);
              if (mounted) widget.onLanguage(code);
            }),
            itemBuilder: (_) => [
              PopupMenuItem(
                value: '',
                child: Text(context.strings.systemLanguage),
              ),
              for (final e in languageNames.entries)
                CheckedPopupMenuItem(
                  value: e.key,
                  checked: widget.language == e.key,
                  child: Text(e.value),
                ),
            ],
          ),
          if (ready)
            IconButton(
              tooltip: context.strings.reloadConfig,
              onPressed: busy ? null : load,
              icon: const Icon(Icons.refresh),
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
                    selectedIcon: Icon(p.$3),
                    label: p.$1,
                  ),
              ],
            ),
      body: SafeArea(
        child: Row(
          children: [
            if (wide)
              NavigationRail(
                extended: true,
                scrollable: true,
                minExtendedWidth: 182,
                backgroundColor: colors.surfaceContainerLow,
                selectedIndex: page,
                onDestinationSelected: navigate,
                destinations: [
                  for (final p in pages)
                    NavigationRailDestination(
                      icon: Icon(p.$2),
                      selectedIcon: Icon(p.$3),
                      label: Text(p.$1),
                    ),
                ],
              ),
            Expanded(
              child: Center(
                child: ConstrainedBox(
                  constraints: const BoxConstraints(maxWidth: 1080),
                  child: AnimatedSwitcher(
                    duration: MediaQuery.disableAnimationsOf(context)
                        ? Duration.zero
                        : Durations.short3,
                    child: body,
                  ),
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget note(String text, IconData icon) => Card.filled(
    child: Padding(
      padding: const EdgeInsets.all(20),
      child: Row(
        children: [
          Icon(icon, color: colors.primary),
          const SizedBox(width: 16),
          Expanded(child: Text(text)),
        ],
      ),
    ),
  );
  Widget heading(String t) => Padding(
    padding: const EdgeInsets.fromLTRB(6, 22, 4, 8),
    child: Text(
      t,
      style: Theme.of(context).textTheme.titleMedium
          ?.copyWith(fontWeight: FontWeight.bold),
    ),
  );
  Widget heroPanel(IconData icon, String title, String detail) => Card.filled(
    color: colors.secondaryContainer,
    shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(32)),
    child: Padding(
      padding: const EdgeInsets.all(24),
      child: Row(
        children: [
          CircleAvatar(
            radius: 26,
            backgroundColor: colors.secondary,
            foregroundColor: colors.onSecondary,
            child: Icon(icon),
          ),
          const SizedBox(width: 18),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  title,
                  style: Theme.of(context).textTheme.headlineSmall?.copyWith(
                    color: colors.onSecondaryContainer,
                    fontWeight: FontWeight.w700,
                  ),
                ),
                Text(
                  detail,
                  style: TextStyle(color: colors.onSecondaryContainer),
                ),
              ],
            ),
          ),
        ],
      ),
    ),
  );

  List<Widget> providerPage() => [
    SliverList.list(
      children: [
        heroPanel(
          Icons.hub_outlined,
          context.strings.providerCount(providers.length),
          context.strings.presetCount(presets.length),
        ),
        if (providers.isEmpty) note(context.strings.providerHelp, Icons.link),
      ],
    ),
    CollectionSliver(
      grid: grid,
      height: 118,
      children: [
        for (final p in providers)
          CollectionTile(
            grid: grid,
            title: collectionTitle(p['id']),
            detail: Text((p['models'] as List).join(' · ')),
            leading: BrandIcon.provider(
              p['web_site'] == null ? p['id'] : '${p['web_site']}-web',
            ),
            onTap: busy || running
                ? null
                : () => p['web_site'] == null
                      ? provider(p)
                      : login(p['web_site']),
            actions: [
              IconButton(
                tooltip: context.strings.renameProvider,
                icon: const Icon(Icons.drive_file_rename_outline),
                onPressed: busy || running
                    ? null
                    : () => renameProvider(p['id']),
              ),
              IconButton(
                tooltip: context.strings.removeProvider,
                onPressed: busy || running
                    ? null
                    : () async {
                        if (await confirm(p['id']) && mounted) {
                          await save({
                            ...settings,
                            'providers': providers
                                .where((i) => i != p)
                                .toList(),
                          });
                        }
                      },
                icon: const Icon(Icons.delete_outline),
              ),
            ],
          ),
      ],
    ),
    SliverList.list(
      children: [
        heading(context.strings.webProviders),
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 6),
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
                icon: BrandIcon.provider('${s.$2}-web', size: 18),
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
            icon: const Icon(Icons.file_download_outlined),
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
          Icons.account_tree_outlined,
        ),
      ),
    CollectionSliver(
      grid: grid,
      height: 166,
      children: [
        for (final e in groups.entries)
          CollectionTile(
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
                              settings['group_off']?[e.key]?.contains(m) !=
                              true,
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
                          ? Icons.toggle_off_outlined
                          : Icons.toggle_on,
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
                : const Icon(Icons.account_tree),
            onTap: busy || running ? null : () => group(e.key),
            actions: [
              IconButton(
                tooltip: WidgetsLocalizations.of(context).reorderItemUp,
                onPressed: busy || running || e.key == groups.keys.first
                    ? null
                    : () => moveGroup(e.key, -1),
                icon: const Icon(Icons.arrow_upward),
              ),
              IconButton(
                tooltip: WidgetsLocalizations.of(context).reorderItemDown,
                onPressed: busy || running || e.key == groups.keys.last
                    ? null
                    : () => moveGroup(e.key, 1),
                icon: const Icon(Icons.arrow_downward),
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
                icon: const Icon(Icons.copy_outlined),
              ),
              IconButton(
                tooltip: context.strings.shareGroup,
                onPressed: busy ? null : () => act(() => shareGroup(e.key)),
                icon: const Icon(Icons.share_outlined),
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
                icon: const Icon(Icons.delete_outline),
              ),
            ],
          ),
      ],
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
    final attempts = status['attempts'] as List;
    return [
      SliverList.list(
        children: [
          heroPanel(
            Icons.alt_route,
            context.strings.recentAttempts(attempts.length),
            context.strings.historyLimit,
          ),
          if (attempts.isEmpty)
            note(
              running ? context.strings.waiting : context.strings.historyHelp,
              Icons.timeline,
            ),
        ],
      ),
      CollectionSliver(
        grid: grid,
        height: 110,
        children: [
          for (final a in attempts)
            CollectionTile(
              grid: grid,
              leading: Icon(
                a['status'] < 400
                    ? Icons.check_circle_outline
                    : Icons.error_outline,
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
            ),
        ],
      ),
    ];
  }

  List<Widget> gatewayPage() {
    final addr = status['address'] ?? settings['listen'];
    return [
      SliverList.list(
        children: [
          heroPanel(
            Icons.dns,
            running ? context.strings.running : context.strings.stopped,
            'http://$addr/v1',
          ),
          const SizedBox(height: 12),
          Wrap(
            spacing: 8,
            runSpacing: 8,
            children: [
              FilledButton.icon(
                onPressed: busy
                    ? null
                    : () => act(() async {
                        await api.setRunning(on_: !running);
                        final s = jsonDecode(await api.status());
                        if (mounted) setState(() => status = s);
                      }),
                icon: Icon(running ? Icons.stop : Icons.play_arrow),
                label: Text(
                  running ? context.strings.stop : context.strings.start,
                ),
              ),
              OutlinedButton.icon(
                onPressed: busy
                    ? null
                    : () => act(() async {
                        final key = await api.gatewayKey();
                        if (mounted) await copy(key, context.strings.keyCopied);
                      }),
                icon: const Icon(Icons.key),
                label: Text(context.strings.copyKey),
              ),
              TextButton.icon(
                onPressed: busy || running ? null : network,
                icon: const Icon(Icons.tune),
                label: Text(context.strings.networkSettings),
              ),
            ],
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
            title: collectionTitle(context.strings.localPeer),
            detail: SelectableText(peerId),
            actions: [
              IconButton(
                tooltip: context.strings.copyPeer,
                onPressed: () => copy(peerId, context.strings.peerCopied),
                icon: const Icon(Icons.copy),
              ),
            ],
          ),
          CollectionTile(
            grid: grid,
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
            title: collectionTitle(context.strings.trustedPeers),
            detail: Text('${(settings['allowed_peers'] as List).length}'),
          ),
          Card.filled(
            child: ListTile(
              leading: const Icon(Icons.devices),
              title: Text(context.strings.clients),
              trailing: const Icon(Icons.chevron_right),
              onTap: () => navigate(5),
            ),
          ),
          ...gatewayConnections(addr),
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
    final addr = status['address'] ?? settings['listen'];
    return [
      SliverToBoxAdapter(
        child: Card.filled(
          child: Padding(
            padding: const EdgeInsets.all(16),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                DropdownMenu<String>(
                  label: const Text('Agent'),
                  expandedInsets: EdgeInsets.zero,
                  enableFilter: true,
                  menuHeight: 240,
                  initialSelection: selectedAgent,
                  leadingIcon: BrandIcon.agent(selectedAgent),
                  dropdownMenuEntries: [
                    for (final (id, name) in agents)
                      DropdownMenuEntry(
                        value: id,
                        label: name,
                        leadingIcon: BrandIcon.agent(id),
                      ),
                  ],
                  onSelected: (v) {
                    if (v != null) setState(() => selectedAgent = v);
                  },
                ),
                ListTile(
                  contentPadding: EdgeInsets.zero,
                  title: Text(context.strings.agentRoot),
                  subtitle: SelectableText('http://$addr/agent/$selectedAgent'),
                  trailing: IconButton(
                    tooltip: context.strings.copyRoot,
                    icon: const Icon(Icons.copy),
                    onPressed: () => copy(
                      'http://$addr/agent/$selectedAgent',
                      context.strings.rootCopied,
                    ),
                  ),
                ),
                Text(context.strings.agentHelp),
                if (selectedAgent == 'omp') ...[
                  TextButton.icon(
                    onPressed: busy
                        ? null
                        : () => act(() => chooseOmpProtocol(context)),
                    icon: const Icon(Icons.settings_ethernet),
                    label: Text(context.strings.gatewayProtocol),
                  ),
                  TextButton.icon(
                    onPressed: busy
                        ? null
                        : () => act(() => editOmpRole(context)),
                    icon: const Icon(Icons.assignment_ind_outlined),
                    label: Text(context.strings.roleModels),
                  ),
                  TextButton.icon(
                    onPressed: busy
                        ? null
                        : () => act(() async {
                            final profile = await chooseOmpProfile(context);
                            if (profile == null) return;
                            final count = await api.syncOmpCatalog(
                              profile: profile,
                            );
                            if (mounted) {
                              notice(
                                '$profile · ${context.strings.models} ($count)',
                              );
                            }
                          }),
                    icon: const Icon(Icons.sync),
                    label: Text(context.strings.syncModelCatalog),
                  ),
                  TextButton.icon(
                    onPressed: busy
                        ? null
                        : () => act(
                            () => editOmpSettings(context, OmpEditor.advisor),
                          ),
                    icon: const Icon(Icons.settings_outlined),
                    label: Text(context.strings.advisorSettings),
                  ),
                  TextButton.icon(
                    onPressed: busy
                        ? null
                        : () => act(
                            () => editOmpSettings(context, OmpEditor.thinking),
                          ),
                    icon: const Icon(Icons.psychology_outlined),
                    label: Text(context.strings.thinkingLevels),
                  ),
                ],
              ],
            ),
          ),
        ),
      ),
      CollectionSliver(
        grid: grid,
        height: 40,
        children: [
          for (final (id, name) in agents)
            Builder(
              builder: (tileContext) => Card.filled(
                child: ListTile(
                  selected: selectedAgent == id,
                  leading: BrandIcon.agent(id),
                  title: Text(
                    name,
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                  ),
                  subtitle: name == id
                      ? null
                      : Text(id, maxLines: 1, overflow: TextOverflow.ellipsis),
                  onTap: () {
                    setState(() => selectedAgent = id);
                    Scrollable.of(tileContext).position.jumpTo(0);
                  },
                ),
              ),
            ),
        ],
      ),
    ];
  }

  List<Widget> gatewayConnections(String addr) => [
    Card.filled(
      child: ListTile(
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
                      final updated = jsonDecode(await api.status());
                      if (mounted) setState(() => status = updated);
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
                      final updated = jsonDecode(await api.status());
                      if (mounted) setState(() => status = updated);
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
    ),
    Card.filled(
      child: ListTile(
        title: Text(context.strings.apiBase),
        subtitle: SelectableText('http://$addr/v1'),
        trailing: IconButton(
          tooltip: context.strings.copyBase,
          onPressed: () => copy('http://$addr/v1', context.strings.baseCopied),
          icon: const Icon(Icons.copy),
        ),
      ),
    ),
  ];
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

class _ModelCatalogState extends State<ModelCatalog> {
  String filter = '';
  @override
  Widget build(BuildContext context) {
    final strings = context.strings;
    final grid = widget.grid;
    final material = MaterialLocalizations.of(context);
    final rows = <({String id, bool group, VoidCallback? edit})>[
      for (final provider in widget.providers)
        for (final model in provider['models'] as List)
          (
            id: '${provider['id']}/$model',
            group: false,
            edit: provider['web_site'] == null
                ? () => widget.onEdit(provider)
                : null,
          ),
      for (final entry in widget.groupRefs.entries)
        (id: entry.value, group: true, edit: () => widget.onGroup(entry.key)),
    ]..removeWhere((r) => !r.id.toLowerCase().contains(filter));
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 20),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Expanded(
                child: Text(
                  strings.models,
                  style: Theme.of(context).textTheme.headlineLarge
                      ?.copyWith(fontWeight: FontWeight.w700),
                ),
              ),
              LayoutToggle(grid: grid, onPressed: widget.onToggle),
            ],
          ),
          const SizedBox(height: 16),
          TextField(
            decoration: InputDecoration(
              labelText: material.searchFieldLabel,
              prefixIcon: const Icon(Icons.search),
            ),
            onChanged: (value) =>
                setState(() => filter = value.trim().toLowerCase()),
          ),
          const SizedBox(height: 12),
          if (rows.isEmpty)
            Text(
              widget.providers.isEmpty
                  ? strings.providerHelp
                  : strings.noModelsFound,
            ),
          Expanded(
            child: LayoutBuilder(
              builder: (context, bounds) {
                Widget item(BuildContext context, int i) {
                  final row = rows[i];
                  final brand = row.group
                      ? const Icon(Icons.account_tree_outlined)
                      : BrandIcon.model(row.id);
                  final actions = Row(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      if (row.edit != null)
                        IconButton(
                          tooltip: row.group
                              ? strings.editGroup
                              : strings.editProvider,
                          onPressed: widget.canEdit ? row.edit : null,
                          icon: const Icon(Icons.edit_outlined),
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
                        icon: const Icon(Icons.copy),
                      ),
                    ],
                  );
                  if (!grid) {
                    return ListTile(
                      leading: brand,
                      title: SelectableText(row.id),
                      trailing: actions,
                    );
                  }
                  return Card.filled(
                    child: Padding(
                      padding: const EdgeInsets.all(12),
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Row(children: [brand, const Spacer(), actions]),
                          const SizedBox(height: 6),
                          Expanded(
                            child: Tooltip(
                              message: row.id,
                              child: Text(
                                row.id,
                                maxLines: 2,
                                overflow: TextOverflow.ellipsis,
                              ),
                            ),
                          ),
                        ],
                      ),
                    ),
                  );
                }

                if (!grid) {
                  return ListView.builder(
                    key: const PageStorageKey('model-list'),
                    itemCount: rows.length,
                    itemBuilder: item,
                  );
                }
                return GridView.builder(
                  key: const PageStorageKey('model-grid'),
                  itemCount: rows.length,
                  itemBuilder: item,
                  gridDelegate: SliverGridDelegateWithFixedCrossAxisCount(
                    crossAxisCount: (bounds.maxWidth / 240).floor().clamp(1, 4),
                    mainAxisSpacing: 4,
                    crossAxisSpacing: 4,
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
