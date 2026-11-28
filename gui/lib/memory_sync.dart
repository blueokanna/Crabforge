import 'dart:convert';

import 'package:flutter/material.dart';

import 'language.dart';
import 'src/rust/api.dart' as api;

class MemorySync extends StatefulWidget {
  const MemorySync({super.key});

  @override
  State<MemorySync> createState() => _MemorySyncState();
}

class _MemorySyncState extends State<MemorySync> {
  final url = TextEditingController();
  final user = TextEditingController();
  final password = TextEditingController();
  final passphrase = TextEditingController();
  Map<String, dynamic>? report;
  String? error;
  bool busy = false;
  bool applied = false;

  @override
  void dispose() {
    url.dispose();
    user.dispose();
    password.dispose();
    passphrase.dispose();
    super.dispose();
  }

  void edit(String _) => setState(() {
    report = null;
    error = null;
  });

  Future<void> sync({String? revision}) async {
    setState(() {
      busy = true;
      error = null;
      report = null;
    });
    try {
      final result = jsonDecode(
        await api.synchronizeMemories(
          url: url.text,
          user: user.text,
          password: password.text,
          passphrase: passphrase.text,
          revision: revision,
        ),
      ) as Map<String, dynamic>;
      if (!mounted) return;
      setState(() {
        report = result;
        applied |= result['applied'] == true;
      });
    } catch (e) {
      if (mounted) setState(() => error = '$e');
    } finally {
      if (mounted) setState(() => busy = false);
    }
  }

  Widget field(
    TextEditingController controller,
    String label, {
    bool secret = false,
    TextInputType? keyboard,
  }) => TextField(
    controller: controller,
    enabled: !busy,
    onChanged: edit,
    obscureText: secret,
    enableSuggestions: !secret,
    autocorrect: false,
    keyboardType: keyboard,
    decoration: InputDecoration(labelText: label),
  );

  @override
  Widget build(BuildContext context) {
    final strings = context.strings;
    final local = MaterialLocalizations.of(context);
    final result = report;
    final revision = result?['revision'] as String?;
    return PopScope(
      canPop: !busy,
      child: AlertDialog(
        title: Text(strings.memorySync),
        scrollable: true,
        content: SizedBox(
          width: 480,
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              field(url, strings.memorySyncUrl, keyboard: TextInputType.url),
              field(user, strings.memorySyncUser),
              field(password, strings.memorySyncPassword, secret: true),
              field(passphrase, strings.memorySyncPassphrase, secret: true),
              const SizedBox(height: 12),
              if (busy) const LinearProgressIndicator(),
              if (result != null)
                Text(
                  strings.memorySyncChanges(
                    local.formatDecimal(result['downloaded'] as int),
                    local.formatDecimal(result['updated'] as int),
                    local.formatDecimal(result['removed'] as int),
                    local.formatDecimal(result['uploaded'] as int),
                  ),
                ),
              if (result?['applied'] == true) Text(strings.memorySyncComplete),
              if (error case final String message) SelectableText(message),
            ],
          ),
        ),
        actions: [
          TextButton(
            onPressed: busy ? null : () => Navigator.pop(context, applied),
            child: Text(local.closeButtonLabel),
          ),
          TextButton(
            onPressed:
                busy || url.text.trim().isEmpty || passphrase.text.isEmpty
                ? null
                : () => sync(),
            child: Text(strings.memorySyncPreview),
          ),
          FilledButton(
            onPressed: busy || revision == null || result?['applied'] == true
                ? null
                : () => sync(revision: revision),
            child: Text(strings.memorySyncApply),
          ),
        ],
      ),
    );
  }
}
