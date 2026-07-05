import 'package:flutter_test/flutter_test.dart';
import 'package:nml_flutter/main.dart';

void main() {
  testWidgets('NML home page renders', (tester) async {
    await tester.pumpWidget(const NMLApp());

    expect(find.text('N0th1ngness Minecraft Launcher'), findsOneWidget);
    expect(find.text('快速启动'), findsOneWidget);
  });
}
