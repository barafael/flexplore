#!/usr/bin/env python3
"""Build an HTML overview page comparing rendered images across testdata cases."""

import os
from pathlib import Path

TESTDATA = Path(__file__).resolve().parent.parent / "testdata"
OUTPUT = TESTDATA / "overview.html"

IMAGES = [
    ("Bevy", "rendered_bevy.png"),
    ("HTML/CSS", "rendered_html.png"),
    ("Tailwind", "rendered_tailwind.png"),
    ("Flutter", "rendered_flutter.png"),
    ("SwiftUI", "rendered_swift.png"),
    ("Iced", "rendered_iced.png"),
]

I = "  "  # indent unit


def build_panel(name: str, label: str, filename: str) -> str:
    path = TESTDATA / name / filename
    lines = [
        f'{I * 4}<div class="panel">',
        f'{I * 5}<div class="label">{label}</div>',
    ]
    if path.exists():
        lines.append(f'{I * 5}<img src="{name}/{filename}" loading="lazy">')
    else:
        lines.append(f'{I * 5}<div class="missing">not rendered</div>')
    lines.append(f'{I * 4}</div>')
    return "\n".join(lines)


def build_case(name: str) -> str:
    panels = "\n".join(
        build_panel(name, label, filename) for label, filename in IMAGES
    )
    return (
        f'{I * 2}<section class="case">\n'
        f'{I * 3}<h2>{name}</h2>\n'
        f'{I * 3}<div class="images">\n'
        f'{panels}\n'
        f'{I * 3}</div>\n'
        f'{I * 2}</section>'
    )


def build_html() -> str:
    cases = sorted(
        d.name for d in TESTDATA.iterdir()
        if d.is_dir() and any((d / img).exists() for _, img in IMAGES)
    )

    cards = "\n".join(build_case(name) for name in cases)

    return f"""\
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>Flexplain Render Overview</title>
  <style>
    * {{ margin: 0; padding: 0; box-sizing: border-box; }}
    body {{ font-family: system-ui, sans-serif; background: #111; color: #ddd; padding: 24px; }}
    h1 {{ text-align: center; margin-bottom: 24px; }}
    .case {{ margin-bottom: 32px; border: 1px solid #333; border-radius: 8px; overflow: hidden; }}
    .case h2 {{ background: #1a1a2e; padding: 10px 16px; font-size: 16px; }}
    .images {{ display: flex; gap: 2px; background: #222; }}
    .panel {{ flex: 1; min-width: 0; background: #1a1a1a; }}
    .label {{ text-align: center; padding: 4px; font-size: 12px; font-weight: 600; background: #0f3460; }}
    .panel img {{ width: 100%; height: auto; display: block; }}
    .missing {{ text-align: center; padding: 40px; color: #666; font-style: italic; }}
  </style>
</head>
<body>
  <h1>Flexplain Render Overview</h1>
{cards}
</body>
</html>
"""


if __name__ == "__main__":
    html = build_html()
    OUTPUT.write_text(html, encoding="utf-8")
    print(f"Written to {OUTPUT}")
