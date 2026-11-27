import 'dart:convert';

import 'package:flutter/material.dart';

import 'material_symbols.dart';
import 'collection_view.dart';
import 'language.dart';
import 'src/rust/api.dart' as api;

const _agents = {
  'claude': 'Claude Code',
  'cline': 'Cline',
  'kiro-cli': 'Kiro CLI',
};

typedef _Json = Map<String, dynamic>;

class AgentRules extends StatefulWidget {
  const AgentRules({super.key});

  @override
  State<AgentRules> createState() => _AgentRulesState();
}

class _AgentRulesState extends State<AgentRules> {
  _Json? library;
  Object? error;
  bool busy = false;

  @override
  void initState() {
    super.initState();
    load(api.agentRules);
  }

  Future<void> load(Future<String> Function() work) async {
    setState(() {
      busy = true;
      error = null;
    });
    try {
      final result = jsonDecode(await work()) as _Json;
      if (mounted) setState(() => library = result);
    } catch (e) {
      if (mounted) setState(() => error = e);
    } finally {
      if (mounted) setState(() => busy = false);
    }
  }

  Future<void> edit(_Json? entry) async {
    final changed = await showDialog<_Json>(
      context: context,
      barrierDismissible: false,
      builder: (_) => _RuleEditor(entry: entry),
    );
    if (changed == null || !mounted) return;
    setState(() {
      library = changed;
      error = null;
    });
  }

  Future<void> remove(_Json entry) async {
    final name = (entry['rule'] as _Json)['name'] as String;
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
    if (confirmed != true || !mounted) return;
    await load(
      () => api.removeAgentRule(
        name: name,
        revision: entry['revision'] as String,
      ),
    );
  }

  Widget placements(BuildContext context, List<_Json> files) => Column(
    crossAxisAlignment: CrossAxisAlignment.start,
    children: [
      for (final file in files) ...[
        Text(
          '${_agents[file['agent']]} · ${context.strings.ruleStatus(file['status'] as String)}',
        ),
        if (file['path'] case final String path)
          SelectableText(path, style: Theme.of(context).textTheme.bodySmall),
        if (file['error'] case final String detail)
          SelectableText(
            detail,
            style: TextStyle(color: Theme.of(context).colorScheme.error),
          ),
      ],
    ],
  );

  @override
  Widget build(BuildContext context) {
    final strings = context.strings;
    final grid = CollectionLayout.of(context).grid;
    final rules =
        (library?['rules'] as List?)?.cast<_Json>() ?? const <_Json>[];
    final byRule = <String, List<_Json>>{};
    for (final file
        in (library?['placements'] as List?)?.cast<_Json>() ??
            const <_Json>[]) {
      (byRule[file['name'] as String] ??= []).add(file);
    }
    final names = rules
        .map((entry) => (entry['rule'] as _Json)['name'])
        .toSet();
    final detached = byRule.entries.where(
      (entry) => !names.contains(entry.key),
    );
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
                child: EditorHeading(strings.agentRules),
              ),
              Wrap(
                runSpacing: 4,
                children: [
                  TextButton.icon(
                    onPressed: busy ? null : () => edit(null),
                    icon: const Icon(CrabforgeSymbols.add),
                    label: Text(strings.addRule),
                  ),
                  TextButton.icon(
                    onPressed: busy ? null : () => load(api.syncAgentRules),
                    icon: const Icon(CrabforgeSymbols.sync),
                    label: Text(strings.syncRules),
                  ),
                  IconButton(
                    onPressed: busy ? null : () => load(api.agentRules),
                    tooltip: strings.reload,
                    icon: const Icon(CrabforgeSymbols.refresh),
                  ),
                ],
              ),
              if (busy) const LinearProgressIndicator(),
              Expanded(
                child: CustomScrollView(
                  slivers: [
                    if (error != null)
                      SliverToBoxAdapter(child: SelectableText('$error')),
                    if (rules.isEmpty && !busy && error == null)
                      SliverToBoxAdapter(child: Text(strings.noRules)),
                    CollectionSliver.builder(
                      grid: grid,
                      height: 170,
                      itemCount: rules.length,
                      itemBuilder: (context, index) {
                        final entry = rules[index];
                        final rule = entry['rule'] as _Json;
                        final name = rule['name'] as String;
                        final paths = (rule['paths'] as List).cast<String>();
                        return CollectionTile(
                          key: ValueKey(name),
                          grid: grid,
                          title: Text(name),
                          leading: const Icon(CrabforgeSymbols.rule),
                          onTap: busy ? null : () => edit(entry),
                          detail: Column(
                            crossAxisAlignment: CrossAxisAlignment.start,
                            children: [
                              Text(
                                rule['body'] as String,
                                maxLines: 2,
                                overflow: TextOverflow.ellipsis,
                              ),
                              if (paths.isNotEmpty)
                                Text(
                                  paths.join(' · '),
                                  maxLines: 1,
                                  overflow: TextOverflow.ellipsis,
                                ),
                              placements(context, byRule[name] ?? const []),
                            ],
                          ),
                          actions: [
                            IconButton(
                              onPressed: busy ? null : () => edit(entry),
                              tooltip: strings.editRule,
                              icon: const Icon(CrabforgeSymbols.edit),
                            ),
                            IconButton(
                              onPressed: busy ? null : () => remove(entry),
                              tooltip: strings.remove,
                              icon: const Icon(CrabforgeSymbols.delete),
                            ),
                          ],
                        );
                      },
                    ),
                    for (final entry in detached)
                      SliverToBoxAdapter(
                        child: ListTile(
                          title: Text(entry.key),
                          subtitle: placements(context, entry.value),
                        ),
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

class _RuleEditor extends StatefulWidget {
  const _RuleEditor({required this.entry});

  final _Json? entry;

  @override
  State<_RuleEditor> createState() => _RuleEditorState();
}

class _RuleEditorState extends State<_RuleEditor> {
  final form = GlobalKey<FormState>();
  late final rule = widget.entry?['rule'] as _Json?;
  late final name = TextEditingController(text: rule?['name'] as String?);
  late final body = TextEditingController(text: rule?['body'] as String?);
  late final paths = TextEditingController(
    text: (rule?['paths'] as List?)?.join('\n'),
  );
  late final agents = {...?(rule?['agents'] as List?)?.cast<String>()};
  bool saving = false;
  Object? error;

  Future<void> save() async {
    if (!form.currentState!.validate()) return;
    setState(() {
      saving = true;
      error = null;
    });
    try {
      final result = await api.saveAgentRule(
        old: rule?['name'] as String?,
        revision: widget.entry?['revision'] as String?,
        rule: jsonEncode({
          'name': name.text,
          'body': body.text,
          'paths': const LineSplitter().convert(paths.text),
          'agents': agents.toList(),
        }),
      );
      if (mounted) Navigator.pop(context, jsonDecode(result) as _Json);
    } catch (e) {
      if (mounted) {
        setState(() {
          saving = false;
          error = e;
        });
      }
    }
  }

  @override
  void dispose() {
    name.dispose();
    body.dispose();
    paths.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final strings = context.strings;
    String? required(String? value) =>
        (value ?? '').trim().isEmpty ? strings.ruleRequired : null;
    return AlertDialog(
      scrollable: true,
      title: Text(rule == null ? strings.addRule : strings.editRule),
      content: SizedBox(
        width: 540,
        child: Form(
          key: form,
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              TextFormField(
                controller: name,
                readOnly: saving,
                validator: required,
                decoration: InputDecoration(labelText: strings.ruleName),
              ),
              TextFormField(
                controller: body,
                readOnly: saving,
                validator: required,
                minLines: 3,
                maxLines: 10,
                decoration: InputDecoration(labelText: strings.ruleBody),
              ),
              TextFormField(
                controller: paths,
                readOnly: saving,
                minLines: 1,
                maxLines: 4,
                decoration: InputDecoration(labelText: strings.rulePaths),
              ),
              const SizedBox(height: 12),
              Wrap(
                spacing: 6,
                runSpacing: 6,
                children: [
                  for (final entry in _agents.entries)
                    FilterChip(
                      selected: agents.contains(entry.key),
                      label: Text(entry.value),
                      onSelected: saving
                          ? null
                          : (selected) => setState(() {
                              if (selected) {
                                agents.add(entry.key);
                              } else {
                                agents.remove(entry.key);
                              }
                            }),
                    ),
                ],
              ),
              if (error != null)
                SelectableText(
                  '$error',
                  style: TextStyle(color: Theme.of(context).colorScheme.error),
                ),
              if (saving) const LinearProgressIndicator(),
            ],
          ),
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
