import 'dart:convert';

import 'package:flutter/material.dart';

import 'collection_view.dart';
import 'language.dart';
import 'src/rust/api.dart' as api;

typedef _Json = Map<String, dynamic>;

const _agents = {
  'claude': 'Claude Code',
  'codex': 'Codex',
  'gemini': 'Gemini CLI',
  'cursor': 'Cursor',
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
  bool _busy = false;
  Object? _error;

  @override
  void initState() {
    super.initState();
    _load(api.agentHooks);
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

  void _inspect(_Json row) {
    showDialog<void>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(row['event'] as String),
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
    final saved = {
      for (final row in (_view['hooks'] as List? ?? const []).cast<_Json>())
        (row['hook'] as _Json)['name'] as String: row,
    };
    final placements = <String, List<_Json>>{};
    for (final row
        in (_view['placements'] as List? ?? const []).cast<_Json>()) {
      (placements[row['name'] as String] ??= []).add(row);
    }
    final names = {...saved.keys, ...placements.keys}.toList();
    final installed = (_view['installed'] as List? ?? const []).cast<_Json>();
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
                children: [
                  TextButton.icon(
                    onPressed: _busy ? null : () => _edit(),
                    icon: const Icon(Icons.add),
                    label: Text(strings.addHook),
                  ),
                  TextButton.icon(
                    onPressed: _busy ? null : () => _load(api.syncAgentHooks),
                    icon: const Icon(Icons.sync),
                    label: Text(strings.syncMcpLibrary),
                  ),
                  IconButton(
                    onPressed: _busy ? null : () => _load(api.agentHooks),
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
                    for (final row in _view['errors'] as List? ?? const [])
                      SliverToBoxAdapter(
                        child: SelectableText('${row[0]['path']}\n${row[1]}'),
                      ),
                    if (!_busy && names.isEmpty && _error == null)
                      SliverToBoxAdapter(child: Text(strings.noHooks)),
                    CollectionSliver.builder(
                      grid: grid,
                      height: 210,
                      itemCount: names.length,
                      itemBuilder: (context, index) {
                        final name = names[index];
                        final entry = saved[name];
                        final hook = entry?['hook'] as _Json?;
                        final description = hook?['description'] as String?;
                        return CollectionTile(
                          key: ValueKey(name),
                          grid: grid,
                          leading: const Icon(Icons.bolt_outlined),
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
                          onTap: _busy || entry == null
                              ? null
                              : () => _edit(saved: entry),
                          actions: [
                            if (entry != null) ...[
                              IconButton(
                                onPressed: _busy
                                    ? null
                                    : () => _edit(saved: entry),
                                icon: const Icon(Icons.edit_outlined),
                                tooltip: strings.editHook,
                              ),
                              IconButton(
                                onPressed: _busy
                                    ? null
                                    : () => _remove(
                                        name,
                                        entry['revision'] as String,
                                      ),
                                icon: const Icon(Icons.delete_outline),
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
                            leading: const Icon(Icons.article_outlined),
                            title: _line(
                              '${_agents[target['agent']]} · ${row['event']}',
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
                              if (hook != null)
                                IconButton(
                                  onPressed: _busy
                                      ? null
                                      : () => _edit(found: row),
                                  icon: const Icon(Icons.bookmark_add_outlined),
                                  tooltip: strings.adoptHook,
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
                        })
                      : null,
                ),
              if (_matchTools)
                DropdownButtonFormField<String>(
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
              else if (_nativeAgent != null || _event == 'session-start')
                TextField(
                  controller: _matcher,
                  enabled: editable,
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
                    if (_nativeAgent == null || _nativeAgent == agent.key)
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
