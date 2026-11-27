part of 'plugin_bundles.dart';

const _assignmentsType = XTypeGroup(label: 'JSON', extensions: ['json']);

class _ProjectPackages extends StatefulWidget {
  const _ProjectPackages(this.agent);

  final api.ProjectPluginAgent agent;

  @override
  State<_ProjectPackages> createState() => _ProjectPackagesState();
}

class _ProjectPackagesState extends State<_ProjectPackages> {
  final _folder = TextEditingController();
  String? _project;
  _Json? _view;
  Object? _error;
  bool _busy = false;

  String get _confirmation => switch (widget.agent) {
    api.ProjectPluginAgent.pi => context.strings.piPackageConfirm,
    api.ProjectPluginAgent.claude => context.strings.claudePackageConfirm,
    api.ProjectPluginAgent.codex || api.ProjectPluginAgent.antigravity =>
      context.strings.workspacePluginConfirm,
  };

  @override
  void dispose() {
    _folder.dispose();
    super.dispose();
  }

  Future<void> _run(String project, Future<String> Function() request) async {
    setState(() {
      _busy = true;
      _error = null;
      if (_project != project) _view = null;
    });
    try {
      final view = jsonDecode(await request()) as _Json;
      if (mounted) {
        setState(() {
          _project = project;
          _view = view;
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
    if (_busy || project.isEmpty) return;
    await _run(
      project,
      () => api.projectPluginBundles(agent: widget.agent, project: project),
    );
  }

  Future<void> _browse() async {
    final project = await getDirectoryPath();
    if (!mounted || project == null) return;
    _folder.text = project;
    await _open();
  }

  Future<void> _set(_Json row, {required bool assign}) async {
    final view = _view!;
    final project = _project!;
    final name = row['name'] as String;
    final strings = context.strings;
    if (assign) {
      final confirmed = await showDialog<bool>(
        context: context,
        builder: (context) => AlertDialog(
          title: Text(name),
          content: Text(_confirmation),
          actions: [
            TextButton(
              onPressed: () => Navigator.pop(context, false),
              child: Text(strings.cancel),
            ),
            FilledButton(
              onPressed: () => Navigator.pop(context, true),
              child: Text(strings.save),
            ),
          ],
        ),
      );
      if (!mounted || confirmed != true) return;
    }
    await _run(
      project,
      () => api.setProjectPluginBundle(
        agent: widget.agent,
        project: project,
        name: name,
        projectRevision: view['revision'] as String,
        bundleRevision: assign ? row['bundle_revision'] as String : null,
      ),
    );
  }

  Future<void> _export() async {
    final output = await getSaveLocation(
      suggestedName: '${widget.agent.name}-project-packages.json',
      acceptedTypeGroups: const [_assignmentsType],
    );
    if (!mounted || output == null) return;
    final project = _project!;
    final revision = _view!['revision'] as String;
    setState(() {
      _busy = true;
      _error = null;
    });
    try {
      final text = await api.transferProjectPluginBundles(
        agent: widget.agent,
        project: project,
        projectRevision: revision,
      );
      await XFile.fromData(
        const Utf8Encoder().convert('$text\n'),
        mimeType: 'application/json',
      ).saveTo(output.path);
    } catch (error) {
      if (mounted) setState(() => _error = error);
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  Future<void> _apply() async {
    final file = await openFile(acceptedTypeGroups: const [_assignmentsType]);
    if (!mounted || file == null) return;
    final project = _project!;
    final revision = _view!['revision'] as String;
    final strings = context.strings;
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(file.name),
        content: Text(_confirmation),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context, false),
            child: Text(strings.cancel),
          ),
          FilledButton(
            onPressed: () => Navigator.pop(context, true),
            child: Text(strings.piApplyAssignments),
          ),
        ],
      ),
    );
    if (!mounted || confirmed != true) return;
    await _run(project, () async {
      final bytes = <int>[];
      await for (final chunk in file.openRead(0, 1024 * 1024 + 1)) {
        bytes.addAll(chunk);
      }
      if (bytes.length > 1024 * 1024) {
        throw FormatException(strings.piAssignmentsTooLarge);
      }
      return api.transferProjectPluginBundles(
        agent: widget.agent,
        project: project,
        projectRevision: revision,
        assignments: utf8.decode(bytes),
      );
    });
  }

  Future<void> _archive(String name) async {
    final output = await getSaveLocation(
      suggestedName: '$name.tar',
      acceptedTypeGroups: const [_archiveType],
    );
    if (!mounted || output == null) return;
    setState(() {
      _busy = true;
      _error = null;
    });
    try {
      await api.archiveProjectPluginBundle(
        agent: widget.agent,
        project: _project!,
        name: name,
        projectRevision: _view!['revision'] as String,
        output: output.path,
      );
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
    final rows = (_view?['packages'] as List?) ?? const [];
    return AlertDialog(
      title: EditorHeading(_projectPackagesTitle(context, widget.agent)),
      content: SizedBox(
        width: 680,
        height: 350,
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          spacing: 8,
          children: [
            TextField(
              controller: _folder,
              enabled: !_busy,
              onSubmitted: (_) => _open(),
              decoration: InputDecoration(
                labelText: strings.projectFolder,
                suffixIcon: IconButton(
                  onPressed: _busy ? null : _browse,
                  tooltip: strings.projectFolder,
                  icon: const Icon(CrabforgeSymbols.folderOpen),
                ),
              ),
            ),
            Wrap(
              spacing: 8,
              children: [
                TextButton(
                  onPressed: _busy ? null : _open,
                  child: Text(strings.openProject),
                ),
                if (_view != null) ...[
                  TextButton(
                    onPressed: _busy ? null : _export,
                    child: Text(strings.piExportAssignments),
                  ),
                  TextButton(
                    onPressed: _busy ? null : _apply,
                    child: Text(strings.piApplyAssignments),
                  ),
                ],
              ],
            ),
            if (_busy) const LinearProgressIndicator(),
            Expanded(
              child: CustomScrollView(
                slivers: [
                  if (_error != null)
                    SliverToBoxAdapter(child: SelectableText('$_error')),
                  if (_view != null && rows.isEmpty)
                    SliverToBoxAdapter(child: Text(strings.noPluginBundles)),
                  CollectionSliver.builder(
                    grid: grid,
                    height: 116,
                    itemCount: rows.length,
                    itemBuilder: (context, index) {
                      final row = rows[index] as _Json;
                      final name = row['name'] as String;
                      final status = row['status'] as String;
                      final assigned = row['assigned'] as bool;
                      final version = row['version'] as String?;
                      return CollectionTile(
                        key: ValueKey(name),
                        grid: grid,
                        leading: const Icon(CrabforgeSymbols.inventory2),
                        title: Text(
                          name,
                          maxLines: 1,
                          overflow: TextOverflow.ellipsis,
                        ),
                        detail: Text(
                          [
                            if (version != null && version.isNotEmpty) version,
                            strings.piPackageStatus(status),
                          ].join(' · '),
                        ),
                        actions: [
                          if (status == 'pending' || status == 'available')
                            IconButton(
                              onPressed: _busy
                                  ? null
                                  : () => _set(row, assign: true),
                              tooltip: assigned
                                  ? strings.syncMcpLibrary
                                  : strings.assignMcpServer,
                              icon: Icon(
                                assigned
                                    ? CrabforgeSymbols.sync
                                    : CrabforgeSymbols.addCircle,
                              ),
                            ),
                          if (row['can_export_snapshot'] == true)
                            IconButton(
                              onPressed: _busy ? null : () => _archive(name),
                              tooltip: strings.exportPluginArchive,
                              icon: const Icon(CrabforgeSymbols.iosShare),
                            ),
                          if (assigned)
                            IconButton(
                              onPressed: _busy
                                  ? null
                                  : () => _set(row, assign: false),
                              tooltip: strings.unassignMcpServer,
                              icon: const Icon(CrabforgeSymbols.removeCircle),
                            ),
                        ],
                      );
                    },
                  ),
                ],
              ),
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
    );
  }
}
