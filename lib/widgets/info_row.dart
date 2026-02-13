import 'package:flutter/material.dart';
import '../theme/nostring_theme.dart';

/// Label + value row with consistent styling.
class InfoRow extends StatelessWidget {
  final String label;
  final String value;
  final bool mono;

  const InfoRow({
    super.key,
    required this.label,
    required this.value,
    this.mono = false,
  });

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: NoStringSpacing.xs),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SizedBox(
            width: 110,
            child: Text(
              label,
              style: const TextStyle(
                color: NoStringColors.textMuted,
                fontSize: 13,
              ),
            ),
          ),
          Expanded(
            child: Text(
              value,
              style: TextStyle(
                color: NoStringColors.goldLight,
                fontFamily: mono ? 'monospace' : null,
                fontSize: 13,
                fontWeight: FontWeight.w500,
              ),
            ),
          ),
        ],
      ),
    );
  }
}
