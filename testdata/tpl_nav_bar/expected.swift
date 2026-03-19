struct ContentView: View {
    var body: some View {
        HStack(alignment: .center, spacing: 0.0) {
            // NOTE: justify-content: SpaceBetween — use Spacer() or custom Layout to replicate
            Text("logo")
                .font(.system(size: 26))
                .foregroundColor(Color(red: 0.05, green: 0.05, blue: 0.1).opacity(0.85))
                .frame(width: 48.0, height: 48.0)
                .padding(8.0)
                .padding(0.0) /* margin */
                .background(Color(red: 0.98, green: 0.71, blue: 0.68))
            HStack(alignment: .center, spacing: 8.0) {
                Text("link-1")
                    .font(.system(size: 26))
                    .foregroundColor(Color(red: 0.05, green: 0.05, blue: 0.1).opacity(0.85))
                    .frame(width: 80.0, height: 36.0)
                    .padding(8.0)
                    .padding(0.0) /* margin */
                    .background(Color(red: 0.70, green: 0.80, blue: 0.89))
                Text("link-2")
                    .font(.system(size: 26))
                    .foregroundColor(Color(red: 0.05, green: 0.05, blue: 0.1).opacity(0.85))
                    .frame(width: 80.0, height: 36.0)
                    .padding(8.0)
                    .padding(0.0) /* margin */
                    .background(Color(red: 0.80, green: 0.92, blue: 0.77))
                Text("link-3")
                    .font(.system(size: 26))
                    .foregroundColor(Color(red: 0.05, green: 0.05, blue: 0.1).opacity(0.85))
                    .frame(width: 80.0, height: 36.0)
                    .padding(8.0)
                    .padding(0.0) /* margin */
                    .background(Color(red: 0.87, green: 0.80, blue: 0.89))
            }
            .frame(minWidth: nil, minHeight: 0.0, maxWidth: nil, maxHeight: nil)
            .padding(0.0)
            .padding(0.0) /* margin */
            .background(Color(red: 0.11, green: 0.11, blue: 0.17))
            HStack(alignment: .center, spacing: 8.0) {
                Text("btn-1")
                    .font(.system(size: 26))
                    .foregroundColor(Color(red: 0.05, green: 0.05, blue: 0.1).opacity(0.85))
                    .frame(width: 36.0, height: 36.0)
                    .padding(8.0)
                    .padding(0.0) /* margin */
                    .background(Color(red: 1.00, green: 0.85, blue: 0.65))
                Text("btn-2")
                    .font(.system(size: 26))
                    .foregroundColor(Color(red: 0.05, green: 0.05, blue: 0.1).opacity(0.85))
                    .frame(width: 36.0, height: 36.0)
                    .padding(8.0)
                    .padding(0.0) /* margin */
                    .background(Color(red: 1.00, green: 1.00, blue: 0.80))
            }
            .frame(minWidth: nil, minHeight: 0.0, maxWidth: nil, maxHeight: nil)
            .padding(0.0)
            .padding(0.0) /* margin */
            .background(Color(red: 0.11, green: 0.11, blue: 0.17))
        }
        .frame(width: 100.0 /* 100.0% — use GeometryReader for relative sizing */, height: 56.0)
        .frame(minWidth: nil, minHeight: 0.0, maxWidth: nil, maxHeight: nil)
        .padding(12.0)
        .padding(0.0) /* margin */
        .background(Color(red: 0.11, green: 0.11, blue: 0.17))
    }
}
