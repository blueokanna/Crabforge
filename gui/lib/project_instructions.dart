import 'dart:convert';

import 'package:file_selector/file_selector.dart';
import 'package:flutter/material.dart';

import 'language.dart';
import 'src/rust/api.dart' as api;

class ProjectInstructionsEditor extends StatefulWidget {
  const ProjectInstructionsEditor({super.key, required this.agent});

  final String agent;

  @override
  State<ProjectInstructionsEditor> createState() =>
      _ProjectInstructionsEditorState();
}

class _ProjectInstructionsEditorState extends State<ProjectInstructionsEditor> {
  final folder = TextEditingController();
  final markdown = TextEditingController();
  ({String folder, String path, String revision, String text, bool canRestore})?
  document;
  bool busy = false;
  String? error;

  @override
  void dispose() {
    folder.dispose();
    markdown.dispose();
    super.dispose();
  }

  Future<void> perform(Future<void> Function() action) async {
    if (busy) return;
    setState(() {
      busy = true;
      error = null;
    });
    try {
      await action();
    } catch (failure) {
      if (mounted) setState(() => error = '$failure');
    } finally {
      if (mounted) setState(() => busy = false);
    }
  }

  Future<void> load(String path) async {
    final result = jsonDecode(
      await api.projectInstructions(project: path, agent: widget.agent),
    );
    if (!mounted) return;
    final text = result['text'] as String;
    setState(() {
      folder.text = path;
      markdown.text = text;
      document = (
        folder: path,
        path: result['path'] as String,
        revision: result['revision'] as String,
        text: text,
        canRestore: result['can_restore'] as bool,
      );
    });
  }

  Future<void> confirmRestore() async {
    final strings = context.strings;
    final allowed = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(strings.restoreInstructions),
        content: SelectableText(document!.path),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context, false),
            child: Text(strings.cancel),
          ),
          FilledButton(
            onPressed: () => Navigator.pop(context, true),
            child: Text(strings.restoreInstructions),
          ),
        ],
      ),
    );
    if (mounted && allowed == true) await save(restore: true);
  }

  Future<void> save({bool restore = false}) => perform(() async {
    final current = document!;
    await api.editProjectInstructions(
      project: current.folder,
      agent: widget.agent,
      revision: current.revision,
      text: restore ? null : markdown.text,
    );
    await load(current.folder);
  });

  @override
  Widget build(BuildContext context) {
    final strings = context.strings;
    final current = document;
    final colors = Theme.of(context).colorScheme;
    return AlertDialog(
      scrollable: true,
      title: Text(strings.projectInstructions),
      content: SizedBox(
        width: 620,
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            TextField(
              controller: folder,
              enabled: !busy && current == null,
              decoration: InputDecoration(
                labelText: strings.projectFolder,
                suffixIcon: current == null
                    ? IconButton(
                        tooltip: strings.projectFolder,
                        icon: const Icon(Icons.folder_open_outlined),
                        onPressed: busy
                            ? null
                            : () => perform(() async {
                                final selected = await getDirectoryPath();
                                if (selected != null && mounted) {
                                  await load(selected);
                                }
                              }),
                      )
                    : null,
              ),
              onSubmitted: (value) {
                if (value.trim().isNotEmpty) perform(() => load(value.trim()));
              },
            ),
            if (busy) const LinearProgressIndicator(),
            if (current != null) ...[
              const SizedBox(height: 12),
              SelectableText(
                current.path,
                style: TextStyle(color: colors.onSurfaceVariant),
              ),
              const SizedBox(height: 14),
              TextField(
                controller: markdown,
                enabled: !busy,
                minLines: 6,
                maxLines: 12,
                decoration: const InputDecoration(labelText: 'Markdown'),
              ),
            ],
            if (error case final String message)
              Padding(
                padding: const EdgeInsets.only(top: 12),
                child: SelectableText(
                  message,
                  style: TextStyle(color: colors.error),
                ),
              ),
          ],
        ),
      ),
      actions: [
        TextButton(
          onPressed: busy ? null : () => Navigator.pop(context),
          child: Text(MaterialLocalizations.of(context).closeButtonLabel),
        ),
        if (current == null)
          FilledButton(
            onPressed: busy
                ? null
                : () {
                    final value = folder.text.trim();
                    if (value.isNotEmpty) perform(() => load(value));
                  },
            child: Text(strings.openProject),
          )
        else ...[
          if (current.canRestore)
            TextButton(
              onPressed: busy ? null : confirmRestore,
              child: Text(strings.restoreInstructions),
            ),
          if (error != null)
            TextButton(
              onPressed: busy
                  ? null
                  : () => perform(() => load(current.folder)),
              child: Text(strings.reload),
            ),
          ValueListenableBuilder(
            valueListenable: markdown,
            builder: (context, value, _) => FilledButton(
              onPressed: busy || current.text == value.text
                  ? null
                  : () => save(),
              child: Text(strings.save),
            ),
          ),
        ],
      ],
    );
  }
}
