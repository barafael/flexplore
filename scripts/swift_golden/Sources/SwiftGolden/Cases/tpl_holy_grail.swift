// AUTO-GENERATED — do not edit. Run tool/generate_cases.py to regenerate.
import SwiftUI

public struct TplHolyGrailView: View {
    var body: some View {
        VStack(alignment: .center, spacing: 0.0) {
            Text("header")
                .font(.system(size: 26))
                .foregroundColor(Color(red: 0.05, green: 0.05, blue: 0.1).opacity(0.85))
                .frame(width: nil, height: 60.0)
                .padding(8.0)
                .padding(0.0) /* margin */
                .background(Color(red: 0.98, green: 0.71, blue: 0.68))
            HStack(alignment: .center, spacing: 0.0) {
                Text("sidebar-left")
                    .font(.system(size: 26))
                    .foregroundColor(Color(red: 0.05, green: 0.05, blue: 0.1).opacity(0.85))
                    .frame(width: 200.0, height: nil)
                    .padding(8.0)
                    .padding(0.0) /* margin */
                    .background(Color(red: 0.70, green: 0.80, blue: 0.89))
                Text("content")
                    .font(.system(size: 26))
                    .foregroundColor(Color(red: 0.05, green: 0.05, blue: 0.1).opacity(0.85))
                    .padding(8.0)
                    .padding(0.0) /* margin */
                    .layoutPriority(1.0) /* flex-grow */
                    .background(Color(red: 0.80, green: 0.92, blue: 0.77))
                Text("sidebar-right")
                    .font(.system(size: 26))
                    .foregroundColor(Color(red: 0.05, green: 0.05, blue: 0.1).opacity(0.85))
                    .frame(width: 200.0, height: nil)
                    .padding(8.0)
                    .padding(0.0) /* margin */
                    .background(Color(red: 0.87, green: 0.80, blue: 0.89))
            }
            .frame(minWidth: nil, minHeight: 0.0, maxWidth: nil, maxHeight: nil)
            .padding(0.0)
            .padding(0.0) /* margin */
            .background(Color(red: 0.11, green: 0.11, blue: 0.17))
            Text("footer")
                .font(.system(size: 26))
                .foregroundColor(Color(red: 0.05, green: 0.05, blue: 0.1).opacity(0.85))
                .frame(width: nil, height: 60.0)
                .padding(8.0)
                .padding(0.0) /* margin */
                .background(Color(red: 1.00, green: 0.85, blue: 0.65))
        }
        .frame(width: 100.0 /* 100.0% — use GeometryReader for relative sizing */, height: 100.0 /* 100.0% — use GeometryReader for relative sizing */)
        .frame(minWidth: nil, minHeight: 0.0, maxWidth: nil, maxHeight: nil)
        .padding(0.0)
        .padding(0.0) /* margin */
        .background(Color(red: 0.11, green: 0.11, blue: 0.17))
    }
}
