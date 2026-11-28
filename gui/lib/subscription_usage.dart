import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:intl/intl.dart';

import 'collection_view.dart';
import 'editors.dart' show JsonMap;
import 'language.dart';

class SubscriptionUsageDialog extends StatefulWidget {
  const SubscriptionUsageDialog({
    super.key,
    required this.provider,
    required this.read,
  });

  final String provider;
  final Future<String> Function() read;

  @override
  State<SubscriptionUsageDialog> createState() =>
      _SubscriptionUsageDialogState();
}

class _SubscriptionUsageDialogState extends State<SubscriptionUsageDialog> {
  late final Future<JsonMap> report = widget.read().then(
    (text) => jsonDecode(text) as JsonMap,
  );

  @override
  Widget build(BuildContext context) => Dialog(
    child: SizedBox(
      width: 600,
      height: 420,
      child: Padding(
        padding: const EdgeInsets.all(20),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            DefaultTextStyle(
              style: Theme.of(context).textTheme.titleLarge!,
              child: EditorHeading(context.strings.subscriptionUsage),
            ),
            const SizedBox(height: 12),
            Expanded(
              child: FutureBuilder<JsonMap>(
                future: report,
                builder: (context, snapshot) {
                  if (snapshot.hasError) {
                    return SingleChildScrollView(
                      child: SelectableText('${snapshot.error}'),
                    );
                  }
                  final data = snapshot.data;
                  if (data == null) {
                    return const Center(child: CircularProgressIndicator());
                  }
                  final windows = (data['windows'] as List).cast<JsonMap>();
                  final plan = data['plan'] as String?;
                  return Column(
                    crossAxisAlignment: CrossAxisAlignment.stretch,
                    children: [
                      Text([widget.provider, ?plan].join(' · ')),
                      const SizedBox(height: 12),
                      if (windows.isEmpty)
                        Text(context.strings.noSubscriptionUsage)
                      else
                        Expanded(
                          child: EditorCollection(
                            count: windows.length,
                            tileHeight: 120,
                            itemBuilder: (_, index) =>
                                _QuotaWindow(windows[index]),
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
                child: Text(MaterialLocalizations.of(context).closeButtonLabel),
              ),
            ),
          ],
        ),
      ),
    ),
  );
}

class _QuotaWindow extends StatelessWidget {
  const _QuotaWindow(this.data);

  final JsonMap data;

  String title(BuildContext context, NumberFormat decimal) {
    final parts = (data['name'] as String).split(' · ');
    final kind = switch (parts.last) {
      'Allowance' => 'allowance',
      'Month' => 'month',
      'Session limit' => 'session',
      'Extra usage' => 'extra',
      'Monthly credits' => 'monthly',
      'Purchased credits' => 'purchased',
      'Free credits' => 'free',
      _ => null,
    };
    final duration = data['duration_seconds'] as int?;
    final name = switch ((kind, duration)) {
      (final kind?, _) => context.strings.quotaWindow(kind),
      (_, final seconds?) => switch (seconds) {
        _ when seconds % 86400 == 0 => context.strings.quotaDuration(
          'days',
          decimal.format(seconds ~/ 86400),
        ),
        _ when seconds % 3600 == 0 => context.strings.quotaDuration(
          'hours',
          decimal.format(seconds ~/ 3600),
        ),
        _ => context.strings.quotaDuration(
          'minutes',
          decimal.format(seconds ~/ 60),
        ),
      },
      _ => parts.last,
    };
    parts[parts.length - 1] = name;
    return parts.join(' · ');
  }

  @override
  Widget build(BuildContext context) {
    final locale = Localizations.localeOf(context).toLanguageTag();
    final decimal = NumberFormat.decimalPattern(locale);
    final percent = (data['used_percent'] as num?)?.toDouble();
    final ratio = percent == null ? null : percent / 100;
    final used = data['used'] as num?;
    final limit = data['limit'] as num?;
    final balance = data['balance'] as num?;
    final unit = switch (data['unit'] as String?) {
      'credits' => context.strings.quotaWindow('credits'),
      'videos' => context.strings.quotaWindow('videos'),
      final unit => unit,
    };
    final time = DateTime.tryParse(data['resets_at'] as String? ?? '')
        ?.toLocal();
    String amount(String text) => [text, ?unit].join(' ');
    return CollectionTile(
      grid: CollectionLayout.of(context).grid,
      title: Text(title(context, decimal)),
      detail: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          if (ratio != null) ...[
            Text(
              context.strings.quotaUsed(
                NumberFormat.percentPattern(locale).format(ratio),
              ),
            ),
            const SizedBox(height: 6),
            LinearProgressIndicator(value: ratio),
          ],
          if (used != null && limit != null)
            Text(amount('${decimal.format(used)} / ${decimal.format(limit)}')),
          if (balance != null)
            Text(context.strings.quotaBalance(amount(decimal.format(balance)))),
          if (time != null)
            Text(
              context.strings.quotaReset(
                DateFormat.yMMMd(locale).add_jm().format(time),
              ),
            ),
          if (data['aside'] == true && balance == null)
            Text(context.strings.quotaSeparate),
        ],
      ),
    );
  }
}
