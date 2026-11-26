part of 'agent_skills.dart';

class _RepositorySkills extends StatefulWidget {
  const _RepositorySkills({required this.source});

  final String source;

  @override
  State<_RepositorySkills> createState() => _RepositorySkillsState();
}

class _RepositorySkillsState extends State<_RepositorySkills> {
  List<_Json> _skills = [];
  Object? _error;
  bool _loading = true;
  bool _dismissing = false;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _dismiss(String source) async {
    setState(() {
      _dismissing = true;
      _error = null;
    });
    try {
      await api.dismissSkillCandidate(source: source);
      if (mounted) {
        setState(
          () => _skills.removeWhere((skill) => skill['source'] == source),
        );
      }
    } catch (error) {
      if (mounted) setState(() => _error = error);
    } finally {
      if (mounted) setState(() => _dismissing = false);
    }
  }

  Future<void> _load() async {
    try {
      final result = await api.repositorySkillCatalog(source: widget.source);
      if (mounted) {
        setState(() => _skills = (jsonDecode(result) as List).cast<_Json>());
      }
    } catch (error) {
      if (mounted) setState(() => _error = error);
    } finally {
      if (mounted) setState(() => _loading = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    final strings = context.strings;
    final grid = CollectionLayout.of(context).grid;
    return Dialog(
      key: const ValueKey('repository-skills'),
      child: SizedBox(
        width: 740,
        height: 450,
        child: Padding(
          padding: const EdgeInsets.all(20),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              DefaultTextStyle(
                style: Theme.of(context).textTheme.titleLarge!,
                child: EditorHeading(strings.browseRepositorySkills),
              ),
              if (_loading) const LinearProgressIndicator(),
              Expanded(
                child: CustomScrollView(
                  slivers: [
                    if (_error != null)
                      SliverToBoxAdapter(child: SelectableText('$_error')),
                    if (!_loading && _error == null && _skills.isEmpty)
                      SliverToBoxAdapter(
                        child: Text(strings.noRepositorySkills),
                      ),
                    CollectionSliver.builder(
                      grid: grid,
                      height: 136,
                      itemCount: _skills.length,
                      itemBuilder: (context, index) {
                        final skill = _skills[index];
                        final source = skill['source'] as String;
                        final folder = skill['folder'] as String;
                        void select() => Navigator.pop(context, source);
                        return CollectionTile(
                          key: ValueKey(source),
                          grid: grid,
                          title: Text(skill['name'] as String),
                          leading: const Icon(Icons.extension_outlined),
                          onTap: _dismissing ? null : select,
                          detail: Column(
                            crossAxisAlignment: CrossAxisAlignment.start,
                            children: [
                              Text(
                                skill['description'] as String,
                                maxLines: 2,
                                overflow: TextOverflow.ellipsis,
                              ),
                              if (folder.isNotEmpty)
                                Text(
                                  folder,
                                  maxLines: 2,
                                  overflow: TextOverflow.ellipsis,
                                ),
                            ],
                          ),
                          actions: [
                            IconButton(
                              icon: const Icon(Icons.playlist_add_outlined),
                              onPressed: _dismissing ? null : select,
                              tooltip: strings.importSkill,
                            ),
                            IconButton(
                              icon: const Icon(Icons.visibility_off_outlined),
                              onPressed: _dismissing
                                  ? null
                                  : () => _dismiss(source),
                              tooltip: strings.dismissSkillCandidate,
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
