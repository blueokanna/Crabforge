import 'package:file_selector/file_selector.dart';
import 'package:flutter/material.dart';

import 'material_symbols.dart';
import 'language.dart';

Future<String?> openProjectFolder(BuildContext context, String? current) {
  final strings = context.strings;
  var folder = current ?? '';
  return showDialog<String>(
    context: context,
    builder: (dialogContext) {
      void open() {
        if (folder.trim().isNotEmpty) {
          Navigator.pop(dialogContext, folder.trim());
        }
      }

      return AlertDialog(
        title: Text(strings.openProject),
        content: TextFormField(
          initialValue: folder,
          onChanged: (value) => folder = value,
          onFieldSubmitted: (_) => open(),
          decoration: InputDecoration(
            labelText: strings.projectFolder,
            suffixIcon: IconButton(
              tooltip: strings.projectFolder,
              icon: const Icon(CrabforgeSymbols.folderOpen),
              onPressed: () async {
                final path = await getDirectoryPath(initialDirectory: current);
                if (dialogContext.mounted && path != null) {
                  Navigator.pop(dialogContext, path);
                }
              },
            ),
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(dialogContext),
            child: Text(strings.cancel),
          ),
          FilledButton(onPressed: open, child: Text(strings.openProject)),
        ],
      );
    },
  );
}
