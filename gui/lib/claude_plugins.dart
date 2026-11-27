import 'dart:convert';

import 'package:flutter/material.dart';

import 'material_symbols.dart';
import 'collection_view.dart';
import 'language.dart';
import 'project_picker.dart';
import 'src/rust/api.dart' as api;

typedef _Json = Map<String, dynamic>;

class ClaudePlugins extends StatefulWidget {
  const ClaudePlugins({super.key});

  @override
  State<ClaudePlugins> createState() => _ClaudePluginsState();
}

class _ClaudePluginsState extends State<ClaudePlugins> {
  _Json _view = const {};
  String? _project;
  Object? _error;
  bool _busy = false;

  @override
  void initState() {
    super.initState();
    _reload();
  }

  Future<void> _load(Future<String> Function() run) async {
    setState(() {
      _busy = true;
      _error = null;
    });
    try {
      final view = jsonDecode(await run()) as _Json;
      if (mounted) setState(() => _view = view);
    } catch (error) {
      if (mounted) setState(() => _error = error);
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  Future<void> _reload() => _load(() => api.claudePlugins(project: _project));

  void _select(String? path) {
    setState(() {
      _project = path;
      _view = const {};
    });
    _reload();
  }

  Future<void> _chooseProject() async {
    final path = await openProjectFolder(context, _project);
    if (mounted && path != null) _select(path);
  }

  Future<void> _set(String id, bool enabled) => _load(
    () => api.claudePlugins(
      project: _project,
      id: id,
      revision: _view['revision'] as String,
      enabled: enabled,
    ),
  );

  Future<void> _manage(String action, String target) =>
      _load(() => api.manageClaudePlugin(action: action, target: target));

  Future<void> _prompt(
    String title,
    String label,
    Future<void> Function(String) apply, {
    bool multiline = false,
  }) async {
    var value = '';
    final target = await showDialog<String>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(title),
        content: TextField(
          autofocus: true,
          minLines: multiline ? 4 : 1,
          maxLines: multiline ? 9 : 1,
          decoration: InputDecoration(labelText: label),
          onChanged: (text) => value = text.trim(),
          onSubmitted: multiline
              ? null
              : (text) => Navigator.pop(context, text.trim()),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: Text(context.strings.cancel),
          ),
          FilledButton(
            onPressed: () => Navigator.pop(context, value),
            child: Text(title),
          ),
        ],
      ),
    );
    if (mounted && target != null && target.isNotEmpty) {
      await apply(target);
    }
  }

  Future<void> _browse() async {
    final catalog = api.availableClaudePlugins().then(
      (source) => (jsonDecode(source) as List).cast<_Json>(),
    );
    final id = await showDialog<String>(
      context: context,
      builder: (context) => AlertDialog(
        title: EditorHeading(context.strings.pluginCatalog),
        content: SizedBox(
          width: 650,
          height: 400,
          child: FutureBuilder<List<_Json>>(
            future: catalog,
            builder: (context, snapshot) {
              if (snapshot.hasError) return SelectableText('${snapshot.error}');
              final entries = snapshot.data;
              if (entries == null) {
                return const Center(child: CircularProgressIndicator());
              }
              if (entries.isEmpty) {
                return Text(context.strings.noAvailablePlugins);
              }
              final grid = CollectionLayout.of(context).grid;
              return CustomScrollView(
                slivers: [
                  CollectionSliver.builder(
                    grid: grid,
                    height: 145,
                    itemCount: entries.length,
                    itemBuilder: (context, index) {
                      final entry = entries[index];
                      final id = entry['id'] as String;
                      return CollectionTile(
                        grid: grid,
                        title: Text(
                          id,
                          maxLines: 2,
                          overflow: TextOverflow.ellipsis,
                        ),
                        detail: Text(
                          entry['description'] as String,
                          maxLines: 2,
                          overflow: TextOverflow.ellipsis,
                        ),
                        actions: [
                          IconButton(
                            tooltip: context.strings.installPlugin,
                            onPressed: () => Navigator.pop(context, id),
                            icon: const Icon(CrabforgeSymbols.download),
                          ),
                        ],
                      );
                    },
                  ),
                ],
              );
            },
          ),
        ),
        actions: [_close(context)],
      ),
    );
    if (mounted && id != null) await _manage('install', id);
  }

  Future<void> _export() async {
    final export = api.exportClaudePluginSettings().then(
      (source) =>
          const JsonEncoder.withIndent('  ').convert(jsonDecode(source)),
    );
    await showDialog<void>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(context.strings.exportPluginSettings),
        content: SizedBox(
          width: 560,
          child: FutureBuilder<String>(
            future: export,
            builder: (context, snapshot) =>
                snapshot.data != null || snapshot.hasError
                ? SingleChildScrollView(
                    child: SelectableText(snapshot.data ?? '${snapshot.error}'),
                  )
                : const LinearProgressIndicator(),
          ),
        ),
        actions: [_close(context)],
      ),
    );
  }

  Widget _close(BuildContext context) => TextButton(
    onPressed: () => Navigator.pop(context),
    child: Text(MaterialLocalizations.of(context).closeButtonLabel),
  );

  Future<void> _remove(String action, String target) async {
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(context.strings.confirmRemove(target)),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: Text(context.strings.cancel),
          ),
          FilledButton(
            onPressed: () => Navigator.pop(context, true),
            child: Text(context.strings.remove),
          ),
        ],
      ),
    );
    if (mounted && confirmed == true) await _manage(action, target);
  }

  @override
  Widget build(BuildContext context) {
    final strings = context.strings;
    final grid = CollectionLayout.of(context).grid;
    final project = _project;
    final plugins = (_view['plugins'] as List? ?? const []).cast<_Json>();
    final marketplaces = (_view['marketplaces'] as List? ?? const [])
        .cast<String>();
    return Dialog(
      key: const ValueKey('claude-plugins'),
      child: SizedBox(
        width: 800,
        height: 580,
        child: Padding(
          padding: const EdgeInsets.all(20),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              DefaultTextStyle(
                style: Theme.of(context).textTheme.titleLarge!,
                child: EditorHeading(strings.claudePlugins),
              ),
              Wrap(
                spacing: 8,
                children: [
                  ChoiceChip(
                    label: Text(strings.resourceUserScope),
                    selected: project == null,
                    onSelected: _busy || project == null
                        ? null
                        : (_) => _select(null),
                  ),
                  ActionChip(
                    avatar: const Icon(CrabforgeSymbols.folderOpen),
                    label: Text(strings.openProject),
                    onPressed: _busy ? null : _chooseProject,
                  ),
                  IconButton(
                    onPressed: _busy ? null : _reload,
                    icon: const Icon(CrabforgeSymbols.refresh),
                    tooltip: strings.reload,
                  ),
                ],
              ),
              if (project != null)
                Tooltip(
                  message: project,
                  child: Text(
                    project,
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                  ),
                ),
              if (project == null)
                Wrap(
                  children: [
                    TextButton.icon(
                      onPressed: _busy
                          ? null
                          : () => _prompt(
                              strings.installPlugin,
                              'name@marketplace',
                              (id) => _manage('install', id),
                            ),
                      icon: const Icon(CrabforgeSymbols.download),
                      label: Text(strings.installPlugin),
                    ),
                    TextButton.icon(
                      onPressed: _busy
                          ? null
                          : () => _prompt(
                              strings.addPluginMarketplace,
                              strings.pluginSource,
                              (source) => _manage('add-marketplace', source),
                            ),
                      icon: const Icon(CrabforgeSymbols.add),
                      label: Text(strings.addPluginMarketplace),
                    ),
                    TextButton.icon(
                      onPressed: _busy ? null : _browse,
                      icon: const Icon(CrabforgeSymbols.storefront),
                      label: Text(strings.pluginCatalog),
                    ),
                    PopupMenuButton<String>(
                      enabled: !_busy,
                      onSelected: (action) {
                        if (action == 'export') {
                          _export();
                        } else {
                          _prompt(
                            strings.importPluginSettings,
                            'JSON',
                            (source) => _load(
                              () => api.importClaudePluginSettings(
                                source: source,
                              ),
                            ),
                            multiline: true,
                          );
                        }
                      },
                      itemBuilder: (_) => [
                        PopupMenuItem(
                          value: 'export',
                          child: Text(strings.exportPluginSettings),
                        ),
                        PopupMenuItem(
                          value: 'import',
                          child: Text(strings.importPluginSettings),
                        ),
                      ],
                    ),
                  ],
                ),
              if (_busy) const LinearProgressIndicator(),
              Expanded(
                child: CustomScrollView(
                  slivers: [
                    if (_error != null)
                      SliverToBoxAdapter(child: SelectableText('$_error')),
                    if (!_busy && plugins.isEmpty && _error == null)
                      SliverToBoxAdapter(child: Text(strings.noClaudePlugins)),
                    CollectionSliver.builder(
                      grid: grid,
                      height: 134,
                      itemCount: plugins.length,
                      itemBuilder: (context, index) {
                        final plugin = plugins[index];
                        final id = plugin['id'] as String;
                        final installed = (plugin['scopes'] as List).contains(
                          'user',
                        );
                        final detail = [
                          ...(plugin['versions'] as List).cast<String>(),
                          plugin['enabled'] == true
                              ? strings.pluginEnabled
                              : strings.disabled,
                        ].join(' · ');
                        return CollectionTile(
                          key: ValueKey(id),
                          grid: grid,
                          leading: const Icon(CrabforgeSymbols.extension),
                          title: Text(
                            id,
                            maxLines: 2,
                            overflow: TextOverflow.ellipsis,
                          ),
                          detail: Text(detail),
                          actions: [
                            if (plugin['can_enable'] == true)
                              IconButton(
                                onPressed: _busy ? null : () => _set(id, true),
                                icon: const Icon(CrabforgeSymbols.addCircle),
                                tooltip: strings.enablePlugin,
                              ),
                            if (plugin['can_disable'] == true)
                              IconButton(
                                onPressed: _busy ? null : () => _set(id, false),
                                icon: const Icon(CrabforgeSymbols.removeCircle),
                                tooltip: project == null
                                    ? strings.disablePlugin
                                    : strings.remove,
                              ),
                            if (project == null && installed) ...[
                              IconButton(
                                onPressed: _busy
                                    ? null
                                    : () => _manage('update', id),
                                icon: const Icon(CrabforgeSymbols.update),
                                tooltip: strings.updatePlugin,
                              ),
                              IconButton(
                                onPressed: _busy
                                    ? null
                                    : () => _remove('uninstall', id),
                                icon: const Icon(CrabforgeSymbols.delete),
                                tooltip: strings.remove,
                              ),
                            ],
                          ],
                        );
                      },
                    ),
                    if (project == null && marketplaces.isNotEmpty) ...[
                      SliverToBoxAdapter(
                        child: Padding(
                          padding: const EdgeInsets.symmetric(vertical: 12),
                          child: Text(
                            strings.pluginMarketplaces,
                            style: Theme.of(context).textTheme.titleMedium,
                          ),
                        ),
                      ),
                      CollectionSliver.builder(
                        grid: grid,
                        height: 100,
                        itemCount: marketplaces.length,
                        itemBuilder: (context, index) {
                          final name = marketplaces[index];
                          return CollectionTile(
                            grid: grid,
                            leading: const Icon(CrabforgeSymbols.storefront),
                            title: Text(
                              name,
                              maxLines: 2,
                              overflow: TextOverflow.ellipsis,
                            ),
                            actions: [
                              IconButton(
                                onPressed: _busy
                                    ? null
                                    : () => _manage('update-marketplace', name),
                                icon: const Icon(CrabforgeSymbols.refresh),
                                tooltip: strings.reload,
                              ),
                              IconButton(
                                onPressed: _busy
                                    ? null
                                    : () => _remove('remove-marketplace', name),
                                icon: const Icon(CrabforgeSymbols.delete),
                                tooltip: strings.remove,
                              ),
                            ],
                          );
                        },
                      ),
                    ],
                  ],
                ),
              ),
              Align(
                alignment: AlignmentDirectional.centerEnd,
                child: _close(context),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
