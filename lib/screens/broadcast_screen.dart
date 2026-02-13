import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import '../src/rust/api.dart' as api;
import '../theme/nostring_theme.dart';
import '../widgets/gold_gradient_text.dart';
import '../widgets/info_row.dart';
import '../widgets/section_header.dart';

class BroadcastScreen extends StatefulWidget {
  final String electrumUrl;
  final String network;

  const BroadcastScreen({
    super.key,
    required this.electrumUrl,
    required this.network,
  });

  @override
  State<BroadcastScreen> createState() => _BroadcastScreenState();
}

class _BroadcastScreenState extends State<BroadcastScreen> {
  final _psbtController = TextEditingController();
  api.FinalizedTx? _finalizedTx;
  api.BroadcastResult? _broadcastResult;
  String? _error;
  bool _loading = false;

  @override
  void dispose() {
    _psbtController.dispose();
    super.dispose();
  }

  Future<void> _finalize() async {
    final psbt = _psbtController.text.trim();
    if (psbt.isEmpty) {
      setState(() => _error = 'Paste the signed PSBT');
      return;
    }
    setState(() { _loading = true; _error = null; });
    try {
      final tx = await api.finalizePsbt(psbtBase64: psbt);
      setState(() { _finalizedTx = tx; _loading = false; });
    } catch (e) {
      setState(() { _error = e.toString(); _loading = false; });
    }
  }

  Future<void> _broadcast() async {
    if (_finalizedTx == null) return;
    setState(() { _loading = true; _error = null; });
    try {
      final result = await api.broadcastTransaction(
        txHex: _finalizedTx!.txHex,
        electrumUrl: widget.electrumUrl,
        network: widget.network,
      );
      setState(() { _broadcastResult = result; _loading = false; });
    } catch (e) {
      setState(() { _error = e.toString(); _loading = false; });
    }
  }

  String get _explorerUrl {
    final txid = _broadcastResult?.txid ?? '';
    return switch (widget.network) {
      'testnet' => 'https://mempool.space/testnet/tx/$txid',
      'signet' => 'https://mempool.space/signet/tx/$txid',
      _ => 'https://mempool.space/tx/$txid',
    };
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Broadcast')),
      body: Padding(
        padding: const EdgeInsets.all(NoStringSpacing.lg),
        child: _broadcastResult != null
            ? _buildSuccess()
            : _finalizedTx != null
                ? _buildReview()
                : _buildInput(),
      ),
    );
  }

  Widget _buildInput() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        const SectionHeader(
          title: 'Import Signed PSBT',
          subtitle: 'Paste the signed PSBT from your wallet.',
        ),
        const SizedBox(height: NoStringSpacing.lg),
        Expanded(
          child: TextField(
            controller: _psbtController,
            maxLines: null,
            expands: true,
            textAlignVertical: TextAlignVertical.top,
            style: const TextStyle(fontFamily: 'monospace', fontSize: 12),
            decoration: const InputDecoration(hintText: 'cHNidP8B...'),
          ),
        ),
        if (_error != null) ...[
          const SizedBox(height: NoStringSpacing.md),
          Container(
            padding: const EdgeInsets.all(NoStringSpacing.md),
            decoration: BoxDecoration(
              color: NoStringColors.error.withValues(alpha: 0.1),
              border: Border.all(color: NoStringColors.error.withValues(alpha: 0.3)),
              borderRadius: NoStringRadius.md,
            ),
            child: Text(
              _error!,
              style: const TextStyle(color: NoStringColors.error, fontSize: 13),
            ),
          ),
        ],
        const SizedBox(height: NoStringSpacing.lg),
        ElevatedButton.icon(
          onPressed: _loading ? null : _finalize,
          icon: _loading
              ? const SizedBox(
                  width: 20, height: 20,
                  child: CircularProgressIndicator(strokeWidth: 2, color: Colors.black))
              : const Icon(Icons.check),
          label: Text(_loading ? 'Validating...' : 'Validate Signed PSBT'),
        ),
      ],
    );
  }

  Widget _buildReview() {
    final tx = _finalizedTx!;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        const SizedBox(height: NoStringSpacing.xl),
        const Icon(Icons.verified, size: 48, color: NoStringColors.success),
        const SizedBox(height: NoStringSpacing.md),
        const GoldGradientText(
          'Transaction Ready',
          fontSize: 22,
          textAlign: TextAlign.center,
        ),
        const SizedBox(height: NoStringSpacing.xl),
        Card(
          child: Padding(
            padding: const EdgeInsets.all(NoStringSpacing.lg),
            child: Column(
              children: [
                InfoRow(label: 'TXID', value: tx.txid, mono: true),
                InfoRow(label: 'Inputs', value: '${tx.numInputs}'),
                InfoRow(label: 'Outputs', value: '${tx.numOutputs}'),
                InfoRow(label: 'Total Output', value: '${tx.totalOutputSat} sats'),
              ],
            ),
          ),
        ),
        if (_error != null) ...[
          const SizedBox(height: NoStringSpacing.md),
          Text(_error!, style: const TextStyle(color: NoStringColors.error)),
        ],
        const Spacer(),
        ElevatedButton.icon(
          onPressed: _loading ? null : _broadcast,
          style: ElevatedButton.styleFrom(backgroundColor: NoStringColors.success),
          icon: _loading
              ? const SizedBox(
                  width: 20, height: 20,
                  child: CircularProgressIndicator(strokeWidth: 2, color: Colors.white))
              : const Icon(Icons.send),
          label: Text(
            _loading ? 'Broadcasting...' : 'Broadcast Transaction',
            style: const TextStyle(color: Colors.white),
          ),
        ),
        const SizedBox(height: NoStringSpacing.sm),
        OutlinedButton(
          onPressed: () => setState(() { _finalizedTx = null; _error = null; }),
          child: const Text('Back'),
        ),
      ],
    );
  }

  Widget _buildSuccess() {
    final r = _broadcastResult!;
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Container(
            padding: const EdgeInsets.all(NoStringSpacing.xl),
            decoration: BoxDecoration(
              shape: BoxShape.circle,
              color: NoStringColors.goldLight.withValues(alpha: 0.1),
            ),
            child: const Icon(
              Icons.celebration,
              size: 64,
              color: NoStringColors.goldLight,
            ),
          ),
          const SizedBox(height: NoStringSpacing.xl),
          const GoldGradientText(
            'Funds Claimed!',
            fontSize: 28,
            textAlign: TextAlign.center,
          ),
          const SizedBox(height: NoStringSpacing.xl),
          Card(
            child: Padding(
              padding: const EdgeInsets.all(NoStringSpacing.lg),
              child: Column(
                children: [
                  const Text(
                    'Transaction ID',
                    style: TextStyle(color: NoStringColors.textMuted, fontSize: 12),
                  ),
                  const SizedBox(height: NoStringSpacing.sm),
                  SelectableText(
                    r.txid,
                    style: const TextStyle(
                      fontFamily: 'monospace',
                      fontSize: 11,
                      color: NoStringColors.goldLight,
                    ),
                    textAlign: TextAlign.center,
                  ),
                ],
              ),
            ),
          ),
          const SizedBox(height: NoStringSpacing.lg),
          ElevatedButton.icon(
            onPressed: () {
              Clipboard.setData(ClipboardData(text: r.txid));
              ScaffoldMessenger.of(context).showSnackBar(
                const SnackBar(content: Text('TXID copied')),
              );
            },
            icon: const Icon(Icons.copy),
            label: const Text('Copy TXID'),
          ),
          const SizedBox(height: NoStringSpacing.sm),
          Text(
            _explorerUrl,
            style: const TextStyle(fontSize: 11, color: NoStringColors.textMuted),
            textAlign: TextAlign.center,
          ),
        ],
      ),
    );
  }
}
