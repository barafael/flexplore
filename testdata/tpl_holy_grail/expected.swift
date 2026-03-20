struct ContentView: View {
    public var body: some View {
        VStack(alignment: .center, spacing: 0.0) {
            Text("header")
                .font(.system(size: 26))
                .foregroundColor(Color(red: 0.05, green: 0.05, blue: 0.1).opacity(0.85))
                .frame(width: nil, height: 60.0)
                .padding(8.0)
                .background(Color(red: 0.98, green: 0.71, blue: 0.68))
            HStack(alignment: .center, spacing: 0.0) {
                Text("sidebar-left")
                    .font(.system(size: 26))
                    .foregroundColor(Color(red: 0.05, green: 0.05, blue: 0.1).opacity(0.85))
                    .frame(width: 120.0, height: nil)
                    .padding(8.0)
                    .background(Color(red: 0.70, green: 0.80, blue: 0.89))
                Text("content")
                    .font(.system(size: 26))
                    .foregroundColor(Color(red: 0.05, green: 0.05, blue: 0.1).opacity(0.85))
                    .frame(maxWidth: .infinity)
                    .padding(8.0)
                    .background(Color(red: 0.80, green: 0.92, blue: 0.77))
                    .layoutPriority(1.0) /* flex-grow */
                Text("sidebar-right")
                    .font(.system(size: 26))
                    .foregroundColor(Color(red: 0.05, green: 0.05, blue: 0.1).opacity(0.85))
                    .frame(width: 120.0, height: nil)
                    .padding(8.0)
                    .background(Color(red: 0.87, green: 0.80, blue: 0.89))
            }
            .frame(maxHeight: .infinity, alignment: .topLeading)
            .background(Color(red: 0.11, green: 0.11, blue: 0.17))
            Text("footer")
                .font(.system(size: 26))
                .foregroundColor(Color(red: 0.05, green: 0.05, blue: 0.1).opacity(0.85))
                .frame(width: nil, height: 60.0)
                .padding(8.0)
                .background(Color(red: 1.00, green: 0.85, blue: 0.65))
        }
        .frame(minWidth: nil, maxWidth: .infinity, minHeight: nil, maxHeight: .infinity, alignment: .topLeading)
        .background(Color(red: 0.11, green: 0.11, blue: 0.17))
    }
}
