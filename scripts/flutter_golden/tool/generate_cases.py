#!/usr/bin/env python3
"""Generate Dart widget files and golden test from testdata/*/expected.dart."""

import os
from pathlib import Path

TESTDATA = Path(__file__).resolve().parent.parent.parent.parent / "testdata"
LIB_DIR = Path(__file__).resolve().parent.parent / "lib" / "cases"
TEST_DIR = Path(__file__).resolve().parent.parent / "test"


def snake_to_camel(s: str) -> str:
    return "".join(w.capitalize() for w in s.split("_"))


def generate():
    LIB_DIR.mkdir(parents=True, exist_ok=True)

    cases = sorted(
        d.name for d in TESTDATA.iterdir()
        if d.is_dir() and (d / "expected.dart").exists()
    )

    # Generate a widget file per case
    for name in cases:
        dart_src = (TESTDATA / name / "expected.dart").read_text(encoding="utf-8")
        class_name = snake_to_camel(name)

        # The generated code defines: Widget build(BuildContext context) { return ...; }
        # We wrap it in a StatelessWidget class.
        widget_code = f"""// AUTO-GENERATED — do not edit. Run tool/generate_cases.py to regenerate.
import 'package:flutter/material.dart';

class {class_name} extends StatelessWidget {{
  const {class_name}({{super.key}});

  @override
  {dart_src}}}
"""
        (LIB_DIR / f"{name}.dart").write_text(widget_code, encoding="utf-8")

    # Generate barrel export
    barrel = "// AUTO-GENERATED\n"
    for name in cases:
        barrel += f"export 'cases/{name}.dart';\n"
    (LIB_DIR.parent / "flutter_golden.dart").write_text(barrel, encoding="utf-8")

    # Generate golden test file
    test_imports = "\n".join(
        f"import 'package:flutter_golden/cases/{name}.dart';"
        for name in cases
    )
    test_cases = "\n".join(
        f"""  testWidgets('{name}', (tester) async {{
    tester.view.devicePixelRatio = 1.0;
    addTearDown(() => tester.view.resetDevicePixelRatio());
    await tester.binding.setSurfaceSize(const Size(400, 300));
    await tester.pumpWidget(
      MaterialApp(
        debugShowCheckedModeBanner: false,
        home: Scaffold(
          backgroundColor: const Color(0xFF1C1C2E),
          body: {snake_to_camel(name)}(),
        ),
      ),
    );
    // Consume any overflow errors so the golden is still captured.
    tester.takeException();
    await expectLater(
      find.byType(MaterialApp),
      matchesGoldenFile('goldens/{name}.png'),
    );
  }});"""
        for name in cases
    )

    test_code = f"""// AUTO-GENERATED — do not edit. Run tool/generate_cases.py to regenerate.
import 'dart:io';
import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

{test_imports}

void main() {{
  setUpAll(() async {{
    // Load Roboto so golden tests render real text instead of placeholder blocks.
    final fontFile = File('fonts/Roboto-Regular.ttf');
    if (fontFile.existsSync()) {{
      await ui.loadFontFromList(fontFile.readAsBytesSync(), fontFamily: 'Roboto');
    }}
  }});

{test_cases}
}}
"""
    (TEST_DIR / "golden_test.dart").write_text(test_code, encoding="utf-8")

    print(f"Generated {len(cases)} widget files in {LIB_DIR}")
    print(f"Generated golden test in {TEST_DIR / 'golden_test.dart'}")


if __name__ == "__main__":
    generate()
