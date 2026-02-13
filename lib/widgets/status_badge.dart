import 'package:flutter/material.dart';
import '../theme/nostring_theme.dart';

enum BadgeType { success, warning, error, info }

/// Colored pill badge for status indicators.
class StatusBadge extends StatelessWidget {
  final String label;
  final BadgeType type;
  final IconData? icon;

  const StatusBadge({
    super.key,
    required this.label,
    required this.type,
    this.icon,
  });

  Color get _color => switch (type) {
        BadgeType.success => NoStringColors.success,
        BadgeType.warning => NoStringColors.warning,
        BadgeType.error => NoStringColors.error,
        BadgeType.info => NoStringColors.info,
      };

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(
        horizontal: NoStringSpacing.md,
        vertical: NoStringSpacing.xs,
      ),
      decoration: BoxDecoration(
        color: _color.withValues(alpha: 0.15),
        borderRadius: NoStringRadius.xl,
        border: Border.all(color: _color.withValues(alpha: 0.3)),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          if (icon != null) ...[
            Icon(icon, size: 14, color: _color),
            const SizedBox(width: NoStringSpacing.xs),
          ],
          Text(
            label,
            style: TextStyle(
              color: _color,
              fontSize: 12,
              fontWeight: FontWeight.w600,
            ),
          ),
        ],
      ),
    );
  }
}
