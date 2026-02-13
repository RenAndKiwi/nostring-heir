import 'package:flutter/material.dart';
import '../theme/nostring_theme.dart';

/// Section title with optional subtitle.
class SectionHeader extends StatelessWidget {
  final String title;
  final String? subtitle;

  const SectionHeader({
    super.key,
    required this.title,
    this.subtitle,
  });

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          title,
          style: const TextStyle(
            fontSize: 20,
            fontWeight: FontWeight.bold,
            color: NoStringColors.textPrimary,
          ),
        ),
        if (subtitle != null) ...[
          const SizedBox(height: NoStringSpacing.xs),
          Text(
            subtitle!,
            style: const TextStyle(
              color: NoStringColors.textMuted,
              fontSize: 14,
            ),
          ),
        ],
      ],
    );
  }
}
