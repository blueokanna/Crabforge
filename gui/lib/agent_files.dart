import 'dart:convert';

import 'package:file_selector/file_selector.dart';
import 'package:flutter/material.dart';

import 'collection_view.dart';
import 'language.dart';
import 'src/rust/api.dart' as api;

typedef _Json = Map<String, dynamic>;

const _agents = {
  'claude': 'Claude Code',
  'codex': 'Codex',
  'opencode': 'OpenCode',
  'pi': 'Pi',
  'gemini': 'Gemini CLI',
  'qwen': 'Qwen Code',
  'droid': 'Factory Droid',
  'grok': 'Grok Build',
  'cursor': 'Cursor',
  'copilot': 'Copilot CLI',
  'agy': 'Antigravity',
};

class AgentFiles extends StatefulWidget {
  const AgentFiles({super.key, required this.subagents});

  final bool subagents;

  @override
  State<AgentFiles> createState() => _AgentFilesState();
}

class _AgentFilesState extends State<AgentFiles> {
  _Json _view = const {};
  String? _project;
  Object? _error;
  bool _busy = true;

  @override
  void initState() {
    super.initState();
    _load('list');
  }

  Future<String> _request(
    String action, {
    String? source,
    String? name,
    String? revision,
  }) => api.agentFiles(
    subagents: widget.subagents,
    project: _project,
    action: action,
    source: source,
    name: name,
    revision: revision,
  );

  Future<void> _load(
    String action, {
    String? name,
    String? revision,
    String? source,
  }) async {
    setState(() {
      _busy = true;
      _error = null;
    });
    try {
      final response = await _request(
        action,
        name: name,
        revision: revision,
        source: source,
      );
      if (mounted) setState(() => _view = jsonDecode(response) as _Json);
    } catch (error) {
      if (mounted) setState(() => _error = error);
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  Future<void> _discover() async {
    setState(() {
      _busy = true;
      _error = null;
    });
    final _Json discovery;
    try {
      discovery = jsonDecode(await _request('discover')) as _Json;
    } catch (error) {
      if (mounted) setState(() => _error = error);
      return;
    } finally {
      if (mounted) setState(() => _busy = false);
    }
    if (!mounted) return;
    final candidate = await showDialog<_Json>(
      context: context,
      builder: (context) => _NativeResources(discovery: discovery),
    );
    if (!mounted || candidate == null) return;
    final strings = context.strings;
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(strings.adoptSkill),
        content: Text(strings.adoptResourceDetails),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context, false),
            child: Text(strings.cancel),
          ),
          FilledButton(
            onPressed: () => Navigator.pop(context, true),
            child: Text(strings.adoptSkill),
          ),
        ],
      ),
    );
    if (mounted && confirmed == true) {
      await _load(
        'adopt',
        name: candidate['agent'] as String,
        source: candidate['path'] as String,
        revision: candidate['revision'] as String,
      );
    }
  }

  Future<void> _chooseProject() async {
    final project = await getDirectoryPath(initialDirectory: _project);
    if (mounted && project != null) _scope(project);
  }

  Future<void> _import([String? source]) async {
    final result = await showDialog<String>(
      context: context,
      barrierDismissible: false,
      builder: (_) => _ImportResources(
        subagents: widget.subagents,
        source: source,
        scan: (source) => _request('probe-source', source: source),
        save: (source) => _request('import-source', source: source),
      ),
    );
    if (mounted && result != null) {
      setState(() {
        _view = jsonDecode(result) as _Json;
        _error = null;
      });
    }
  }

  void _scope(String? path) {
    setState(() {
      _project = path;
      _view = const {};
    });
    _load('list');
  }

  Future<void> _edit([_Json? row]) async {
    final entry = row?['entry'] as _Json?;
    final response = await showDialog<String>(
      context: context,
      barrierDismissible: false,
      builder: (_) => _EntryEditor(
        subagents: widget.subagents,
        agents: (_view['agents'] as List).cast<String>(),
        entry: entry,
        save: (source) => _request(
          'save',
          source: source,
          name: entry?['name'] as String?,
          revision: row?['revision'] as String?,
        ),
      ),
    );
    if (mounted && response != null) {
      setState(() {
        _view = jsonDecode(response) as _Json;
        _error = null;
      });
    }
  }

  Future<void> _remove(String name, String revision) async {
    final strings = context.strings;
    final confirm = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(strings.confirmRemove(name)),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context, false),
            child: Text(strings.cancel),
          ),
          TextButton(
            onPressed: () => Navigator.pop(context, true),
            child: Text(strings.remove),
          ),
        ],
      ),
    );
    if (mounted && confirm == true) {
      await _load('remove', name: name, revision: revision);
    }
  }

  void _placements(String name, List<_Json> rows) => showDialog<void>(
    context: context,
    builder: (context) => AlertDialog(
      title: Text(name),
      scrollable: true,
      content: SizedBox(
        width: 600,
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          spacing: 12,
          children: [
            for (final row in rows)
              SelectableText(
                [
                  _status(context, row),
                  if (row['path'] case final String path) path,
                  if (row['error'] case final String error) error,
                ].join('\n'),
              ),
          ],
        ),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.pop(context),
          child: Text(MaterialLocalizations.of(context).closeButtonLabel),
        ),
      ],
    ),
  );

  String _status(BuildContext context, _Json row) =>
      '${_agents[row['agent']]} · ${context.strings.ruleStatus(row['status'] as String)}';

  @override
  Widget build(BuildContext context) {
    final strings = context.strings;
    final subagents = widget.subagents;
    final title = subagents ? strings.agentSubagents : strings.agentCommands;
    final grid = CollectionLayout.of(context).grid;
    final saved = {
      for (final row in (_view['entries'] as List? ?? const []).cast<_Json>())
        (row['entry'] as _Json)['name'] as String: row,
    };
    final placements = <String, List<_Json>>{};
    for (final row
        in (_view['placements'] as List? ?? const []).cast<_Json>()) {
      (placements[row['name'] as String] ??= []).add(row);
    }
    final names = {...saved.keys, ...placements.keys}.toList()..sort();
    final available = !_busy && _view.containsKey('agents');
    return Dialog(
      key: ValueKey(subagents ? 'agent-subagents' : 'agent-commands'),
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
                child: EditorHeading(title),
              ),
              if (subagents)
                Wrap(
                  spacing: 8,
                  children: [
                    ChoiceChip(
                      label: Text(strings.resourceUserScope),
                      selected: _project == null,
                      onSelected: !_busy && _project != null
                          ? (_) => _scope(null)
                          : null,
                    ),
                    ActionChip(
                      avatar: const Icon(Icons.folder_open_outlined),
                      label: Text(strings.openProject),
                      onPressed: _busy ? null : _chooseProject,
                    ),
                    if (_project case final String path)
                      Tooltip(message: path, child: _line(path)),
                  ],
                ),
              Wrap(
                children: [
                  TextButton.icon(
                    onPressed: available ? _edit : null,
                    icon: const Icon(Icons.add),
                    label: Text(
                      subagents ? strings.addSubagent : strings.addSlashCommand,
                    ),
                  ),
                  TextButton.icon(
                    onPressed: available ? _discover : null,
                    icon: const Icon(Icons.folder_copy_outlined),
                    label: Text(strings.findNativeResources),
                  ),
                  if (_project == null)
                    TextButton.icon(
                      onPressed: available ? _import : null,
                      icon: const Icon(Icons.file_download_outlined),
                      label: Text(
                        subagents
                            ? strings.importSubagents
                            : strings.importSlashCommands,
                      ),
                    ),
                  TextButton.icon(
                    onPressed: available ? () => _load('sync') : null,
                    icon: const Icon(Icons.sync),
                    label: Text(strings.syncMcpLibrary),
                  ),
                  IconButton(
                    onPressed: _busy ? null : () => _load('list'),
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
                    if (available && names.isEmpty)
                      SliverToBoxAdapter(child: Text(strings.noAgentResources)),
                    CollectionSliver.builder(
                      grid: grid,
                      height: 125,
                      itemCount: names.length,
                      itemBuilder: (context, index) {
                        final name = names[index];
                        final row = saved[name];
                        final entry = row?['entry'] as _Json?;
                        final status = placements[name] ?? const <_Json>[];
                        final issue = status
                            .where(
                              (item) =>
                                  item['status'] == 'error' ||
                                  item['status'] == 'conflict',
                            )
                            .firstOrNull;
                        return CollectionTile(
                          key: ValueKey(name),
                          grid: grid,
                          leading: Icon(
                            subagents
                                ? Icons.group_work_outlined
                                : Icons.shortcut_outlined,
                          ),
                          title: _line(subagents ? name : '/$name'),
                          detail: Column(
                            crossAxisAlignment: CrossAxisAlignment.start,
                            children: [
                              if (entry?['description']
                                  case final String description
                                  when description.isNotEmpty)
                                _line(description),
                              if (issue != null)
                                _line(_status(context, issue))
                              else if (status.isNotEmpty)
                                _line(
                                  status
                                      .map((item) => _agents[item['agent']])
                                      .toSet()
                                      .join(' · '),
                                ),
                              if (row?['source'] case final String source)
                                Tooltip(message: source, child: _line(source)),
                            ],
                          ),
                          onTap: available && row != null
                              ? () => _edit(row)
                              : null,
                          actions: [
                            if (row?['source'] case final String source)
                              IconButton(
                                onPressed: available
                                    ? () => _import(source)
                                    : null,
                                icon: const Icon(Icons.update_outlined),
                                tooltip: strings.checkSkillUpdate,
                              ),
                            if (status.isNotEmpty)
                              IconButton(
                                onPressed: () => _placements(name, status),
                                icon: const Icon(Icons.info_outline),
                                tooltip: strings.resourcePlacements,
                              ),
                            if (row != null)
                              IconButton(
                                onPressed: available
                                    ? () => _remove(
                                        name,
                                        row['revision'] as String,
                                      )
                                    : null,
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

class _ImportResources extends StatefulWidget {
  const _ImportResources({
    required this.subagents,
    this.source,
    required this.scan,
    required this.save,
  });

  final bool subagents;
  final String? source;
  final Future<String> Function(String source) scan;
  final Future<String> Function(String source) save;

  @override
  State<_ImportResources> createState() => _ImportResourcesState();
}

class _ImportResourcesState extends State<_ImportResources> {
  late final _source = TextEditingController(text: widget.source);
  final _selected = <String>{};
  final _clients = <String>{};
  _Json? _preview;
  Object? _error;
  bool _busy = false;

  @override
  void dispose() {
    _source.dispose();
    super.dispose();
  }

  void _changeSource() => setState(() {
    _preview = null;
    _error = null;
    _selected.clear();
  });

  Future<void> _pick({required bool folder}) async {
    final path = folder
        ? await getDirectoryPath()
        : (await openFile(
            acceptedTypeGroups: [
              if (widget.subagents)
                const XTypeGroup(label: 'Markdown', extensions: ['md'])
              else
                const XTypeGroup(
                  label: 'Markdown, TOML',
                  extensions: ['md', 'toml'],
                ),
            ],
          ))?.path;
    if (!mounted || path == null) return;
    _source.text = path;
    _changeSource();
  }

  Future<void> _scan() async {
    setState(() {
      _busy = true;
      _error = null;
      _preview = null;
      _selected.clear();
    });
    try {
      final preview =
          jsonDecode(await widget.scan(_source.text.trim())) as _Json;
      if (mounted) setState(() => _preview = preview);
    } catch (error) {
      if (mounted) setState(() => _error = error);
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  Future<void> _save(List<_Json> candidates) async {
    setState(() {
      _busy = true;
      _error = null;
    });
    try {
      final result = await widget.save(
        jsonEncode({
          'source': _source.text.trim(),
          'agents': _clients.toList(),
          'files': [
            for (final row in candidates)
              if (_selected.contains(row['path']))
                {'path': row['path'], 'revision': row['revision']},
          ],
        }),
      );
      if (mounted) Navigator.pop(context, result);
    } catch (error) {
      if (mounted) setState(() => _error = error);
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    final strings = context.strings;
    final grid = CollectionLayout.of(context).grid;
    final subagents = widget.subagents;
    final title = subagents
        ? strings.importSubagents
        : strings.importSlashCommands;
    final candidates =
        (_preview?['candidates'] as List?)?.cast<_Json>() ?? const <_Json>[];
    final agents =
        (_preview?['agents'] as List?)?.cast<String>() ?? const <String>[];
    final anyNew = candidates.any(
      (row) => row['status'] == 'new' && _selected.contains(row['path']),
    );
    return Dialog(
      key: ValueKey(subagents ? 'import-subagents' : 'import-commands'),
      child: SizedBox(
        width: 800,
        height: 640,
        child: Padding(
          padding: const EdgeInsets.all(20),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            spacing: 8,
            children: [
              DefaultTextStyle(
                style: Theme.of(context).textTheme.titleLarge!,
                child: EditorHeading(title),
              ),
              TextField(
                controller: _source,
                enabled: !_busy,
                onChanged: (_) => _changeSource(),
                onSubmitted: (_) {
                  if (_source.text.trim().isNotEmpty && !_busy) _scan();
                },
                decoration: InputDecoration(
                  labelText: subagents
                      ? strings.subagentImportSource
                      : strings.commandImportSource,
                ),
              ),
              Wrap(
                spacing: 8,
                children: [
                  IconButton(
                    onPressed: _busy ? null : () => _pick(folder: true),
                    tooltip: strings.subagentChooseFolder,
                    icon: const Icon(Icons.folder_open_outlined),
                  ),
                  IconButton(
                    onPressed: _busy ? null : () => _pick(folder: false),
                    tooltip: subagents
                        ? strings.subagentChooseFile
                        : strings.commandChooseFile,
                    icon: const Icon(Icons.insert_drive_file_outlined),
                  ),
                  OutlinedButton(
                    onPressed: _busy || _source.text.trim().isEmpty
                        ? null
                        : _scan,
                    child: Text(strings.subagentScanSource),
                  ),
                ],
              ),
              if (_busy) const LinearProgressIndicator(),
              Expanded(
                child: CustomScrollView(
                  slivers: [
                    if (_error != null)
                      SliverToBoxAdapter(child: SelectableText('$_error')),
                    if (_preview != null && candidates.isEmpty)
                      SliverToBoxAdapter(
                        child: Text(strings.noNativeResources),
                      ),
                    if (_preview?['truncated'] == true)
                      SliverToBoxAdapter(
                        child: Text(strings.subagentImportTruncated),
                      ),
                    if (anyNew)
                      SliverToBoxAdapter(
                        child: Wrap(
                          spacing: 8,
                          children: [
                            for (final agent in agents)
                              FilterChip(
                                selected: _clients.contains(agent),
                                label: Text(_agents[agent] ?? agent),
                                onSelected: _busy
                                    ? null
                                    : (value) => setState(() {
                                        if (value) {
                                          _clients.add(agent);
                                        } else {
                                          _clients.remove(agent);
                                        }
                                      }),
                              ),
                          ],
                        ),
                      ),
                    CollectionSliver.builder(
                      grid: grid,
                      height: 142,
                      itemCount: candidates.length,
                      itemBuilder: (context, index) {
                        final row = candidates[index];
                        final path = row['path'] as String;
                        final status = row['status'] as String;
                        final selectable =
                            !_busy && (status == 'new' || status == 'update');
                        return CollectionTile(
                          grid: grid,
                          leading: Checkbox(
                            value: _selected.contains(path),
                            onChanged: selectable
                                ? (value) => setState(() {
                                    if (value == true) {
                                      _selected.add(path);
                                    } else {
                                      _selected.remove(path);
                                    }
                                  })
                                : null,
                          ),
                          title: _line(row['name'] as String),
                          detail: Column(
                            crossAxisAlignment: CrossAxisAlignment.start,
                            children: [
                              Tooltip(message: path, child: _line(path)),
                              _line(row['description'] as String),
                              if (status != 'new')
                                _line(strings.subagentImportStatus(status)),
                            ],
                          ),
                        );
                      },
                    ),
                  ],
                ),
              ),
              OverflowBar(
                alignment: MainAxisAlignment.end,
                spacing: 8,
                children: [
                  TextButton(
                    onPressed: _busy ? null : () => Navigator.pop(context),
                    child: Text(strings.cancel),
                  ),
                  FilledButton(
                    onPressed: _busy || _selected.isEmpty
                        ? null
                        : () => _save(candidates),
                    child: Text(title),
                  ),
                ],
              ),
            ],
          ),
        ),
      ),
    );
  }
}

class _NativeResources extends StatelessWidget {
  const _NativeResources({required this.discovery});

  final _Json discovery;

  @override
  Widget build(BuildContext context) {
    final strings = context.strings;
    final grid = CollectionLayout.of(context).grid;
    final candidates = (discovery['candidates'] as List).cast<_Json>();
    return Dialog(
      key: const ValueKey('agent-file-discovery'),
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
                child: EditorHeading(strings.findNativeResources),
              ),
              if (discovery['truncated'] == true)
                Text(strings.nativeResourceScanLimit),
              Expanded(
                child: CustomScrollView(
                  slivers: [
                    if (candidates.isEmpty)
                      SliverToBoxAdapter(
                        child: Text(strings.noNativeResources),
                      ),
                    CollectionSliver.builder(
                      grid: grid,
                      height: 142,
                      itemCount: candidates.length,
                      itemBuilder: (context, index) {
                        final row = candidates[index];
                        final path = row['path'] as String;
                        final status = row['status'] as String;
                        return CollectionTile(
                          grid: grid,
                          leading: const Icon(Icons.description_outlined),
                          title: _line(row['name'] as String),
                          detail: Column(
                            crossAxisAlignment: CrossAxisAlignment.start,
                            children: [
                              _line(
                                _agents[row['agent']] ?? row['agent'] as String,
                              ),
                              Tooltip(message: path, child: _line(path)),
                              if (row['error'] case final String error)
                                Tooltip(message: error, child: _line(error))
                              else if (status == 'conflict')
                                _line(strings.ruleStatus(status))
                              else if (row['description']
                                  case final String description
                                  when description.isNotEmpty)
                                _line(description),
                            ],
                          ),
                          actions: [
                            if (status == 'available')
                              IconButton(
                                onPressed: () => Navigator.pop(context, row),
                                tooltip: strings.adoptSkill,
                                icon: const Icon(Icons.add_link_outlined),
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

Text _line(String text) =>
    Text(text, maxLines: 1, overflow: TextOverflow.ellipsis);

class _EntryEditor extends StatefulWidget {
  const _EntryEditor({
    required this.subagents,
    required this.agents,
    required this.entry,
    required this.save,
  });

  final bool subagents;
  final List<String> agents;
  final _Json? entry;
  final Future<String> Function(String source) save;

  @override
  State<_EntryEditor> createState() => _EntryEditorState();
}

class _EntryEditorState extends State<_EntryEditor> {
  final _form = GlobalKey<FormState>();
  late final _name = _controller('name');
  late final _description = _controller('description');
  late final _prompt = _controller('prompt');
  late final _model = _controller('model');
  late final _skills = _controller('skills');
  late final _mcp = _controller('mcp');
  late final _selected = (widget.entry?['agents'] as List? ?? const [])
      .cast<String>()
      .toSet();
  late String? _effort = widget.entry?['effort'] as String?;
  bool _saving = false;
  Object? _error;

  TextEditingController _controller(String key) => TextEditingController(
    text: switch (widget.entry?[key]) {
      final List values => values.join('\n'),
      final String value => value,
      _ => '',
    },
  );

  @override
  void dispose() {
    for (final controller in [
      _name,
      _description,
      _prompt,
      _model,
      _skills,
      _mcp,
    ]) {
      controller.dispose();
    }
    super.dispose();
  }

  Future<void> _save() async {
    if (!_form.currentState!.validate()) return;
    setState(() {
      _saving = true;
      _error = null;
    });
    List<String> lines(TextEditingController controller) => const LineSplitter()
        .convert(controller.text)
        .map((line) => line.trim())
        .where((line) => line.isNotEmpty)
        .toSet()
        .toList();
    try {
      final result = await widget.save(
        jsonEncode({
          'name': _name.text.trim(),
          'description': _description.text,
          'prompt': _prompt.text,
          'agents': _selected.toList(),
          if (widget.subagents) ...{
            if (_model.text.trim().isNotEmpty) 'model': _model.text.trim(),
            if (_effort != null) 'effort': _effort,
            'skills': lines(_skills),
            'mcp': lines(_mcp),
          },
        }),
      );
      if (mounted) Navigator.pop(context, result);
    } catch (error) {
      if (mounted) {
        setState(() {
          _saving = false;
          _error = error;
        });
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    final strings = context.strings;
    final subagents = widget.subagents;
    Widget field(
      TextEditingController controller,
      String label, {
      bool required = false,
      int minLines = 1,
      int maxLines = 1,
      String? helper,
    }) => TextFormField(
      enabled: !_saving,
      controller: controller,
      decoration: InputDecoration(
        labelText: label,
        helperText: helper,
        helperMaxLines: 3,
      ),
      minLines: minLines,
      maxLines: maxLines,
      validator: required
          ? (value) =>
                value?.trim().isNotEmpty == true ? null : strings.ruleRequired
          : null,
    );
    return PopScope(
      canPop: !_saving,
      child: AlertDialog(
        title: Text(
          widget.entry != null
              ? widget.entry!['name'] as String
              : subagents
              ? strings.addSubagent
              : strings.addSlashCommand,
        ),
        scrollable: true,
        content: SizedBox(
          width: 600,
          child: Form(
            key: _form,
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              spacing: 12,
              children: [
                field(_name, strings.agentResourceName, required: true),
                field(
                  _description,
                  strings.agentResourceDescription,
                  required: subagents,
                  maxLines: 2,
                ),
                field(
                  _prompt,
                  strings.agentResourcePrompt,
                  required: true,
                  minLines: 4,
                  maxLines: 10,
                  helper: subagents ? null : strings.commandArgumentHelp,
                ),
                if (subagents) ...[
                  field(_model, 'Codex · ${strings.roleModelId}'),
                  DropdownButtonFormField<String>(
                    initialValue: _effort ?? '',
                    isExpanded: true,
                    decoration: InputDecoration(
                      labelText:
                          'Claude Code / Codex · ${strings.sessionThinking}',
                    ),
                    items: [
                      DropdownMenuItem(
                        value: '',
                        child: Text(strings.thinkingAuto),
                      ),
                      for (final (id, label) in [
                        ('low', strings.thinkingLow),
                        ('medium', strings.thinkingMedium),
                        ('high', strings.thinkingHigh),
                        ('xhigh', strings.thinkingExtraHigh),
                        ('max', strings.thinkingMax),
                      ])
                        DropdownMenuItem(value: id, child: Text(label)),
                    ],
                    onChanged: _saving
                        ? null
                        : (value) => setState(
                            () => _effort = value == '' ? null : value,
                          ),
                  ),
                  field(
                    _skills,
                    'Claude Code · ${strings.resourceSkillNames}',
                    maxLines: 3,
                  ),
                  field(
                    _mcp,
                    'Claude Code / Factory Droid · ${strings.resourceMcpNames}',
                    maxLines: 3,
                  ),
                ],
                Wrap(
                  spacing: 8,
                  runSpacing: 4,
                  children: [
                    for (final agent in widget.agents)
                      FilterChip(
                        label: Text(_agents[agent] ?? agent),
                        selected: _selected.contains(agent),
                        onSelected: _saving
                            ? null
                            : (selected) => setState(() {
                                if (selected) {
                                  _selected.add(agent);
                                } else {
                                  _selected.remove(agent);
                                }
                              }),
                      ),
                  ],
                ),
                if (_saving) const LinearProgressIndicator(),
                if (_error != null) SelectableText('$_error'),
              ],
            ),
          ),
        ),
        actions: [
          TextButton(
            onPressed: _saving ? null : () => Navigator.pop(context),
            child: Text(strings.cancel),
          ),
          FilledButton(
            onPressed: _saving ? null : _save,
            child: Text(strings.save),
          ),
        ],
      ),
    );
  }
}
