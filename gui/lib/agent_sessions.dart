import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import 'collection_view.dart';
import 'language.dart';
import 'src/rust/api.dart' as api;

const _agentNames = {'claude': 'Claude Code', 'codex': 'Codex'};

class AgentSessions extends StatefulWidget {
  const AgentSessions({super.key});

  @override
  State<AgentSessions> createState() => _AgentSessionsState();
}

class _AgentSessionsState extends State<AgentSessions> {
  String agent = '';
  late Future<Map<String, dynamic>> query = fetch();

  Future<Map<String, dynamic>> fetch() async =>
      jsonDecode(await api.agentSessions(agent: agent.isEmpty ? null : agent))
          as Map<String, dynamic>;

  void refresh() {
    final result = fetch();
    setState(() {
      query = result;
    });
  }

  Future<void> details(Map<String, dynamic> session) async {
    final id = session['id'] as String;
    final locale = MaterialLocalizations.of(context);
    final time = DateTime.parse(session['modified'] as String).toLocal();
    await showDialog<void>(
      context: context,
      builder: (context) => AlertDialog(
        scrollable: true,
        title: Text(_agentNames[session['agent']]!),
        content: SizedBox(
          width: 540,
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            mainAxisSize: MainAxisSize.min,
            children: [
              SelectableText(session['title'] as String),
              const SizedBox(height: 12),
              SelectableText(session['cwd'] as String),
              SelectableText(id),
              Text(
                '${locale.formatMediumDate(time)} · '
                '${locale.formatTimeOfDay(TimeOfDay.fromDateTime(time))}',
              ),
            ],
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: Text(locale.closeButtonLabel),
          ),
          TextButton.icon(
            onPressed: () async {
              await Clipboard.setData(ClipboardData(text: id));
              if (context.mounted) Navigator.pop(context);
            },
            icon: const Icon(Icons.copy_outlined),
            label: Text(context.strings.copySessionId),
          ),
        ],
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final strings = context.strings;
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
                child: EditorHeading(strings.agentSessions),
              ),
              Row(
                children: [
                  Expanded(
                    child: DropdownButton<String>(
                      value: agent,
                      isExpanded: true,
                      items: [
                        DropdownMenuItem(
                          value: '',
                          child: Text(strings.allAgents),
                        ),
                        for (final entry in _agentNames.entries)
                          DropdownMenuItem(
                            value: entry.key,
                            child: Text(entry.value),
                          ),
                      ],
                      onChanged: (value) {
                        if (value == null || value == agent) return;
                        agent = value;
                        refresh();
                      },
                    ),
                  ),
                  IconButton(
                    onPressed: refresh,
                    icon: const Icon(Icons.refresh),
                    tooltip: strings.reload,
                  ),
                ],
              ),
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
                    final sessions = (result['sessions'] as List)
                        .cast<Map<String, dynamic>>();
                    final unreadable = result['unreadable'] as int;
                    return Column(
                      crossAxisAlignment: CrossAxisAlignment.stretch,
                      children: [
                        if (unreadable > 0)
                          Text(strings.unreadableSessions(unreadable)),
                        Expanded(
                          child: sessions.isEmpty
                              ? Center(child: Text(strings.noAgentSessions))
                              : EditorCollection(
                                  count: sessions.length,
                                  itemBuilder: (context, index) {
                                    final session = sessions[index];
                                    final name = _agentNames[session['agent']];
                                    final status = session['archived'] == true
                                        ? ' · ${strings.memoryState('archived')}'
                                        : '';
                                    return Card.outlined(
                                      clipBehavior: Clip.hardEdge,
                                      child: ListTile(
                                        title: Text(
                                          session['title'] as String,
                                          maxLines: 2,
                                          overflow: TextOverflow.ellipsis,
                                        ),
                                        subtitle: Text('$name$status'),
                                        onTap: () => details(session),
                                      ),
                                    );
                                  },
                                ),
                        ),
                      ],
                    );
                  },
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
