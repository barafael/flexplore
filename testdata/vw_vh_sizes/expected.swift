struct ContentView: View {
    var body: some View {
        VStack(alignment: .leading, spacing: 8.0) {
            // NOTE: flex-wrap: Wrap — SwiftUI stacks don't wrap; consider a custom Layout
            Text("50vw x 20vh")
                .font(.system(size: 26))
                .foregroundColor(Color(red: 0.05, green: 0.05, blue: 0.1).opacity(0.85))
                .frame(width: UIScreen.main.bounds.width * 0.500, height: UIScreen.main.bounds.height * 0.200)
                .padding(8.0)
                .padding(0.0) /* margin */
                .background(Color(red: 0.98, green: 0.71, blue: 0.68))
            Text("75vw x 30vh")
                .font(.system(size: 26))
                .foregroundColor(Color(red: 0.05, green: 0.05, blue: 0.1).opacity(0.85))
                .frame(width: UIScreen.main.bounds.width * 0.750, height: UIScreen.main.bounds.height * 0.300)
                .padding(8.0)
                .padding(0.0) /* margin */
                .background(Color(red: 0.70, green: 0.80, blue: 0.89))
        }
        .frame(width: 100.0 /* 100.0% — use GeometryReader for relative sizing */, height: nil)
        .frame(minWidth: nil, minHeight: 0.0, maxWidth: nil, maxHeight: nil)
        .padding(12.0)
        .padding(0.0) /* margin */
        .background(Color(red: 0.11, green: 0.11, blue: 0.17))
    }
}
