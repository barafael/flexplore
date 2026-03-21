struct ContentView: View {
    public var body: some View {
        HStack(alignment: .top, spacing: 8.0) {
            Text("grow-1")
                .font(.system(size: 26))
                .foregroundColor(Color(red: 0.05, green: 0.05, blue: 0.1).opacity(0.85))
                .frame(width: nil, height: 80.0)
                .frame(maxWidth: .infinity)
                .padding(8.0)
                .background(Color(red: 0.98, green: 0.71, blue: 0.68))
            Text("grow-2")
                .font(.system(size: 26))
                .foregroundColor(Color(red: 0.05, green: 0.05, blue: 0.1).opacity(0.85))
                .frame(width: nil, height: 80.0)
                .frame(maxWidth: .infinity)
                .padding(8.0)
                .background(Color(red: 0.70, green: 0.80, blue: 0.89))
            Text("fixed")
                .font(.system(size: 26))
                .foregroundColor(Color(red: 0.05, green: 0.05, blue: 0.1).opacity(0.85))
                .frame(width: 100.0, height: 80.0)
                .padding(8.0)
                .background(Color(red: 0.80, green: 0.92, blue: 0.77))
        }
        .frame(minWidth: nil, maxWidth: .infinity, minHeight: nil, maxHeight: nil, alignment: .topLeading)
        .padding(12.0)
        .background(Color(red: 0.11, green: 0.11, blue: 0.17))
    }
}
