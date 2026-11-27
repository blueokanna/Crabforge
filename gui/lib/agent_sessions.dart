import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import 'material_symbols.dart';
import 'collection_view.dart';
import 'language.dart';
import 'memory.dart';
import 'src/rust/api.dart' as api;

const _agentNames = {
  'claude': 'Claude Code',
  'codex': 'Codex',
  'gemini': 'Gemini CLI',
  'grok': 'Grok Build',
  'pi': 'Pi',
};

class AgentSessions extends StatefulWidget {
  const AgentSessions({super.key, required this.models});

  final List<String> models;

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
    final strings = context.strings;
    final time = DateTime.parse(session['modified'] as String).toLocal();
    final propose = await showDialog<bool>(
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
              Text(strings.sessionWorkingFolder),
              SelectableText(session['cwd'] as String),
              const SizedBox(height: 8),
              Text(strings.sessionProfileFolder),
              SelectableText(session['profile'] as String),
              const SizedBox(height: 8),
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
            icon: const Icon(CrabforgeSymbols.contentCopy),
            label: Text(context.strings.copySessionId),
          ),
          TextButton(
            onPressed: widget.models.isEmpty
                ? null
                : () => Navigator.pop(context, true),
            child: Text(strings.distillTitle),
          ),
        ],
      ),
    );
    if (propose != true || !mounted) return;
    final saved = await showDialog<bool>(
      context: context,
      barrierDismissible: false,
      builder: (_) =>
          _SessionMemoryPreview(session: session, models: widget.models),
    );
    if (saved != true || !mounted) return;
    await showDialog<void>(
      context: context,
      builder: (_) => const MemoryLibrary(initialStatus: 'candidate'),
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
                      icon: const Icon(CrabforgeSymbols.arrowDropDown),
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
                    icon: const Icon(CrabforgeSymbols.refresh),
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

class _SessionMemoryPreview extends StatefulWidget {
  const _SessionMemoryPreview({required this.session, required this.models});

  final Map<String, dynamic> session;
  final List<String> models;

  @override
  State<_SessionMemoryPreview> createState() => _SessionMemoryPreviewState();
}

class _SessionMemoryPreviewState extends State<_SessionMemoryPreview> {
  late final agent = widget.session['agent'] as String;
  late final profile = widget.session['profile'] as String;
  late final session = widget.session['id'] as String;
  late Future<Map<String, dynamic>> preview = load();
  String? model;
  String? error;
  bool busy = false;
  bool empty = false;

  Future<Map<String, dynamic>> load() async => jsonDecode(
    await api.previewSessionMemory(
      agent: agent,
      profile: profile,
      session: session,
    ),
  ) as Map<String, dynamic>;

  Future<void> send(String revision) async {
    setState(() {
      busy = true;
      empty = false;
      error = null;
    });
    try {
      final result = jsonDecode(
        await api.proposeSessionMemory(
          agent: agent,
          profile: profile,
          session: session,
          model: model!,
          revision: revision,
        ),
      ) as List;
      if (!mounted) return;
      if (result.isNotEmpty) {
        Navigator.pop(context, true);
      } else {
        setState(() => empty = true);
      }
    } catch (e) {
      if (mounted) setState(() => error = '$e');
    } finally {
      if (mounted) setState(() => busy = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    final strings = context.strings;
    return PopScope(
      canPop: !busy,
      child: FutureBuilder<Map<String, dynamic>>(
        future: preview,
        builder: (context, snapshot) {
          final ready =
              snapshot.connectionState == ConnectionState.done &&
              snapshot.hasData;
          return AlertDialog(
            title: Text(strings.distillTitle),
            scrollable: true,
            content: SizedBox(
              width: 560,
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                mainAxisSize: MainAxisSize.min,
                children: [
                  if (snapshot.connectionState != ConnectionState.done || busy)
                    const LinearProgressIndicator(),
                  if (ready) ...[
                    Text(strings.distillPreview),
                    const SizedBox(height: 12),
                    DropdownButtonFormField<String>(
                      icon: const Icon(CrabforgeSymbols.arrowDropDown),
                      initialValue: model,
                      isExpanded: true,
                      menuMaxHeight: 360,
                      decoration: InputDecoration(labelText: strings.models),
                      items: [
                        for (final id in widget.models)
                          DropdownMenuItem(
                            value: id,
                            child: Text(id, overflow: TextOverflow.ellipsis),
                          ),
                      ],
                      onChanged: busy
                          ? null
                          : (value) => setState(() => model = value),
                    ),
                    const SizedBox(height: 12),
                    SelectableText(
                      (snapshot.requireData['messages'] as List).join('\n\n'),
                    ),
                  ],
                  if (snapshot.hasError || error != null)
                    SelectableText(error ?? '${snapshot.error}'),
                  if (empty) Text(strings.distillEmpty),
                ],
              ),
            ),
            actions: [
              TextButton(
                onPressed: busy ? null : () => Navigator.pop(context),
                child: Text(strings.cancel),
              ),
              if (error != null || snapshot.hasError)
                TextButton(
                  onPressed: busy
                      ? null
                      : () => setState(() {
                          preview = load();
                          model = null;
                          error = null;
                        }),
                  child: Text(strings.reload),
                ),
              FilledButton(
                onPressed: ready && !busy && model != null
                    ? () => send(snapshot.requireData['revision'] as String)
                    : null,
                child: Text(strings.distillSend),
              ),
            ],
          );
        },
      ),
    );
  }
}
