import 'package:flutter/material.dart';

ThemeData workspaceTheme(Brightness brightness) {
  final dark = brightness == Brightness.dark;
  final colors =
      ColorScheme.fromSeed(
        seedColor: const Color(0xffb45032),
        brightness: brightness,
        dynamicSchemeVariant: DynamicSchemeVariant.expressive,
      ).copyWith(
        primary: dark ? const Color(0xffffb59a) : const Color(0xff963c24),
        onPrimary: dark ? const Color(0xff502010) : Colors.white,
        primaryContainer: dark
            ? const Color(0xff532d23)
            : const Color(0xfff8ded3),
        onPrimaryContainer: dark
            ? const Color(0xffffdacc)
            : const Color(0xff602416),
        surface: dark ? const Color(0xff171819) : const Color(0xfffaf9f6),
        onSurface: dark ? const Color(0xfff1eeee) : const Color(0xff232121),
        onSurfaceVariant: dark
            ? const Color(0xffbbb5b1)
            : const Color(0xff655c57),
        surfaceContainerLowest: dark ? const Color(0xff141516) : Colors.white,
        surfaceContainerLow: dark
            ? const Color(0xff1e2021)
            : const Color(0xfff2f0ec),
        surfaceContainer: dark
            ? const Color(0xff242627)
            : const Color(0xffeeebe6),
        surfaceContainerHigh: dark
            ? const Color(0xff2b2d2e)
            : const Color(0xffe9e5df),
        outlineVariant: dark
            ? const Color(0xff393b3c)
            : const Color(0xffded9d2),
      );
  final border = OutlineInputBorder(
    borderRadius: BorderRadius.circular(12),
    borderSide: BorderSide(color: colors.outlineVariant),
  );
  final base = ThemeData(
    useMaterial3: true,
    fontFamily: 'packages/crabforge_assets/Google Sans',
    fontFamilyFallback: const [
      'packages/crabforge_assets/HarmonyOS Sans SC',
      'packages/crabforge_assets/Noto Sans KR',
    ],
    colorScheme: colors,
    scaffoldBackgroundColor: colors.surface,
    splashFactory: InkRipple.splashFactory,
    appBarTheme: AppBarThemeData(
      backgroundColor: colors.surface,
      scrolledUnderElevation: 0,
    ),
    cardTheme: CardThemeData(
      margin: EdgeInsets.zero,
      elevation: 0,
      color: colors.surfaceContainerLow,
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(16)),
    ),
    inputDecorationTheme: InputDecorationThemeData(
      filled: true,
      fillColor: colors.surfaceContainerLowest,
      border: border,
      enabledBorder: border,
      focusedBorder: border.copyWith(
        borderSide: BorderSide(color: colors.primary, width: 2),
      ),
    ),
    navigationRailTheme: NavigationRailThemeData(
      groupAlignment: -1,
      indicatorColor: colors.primaryContainer,
      indicatorShape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(12),
      ),
      selectedIconTheme: IconThemeData(
        color: colors.onPrimaryContainer,
        size: 21,
      ),
      unselectedIconTheme: IconThemeData(
        color: colors.onSurfaceVariant,
        size: 21,
      ),
      selectedLabelTextStyle: TextStyle(
        color: colors.onPrimaryContainer,
        fontWeight: FontWeight.w700,
      ),
      unselectedLabelTextStyle: TextStyle(
        color: colors.onSurfaceVariant,
        fontWeight: FontWeight.w500,
      ),
    ),
    navigationBarTheme: NavigationBarThemeData(
      height: 68,
      backgroundColor: colors.surfaceContainerLow,
      indicatorColor: colors.primaryContainer,
      indicatorShape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(12),
      ),
      labelTextStyle: const WidgetStatePropertyAll(
        TextStyle(fontSize: 11, fontWeight: FontWeight.w600),
      ),
    ),
    tooltipTheme: TooltipThemeData(
      waitDuration: const Duration(milliseconds: 400),
      preferBelow: false,
      decoration: BoxDecoration(
        color: colors.inverseSurface,
        borderRadius: BorderRadius.circular(8),
      ),
      textStyle: TextStyle(color: colors.onInverseSurface, fontSize: 12),
    ),
  );
  return base.copyWith(
    textTheme: base.textTheme.copyWith(
      headlineLarge: base.textTheme.headlineLarge?.copyWith(
        fontWeight: FontWeight.w700,
        letterSpacing: -1.15,
      ),
      titleMedium: base.textTheme.titleMedium?.copyWith(
        fontWeight: FontWeight.w600,
      ),
    ),
  );
}

class WorkspaceBrand extends StatelessWidget {
  const WorkspaceBrand({super.key});

  @override
  Widget build(BuildContext context) => Row(
    mainAxisSize: MainAxisSize.min,
    children: [
      Image.asset(
        'crabforge.png',
        package: 'crabforge_assets',
        width: 32,
        height: 32,
        cacheWidth: (32 * MediaQuery.devicePixelRatioOf(context)).ceil(),
        excludeFromSemantics: true,
      ),
      const SizedBox(width: 10),
      Text(
        'Crabforge',
        style: Theme.of(context).textTheme.titleLarge
            ?.copyWith(fontWeight: FontWeight.w700, letterSpacing: -0.7),
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
        ? theme.brightness == Brightness.dark
              ? const Color(0xff88d4aa)
              : const Color(0xff276747)
        : theme.colorScheme.onSurfaceVariant;
    return TextButton.icon(
      onPressed: onPressed,
      icon: Icon(
        running ? Icons.circle : Icons.circle_outlined,
        size: 9,
        color: statusColor,
      ),
      label: Text(label, style: TextStyle(color: statusColor)),
    );
  }
}
