import 'package:flutter/material.dart';

import 'language.dart';

class CollectionLayout extends InheritedWidget {
  const CollectionLayout({
    super.key,
    required this.grid,
    required this.toggle,
    required super.child,
  });
  final bool grid;
  final VoidCallback? toggle;

  static CollectionLayout of(BuildContext context) =>
      context.dependOnInheritedWidgetOfExactType<CollectionLayout>()!;

  @override
  bool updateShouldNotify(CollectionLayout oldWidget) =>
      grid != oldWidget.grid || toggle != oldWidget.toggle;
}

class EditorHeading extends StatelessWidget {
  const EditorHeading(this.title, {super.key});
  final String title;

  @override
  Widget build(BuildContext context) {
    final layout = CollectionLayout.of(context);
    return Row(
      children: [
        Expanded(child: Text(title)),
        LayoutToggle(grid: layout.grid, onPressed: layout.toggle),
      ],
    );
  }
}

/// Lists embedded in an editor can share its scroll view or scroll independently.
class EditorCollection extends StatelessWidget {
  const EditorCollection({
    super.key,
    required this.count,
    required this.itemBuilder,
    this.embedded = false,
    this.columnWidth = 208,
    this.tileHeight = 104,
  });
  final int count;
  final IndexedWidgetBuilder itemBuilder;
  final bool embedded;
  final double columnWidth, tileHeight;

  @override
  Widget build(BuildContext context) {
    final physics = embedded ? const NeverScrollableScrollPhysics() : null;
    if (!CollectionLayout.of(context).grid) {
      return ListView.builder(
        primary: false,
        shrinkWrap: true,
        padding: EdgeInsets.zero,
        physics: physics,
        itemCount: count,
        itemBuilder: itemBuilder,
      );
    }
    return LayoutBuilder(
      builder: (context, bounds) => GridView.builder(
        primary: false,
        shrinkWrap: true,
        padding: EdgeInsets.zero,
        physics: physics,
        itemCount: count,
        gridDelegate: SliverGridDelegateWithFixedCrossAxisCount(
          crossAxisCount: (bounds.maxWidth / columnWidth).floor().clamp(1, 4),
          mainAxisExtent:
              tileHeight + MediaQuery.textScalerOf(context).scale(32),
          mainAxisSpacing: 4,
          crossAxisSpacing: 4,
        ),
        itemBuilder: itemBuilder,
      ),
    );
  }
}

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
  }) : itemCount = null,
       itemBuilder = null;

  const CollectionSliver.builder({
    super.key,
    required this.grid,
    required this.itemCount,
    required this.itemBuilder,
    this.minWidth = 290,
    required this.height,
  }) : children = null;

  final bool grid;
  final List<Widget>? children;
  final int? itemCount;
  final IndexedWidgetBuilder? itemBuilder;
  final double minWidth, height;

  @override
  Widget build(BuildContext context) {
    final data = children;
    final count = data?.length ?? itemCount!;
    final builder = itemBuilder ?? (_, index) => data![index];
    if (!grid) {
      return SliverList.separated(
        itemCount: count,
        itemBuilder: builder,
        separatorBuilder: (_, _) => const SizedBox(height: 6),
      );
    }
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
            crossAxisSpacing: 12,
            mainAxisSpacing: 12,
          ),
          delegate: SliverChildBuilderDelegate(builder, childCount: count),
        );
      },
    );
  }
}

class CollectionGlyph extends StatelessWidget {
  const CollectionGlyph(this.child, {super.key});

  final Widget child;

  @override
  Widget build(BuildContext context) {
    final colors = Theme.of(context).colorScheme;
    return DecoratedBox(
      decoration: BoxDecoration(
        color: colors.surfaceContainer,
        borderRadius: BorderRadius.circular(10),
      ),
      child: SizedBox.square(dimension: 40, child: Center(child: child)),
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
  Widget build(BuildContext context) => Card.outlined(
    clipBehavior: Clip.hardEdge,
    color: Theme.of(context).colorScheme.surfaceContainerLowest,
    shape: RoundedRectangleBorder(
      borderRadius: BorderRadius.circular(14),
      side: BorderSide(color: Theme.of(context).colorScheme.outlineVariant),
    ),
    child: LayoutBuilder(
      builder: (context, bounds) {
        if (!grid) {
          final stacked =
              actions.isNotEmpty && bounds.maxWidth < 260 + actions.length * 48;
          return ListTile(
            contentPadding: const EdgeInsets.symmetric(
              horizontal: 14,
              vertical: 6,
            ),
            leading: leading == null ? null : CollectionGlyph(leading!),
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
                      CollectionGlyph(leading!),
                      const SizedBox(width: 12),
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
                    child: Padding(
                      padding: const EdgeInsets.only(top: 8),
                      child: SingleChildScrollView(
                        primary: false,
                        child: Align(
                          alignment: AlignmentDirectional.topStart,
                          child: detail,
                        ),
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
