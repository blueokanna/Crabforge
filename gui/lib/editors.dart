import 'dart:convert';

import 'package:flutter/material.dart';

import 'material_symbols.dart';
import 'brand_icon.dart';
import 'collection_view.dart';
import 'language.dart';
import 'src/rust/api.dart' as api;

typedef JsonMap = Map<String, dynamic>;

class SharedGroupImport extends StatefulWidget {
  const SharedGroupImport({super.key, required this.configuration});
  final String configuration;

  @override
  State<SharedGroupImport> createState() => _SharedGroupImportState();
}

class _SharedGroupImportState extends State<SharedGroupImport> {
  String text = '';
  String? error;
  bool replace = false;
  bool loading = false;

  Future<void> submit() async {
    setState(() {
      loading = true;
      error = null;
    });
    try {
      final result = await api.importedGroup(
        configuration: widget.configuration,
        text: text,
        replace: replace,
      );
      if (mounted) Navigator.pop(context, jsonDecode(result) as JsonMap);
    } catch (e) {
      if (mounted) {
        setState(() {
          error = '$e';
          loading = false;
        });
      }
    }
  }

  @override
  Widget build(BuildContext context) => AlertDialog(
    scrollable: true,
    title: Text(context.strings.importGroup),
    content: SizedBox(
      width: 480,
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          TextField(
            autofocus: true,
            enabled: !loading,
            minLines: 3,
            maxLines: 8,
            decoration: InputDecoration(
              labelText: context.strings.sharedGroupText,
              errorText: error,
              errorMaxLines: 5,
            ),
            onChanged: (value) => setState(() {
              text = value;
              error = null;
            }),
          ),
          CheckboxListTile(
            contentPadding: EdgeInsets.zero,
            title: Text(context.strings.replaceGroup),
            value: replace,
            onChanged: loading
                ? null
                : (value) => setState(() => replace = value ?? false),
          ),
        ],
      ),
    ),
    actions: [
      TextButton(
        onPressed: () => Navigator.pop(context),
        child: Text(context.strings.cancel),
      ),
      FilledButton(
        onPressed: loading || text.trim().isEmpty ? null : submit,
        child: Text(context.strings.save),
      ),
    ],
  );
}

const protocolLabels = {
  'chat': 'Chat Completions',
  'responses': 'Responses',
  'anthropic': 'Anthropic Messages',
  'gemini': 'Gemini',
};

class ProviderEditor extends StatefulWidget {
  const ProviderEditor({super.key, required this.presets, this.provider});
  final List<JsonMap> presets;
  final JsonMap? provider;
  @override
  State<ProviderEditor> createState() => _ProviderEditorState();
}

class _ProviderEditorState extends State<ProviderEditor> {
  final form = GlobalKey<FormState>();
  bool loading = false;
  String? discoveryMessage;
  String presetModelText = '';
  late String authHeader = widget.provider?['endpoints']?['auth_header'] ?? '';
  JsonMap? resource;
  static const expressBase =
      'https://aiplatform.googleapis.com/v1/publishers/google';
  late bool vertexPreset =
      widget.provider?['endpoints']?['vertex'] != null ||
      widget.provider?['endpoints']?['gemini'] == expressBase;
  late bool vertexProject = savedVertex != null;
  JsonMap? get savedVertex =>
      widget.provider?['endpoints']?['vertex'] as JsonMap?;
  late final project = TextEditingController(text: savedVertex?['project']);
  late final location = TextEditingController(
    text: savedVertex?['location'] ?? 'global',
  );
  late final credentials = TextEditingController(
    text: savedVertex?['credentials'],
  );
  late final impersonate = TextEditingController(
    text: savedVertex?['impersonate'],
  );
  late final vertexBase = TextEditingController(
    text: savedVertex?['base_url'] ?? savedVertex?['baseUrl'],
  );
  final resourceUrl = TextEditingController();
  JsonMap get endpoints => {
    if (vertexProject)
      'vertex': {
        'project': project.text.trim().toLowerCase(),
        'location': location.text.trim().toLowerCase(),
        'credentials': credentials.text.trim(),
        'impersonate': impersonate.text.trim(),
        'base_url': vertexBase.text.trim(),
      }
    else
      for (final e in urls.entries) e.key: e.value.text.trim(),
    if (modelsUrl.text.trim().isNotEmpty) 'models_url': modelsUrl.text.trim(),
    if (!vertexProject &&
        urls['anthropic']!.text.trim().isNotEmpty &&
        authHeader.isNotEmpty)
      'auth_header': authHeader,
  };

  Future<void> discover() async {
    final bases = jsonEncode(endpoints), key = keyEnv.text.trim();
    setState(() {
      loading = true;
      discoveryMessage = null;
    });
    try {
      final result = jsonDecode(
        await api.discoverModelCatalog(endpoints: bases, keyEnv: key),
      ) as JsonMap;
      final ids = List<String>.from(result['models']);
      if (!mounted) return;
      if (bases != jsonEncode(endpoints) || key != keyEnv.text.trim()) {
        discoveryMessage = context.strings.changed;
      } else {
        models.text = {...linesOf(models.text), ...ids}.join('\n');
        freeModels = Set<String>.from(result['free_models']);
        modelProtocols = Map<String, dynamic>.from(
          result['model_protocols'] ?? {},
        );
        discoveryMessage = context.strings.modelsRead(ids.length);
      }
    } catch (e) {
      discoveryMessage = '$e';
    } finally {
      if (mounted) setState(() => loading = false);
    }
  }

  late final id = TextEditingController(text: widget.provider?['id']);
  late Set<String> freeModels = Set<String>.from(
    widget.provider?['free_models'] ?? [],
  );
  late JsonMap modelProtocols = Map<String, dynamic>.from(
    widget.provider?['model_protocols'] ?? {},
  );
  late final models = TextEditingController(
    text: (widget.provider?['models'] as List?)?.join('\n'),
  );
  late final fallback = TextEditingController(
    text: (widget.provider?['fallback'] as List?)?.join('\n'),
  );
  late bool allowFallback = widget.provider?['no_stand_in'] != true;
  late final keyEnv = TextEditingController(text: widget.provider?['key_env']);
  late final catalog = TextEditingController(text: widget.provider?['catalog']);
  late final modelsUrl = TextEditingController(
    text: widget.provider?['endpoints']['models_url'],
  );
  late final urls = {
    for (final p in protocolLabels.keys)
      p: TextEditingController(text: widget.provider?['endpoints'][p]),
  };

  @override
  void dispose() {
    for (final c in [
      id,
      models,
      fallback,
      keyEnv,
      catalog,
      modelsUrl,
      resourceUrl,
      project,
      location,
      credentials,
      impersonate,
      vertexBase,
      ...urls.values,
    ]) {
      c.dispose();
    }
    super.dispose();
  }

  void select(JsonMap? p) {
    if (p == null) return;
    freeModels.clear();
    modelProtocols.clear();
    id.text = p['id'];
    catalog.text = p['catalog'] ?? '';
    modelsUrl.text = p['endpoints']['models_url'] ?? '';
    final presetModels = (p['models'] as List?)?.cast<String>() ?? [];
    final usePresetModels =
        models.text.isEmpty || models.text == presetModelText;
    presetModelText = presetModels.join('\n');
    if (usePresetModels) models.text = presetModelText;
    setState(() {
      resource = p['resource'];
      authHeader = p['endpoints']['auth_header'] ?? '';
      vertexPreset = p['id'] == 'google-vertex';
      vertexProject = false;
      resourceUrl.clear();
      discoveryMessage = usePresetModels && presetModels.isNotEmpty
          ? context.strings.presetLoaded
          : null;
    });
    keyEnv.text =
        resource == null &&
            (p['endpoints'].values as Iterable).every(
              (v) => v.toString().startsWith('http://localhost'),
            )
        ? ''
        : '${RegExp(r'^[0-9]').hasMatch(id.text) ? 'PROVIDER_' : ''}${id.text.toUpperCase().replaceAll(RegExp('[^A-Z0-9_]'), '_')}_API_KEY';
    for (final e in urls.entries) {
      e.value.text = p['endpoints'][e.key] ?? '';
    }
  }

  @override
  Widget build(BuildContext context) => AlertDialog(
    title: Text(
      widget.provider == null
          ? context.strings.addProvider
          : context.strings.editProvider,
    ),
    content: SizedBox(
      width: 520,
      child: SingleChildScrollView(
        child: Form(
          key: form,
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              if (widget.provider == null)
                DropdownMenu<JsonMap>(
                  trailingIcon: const Icon(CrabforgeSymbols.arrowDropDown),
                  selectedTrailingIcon: const Icon(
                    CrabforgeSymbols.arrowDropUp,
                  ),
                  label: Text(context.strings.searchPresets),
                  expandedInsets: EdgeInsets.zero,
                  enableFilter: true,
                  requestFocusOnTap: true,
                  menuHeight: 280,
                  dropdownMenuEntries: [
                    for (final p in widget.presets)
                      DropdownMenuEntry(
                        value: p,
                        label: p['name'],
                        leadingIcon: BrandIcon.provider(p['id']),
                      ),
                  ],
                  onSelected: select,
                ),
              field(
                context,
                id,
                context.strings.providerId,
                enabled: widget.provider == null,
                required: true,
              ),
              field(
                context,
                keyEnv,
                context.strings.keyEnv,
                hint: context.strings.keyHint,
              ),
              if (vertexPreset)
                SwitchListTile(
                  title: Text(context.strings.vertexProject),
                  value: vertexProject,
                  onChanged: (enabled) => setState(() {
                    vertexProject = enabled;
                    if (enabled && keyEnv.text == 'GOOGLE_VERTEX_API_KEY') {
                      keyEnv.clear();
                    }
                    if (!enabled) {
                      urls['gemini']!.text = expressBase;
                      if (keyEnv.text.isEmpty) {
                        keyEnv.text = 'GOOGLE_VERTEX_API_KEY';
                      }
                    }
                  }),
                ),
              if (vertexProject) ...[
                field(context, project, context.strings.cloudProjectId),
                field(
                  context,
                  location,
                  context.strings.vertexLocation,
                  hint: 'global, eu, us-central1',
                ),
                field(
                  context,
                  credentials,
                  context.strings.googleCredentials,
                  hint: 'GOOGLE_APPLICATION_CREDENTIALS',
                ),
                field(context, impersonate, context.strings.impersonateAccount),
                field(
                  context,
                  vertexBase,
                  context.strings.resourceRoot,
                  hint: 'https://aiplatform.googleapis.com',
                ),
              ],
              if (resource != null)
                Padding(
                  padding: const EdgeInsets.only(top: 14),
                  child: TextFormField(
                    controller: resourceUrl,
                    decoration: InputDecoration(
                      labelText: context.strings.resourceRoot,
                      hintText: resource!['hint'],
                      border: const OutlineInputBorder(),
                    ),
                    onChanged: (raw) {
                      final base = raw.trim().replaceFirst(RegExp(r'/+$'), '');
                      for (final e in (resource!['paths'] as JsonMap).entries) {
                        urls[e.key]!.text = base.isEmpty
                            ? ''
                            : '$base${e.value}';
                      }
                    },
                  ),
                ),
              field(
                context,
                models,
                context.strings.modelIds,
                lines: 3,
                required: true,
              ),
              if (!vertexProject)
                for (final e in urls.entries)
                  field(
                    context,
                    e.value,
                    context.strings.protocolBase(protocolLabels[e.key]!),
                  ),
              if (!vertexProject)
                ValueListenableBuilder<TextEditingValue>(
                  valueListenable: urls['anthropic']!,
                  builder: (context, url, _) => url.text.trim().isEmpty
                      ? const SizedBox.shrink()
                      : Padding(
                          padding: const EdgeInsets.only(top: 14),
                          child: DropdownMenu<String>(
                            trailingIcon: const Icon(
                              CrabforgeSymbols.arrowDropDown,
                            ),
                            selectedTrailingIcon: const Icon(
                              CrabforgeSymbols.arrowDropUp,
                            ),
                            key: ValueKey(authHeader),
                            label: Text(context.strings.anthropicAuthHeader),
                            initialSelection: authHeader,
                            expandedInsets: EdgeInsets.zero,
                            dropdownMenuEntries: [
                              DropdownMenuEntry(
                                value: '',
                                label: context.strings.automaticAuthHeader,
                              ),
                              const DropdownMenuEntry(
                                value: 'x-api-key',
                                label: 'x-api-key',
                              ),
                              const DropdownMenuEntry(
                                value: 'bearer',
                                label: 'Authorization: Bearer',
                              ),
                            ],
                            onSelected: (value) {
                              if (value != null) {
                                setState(() => authHeader = value);
                              }
                            },
                          ),
                        ),
                ),
              field(
                context,
                modelsUrl,
                context.strings.catalogUrl,
                hint: context.strings.catalogHint,
              ),
              field(
                context,
                catalog,
                context.strings.providerCatalogs,
                hint: 'kimi-code-plan-global, vercel',
              ),
              TextButton.icon(
                onPressed: loading ? null : discover,
                icon: const Icon(CrabforgeSymbols.sync),
                label: Text(
                  loading
                      ? context.strings.readingModels
                      : context.strings.readModels,
                ),
              ),
              if (discoveryMessage != null) SelectableText(discoveryMessage!),
              field(
                context,
                fallback,
                context.strings.providerFallback,
                hint: 'provider/model\ngroup/backup',
                lines: 2,
              ),
              SwitchListTile(
                title: Text(context.strings.allowProviderFallback),
                value: allowFallback,
                onChanged: (value) => setState(() => allowFallback = value),
              ),
            ],
          ),
        ),
      ),
    ),
    actions: [
      TextButton(
        onPressed: () => Navigator.pop(context),
        child: Text(context.strings.cancel),
      ),
      FilledButton(
        onPressed: () {
          if (form.currentState!.validate()) {
            Navigator.pop(context, <String, dynamic>{
              'id': id.text.trim(),
              'key_env': keyEnv.text.trim(),
              if (catalog.text.trim().isNotEmpty)
                'catalog': catalog.text.trim(),
              'models': linesOf(models.text),
              if (fallback.text.trim().isNotEmpty)
                'fallback': linesOf(fallback.text),
              if (!allowFallback) 'no_stand_in': true,
              if (modelProtocols.isNotEmpty)
                'model_protocols': {
                  for (final id in linesOf(models.text))
                    if (modelProtocols.containsKey(id)) id: modelProtocols[id],
                },
              if (freeModels.isNotEmpty)
                'free_models': linesOf(models.text)
                    .where(freeModels.contains)
                    .toList(),
              'endpoints': endpoints,
            });
          }
        },
        child: Text(context.strings.save),
      ),
    ],
  );
}

Widget field(
  BuildContext context,
  TextEditingController c,
  String label, {
  String? hint,
  int lines = 1,
  bool required = false,
  bool enabled = true,
}) => Padding(
  padding: const EdgeInsets.only(top: 14),
  child: TextFormField(
    controller: c,
    minLines: lines,
    maxLines: lines == 1 ? 1 : 6,
    enabled: enabled,
    decoration: InputDecoration(
      labelText: label,
      hintText: hint,
      border: const OutlineInputBorder(),
    ),
    validator: (s) => required && (s == null || s.trim().isEmpty)
        ? context.strings.requiredField
        : null,
  ),
);

List<String> linesOf(String text) =>
    text.split('\n').map((s) => s.trim()).where((s) => s.isNotEmpty).toList();

enum GroupStrategy { order, rotate, weight, manual }

typedef GroupSelection = ({
  String name,
  String note,
  int firstContent,
  int patience,
  bool listMembers,
  List<String> off,
  List<String> members,
  List<String> patterns,
  List<String> placed,
  GroupStrategy strategy,
  Map<String, int> weights,
  String? pick,
});

class GroupEditor extends StatefulWidget {
  const GroupEditor({
    super.key,
    required this.choices,
    required this.configuration,
    required this.patterns,
    required this.placed,
    required this.members,
    required this.strategy,
    required this.weights,
    required this.off,
    required this.listMembers,
    required this.firstContent,
    required this.patience,
    this.pick,
    this.name,
    this.note,
  });
  final String? name, note;
  final bool listMembers;
  final int firstContent;
  final int patience;
  final GroupStrategy strategy;
  final Map<String, int> weights;
  final String? pick;
  final String configuration;
  final List<String> choices, members, off, patterns, placed;
  @override
  State<GroupEditor> createState() => _GroupEditorState();
}

class _GroupEditorState extends State<GroupEditor> {
  late final name = TextEditingController(text: widget.name);
  late final note = TextEditingController(text: widget.note);
  final firstContent = GlobalKey<FormFieldState<String>>();
  final patience = GlobalKey<FormFieldState<String>>();
  late bool listMembers = widget.listMembers;
  late final named = [...widget.members];
  late final patterns = TextEditingController(text: widget.patterns.join('\n'));
  late List<String> placed = [...widget.placed];
  List<String> matched = [];
  bool matching = false;
  String? patternError;
  int revision = 0;

  @override
  void initState() {
    super.initState();
    patterns.addListener(preview);
    if (widget.patterns.isNotEmpty) preview();
  }

  Future<void> preview() async {
    final rules = linesOf(patterns.text);
    final current = ++revision;
    setState(() {
      matching = rules.isNotEmpty;
      patternError = null;
      matched = [];
    });
    if (rules.isEmpty) return;
    try {
      final ids = await api.matchGroupModels(
        configuration: widget.configuration,
        patterns: rules,
      );
      if (mounted && current == revision) {
        setState(() {
          matched = ids;
          matching = false;
        });
      }
    } catch (e) {
      if (mounted && current == revision) {
        setState(() {
          patternError = '$e';
          matching = false;
        });
      }
    }
  }

  late final off = widget.off.toSet();
  late GroupStrategy strategy = widget.strategy;
  late final weights = {...widget.weights};
  final weightInputs = <String, TextEditingController>{};
  late String? pick = widget.pick;
  @override
  void dispose() {
    name.dispose();
    note.dispose();
    patterns.dispose();
    for (final input in weightInputs.values) {
      input.dispose();
    }
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final memberSet = {...named, ...matched};
    final remaining = {...memberSet};
    final members = [...placed.where(remaining.remove), ...remaining];
    final selected = memberSet.contains(pick) ? pick : members.firstOrNull;
    final rules = linesOf(patterns.text);
    final grid = CollectionLayout.of(context).grid;
    return AlertDialog(
      title: EditorHeading(
        widget.name == null
            ? context.strings.createGroup
            : context.strings.editGroup,
      ),
      content: SizedBox(
        width: grid ? 660 : 480,
        child: SingleChildScrollView(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              field(context, name, context.strings.groupId),
              field(context, note, context.strings.groupNote),
              _SecondsField(
                fieldKey: firstContent,
                seconds: widget.firstContent,
                label: context.strings.firstContentSeconds,
                invalid: context.strings.invalidFirstContent,
                maximum: 2147483647,
                hint: context.strings.groupFirstContentHelp,
              ),
              _SecondsField(
                fieldKey: patience,
                seconds: widget.patience,
                label: context.strings.groupPatienceSeconds,
                invalid: context.strings.invalidGroupPatience,
                maximum: 240,
                hint: context.strings.groupPatienceHelp,
              ),
              Padding(
                padding: const EdgeInsets.only(top: 14),
                child: TextField(
                  controller: patterns,
                  minLines: 1,
                  maxLines: 4,
                  decoration: InputDecoration(
                    labelText: context.strings.memberPatterns,
                    hintText: 'free:workbuddy/*',
                    errorText: patternError,
                    border: const OutlineInputBorder(),
                  ),
                ),
              ),
              SwitchListTile(
                contentPadding: EdgeInsets.zero,
                title: Text(context.strings.listGroupModels),
                value: listMembers,
                onChanged: (value) => setState(() => listMembers = value),
              ),
              const SizedBox(height: 14),
              DropdownButtonFormField<GroupStrategy>(
                icon: const Icon(CrabforgeSymbols.arrowDropDown),
                decoration: InputDecoration(
                  labelText: context.strings.routingStrategy,
                ),
                initialValue: strategy,
                items: [
                  DropdownMenuItem(
                    value: GroupStrategy.order,
                    child: Text(context.strings.listedOrder),
                  ),
                  DropdownMenuItem(
                    value: GroupStrategy.rotate,
                    child: Text(context.strings.rotateMembers),
                  ),
                  DropdownMenuItem(
                    value: GroupStrategy.weight,
                    child: Text(context.strings.weightedOrder),
                  ),
                  DropdownMenuItem(
                    value: GroupStrategy.manual,
                    child: Text(context.strings.manualRouting),
                  ),
                ],
                onChanged: (v) {
                  if (v != null) setState(() => strategy = v);
                },
              ),
              const SizedBox(height: 16),
              EditorCollection(
                count: members.length,
                embedded: true,
                columnWidth: 236,
                tileHeight: strategy == GroupStrategy.weight ? 200 : 128,
                itemBuilder: (context, index) {
                  final m = members[index];
                  return CollectionTile(
                    key: ValueKey(m),
                    grid: grid,
                    title: Text(
                      m,
                      maxLines: grid ? 2 : null,
                      overflow: grid ? TextOverflow.ellipsis : null,
                    ),
                    detail: strategy != GroupStrategy.weight
                        ? null
                        : TextFormField(
                            controller: weightInputs.putIfAbsent(
                              m,
                              () => TextEditingController(
                                text: (weights[m] ?? 1).toString(),
                              ),
                            ),
                            keyboardType: TextInputType.number,
                            decoration: InputDecoration(
                              labelText: context.strings.memberWeight,
                            ),
                            autovalidateMode:
                                AutovalidateMode.onUserInteraction,
                            validator: (text) => validWeight(text)
                                ? null
                                : context.strings.invalidWeight,
                            onChanged: (text) => setState(
                              () => weights[m] = validWeight(text)
                                  ? int.parse(text)
                                  : 0,
                            ),
                          ),
                    leading: strategy == GroupStrategy.manual
                        ? IconButton(
                            isSelected: selected == m,
                            tooltip: context.strings.selectMember,
                            icon: const Icon(
                              CrabforgeSymbols.radioButtonUnchecked,
                            ),
                            selectedIcon: const Icon(
                              CrabforgeSymbols.radioButtonChecked,
                            ),
                            onPressed: () => setState(() => pick = m),
                          )
                        : Text('${index + 1}'),
                    actions: [
                      if (strategy != GroupStrategy.manual)
                        IconButton(
                          tooltip: off.contains(m)
                              ? context.strings.enableMember
                              : context.strings.disableMember,
                          icon: Icon(
                            off.contains(m)
                                ? CrabforgeSymbols.toggleOff
                                : CrabforgeSymbols.toggleOn,
                          ),
                          onPressed: () => setState(() {
                            if (!off.remove(m)) off.add(m);
                          }),
                        ),
                      IconButton(
                        tooltip: context.strings.raisePriority,
                        onPressed: members.first == m
                            ? null
                            : () {
                                setState(() {
                                  final reordered = [...members]
                                    ..removeAt(index)
                                    ..insert(index - 1, m);
                                  if (rules.isNotEmpty) {
                                    placed = reordered;
                                  } else {
                                    named
                                      ..clear()
                                      ..addAll(reordered);
                                    placed = [];
                                  }
                                });
                              },
                        icon: const Icon(CrabforgeSymbols.arrowUpward),
                      ),
                      IconButton(
                        tooltip: context.strings.removeMember,
                        onPressed: !named.contains(m)
                            ? null
                            : () => setState(() {
                                named.remove(m);
                                off.remove(m);
                              }),
                        icon: const Icon(CrabforgeSymbols.removeCircle),
                      ),
                    ],
                  );
                },
              ),
              DropdownMenu<String>(
                trailingIcon: const Icon(CrabforgeSymbols.arrowDropDown),
                selectedTrailingIcon: const Icon(CrabforgeSymbols.arrowDropUp),
                key: ValueKey(('choices', members.join('\n'))),
                label: Text(context.strings.addCandidate),
                expandedInsets: EdgeInsets.zero,
                menuHeight: 220,
                dropdownMenuEntries: [
                  for (final m in widget.choices.where(
                    (m) => !memberSet.contains(m),
                  ))
                    DropdownMenuEntry(value: m, label: m),
                ],
                onSelected: (v) {
                  if (v != null) {
                    setState(() => named.add(v));
                  }
                },
              ),
            ],
          ),
        ),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.pop(context),
          child: Text(context.strings.cancel),
        ),
        FilledButton(
          onPressed:
              matching ||
                  patternError != null ||
                  (members.isEmpty && rules.isEmpty) ||
                  (rules.isEmpty &&
                      strategy != GroupStrategy.manual &&
                      members.every(off.contains)) ||
                  (strategy == GroupStrategy.weight &&
                      members.any((m) => weights[m] == 0))
              ? null
              : () {
                  final field = firstContent.currentState!;
                  final wait = patience.currentState!;
                  if (!field.validate() || !wait.validate()) return;
                  Navigator.pop<GroupSelection>(context, (
                    name: name.text.trim(),
                    note: note.text,
                    firstContent: int.parse(field.value!.trim()),
                    patience: int.parse(wait.value!.trim()),
                    listMembers: listMembers,
                    off: members.where(off.contains).toList(),
                    members: members.where(named.contains).toList(),
                    patterns: rules,
                    placed: rules.isEmpty ? [] : placed,
                    strategy: strategy,
                    pick:
                        strategy == GroupStrategy.manual ||
                            memberSet.contains(pick)
                        ? selected
                        : null,
                    weights: {for (final m in members) m: weights[m] ?? 1},
                  ));
                },
          child: Text(context.strings.save),
        ),
      ],
    );
  }

  bool validWeight(String? text) {
    final value = int.tryParse(text ?? '');
    return value != null && value > 0 && value <= 1000;
  }
}

class _SecondsField extends StatelessWidget {
  const _SecondsField({
    required this.fieldKey,
    required this.seconds,
    required this.label,
    required this.invalid,
    required this.maximum,
    required this.hint,
  });

  final GlobalKey<FormFieldState<String>> fieldKey;
  final int seconds, maximum;
  final String label, invalid, hint;

  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.only(top: 14),
    child: TextFormField(
      key: fieldKey,
      initialValue: '$seconds',
      keyboardType: const TextInputType.numberWithOptions(signed: true),
      autovalidateMode: AutovalidateMode.onUserInteraction,
      decoration: InputDecoration(
        labelText: label,
        helperText: hint,
        helperMaxLines: 3,
        border: const OutlineInputBorder(),
      ),
      validator: (text) {
        final value = int.tryParse(text?.trim() ?? '', radix: 10);
        return value == null || value < -1 || value > maximum ? invalid : null;
      },
    ),
  );
}

class NetworkEditor extends StatefulWidget {
  const NetworkEditor({super.key, required this.config});
  final JsonMap config;
  @override
  State<NetworkEditor> createState() => _NetworkEditorState();
}

class TunnelEditor extends StatefulWidget {
  const TunnelEditor({super.key});
  @override
  State<TunnelEditor> createState() => _TunnelEditorState();
}

class _TunnelEditorState extends State<TunnelEditor> {
  final remote = TextEditingController(),
      relay = TextEditingController(),
      local = TextEditingController(text: '127.0.0.1:3426');
  @override
  void dispose() {
    remote.dispose();
    relay.dispose();
    local.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => AlertDialog(
    title: Text(context.strings.connectRemote),
    content: SizedBox(
      width: 480,
      child: SingleChildScrollView(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            field(context, remote, context.strings.remoteQuic),
            field(context, relay, context.strings.relayFallback),
            field(context, local, context.strings.localTunnel),
            Padding(
              padding: const EdgeInsets.only(top: 16),
              child: Text(context.strings.tunnelHelp),
            ),
          ],
        ),
      ),
    ),
    actions: [
      TextButton(
        onPressed: () => Navigator.pop(context),
        child: Text(context.strings.cancel),
      ),
      FilledButton(
        onPressed: () => Navigator.pop(context, (
          remote.text.trim(),
          local.text.trim(),
          relay.text.trim(),
        )),
        child: Text(context.strings.connect),
      ),
    ],
  );
}

class _NetworkEditorState extends State<NetworkEditor> {
  final firstContent = GlobalKey<FormFieldState<String>>();
  final patience = GlobalKey<FormFieldState<String>>();
  late final origins = TextEditingController(
    text: (widget.config['cors_origins'] as List? ?? []).join('\n'),
  );
  late final listen = TextEditingController(text: widget.config['listen']);
  late final peer = TextEditingController(text: widget.config['peer_listen']);
  late final allowed = TextEditingController(
    text: (widget.config['allowed_peers'] as List).join('\n'),
  );
  @override
  void dispose() {
    listen.dispose();
    peer.dispose();
    allowed.dispose();
    origins.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => AlertDialog(
    title: Text(context.strings.networkSettings),
    content: SizedBox(
      width: 500,
      child: SingleChildScrollView(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            field(context, listen, context.strings.loopback),
            field(
              context,
              peer,
              context.strings.peerListen,
              hint: '/ip4/0.0.0.0/udp/3427/quic-v1',
            ),
            field(context, allowed, context.strings.allowedPeers, lines: 3),
            field(
              context,
              origins,
              context.strings.browserOrigins,
              hint: 'https://app.example.com',
              lines: 3,
            ),
            _SecondsField(
              fieldKey: firstContent,
              seconds: widget.config['default_group_first_content'] ?? 0,
              label: context.strings.firstContentSeconds,
              invalid: context.strings.invalidFirstContent,
              maximum: 2147483647,
              hint: context.strings.defaultFirstContentHelp,
            ),
            _SecondsField(
              fieldKey: patience,
              seconds: widget.config['default_group_patience'] ?? -1,
              label: context.strings.groupPatienceSeconds,
              invalid: context.strings.invalidGroupPatience,
              maximum: 240,
              hint: context.strings.defaultGroupPatienceHelp,
            ),
          ],
        ),
      ),
    ),
    actions: [
      TextButton(
        onPressed: () => Navigator.pop(context),
        child: Text(context.strings.cancel),
      ),
      FilledButton(
        onPressed: () {
          final field = firstContent.currentState!;
          final wait = patience.currentState!;
          if (!field.validate() || !wait.validate()) return;
          Navigator.pop(context, <String, dynamic>{
            'listen': listen.text.trim(),
            'peer_listen': peer.text.trim(),
            'allowed_peers': linesOf(allowed.text),
            'cors_origins': linesOf(origins.text),
            'default_group_first_content': int.parse(field.value!.trim()),
            'default_group_patience': int.parse(wait.value!.trim()),
          });
        },
        child: Text(context.strings.save),
      ),
    ],
  );
}
