// AUTO-GENERATED — do not edit. Run tool/generate_cases.py to regenerate.
import XCTest
import SwiftUI
import AppKit
import SnapshotTesting

@testable import SwiftGolden

final class GoldenTests: XCTestCase {
    override func invokeTest() {
        // When SWIFT_SNAPSHOT_RECORD=1, always (re-)generate golden PNGs.
        if ProcessInfo.processInfo.environment["SWIFT_SNAPSHOT_RECORD"] == "1" {
            withSnapshotTesting(record: .all) {
                super.invokeTest()
            }
        } else {
            super.invokeTest()
        }
    }

    func test_align_content_space_between() {
        let view = NSHostingController(rootView:
            AlignContentSpaceBetweenView()
                .frame(width: 400, height: 300)
                .background(Color(red: 0.11, green: 0.11, blue: 0.18))
        )
        assertSnapshot(of: view, as: .image(size: CGSize(width: 400, height: 300)))
    }

    func test_align_items_center() {
        let view = NSHostingController(rootView:
            AlignItemsCenterView()
                .frame(width: 400, height: 300)
                .background(Color(red: 0.11, green: 0.11, blue: 0.18))
        )
        assertSnapshot(of: view, as: .image(size: CGSize(width: 400, height: 300)))
    }

    func test_align_items_stretch() {
        let view = NSHostingController(rootView:
            AlignItemsStretchView()
                .frame(width: 400, height: 300)
                .background(Color(red: 0.11, green: 0.11, blue: 0.18))
        )
        assertSnapshot(of: view, as: .image(size: CGSize(width: 400, height: 300)))
    }

    func test_align_self_center() {
        let view = NSHostingController(rootView:
            AlignSelfCenterView()
                .frame(width: 400, height: 300)
                .background(Color(red: 0.11, green: 0.11, blue: 0.18))
        )
        assertSnapshot(of: view, as: .image(size: CGSize(width: 400, height: 300)))
    }

    func test_all_hidden() {
        let view = NSHostingController(rootView:
            AllHiddenView()
                .frame(width: 400, height: 300)
                .background(Color(red: 0.11, green: 0.11, blue: 0.18))
        )
        assertSnapshot(of: view, as: .image(size: CGSize(width: 400, height: 300)))
    }

    func test_dark2_palette() {
        let view = NSHostingController(rootView:
            Dark2PaletteView()
                .frame(width: 400, height: 300)
                .background(Color(red: 0.11, green: 0.11, blue: 0.18))
        )
        assertSnapshot(of: view, as: .image(size: CGSize(width: 400, height: 300)))
    }

    func test_deep_chain_3() {
        let view = NSHostingController(rootView:
            DeepChain3View()
                .frame(width: 400, height: 300)
                .background(Color(red: 0.11, green: 0.11, blue: 0.18))
        )
        assertSnapshot(of: view, as: .image(size: CGSize(width: 400, height: 300)))
    }

    func test_direction_column() {
        let view = NSHostingController(rootView:
            DirectionColumnView()
                .frame(width: 400, height: 300)
                .background(Color(red: 0.11, green: 0.11, blue: 0.18))
        )
        assertSnapshot(of: view, as: .image(size: CGSize(width: 400, height: 300)))
    }

    func test_direction_column_reverse() {
        let view = NSHostingController(rootView:
            DirectionColumnReverseView()
                .frame(width: 400, height: 300)
                .background(Color(red: 0.11, green: 0.11, blue: 0.18))
        )
        assertSnapshot(of: view, as: .image(size: CGSize(width: 400, height: 300)))
    }

    func test_direction_row_reverse() {
        let view = NSHostingController(rootView:
            DirectionRowReverseView()
                .frame(width: 400, height: 300)
                .background(Color(red: 0.11, green: 0.11, blue: 0.18))
        )
        assertSnapshot(of: view, as: .image(size: CGSize(width: 400, height: 300)))
    }

    func test_flex_basis_percent() {
        let view = NSHostingController(rootView:
            FlexBasisPercentView()
                .frame(width: 400, height: 300)
                .background(Color(red: 0.11, green: 0.11, blue: 0.18))
        )
        assertSnapshot(of: view, as: .image(size: CGSize(width: 400, height: 300)))
    }

    func test_gaps_mixed() {
        let view = NSHostingController(rootView:
            GapsMixedView()
                .frame(width: 400, height: 300)
                .background(Color(red: 0.11, green: 0.11, blue: 0.18))
        )
        assertSnapshot(of: view, as: .image(size: CGSize(width: 400, height: 300)))
    }

    func test_grow_shrink() {
        let view = NSHostingController(rootView:
            GrowShrinkView()
                .frame(width: 400, height: 300)
                .background(Color(red: 0.11, green: 0.11, blue: 0.18))
        )
        assertSnapshot(of: view, as: .image(size: CGSize(width: 400, height: 300)))
    }

    func test_hidden_child() {
        let view = NSHostingController(rootView:
            HiddenChildView()
                .frame(width: 400, height: 300)
                .background(Color(red: 0.11, green: 0.11, blue: 0.18))
        )
        assertSnapshot(of: view, as: .image(size: CGSize(width: 400, height: 300)))
    }

    func test_justify_center() {
        let view = NSHostingController(rootView:
            JustifyCenterView()
                .frame(width: 400, height: 300)
                .background(Color(red: 0.11, green: 0.11, blue: 0.18))
        )
        assertSnapshot(of: view, as: .image(size: CGSize(width: 400, height: 300)))
    }

    func test_justify_space_between() {
        let view = NSHostingController(rootView:
            JustifySpaceBetweenView()
                .frame(width: 400, height: 300)
                .background(Color(red: 0.11, green: 0.11, blue: 0.18))
        )
        assertSnapshot(of: view, as: .image(size: CGSize(width: 400, height: 300)))
    }

    func test_justify_space_evenly() {
        let view = NSHostingController(rootView:
            JustifySpaceEvenlyView()
                .frame(width: 400, height: 300)
                .background(Color(red: 0.11, green: 0.11, blue: 0.18))
        )
        assertSnapshot(of: view, as: .image(size: CGSize(width: 400, height: 300)))
    }

    func test_min_max_sizes() {
        let view = NSHostingController(rootView:
            MinMaxSizesView()
                .frame(width: 400, height: 300)
                .background(Color(red: 0.11, green: 0.11, blue: 0.18))
        )
        assertSnapshot(of: view, as: .image(size: CGSize(width: 400, height: 300)))
    }

    func test_nested_mixed() {
        let view = NSHostingController(rootView:
            NestedMixedView()
                .frame(width: 400, height: 300)
                .background(Color(red: 0.11, green: 0.11, blue: 0.18))
        )
        assertSnapshot(of: view, as: .image(size: CGSize(width: 400, height: 300)))
    }

    func test_ordered_children() {
        let view = NSHostingController(rootView:
            OrderedChildrenView()
                .frame(width: 400, height: 300)
                .background(Color(red: 0.11, green: 0.11, blue: 0.18))
        )
        assertSnapshot(of: view, as: .image(size: CGSize(width: 400, height: 300)))
    }

    func test_padding_margin() {
        let view = NSHostingController(rootView:
            PaddingMarginView()
                .frame(width: 400, height: 300)
                .background(Color(red: 0.11, green: 0.11, blue: 0.18))
        )
        assertSnapshot(of: view, as: .image(size: CGSize(width: 400, height: 300)))
    }

    func test_single_leaf() {
        let view = NSHostingController(rootView:
            SingleLeafView()
                .frame(width: 400, height: 300)
                .background(Color(red: 0.11, green: 0.11, blue: 0.18))
        )
        assertSnapshot(of: view, as: .image(size: CGSize(width: 400, height: 300)))
    }

    func test_tpl_card_grid() {
        let view = NSHostingController(rootView:
            TplCardGridView()
                .frame(width: 400, height: 300)
                .background(Color(red: 0.11, green: 0.11, blue: 0.18))
        )
        assertSnapshot(of: view, as: .image(size: CGSize(width: 400, height: 300)))
    }

    func test_tpl_holy_grail() {
        let view = NSHostingController(rootView:
            TplHolyGrailView()
                .frame(width: 400, height: 300)
                .background(Color(red: 0.11, green: 0.11, blue: 0.18))
        )
        assertSnapshot(of: view, as: .image(size: CGSize(width: 400, height: 300)))
    }

    func test_tpl_nav_bar() {
        let view = NSHostingController(rootView:
            TplNavBarView()
                .frame(width: 400, height: 300)
                .background(Color(red: 0.11, green: 0.11, blue: 0.18))
        )
        assertSnapshot(of: view, as: .image(size: CGSize(width: 400, height: 300)))
    }

    func test_tpl_sidebar_content() {
        let view = NSHostingController(rootView:
            TplSidebarContentView()
                .frame(width: 400, height: 300)
                .background(Color(red: 0.11, green: 0.11, blue: 0.18))
        )
        assertSnapshot(of: view, as: .image(size: CGSize(width: 400, height: 300)))
    }

    func test_two_children() {
        let view = NSHostingController(rootView:
            TwoChildrenView()
                .frame(width: 400, height: 300)
                .background(Color(red: 0.11, green: 0.11, blue: 0.18))
        )
        assertSnapshot(of: view, as: .image(size: CGSize(width: 400, height: 300)))
    }

    func test_vw_vh_sizes() {
        let view = NSHostingController(rootView:
            VwVhSizesView()
                .frame(width: 400, height: 300)
                .background(Color(red: 0.11, green: 0.11, blue: 0.18))
        )
        assertSnapshot(of: view, as: .image(size: CGSize(width: 400, height: 300)))
    }

    func test_wide_flat_5() {
        let view = NSHostingController(rootView:
            WideFlat5View()
                .frame(width: 400, height: 300)
                .background(Color(red: 0.11, green: 0.11, blue: 0.18))
        )
        assertSnapshot(of: view, as: .image(size: CGSize(width: 400, height: 300)))
    }

    func test_wrap_nowrap() {
        let view = NSHostingController(rootView:
            WrapNowrapView()
                .frame(width: 400, height: 300)
                .background(Color(red: 0.11, green: 0.11, blue: 0.18))
        )
        assertSnapshot(of: view, as: .image(size: CGSize(width: 400, height: 300)))
    }

    func test_wrap_reverse() {
        let view = NSHostingController(rootView:
            WrapReverseView()
                .frame(width: 400, height: 300)
                .background(Color(red: 0.11, green: 0.11, blue: 0.18))
        )
        assertSnapshot(of: view, as: .image(size: CGSize(width: 400, height: 300)))
    }
}
