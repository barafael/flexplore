struct ContentView: View {
    public var body: some View {
        LazyHGrid(rows: [GridItem(.fixed(80.0)), GridItem(.fixed(80.0))], spacing: 8.0) {
            Text("cell-1")
                .font(.system(size: 26))
                .foregroundColor(Color(red: 0.05, green: 0.05, blue: 0.1).opacity(0.85))
                .padding(8.0)
                .background(Color(red: 0.98, green: 0.71, blue: 0.68))
            Text("cell-2")
                .font(.system(size: 26))
                .foregroundColor(Color(red: 0.05, green: 0.05, blue: 0.1).opacity(0.85))
                .padding(8.0)
                .background(Color(red: 0.70, green: 0.80, blue: 0.89))
            Text("cell-3")
                .font(.system(size: 26))
                .foregroundColor(Color(red: 0.05, green: 0.05, blue: 0.1).opacity(0.85))
                .padding(8.0)
                .background(Color(red: 0.80, green: 0.92, blue: 0.77))
            Text("cell-4")
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
