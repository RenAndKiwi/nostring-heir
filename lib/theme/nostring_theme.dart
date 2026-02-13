import 'package:flutter/material.dart';

/// Bitcoin Butlers brand colors — shared across all NoString apps.
class NoStringColors {
  NoStringColors._();

  // Backgrounds
  static const background = Color(0xFF0A0A0A);
  static const surface = Color(0xFF1A1A2E);
  static const surfaceVariant = Color(0xFF262834);

  // Gold palette
  static const goldLight = Color(0xFFFBDC7B);
  static const gold = Color(0xFFFDA24A);
  static const goldDark = Color(0xFFFF9125);

  // Text
  static const textPrimary = Color(0xFFFFFFFF);
  static const textMuted = Color(0xFF9CA3AF);

  // Semantic
  static const success = Color(0xFF10B981);
  static const warning = Color(0xFFF59E0B);
  static const error = Color(0xFFEF4444);
  static const info = Color(0xFF3B82F6);

  // Borders
  static const border = Color(0xFF333333);
  static const borderFocus = goldLight;

  // Gold gradient
  static const goldGradient = LinearGradient(
    begin: Alignment.topCenter,
    end: Alignment.bottomCenter,
    colors: [Color(0xFFFFD700), Color(0xFFFFC107), Color(0xFFFFA500)],
  );

  static const goldHorizontal = LinearGradient(
    colors: [goldLight, gold],
  );
}

/// Consistent spacing values.
class NoStringSpacing {
  NoStringSpacing._();
  static const double xs = 4;
  static const double sm = 8;
  static const double md = 12;
  static const double lg = 16;
  static const double xl = 24;
  static const double xxl = 32;
}

/// Consistent border radius.
class NoStringRadius {
  NoStringRadius._();
  static final sm = BorderRadius.circular(4);
  static final md = BorderRadius.circular(8);
  static final lg = BorderRadius.circular(12);
  static final xl = BorderRadius.circular(16);
}

/// Build the full app ThemeData.
ThemeData noStringTheme() {
  return ThemeData.dark().copyWith(
    scaffoldBackgroundColor: NoStringColors.background,
    colorScheme: const ColorScheme.dark(
      primary: NoStringColors.goldLight,
      secondary: NoStringColors.gold,
      surface: NoStringColors.surface,
      error: NoStringColors.error,
      onPrimary: Colors.black,
      onSecondary: Colors.black,
      onSurface: NoStringColors.textPrimary,
    ),
    appBarTheme: const AppBarTheme(
      backgroundColor: NoStringColors.surfaceVariant,
      foregroundColor: NoStringColors.goldLight,
      elevation: 0,
      centerTitle: true,
    ),
    cardTheme: CardThemeData(
      color: NoStringColors.surface,
      elevation: 0,
      shape: RoundedRectangleBorder(
        borderRadius: NoStringRadius.md,
        side: const BorderSide(color: NoStringColors.border, width: 1),
      ),
    ),
    elevatedButtonTheme: ElevatedButtonThemeData(
      style: ElevatedButton.styleFrom(
        backgroundColor: NoStringColors.goldLight,
        foregroundColor: Colors.black,
        padding: const EdgeInsets.symmetric(
          vertical: NoStringSpacing.lg,
          horizontal: NoStringSpacing.xl,
        ),
        shape: RoundedRectangleBorder(borderRadius: NoStringRadius.md),
        textStyle: const TextStyle(
          fontSize: 16,
          fontWeight: FontWeight.w600,
        ),
      ),
    ),
    outlinedButtonTheme: OutlinedButtonThemeData(
      style: OutlinedButton.styleFrom(
        foregroundColor: NoStringColors.goldLight,
        side: const BorderSide(color: NoStringColors.border),
        padding: const EdgeInsets.symmetric(
          vertical: NoStringSpacing.lg,
          horizontal: NoStringSpacing.xl,
        ),
        shape: RoundedRectangleBorder(borderRadius: NoStringRadius.md),
      ),
    ),
    inputDecorationTheme: InputDecorationTheme(
      filled: true,
      fillColor: NoStringColors.surfaceVariant,
      border: OutlineInputBorder(
        borderRadius: NoStringRadius.md,
        borderSide: const BorderSide(color: NoStringColors.border),
      ),
      enabledBorder: OutlineInputBorder(
        borderRadius: NoStringRadius.md,
        borderSide: const BorderSide(color: NoStringColors.border),
      ),
      focusedBorder: OutlineInputBorder(
        borderRadius: NoStringRadius.md,
        borderSide: const BorderSide(color: NoStringColors.goldLight),
      ),
      hintStyle: const TextStyle(color: NoStringColors.textMuted),
      labelStyle: const TextStyle(color: NoStringColors.textMuted),
    ),
    sliderTheme: const SliderThemeData(
      activeTrackColor: NoStringColors.goldLight,
      thumbColor: NoStringColors.gold,
      inactiveTrackColor: NoStringColors.border,
      overlayColor: Color(0x29FBDC7B),
    ),
    snackBarTheme: SnackBarThemeData(
      backgroundColor: NoStringColors.surface,
      contentTextStyle: const TextStyle(color: NoStringColors.textPrimary),
      shape: RoundedRectangleBorder(borderRadius: NoStringRadius.md),
      behavior: SnackBarBehavior.floating,
    ),
    dividerColor: NoStringColors.border,
  );
}
