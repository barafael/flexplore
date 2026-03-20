#!/usr/bin/env python3
"""Generate Swift view files and snapshot test from testdata/*/expected.swift."""

import re
import os
from pathlib import Path

TESTDATA = Path(__file__).resolve().parent.parent.parent.parent / "testdata"
CASES_DIR = Path(__file__).resolve().parent.parent / "Sources" / "SwiftGolden" / "Cases"
TEST_DIR = Path(__file__).resolve().parent.parent / "Tests" / "SwiftGoldenTests"

# Viewport used for rendering (must match the snapshot test size)
VIEWPORT_W = 400.0
VIEWPORT_H = 300.0


def snake_to_camel(s: str) -> str:
    return "".join(w.capitalize() for w in s.split("_"))


def adapt_swift_source(src: str, class_name: str) -> str:
    """Transform generated expected.swift into a compilable macOS SwiftUI view."""
    # Rename ContentView → {ClassName}View
    src = src.replace("struct ContentView:", f"public struct {class_name}View:")

    # Replace iOS UIScreen viewport references with fixed constants (macOS has no UIScreen)
    src = re.sub(
        r"UIScreen\.main\.bounds\.width\s*\*\s*([\d.]+)",
        lambda m: f"{VIEWPORT_W * float(m.group(1)):.1f}",
        src,
    )
    src = re.sub(
        r"UIScreen\.main\.bounds\.height\s*\*\s*([\d.]+)",
        lambda m: f"{VIEWPORT_H * float(m.group(1)):.1f}",
        src,
    )

    return src


def generate():
    CASES_DIR.mkdir(parents=True, exist_ok=True)

    cases = sorted(
        d.name
        for d in TESTDATA.iterdir()
        if d.is_dir() and (d / "expected.swift").exists()
    )

    # Generate a view file per case
    for name in cases:
        swift_src = (TESTDATA / name / "expected.swift").read_text(encoding="utf-8")
        class_name = snake_to_camel(name)
        adapted = adapt_swift_source(swift_src, class_name)

        view_code = (
            f"// AUTO-GENERATED — do not edit. Run tool/generate_cases.py to regenerate.\n"
            f"import SwiftUI\n\n"
            f"{adapted}"
        )
        (CASES_DIR / f"{name}.swift").write_text(view_code, encoding="utf-8")

    # Generate snapshot test file
    test_imports = "\n".join(
        f"@testable import SwiftGolden" for _ in cases[:1]  # single import suffices
    )

    test_funcs = "\n\n".join(
        f"""    func test_{name}() {{
        let view = NSHostingController(rootView:
            {snake_to_camel(name)}View()
                .frame(width: {VIEWPORT_W:.0f}, height: {VIEWPORT_H:.0f}, alignment: .topLeading)
                .background(Color(red: 0.11, green: 0.11, blue: 0.18))
        )
        assertSnapshot(of: view, as: .image(size: CGSize(width: {VIEWPORT_W:.0f}, height: {VIEWPORT_H:.0f})))
    }}"""
        for name in cases
    )

    test_code = f"""// AUTO-GENERATED — do not edit. Run tool/generate_cases.py to regenerate.
import XCTest
import SwiftUI
import AppKit
import SnapshotTesting

@testable import SwiftGolden

final class GoldenTests: XCTestCase {{
    override func invokeTest() {{
        // When SWIFT_SNAPSHOT_RECORD=1, always (re-)generate golden PNGs.
        if ProcessInfo.processInfo.environment["SWIFT_SNAPSHOT_RECORD"] == "1" {{
            withSnapshotTesting(record: .all) {{
                super.invokeTest()
            }}
        }} else {{
            super.invokeTest()
        }}
    }}

{test_funcs}
}}
"""
    (TEST_DIR / "GoldenTests.swift").write_text(test_code, encoding="utf-8")

    print(f"Generated {len(cases)} view files in {CASES_DIR}")
    print(f"Generated snapshot test in {TEST_DIR / 'GoldenTests.swift'}")


if __name__ == "__main__":
    generate()
