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
      Icon(
        CrabforgeSymbols.hub,
        size: 28,
        color: Theme.of(context).colorScheme.primary,
      ),
      const SizedBox(width: 10),
      Text('Crabforge', style: Theme.of(context).textTheme.titleLarge),
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
      padding: const EdgeInsets.only(bottom: 22),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          LayoutBuilder(
            builder: (context, bounds) {
              final heading = Semantics(
                header: true,
                child: Text(title, style: theme.textTheme.headlineLarge),
              );
              if (actions.length > 1 &&
                  (bounds.maxWidth < 440 ||
                      MediaQuery.textScalerOf(context).scale(16) >= 22)) {
                return Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    heading,
                    const SizedBox(height: 8),
                    Wrap(
                      crossAxisAlignment: WrapCrossAlignment.center,
                      children: actions,
                    ),
                  ],
                );
              }
              return Row(
                children: [
                  Expanded(child: heading),
                  ...actions,
                ],
              );
            },
          ),
          if (detail case final String detail)
            Padding(
              padding: const EdgeInsets.only(top: 5),
              child: Text(
                detail,
                style: theme.textTheme.bodyMedium?.copyWith(
                  color: theme.colorScheme.onSurfaceVariant,
                ),
              ),
            ),
        ],
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
        transitionBuilder: (child, animation) => SlideTransition(
          position: Tween(
            begin: const Offset(0, 0.018),
            end: Offset.zero,
          ).animate(animation),
          child: child,
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
    final statusColor = running
        ? theme.colorScheme.tertiary
        : theme.colorScheme.onSurfaceVariant;
    return TextButton.icon(
      onPressed: onPressed,
      icon: Icon(
        CrabforgeSymbols.circle,
        fill: running ? 1 : 0,
        size: 12,
        color: statusColor,
      ),
      label: Text(label, style: TextStyle(color: statusColor)),
    );
  }
}
