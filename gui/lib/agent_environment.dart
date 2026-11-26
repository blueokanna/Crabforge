import 'dart:convert';

import 'package:flutter/material.dart';

import 'collection_view.dart';
import 'language.dart';
import 'src/rust/api.dart' as api;

typedef _Json = Map<String, dynamic>;

const _agents = {'claude': 'Claude Code', 'codex': 'Codex', 'pi': 'Pi'};

class AgentEnvironment extends StatefulWidget {
  const AgentEnvironment({super.key});

  @override
  State<AgentEnvironment> createState() => _AgentEnvironmentState();
}

class _AgentEnvironmentState extends State<AgentEnvironment> {
  _Json _view = const {};
  Object? _error;
  bool _busy = false;

  @override
  void initState() {
    super.initState();
    _load(api.agentEnvironment);
  }

  Future<void> _load(Future<String> Function() work) async {
    setState(() {
      _busy = true;
      _error = null;
    });
    try {
      final view = jsonDecode(await work()) as _Json;
      if (mounted) setState(() => _view = view);
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
      builder: (_) => _VariableEditor(entry: entry),
    );
    if (mounted && result != null) {
      setState(() => _view = jsonDecode(result) as _Json);
    }
  }

  Future<void> _remove(_Json entry) async {
    final name = entry['name'] as String;
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(context.strings.confirmRemove(name)),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: Text(context.strings.cancel),
          ),
          TextButton(
            onPressed: () => Navigator.pop(context, true),
            child: Text(context.strings.remove),
          ),
        ],
      ),
    );
    if (mounted && confirmed == true) {
      await _load(
        () => api.removeAgentVariable(
          name: name,
          revision: entry['revision'] as String,
        ),
      );
    }
  }

  Widget _placement(BuildContext context, _Json row) {
    final target = row['target'] as _Json;
    final status = row['status'] as String;
    final strings = context.strings;
    final description = switch (status) {
      'existing' => strings.nativeVariableExists,
      'conflict' => strings.nativeVariableConflict,
      _ => strings.ruleStatus(status),
    };
    final label = '${_agents[target['agent']]} · $description';
    return Tooltip(
      message: [label, target['path'], ?row['detail']].join('\n'),
      child: Text(label, maxLines: 1, overflow: TextOverflow.ellipsis),
    );
  }

  @override
  Widget build(BuildContext context) {
    final strings = context.strings;
    final grid = CollectionLayout.of(context).grid;
    final variables =
        (_view['variables'] as List?)?.cast<_Json>() ?? const <_Json>[];
    final byName = <String, List<_Json>>{};
    for (final row
        in (_view['placements'] as List?)?.cast<_Json>() ?? const <_Json>[]) {
      (byName[row['name'] as String] ??= []).add(row);
    }
    final entries = {
      for (final variable in variables) variable['name'] as String: variable,
    };
    final names = {...entries.keys, ...byName.keys}.toList(growable: false);
    return Dialog(
      key: const ValueKey('agent-environment'),
      child: SizedBox(
        width: 760,
        height: 550,
        child: Padding(
          padding: const EdgeInsets.all(20),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              DefaultTextStyle(
                style: Theme.of(context).textTheme.titleLarge!,
                child: EditorHeading(strings.agentEnvironment),
              ),
              Wrap(
                children: [
                  TextButton.icon(
                    onPressed: _busy ? null : () => _edit(null),
                    icon: const Icon(Icons.add),
                    label: Text(strings.addVariable),
                  ),
                  TextButton.icon(
                    onPressed: _busy
                        ? null
                        : () => _load(api.syncAgentEnvironment),
                    icon: const Icon(Icons.sync),
                    label: Text(strings.syncMcpLibrary),
                  ),
                  IconButton(
                    onPressed: _busy ? null : () => _load(api.agentEnvironment),
                    tooltip: strings.reload,
                    icon: const Icon(Icons.refresh),
                  ),
                ],
              ),
              if (_busy) const LinearProgressIndicator(),
              Expanded(
                child: CustomScrollView(
                  slivers: [
                    if (_error case final Object error)
                      SliverToBoxAdapter(child: SelectableText('$error')),
                    if (!_busy && _error == null && names.isEmpty)
                      SliverToBoxAdapter(
                        child: Text(strings.noEnvironmentVariables),
                      ),
                    CollectionSliver.builder(
                      grid: grid,
                      height: 156,
                      itemCount: names.length,
                      itemBuilder: (context, index) {
                        final name = names[index];
                        final entry = entries[name];
                        final placements = byName[name] ?? const <_Json>[];
                        return CollectionTile(
                          key: ValueKey(name),
                          grid: grid,
                          leading: const Icon(Icons.terminal_outlined),
                          title: Text(
                            name,
                            maxLines: 2,
                            overflow: TextOverflow.ellipsis,
                          ),
                          onTap: _busy || entry == null
                              ? null
                              : () => _edit(entry),
                          detail: Column(
                            crossAxisAlignment: CrossAxisAlignment.start,
                            children: [
                              for (final row in placements)
                                _placement(context, row),
                            ],
                          ),
                          actions: entry == null
                              ? const []
                              : [
                                  IconButton(
                                    onPressed: _busy
                                        ? null
                                        : () => _edit(entry),
                                    icon: const Icon(Icons.edit_outlined),
                                    tooltip: strings.editVariable,
                                  ),
                                  IconButton(
                                    onPressed: _busy
                                        ? null
                                        : () => _remove(entry),
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

class _VariableEditor extends StatefulWidget {
  const _VariableEditor({required this.entry});

  final _Json? entry;

  @override
  State<_VariableEditor> createState() => _VariableEditorState();
}

class _VariableEditorState extends State<_VariableEditor> {
  late final _name = TextEditingController(
    text: widget.entry?['name'] as String?,
  );
  late final _value = TextEditingController(
    text: widget.entry?['value'] as String?,
  );
  late final _selected =
      (widget.entry?['agents'] as List?)?.cast<String>().toSet() ?? <String>{};
  bool _visible = false;
  bool _busy = false;
  Object? _error;

  @override
  void dispose() {
    _name.dispose();
    _value.dispose();
    super.dispose();
  }

  Future<void> _save() async {
    setState(() {
      _busy = true;
      _error = null;
    });
    try {
      final result = await api.saveAgentVariable(
        name: _name.text.trim(),
        value: _value.text,
        agents: _selected.toList(),
        oldName: widget.entry?['name'] as String?,
        revision: widget.entry?['revision'] as String?,
      );
      if (mounted) Navigator.pop(context, result);
    } catch (error) {
      if (mounted) {
        setState(() {
          _error = error;
          _busy = false;
        });
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    final strings = context.strings;
    return PopScope(
      canPop: !_busy,
      child: AlertDialog(
        title: Text(
          widget.entry == null ? strings.addVariable : strings.editVariable,
        ),
        scrollable: true,
        content: SizedBox(
          width: 420,
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            mainAxisSize: MainAxisSize.min,
            children: [
              TextField(
                controller: _name,
                enabled: !_busy,
                onChanged: (_) => setState(() {}),
                decoration: InputDecoration(labelText: strings.variableName),
              ),
              const SizedBox(height: 12),
              TextField(
                controller: _value,
                enabled: !_busy,
                obscureText: !_visible,
                maxLines: _visible ? 4 : 1,
                autocorrect: false,
                enableSuggestions: false,
                decoration: InputDecoration(
                  labelText: strings.variableValue,
                  suffixIcon: IconButton(
                    onPressed: () => setState(() => _visible = !_visible),
                    icon: Icon(
                      _visible
                          ? Icons.visibility_off_outlined
                          : Icons.visibility_outlined,
                    ),
                    tooltip: _visible
                        ? strings.hideVariableValue
                        : strings.showVariableValue,
                  ),
                ),
              ),
              const SizedBox(height: 12),
              Wrap(
                spacing: 6,
                children: [
                  for (final agent in _agents.entries)
                    FilterChip(
                      label: Text(agent.value),
                      selected: _selected.contains(agent.key),
                      onSelected: _busy
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
              if (_error case final Object error) SelectableText('$error'),
              if (_busy) const LinearProgressIndicator(),
            ],
          ),
        ),
        actions: [
          TextButton(
            onPressed: _busy ? null : () => Navigator.pop(context),
            child: Text(strings.cancel),
          ),
          FilledButton(
            onPressed: _busy || _name.text.trim().isEmpty ? null : _save,
            child: Text(strings.save),
          ),
        ],
      ),
    );
  }
}
