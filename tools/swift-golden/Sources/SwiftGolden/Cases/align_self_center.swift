// AUTO-GENERATED — do not edit. Run tool/generate_cases.py to regenerate.
import SwiftUI

public struct AlignSelfCenterView: View {
    var body: some View {
        HStack(alignment: .top, spacing: 8.0) {
            Text("top")
                .font(.system(size: 26))
                .foregroundColor(Color(red: 0.05, green: 0.05, blue: 0.1).opacity(0.85))
                .frame(width: 100.0, height: 60.0)
                .padding(8.0)
                .background(Color(red: 0.98, green: 0.71, blue: 0.68))
                .padding(0.0) /* margin */
            Text("centered")
                .font(.system(size: 26))
                .foregroundColor(Color(red: 0.05, green: 0.05, blue: 0.1).opacity(0.85))
                .frame(width: 120.0, height: 60.0)
                .padding(8.0)
                .background(Color(red: 0.70, green: 0.80, blue: 0.89))
                .padding(0.0) /* margin */
                /* align-self: Center — override manually with .alignmentGuide() */
            Text("top")
                .font(.system(size: 26))
                .foregroundColor(Color(red: 0.05, green: 0.05, blue: 0.1).opacity(0.85))
                .frame(width: 100.0, height: 60.0)
                .padding(8.0)
                .background(Color(red: 0.80, green: 0.92, blue: 0.77))
                .padding(0.0) /* margin */
        }
        .frame(width: 100.0 /* 100.0% — use GeometryReader for relative sizing */, height: 300.0)
        .frame(minWidth: nil, minHeight: 0.0, maxWidth: nil, maxHeight: nil)
        .padding(12.0)
        .background(Color(red: 0.11, green: 0.11, blue: 0.17))
        .padding(0.0) /* margin */
    }
}
