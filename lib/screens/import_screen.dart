import 'package:flutter/material.dart';
import '../src/rust/api.dart';
import '../theme/nostring_theme.dart';
import '../widgets/gold_gradient_text.dart';
import '../widgets/info_row.dart';
import '../widgets/status_badge.dart';
import 'status_screen.dart';

class ImportScreen extends StatefulWidget {
  const ImportScreen({super.key});

  @override
  State<ImportScreen> createState() => _ImportScreenState();
}

class _ImportScreenState extends State<ImportScreen> {
  final _controller = TextEditingController();
  bool _loading = false;
  VaultInfo? _vaultInfo;
  String? _error;

  Future<void> _handleImport() async {
    final json = _controller.text.trim();
    if (json.isEmpty) {
      setState(() => _error = 'Please paste a VaultBackup JSON');
      return;
    }

    setState(() {
      _loading = true;
      _error = null;
      _vaultInfo = null;
    });

    try {
      final info = await importVaultBackup(json: json);
      setState(() {
        _vaultInfo = info;
        _loading = false;
      });
    } catch (e) {
      setState(() {
        _error = e.toString();
        _loading = false;
      });
    }
  }

  void _navigateToStatus() {
    final net = _vaultInfo!.network;
    final defaultElectrum = net == 'testnet'
        ? 'ssl://electrum.blockstream.info:60002'
        : net == 'signet'
            ? 'ssl://mempool.space:60602'
            : 'ssl://electrum.blockstream.info:50002';
    Navigator.push(
      context,
      MaterialPageRoute(
        builder: (_) => StatusScreen(
          vaultJson: _controller.text.trim(),
          electrumUrl: defaultElectrum,
          network: net,
        ),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('NoString Heir')),
      body: Padding(
        padding: const EdgeInsets.all(NoStringSpacing.lg),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            const GoldGradientText('Import Vault Backup'),
            const SizedBox(height: NoStringSpacing.sm),
            const Text(
              'Paste the VaultBackup JSON from the vault owner.',
              style: TextStyle(color: NoStringColors.textMuted),
            ),
            const SizedBox(height: NoStringSpacing.lg),
            Expanded(
              child: TextField(
                controller: _controller,
                maxLines: null,
                expands: true,
                textAlignVertical: TextAlignVertical.top,
                style: const TextStyle(
                  fontFamily: 'monospace',
                  fontSize: 12,
                  color: NoStringColors.textPrimary,
                ),
                decoration: const InputDecoration(
                  hintText: '{"version": 1, "network": "testnet", ...}',
                ),
              ),
            ),
            const SizedBox(height: NoStringSpacing.lg),
            ElevatedButton(
              onPressed: _loading ? null : _handleImport,
              child: _loading
                  ? const SizedBox(
                      height: 20,
                      width: 20,
                      child: CircularProgressIndicator(
                        strokeWidth: 2,
                        color: Colors.black,
                      ),
                    )
                  : const Text('Verify & Import'),
            ),
            if (_error != null) ...[
              const SizedBox(height: NoStringSpacing.lg),
              _buildError(),
            ],
            if (_vaultInfo != null) ...[
              const SizedBox(height: NoStringSpacing.lg),
              _buildSuccess(),
            ],
          ],
        ),
      ),
    );
  }

  Widget _buildError() {
    return Container(
      padding: const EdgeInsets.all(NoStringSpacing.md),
      decoration: BoxDecoration(
        color: NoStringColors.error.withValues(alpha: 0.1),
        border: Border.all(color: NoStringColors.error.withValues(alpha: 0.3)),
        borderRadius: NoStringRadius.md,
      ),
      child: Row(
        children: [
          const Icon(Icons.error_outline, color: NoStringColors.error, size: 20),
          const SizedBox(width: NoStringSpacing.sm),
          Expanded(
            child: Text(
              _error!,
              style: const TextStyle(color: NoStringColors.error, fontSize: 13),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildSuccess() {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(NoStringSpacing.lg),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                const Icon(Icons.verified, color: NoStringColors.success, size: 24),
                const SizedBox(width: NoStringSpacing.sm),
                const Text(
                  'Vault Verified',
                  style: TextStyle(
                    color: NoStringColors.success,
                    fontSize: 18,
                    fontWeight: FontWeight.bold,
                  ),
                ),
                const Spacer(),
                StatusBadge(
                  label: _vaultInfo!.network.toUpperCase(),
                  type: _vaultInfo!.network == 'bitcoin'
                      ? BadgeType.success
                      : BadgeType.warning,
                ),
              ],
            ),
            const SizedBox(height: NoStringSpacing.md),
            InfoRow(label: 'Address', value: _vaultInfo!.vaultAddress, mono: true),
            InfoRow(
              label: 'Timelock',
              value: '${_vaultInfo!.timelockBlocks} blocks (~${(_vaultInfo!.timelockBlocks / 144).toStringAsFixed(0)} days)',
            ),
            InfoRow(label: 'Heirs', value: _vaultInfo!.heirLabels.join(', ')),
            InfoRow(
              label: 'Recovery Data',
              value: _vaultInfo!.hasRecoveryLeaves ? 'Included' : 'Missing',
            ),
            const SizedBox(height: NoStringSpacing.lg),
            SizedBox(
              width: double.infinity,
              child: ElevatedButton.icon(
                onPressed: _navigateToStatus,
                icon: const Icon(Icons.search),
                label: const Text('Check Vault Status'),
              ),
            ),
          ],
        ),
      ),
    );
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }
}
