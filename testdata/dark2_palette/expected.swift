struct ContentView: View {
    var body: some View {
        HStack(alignment: .top, spacing: 8.0) {
            // NOTE: flex-wrap: Wrap — SwiftUI stacks don't wrap; consider a custom Layout
            Text("A")
                .font(.system(size: 26))
                .foregroundColor(Color(red: 0.05, green: 0.05, blue: 0.1).opacity(0.85))
                .frame(width: 80.0, height: 80.0)
                .padding(8.0)
                .background(Color(red: 0.11, green: 0.62, blue: 0.47))
                .padding(0.0) /* margin */
            Text("B")
                .font(.system(size: 26))
                .foregroundColor(Color(red: 0.05, green: 0.05, blue: 0.1).opacity(0.85))
                .frame(width: 80.0, height: 80.0)
                .padding(8.0)
                .background(Color(red: 0.85, green: 0.37, blue: 0.01))
                .padding(0.0) /* margin */
            Text("C")
                .font(.system(size: 26))
                .foregroundColor(Color(red: 0.05, green: 0.05, blue: 0.1).opacity(0.85))
                .frame(width: 80.0, height: 80.0)
                .padding(8.0)
                .background(Color(red: 0.46, green: 0.44, blue: 0.70))
                .padding(0.0) /* margin */
        }
        .frame(width: 100.0 /* 100.0% — use GeometryReader for relative sizing */, height: nil)
        .frame(minWidth: nil, minHeight: 0.0, maxWidth: nil, maxHeight: nil)
        .padding(12.0)
        .background(Color(red: 0.11, green: 0.11, blue: 0.17))
        .padding(0.0) /* margin */
    }
}
