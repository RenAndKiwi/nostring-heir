import 'package:flutter/material.dart';
import 'src/rust/frb_generated.dart';
import 'screens/import_screen.dart';
import 'theme/nostring_theme.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await RustLib.init();
  runApp(const NoStringHeirApp());
}

class NoStringHeirApp extends StatelessWidget {
  const NoStringHeirApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'NoString Heir',
      theme: noStringTheme(),
      home: const ImportScreen(),
      debugShowCheckedModeBanner: false,
    );
  }
}
