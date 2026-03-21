struct ContentView: View {
    public var body: some View {
        HStack(alignment: .top, spacing: 0.0) {
            VStack(alignment: .leading, spacing: 4.0) {
                Text("nav-1")
                    .font(.system(size: 26))
                    .foregroundColor(Color(red: 0.05, green: 0.05, blue: 0.1).opacity(0.85))
                    .frame(width: nil, height: 44.0)
                    .frame(minWidth: nil, maxWidth: .infinity, minHeight: nil, maxHeight: nil)
                    .padding(8.0)
                    .background(Color(red: 0.98, green: 0.71, blue: 0.68))
                Text("nav-2")
                    .font(.system(size: 26))
                    .foregroundColor(Color(red: 0.05, green: 0.05, blue: 0.1).opacity(0.85))
                    .frame(width: nil, height: 44.0)
                    .frame(minWidth: nil, maxWidth: .infinity, minHeight: nil, maxHeight: nil)
                    .padding(8.0)
                    .background(Color(red: 0.70, green: 0.80, blue: 0.89))
                Text("nav-3")
                    .font(.system(size: 26))
                    .foregroundColor(Color(red: 0.05, green: 0.05, blue: 0.1).opacity(0.85))
                    .frame(width: nil, height: 44.0)
                    .frame(minWidth: nil, maxWidth: .infinity, minHeight: nil, maxHeight: nil)
                    .padding(8.0)
                    .background(Color(red: 0.80, green: 0.92, blue: 0.77))
            }
            .frame(width: 120.0, height: nil, alignment: .topLeading)
            .frame(minWidth: nil, maxWidth: nil, minHeight: nil, maxHeight: .infinity, alignment: .topLeading)
            .padding(8.0)
            .background(Color(red: 0.11, green: 0.11, blue: 0.17))
            Text("content")
                .font(.system(size: 26))
                .foregroundColor(Color(red: 0.05, green: 0.05, blue: 0.1).opacity(0.85))
                .frame(minWidth: nil, maxWidth: .infinity, minHeight: nil, maxHeight: .infinity)
                .padding(8.0)
                .background(Color(red: 0.87, green: 0.80, blue: 0.89))
        }
        .frame(minWidth: nil, maxWidth: .infinity, minHeight: nil, maxHeight: .infinity, alignment: .topLeading)
        .background(Color(red: 0.11, green: 0.11, blue: 0.17))
    }
}
