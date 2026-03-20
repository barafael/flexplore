struct ContentView: View {
    var body: some View {
        HStack(alignment: .top, spacing: 8.0) {
            Text("A")
                .font(.system(size: 26))
                .foregroundColor(Color(red: 0.05, green: 0.05, blue: 0.1).opacity(0.85))
                .frame(width: 200.0, height: 80.0)
                .padding(8.0)
                .padding(0.0) /* margin */
                .background(Color(red: 0.98, green: 0.71, blue: 0.68))
            Text("B")
                .font(.system(size: 26))
                .foregroundColor(Color(red: 0.05, green: 0.05, blue: 0.1).opacity(0.85))
                .frame(width: 200.0, height: 80.0)
                .padding(8.0)
                .padding(0.0) /* margin */
                .background(Color(red: 0.70, green: 0.80, blue: 0.89))
            Text("C")
                .font(.system(size: 26))
                .foregroundColor(Color(red: 0.05, green: 0.05, blue: 0.1).opacity(0.85))
                .frame(width: 200.0, height: 80.0)
                .padding(8.0)
                .padding(0.0) /* margin */
                .background(Color(red: 0.80, green: 0.92, blue: 0.77))
            Text("D")
                .font(.system(size: 26))
                .foregroundColor(Color(red: 0.05, green: 0.05, blue: 0.1).opacity(0.85))
                .frame(width: 200.0, height: 80.0)
                .padding(8.0)
                .padding(0.0) /* margin */
                .background(Color(red: 0.87, green: 0.80, blue: 0.89))
            Text("E")
                .font(.system(size: 26))
                .foregroundColor(Color(red: 0.05, green: 0.05, blue: 0.1).opacity(0.85))
                .frame(width: 200.0, height: 80.0)
                .padding(8.0)
                .padding(0.0) /* margin */
                .background(Color(red: 1.00, green: 0.85, blue: 0.65))
            Text("F")
                .font(.system(size: 26))
                .foregroundColor(Color(red: 0.05, green: 0.05, blue: 0.1).opacity(0.85))
                .frame(width: 200.0, height: 80.0)
                .padding(8.0)
                .padding(0.0) /* margin */
                .background(Color(red: 1.00, green: 1.00, blue: 0.80))
        }
        .frame(width: 100.0 /* 100.0% — use GeometryReader for relative sizing */, height: nil)
        .frame(minWidth: nil, minHeight: 0.0, maxWidth: nil, maxHeight: nil)
        .padding(12.0)
        .padding(0.0) /* margin */
        .background(Color(red: 0.11, green: 0.11, blue: 0.17))
    }
}
