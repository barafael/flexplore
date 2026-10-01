// AUTO-GENERATED — do not edit. Run `cargo run -p build-overview -- --generate-only` to regenerate.
import SwiftUI

public struct DirectionColumnView: View {
    public var body: some View {
        FlowLayout(axis: .vertical, spacing: 8.0, lineSpacing: 8.0) {
            Text("A")
                .font(.system(size: 26))
                .foregroundColor(Color(red: 0.05, green: 0.05, blue: 0.1).opacity(0.85))
                .frame(width: 184.0, height: 44.0)
                .padding(8.0)
                .background(Color(red: 0.98, green: 0.71, blue: 0.68))
            Text("B")
                .font(.system(size: 26))
                .foregroundColor(Color(red: 0.05, green: 0.05, blue: 0.1).opacity(0.85))
                .frame(width: 104.0, height: 64.0)
                .padding(8.0)
                .background(Color(red: 0.70, green: 0.80, blue: 0.89))
            Text("C")
                .font(.system(size: 26))
                .foregroundColor(Color(red: 0.05, green: 0.05, blue: 0.1).opacity(0.85))
                .frame(width: 44.0, height: 24.0)
                .padding(8.0)
                .background(Color(red: 0.80, green: 0.92, blue: 0.77))
        }
        .frame(minWidth: nil, maxWidth: .infinity, minHeight: nil, maxHeight: .infinity, alignment: .topLeading)
        .padding(12.0)
        .background(Color(red: 0.11, green: 0.11, blue: 0.17))
    }
}
