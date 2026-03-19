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
]


def build_html() -> str:
    cases = sorted(
        d.name for d in TESTDATA.iterdir()
        if d.is_dir() and any((d / img).exists() for _, img in IMAGES)
    )

    cards = []
    for name in cases:
        imgs = []
        for label, filename in IMAGES:
            path = TESTDATA / name / filename
            if path.exists():
                # Use relative path from the overview.html location
                src = f"{name}/{filename}"
                imgs.append(
                    f'<div class="panel">'
                    f'<div class="label">{label}</div>'
                    f'<img src="{src}" loading="lazy">'
                    f'</div>'
                )
            else:
                imgs.append(
                    f'<div class="panel">'
                    f'<div class="label">{label}</div>'
                    f'<div class="missing">not rendered</div>'
                    f'</div>'
                )

        cards.append(
            f'<section class="case">'
            f'<h2>{name}</h2>'
            f'<div class="images">{"".join(imgs)}</div>'
            f'</section>'
        )

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
{"".join(cards)}
</body>
</html>"""


if __name__ == "__main__":
    html = build_html()
    OUTPUT.write_text(html, encoding="utf-8")
    print(f"Written to {OUTPUT}")
