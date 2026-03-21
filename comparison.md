# Flexplore — Competitive Landscape

## Similar Projects

| Project | Type | Interactive Explorer | Multi-Framework Codegen | Targets | Open Source | Platform |
| --- | --- | --- | --- | --- | --- | --- |
| **Flexplore** | Layout explorer + codegen | Yes (tree-based) | Yes (7 targets) | Bevy, HTML/CSS, Tailwind, React, SwiftUI, Flutter, Iced | Yes | Desktop + WASM |
| [Flexy Boxes](https://the-echoplex.net/flexyboxes/) | Flexbox playground | Yes (flat items) | CSS only | HTML/CSS | Yes ([source](https://github.com/nicholascloud/flexyboxes)) | Web |
| [Flexbox.tech](https://flexbox.tech/) | Flexbox playground | Yes (flat items) | CSS only | HTML/CSS | — | Web |
| [CSS Portal Flexbox Generator](https://www.cssportal.com/css-flexbox-generator/) | Flexbox playground | Yes (flat items) | CSS only | HTML/CSS | No | Web |
| [Loading.io Flexbox](https://loading.io/flexbox/) | Flexbox playground | Yes (flat items) | CSS only | HTML/CSS | No | Web |
| [FlexboxGenerator.com](https://flexboxgenerator.com/) | Flexbox playground | Yes (flat items) | CSS only | HTML/CSS | No | Web |
| [Build with Flexbox](https://flexbox.buildwithreact.com/) | Flexbox playground | Yes (flat items) | React only | React | Yes | Web |
| [FigmaToCode](https://github.com/bernaferrari/FigmaToCode) | Design-to-code | No (Figma plugin) | Yes (4 targets) | HTML, Tailwind, Flutter, SwiftUI ([README](https://github.com/bernaferrari/FigmaToCode/blob/main/README.md)) | Yes (MIT) | Figma plugin |
| [Builder.io Visual Copilot](https://www.builder.io/blog/figma-to-code-visual-copilot) | Design-to-code | No (Figma plugin) | Yes (7+ targets) | React, Vue, Angular, Svelte, Qwik, Solid, HTML ([source](https://www.builder.io/blog/visual-copilot-2)) | No (freemium) | Figma plugin |
| [DhiWise](https://www.dhiwise.com/design-converter) | Design-to-code | No (Figma-based) | Yes (6 targets) | React, Next.js, HTML, Flutter, Kotlin, SwiftUI ([source](https://www.dhiwise.com/post/figma-to-code-with-dhiwise)) | No ($149/yr Pro) | Figma plugin + Web |
| [FlutterViz](https://flutterviz.com/) | Visual UI builder | Yes (drag & drop) | Flutter only | Flutter | Yes ([GitHub](https://github.com/iqonic-design/flutter_viz)) | Web |

## Feature Comparison

| Feature | Flexplore | Web Playgrounds | Figma-to-Code Tools |
| --- | --- | --- | --- |
| Real-time visual preview | Yes | Yes | Preview in Figma |
| Nested tree building | Yes | No (flat items only) | Design-based (not flexbox-specific) |
| Multi-framework code export | 7 targets | CSS only | 4–7 targets |
| Undo / redo | Yes (100 snapshots) | Limited or none | Figma history |
| Preset templates | Yes (Holy Grail, Card Grid, Nav Bar, …) | Few or none | N/A |
| Theming (dark/light) | Yes (Catppuccin) | Rare | N/A |
| Procedural art backgrounds | Yes | No | No |
| Golden test infrastructure | Yes (cross-framework visual regression) | No | No |
| Works offline | Yes (desktop binary) | No (requires internet) | No (requires Figma) |
| WASM deployment | Yes | N/A | N/A |
| Hover preview before commit | Yes | Rare | No |
| Free | Yes | Yes (most) | Free tier or paid ([DhiWise pricing](https://www.dhiwise.com/post/dhiwise-figma-to-code), [Builder.io pricing](https://www.builder.io/m/pricing)) |

## Layout Engine Comparison

| Engine | Used By | Language | Flexbox | CSS Grid | Code Generation | Source |
| --- | --- | --- | --- | --- | --- | --- |
| [Taffy](https://github.com/DioxusLabs/taffy) | Flexplore, Bevy, Dioxus | Rust | Yes | Yes | No (layout only) | [crates.io](https://crates.io/crates/taffy), [docs.rs](https://docs.rs/taffy) |
| [Yoga](https://www.yogalayout.dev/) | React Native, Litho | C++ | Yes | [PR open](https://github.com/facebook/yoga/pull/1865) | No (layout only) | [GitHub](https://github.com/facebook/yoga), [Meta blog](https://engineering.fb.com/2016/12/07/android/yoga-a-cross-platform-layout-engine/) |
| [cuicui_layout](https://crates.io/crates/cuicui_layout) | Bevy (alt) | Rust | Custom model | No | No (layout only) | [crates.io](https://crates.io/crates/cuicui_layout) |
| Browser engine | Web playgrounds | C++ | Yes | Yes | N/A | — |

## Sources

- Flexy Boxes: <https://the-echoplex.net/flexyboxes/>
- FigmaToCode: <https://github.com/bernaferrari/FigmaToCode>
- Builder.io Visual Copilot announcement: <https://www.builder.io/blog/figma-to-code-visual-copilot>
- Builder.io Visual Copilot 2.0: <https://www.builder.io/blog/visual-copilot-2>
- DhiWise Figma-to-code: <https://www.dhiwise.com/post/figma-to-code-with-dhiwise>
- FlutterViz GitHub: <https://github.com/iqonic-design/flutter_viz>
- Taffy layout engine: <https://github.com/DioxusLabs/taffy>
- Yoga layout engine: <https://github.com/facebook/yoga>
- Yoga CSS Grid PR: <https://github.com/facebook/yoga/pull/1865>
- Yoga feature request for Grid: <https://github.com/facebook/yoga/issues/867>
- cuicui_layout: <https://crates.io/crates/cuicui_layout>
- Builder.io pricing: <https://www.builder.io/m/pricing>
- DhiWise pricing: <https://www.dhiwise.com/post/dhiwise-figma-to-code>
