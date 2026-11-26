import 'dart:convert';

import 'package:file_selector/file_selector.dart';
import 'package:flutter/material.dart';

import 'collection_view.dart';
import 'language.dart';
import 'src/rust/api.dart' as api;

part 'project_mcp_library.dart';

const projectMcpAgents = {
  'agy',
  'antigravity',
  'claude',
  'codebuddy',
  'codex',
  'cursor',
  'droid',
  'gemini',
  'opencode',
  'pi',
  'qoder',
  'qoder-cn',
  'trae-cn',
  'zcode',
};

typedef _View = Map<String, dynamic>;

class ProjectMcp extends StatefulWidget {
  const ProjectMcp({super.key, required this.agent});

  final String agent;

  @override
  State<ProjectMcp> createState() => _ProjectMcpState();
}

class _ProjectMcpState extends State<ProjectMcp> {
  final _folder = TextEditingController();
  String? _project;
  _View? _view;
  Object? _error;
  bool _busy = false;

  @override
  void dispose() {
    _folder.dispose();
    super.dispose();
  }

  Future<void> _run(Future<String> Function() read, {String? project}) async {
    setState(() {
      _busy = true;
      _error = null;
      if (project != null) _view = null;
    });
    try {
      final view = jsonDecode(await read()) as _View;
      if (mounted) {
        setState(() {
          _view = view;
          _error = null;
          if (project != null) _project = project;
        });
      }
    } catch (error) {
      if (mounted) setState(() => _error = error);
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  Future<void> _open() async {
    final project = _folder.text.trim();
    if (project.isEmpty || _busy) return;
    await _run(
      () => api.projectMcpServers(project: project, agent: widget.agent),
      project: project,
    );
  }

  Future<void> _browse() async {
    final project = await getDirectoryPath();
    if (project == null || !mounted) return;
    _folder.text = project;
    await _open();
  }

  Future<void> _edit({String? name, String? source}) async {
    final view = _view!;
    final result = await showDialog<_View>(
      context: context,
      barrierDismissible: false,
      builder: (_) => _ServerEditor(
        name: name,
        source: source ?? view['template'] as String,
        syntax: view['syntax'] as String,
        save: (server, config) => api.saveProjectMcpServer(
          project: _project!,
          agent: widget.agent,
          name: server,
          source: config,
          revision: view['revision'] as String,
          replace: name != null,
        ),
      ),
    );
    if (mounted && result != null) {
      setState(() {
        _view = result;
        _error = null;
      });
    }
  }

  Future<void> _remove(String name) async {
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
    if (!mounted || confirmed != true) return;
    await _run(
      () => api.removeProjectMcpServer(
        project: _project!,
        agent: widget.agent,
        name: name,
        revision: _view!['revision'] as String,
      ),
    );
  }

  Future<void> _library() async {
    await showDialog<void>(
      context: context,
      builder: (_) => _ProjectMcpLibrary(
        project: _view!['project'] as String,
        agent: widget.agent,
      ),
    );
    if (mounted) await _open();
  }

  @override
  Widget build(BuildContext context) {
    final strings = context.strings;
    final grid = CollectionLayout.of(context).grid;
    final servers =
        (_view?['servers'] as Map?)?.cast<String, String>().entries.toList() ??
        [];
    final loaded = _view != null && _project == _folder.text.trim();
    final editable = !_busy && loaded;
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
                child: EditorHeading(strings.projectMcp),
              ),
              TextField(
                controller: _folder,
                enabled: !_busy,
                onChanged: (_) => setState(() {}),
                onSubmitted: (_) => _open(),
                decoration: InputDecoration(
                  labelText: strings.projectFolder,
                  suffixIcon: IconButton(
                    onPressed: _busy ? null : _browse,
                    tooltip: strings.projectFolder,
                    icon: const Icon(Icons.folder_open_outlined),
                  ),
                ),
              ),
              const SizedBox(height: 8),
              Wrap(
                children: [
                  TextButton.icon(
                    onPressed: _busy || _folder.text.trim().isEmpty
                        ? null
                        : _open,
                    icon: Icon(
                      loaded ? Icons.refresh : Icons.folder_open_outlined,
                    ),
                    label: Text(loaded ? strings.reload : strings.openProject),
                  ),
                  if (loaded)
                    TextButton.icon(
                      onPressed: editable ? () => _edit() : null,
                      icon: const Icon(Icons.add),
                      label: Text(strings.addMcpServer),
                    ),
                  if (loaded)
                    TextButton.icon(
                      onPressed: editable ? _library : null,
                      icon: const Icon(Icons.dns_outlined),
                      label: Text(strings.mcpLibrary),
                    ),
                ],
              ),
              if (_busy) const LinearProgressIndicator(),
              Expanded(
                child: CustomScrollView(
                  slivers: [
                    if (_error != null)
                      SliverToBoxAdapter(child: SelectableText('$_error')),
                    if (loaded) ...[
                      SliverToBoxAdapter(
                        child: Padding(
                          padding: const EdgeInsets.symmetric(vertical: 8),
                          child: SelectableText(_view!['path'] as String),
                        ),
                      ),
                      if (servers.isEmpty)
                        SliverToBoxAdapter(child: Text(strings.noMcpServers)),
                      CollectionSliver.builder(
                        grid: grid,
                        height: 96,
                        itemCount: servers.length,
                        itemBuilder: (context, index) {
                          final server = servers[index];
                          return CollectionTile(
                            key: ValueKey(server.key),
                            grid: grid,
                            title: Text(server.key),
                            leading: const Icon(Icons.dns_outlined),
                            onTap: editable
                                ? () => _edit(
                                    name: server.key,
                                    source: server.value,
                                  )
                                : null,
                            actions: [
                              IconButton(
                                onPressed: editable
                                    ? () => _edit(
                                        name: server.key,
                                        source: server.value,
                                      )
                                    : null,
                                tooltip: strings.editMcpServer,
                                icon: const Icon(Icons.edit_outlined),
                              ),
                              IconButton(
                                onPressed: editable
                                    ? () => _remove(server.key)
                                    : null,
                                tooltip: strings.remove,
                                icon: const Icon(Icons.delete_outline),
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

class _ServerEditor extends StatefulWidget {
  const _ServerEditor({
    this.name,
    required this.source,
    required this.syntax,
    required this.save,
  });

  final String? name;
  final String source, syntax;
  final Future<String> Function(String name, String source) save;

  @override
  State<_ServerEditor> createState() => _ServerEditorState();
}

class _ServerEditorState extends State<_ServerEditor> {
  late final _name = TextEditingController(text: widget.name);
  late final _config = TextEditingController(text: widget.source);
  bool _busy = false;
  Object? _error;

  @override
  void dispose() {
    _name.dispose();
    _config.dispose();
    super.dispose();
  }

  Future<void> _save() async {
    setState(() {
      _busy = true;
      _error = null;
    });
    try {
      final view = jsonDecode(
        await widget.save(_name.text.trim(), _config.text),
      ) as _View;
      if (mounted) Navigator.pop(context, view);
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
        title: Text(widget.name ?? strings.addMcpServer),
        scrollable: true,
        content: SizedBox(
          width: 500,
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              if (widget.name == null) ...[
                TextField(
                  controller: _name,
                  enabled: !_busy,
                  onChanged: (_) => setState(() {}),
                  decoration: InputDecoration(labelText: strings.mcpServerName),
                ),
                const SizedBox(height: 12),
              ],
              TextField(
                controller: _config,
                enabled: !_busy,
                minLines: 5,
                maxLines: 12,
                autocorrect: false,
                enableSuggestions: false,
                smartDashesType: SmartDashesType.disabled,
                smartQuotesType: SmartQuotesType.disabled,
                decoration: InputDecoration(
                  labelText: widget.syntax.toUpperCase(),
                ),
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
            onPressed: _busy || _name.text.trim().isEmpty ? null : _save,
            child: Text(strings.save),
          ),
        ],
      ),
    );
  }
}
