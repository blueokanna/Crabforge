import 'dart:convert';

import 'package:flutter/material.dart';

import 'collection_view.dart';
import 'language.dart';
import 'src/rust/api.dart' as api;

enum OmpEditor { advisor, thinking }

class _ChoiceDialog extends StatelessWidget {
  const _ChoiceDialog({required this.title, required this.children});
  final String title;
  final List<Widget> children;

  @override
  Widget build(BuildContext context) => AlertDialog(
    title: EditorHeading(title),
    content: SizedBox(
      width: 520,
      child: EditorCollection(
        count: children.length,
        itemBuilder: (_, i) => children[i],
      ),
    ),
  );
}

Future<void> editOmpRole(BuildContext context) async {
  final profile = await chooseOmpProfile(context);
  if (profile == null) return;
  final previous = await api.ompRoles(profile: profile);
  final data = jsonDecode(previous) as Map<String, dynamic>;
  final assignments = data['assignments'] as Map<String, dynamic>;
  if (!context.mounted) return;
  final role = await showDialog<String>(
    context: context,
    builder: (context) => _ChoiceDialog(
      title: '${context.strings.roleModels} · $profile',
      children: [
        for (final entry in assignments.entries)
          ListTile(
            title: Text(entry.key),
            subtitle: Text(
              entry.value?.toString() ?? context.strings.inheritSetting,
              maxLines: 2,
              overflow: TextOverflow.ellipsis,
            ),
            onTap: () => Navigator.pop(context, entry.key),
          ),
      ],
    ),
  );
  if (!context.mounted || role == null) return;
  await showDialog<void>(
    context: context,
    builder: (context) => _RoleEditor(
      profile: profile,
      role: role,
      previous: previous,
      current: assignments[role],
      models: (data['models'] as List).cast<String>(),
    ),
  );
}

class _RoleEditor extends StatefulWidget {
  const _RoleEditor({
    required this.profile,
    required this.role,
    required this.previous,
    required this.current,
    required this.models,
  });
  final String profile, role, previous;
  final Object? current;
  final List<String> models;

  @override
  State<_RoleEditor> createState() => _RoleEditorState();
}

class _RoleEditorState extends State<_RoleEditor> {
  final model = TextEditingController();
  bool saving = false;
  String? error;

  @override
  void dispose() {
    model.dispose();
    super.dispose();
  }

  Future<void> save(String? pick) async {
    setState(() {
      saving = true;
      error = null;
    });
    try {
      await api.saveOmpRole(
        profile: widget.profile,
        previous: widget.previous,
        role: widget.role,
        model: pick,
      );
      if (mounted) Navigator.pop(context);
    } catch (e) {
      if (mounted) {
        setState(() {
          saving = false;
          error = '$e';
        });
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    final s = context.strings;
    final query = model.text.trim();
    final current = widget.current;
    final filter = query.toLowerCase();
    final options = {
      ...widget.models,
      if (current is String && !current.contains(',') && current.isNotEmpty)
        current,
    };
    final candidates = options
        .where((v) => v.toLowerCase().contains(filter))
        .toList();
    return AlertDialog(
      title: EditorHeading('${widget.role} · ${widget.profile}'),
      content: SizedBox(
        width: 440,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(widget.current?.toString() ?? s.inheritSetting),
            TextField(
              key: const ValueKey('omp-role-model'),
              controller: model,
              enabled: !saving,
              decoration: InputDecoration(labelText: s.roleModelId),
              onChanged: (_) => setState(() {}),
            ),
            Flexible(
              child: ConstrainedBox(
                constraints: const BoxConstraints(maxHeight: 240),
                child: EditorCollection(
                  count: candidates.length,
                  itemBuilder: (context, i) => ListTile(
                    title: Text(
                      candidates[i],
                      maxLines: 3,
                      overflow: TextOverflow.ellipsis,
                    ),
                    onTap: saving
                        ? null
                        : () => setState(() => model.text = candidates[i]),
                  ),
                ),
              ),
            ),
            if (error != null) SelectableText(error!),
          ],
        ),
      ),
      actions: [
        TextButton(
          onPressed: saving || widget.current == null ? null : () => save(null),
          child: Text(s.resetRole),
        ),
        TextButton(
          onPressed: saving ? null : () => Navigator.pop(context),
          child: Text(s.cancel),
        ),
        FilledButton(
          onPressed: saving || query.isEmpty ? null : () => save(query),
          child: Text(s.save),
        ),
      ],
    );
  }
}

Future<String?> chooseOmpProfile(BuildContext context) async {
  final profiles = (jsonDecode(await api.ompProfiles()) as List)
      .cast<Map<String, dynamic>>();
  if (!context.mounted) return null;
  return profiles.length == 1
      ? profiles.single['name'] as String
      : await showDialog<String>(
          context: context,
          builder: (context) => _ChoiceDialog(
            title: context.strings.ompProfile,
            children: [
              for (final row in profiles)
                ListTile(
                  title: Text(row['name'] as String),
                  subtitle: Tooltip(
                    message: row['directory'] as String,
                    child: Text(
                      row['directory'] as String,
                      maxLines: 2,
                      overflow: TextOverflow.ellipsis,
                    ),
                  ),
                  trailing: row['active'] == true
                      ? const Icon(Icons.check)
                      : null,
                  onTap: () => Navigator.pop(context, row['name'] as String),
                ),
            ],
          ),
        );
}

Future<void> chooseOmpProtocol(BuildContext context) async {
  final profile = await chooseOmpProfile(context);
  if (profile == null) return;
  final previous = await api.ompProtocol(profile: profile);
  if (!context.mounted) return;
  final options = {
    'auto': context.strings.nativeModelProtocol,
    'openai-completions': 'Chat Completions',
    'openai-responses': 'OpenAI Responses',
    'anthropic-messages': 'Anthropic Messages',
    'google-generative-ai': 'Gemini',
  };
  final selected = await showDialog<String>(
    context: context,
    builder: (context) => _ChoiceDialog(
      title: '${context.strings.gatewayProtocol} · $profile',
      children: [
        for (final option in options.entries)
          ListTile(
            title: Text(option.value),
            trailing: option.key == previous ? const Icon(Icons.check) : null,
            onTap: () => Navigator.pop(context, option.key),
          ),
      ],
    ),
  );
  if (selected != null) {
    await api.saveOmpProtocol(
      profile: profile,
      previous: previous,
      protocol: selected,
    );
  }
}

Future<void> editOmpSettings(BuildContext context, OmpEditor editor) async {
  final profile = await chooseOmpProfile(context);
  if (profile == null) return;
  final previous = switch (editor) {
    OmpEditor.advisor => await api.ompAdvisor(profile: profile),
    OmpEditor.thinking => await api.ompThinking(profile: profile),
  };
  if (!context.mounted) return;
  await showDialog<void>(
    context: context,
    builder: (context) =>
        _Editor(profile: profile, previous: previous, editor: editor),
  );
}

class _Editor extends StatefulWidget {
  const _Editor({
    required this.profile,
    required this.previous,
    required this.editor,
  });
  final OmpEditor editor;
  final String profile;
  final String previous;

  @override
  State<_Editor> createState() => _EditorState();
}

class _EditorState extends State<_Editor> {
  late final source = jsonDecode(widget.previous) as Map<String, dynamic>;
  late final roles = Map<String, dynamic>.from(source['roles'] ?? {});
  late final initial = widget.editor == OmpEditor.advisor
      ? Map<String, String>.from(source)
      : <String, String>{
          'session': source['session'] ?? 'inherit',
          for (final r in roles.entries) r.key: r.value['level'] ?? 'inherit',
        };
  final changes = <String, String>{};
  bool saving = false;
  String? error;

  Future<void> save() async {
    setState(() {
      saving = true;
      error = null;
    });
    try {
      switch (widget.editor) {
        case OmpEditor.advisor:
          await api.saveOmpAdvisor(
            profile: widget.profile,
            previous: widget.previous,
            changes: jsonEncode(changes),
          );
        case OmpEditor.thinking:
          final update = {
            if (changes.containsKey('session')) 'session': changes['session'],
            'roles': Map.of(changes)..remove('session'),
          };
          await api.saveOmpThinking(
            profile: widget.profile,
            previous: widget.previous,
            changes: jsonEncode(update),
          );
      }
      if (mounted) Navigator.pop(context);
    } catch (e) {
      if (mounted) {
        setState(() {
          saving = false;
          error = '$e';
        });
      }
    }
  }

  Widget field(String key, String title, Map<String, String> options) {
    final selected = changes[key] ?? initial[key]!;
    final labels = {'inherit': context.strings.inheritSetting, ...options};
    labels.putIfAbsent(selected, () => selected);
    return Padding(
      padding: const EdgeInsets.only(top: 12),
      child: DropdownButtonFormField<String>(
        key: ValueKey('omp-${widget.editor.name}-$key'),
        initialValue: selected,
        decoration: InputDecoration(labelText: title),
        isExpanded: true,
        items: [
          for (final entry in labels.entries)
            DropdownMenuItem(value: entry.key, child: Text(entry.value)),
        ],
        onChanged: saving
            ? null
            : (value) {
                if (value == null) return;
                setState(() {
                  if (value == initial[key]) {
                    changes.remove(key);
                  } else {
                    changes[key] = value;
                  }
                });
              },
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final s = context.strings;
    return AlertDialog(
      scrollable: true,
      title: Text(
        '${widget.editor == OmpEditor.advisor ? s.advisorSettings : s.thinkingLevels} · ${widget.profile}',
      ),
      content: SizedBox(
        width: 420,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            if (widget.editor == OmpEditor.thinking) ...[
              field('session', s.sessionThinking, thinkingLevels()),
              for (final role in roles.entries)
                field(
                  role.key,
                  '${role.key} · ${role.value['model']}',
                  thinkingLevels(allowOff: true),
                ),
            ] else ...[
              field('review', s.advisorReview, {
                'off': s.disabled,
                'turn': s.everyTurn,
                'agent-end': s.runEnd,
              }),
              field('wait', s.advisorWait, {
                'off': s.disabled,
                '1': '1',
                '3': '3',
                '5': '5',
                'strict': s.allReviews,
              }),
              field('wake', s.advisorWake, {
                'nit': s.minorNote,
                'concern': s.concernNote,
                'blocker': s.blockerNote,
              }),
            ],
            if (error != null) SelectableText(error!),
          ],
        ),
      ),
      actions: [
        TextButton(
          onPressed: saving ? null : () => Navigator.pop(context),
          child: Text(s.cancel),
        ),
        FilledButton(
          onPressed: saving || changes.isEmpty ? null : save,
          child: Text(s.save),
        ),
      ],
    );
  }

  Map<String, String> thinkingLevels({bool allowOff = false}) {
    final s = context.strings;
    return {
      'auto': s.thinkingAuto,
      if (allowOff) 'off': s.disabled,
      'minimal': s.thinkingMinimal,
      'low': s.thinkingLow,
      'medium': s.thinkingMedium,
      'high': s.thinkingHigh,
      'xhigh': s.thinkingExtraHigh,
      'max': s.thinkingMax,
    };
  }
}
