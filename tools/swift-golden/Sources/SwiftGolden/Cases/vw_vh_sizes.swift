// AUTO-GENERATED — do not edit. Run `cargo run -p build-overview -- --generate-only` to regenerate.
import SwiftUI

public struct VwVhSizesView: View {
    public var body: some View {
        FlowLayout(axis: .vertical, spacing: 8.0, lineSpacing: 8.0) {
            Text("50vw x 20vh")
                .font(.system(size: 26))
                .foregroundColor(Color(red: 0.05, green: 0.05, blue: 0.1).opacity(0.85))
                .frame(width: 200.0, height: 60.0)
                .padding(8.0)
                .background(Color(red: 0.98, green: 0.71, blue: 0.68))
            Text("75vw x 30vh")
                .font(.system(size: 26))
                .foregroundColor(Color(red: 0.05, green: 0.05, blue: 0.1).opacity(0.85))
                .frame(width: 300.0, height: 90.0)
                .padding(8.0)
                .background(Color(red: 0.70, green: 0.80, blue: 0.89))
        }
        .frame(minWidth: nil, maxWidth: .infinity, minHeight: nil, maxHeight: .infinity, alignment: .topLeading)
        .padding(12.0)
        .background(Color(red: 0.11, green: 0.11, blue: 0.17))
    }
}
