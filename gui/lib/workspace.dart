import 'package:flutter/material.dart';

import 'material_symbols.dart';

PopupMenuItem<T> workspaceMenuChoice<T>({
  required T value,
  required bool selected,
  required Widget label,
}) => PopupMenuItem<T>(
  value: value,
  child: Semantics(
    selected: selected,
    child: Row(
      children: [
        SizedBox.square(
          dimension: 24,
          child: selected ? const Icon(CrabforgeSymbols.check, size: 20) : null,
        ),
        const SizedBox(width: 12),
        Flexible(child: label),
      ],
    ),
  ),
);

class WorkspaceBrand extends StatelessWidget {
  const WorkspaceBrand({super.key});

  @override
  Widget build(BuildContext context) => Row(
    mainAxisSize: MainAxisSize.min,
    children: [
      Image.asset(
        'crabforge.png',
        package: 'crabforge_assets',
        scale: 1,
        width: 28,
        height: 28,
        cacheWidth: (28 * MediaQuery.devicePixelRatioOf(context)).ceil(),
        excludeFromSemantics: true,
      ),
      const SizedBox(width: 10),
      Flexible(
        child: Text(
          'Crabforge',
          maxLines: 1,
          overflow: TextOverflow.ellipsis,
          style: Theme.of(context).textTheme.titleLarge
              ?.copyWith(fontWeight: FontWeight.w600),
        ),
      ),
    ],
  );
}

class WorkspaceHeader extends StatelessWidget {
  const WorkspaceHeader({
    super.key,
    required this.title,
    this.detail,
    this.actions = const [],
  });

  final String title;
  final String? detail;
  final List<Widget> actions;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return Padding(
      padding: const EdgeInsets.only(bottom: 24),
      child: LayoutBuilder(
        builder: (context, bounds) {
          final typography = theme.textTheme;
          final heading = Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Semantics(
                header: true,
                child: Text(
                  title,
                  style:
                      (bounds.maxWidth >= 600
                              ? typography.displaySmall
                              : typography.headlineLarge)
                          ?.copyWith(fontWeight: FontWeight.w600),
                ),
              ),
              if (detail case final String detail)
                Padding(
                  padding: const EdgeInsets.only(top: 6),
                  child: Text(
                    detail,
                    style: typography.bodyMedium?.copyWith(
                      color: theme.colorScheme.onSurfaceVariant,
                    ),
                  ),
                ),
            ],
          );
          if (actions.isEmpty) return heading;
          final controls = Wrap(
            spacing: 8,
            runSpacing: 8,
            crossAxisAlignment: WrapCrossAlignment.center,
            children: actions,
          );
          if (actions.length > 1 &&
              (bounds.maxWidth < 540 ||
                  MediaQuery.textScalerOf(context).scale(16) >= 22)) {
            return Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [heading, const SizedBox(height: 12), controls],
            );
          }
          return Row(
            children: [
              Expanded(child: heading),
              const SizedBox(width: 24),
              controls,
            ],
          );
        },
      ),
    );
  }
}

class WorkspacePageTransition extends StatelessWidget {
  const WorkspacePageTransition({
    super.key,
    required this.page,
    required this.child,
  });

  final int page;
  final Widget child;

  @override
  Widget build(BuildContext context) {
    final reducedMotion = MediaQuery.disableAnimationsOf(context);
    return ClipRect(
      child: AnimatedSwitcher(
        duration: reducedMotion
            ? Duration.zero
            : const Duration(milliseconds: 200),
        switchInCurve: Curves.easeOutCubic,
        layoutBuilder: (current, _) => current ?? const SizedBox.shrink(),
        transitionBuilder: (child, animation) => FadeTransition(
          opacity: animation,
          child: SlideTransition(
            position: Tween(
              begin: const Offset(0, 0.018),
              end: Offset.zero,
            ).animate(animation),
            child: child,
          ),
        ),
        child: KeyedSubtree(
          key: ValueKey(page),
          child: RepaintBoundary(child: child),
        ),
      ),
    );
  }
}

class WorkspaceStatus extends StatelessWidget {
  const WorkspaceStatus({
    super.key,
    required this.running,
    required this.label,
    required this.onPressed,
  });

  final bool running;
  final String label;
  final VoidCallback onPressed;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final colors = theme.colorScheme;
    return TextButton.icon(
      onPressed: onPressed,
      style: TextButton.styleFrom(
        backgroundColor: running
            ? colors.tertiaryContainer
            : colors.surfaceContainerHigh,
        foregroundColor: running
            ? colors.onTertiaryContainer
            : colors.onSurfaceVariant,
      ),
      icon: Icon(CrabforgeSymbols.circle, fill: running ? 1 : 0, size: 12),
      label: Text(label),
    );
  }
}
