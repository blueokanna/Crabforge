import 'package:flutter/material.dart';

import 'language.dart';

class LayoutToggle extends StatelessWidget {
  const LayoutToggle({super.key, required this.grid, required this.onPressed});
  final bool grid;
  final VoidCallback? onPressed;

  @override
  Widget build(BuildContext context) => IconButton(
    isSelected: grid,
    tooltip: grid ? context.strings.listView : context.strings.gridView,
    icon: Icon(grid ? Icons.view_list_outlined : Icons.grid_view_outlined),
    onPressed: onPressed,
  );
}

/// A collection in the page's single scrolling viewport, alongside its header and actions.
class CollectionSliver extends StatelessWidget {
  const CollectionSliver({
    super.key,
    required this.grid,
    required this.children,
    this.minWidth = 290,
    required this.height,
  });
  final bool grid;
  final List<Widget> children;
  final double minWidth, height;

  @override
  Widget build(BuildContext context) {
    if (!grid) return SliverList.list(children: children);
    return SliverLayoutBuilder(
      builder: (context, constraints) {
        final columns = (constraints.crossAxisExtent / minWidth).floor().clamp(
          1,
          4,
        );
        return SliverGrid(
          gridDelegate: SliverGridDelegateWithFixedCrossAxisCount(
            crossAxisCount: columns,
            mainAxisExtent: height + MediaQuery.textScalerOf(context).scale(48),
            crossAxisSpacing: 4,
            mainAxisSpacing: 4,
          ),
          delegate: SliverChildListDelegate(children),
        );
      },
    );
  }
}

class CollectionTile extends StatelessWidget {
  const CollectionTile({
    super.key,
    required this.grid,
    required this.title,
    this.leading,
    this.detail,
    this.actions = const [],
    this.onTap,
  });
  final bool grid;
  final Widget title;
  final Widget? leading, detail;
  final List<Widget> actions;
  final VoidCallback? onTap;

  @override
  Widget build(BuildContext context) => Card.filled(
    clipBehavior: Clip.antiAlias,
    child: LayoutBuilder(
      builder: (context, bounds) {
        if (!grid) {
          final stacked =
              actions.isNotEmpty && bounds.maxWidth < 260 + actions.length * 48;
          return ListTile(
            leading: leading,
            title: title,
            onTap: onTap,
            subtitle: stacked
                ? Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      ?detail,
                      Wrap(children: actions),
                    ],
                  )
                : detail,
            trailing: stacked || actions.isEmpty
                ? null
                : Row(mainAxisSize: MainAxisSize.min, children: actions),
          );
        }
        return InkWell(
          onTap: onTap,
          child: Padding(
            padding: const EdgeInsets.fromLTRB(14, 12, 14, 6),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Row(
                  children: [
                    if (leading != null) ...[
                      leading!,
                      const SizedBox(width: 10),
                    ],
                    Expanded(
                      child: DefaultTextStyle.merge(
                        style: Theme.of(context).textTheme.titleMedium,
                        child: title,
                      ),
                    ),
                  ],
                ),
                if (detail != null)
                  Expanded(
                    child: SingleChildScrollView(
                      primary: false,
                      child: Align(
                        alignment: AlignmentDirectional.topStart,
                        child: detail,
                      ),
                    ),
                  ),
                if (actions.isNotEmpty)
                  Align(
                    alignment: AlignmentDirectional.bottomEnd,
                    child: Wrap(children: actions),
                  ),
              ],
            ),
          ),
        );
      },
    ),
  );
}
