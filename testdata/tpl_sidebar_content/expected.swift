struct ContentView: View {
    public var body: some View {
        HStack(alignment: .center, spacing: 0.0) {
            VStack(alignment: .center, spacing: 4.0) {
                Text("nav-1")
                    .font(.system(size: 26))
                    .foregroundColor(Color(red: 0.05, green: 0.05, blue: 0.1).opacity(0.85))
                    .frame(width: nil, height: 44.0)
                    .padding(8.0)
                    .background(Color(red: 0.98, green: 0.71, blue: 0.68))
                    .padding(0.0) /* margin */
                Text("nav-2")
                    .font(.system(size: 26))
                    .foregroundColor(Color(red: 0.05, green: 0.05, blue: 0.1).opacity(0.85))
                    .frame(width: nil, height: 44.0)
                    .padding(8.0)
                    .background(Color(red: 0.70, green: 0.80, blue: 0.89))
                    .padding(0.0) /* margin */
                Text("nav-3")
                    .font(.system(size: 26))
                    .foregroundColor(Color(red: 0.05, green: 0.05, blue: 0.1).opacity(0.85))
                    .frame(width: nil, height: 44.0)
                    .padding(8.0)
                    .background(Color(red: 0.80, green: 0.92, blue: 0.77))
                    .padding(0.0) /* margin */
            }
            .frame(width: 250.0, height: nil)
            .frame(minWidth: nil, maxWidth: nil, minHeight: 0.0, maxHeight: nil)
            .padding(8.0)
            .background(Color(red: 0.11, green: 0.11, blue: 0.17))
            .padding(0.0) /* margin */
            Text("content")
                .font(.system(size: 26))
                .foregroundColor(Color(red: 0.05, green: 0.05, blue: 0.1).opacity(0.85))
                .padding(8.0)
                .background(Color(red: 0.87, green: 0.80, blue: 0.89))
                .padding(0.0) /* margin */
                .layoutPriority(1.0) /* flex-grow */
        }
        .frame(width: 100.0 /* 100.0% — use GeometryReader for relative sizing */, height: 100.0 /* 100.0% — use GeometryReader for relative sizing */)
        .frame(minWidth: nil, maxWidth: nil, minHeight: 0.0, maxHeight: nil)
        .padding(0.0)
        .background(Color(red: 0.11, green: 0.11, blue: 0.17))
        .padding(0.0) /* margin */
    }
}
