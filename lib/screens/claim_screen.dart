import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import '../src/rust/api.dart' as api;
import 'broadcast_screen.dart';

class ClaimScreen extends StatefulWidget {
  final String vaultJson;
  final String electrumUrl;
  final String network;

  const ClaimScreen({
    super.key,
    required this.vaultJson,
    required this.electrumUrl,
    required this.network,
  });

  @override
  State<ClaimScreen> createState() => _ClaimScreenState();
}

class _ClaimScreenState extends State<ClaimScreen> {
  final _addressController = TextEditingController();
  double _feeRate = 2.0;
  final int _heirIndex = 0;
  api.ClaimPsbt? _psbt;
  String? _error;
  bool _building = false;

  @override
  void dispose() {
    _addressController.dispose();
    super.dispose();
  }

  Future<void> _buildPsbt() async {
    final address = _addressController.text.trim();
    if (address.isEmpty) {
      setState(() => _error = 'Enter a destination address');
      return;
    }

    setState(() {
      _building = true;
      _error = null;
      _psbt = null;
    });

    try {
      final psbt = await api.buildClaimPsbt(
        vaultJson: widget.vaultJson,
        electrumUrl: widget.electrumUrl,
        destinationAddress: address,
        heirIndex: BigInt.from(_heirIndex),
        feeRateSatVb: BigInt.from(_feeRate.round()),
      );
      setState(() {
        _psbt = psbt;
        _building = false;
      });
    } catch (e) {
      setState(() {
        _error = e.toString();
        _building = false;
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Claim Funds')),
      body: Padding(
        padding: const EdgeInsets.all(16.0),
        child: _psbt != null ? _buildResult() : _buildForm(),
      ),
    );
  }

  Widget _buildForm() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        const Text(
          'Where should the funds be sent?',
          style: TextStyle(fontSize: 18, fontWeight: FontWeight.bold),
        ),
        const SizedBox(height: 16),
        TextField(
          controller: _addressController,
          decoration: const InputDecoration(
            labelText: 'Destination Bitcoin Address',
            hintText: 'bc1q... or tb1q...',
            border: OutlineInputBorder(),
          ),
        ),
        const SizedBox(height: 16),
        Text('Fee Rate: ${_feeRate.round()} sat/vB'),
        Slider(
          value: _feeRate,
          min: 1,
          max: 100,
          divisions: 99,
          label: '${_feeRate.round()} sat/vB',
          onChanged: (v) => setState(() => _feeRate = v),
        ),
        const SizedBox(height: 8),
        Text(
          'Heir Index: $_heirIndex',
          style: const TextStyle(color: Colors.grey),
        ),
        if (_error != null) ...[
          const SizedBox(height: 16),
          Text(_error!, style: const TextStyle(color: Colors.red)),
        ],
        const Spacer(),
        ElevatedButton.icon(
          onPressed: _building ? null : _buildPsbt,
          icon: _building
              ? const SizedBox(
                  width: 20,
                  height: 20,
                  child: CircularProgressIndicator(strokeWidth: 2),
                )
              : const Icon(Icons.build),
          label: Text(_building ? 'Building...' : 'Build Unsigned PSBT'),
          style: ElevatedButton.styleFrom(
            padding: const EdgeInsets.symmetric(vertical: 16),
          ),
        ),
      ],
    );
  }

  Widget _buildResult() {
    final p = _psbt!;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        const Icon(Icons.check_circle, size: 48, color: Colors.green),
        const SizedBox(height: 12),
        const Text(
          'Unsigned PSBT Ready',
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
                _row('Inputs', '${p.numInputs}'),
                _row('Total In', '${p.totalInputSat} sats'),
                _row('Fee', '${p.feeSat} sats'),
                _row('Output', '${p.outputSat} sats'),
                const Divider(),
                Text(
                  'To: ${p.destination}',
                  style: const TextStyle(fontSize: 12, color: Colors.grey),
                ),
              ],
            ),
          ),
        ),
        const SizedBox(height: 16),
        const Text(
          'Copy this PSBT and sign it with your wallet (Sparrow, hardware wallet, etc.)',
          style: TextStyle(color: Colors.grey),
          textAlign: TextAlign.center,
        ),
        const SizedBox(height: 16),
        ElevatedButton.icon(
          onPressed: () {
            Clipboard.setData(ClipboardData(text: p.psbtBase64));
            ScaffoldMessenger.of(context).showSnackBar(
              const SnackBar(content: Text('PSBT copied to clipboard')),
            );
          },
          icon: const Icon(Icons.copy),
          label: const Text('Copy PSBT (Base64)'),
          style: ElevatedButton.styleFrom(
            padding: const EdgeInsets.symmetric(vertical: 16),
          ),
        ),
        const SizedBox(height: 8),
        ElevatedButton.icon(
          onPressed: () {
            Navigator.push(
              context,
              MaterialPageRoute(
                builder: (_) => BroadcastScreen(
                  electrumUrl: widget.electrumUrl,
                  network: widget.network,
                ),
              ),
            );
          },
          icon: const Icon(Icons.arrow_forward),
          label: const Text("I've Signed It — Broadcast"),
          style: ElevatedButton.styleFrom(
            backgroundColor: Colors.green,
            padding: const EdgeInsets.symmetric(vertical: 16),
          ),
        ),
        const SizedBox(height: 8),
        OutlinedButton(
          onPressed: () {
            setState(() {
              _psbt = null;
              _error = null;
            });
          },
          child: const Text('Build Another'),
        ),
      ],
    );
  }

  Widget _row(String label, String value) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          Text(label, style: const TextStyle(color: Colors.grey)),
          Text(value, style: const TextStyle(fontWeight: FontWeight.bold)),
        ],
      ),
    );
  }
}
