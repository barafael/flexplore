// AUTO-GENERATED — do not edit. Run `cargo run -p build-overview -- --generate-only` to regenerate.
import SwiftUI

public struct TplGridDashboardView: View {
    public var body: some View {
        // NOTE: CSS Grid approximated with LazyVGrid — tracks map to GridItems, items flow in order; grid-column/grid-row spans and explicit placement are not supported
        LazyVGrid(columns: [GridItem(.flexible()), GridItem(.flexible()), GridItem(.flexible())], spacing: 0.0) {
            // grid-column: 1 / span 3 — not expressible in LazyVGrid; item takes one cell
            Text("header")
                .font(.system(size: 26))
                .foregroundColor(Color(red: 0.05, green: 0.05, blue: 0.1).opacity(0.85))
                .padding(8.0)
                .background(Color(red: 0.98, green: 0.71, blue: 0.68))
            Text("sidebar")
                .font(.system(size: 26))
                .foregroundColor(Color(red: 0.05, green: 0.05, blue: 0.1).opacity(0.85))
                .padding(8.0)
                .background(Color(red: 0.70, green: 0.80, blue: 0.89))
            // grid-column: span 2 — not expressible in LazyVGrid; item takes one cell
            Text("main")
                .font(.system(size: 26))
                .foregroundColor(Color(red: 0.05, green: 0.05, blue: 0.1).opacity(0.85))
                .padding(8.0)
                .background(Color(red: 0.80, green: 0.92, blue: 0.77))
            // grid-column: 1 / span 3 — not expressible in LazyVGrid; item takes one cell
            Text("footer")
                .font(.system(size: 26))
                .foregroundColor(Color(red: 0.05, green: 0.05, blue: 0.1).opacity(0.85))
                .padding(8.0)
                .background(Color(red: 0.87, green: 0.80, blue: 0.89))
        }
        .frame(minWidth: nil, maxWidth: .infinity, minHeight: nil, maxHeight: .infinity, alignment: .topLeading)
        .background(Color(red: 0.11, green: 0.11, blue: 0.17))
    }
}

