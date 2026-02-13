import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import '../src/rust/api.dart' as api;
import '../theme/nostring_theme.dart';
import '../widgets/gold_gradient_text.dart';
import '../widgets/info_row.dart';
import '../widgets/section_header.dart';
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
        padding: const EdgeInsets.all(NoStringSpacing.lg),
        child: _psbt != null ? _buildResult() : _buildForm(),
      ),
    );
  }

  Widget _buildForm() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        const SectionHeader(
          title: 'Where should the funds go?',
          subtitle: 'Enter your Bitcoin address. This is where your inheritance will be sent.',
        ),
        const SizedBox(height: NoStringSpacing.xl),
        TextField(
          controller: _addressController,
          decoration: const InputDecoration(
            labelText: 'Destination Address',
            hintText: 'bc1q... or tb1q...',
            prefixIcon: Icon(Icons.account_balance_wallet, color: NoStringColors.goldLight),
          ),
        ),
        const SizedBox(height: NoStringSpacing.xl),
        Row(
          children: [
            const Text('Fee Rate', style: TextStyle(color: NoStringColors.textMuted)),
            const Spacer(),
            Text(
              '${_feeRate.round()} sat/vB',
              style: const TextStyle(
                color: NoStringColors.goldLight,
                fontWeight: FontWeight.w600,
              ),
            ),
          ],
        ),
        Slider(
          value: _feeRate,
          min: 1,
          max: 100,
          divisions: 99,
          onChanged: (v) => setState(() => _feeRate = v),
        ),
        if (_error != null) ...[
          const SizedBox(height: NoStringSpacing.lg),
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
        const Spacer(),
        ElevatedButton.icon(
          onPressed: _building ? null : _buildPsbt,
          icon: _building
              ? const SizedBox(
                  width: 20,
                  height: 20,
                  child: CircularProgressIndicator(strokeWidth: 2, color: Colors.black),
                )
              : const Icon(Icons.build),
          label: Text(_building ? 'Building...' : 'Build Unsigned PSBT'),
        ),
      ],
    );
  }

  Widget _buildResult() {
    final p = _psbt!;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        const SizedBox(height: NoStringSpacing.xl),
        const Icon(Icons.check_circle, size: 56, color: NoStringColors.success),
        const SizedBox(height: NoStringSpacing.md),
        const GoldGradientText(
          'Unsigned PSBT Ready',
          fontSize: 22,
          textAlign: TextAlign.center,
        ),
        const SizedBox(height: NoStringSpacing.xl),
        Card(
          child: Padding(
            padding: const EdgeInsets.all(NoStringSpacing.lg),
            child: Column(
              children: [
                InfoRow(label: 'Inputs', value: '${p.numInputs}'),
                InfoRow(label: 'Total In', value: '${p.totalInputSat} sats'),
                InfoRow(label: 'Fee', value: '${p.feeSat} sats'),
                InfoRow(label: 'You Receive', value: '${p.outputSat} sats'),
                const Divider(height: NoStringSpacing.xl),
                InfoRow(label: 'Destination', value: p.destination, mono: true),
              ],
            ),
          ),
        ),
        const SizedBox(height: NoStringSpacing.lg),
        const Text(
          'Copy this PSBT and sign it with your wallet\n(Sparrow, hardware wallet, etc.)',
          style: TextStyle(color: NoStringColors.textMuted, fontSize: 13),
          textAlign: TextAlign.center,
        ),
        const Spacer(),
        ElevatedButton.icon(
          onPressed: () {
            Clipboard.setData(ClipboardData(text: p.psbtBase64));
            ScaffoldMessenger.of(context).showSnackBar(
              const SnackBar(content: Text('PSBT copied to clipboard')),
            );
          },
          icon: const Icon(Icons.copy),
          label: const Text('Copy PSBT'),
        ),
        const SizedBox(height: NoStringSpacing.sm),
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
          style: ElevatedButton.styleFrom(
            backgroundColor: NoStringColors.success,
          ),
          icon: const Icon(Icons.arrow_forward),
          label: const Text("I've Signed It"),
        ),
        const SizedBox(height: NoStringSpacing.sm),
        OutlinedButton(
          onPressed: () => setState(() {
            _psbt = null;
            _error = null;
          }),
          child: const Text('Build Another'),
        ),
      ],
    );
  }
}
