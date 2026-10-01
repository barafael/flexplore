struct ContentView: View {
    public var body: some View {
        VStack(alignment: .center, spacing: 8.0) {
            Text("A")
                .font(.system(size: 26))
                .foregroundColor(Color(red: 0.05, green: 0.05, blue: 0.1).opacity(0.85))
                .frame(width: 184.0, height: nil)
                .frame(minWidth: nil, maxWidth: nil, minHeight: nil, maxHeight: 34.0)
                .padding(8.0)
                .background(Color(red: 0.98, green: 0.71, blue: 0.68))
            Text("B")
                .font(.system(size: 26))
                .foregroundColor(Color(red: 0.05, green: 0.05, blue: 0.1).opacity(0.85))
                .frame(width: 104.0, height: nil)
                .frame(minWidth: nil, maxWidth: nil, minHeight: nil, maxHeight: 34.0)
                .padding(8.0)
                .background(Color(red: 0.70, green: 0.80, blue: 0.89))
            Text("C")
                .font(.system(size: 26))
                .foregroundColor(Color(red: 0.05, green: 0.05, blue: 0.1).opacity(0.85))
                .frame(width: 64.0, height: nil)
                .frame(minWidth: nil, maxWidth: nil, minHeight: nil, maxHeight: 34.0)
                .padding(8.0)
                .background(Color(red: 0.80, green: 0.92, blue: 0.77))
        }
        .frame(minWidth: nil, maxWidth: .infinity, minHeight: nil, maxHeight: .infinity, alignment: .top)
        .padding(12.0)
        .background(Color(red: 0.11, green: 0.11, blue: 0.17))
    }
}
