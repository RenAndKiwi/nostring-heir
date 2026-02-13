import 'package:flutter/material.dart';
import 'package:mobile_scanner/mobile_scanner.dart';
import '../theme/nostring_theme.dart';
import '../widgets/gold_gradient_text.dart';

class ScanScreen extends StatefulWidget {
  final void Function(String payload) onScanned;
  final VoidCallback onManualEntry;

  const ScanScreen({
    super.key,
    required this.onScanned,
    required this.onManualEntry,
  });

  @override
  State<ScanScreen> createState() => _ScanScreenState();
}

class _ScanScreenState extends State<ScanScreen> {
  final MobileScannerController _controller = MobileScannerController(
    detectionSpeed: DetectionSpeed.normal,
    facing: CameraFacing.back,
  );
  bool _scanned = false;
  bool _cameraError = false;
  String? _errorMessage;

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  void _onDetect(BarcodeCapture capture) {
    if (_scanned) return;
    final barcode = capture.barcodes.firstOrNull;
    if (barcode == null || barcode.rawValue == null) return;

    final value = barcode.rawValue!;
    // Accept nostring: URI or raw JSON
    if (value.startsWith('nostring:') || value.trimLeft().startsWith('{')) {
      setState(() => _scanned = true);
      _controller.stop();
      widget.onScanned(value);
    }
  }

  Widget _buildCameraError() {
    return Container(
      margin: const EdgeInsets.symmetric(horizontal: NoStringSpacing.lg),
      padding: const EdgeInsets.all(NoStringSpacing.xl),
      decoration: BoxDecoration(
        color: NoStringColors.surface,
        border: Border.all(color: NoStringColors.border),
        borderRadius: BorderRadius.circular(12),
      ),
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Icon(Icons.no_photography, size: 48, color: NoStringColors.textMuted),
          const SizedBox(height: NoStringSpacing.md),
          Text(
            _errorMessage ?? 'Camera not available',
            style: TextStyle(color: NoStringColors.textMuted),
            textAlign: TextAlign.center,
          ),
          const SizedBox(height: NoStringSpacing.sm),
          Text(
            'Grant camera permission in Settings, or use manual paste below.',
            style: TextStyle(color: NoStringColors.textMuted, fontSize: 12),
            textAlign: TextAlign.center,
          ),
        ],
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: NoStringColors.background,
      body: SafeArea(
        child: Column(
          children: [
            Padding(
              padding: const EdgeInsets.all(NoStringSpacing.lg),
              child: Column(
                children: [
                  const GoldGradientText('Scan Vault Backup', fontSize: 24),
                  const SizedBox(height: NoStringSpacing.sm),
                  Text(
                    'Point your camera at the QR code from the owner\'s app.',
                    style: TextStyle(color: NoStringColors.textMuted, fontSize: 14),
                    textAlign: TextAlign.center,
                  ),
                ],
              ),
            ),
            Expanded(
              child: Padding(
                padding: const EdgeInsets.symmetric(horizontal: NoStringSpacing.lg),
                child: _cameraError
                  ? _buildCameraError()
                  : ClipRRect(
                  borderRadius: BorderRadius.circular(12),
                  child: Stack(
                    alignment: Alignment.center,
                    children: [
                      MobileScanner(
                        controller: _controller,
                        onDetect: _onDetect,
                        errorBuilder: (context, error) {
                          WidgetsBinding.instance.addPostFrameCallback((_) {
                            if (!_cameraError) {
                              setState(() {
                                _cameraError = true;
                                _errorMessage = error.errorDetails?.message ?? 'Camera access denied';
                              });
                            }
                          });
                          return const SizedBox();
                        },
                      ),
                      Container(
                        width: 250,
                        height: 250,
                        decoration: BoxDecoration(
                          border: Border.all(
                            color: NoStringColors.goldLight.withValues(alpha: 0.5),
                            width: 2,
                          ),
                          borderRadius: BorderRadius.circular(12),
                        ),
                      ),
                    ],
                  ),
                ),
              ),
            ),
            Padding(
              padding: const EdgeInsets.all(NoStringSpacing.lg),
              child: Column(
                children: [
                  if (_cameraError)
                    Padding(
                      padding: const EdgeInsets.only(bottom: NoStringSpacing.md),
                      child: SizedBox(
                        width: double.infinity,
                        child: ElevatedButton.icon(
                          onPressed: () {
                            setState(() {
                              _cameraError = false;
                              _errorMessage = null;
                            });
                            _controller.start();
                          },
                          icon: const Icon(Icons.refresh),
                          label: const Text('Retry Camera'),
                        ),
                      ),
                    ),
                  SizedBox(
                    width: double.infinity,
                    child: OutlinedButton.icon(
                      onPressed: widget.onManualEntry,
                      icon: const Icon(Icons.edit_note),
                      label: const Text('Paste JSON Manually'),
                      style: OutlinedButton.styleFrom(
                        foregroundColor: NoStringColors.textMuted,
                        side: BorderSide(color: NoStringColors.border),
                        padding: const EdgeInsets.symmetric(vertical: NoStringSpacing.md),
                      ),
                    ),
                  ),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }
}
