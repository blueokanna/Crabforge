import 'dart:convert';

import 'package:flutter/material.dart';

import 'language.dart';
import 'material_symbols.dart';
import 'src/rust/api.dart' as api;

class MemoryTidySettings extends StatefulWidget {
  const MemoryTidySettings({super.key, required this.models});

  final List<String> models;

  @override
  State<MemoryTidySettings> createState() => _MemoryTidySettingsState();
}

class _MemoryTidySettingsState extends State<MemoryTidySettings> {
  Map<String, dynamic>? status;
  String? selected;
  String? error;
  bool enabled = false;
  bool busy = true;

  @override
  void initState() {
    super.initState();
    load();
  }

  Future<void> load() => perform(api.memoryTidyStatus, readSchedule: true);

  Future<void> perform(
    Future<String> Function() action, {
    bool readSchedule = false,
  }) async {
    setState(() {
      busy = true;
      error = null;
    });
    try {
      final result = jsonDecode(await action()) as Map<String, dynamic>;
      if (!mounted) return;
      setState(() {
        status = result;
        if (readSchedule) {
          final saved = result['schedule'] as Map<String, dynamic>?;
          enabled = saved != null;
          selected = saved?['model'] as String? ?? selected;
        }
      });
    } catch (e) {
      if (mounted) setState(() => error = '$e');
    } finally {
      if (mounted) setState(() => busy = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    final strings = context.strings;
    final local = MaterialLocalizations.of(context);
    final saved = status?['schedule'] as Map<String, dynamic>?;
    final last = status?['last'] as Map<String, dynamic>?;
    final model = selected;
    final models = {...widget.models, ?model};
    final dirty =
        enabled != (saved != null) || (enabled && model != saved?['model']);
    final shownError = error ?? status?['error'] as String?;
    return AlertDialog(
      title: Text(strings.memoryTidy),
      scrollable: true,
      content: SizedBox(
        width: 480,
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          mainAxisSize: MainAxisSize.min,
          children: [
            Text(strings.memoryTidyHelp),
            if (busy) const LinearProgressIndicator(),
            if (status != null) ...[
              SwitchListTile(
                contentPadding: EdgeInsets.zero,
                title: Text(
                  enabled ? strings.distillationEnabled : strings.disabled,
                ),
                value: enabled,
                onChanged: busy
                    ? null
                    : (value) => setState(() => enabled = value),
              ),
              DropdownButtonFormField<String>(
                key: ValueKey(saved?['model']),
                icon: const Icon(CrabforgeSymbols.arrowDropDown),
                initialValue: model,
                isExpanded: true,
                menuMaxHeight: 360,
                decoration: InputDecoration(labelText: strings.models),
                items: [
                  for (final id in models)
                    DropdownMenuItem(
                      value: id,
                      child: Text(id, overflow: TextOverflow.ellipsis),
                    ),
                ],
                onChanged: busy
                    ? null
                    : (value) => setState(() => selected = value),
              ),
              if (last?['changed'] case final int changed) ...[
                const SizedBox(height: 12),
                Text(strings.memoryTidyChanged(local.formatDecimal(changed))),
              ],
              if (last?['restored'] case final int restored)
                Text(strings.memoryTidyRestored(local.formatDecimal(restored))),
            ],
            if (shownError != null) SelectableText(shownError),
          ],
        ),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.pop(context),
          child: Text(local.closeButtonLabel),
        ),
        if (status == null && shownError != null)
          TextButton(
            onPressed: busy ? null : load,
            child: Text(strings.reload),
          ),
        if (status?['can_undo'] == true)
          TextButton(
            onPressed: busy ? null : () => perform(api.undoMemoryTidy),
            child: Text(strings.memoryTidyUndo),
          ),
        TextButton(
          onPressed: busy || model == null
              ? null
              : () => perform(() => api.runMemoryTidy(model: model)),
          child: Text(strings.memoryTidyRun),
        ),
        FilledButton(
          onPressed: busy || !dirty || (enabled && model == null)
              ? null
              : () => perform(
                  () => api.configureMemoryTidy(model: enabled ? model : null),
                  readSchedule: true,
                ),
          child: Text(strings.save),
        ),
      ],
    );
  }
}
