import 'package:flutter/material.dart';
import '../theme/nostring_theme.dart';

/// Text with the Bitcoin Butlers gold gradient shader.
class GoldGradientText extends StatelessWidget {
  final String text;
  final double fontSize;
  final FontWeight fontWeight;
  final TextAlign textAlign;

  const GoldGradientText(
    this.text, {
    super.key,
    this.fontSize = 24,
    this.fontWeight = FontWeight.bold,
    this.textAlign = TextAlign.start,
  });

  @override
  Widget build(BuildContext context) {
    return ShaderMask(
      shaderCallback: (bounds) =>
          NoStringColors.goldGradient.createShader(bounds),
      child: Text(
        text,
        textAlign: textAlign,
        style: TextStyle(
          fontSize: fontSize,
          fontWeight: fontWeight,
          color: Colors.white,
        ),
      ),
    );
  }
}
