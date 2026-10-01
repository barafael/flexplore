// AUTO-GENERATED — do not edit. Run `cargo run -p build-overview -- --generate-only` to regenerate.
import SwiftUI

public struct DeepChain3View: View {
    public var body: some View {
        FlowLayout(axis: .horizontal, spacing: 8.0, lineSpacing: 8.0) {
            FlowLayout(axis: .horizontal, spacing: 8.0, lineSpacing: 8.0) {
                FlowLayout(axis: .horizontal, spacing: 8.0, lineSpacing: 8.0) {
                    Text("leaf")
                        .font(.system(size: 26))
                        .foregroundColor(Color(red: 0.05, green: 0.05, blue: 0.1).opacity(0.85))
                        .frame(width: 34.0, height: 34.0)
                        .padding(8.0)
                        .background(Color(red: 0.98, green: 0.71, blue: 0.68))
                }
                .frame(minWidth: nil, maxWidth: .infinity, minHeight: nil, maxHeight: nil, alignment: .topLeading)
                .padding(12.0)
                .background(Color(red: 0.11, green: 0.11, blue: 0.17))
            }
            .frame(minWidth: nil, maxWidth: .infinity, minHeight: nil, maxHeight: nil, alignment: .topLeading)
            .padding(12.0)
            .background(Color(red: 0.11, green: 0.11, blue: 0.17))
        }
        .frame(minWidth: nil, maxWidth: .infinity, minHeight: nil, maxHeight: .infinity, alignment: .topLeading)
        .padding(12.0)
        .background(Color(red: 0.11, green: 0.11, blue: 0.17))
    }
}
