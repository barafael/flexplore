// AUTO-GENERATED — do not edit. Run `cargo run -p build-overview -- --generate-only` to regenerate.
import SwiftUI

public struct AlignSelfCenterView: View {
    public var body: some View {
        HStack(alignment: .top, spacing: 8.0) {
            Text("top-1")
                .font(.system(size: 26))
                .foregroundColor(Color(red: 0.05, green: 0.05, blue: 0.1).opacity(0.85))
                .frame(width: nil, height: 44.0)
                .frame(minWidth: nil, maxWidth: 84.0, minHeight: nil, maxHeight: nil)
                .padding(8.0)
                .background(Color(red: 0.98, green: 0.71, blue: 0.68))
            Text("centered")
                .font(.system(size: 26))
                .foregroundColor(Color(red: 0.05, green: 0.05, blue: 0.1).opacity(0.85))
                .frame(width: nil, height: 44.0)
                .frame(minWidth: nil, maxWidth: 104.0, minHeight: nil, maxHeight: nil)
                .padding(8.0)
                .background(Color(red: 0.70, green: 0.80, blue: 0.89))
                .frame(maxHeight: .infinity, alignment: .center)
            Text("top-2")
                .font(.system(size: 26))
                .foregroundColor(Color(red: 0.05, green: 0.05, blue: 0.1).opacity(0.85))
                .frame(width: nil, height: 44.0)
                .frame(minWidth: nil, maxWidth: 84.0, minHeight: nil, maxHeight: nil)
                .padding(8.0)
                .background(Color(red: 0.80, green: 0.92, blue: 0.77))
        }
        .frame(minWidth: nil, maxWidth: .infinity, minHeight: nil, maxHeight: .infinity, alignment: .topLeading)
        .padding(12.0)
        .background(Color(red: 0.11, green: 0.11, blue: 0.17))
    }
}

