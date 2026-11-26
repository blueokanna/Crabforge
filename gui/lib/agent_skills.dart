import 'dart:convert';

import 'package:file_selector/file_selector.dart';
import 'package:flutter/material.dart';

import 'collection_view.dart';
import 'language.dart';
import 'src/rust/api.dart' as api;

part 'installed_skills.dart';
part 'repository_skills.dart';

const _agents = {
  'claude': 'Claude Code',
  'codex': 'Codex',
  'gemini': 'Gemini CLI',
  'opencode': 'OpenCode',
  'pi': 'Pi',
};

typedef _Json = Map<String, dynamic>;

class AgentSkills extends StatefulWidget {
  const AgentSkills({super.key});

  @override
  State<AgentSkills> createState() => _AgentSkillsState();
}

class _AgentSkillsState extends State<AgentSkills> {
  _Json? _library;
  Object? _error;
  bool _busy = true;

  @override
  void initState() {
    super.initState();
    _load(api.agentSkills);
  }

  Future<void> _load(Future<String> Function() work) async {
    setState(() {
      _busy = true;
      _error = null;
    });
    try {
      final library = jsonDecode(await work()) as _Json;
      if (mounted) setState(() => _library = library);
    } catch (error) {
      if (mounted) setState(() => _error = error);
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  Future<void> _edit(_Json? entry) async {
    final result = await showDialog<String>(
      context: context,
      barrierDismissible: false,
      builder: (_) => _SkillEditor(entry: entry),
    );
    if (!mounted || result == null) return;
    setState(() {
      _library = jsonDecode(result) as _Json;
      _error = null;
    });
  }

  Future<void> _discover() async {
    final result = await showDialog<String>(
      context: context,
      barrierDismissible: false,
      builder: (_) => const _InstalledSkills(),
    );
    if (mounted && result != null) {
      setState(() {
        _library = jsonDecode(result) as _Json;
        _error = null;
      });
    }
  }

  Future<void> _remove(String name, String revision) async {
    final strings = context.strings;
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(strings.confirmRemove(name)),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: Text(strings.cancel),
          ),
          TextButton(
            onPressed: () => Navigator.pop(context, true),
            child: Text(strings.remove),
          ),
        ],
      ),
    );
    if (mounted && confirmed == true) {
      await _load(() => api.removeAgentSkill(name: name, revision: revision));
    }
  }

  Widget _placements(List<_Json> placements) => Column(
    crossAxisAlignment: CrossAxisAlignment.start,
    children: [
      for (final placement in placements) ...[
        Text(
          '${_agents[placement['agent']]} · ${context.strings.ruleStatus(placement['status'] as String)}',
        ),
        if (placement['path'] case final String path)
          SelectableText(path, style: Theme.of(context).textTheme.bodySmall),
        if (placement['error'] case final String detail)
          SelectableText(
            detail,
            style: TextStyle(color: Theme.of(context).colorScheme.error),
          ),
      ],
    ],
  );

  @override
  Widget build(BuildContext context) {
    final strings = context.strings;
    final grid = CollectionLayout.of(context).grid;
    final skills = (_library?['skills'] as List?)?.cast<_Json>() ?? [];
    final placements = <String, List<_Json>>{};
    for (final placement
        in (_library?['placements'] as List?)?.cast<_Json>() ?? []) {
      (placements[placement['name'] as String] ??= []).add(placement);
    }
    final names = skills
        .map((entry) => (entry['skill'] as _Json)['name'])
        .toSet();
    final missing = placements.entries.where(
      (entry) => !names.contains(entry.key),
    );
    return Dialog(
      child: SizedBox(
        width: 780,
        height: 620,
        child: Padding(
          padding: const EdgeInsets.all(20),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              DefaultTextStyle(
                style: Theme.of(context).textTheme.titleLarge!,
                child: EditorHeading(strings.agentSkills),
              ),
              Wrap(
                runSpacing: 4,
                children: [
                  TextButton.icon(
                    onPressed: _busy ? null : () => _edit(null),
                    icon: const Icon(Icons.create_new_folder_outlined),
                    label: Text(strings.importSkill),
                  ),
                  TextButton.icon(
                    onPressed: _busy ? null : _discover,
                    icon: const Icon(Icons.folder_open_outlined),
                    label: Text(strings.installedSkills),
                  ),
                  TextButton.icon(
                    onPressed: _busy ? null : () => _load(api.syncAgentSkills),
                    icon: const Icon(Icons.sync),
                    label: Text(strings.applySkills),
                  ),
                  IconButton(
                    onPressed: _busy ? null : () => _load(api.agentSkills),
                    tooltip: strings.reload,
                    icon: const Icon(Icons.refresh),
                  ),
                ],
              ),
              if (_busy) const LinearProgressIndicator(),
              Expanded(
                child: CustomScrollView(
                  slivers: [
                    if (_error != null)
                      SliverToBoxAdapter(child: SelectableText('$_error')),
                    if (skills.isEmpty && !_busy && _error == null)
                      SliverToBoxAdapter(child: Text(strings.noSkills)),
                    CollectionSliver.builder(
                      grid: grid,
                      height: 174,
                      itemCount: skills.length,
                      itemBuilder: (context, index) {
                        final entry = skills[index];
                        final skill = entry['skill'] as _Json;
                        final name = skill['name'] as String;
                        return CollectionTile(
                          key: ValueKey(name),
                          grid: grid,
                          title: Text(name),
                          leading: const Icon(Icons.extension_outlined),
                          onTap: _busy ? null : () => _edit(entry),
                          detail: Column(
                            crossAxisAlignment: CrossAxisAlignment.start,
                            children: [
                              Text(
                                skill['description'] as String,
                                maxLines: 2,
                                overflow: TextOverflow.ellipsis,
                              ),
                              _placements(placements[name] ?? []),
                            ],
                          ),
                          actions: [
                            IconButton(
                              onPressed: _busy ? null : () => _edit(entry),
                              tooltip: strings.editSkill,
                              icon: const Icon(Icons.edit_outlined),
                            ),
                            IconButton(
                              onPressed: _busy
                                  ? null
                                  : () => _remove(
                                      name,
                                      entry['revision'] as String,
                                    ),
                              tooltip: strings.remove,
                              icon: const Icon(Icons.delete_outline),
                            ),
                          ],
                        );
                      },
                    ),
                    for (final entry in missing)
                      SliverToBoxAdapter(
                        child: ListTile(
                          title: Text(entry.key),
                          subtitle: _placements(entry.value),
                        ),
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

class _SkillEditor extends StatefulWidget {
  const _SkillEditor({required this.entry, this.source, this.agents});

  final _Json? entry;
  final String? source;
  final Set<String>? agents;

  @override
  State<_SkillEditor> createState() => _SkillEditorState();
}

class _SkillEditorState extends State<_SkillEditor> {
  late final _Json? _skill = widget.entry?['skill'] as _Json?;
  late final _source = TextEditingController(
    text: widget.source ?? _skill?['source'] as String?,
  );
  late final _selected =
      widget.agents?.toSet() ??
      (_skill?['agents'] as List?)?.cast<String>().toSet() ??
      <String>{};
  bool _working = false;
  Object? _error;
  String? _updateStatus;

  @override
  void dispose() {
    _source.dispose();
    super.dispose();
  }

  Future<void> _chooseFolder() async {
    final path = await getDirectoryPath();
    if (path != null && mounted) {
      setState(() {
        _source.text = path;
        _updateStatus = null;
      });
    }
  }

  Future<void> _checkUpdate() async {
    setState(() {
      _working = true;
      _error = null;
      _updateStatus = null;
    });
    try {
      final result = jsonDecode(
        await api.checkAgentSkillUpdate(
          name: _skill!['name'] as String,
          revision: widget.entry!['revision'] as String,
        ),
      ) as _Json;
      if (mounted) setState(() => _updateStatus = result['status'] as String);
    } catch (error) {
      if (mounted) setState(() => _error = error);
    } finally {
      if (mounted) setState(() => _working = false);
    }
  }

  Future<void> _browseRepository() async {
    final source = await showDialog<String>(
      context: context,
      builder: (_) => _RepositorySkills(source: _source.text.trim()),
    );
    if (!mounted || source == null) return;
    if (_skill == null) {
      setState(() => _source.text = source);
      return;
    }
    final library = await showDialog<String>(
      context: context,
      barrierDismissible: false,
      builder: (_) =>
          _SkillEditor(entry: null, source: source, agents: _selected),
    );
    if (mounted && library != null) Navigator.pop(context, library);
  }

  Future<void> _save({required bool reimport}) async {
    setState(() {
      _working = true;
      _error = null;
      _updateStatus = null;
    });
    try {
      final revision = widget.entry?['revision'] as String?;
      final source = _source.text.trim();
      final result = reimport || _skill == null || source != _skill['source']
          ? await api.importAgentSkill(
              source: source,
              agents: _selected.toList(),
              revision: revision,
            )
          : await api.assignAgentSkill(
              name: _skill['name'] as String,
              agents: _selected.toList(),
              revision: revision!,
            );
      if (mounted) Navigator.pop(context, result);
    } catch (error) {
      if (mounted) {
        setState(() {
          _working = false;
          _error = error;
        });
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    final strings = context.strings;
    final source = _source.text.trim();
    final ready = !_working && source.isNotEmpty;
    final savedSource = _skill?['source'] as String?;
    final canCheck =
        savedSource != null &&
        savedSource.contains('://') &&
        source == savedSource;
    return PopScope(
      canPop: !_working,
      child: AlertDialog(
        title: Text(_skill?['name'] as String? ?? strings.importSkill),
        scrollable: true,
        content: SizedBox(
          width: 440,
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            mainAxisSize: MainAxisSize.min,
            children: [
              if (_skill?['description'] case final String description) ...[
                Text(description),
                const SizedBox(height: 12),
              ],
              TextField(
                controller: _source,
                enabled: !_working,
                onChanged: (_) => setState(() => _updateStatus = null),
                decoration: InputDecoration(
                  labelText: strings.skillSourceFolder,
                  suffixIcon: IconButton(
                    onPressed: _working ? null : _chooseFolder,
                    tooltip: strings.skillSourceFolder,
                    icon: const Icon(Icons.folder_open_outlined),
                  ),
                ),
              ),
              const SizedBox(height: 12),
              Wrap(
                spacing: 6,
                runSpacing: 6,
                children: [
                  for (final agent in _agents.entries)
                    FilterChip(
                      label: Text(agent.value),
                      selected: _selected.contains(agent.key),
                      onSelected: _working
                          ? null
                          : (selected) => setState(() {
                              if (selected) {
                                _selected.add(agent.key);
                              } else {
                                _selected.remove(agent.key);
                              }
                            }),
                    ),
                ],
              ),
              if (source.isNotEmpty)
                Wrap(
                  spacing: 6,
                  children: [
                    if (canCheck)
                      TextButton.icon(
                        onPressed: _working ? null : _checkUpdate,
                        icon: const Icon(Icons.update_outlined),
                        label: Text(strings.checkSkillUpdate),
                      ),
                    TextButton.icon(
                      onPressed: _working ? null : _browseRepository,
                      icon: const Icon(Icons.folder_open_outlined),
                      label: Text(strings.browseRepositorySkills),
                    ),
                  ],
                ),
              if (_updateStatus case final String status)
                Text(switch (status) {
                  'current' => strings.skillUpToDate,
                  'gone' => strings.skillRemovedUpstream,
                  _ => strings.skillUpdateAvailable,
                }),
              if (_error != null) SelectableText('$_error'),
              if (_working) const LinearProgressIndicator(),
            ],
          ),
        ),
        actions: [
          TextButton(
            onPressed: _working ? null : () => Navigator.pop(context),
            child: Text(strings.cancel),
          ),
          if (_skill != null)
            TextButton(
              onPressed: ready ? () => _save(reimport: true) : null,
              child: Text(strings.refreshSkill),
            ),
          FilledButton(
            onPressed: ready ? () => _save(reimport: false) : null,
            child: Text(_skill == null ? strings.importSkill : strings.save),
          ),
        ],
      ),
    );
  }
}
