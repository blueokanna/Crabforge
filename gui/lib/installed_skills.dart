part of 'agent_skills.dart';

class _InstalledSkills extends StatefulWidget {
  const _InstalledSkills();

  @override
  State<_InstalledSkills> createState() => _InstalledSkillsState();
}

class _InstalledSkillsState extends State<_InstalledSkills> {
  List<_Json> _skills = [];
  Object? _error;
  bool _busy = true;

  @override
  void initState() {
    super.initState();
    _scan();
  }

  Future<void> _scan() async {
    setState(() {
      _busy = true;
      _error = null;
    });
    try {
      final skills = (jsonDecode(await api.discoverAgentSkills()) as List)
          .cast<_Json>();
      if (mounted) setState(() => _skills = skills);
    } catch (error) {
      if (mounted) setState(() => _error = error);
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  Future<void> _adopt(_Json skill) async {
    final strings = context.strings;
    final path = skill['path'] as String;
    final approved = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(skill['name'] as String),
        scrollable: true,
        content: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [Text(strings.adoptSkillHint), SelectableText(path)],
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: Text(strings.cancel),
          ),
          FilledButton(
            onPressed: () => Navigator.pop(context, true),
            child: Text(strings.adoptSkill),
          ),
        ],
      ),
    );
    if (!mounted || approved != true) return;
    setState(() {
      _busy = true;
      _error = null;
    });
    try {
      final library = await api.adoptAgentSkill(
        agent: skill['agent'] as String,
        name: skill['name'] as String,
        path: path,
        content: skill['content'] as String,
      );
      if (mounted) Navigator.pop(context, library);
    } catch (error) {
      if (mounted) {
        setState(() {
          _busy = false;
          _error = error;
        });
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    final strings = context.strings;
    final grid = CollectionLayout.of(context).grid;
    return Dialog(
      key: const ValueKey('installed-skills'),
      child: SizedBox(
        width: 780,
        height: 560,
        child: Padding(
          padding: const EdgeInsets.all(20),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              DefaultTextStyle(
                style: Theme.of(context).textTheme.titleLarge!,
                child: EditorHeading(strings.installedSkills),
              ),
              if (_busy) const LinearProgressIndicator(),
              Expanded(
                child: CustomScrollView(
                  slivers: [
                    if (_error != null)
                      SliverToBoxAdapter(child: SelectableText('$_error')),
                    if (!_busy && _error == null && _skills.isEmpty)
                      SliverToBoxAdapter(child: Text(strings.noSkills)),
                    CollectionSliver.builder(
                      grid: grid,
                      height: 150,
                      itemCount: _skills.length,
                      itemBuilder: (context, index) {
                        final skill = _skills[index];
                        return CollectionTile(
                          key: ValueKey(skill['path'] as String),
                          grid: grid,
                          title: Text(skill['name'] as String),
                          leading: const Icon(Icons.extension_outlined),
                          detail: Column(
                            crossAxisAlignment: CrossAxisAlignment.start,
                            children: [
                              Text(_agents[skill['agent']]!),
                              Text(
                                skill['description'] as String,
                                maxLines: 2,
                                overflow: TextOverflow.ellipsis,
                              ),
                              Text(
                                skill['path'] as String,
                                maxLines: 2,
                                overflow: TextOverflow.ellipsis,
                              ),
                            ],
                          ),
                          actions: [
                            IconButton(
                              onPressed: _busy ? null : () => _adopt(skill),
                              tooltip: strings.adoptSkill,
                              icon: const Icon(Icons.playlist_add_outlined),
                            ),
                          ],
                        );
                      },
                    ),
                  ],
                ),
              ),
              Row(
                mainAxisAlignment: MainAxisAlignment.end,
                children: [
                  IconButton(
                    onPressed: _busy ? null : _scan,
                    tooltip: strings.reload,
                    icon: const Icon(Icons.refresh),
                  ),
                  TextButton(
                    onPressed: _busy ? null : () => Navigator.pop(context),
                    child: Text(
                      MaterialLocalizations.of(context).closeButtonLabel,
                    ),
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
