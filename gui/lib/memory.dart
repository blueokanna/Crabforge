import 'dart:convert';

import 'package:file_selector/file_selector.dart';
import 'package:flutter/material.dart';

import 'material_symbols.dart';
import 'collection_view.dart';
import 'language.dart';
import 'memory_sync.dart';
import 'src/rust/api.dart' as api;

const _kinds = [
  'profile',
  'preference',
  'rule',
  'reference',
  'fact',
  'procedure',
  'episode',
];
const _states = ['active', 'candidate', 'archived', 'superseded'];

class MemoryLibrary extends StatefulWidget {
  const MemoryLibrary({super.key, this.initialStatus = ''});

  final String initialStatus;

  @override
  State<MemoryLibrary> createState() => _MemoryLibraryState();
}

class _MemoryLibraryState extends State<MemoryLibrary> {
  static const pageSize = 50;
  final search = TextEditingController();
  late String status = widget.initialStatus;
  int offset = 0;
  late Future<Map<String, dynamic>> query;

  @override
  void initState() {
    super.initState();
    query = fetch();
  }

  @override
  void dispose() {
    search.dispose();
    super.dispose();
  }

  Future<Map<String, dynamic>> fetch() async => jsonDecode(
    await api.searchMemories(
      query: jsonEncode({
        'text': search.text,
        if (status.isNotEmpty) 'status': status,
        'offset': offset,
        'limit': pageSize,
      }),
    ),
  ) as Map<String, dynamic>;

  void refresh([int start = 0]) => setState(() {
    offset = start;
    query = fetch();
  });

  Future<void> edit([Map<String, dynamic>? entry]) async {
    final saved = await showDialog<bool>(
      context: context,
      barrierDismissible: false,
      builder: (_) => _MemoryEditor(entry: entry),
    );
    if (saved == true && mounted) refresh();
  }

  @override
  Widget build(BuildContext context) {
    final strings = context.strings;
    final colors = Theme.of(context).colorScheme;
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
                child: EditorHeading(strings.memoryLibrary),
              ),
              const SizedBox(height: 12),
              TextField(
                controller: search,
                maxLines: 1,
                onSubmitted: (_) => refresh(),
                decoration: InputDecoration(
                  labelText: strings.searchMemories,
                  suffixIcon: IconButton(
                    onPressed: refresh,
                    tooltip: strings.searchMemories,
                    icon: const Icon(CrabforgeSymbols.search),
                  ),
                ),
              ),
              const SizedBox(height: 12),
              Align(
                alignment: AlignmentDirectional.centerStart,
                child: DropdownButton<String>(
                  icon: const Icon(CrabforgeSymbols.arrowDropDown),
                  value: status,
                  items: [
                    for (final value in ['', ..._states])
                      DropdownMenuItem(
                        value: value,
                        child: Text(strings.memoryState(value)),
                      ),
                  ],
                  onChanged: (value) {
                    if (value == null) return;
                    status = value;
                    refresh();
                  },
                ),
              ),
              const SizedBox(height: 8),
              Expanded(
                child: FutureBuilder<Map<String, dynamic>>(
                  future: query,
                  builder: (context, snapshot) {
                    if (snapshot.connectionState != ConnectionState.done) {
                      return const Center(child: CircularProgressIndicator());
                    }
                    if (snapshot.hasError) {
                      return SingleChildScrollView(
                        child: SelectableText('${snapshot.error}'),
                      );
                    }
                    final result = snapshot.requireData;
                    final entries = (result['entries'] as List)
                        .cast<Map<String, dynamic>>();
                    final unreadable = (result['unreadable'] as List)
                        .cast<String>();
                    final total = result['total'] as int;
                    return Column(
                      crossAxisAlignment: CrossAxisAlignment.stretch,
                      children: [
                        if (unreadable.isNotEmpty)
                          Text(
                            '${strings.unreadableMemories} ${unreadable.join(', ')}',
                            maxLines: 2,
                            overflow: TextOverflow.ellipsis,
                            style: TextStyle(color: colors.error),
                          ),
                        Expanded(
                          child: entries.isEmpty
                              ? Center(child: Text(strings.noMemoriesFound))
                              : EditorCollection(
                                  count: entries.length,
                                  itemBuilder: (context, index) {
                                    final item = entries[index];
                                    return Card.outlined(
                                      clipBehavior: Clip.hardEdge,
                                      child: ListTile(
                                        leading: item['pinned'] == true
                                            ? const Icon(
                                                CrabforgeSymbols.pushPin,
                                              )
                                            : null,
                                        title: Text(
                                          item['text'] as String,
                                          maxLines: 2,
                                          overflow: TextOverflow.ellipsis,
                                        ),
                                        subtitle: Text(
                                          '${strings.memoryKind(item['kind'] as String)} · ${strings.memoryState(item['status'] as String)}',
                                          maxLines: 2,
                                          overflow: TextOverflow.ellipsis,
                                        ),
                                        onTap: () => edit(item),
                                      ),
                                    );
                                  },
                                ),
                        ),
                        if (total > pageSize || offset > 0)
                          Row(
                            mainAxisAlignment: MainAxisAlignment.end,
                            children: [
                              IconButton(
                                tooltip: MaterialLocalizations.of(context)
                                    .previousPageTooltip,
                                onPressed: offset == 0
                                    ? null
                                    : () => refresh(offset - pageSize),
                                icon: const Icon(CrabforgeSymbols.chevronLeft),
                              ),
                              Text(
                                '${total == 0 ? 0 : offset + 1}–${offset + entries.length} / $total',
                              ),
                              IconButton(
                                tooltip: MaterialLocalizations.of(context)
                                    .nextPageTooltip,
                                onPressed: offset + pageSize >= total
                                    ? null
                                    : () => refresh(offset + pageSize),
                                icon: const Icon(CrabforgeSymbols.chevronRight),
                              ),
                            ],
                          ),
                      ],
                    );
                  },
                ),
              ),
              const SizedBox(height: 8),
              Wrap(
                alignment: WrapAlignment.end,
                spacing: 8,
                children: [
                  TextButton(
                    onPressed: () => Navigator.pop(context),
                    child: Text(
                      MaterialLocalizations.of(context).closeButtonLabel,
                    ),
                  ),
                  TextButton(
                    onPressed: () async {
                      final changed = await showDialog<bool>(
                        context: context,
                        barrierDismissible: false,
                        builder: (_) => const MemorySync(),
                      );
                      if (mounted && changed == true) refresh();
                    },
                    child: Text(strings.memorySync),
                  ),
                  FilledButton.icon(
                    onPressed: edit,
                    icon: const Icon(CrabforgeSymbols.add),
                    label: Text(strings.addMemory),
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

class _MemoryEditor extends StatefulWidget {
  const _MemoryEditor({this.entry});
  final Map<String, dynamic>? entry;

  @override
  State<_MemoryEditor> createState() => _MemoryEditorState();
}

class _MemoryEditorState extends State<_MemoryEditor> {
  late final text = TextEditingController(
    text: widget.entry?['text'] as String?,
  );
  late final scope = TextEditingController(
    text: widget.entry?['scope'] as String? ?? 'global',
  );
  late final tags = TextEditingController(
    text: (widget.entry?['tags'] as List?)?.join('\n'),
  );
  late final why = TextEditingController(text: widget.entry?['why'] as String?);
  late String kind = widget.entry?['kind'] as String? ?? 'fact';
  late String status = widget.entry?['status'] as String? ?? 'active';
  late bool pinned = widget.entry?['pinned'] == true;
  bool saving = false;
  String? error;

  @override
  void dispose() {
    for (final field in [text, scope, tags, why]) {
      field.dispose();
    }
    super.dispose();
  }

  Future<void> save() async {
    if (text.text.trim().isEmpty) {
      setState(() => error = context.strings.requiredField);
      return;
    }
    setState(() {
      saving = true;
      error = null;
    });
    try {
      await api.saveMemory(
        id: widget.entry?['id'] as String?,
        revision: widget.entry?['revision'] as String?,
        changes: jsonEncode({
          'text': text.text,
          'scope': scope.text,
          'tags': tags.text.split('\n'),
          'why': why.text,
          'kind': kind,
          'status': status,
          'pinned': pinned,
        }),
      );
      if (mounted) Navigator.pop(context, true);
    } catch (failure) {
      if (mounted) setState(() => error = '$failure');
    } finally {
      if (mounted) setState(() => saving = false);
    }
  }

  Future<void> chooseProject() async {
    try {
      final folder = await getDirectoryPath();
      if (folder == null || !mounted) return;
      final key = await api.memoryProjectKey(project: folder);
      if (mounted && !saving) {
        setState(() {
          scope.text = 'project:$key';
          error = null;
        });
      }
    } catch (failure) {
      if (mounted) setState(() => error = '$failure');
    }
  }

  @override
  Widget build(BuildContext context) {
    final strings = context.strings;
    return AlertDialog(
      title: Text(
        widget.entry == null ? strings.addMemory : strings.editMemory,
      ),
      scrollable: true,
      content: SizedBox(
        width: 540,
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            TextField(
              controller: text,
              enabled: !saving,
              minLines: 3,
              maxLines: 6,
              maxLength: 2000,
              decoration: const InputDecoration(labelText: 'Markdown'),
            ),
            TextField(
              controller: scope,
              enabled: !saving,
              decoration: InputDecoration(
                labelText: strings.memoryScope,
                helperText: 'global · project:<key> · agent:<id>',
                suffixIcon: IconButton(
                  tooltip: strings.projectFolder,
                  onPressed: saving ? null : chooseProject,
                  icon: const Icon(CrabforgeSymbols.folderOpen),
                ),
              ),
            ),
            const SizedBox(height: 10),
            TextField(
              controller: tags,
              enabled: !saving,
              minLines: 1,
              maxLines: 3,
              decoration: InputDecoration(labelText: strings.memoryTags),
            ),
            const SizedBox(height: 10),
            TextField(
              controller: why,
              enabled: !saving,
              minLines: 1,
              maxLines: 3,
              decoration: InputDecoration(labelText: strings.memoryWhy),
            ),
            if (widget.entry?['source_quote'] case final String quote) ...[
              const SizedBox(height: 10),
              Text(strings.memorySourceQuote),
              SelectableText(quote),
            ],
            const SizedBox(height: 10),
            DropdownButtonFormField<String>(
              icon: const Icon(CrabforgeSymbols.arrowDropDown),
              initialValue: kind,
              decoration: InputDecoration(labelText: strings.memoryKindLabel),
              items: [
                for (final value in _kinds)
                  DropdownMenuItem(
                    value: value,
                    child: Text(strings.memoryKind(value)),
                  ),
              ],
              onChanged: saving ? null : (value) => kind = value!,
            ),
            const SizedBox(height: 10),
            DropdownButtonFormField<String>(
              icon: const Icon(CrabforgeSymbols.arrowDropDown),
              initialValue: status,
              decoration: InputDecoration(labelText: strings.memoryStateLabel),
              items: [
                for (final value in _states)
                  DropdownMenuItem(
                    value: value,
                    child: Text(strings.memoryState(value)),
                  ),
              ],
              onChanged: saving ? null : (value) => status = value!,
            ),
            SwitchListTile(
              contentPadding: EdgeInsets.zero,
              value: pinned,
              onChanged: saving
                  ? null
                  : (value) => setState(() => pinned = value),
              title: Text(strings.pinMemory),
            ),
            if (saving) const LinearProgressIndicator(),
            if (error case final message?)
              SelectableText(
                message,
                style: TextStyle(color: Theme.of(context).colorScheme.error),
              ),
          ],
        ),
      ),
      actions: [
        TextButton(
          onPressed: saving ? null : () => Navigator.pop(context),
          child: Text(strings.cancel),
        ),
        FilledButton(
          onPressed: saving ? null : save,
          child: Text(strings.save),
        ),
      ],
    );
  }
}
