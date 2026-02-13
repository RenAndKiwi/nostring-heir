import 'package:flutter/material.dart';
import '../src/rust/api.dart' as api;
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
          ),
        ],
      ),
      body: Padding(
        padding: const EdgeInsets.all(16.0),
        child: _loading
            ? const Center(child: CircularProgressIndicator())
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
          const Icon(Icons.error_outline, size: 48, color: Colors.red),
          const SizedBox(height: 16),
          Text(
            _error!,
            style: const TextStyle(color: Colors.red),
            textAlign: TextAlign.center,
          ),
          const SizedBox(height: 16),
          ElevatedButton(
            onPressed: _fetchStatus,
            child: const Text('Retry'),
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
        Card(
          child: Padding(
            padding: const EdgeInsets.all(16.0),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                const Text('Balance',
                    style: TextStyle(fontSize: 14, color: Colors.grey)),
                const SizedBox(height: 4),
                Text(
                  '${s.balanceSat} sats',
                  style: const TextStyle(
                      fontSize: 28, fontWeight: FontWeight.bold),
                ),
                const SizedBox(height: 8),
                Text('${s.utxoCount} UTXO(s)',
                    style: const TextStyle(color: Colors.grey)),
              ],
            ),
          ),
        ),
        const SizedBox(height: 12),
        Card(
          child: Padding(
            padding: const EdgeInsets.all(16.0),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                const Text('Timelock Status',
                    style: TextStyle(fontSize: 14, color: Colors.grey)),
                const SizedBox(height: 8),
                Row(
                  children: [
                    Icon(
                      s.eligible ? Icons.check_circle : Icons.hourglass_bottom,
                      color: s.eligible ? Colors.green : Colors.orange,
                      size: 32,
                    ),
                    const SizedBox(width: 12),
                    Expanded(
                      child: Text(
                        s.eligible
                            ? 'Timelock expired — eligible to claim'
                            : '~${s.daysRemaining.toStringAsFixed(1)} days remaining (${s.blocksRemaining} blocks)',
                        style: TextStyle(
                          fontSize: 16,
                          color: s.eligible ? Colors.green : Colors.orange,
                        ),
                      ),
                    ),
                  ],
                ),
                const SizedBox(height: 8),
                Text(
                  'Current height: ${s.currentHeight}  |  Confirmed at: ${s.confirmationHeight}',
                  style: const TextStyle(fontSize: 12, color: Colors.grey),
                ),
              ],
            ),
          ),
        ),
        const Spacer(),
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
          style: ElevatedButton.styleFrom(
            padding: const EdgeInsets.symmetric(vertical: 16),
          ),
        ),
      ],
    );
  }
}

