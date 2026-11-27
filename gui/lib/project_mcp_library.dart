part of 'project_mcp.dart';

class _ProjectMcpLibrary extends StatefulWidget {
  const _ProjectMcpLibrary({required this.project, required this.agent});

  final String project, agent;

  @override
  State<_ProjectMcpLibrary> createState() => _ProjectMcpLibraryState();
}

class _ProjectMcpLibraryState extends State<_ProjectMcpLibrary> {
  _View? _library;
  Object? _error;
  bool _busy = true;

  @override
  void initState() {
    super.initState();
    _load(api.projectMcpLibrary);
  }

  Future<void> _load(Future<String> Function() run) async {
    setState(() {
      _busy = true;
      _error = null;
    });
    try {
      final result = jsonDecode(await run()) as _View;
      if (mounted) setState(() => _library = result);
    } catch (error) {
      if (mounted) setState(() => _error = error);
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  Future<void> _edit([_View? entry]) async {
    final server = entry?['server'] as _View?;
    final result = await showDialog<_View>(
      context: context,
      barrierDismissible: false,
      builder: (_) => _ServerEditor(
        name: entry?['name'] as String?,
        source: const JsonEncoder.withIndent('  ').convert(
          server?['config'] ??
              {
                'transport': 'stdio',
                'command': 'path-to-server',
                'args': <String>[],
              },
        ),
        syntax: 'json',
        save: (name, source) => api.saveMcpLibraryServer(
          name: name,
          source: source,
          revision: entry?['revision'] as String?,
        ),
      ),
    );
    if (mounted && result != null) {
      setState(() {
        _library = result;
        _error = null;
      });
    }
  }

  Future<void> _remove(_View entry) async {
    final strings = context.strings;
    final name = entry['name'] as String;
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
      await _load(
        () => api.removeMcpLibraryServer(
          name: name,
          revision: entry['revision'] as String,
        ),
      );
    }
  }

  bool _matches(_View value) =>
      value['project'] == widget.project && value['agent'] == widget.agent;

  @override
  Widget build(BuildContext context) {
    final strings = context.strings;
    final grid = CollectionLayout.of(context).grid;
    final servers = (_library?['servers'] as List?)?.cast<_View>() ?? [];
    final placements = (_library?['placements'] as List?)?.cast<_View>() ?? [];
    final current = {
      for (final placement in placements.where(_matches))
        placement['name'] as String: placement,
    };
    final savedNames = servers.map((entry) => entry['name']).toSet();
    final retained = current.values.where(
      (placement) =>
          !savedNames.contains(placement['name']) &&
          (placement['detail'] != null || placement['status'] == 'conflict'),
    );
    return Dialog(
      key: const ValueKey('project-mcp-library'),
      child: SizedBox(
        width: 760,
        height: 590,
        child: Padding(
          padding: const EdgeInsets.all(20),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              DefaultTextStyle(
                style: Theme.of(context).textTheme.titleLarge!,
                child: EditorHeading(strings.mcpLibrary),
              ),
              SelectableText('${widget.agent} · ${widget.project}'),
              Wrap(
                children: [
                  TextButton.icon(
                    onPressed: _busy ? null : _edit,
                    icon: const Icon(CrabforgeSymbols.add),
                    label: Text(strings.addMcpServer),
                  ),
                  TextButton.icon(
                    onPressed: _busy ? null : () => _load(api.syncMcpLibrary),
                    icon: const Icon(CrabforgeSymbols.sync),
                    label: Text(strings.syncMcpLibrary),
                  ),
                ],
              ),
              if (_busy) const LinearProgressIndicator(),
              Expanded(
                child: CustomScrollView(
                  slivers: [
                    if (_error != null)
                      SliverToBoxAdapter(child: SelectableText('$_error')),
                    if (!_busy && servers.isEmpty)
                      SliverToBoxAdapter(child: Text(strings.noMcpServers)),
                    CollectionSliver.builder(
                      grid: grid,
                      height: 130,
                      itemCount: servers.length,
                      itemBuilder: (context, index) {
                        final entry = servers[index];
                        final name = entry['name'] as String;
                        final server = entry['server'] as _View;
                        final assignments = (server['assignments'] as List)
                            .cast<_View>();
                        final assigned = assignments.any(_matches);
                        final placement = current[name];
                        return CollectionTile(
                          key: ValueKey(name),
                          grid: grid,
                          title: Text(name),
                          leading: const Icon(CrabforgeSymbols.dns),
                          onTap: _busy ? null : () => _edit(entry),
                          detail: placement == null
                              ? null
                              : Column(
                                  crossAxisAlignment: CrossAxisAlignment.start,
                                  children: [
                                    Text(
                                      strings.ruleStatus(
                                        placement['status'] as String,
                                      ),
                                    ),
                                    if (placement['detail']
                                        case final String detail)
                                      SelectableText(detail),
                                  ],
                                ),
                          actions: [
                            IconButton(
                              onPressed: _busy
                                  ? null
                                  : () => _load(
                                      () => api.assignMcpLibraryServer(
                                        name: name,
                                        project: widget.project,
                                        agent: widget.agent,
                                        enabled: !assigned,
                                        revision: entry['revision'] as String,
                                      ),
                                    ),
                              tooltip: assigned
                                  ? strings.unassignMcpServer
                                  : strings.assignMcpServer,
                              icon: Icon(
                                assigned
                                    ? CrabforgeSymbols.linkOff
                                    : CrabforgeSymbols.link,
                              ),
                            ),
                            IconButton(
                              onPressed: _busy ? null : () => _edit(entry),
                              tooltip: strings.editMcpServer,
                              icon: const Icon(CrabforgeSymbols.edit),
                            ),
                            IconButton(
                              onPressed: _busy ? null : () => _remove(entry),
                              tooltip: strings.remove,
                              icon: const Icon(CrabforgeSymbols.delete),
                            ),
                          ],
                        );
                      },
                    ),
                    if (retained.isNotEmpty)
                      CollectionSliver(
                        grid: grid,
                        height: 130,
                        children: [
                          for (final placement in retained)
                            CollectionTile(
                              grid: grid,
                              leading: const Icon(CrabforgeSymbols.dns),
                              title: Text(placement['name'] as String),
                              detail: SelectableText(
                                placement['detail'] as String? ??
                                    strings.ruleStatus(
                                      placement['status'] as String,
                                    ),
                              ),
                            ),
                        ],
                      ),
                  ],
                ),
              ),
              Align(
                alignment: AlignmentDirectional.centerEnd,
                child: TextButton(
                  onPressed: _busy ? null : () => Navigator.pop(context),
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
