import 'package:flutter/material.dart';
import 'src/rust/frb_generated.dart';
import 'screens/import_screen.dart';

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
      theme: ThemeData.dark().copyWith(
        colorScheme: ColorScheme.dark(
          primary: const Color(0xFFF7931A),
          secondary: const Color(0xFFF7931A),
          surface: const Color(0xFF0A0A0A),
        ),
        scaffoldBackgroundColor: const Color(0xFF0A0A0A),
      ),
      home: const ImportScreen(),
      debugShowCheckedModeBanner: false,
    );
  }
}
