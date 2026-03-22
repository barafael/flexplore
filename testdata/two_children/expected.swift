struct ContentView: View {
    public var body: some View {
        FlowLayout(axis: .horizontal, spacing: 8.0, lineSpacing: 8.0) {
            Text("A")
                .font(.system(size: 26))
                .foregroundColor(Color(red: 0.05, green: 0.05, blue: 0.1).opacity(0.85))
                .frame(width: 80.0, height: 80.0)
                .padding(8.0)
                .background(Color(red: 0.98, green: 0.71, blue: 0.68))
            Text("B")
                .font(.system(size: 26))
                .foregroundColor(Color(red: 0.05, green: 0.05, blue: 0.1).opacity(0.85))
                .frame(width: 120.0, height: 100.0)
                .padding(8.0)
                .background(Color(red: 0.70, green: 0.80, blue: 0.89))
        }
        .frame(minWidth: nil, maxWidth: .infinity, minHeight: nil, maxHeight: .infinity, alignment: .topLeading)
        .padding(12.0)
        .background(Color(red: 0.11, green: 0.11, blue: 0.17))
    }
}

struct FlowLayout: Layout {
    var axis: Axis = .horizontal
    var spacing: CGFloat = 0
    var lineSpacing: CGFloat = 0
    var lineAlignment: LineAlignment = .start
    var mainReversed: Bool = false
    var reversed: Bool = false

    enum LineAlignment: Sendable {
        case start, center, end, spaceBetween, spaceAround, spaceEvenly
    }

    private struct FlowLine {
        var range: Range<Int>
        var mainLength: CGFloat
        var crossLength: CGFloat
    }

    private func mainLength(_ s: CGSize) -> CGFloat {
        axis == .horizontal ? s.width : s.height
    }

    private func crossLength(_ s: CGSize) -> CGFloat {
        axis == .horizontal ? s.height : s.width
    }

    private func breakLines(sizes: [CGSize], maxMain: CGFloat) -> [FlowLine] {
        var lines: [FlowLine] = []
        var start = 0, main: CGFloat = 0, cross: CGFloat = 0
        for (i, size) in sizes.enumerated() {
            let m = mainLength(size)
            if main + m > maxMain && main > 0 {
                lines.append(FlowLine(range: start..<i, mainLength: main - spacing, crossLength: cross))
                start = i; main = 0; cross = 0
            }
            main += m + spacing
            cross = max(cross, crossLength(size))
        }
        if start < sizes.count {
            lines.append(FlowLine(range: start..<sizes.count, mainLength: main - spacing, crossLength: cross))
        }
        return lines
    }

    func sizeThatFits(proposal: ProposedViewSize, subviews: Subviews, cache: inout ()) -> CGSize {
        let sizes = subviews.map { $0.sizeThatFits(.unspecified) }
        let maxMain = axis == .horizontal ? (proposal.width ?? .infinity) : (proposal.height ?? .infinity)
        let lines = breakLines(sizes: sizes, maxMain: maxMain)
        let mainMax = lines.map(\.mainLength).max() ?? 0
        let crossTotal = lines.map(\.crossLength).reduce(0, +)
            + CGFloat(max(lines.count - 1, 0)) * lineSpacing
        return axis == .horizontal
            ? CGSize(width: mainMax, height: crossTotal)
            : CGSize(width: crossTotal, height: mainMax)
    }

    func placeSubviews(in bounds: CGRect, proposal: ProposedViewSize, subviews: Subviews, cache: inout ()) {
        let sizes = subviews.map { $0.sizeThatFits(.unspecified) }
        let maxMain = axis == .horizontal ? bounds.width : bounds.height
        let maxCross = axis == .horizontal ? bounds.height : bounds.width
        var lines = breakLines(sizes: sizes, maxMain: maxMain)
        if reversed { lines.reverse() }

        let totalCross = lines.map(\.crossLength).reduce(0, +)
        let remaining = maxCross - totalCross
        let n = CGFloat(lines.count)
        var crossStart: CGFloat = 0
        var gap = lineSpacing

        switch lineAlignment {
        case .start: break
        case .center:
            crossStart = (remaining - CGFloat(max(lines.count - 1, 0)) * lineSpacing) / 2
        case .end:
            crossStart = remaining - CGFloat(max(lines.count - 1, 0)) * lineSpacing
        case .spaceBetween:
            gap = n > 1 ? remaining / (n - 1) : 0
        case .spaceAround:
            gap = n > 0 ? remaining / n : 0
            crossStart = gap / 2
        case .spaceEvenly:
            gap = n > 0 ? remaining / (n + 1) : 0
            crossStart = gap
        }

        var cross = crossStart
        for line in lines {
            var main: CGFloat = mainReversed ? maxMain : 0
            for idx in line.range {
                if mainReversed { main -= mainLength(sizes[idx]) }
                let pt = axis == .horizontal
                    ? CGPoint(x: bounds.minX + main, y: bounds.minY + cross)
                    : CGPoint(x: bounds.minX + cross, y: bounds.minY + main)
                subviews[idx].place(at: pt, proposal: .unspecified)
                if mainReversed {
                    main -= spacing
                } else {
                    main += mainLength(sizes[idx]) + spacing
                }
            }
            cross += line.crossLength + gap
        }
    }
}
