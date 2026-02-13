import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import '../src/rust/api.dart' as api;

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

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Broadcast Transaction')),
      body: Padding(
        padding: const EdgeInsets.all(16.0),
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
        const Text(
          'Import Signed PSBT',
          style: TextStyle(fontSize: 18, fontWeight: FontWeight.bold),
        ),
        const SizedBox(height: 8),
        const Text(
          'Paste the signed PSBT (base64) from your signing device.',
          style: TextStyle(color: Colors.grey),
        ),
        const SizedBox(height: 16),
        Expanded(
          child: TextField(
            controller: _psbtController,
            maxLines: null,
            expands: true,
            textAlignVertical: TextAlignVertical.top,
            style: const TextStyle(fontFamily: 'monospace', fontSize: 12),
            decoration: const InputDecoration(
              hintText: 'cHNidP8B...',
              border: OutlineInputBorder(),
            ),
          ),
        ),
        if (_error != null) ...[
          const SizedBox(height: 12),
          Text(_error!, style: const TextStyle(color: Colors.red)),
        ],
        const SizedBox(height: 16),
        ElevatedButton.icon(
          onPressed: _loading ? null : _finalize,
          icon: _loading
              ? const SizedBox(
                  width: 20, height: 20,
                  child: CircularProgressIndicator(strokeWidth: 2))
              : const Icon(Icons.check),
          label: Text(_loading ? 'Validating...' : 'Validate Signed PSBT'),
          style: ElevatedButton.styleFrom(
            padding: const EdgeInsets.symmetric(vertical: 16),
          ),
        ),
      ],
    );
  }

  Widget _buildReview() {
    final tx = _finalizedTx!;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        const Icon(Icons.verified, size: 48, color: Colors.green),
        const SizedBox(height: 12),
        const Text(
          'Transaction Ready',
          style: TextStyle(fontSize: 20, fontWeight: FontWeight.bold),
          textAlign: TextAlign.center,
        ),
        const SizedBox(height: 16),
        Card(
          child: Padding(
            padding: const EdgeInsets.all(16.0),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                _row('TXID', tx.txid),
                _row('Inputs', '${tx.numInputs}'),
                _row('Outputs', '${tx.numOutputs}'),
                _row('Total Output', '${tx.totalOutputSat} sats'),
              ],
            ),
          ),
        ),
        if (_error != null) ...[
          const SizedBox(height: 12),
          Text(_error!, style: const TextStyle(color: Colors.red)),
        ],
        const Spacer(),
        ElevatedButton.icon(
          onPressed: _loading ? null : _broadcast,
          icon: _loading
              ? const SizedBox(
                  width: 20, height: 20,
                  child: CircularProgressIndicator(strokeWidth: 2))
              : const Icon(Icons.send),
          label: Text(_loading ? 'Broadcasting...' : 'Broadcast Transaction'),
          style: ElevatedButton.styleFrom(
            backgroundColor: Colors.green,
            padding: const EdgeInsets.symmetric(vertical: 16),
          ),
        ),
        const SizedBox(height: 8),
        OutlinedButton(
          onPressed: () => setState(() { _finalizedTx = null; _error = null; }),
          child: const Text('Back'),
        ),
      ],
    );
  }

  Widget _buildSuccess() {
    final r = _broadcastResult!;
    final explorerUrl = widget.network == 'testnet'
        ? 'https://mempool.space/testnet/tx/${r.txid}'
        : widget.network == 'signet'
            ? 'https://mempool.space/signet/tx/${r.txid}'
            : 'https://mempool.space/tx/${r.txid}';
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          const Icon(Icons.celebration, size: 64, color: Color(0xFFF7931A)),
          const SizedBox(height: 16),
          const Text(
            'Transaction Broadcast!',
            style: TextStyle(fontSize: 24, fontWeight: FontWeight.bold),
          ),
          const SizedBox(height: 24),
          SelectableText(
            r.txid,
            style: const TextStyle(fontFamily: 'monospace', fontSize: 12),
            textAlign: TextAlign.center,
          ),
          const SizedBox(height: 16),
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
          const SizedBox(height: 8),
          Text(
            explorerUrl,
            style: const TextStyle(fontSize: 11, color: Colors.grey),
            textAlign: TextAlign.center,
          ),
        ],
      ),
    );
  }

  Widget _row(String label, String value) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          Text(label, style: const TextStyle(color: Colors.grey)),
          Flexible(
            child: Text(
              value,
              style: const TextStyle(fontWeight: FontWeight.bold, fontSize: 12),
              overflow: TextOverflow.ellipsis,
            ),
          ),
        ],
      ),
    );
  }
}
