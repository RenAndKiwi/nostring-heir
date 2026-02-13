import 'package:flutter_test/flutter_test.dart';
import 'package:nostring_heir/main.dart';

void main() {
  testWidgets('App renders import screen', (WidgetTester tester) async {
    await tester.pumpWidget(const NoStringHeirApp());
    expect(find.text('Import Vault Backup'), findsOneWidget);
    expect(find.text('Import Backup'), findsOneWidget);
  });
}
