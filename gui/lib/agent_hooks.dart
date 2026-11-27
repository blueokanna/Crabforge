import 'dart:convert';
import 'dart:math' as math;

import 'package:flutter/material.dart';

import 'material_symbols.dart';
import 'collection_view.dart';
import 'language.dart';
import 'project_picker.dart';
import 'src/rust/api.dart' as api;

typedef _Json = Map<String, dynamic>;

const _agents = {
  'claude': 'Claude Code',
  'codex': 'Codex',
  'gemini': 'Gemini CLI',
  'cursor': 'Cursor',
  'agy': 'Antigravity',
  'grok': 'Grok Build',
  'dsh': 'DeepSeek Harness',
};

String _eventName(BuildContext context, String event) => switch (event) {
  'turn-end' => context.strings.hookTurnEnd,
  'after-tool' => context.strings.hookAfterTool,
  'before-tool' => context.strings.hookBeforeTool,
  'prompt' => context.strings.hookPrompt,
  'session-start' => context.strings.hookSessionStart,
  _ => '${_agents[event.split(':').first]} · ${event.split(':').last}',
};

class AgentHooks extends StatefulWidget {
  const AgentHooks({super.key});

  @override
  State<AgentHooks> createState() => _AgentHooksState();
}

class _AgentHooksState extends State<AgentHooks> {
  _Json _view = const {};
  String? _project;
  bool _busy = false;
  Object? _error;

  @override
  void initState() {
    super.initState();
    _load(_read);
  }

  Future<String> _projectRequest(
    String action, {
    String? key,
    String? revision,
    List<String> agents = const [],
  }) => api.projectAgentHooks(
    project: _project!,
    action: action,
    key: key,
    revision: revision,
    agents: agents,
  );

  Future<String> _read() =>
      _project == null ? api.agentHooks() : _projectRequest('list');

  Future<String> _sync() =>
      _project == null ? api.syncAgentHooks() : _projectRequest('sync');

  void _scope(String? project) {
    setState(() {
      _project = project;
      _view = const {};
    });
    _load(_read);
  }

  Future<void> _chooseProject() async {
    final project = await openProjectFolder(context, _project);
    if (mounted && project != null) _scope(project);
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

  Future<void> _edit({_Json? saved, _Json? found}) async {
    final result = await showDialog<String>(
      context: context,
      barrierDismissible: false,
      builder: (_) => _HookEditor(
        events: (_view['events'] as _Json?) ?? const {},
        saved: saved,
        found: found,
      ),
    );
    if (mounted && result != null) {
      setState(() {
        _view = jsonDecode(result) as _Json;
        _error = null;
      });
    }
  }

  Future<void> _remove(String name, String revision) async {
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
      await _load(() => api.removeAgentHook(name: name, revision: revision));
    }
  }

  Future<void> _assign(_Json choice) async {
    final hook = choice['hook'] as _Json;
    final name = hook['name'] as String;
    final event = hook['event'] as String;
    final agents = (choice['compatible'] as List).cast<String>();
    final selected = (choice['agents'] as List).cast<String>().toSet();
    final result = await showDialog<bool>(
      context: context,
      builder: (dialogContext) => StatefulBuilder(
        builder: (context, setState) => AlertDialog(
          title: Text(name),
          scrollable: true,
          content: SizedBox(
            width: 450,
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              spacing: 12,
              children: [
                Text(_eventName(context, event)),
                SelectableText(hook['command'] as String),
                Wrap(
                  spacing: 8,
                  children: [
                    for (final agent in agents)
                      FilterChip(
                        label: Text(_agents[agent] ?? agent),
                        selected: selected.contains(agent),
                        onSelected: (value) => setState(() {
                          if (value) {
                            selected.add(agent);
                          } else {
                            selected.remove(agent);
                          }
                        }),
                      ),
                  ],
                ),
              ],
            ),
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.pop(dialogContext),
              child: Text(context.strings.cancel),
            ),
            FilledButton(
              onPressed: () => Navigator.pop(dialogContext, true),
              child: Text(context.strings.save),
            ),
          ],
        ),
      ),
    );
    if (mounted && result == true) {
      await _load(
        () => _projectRequest(
          'assign',
          key: name,
          revision: choice['revision'] as String,
          agents: selected.toList(),
        ),
      );
    }
  }

  Future<void> _changeInstalled(_Json row, {required bool restore}) async {
    final strings = context.strings;
    final target = row['target'] as _Json;
    final entry = restore
        ? ''
        : '\n${(row['hook'] as _Json?)?['command'] ?? const JsonEncoder.withIndent('  ').convert(row['entry'])}';
    final label = restore
        ? strings.restoreHookBackup
        : strings.removeInstalledHook;
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(label),
        scrollable: true,
        content: SelectableText(
          '${[_agents[target['agent']], ?row['group'], row['event']].join(' · ')}\n${target['path']}$entry',
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: Text(strings.cancel),
          ),
          FilledButton(
            onPressed: () => Navigator.pop(context, true),
            child: Text(restore ? strings.restoreHookBackup : strings.remove),
          ),
        ],
      ),
    );
    if (mounted && confirmed == true) {
      final key = row[restore ? 'id' : 'key'] as String;
      await _load(
        () => _project == null
            ? api.recoverAgentHook(key: key, restore: restore)
            : _projectRequest(
                restore ? 'restore' : 'remove-installed',
                key: key,
              ),
      );
    }
  }

  void _inspect(_Json row) {
    showDialog<void>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text([?row['group'], row['event']].join(' · ')),
        scrollable: true,
        content: SelectableText(
          const JsonEncoder.withIndent('  ').convert(row['entry']),
        ),
        actions: [_close(context)],
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final strings = context.strings;
    final grid = CollectionLayout.of(context).grid;
    final project = _project;
    final saved = {
      for (final row
          in (_view[project == null ? 'hooks' : 'shared'] as List? ?? const [])
              .cast<_Json>())
        (row['hook'] as _Json)['name'] as String: row,
    };
    final placements = <String, List<_Json>>{};
    for (final row
        in (_view['placements'] as List? ?? const []).cast<_Json>()) {
      (placements[row['name'] as String] ??= []).add(row);
    }
    final names = {...saved.keys, ...placements.keys}.toList();
    final installed = (_view['installed'] as List? ?? const []).cast<_Json>();
    final backups = (_view['backups'] as List? ?? const []).cast<_Json>();
    return Dialog(
      key: const ValueKey('agent-hooks'),
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
                child: EditorHeading(strings.agentHooks),
              ),
              Wrap(
                spacing: 8,
                children: [
                  ChoiceChip(
                    label: Text(strings.resourceUserScope),
                    selected: project == null,
                    onSelected: _busy || project == null
                        ? null
                        : (_) => _scope(null),
                  ),
                  ActionChip(
                    avatar: const Icon(CrabforgeSymbols.folderOpen),
                    label: Text(strings.openProject),
                    onPressed: _busy ? null : _chooseProject,
                  ),
                ],
              ),
              if (project != null)
                Tooltip(message: project, child: _line(project)),
              Wrap(
                children: [
                  if (project == null)
                    TextButton.icon(
                      onPressed: _busy ? null : () => _edit(),
                      icon: const Icon(CrabforgeSymbols.add),
                      label: Text(strings.addHook),
                    ),
                  TextButton.icon(
                    onPressed: _busy ? null : () => _load(_sync),
                    icon: const Icon(CrabforgeSymbols.sync),
                    label: Text(strings.syncMcpLibrary),
                  ),
                  IconButton(
                    onPressed: _busy ? null : () => _load(_read),
                    icon: const Icon(CrabforgeSymbols.refresh),
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
                    for (final issue
                        in (_view['project_errors'] as _Json? ?? const {})
                            .entries)
                      SliverToBoxAdapter(
                        child: SelectableText('${issue.key}\n${issue.value}'),
                      ),
                    for (final row in _view['errors'] as List? ?? const [])
                      SliverToBoxAdapter(
                        child: SelectableText('${row[0]['path']}\n${row[1]}'),
                      ),
                    if (!_busy && names.isEmpty && _error == null)
                      SliverToBoxAdapter(child: Text(strings.noHooks)),
                    CollectionSliver.builder(
                      grid: grid,
                      height: placements.values.fold<double>(
                        210,
                        (height, rows) => rows.length > 4
                            ? math.max(height, 118.0 + rows.length * 23)
                            : height,
                      ),
                      itemCount: names.length,
                      itemBuilder: (context, index) {
                        final name = names[index];
                        final entry = saved[name];
                        final hook = entry?['hook'] as _Json?;
                        final description = hook?['description'] as String?;
                        return CollectionTile(
                          key: ValueKey(name),
                          grid: grid,
                          leading: const Icon(CrabforgeSymbols.bolt),
                          title: _line(name),
                          detail: Column(
                            crossAxisAlignment: CrossAxisAlignment.start,
                            children: [
                              if (hook != null)
                                _line(
                                  _eventName(context, hook['event'] as String),
                                ),
                              if (description != null && description.isNotEmpty)
                                _line(description),
                              for (final row
                                  in placements[name] ?? const <_Json>[])
                                _placement(context, row),
                            ],
                          ),
                          onTap:
                              _busy ||
                                  entry == null ||
                                  (project != null &&
                                      (entry['compatible'] as List).isEmpty)
                              ? null
                              : () => project == null
                                    ? _edit(saved: entry)
                                    : _assign(entry),
                          actions: [
                            if (entry != null &&
                                project != null &&
                                (entry['compatible'] as List).isNotEmpty)
                              IconButton(
                                onPressed: _busy ? null : () => _assign(entry),
                                icon: const Icon(CrabforgeSymbols.groupAdd),
                                tooltip: strings.projectHookAssignment,
                              ),
                            if (entry != null && project == null) ...[
                              IconButton(
                                onPressed: _busy
                                    ? null
                                    : () => _edit(saved: entry),
                                icon: const Icon(CrabforgeSymbols.edit),
                                tooltip: strings.editHook,
                              ),
                              IconButton(
                                onPressed: _busy
                                    ? null
                                    : () => _remove(
                                        name,
                                        entry['revision'] as String,
                                      ),
                                icon: const Icon(CrabforgeSymbols.delete),
                                tooltip: strings.remove,
                              ),
                            ],
                          ],
                        );
                      },
                    ),
                    if (installed.isNotEmpty) ...[
                      SliverToBoxAdapter(
                        child: Padding(
                          padding: const EdgeInsets.symmetric(vertical: 12),
                          child: Text(
                            strings.installedHooks,
                            style: Theme.of(context).textTheme.titleMedium,
                          ),
                        ),
                      ),
                      CollectionSliver.builder(
                        grid: grid,
                        height: 130,
                        itemCount: installed.length,
                        itemBuilder: (context, index) {
                          final row = installed[index];
                          final target = row['target'] as _Json;
                          final hook = row['hook'] as _Json?;
                          return CollectionTile(
                            key: ValueKey(row['key']),
                            grid: grid,
                            leading: const Icon(CrabforgeSymbols.article),
                            title: _line(
                              [
                                _agents[target['agent']],
                                ?row['group'],
                                row['event'],
                              ].join(' · '),
                            ),
                            detail: Tooltip(
                              message: target['path'] as String,
                              child: _line(
                                hook?['command'] as String? ??
                                    target['path'] as String,
                              ),
                            ),
                            onTap: () => _inspect(row),
                            actions: [
                              if (hook != null && project == null)
                                IconButton(
                                  onPressed: _busy
                                      ? null
                                      : () => _edit(found: row),
                                  icon: const Icon(
                                    CrabforgeSymbols.bookmarkAdd,
                                  ),
                                  tooltip: strings.adoptHook,
                                ),
                              if (row['removable'] == true)
                                IconButton(
                                  onPressed: _busy
                                      ? null
                                      : () => _changeInstalled(
                                          row,
                                          restore: false,
                                        ),
                                  icon: const Icon(CrabforgeSymbols.delete),
                                  tooltip: strings.removeInstalledHook,
                                ),
                            ],
                          );
                        },
                      ),
                    ],
                    if (backups.isNotEmpty) ...[
                      SliverToBoxAdapter(
                        child: Padding(
                          padding: const EdgeInsets.symmetric(vertical: 12),
                          child: Text(
                            strings.hookBackups,
                            style: Theme.of(context).textTheme.titleMedium,
                          ),
                        ),
                      ),
                      CollectionSliver.builder(
                        grid: grid,
                        height: 130,
                        itemCount: backups.length,
                        itemBuilder: (context, index) {
                          final row = backups[index];
                          final target = row['target'] as _Json;
                          final status = switch (row['status']) {
                            'ready' => null,
                            'restored' => strings.hookBackupRestored,
                            'conflict' => strings.hookBackupChanged,
                            _ => strings.hookBackupUnavailable,
                          };
                          return CollectionTile(
                            key: ValueKey(row['id']),
                            grid: grid,
                            leading: const Icon(
                              CrabforgeSymbols.settingsBackupRestore,
                            ),
                            title: _line(
                              '${_agents[target['agent']]} · ${row['event']}',
                            ),
                            detail: Tooltip(
                              message: target['path'] as String,
                              child: _line(status ?? target['path'] as String),
                            ),
                            actions: [
                              if (status == null)
                                IconButton(
                                  onPressed: _busy
                                      ? null
                                      : () => _changeInstalled(
                                          row,
                                          restore: true,
                                        ),
                                  icon: const Icon(CrabforgeSymbols.restore),
                                  tooltip: strings.restoreHookBackup,
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

  Widget _placement(BuildContext context, _Json row) {
    final target = row['target'] as _Json;
    final status = row['status'] as String;
    final strings = context.strings;
    final label = switch (status) {
      'existing' => strings.hookExisting,
      'conflict' => strings.hookConflict,
      _ => strings.ruleStatus(status),
    };
    return Tooltip(
      message: [target['path'], ?row['detail']].join('\n'),
      child: _line('${_agents[target['agent']]} · $label'),
    );
  }
}

Widget _line(String text) =>
    Text(text, maxLines: 1, overflow: TextOverflow.ellipsis);

Widget _close(BuildContext context) => TextButton(
  onPressed: () => Navigator.pop(context),
  child: Text(MaterialLocalizations.of(context).closeButtonLabel),
);

class _HookEditor extends StatefulWidget {
  const _HookEditor({required this.events, this.saved, this.found});

  final _Json events;
  final _Json? saved, found;

  @override
  State<_HookEditor> createState() => _HookEditorState();
}

class _HookEditorState extends State<_HookEditor> {
  late final _Json _hook =
      (widget.saved ?? widget.found)?['hook'] as _Json? ?? const {};
  late final _name = TextEditingController(
    text: widget.found == null ? _hook['name'] as String? : null,
  );
  late final _description = TextEditingController(
    text: _hook['description'] as String?,
  );
  late final _command = TextEditingController(
    text: _hook['command'] as String?,
  );
  late final _matcher = TextEditingController(
    text: _hook['matcher'] as String?,
  );
  late final _timeout = TextEditingController(
    text: _hook['timeout'] == null || _hook['timeout'] == 0
        ? null
        : '${_hook['timeout']}',
  );
  late final _agentsSelected =
      (_hook['agents'] as List?)?.cast<String>().toSet() ?? <String>{};
  late String _event = _hook['event'] as String? ?? 'turn-end';
  bool _busy = false;
  Object? _error;

  bool get _adopting => widget.found != null;
  String? get _nativeAgent =>
      _event.contains(':') ? _event.split(':').first : null;
  bool get _matchTools => _event == 'before-tool' || _event == 'after-tool';
  bool get _matcherVisible =>
      _event == 'session-start' ||
      (_nativeAgent != null &&
          (_nativeAgent != 'agy' ||
              _event == 'agy:PreToolUse' ||
              _event == 'agy:PostToolUse'));
  bool get _antigravitySupported =>
      _event != 'prompt' &&
      (_event != 'session-start' || _matcher.text.trim().isEmpty);

  @override
  void dispose() {
    for (final controller in [
      _name,
      _description,
      _command,
      _matcher,
      _timeout,
    ]) {
      controller.dispose();
    }
    super.dispose();
  }

  Future<void> _save() async {
    setState(() {
      _busy = true;
      _error = null;
    });
    try {
      final name = _name.text.trim();
      final result = _adopting
          ? await api.adoptAgentHook(
              key: widget.found!['key'] as String,
              name: name,
            )
          : await api.saveAgentHook(
              source: jsonEncode({
                'name': name,
                'description': _description.text.trim(),
                'event': _event,
                'matcher': _nativeAgent == null
                    ? _matcher.text.trim()
                    : _matcher.text,
                'command': _command.text,
                'timeout': _timeout.text.trim().isEmpty
                    ? 0
                    : int.parse(_timeout.text.trim()),
                'agents': _agentsSelected.toList(),
              }),
              old: _hook['name'] as String?,
              revision: widget.saved?['revision'] as String?,
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
    final editable = !_busy && !_adopting;
    final events = [
      'turn-end',
      'after-tool',
      'before-tool',
      'prompt',
      'session-start',
      for (final agent in widget.events.entries)
        for (final event in agent.value as List) '${agent.key}:$event',
    ];
    return PopScope(
      canPop: !_busy,
      child: AlertDialog(
        title: Text(
          _adopting
              ? strings.adoptHook
              : widget.saved == null
              ? strings.addHook
              : strings.editHook,
        ),
        scrollable: true,
        content: SizedBox(
          width: 450,
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.stretch,
            spacing: 12,
            children: [
              TextField(
                controller: _name,
                enabled: !_busy,
                onChanged: (_) => setState(() {}),
                decoration: InputDecoration(labelText: strings.hookName),
              ),
              if (!_adopting)
                TextField(
                  controller: _description,
                  enabled: editable,
                  decoration: InputDecoration(
                    labelText: strings.hookDescription,
                  ),
                ),
              if (_adopting)
                Text(_eventName(context, _event))
              else
                DropdownButtonFormField<String>(
                  icon: const Icon(CrabforgeSymbols.arrowDropDown),
                  initialValue: _event,
                  isExpanded: true,
                  decoration: InputDecoration(labelText: strings.hookEvent),
                  items: [
                    for (final event in events)
                      DropdownMenuItem(
                        value: event,
                        child: _line(_eventName(context, event)),
                      ),
                  ],
                  onChanged: editable
                      ? (event) => setState(() {
                          _event = event!;
                          _matcher.clear();
                          if (_nativeAgent case final String agent) {
                            _agentsSelected.retainAll({agent});
                          }
                          if (_event == 'prompt') _agentsSelected.remove('agy');
                        })
                      : null,
                ),
              if (_matchTools)
                DropdownButtonFormField<String>(
                  icon: const Icon(CrabforgeSymbols.arrowDropDown),
                  key: ValueKey(_event),
                  initialValue: _matcher.text,
                  decoration: InputDecoration(labelText: strings.hookMatcher),
                  items: [
                    DropdownMenuItem(
                      value: '',
                      child: Text(strings.hookAllTools),
                    ),
                    DropdownMenuItem(
                      value: 'shell',
                      child: Text(strings.hookShellTool),
                    ),
                    DropdownMenuItem(
                      value: 'edit',
                      child: Text(strings.hookEditTool),
                    ),
                  ],
                  onChanged: editable
                      ? (matcher) => _matcher.text = matcher!
                      : null,
                )
              else if (_matcherVisible)
                TextField(
                  controller: _matcher,
                  enabled: editable,
                  onChanged: (_) => setState(() {
                    if (!_antigravitySupported) _agentsSelected.remove('agy');
                  }),
                  decoration: InputDecoration(
                    labelText: strings.hookMatcher,
                    hintText: _event == 'session-start'
                        ? 'startup|resume'
                        : null,
                  ),
                ),
              TextField(
                controller: _command,
                readOnly: !editable,
                maxLines: 4,
                minLines: 2,
                onChanged: (_) => setState(() {}),
                decoration: InputDecoration(labelText: strings.hookCommand),
              ),
              TextField(
                controller: _timeout,
                enabled: editable,
                keyboardType: TextInputType.number,
                onChanged: (_) => setState(() {}),
                decoration: InputDecoration(labelText: strings.hookTimeout),
              ),
              Wrap(
                spacing: 6,
                runSpacing: 6,
                children: [
                  for (final agent in _agents.entries)
                    if ((_nativeAgent == null || _nativeAgent == agent.key) &&
                        (agent.key != 'agy' || _antigravitySupported))
                      FilterChip(
                        label: Text(agent.value),
                        selected: _agentsSelected.contains(agent.key),
                        onSelected: !editable
                            ? null
                            : (selected) => setState(() {
                                if (selected) {
                                  _agentsSelected.add(agent.key);
                                } else {
                                  _agentsSelected.remove(agent.key);
                                }
                              }),
                      ),
                ],
              ),
              if (_error != null) SelectableText('$_error'),
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
            onPressed:
                !_busy &&
                    _name.text.trim().isNotEmpty &&
                    _command.text.trim().isNotEmpty &&
                    (_timeout.text.trim().isEmpty ||
                        int.tryParse(_timeout.text.trim()) != null)
                ? _save
                : null,
            child: Text(_adopting ? strings.adoptHook : strings.save),
          ),
        ],
      ),
    );
  }
}
