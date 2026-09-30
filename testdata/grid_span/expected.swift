struct ContentView: View {
    public var body: some View {
        LazyVGrid(columns: [GridItem(.flexible()), GridItem(.flexible()), GridItem(.flexible())], spacing: 8.0) {
            Text("wide")
                .font(.system(size: 26))
                .foregroundColor(Color(red: 0.05, green: 0.05, blue: 0.1).opacity(0.85))
                .padding(8.0)
                .background(Color(red: 0.98, green: 0.71, blue: 0.68))
            Text("tall")
                .font(.system(size: 26))
                .foregroundColor(Color(red: 0.05, green: 0.05, blue: 0.1).opacity(0.85))
                .padding(8.0)
                .background(Color(red: 0.70, green: 0.80, blue: 0.89))
            Text("cell")
                .font(.system(size: 26))
                .foregroundColor(Color(red: 0.05, green: 0.05, blue: 0.1).opacity(0.85))
                .padding(8.0)
                .background(Color(red: 0.80, green: 0.92, blue: 0.77))
            Text("cell")
                .font(.system(size: 26))
                .foregroundColor(Color(red: 0.05, green: 0.05, blue: 0.1).opacity(0.85))
                .padding(8.0)
                .background(Color(red: 0.87, green: 0.80, blue: 0.89))
        }
        .frame(minWidth: nil, maxWidth: .infinity, minHeight: nil, maxHeight: .infinity, alignment: .topLeading)
        .padding(12.0)
        .background(Color(red: 0.11, green: 0.11, blue: 0.17))
    }
}
