import 'dart:convert';

import 'package:file_selector/file_selector.dart';
import 'package:flutter/material.dart';

import 'collection_view.dart';
import 'language.dart';
import 'src/rust/api.dart' as api;

part 'plugin_bundles_project.dart';

String _projectPackagesTitle(
  BuildContext context,
  api.ProjectPluginAgent agent,
) => switch (agent) {
  api.ProjectPluginAgent.pi => context.strings.piProjectPackages,
  api.ProjectPluginAgent.claude => context.strings.claudeProjectPackages,
  api.ProjectPluginAgent.codex => context.strings.codexProjectPackages,
  api.ProjectPluginAgent.antigravity =>
    context.strings.antigravityProjectPackages,
};

typedef _Json = Map<String, dynamic>;

const _archiveType = XTypeGroup(label: 'TAR', extensions: ['tar']);

class PluginBundles extends StatefulWidget {
  const PluginBundles({super.key});

  @override
  State<PluginBundles> createState() => _PluginBundlesState();
}

class _PluginBundlesState extends State<PluginBundles> {
  List<_Json> _entries = const [];
  Object? _error;
  bool _busy = false;

  @override
  void initState() {
    super.initState();
    _refresh(api.pluginBundles);
  }

  Future<void> _refresh(Future<String> Function() operation) async {
    setState(() {
      _busy = true;
      _error = null;
    });
    try {
      final rows = (jsonDecode(await operation()) as List).cast<_Json>();
      if (mounted) setState(() => _entries = rows);
    } catch (error) {
      if (mounted) setState(() => _error = error);
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  Future<({String source, bool marketplace})?> _chooseSource() async {
    var value = '';
    return showDialog<({String source, bool marketplace})>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(context.strings.importPluginBundle),
        content: TextField(
          autofocus: true,
          decoration: InputDecoration(
            labelText: context.strings.pluginBundleSource,
          ),
          onChanged: (text) => value = text.trim(),
          onSubmitted: (text) =>
              Navigator.pop(context, (source: text.trim(), marketplace: false)),
        ),
        actions: [
          TextButton(
            onPressed: () async {
              final file = await openFile(acceptedTypeGroups: [_archiveType]);
              if (context.mounted && file != null) {
                Navigator.pop(context, (source: file.path, marketplace: false));
              }
            },
            child: Text(context.strings.openPluginArchive),
          ),
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: Text(context.strings.cancel),
          ),
          TextButton(
            onPressed: () =>
                Navigator.pop(context, (source: value, marketplace: true)),
            child: Text(context.strings.pluginCatalog),
          ),
          FilledButton(
            onPressed: () =>
                Navigator.pop(context, (source: value, marketplace: false)),
            child: Text(context.strings.previewPluginBundle),
          ),
        ],
      ),
    );
  }

  Future<String?> _browse(String source) async {
    final market = jsonDecode(
      await api.browsePluginBundleMarketplace(source: source),
    ) as _Json;
    if (!mounted) return null;
    setState(() => _busy = false);
    final rows = (market['candidates'] as List).cast<_Json>();
    final name = market['name'] as String;
    return showDialog<String>(
      context: context,
      builder: (context) {
        final grid = CollectionLayout.of(context).grid;
        return AlertDialog(
          title: EditorHeading(
            name.isEmpty ? context.strings.pluginCatalog : name,
          ),
          content: SizedBox(
            width: 670,
            height: 400,
            child: CustomScrollView(
              slivers: [
                CollectionSliver.builder(
                  grid: grid,
                  height: 104,
                  itemCount: rows.length,
                  itemBuilder: (context, index) {
                    final row = rows[index];
                    final source = row['source'] as String?;
                    final detail =
                        row['detail'] as String? ??
                        [row['version'], row['description']]
                            .cast<String>()
                            .where((part) => part.isNotEmpty)
                            .join(' · ');
                    return CollectionTile(
                      grid: grid,
                      title: Text(
                        row['name'] as String,
                        maxLines: 2,
                        overflow: TextOverflow.ellipsis,
                      ),
                      detail: detail.isEmpty
                          ? null
                          : Text(
                              detail,
                              maxLines: 3,
                              overflow: TextOverflow.ellipsis,
                            ),
                      onTap: source == null
                          ? null
                          : () => Navigator.pop(context, source),
                    );
                  },
                ),
              ],
            ),
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.pop(context),
              child: Text(context.strings.cancel),
            ),
          ],
        );
      },
    );
  }

  Future<void> _import([_Json? previous]) async {
    final choice = previous == null
        ? await _chooseSource()
        : (
            source: (previous['bundle'] as _Json)['source'] as String,
            marketplace: false,
          );
    if (!mounted || choice == null || choice.source.isEmpty) return;
    setState(() {
      _busy = true;
      _error = null;
    });
    try {
      final source = choice.marketplace
          ? await _browse(choice.source)
          : choice.source;
      if (!mounted || source == null) return;
      setState(() => _busy = true);
      final bundle =
          jsonDecode(await api.previewPluginBundle(source: source)) as _Json;
      if (!mounted) return;
      setState(() => _busy = false);
      if (!await _details(bundle) || !mounted) return;
      await _refresh(
        () => api.savePluginBundle(
          source: source,
          content: bundle['content'] as String,
          revision: previous?['revision'] as String?,
        ),
      );
    } catch (error) {
      if (mounted) setState(() => _error = error);
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  Future<void> _remove(_Json entry) async {
    final name = (entry['bundle'] as _Json)['name'] as String;
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(context.strings.confirmRemove(name)),
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
    if (mounted && confirmed == true) {
      await _refresh(
        () => api.removePluginBundle(
          name: name,
          revision: entry['revision'] as String,
        ),
      );
    }
  }

  Future<void> _export(_Json entry) async {
    final name = (entry['bundle'] as _Json)['name'] as String;
    setState(() {
      _busy = true;
      _error = null;
    });
    try {
      final file = await getSaveLocation(
        acceptedTypeGroups: [_archiveType],
        suggestedName: '$name.tar',
      );
      if (file == null || !mounted) return;
      await api.exportPluginBundle(
        name: name,
        revision: entry['revision'] as String,
        output: file.path.toLowerCase().endsWith('.tar')
            ? file.path
            : '${file.path}.tar',
      );
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text(context.strings.pluginArchiveSaved)),
        );
      }
    } catch (error) {
      if (mounted) setState(() => _error = error);
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  Future<bool> _details(
    _Json bundle, {
    String? folder,
    String? revision,
  }) async {
    final preview = revision == null;
    return await showDialog<bool>(
          context: context,
          builder: (context) {
            final strings = context.strings;
            final grid = CollectionLayout.of(context).grid;
            final groups = {
              ...(bundle['components'] as _Json),
              if ((bundle['other_manifest_entries'] as List).isNotEmpty)
                'other': bundle['other_manifest_entries'],
            };
            String label(String kind) => switch (kind) {
              'skills' => strings.agentSkills,
              'subagents' => strings.agentSubagents,
              'hooks' => strings.agentHooks,
              'mcp' => 'MCP',
              'commands' => strings.agentCommands,
              _ => strings.otherPluginEntries,
            };
            return AlertDialog(
              title: EditorHeading(bundle['name'] as String),
              content: SizedBox(
                width: 670,
                height: 400,
                child: CustomScrollView(
                  slivers: [
                    SliverToBoxAdapter(
                      child: Padding(
                        padding: const EdgeInsets.only(bottom: 12),
                        child: Column(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          spacing: 8,
                          children: [
                            if ((bundle['description'] as String).isNotEmpty)
                              Text(bundle['description'] as String),
                            if ((bundle['version'] as String).isNotEmpty)
                              Text(bundle['version'] as String),
                            SelectableText(
                              folder ?? bundle['source'] as String,
                            ),
                          ],
                        ),
                      ),
                    ),
                    for (final group in groups.entries) ...[
                      SliverToBoxAdapter(
                        child: Padding(
                          padding: const EdgeInsets.symmetric(vertical: 8),
                          child: Text(
                            label(group.key),
                            style: Theme.of(context).textTheme.titleMedium,
                          ),
                        ),
                      ),
                      CollectionSliver.builder(
                        grid: grid,
                        height: 74,
                        itemCount: (group.value as List).length,
                        itemBuilder: (_, i) {
                          final path = (group.value as List)[i] as String;
                          return CollectionTile(
                            grid: grid,
                            title: Text(
                              path,
                              maxLines: 2,
                              overflow: TextOverflow.ellipsis,
                            ),
                            onTap: revision == null || group.key == 'other'
                                ? null
                                : () => showDialog<void>(
                                    context: context,
                                    builder: (_) => _BundleResources(
                                      name: bundle['name'] as String,
                                      revision: revision,
                                      kind: group.key,
                                      path: path,
                                    ),
                                  ),
                          );
                        },
                      ),
                    ],
                  ],
                ),
              ),
              actions: [
                TextButton(
                  onPressed: () => Navigator.pop(context),
                  child: Text(
                    preview
                        ? strings.cancel
                        : MaterialLocalizations.of(context).closeButtonLabel,
                  ),
                ),
                if (preview)
                  FilledButton(
                    onPressed: () => Navigator.pop(context, true),
                    child: Text(strings.save),
                  ),
              ],
            );
          },
        ) ??
        false;
  }

  @override
  Widget build(BuildContext context) {
    final strings = context.strings;
    final grid = CollectionLayout.of(context).grid;
    return Dialog(
      key: const ValueKey('plugin-bundles'),
      child: SizedBox(
        width: 800,
        height: 550,
        child: Padding(
          padding: const EdgeInsets.all(20),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              DefaultTextStyle(
                style: Theme.of(context).textTheme.titleLarge!,
                child: EditorHeading(strings.pluginBundles),
              ),
              Wrap(
                children: [
                  TextButton.icon(
                    onPressed: _busy ? null : () => _import(),
                    icon: const Icon(Icons.download_outlined),
                    label: Text(strings.importPluginBundle),
                  ),
                  for (final agent in api.ProjectPluginAgent.values)
                    TextButton.icon(
                      onPressed: _busy
                          ? null
                          : () => showDialog<void>(
                              context: context,
                              builder: (_) => _ProjectPackages(agent),
                            ),
                      icon: const Icon(Icons.folder_outlined),
                      label: Text(_projectPackagesTitle(context, agent)),
                    ),
                  IconButton(
                    onPressed: _busy ? null : () => _refresh(api.pluginBundles),
                    icon: const Icon(Icons.refresh),
                    tooltip: strings.reload,
                  ),
                ],
              ),
              if (_busy) const LinearProgressIndicator(),
              Expanded(
                child: CustomScrollView(
                  slivers: [
                    if (_error != null)
                      SliverToBoxAdapter(child: SelectableText('$_error')),
                    if (!_busy && _entries.isEmpty && _error == null)
                      SliverToBoxAdapter(child: Text(strings.noPluginBundles)),
                    CollectionSliver.builder(
                      grid: grid,
                      height: 130,
                      itemCount: _entries.length,
                      itemBuilder: (context, index) {
                        final entry = _entries[index];
                        final bundle = entry['bundle'] as _Json;
                        final name = bundle['name'] as String;
                        final description = bundle['description'] as String;
                        final version = bundle['version'] as String;
                        return CollectionTile(
                          key: ValueKey(name),
                          grid: grid,
                          onTap: () => _details(
                            bundle,
                            folder: entry['folder'] as String,
                            revision: entry['revision'] as String,
                          ),
                          leading: const Icon(Icons.inventory_2_outlined),
                          title: Text(
                            name,
                            maxLines: 1,
                            overflow: TextOverflow.ellipsis,
                          ),
                          detail: Text(
                            [
                              version,
                              description,
                            ].where((text) => text.isNotEmpty).join(' · '),
                            maxLines: 2,
                            overflow: TextOverflow.ellipsis,
                          ),
                          actions: [
                            IconButton(
                              onPressed: _busy ? null : () => _export(entry),
                              icon: const Icon(Icons.ios_share_outlined),
                              tooltip: strings.exportPluginArchive,
                            ),
                            IconButton(
                              onPressed: _busy ? null : () => _import(entry),
                              icon: const Icon(Icons.update_outlined),
                              tooltip: strings.updatePlugin,
                            ),
                            IconButton(
                              onPressed: _busy ? null : () => _remove(entry),
                              icon: const Icon(Icons.delete_outline),
                              tooltip: strings.remove,
                            ),
                          ],
                        );
                      },
                    ),
                  ],
                ),
              ),
              Align(
                alignment: AlignmentDirectional.centerEnd,
                child: TextButton(
                  onPressed: () => Navigator.pop(context),
                  child: Text(
                    MaterialLocalizations.of(context).closeButtonLabel,
                  ),
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

class _BundleResources extends StatefulWidget {
  const _BundleResources({
    required this.name,
    required this.revision,
    required this.kind,
    required this.path,
  });

  final String name, revision, kind, path;

  @override
  State<_BundleResources> createState() => _BundleResourcesState();
}

class _BundleResourcesState extends State<_BundleResources> {
  late final Future<List<_Json>> _resources;
  final _copied = <String>{};
  Object? _error;
  bool _busy = false;

  @override
  void initState() {
    super.initState();
    _resources = _request().then(
      (source) => (jsonDecode(source) as List).cast<_Json>(),
    );
  }

  Future<String> _request([String? key]) => api.pluginBundleResources(
    name: widget.name,
    revision: widget.revision,
    kind: widget.kind,
    path: widget.path,
    copy: key,
  );

  Future<void> _review(_Json entry) async {
    final key = entry['key'] as String;
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(entry['name'] as String),
        content: SizedBox(
          width: 580,
          height: 320,
          child: SingleChildScrollView(
            child: SelectableText(
              const JsonEncoder.withIndent('  ').convert(entry['definition']),
            ),
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: Text(context.strings.cancel),
          ),
          FilledButton(
            onPressed: () => Navigator.pop(context, true),
            child: Text(context.strings.copyBundleResource),
          ),
        ],
      ),
    );
    if (!mounted || confirmed != true) return;
    setState(() {
      _busy = true;
      _error = null;
    });
    try {
      await _request(key);
      if (mounted) setState(() => _copied.add(key));
    } catch (error) {
      if (mounted) setState(() => _error = error);
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    final grid = CollectionLayout.of(context).grid;
    final strings = context.strings;
    return AlertDialog(
      title: EditorHeading(widget.path),
      content: SizedBox(
        width: 670,
        height: 360,
        child: FutureBuilder<List<_Json>>(
          future: _resources,
          builder: (context, snapshot) {
            if (snapshot.hasError) return SelectableText('${snapshot.error}');
            final rows = snapshot.data;
            if (rows == null) {
              return const Center(child: CircularProgressIndicator());
            }
            if (rows.isEmpty) return Text(strings.noBundleResources);
            return CustomScrollView(
              slivers: [
                if (_error != null)
                  SliverToBoxAdapter(child: SelectableText('$_error')),
                if (_busy)
                  const SliverToBoxAdapter(child: LinearProgressIndicator()),
                CollectionSliver.builder(
                  grid: grid,
                  height: 108,
                  itemCount: rows.length,
                  itemBuilder: (context, index) {
                    final entry = rows[index];
                    final copied = _copied.contains(entry['key']);
                    final detail = copied
                        ? strings.bundleResourceCopied
                        : entry['detail'] as String?;
                    return CollectionTile(
                      grid: grid,
                      title: Text(
                        entry['name'] as String,
                        maxLines: 2,
                        overflow: TextOverflow.ellipsis,
                      ),
                      detail: detail == null
                          ? null
                          : Text(
                              detail,
                              maxLines: 3,
                              overflow: TextOverflow.ellipsis,
                            ),
                      onTap: _busy || copied || entry['definition'] == null
                          ? null
                          : () => _review(entry),
                    );
                  },
                ),
              ],
            );
          },
        ),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.pop(context),
          child: Text(MaterialLocalizations.of(context).closeButtonLabel),
        ),
      ],
    );
  }
}
