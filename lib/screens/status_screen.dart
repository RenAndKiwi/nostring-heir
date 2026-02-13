import 'package:flutter/material.dart';
import '../src/rust/api.dart' as api;
import '../theme/nostring_theme.dart';
import '../widgets/gold_gradient_text.dart';
import '../widgets/info_row.dart';
import '../widgets/status_badge.dart';
import 'claim_screen.dart';

class StatusScreen extends StatefulWidget {
  final String vaultJson;
  final String electrumUrl;
  final String network;

  const StatusScreen({
    super.key,
    required this.vaultJson,
    required this.electrumUrl,
    required this.network,
  });

  @override
  State<StatusScreen> createState() => _StatusScreenState();
}

class _StatusScreenState extends State<StatusScreen> {
  api.VaultStatus? _status;
  String? _error;
  bool _loading = true;

  @override
  void initState() {
    super.initState();
    _fetchStatus();
  }

  Future<void> _fetchStatus() async {
    setState(() {
      _loading = true;
      _error = null;
    });
    try {
      final status = await api.fetchVaultStatus(
        vaultJson: widget.vaultJson,
        electrumUrl: widget.electrumUrl,
      );
      setState(() {
        _status = status;
        _loading = false;
      });
    } catch (e) {
      setState(() {
        _error = e.toString();
        _loading = false;
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Vault Status'),
        actions: [
          IconButton(
            icon: const Icon(Icons.refresh),
            onPressed: _fetchStatus,
            tooltip: 'Refresh',
          ),
        ],
      ),
      body: Padding(
        padding: const EdgeInsets.all(NoStringSpacing.lg),
        child: _loading
            ? const Center(
                child: CircularProgressIndicator(color: NoStringColors.goldLight),
              )
            : _error != null
                ? _buildError()
                : _buildStatus(),
      ),
    );
  }

  Widget _buildError() {
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          const Icon(Icons.cloud_off, size: 48, color: NoStringColors.error),
          const SizedBox(height: NoStringSpacing.lg),
          Text(
            _error!,
            style: const TextStyle(color: NoStringColors.error),
            textAlign: TextAlign.center,
          ),
          const SizedBox(height: NoStringSpacing.xl),
          OutlinedButton.icon(
            onPressed: _fetchStatus,
            icon: const Icon(Icons.refresh),
            label: const Text('Retry'),
          ),
        ],
      ),
    );
  }

  Widget _buildStatus() {
    final s = _status!;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        // Balance card
        Card(
          child: Padding(
            padding: const EdgeInsets.all(NoStringSpacing.xl),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                const Text(
                  'Balance',
                  style: TextStyle(fontSize: 13, color: NoStringColors.textMuted),
                ),
                const SizedBox(height: NoStringSpacing.xs),
                GoldGradientText(
                  '${s.balanceSat} sats',
                  fontSize: 32,
                ),
                const SizedBox(height: NoStringSpacing.sm),
                Text(
                  '${s.utxoCount} UTXO(s)',
                  style: const TextStyle(
                    color: NoStringColors.textMuted,
                    fontSize: 13,
                  ),
                ),
              ],
            ),
          ),
        ),
        const SizedBox(height: NoStringSpacing.md),

        // Timelock card
        Card(
          child: Padding(
            padding: const EdgeInsets.all(NoStringSpacing.lg),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Row(
                  children: [
                    const Text(
                      'Timelock',
                      style: TextStyle(fontSize: 13, color: NoStringColors.textMuted),
                    ),
                    const Spacer(),
                    StatusBadge(
                      label: s.eligible ? 'ELIGIBLE' : 'LOCKED',
                      type: s.eligible ? BadgeType.success : BadgeType.warning,
                      icon: s.eligible
                          ? Icons.lock_open
                          : Icons.hourglass_bottom,
                    ),
                  ],
                ),
                const SizedBox(height: NoStringSpacing.md),
                if (s.eligible)
                  const Text(
                    'Timelock has expired. You can claim your funds.',
                    style: TextStyle(
                      color: NoStringColors.success,
                      fontSize: 15,
                    ),
                  )
                else
                  Text(
                    '~${s.daysRemaining.toStringAsFixed(1)} days remaining (${s.blocksRemaining} blocks)',
                    style: const TextStyle(
                      color: NoStringColors.warning,
                      fontSize: 15,
                    ),
                  ),
                const SizedBox(height: NoStringSpacing.md),
                InfoRow(
                  label: 'Current Height',
                  value: '${s.currentHeight}',
                  mono: true,
                ),
                InfoRow(
                  label: 'Confirmed At',
                  value: '${s.confirmationHeight}',
                  mono: true,
                ),
              ],
            ),
          ),
        ),

        const Spacer(),

        // Claim button
        ElevatedButton.icon(
          onPressed: s.eligible && s.balanceSat > BigInt.zero
              ? () {
                  Navigator.push(
                    context,
                    MaterialPageRoute(
                      builder: (_) => ClaimScreen(
                        vaultJson: widget.vaultJson,
                        electrumUrl: widget.electrumUrl,
                        network: widget.network,
                      ),
                    ),
                  );
                }
              : null,
          icon: const Icon(Icons.send),
          label: const Text('Claim Funds'),
        ),
      ],
    );
  }
}
