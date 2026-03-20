struct ContentView: View {
    public var body: some View {
        VStack(alignment: .leading, spacing: 8.0) {
            // NOTE: flex-wrap: Wrap — SwiftUI stacks don't wrap; consider a custom Layout
            // NOTE: flex-direction: ColumnReverse — children reversed in source to approximate visual order
            Text("C")
                .font(.system(size: 26))
                .foregroundColor(Color(red: 0.05, green: 0.05, blue: 0.1).opacity(0.85))
                .frame(width: 60.0, height: 40.0)
                .padding(8.0)
                .background(Color(red: 0.80, green: 0.92, blue: 0.77))
            Text("B")
                .font(.system(size: 26))
                .foregroundColor(Color(red: 0.05, green: 0.05, blue: 0.1).opacity(0.85))
                .frame(width: 120.0, height: 80.0)
                .padding(8.0)
                .background(Color(red: 0.70, green: 0.80, blue: 0.89))
            Text("A")
                .font(.system(size: 26))
                .foregroundColor(Color(red: 0.05, green: 0.05, blue: 0.1).opacity(0.85))
                .frame(width: 200.0, height: 60.0)
                .padding(8.0)
                .background(Color(red: 0.98, green: 0.71, blue: 0.68))
        }
        .frame(minWidth: nil, maxWidth: .infinity, minHeight: nil, maxHeight: nil, alignment: .topLeading)
        .padding(12.0)
        .background(Color(red: 0.11, green: 0.11, blue: 0.17))
    }
}
