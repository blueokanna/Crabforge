part of 'plugin_bundles.dart';

const _piAssignmentsType = XTypeGroup(label: 'JSON', extensions: ['json']);

class _PiProjectPackages extends StatefulWidget {
  const _PiProjectPackages();

  @override
  State<_PiProjectPackages> createState() => _PiProjectPackagesState();
}

class _PiProjectPackagesState extends State<_PiProjectPackages> {
  final _folder = TextEditingController();
  String? _project;
  _Json? _view;
  Object? _error;
  bool _busy = false;

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
    await _run(project, () => api.piProjectPluginBundles(project: project));
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
          content: Text(strings.piPackageConfirm),
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
      () => api.setPiProjectPluginBundle(
        project: project,
        name: name,
        projectRevision: view['revision'] as String,
        bundleRevision: assign ? row['bundle_revision'] as String : null,
      ),
    );
  }

  Future<void> _export() async {
    final output = await getSaveLocation(
      suggestedName: 'pi-project-packages.json',
      acceptedTypeGroups: const [_piAssignmentsType],
    );
    if (!mounted || output == null) return;
    final project = _project!;
    final revision = _view!['revision'] as String;
    setState(() {
      _busy = true;
      _error = null;
    });
    try {
      final text = await api.transferPiProjectPluginBundles(
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
    final file = await openFile(acceptedTypeGroups: const [_piAssignmentsType]);
    if (!mounted || file == null) return;
    final project = _project!;
    final revision = _view!['revision'] as String;
    final strings = context.strings;
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(file.name),
        content: Text(strings.piPackageConfirm),
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
      return api.transferPiProjectPluginBundles(
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
      await api.archivePiProjectPluginBundle(
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
      title: EditorHeading(strings.piProjectPackages),
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
                  icon: const Icon(Icons.folder_open_outlined),
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
                        leading: const Icon(Icons.inventory_2_outlined),
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
                                    ? Icons.sync
                                    : Icons.add_circle_outline,
                              ),
                            ),
                          if (row['can_export_snapshot'] == true)
                            IconButton(
                              onPressed: _busy ? null : () => _archive(name),
                              tooltip: strings.exportPluginArchive,
                              icon: const Icon(Icons.ios_share_outlined),
                            ),
                          if (assigned)
                            IconButton(
                              onPressed: _busy
                                  ? null
                                  : () => _set(row, assign: false),
                              tooltip: strings.unassignMcpServer,
                              icon: const Icon(Icons.remove_circle_outline),
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
