// AUTO-GENERATED — do not edit. Run `cargo run -p build-overview -- --generate-only` to regenerate.
import SwiftUI

public struct Dark2PaletteView: View {
    public var body: some View {
        FlowLayout(axis: .horizontal, spacing: 8.0, lineSpacing: 8.0) {
            Text("A")
                .font(.system(size: 26))
                .foregroundColor(Color(red: 0.05, green: 0.05, blue: 0.1).opacity(0.85))
                .frame(width: 80.0, height: 80.0)
                .padding(8.0)
                .background(Color(red: 0.11, green: 0.62, blue: 0.47))
            Text("B")
                .font(.system(size: 26))
                .foregroundColor(Color(red: 0.05, green: 0.05, blue: 0.1).opacity(0.85))
                .frame(width: 80.0, height: 80.0)
                .padding(8.0)
                .background(Color(red: 0.85, green: 0.37, blue: 0.01))
            Text("C")
                .font(.system(size: 26))
                .foregroundColor(Color(red: 0.05, green: 0.05, blue: 0.1).opacity(0.85))
                .frame(width: 80.0, height: 80.0)
                .padding(8.0)
                .background(Color(red: 0.46, green: 0.44, blue: 0.70))
        }
        .frame(minWidth: nil, maxWidth: .infinity, minHeight: nil, maxHeight: .infinity, alignment: .topLeading)
        .padding(12.0)
        .background(Color(red: 0.11, green: 0.11, blue: 0.17))
    }
}
