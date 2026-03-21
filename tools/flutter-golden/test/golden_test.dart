// AUTO-GENERATED — do not edit. Run `cargo run -p build-overview` to regenerate.
import 'dart:io';
import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:flutter_golden/cases/align_content_space_between.dart';
import 'package:flutter_golden/cases/align_items_center.dart';
import 'package:flutter_golden/cases/align_items_stretch.dart';
import 'package:flutter_golden/cases/align_self_center.dart';
import 'package:flutter_golden/cases/all_hidden.dart';
import 'package:flutter_golden/cases/dark2_palette.dart';
import 'package:flutter_golden/cases/deep_chain_3.dart';
import 'package:flutter_golden/cases/direction_column.dart';
import 'package:flutter_golden/cases/direction_column_reverse.dart';
import 'package:flutter_golden/cases/direction_row_reverse.dart';
import 'package:flutter_golden/cases/flex_basis_percent.dart';
import 'package:flutter_golden/cases/gaps_mixed.dart';
import 'package:flutter_golden/cases/grow_shrink.dart';
import 'package:flutter_golden/cases/hidden_child.dart';
import 'package:flutter_golden/cases/justify_center.dart';
import 'package:flutter_golden/cases/justify_space_between.dart';
import 'package:flutter_golden/cases/justify_space_evenly.dart';
import 'package:flutter_golden/cases/min_max_sizes.dart';
import 'package:flutter_golden/cases/nested_mixed.dart';
import 'package:flutter_golden/cases/ordered_children.dart';
import 'package:flutter_golden/cases/padding_margin.dart';
import 'package:flutter_golden/cases/single_leaf.dart';
import 'package:flutter_golden/cases/tpl_card_grid.dart';
import 'package:flutter_golden/cases/tpl_holy_grail.dart';
import 'package:flutter_golden/cases/tpl_nav_bar.dart';
import 'package:flutter_golden/cases/tpl_sidebar_content.dart';
import 'package:flutter_golden/cases/two_children.dart';
import 'package:flutter_golden/cases/vw_vh_sizes.dart';
import 'package:flutter_golden/cases/wide_flat_5.dart';
import 'package:flutter_golden/cases/wrap_nowrap.dart';
import 'package:flutter_golden/cases/wrap_reverse.dart';

void main() {
  setUpAll(() async {
    // Load Roboto so golden tests render real text instead of placeholder blocks.
    final fontFile = File('fonts/Roboto-Regular.ttf');
    if (fontFile.existsSync()) {
      await ui.loadFontFromList(fontFile.readAsBytesSync(), fontFamily: 'Roboto');
    }
  });

  testWidgets('align_content_space_between', (tester) async {
    tester.view.devicePixelRatio = 1.0;
    addTearDown(() => tester.view.resetDevicePixelRatio());
    await tester.binding.setSurfaceSize(const Size(400, 300));
    await tester.pumpWidget(
      MaterialApp(
        debugShowCheckedModeBanner: false,
        home: Scaffold(
          backgroundColor: const Color(0xFF1C1C2E),
          body: AlignContentSpaceBetween(),
        ),
      ),
    );
    // Consume any overflow errors so the golden is still captured.
    tester.takeException();
    await expectLater(
      find.byType(MaterialApp),
      matchesGoldenFile('goldens/align_content_space_between.png'),
    );
  });

  testWidgets('align_items_center', (tester) async {
    tester.view.devicePixelRatio = 1.0;
    addTearDown(() => tester.view.resetDevicePixelRatio());
    await tester.binding.setSurfaceSize(const Size(400, 300));
    await tester.pumpWidget(
      MaterialApp(
        debugShowCheckedModeBanner: false,
        home: Scaffold(
          backgroundColor: const Color(0xFF1C1C2E),
          body: AlignItemsCenter(),
        ),
      ),
    );
    // Consume any overflow errors so the golden is still captured.
    tester.takeException();
    await expectLater(
      find.byType(MaterialApp),
      matchesGoldenFile('goldens/align_items_center.png'),
    );
  });

  testWidgets('align_items_stretch', (tester) async {
    tester.view.devicePixelRatio = 1.0;
    addTearDown(() => tester.view.resetDevicePixelRatio());
    await tester.binding.setSurfaceSize(const Size(400, 300));
    await tester.pumpWidget(
      MaterialApp(
        debugShowCheckedModeBanner: false,
        home: Scaffold(
          backgroundColor: const Color(0xFF1C1C2E),
          body: AlignItemsStretch(),
        ),
      ),
    );
    // Consume any overflow errors so the golden is still captured.
    tester.takeException();
    await expectLater(
      find.byType(MaterialApp),
      matchesGoldenFile('goldens/align_items_stretch.png'),
    );
  });

  testWidgets('align_self_center', (tester) async {
    tester.view.devicePixelRatio = 1.0;
    addTearDown(() => tester.view.resetDevicePixelRatio());
    await tester.binding.setSurfaceSize(const Size(400, 300));
    await tester.pumpWidget(
      MaterialApp(
        debugShowCheckedModeBanner: false,
        home: Scaffold(
          backgroundColor: const Color(0xFF1C1C2E),
          body: AlignSelfCenter(),
        ),
      ),
    );
    // Consume any overflow errors so the golden is still captured.
    tester.takeException();
    await expectLater(
      find.byType(MaterialApp),
      matchesGoldenFile('goldens/align_self_center.png'),
    );
  });

  testWidgets('all_hidden', (tester) async {
    tester.view.devicePixelRatio = 1.0;
    addTearDown(() => tester.view.resetDevicePixelRatio());
    await tester.binding.setSurfaceSize(const Size(400, 300));
    await tester.pumpWidget(
      MaterialApp(
        debugShowCheckedModeBanner: false,
        home: Scaffold(
          backgroundColor: const Color(0xFF1C1C2E),
          body: AllHidden(),
        ),
      ),
    );
    // Consume any overflow errors so the golden is still captured.
    tester.takeException();
    await expectLater(
      find.byType(MaterialApp),
      matchesGoldenFile('goldens/all_hidden.png'),
    );
  });

  testWidgets('dark2_palette', (tester) async {
    tester.view.devicePixelRatio = 1.0;
    addTearDown(() => tester.view.resetDevicePixelRatio());
    await tester.binding.setSurfaceSize(const Size(400, 300));
    await tester.pumpWidget(
      MaterialApp(
        debugShowCheckedModeBanner: false,
        home: Scaffold(
          backgroundColor: const Color(0xFF1C1C2E),
          body: Dark2Palette(),
        ),
      ),
    );
    // Consume any overflow errors so the golden is still captured.
    tester.takeException();
    await expectLater(
      find.byType(MaterialApp),
      matchesGoldenFile('goldens/dark2_palette.png'),
    );
  });

  testWidgets('deep_chain_3', (tester) async {
    tester.view.devicePixelRatio = 1.0;
    addTearDown(() => tester.view.resetDevicePixelRatio());
    await tester.binding.setSurfaceSize(const Size(400, 300));
    await tester.pumpWidget(
      MaterialApp(
        debugShowCheckedModeBanner: false,
        home: Scaffold(
          backgroundColor: const Color(0xFF1C1C2E),
          body: DeepChain3(),
        ),
      ),
    );
    // Consume any overflow errors so the golden is still captured.
    tester.takeException();
    await expectLater(
      find.byType(MaterialApp),
      matchesGoldenFile('goldens/deep_chain_3.png'),
    );
  });

  testWidgets('direction_column', (tester) async {
    tester.view.devicePixelRatio = 1.0;
    addTearDown(() => tester.view.resetDevicePixelRatio());
    await tester.binding.setSurfaceSize(const Size(400, 300));
    await tester.pumpWidget(
      MaterialApp(
        debugShowCheckedModeBanner: false,
        home: Scaffold(
          backgroundColor: const Color(0xFF1C1C2E),
          body: DirectionColumn(),
        ),
      ),
    );
    // Consume any overflow errors so the golden is still captured.
    tester.takeException();
    await expectLater(
      find.byType(MaterialApp),
      matchesGoldenFile('goldens/direction_column.png'),
    );
  });

  testWidgets('direction_column_reverse', (tester) async {
    tester.view.devicePixelRatio = 1.0;
    addTearDown(() => tester.view.resetDevicePixelRatio());
    await tester.binding.setSurfaceSize(const Size(400, 300));
    await tester.pumpWidget(
      MaterialApp(
        debugShowCheckedModeBanner: false,
        home: Scaffold(
          backgroundColor: const Color(0xFF1C1C2E),
          body: DirectionColumnReverse(),
        ),
      ),
    );
    // Consume any overflow errors so the golden is still captured.
    tester.takeException();
    await expectLater(
      find.byType(MaterialApp),
      matchesGoldenFile('goldens/direction_column_reverse.png'),
    );
  });

  testWidgets('direction_row_reverse', (tester) async {
    tester.view.devicePixelRatio = 1.0;
    addTearDown(() => tester.view.resetDevicePixelRatio());
    await tester.binding.setSurfaceSize(const Size(400, 300));
    await tester.pumpWidget(
      MaterialApp(
        debugShowCheckedModeBanner: false,
        home: Scaffold(
          backgroundColor: const Color(0xFF1C1C2E),
          body: DirectionRowReverse(),
        ),
      ),
    );
    // Consume any overflow errors so the golden is still captured.
    tester.takeException();
    await expectLater(
      find.byType(MaterialApp),
      matchesGoldenFile('goldens/direction_row_reverse.png'),
    );
  });

  testWidgets('flex_basis_percent', (tester) async {
    tester.view.devicePixelRatio = 1.0;
    addTearDown(() => tester.view.resetDevicePixelRatio());
    await tester.binding.setSurfaceSize(const Size(400, 300));
    await tester.pumpWidget(
      MaterialApp(
        debugShowCheckedModeBanner: false,
        home: Scaffold(
          backgroundColor: const Color(0xFF1C1C2E),
          body: FlexBasisPercent(),
        ),
      ),
    );
    // Consume any overflow errors so the golden is still captured.
    tester.takeException();
    await expectLater(
      find.byType(MaterialApp),
      matchesGoldenFile('goldens/flex_basis_percent.png'),
    );
  });

  testWidgets('gaps_mixed', (tester) async {
    tester.view.devicePixelRatio = 1.0;
    addTearDown(() => tester.view.resetDevicePixelRatio());
    await tester.binding.setSurfaceSize(const Size(400, 300));
    await tester.pumpWidget(
      MaterialApp(
        debugShowCheckedModeBanner: false,
        home: Scaffold(
          backgroundColor: const Color(0xFF1C1C2E),
          body: GapsMixed(),
        ),
      ),
    );
    // Consume any overflow errors so the golden is still captured.
    tester.takeException();
    await expectLater(
      find.byType(MaterialApp),
      matchesGoldenFile('goldens/gaps_mixed.png'),
    );
  });

  testWidgets('grow_shrink', (tester) async {
    tester.view.devicePixelRatio = 1.0;
    addTearDown(() => tester.view.resetDevicePixelRatio());
    await tester.binding.setSurfaceSize(const Size(400, 300));
    await tester.pumpWidget(
      MaterialApp(
        debugShowCheckedModeBanner: false,
        home: Scaffold(
          backgroundColor: const Color(0xFF1C1C2E),
          body: GrowShrink(),
        ),
      ),
    );
    // Consume any overflow errors so the golden is still captured.
    tester.takeException();
    await expectLater(
      find.byType(MaterialApp),
      matchesGoldenFile('goldens/grow_shrink.png'),
    );
  });

  testWidgets('hidden_child', (tester) async {
    tester.view.devicePixelRatio = 1.0;
    addTearDown(() => tester.view.resetDevicePixelRatio());
    await tester.binding.setSurfaceSize(const Size(400, 300));
    await tester.pumpWidget(
      MaterialApp(
        debugShowCheckedModeBanner: false,
        home: Scaffold(
          backgroundColor: const Color(0xFF1C1C2E),
          body: HiddenChild(),
        ),
      ),
    );
    // Consume any overflow errors so the golden is still captured.
    tester.takeException();
    await expectLater(
      find.byType(MaterialApp),
      matchesGoldenFile('goldens/hidden_child.png'),
    );
  });

  testWidgets('justify_center', (tester) async {
    tester.view.devicePixelRatio = 1.0;
    addTearDown(() => tester.view.resetDevicePixelRatio());
    await tester.binding.setSurfaceSize(const Size(400, 300));
    await tester.pumpWidget(
      MaterialApp(
        debugShowCheckedModeBanner: false,
        home: Scaffold(
          backgroundColor: const Color(0xFF1C1C2E),
          body: JustifyCenter(),
        ),
      ),
    );
    // Consume any overflow errors so the golden is still captured.
    tester.takeException();
    await expectLater(
      find.byType(MaterialApp),
      matchesGoldenFile('goldens/justify_center.png'),
    );
  });

  testWidgets('justify_space_between', (tester) async {
    tester.view.devicePixelRatio = 1.0;
    addTearDown(() => tester.view.resetDevicePixelRatio());
    await tester.binding.setSurfaceSize(const Size(400, 300));
    await tester.pumpWidget(
      MaterialApp(
        debugShowCheckedModeBanner: false,
        home: Scaffold(
          backgroundColor: const Color(0xFF1C1C2E),
          body: JustifySpaceBetween(),
        ),
      ),
    );
    // Consume any overflow errors so the golden is still captured.
    tester.takeException();
    await expectLater(
      find.byType(MaterialApp),
      matchesGoldenFile('goldens/justify_space_between.png'),
    );
  });

  testWidgets('justify_space_evenly', (tester) async {
    tester.view.devicePixelRatio = 1.0;
    addTearDown(() => tester.view.resetDevicePixelRatio());
    await tester.binding.setSurfaceSize(const Size(400, 300));
    await tester.pumpWidget(
      MaterialApp(
        debugShowCheckedModeBanner: false,
        home: Scaffold(
          backgroundColor: const Color(0xFF1C1C2E),
          body: JustifySpaceEvenly(),
        ),
      ),
    );
    // Consume any overflow errors so the golden is still captured.
    tester.takeException();
    await expectLater(
      find.byType(MaterialApp),
      matchesGoldenFile('goldens/justify_space_evenly.png'),
    );
  });

  testWidgets('min_max_sizes', (tester) async {
    tester.view.devicePixelRatio = 1.0;
    addTearDown(() => tester.view.resetDevicePixelRatio());
    await tester.binding.setSurfaceSize(const Size(400, 300));
    await tester.pumpWidget(
      MaterialApp(
        debugShowCheckedModeBanner: false,
        home: Scaffold(
          backgroundColor: const Color(0xFF1C1C2E),
          body: MinMaxSizes(),
        ),
      ),
    );
    // Consume any overflow errors so the golden is still captured.
    tester.takeException();
    await expectLater(
      find.byType(MaterialApp),
      matchesGoldenFile('goldens/min_max_sizes.png'),
    );
  });

  testWidgets('nested_mixed', (tester) async {
    tester.view.devicePixelRatio = 1.0;
    addTearDown(() => tester.view.resetDevicePixelRatio());
    await tester.binding.setSurfaceSize(const Size(400, 300));
    await tester.pumpWidget(
      MaterialApp(
        debugShowCheckedModeBanner: false,
        home: Scaffold(
          backgroundColor: const Color(0xFF1C1C2E),
          body: NestedMixed(),
        ),
      ),
    );
    // Consume any overflow errors so the golden is still captured.
    tester.takeException();
    await expectLater(
      find.byType(MaterialApp),
      matchesGoldenFile('goldens/nested_mixed.png'),
    );
  });

  testWidgets('ordered_children', (tester) async {
    tester.view.devicePixelRatio = 1.0;
    addTearDown(() => tester.view.resetDevicePixelRatio());
    await tester.binding.setSurfaceSize(const Size(400, 300));
    await tester.pumpWidget(
      MaterialApp(
        debugShowCheckedModeBanner: false,
        home: Scaffold(
          backgroundColor: const Color(0xFF1C1C2E),
          body: OrderedChildren(),
        ),
      ),
    );
    // Consume any overflow errors so the golden is still captured.
    tester.takeException();
    await expectLater(
      find.byType(MaterialApp),
      matchesGoldenFile('goldens/ordered_children.png'),
    );
  });

  testWidgets('padding_margin', (tester) async {
    tester.view.devicePixelRatio = 1.0;
    addTearDown(() => tester.view.resetDevicePixelRatio());
    await tester.binding.setSurfaceSize(const Size(400, 300));
    await tester.pumpWidget(
      MaterialApp(
        debugShowCheckedModeBanner: false,
        home: Scaffold(
          backgroundColor: const Color(0xFF1C1C2E),
          body: PaddingMargin(),
        ),
      ),
    );
    // Consume any overflow errors so the golden is still captured.
    tester.takeException();
    await expectLater(
      find.byType(MaterialApp),
      matchesGoldenFile('goldens/padding_margin.png'),
    );
  });

  testWidgets('single_leaf', (tester) async {
    tester.view.devicePixelRatio = 1.0;
    addTearDown(() => tester.view.resetDevicePixelRatio());
    await tester.binding.setSurfaceSize(const Size(400, 300));
    await tester.pumpWidget(
      MaterialApp(
        debugShowCheckedModeBanner: false,
        home: Scaffold(
          backgroundColor: const Color(0xFF1C1C2E),
          body: SingleLeaf(),
        ),
      ),
    );
    // Consume any overflow errors so the golden is still captured.
    tester.takeException();
    await expectLater(
      find.byType(MaterialApp),
      matchesGoldenFile('goldens/single_leaf.png'),
    );
  });

  testWidgets('tpl_card_grid', (tester) async {
    tester.view.devicePixelRatio = 1.0;
    addTearDown(() => tester.view.resetDevicePixelRatio());
    await tester.binding.setSurfaceSize(const Size(400, 300));
    await tester.pumpWidget(
      MaterialApp(
        debugShowCheckedModeBanner: false,
        home: Scaffold(
          backgroundColor: const Color(0xFF1C1C2E),
          body: TplCardGrid(),
        ),
      ),
    );
    // Consume any overflow errors so the golden is still captured.
    tester.takeException();
    await expectLater(
      find.byType(MaterialApp),
      matchesGoldenFile('goldens/tpl_card_grid.png'),
    );
  });

  testWidgets('tpl_holy_grail', (tester) async {
    tester.view.devicePixelRatio = 1.0;
    addTearDown(() => tester.view.resetDevicePixelRatio());
    await tester.binding.setSurfaceSize(const Size(400, 300));
    await tester.pumpWidget(
      MaterialApp(
        debugShowCheckedModeBanner: false,
        home: Scaffold(
          backgroundColor: const Color(0xFF1C1C2E),
          body: TplHolyGrail(),
        ),
      ),
    );
    // Consume any overflow errors so the golden is still captured.
    tester.takeException();
    await expectLater(
      find.byType(MaterialApp),
      matchesGoldenFile('goldens/tpl_holy_grail.png'),
    );
  });

  testWidgets('tpl_nav_bar', (tester) async {
    tester.view.devicePixelRatio = 1.0;
    addTearDown(() => tester.view.resetDevicePixelRatio());
    await tester.binding.setSurfaceSize(const Size(400, 300));
    await tester.pumpWidget(
      MaterialApp(
        debugShowCheckedModeBanner: false,
        home: Scaffold(
          backgroundColor: const Color(0xFF1C1C2E),
          body: TplNavBar(),
        ),
      ),
    );
    // Consume any overflow errors so the golden is still captured.
    tester.takeException();
    await expectLater(
      find.byType(MaterialApp),
      matchesGoldenFile('goldens/tpl_nav_bar.png'),
    );
  });

  testWidgets('tpl_sidebar_content', (tester) async {
    tester.view.devicePixelRatio = 1.0;
    addTearDown(() => tester.view.resetDevicePixelRatio());
    await tester.binding.setSurfaceSize(const Size(400, 300));
    await tester.pumpWidget(
      MaterialApp(
        debugShowCheckedModeBanner: false,
        home: Scaffold(
          backgroundColor: const Color(0xFF1C1C2E),
          body: TplSidebarContent(),
        ),
      ),
    );
    // Consume any overflow errors so the golden is still captured.
    tester.takeException();
    await expectLater(
      find.byType(MaterialApp),
      matchesGoldenFile('goldens/tpl_sidebar_content.png'),
    );
  });

  testWidgets('two_children', (tester) async {
    tester.view.devicePixelRatio = 1.0;
    addTearDown(() => tester.view.resetDevicePixelRatio());
    await tester.binding.setSurfaceSize(const Size(400, 300));
    await tester.pumpWidget(
      MaterialApp(
        debugShowCheckedModeBanner: false,
        home: Scaffold(
          backgroundColor: const Color(0xFF1C1C2E),
          body: TwoChildren(),
        ),
      ),
    );
    // Consume any overflow errors so the golden is still captured.
    tester.takeException();
    await expectLater(
      find.byType(MaterialApp),
      matchesGoldenFile('goldens/two_children.png'),
    );
  });

  testWidgets('vw_vh_sizes', (tester) async {
    tester.view.devicePixelRatio = 1.0;
    addTearDown(() => tester.view.resetDevicePixelRatio());
    await tester.binding.setSurfaceSize(const Size(400, 300));
    await tester.pumpWidget(
      MaterialApp(
        debugShowCheckedModeBanner: false,
        home: Scaffold(
          backgroundColor: const Color(0xFF1C1C2E),
          body: VwVhSizes(),
        ),
      ),
    );
    // Consume any overflow errors so the golden is still captured.
    tester.takeException();
    await expectLater(
      find.byType(MaterialApp),
      matchesGoldenFile('goldens/vw_vh_sizes.png'),
    );
  });

  testWidgets('wide_flat_5', (tester) async {
    tester.view.devicePixelRatio = 1.0;
    addTearDown(() => tester.view.resetDevicePixelRatio());
    await tester.binding.setSurfaceSize(const Size(400, 300));
    await tester.pumpWidget(
      MaterialApp(
        debugShowCheckedModeBanner: false,
        home: Scaffold(
          backgroundColor: const Color(0xFF1C1C2E),
          body: WideFlat5(),
        ),
      ),
    );
    // Consume any overflow errors so the golden is still captured.
    tester.takeException();
    await expectLater(
      find.byType(MaterialApp),
      matchesGoldenFile('goldens/wide_flat_5.png'),
    );
  });

  testWidgets('wrap_nowrap', (tester) async {
    tester.view.devicePixelRatio = 1.0;
    addTearDown(() => tester.view.resetDevicePixelRatio());
    await tester.binding.setSurfaceSize(const Size(400, 300));
    await tester.pumpWidget(
      MaterialApp(
        debugShowCheckedModeBanner: false,
        home: Scaffold(
          backgroundColor: const Color(0xFF1C1C2E),
          body: WrapNowrap(),
        ),
      ),
    );
    // Consume any overflow errors so the golden is still captured.
    tester.takeException();
    await expectLater(
      find.byType(MaterialApp),
      matchesGoldenFile('goldens/wrap_nowrap.png'),
    );
  });

  testWidgets('wrap_reverse', (tester) async {
    tester.view.devicePixelRatio = 1.0;
    addTearDown(() => tester.view.resetDevicePixelRatio());
    await tester.binding.setSurfaceSize(const Size(400, 300));
    await tester.pumpWidget(
      MaterialApp(
        debugShowCheckedModeBanner: false,
        home: Scaffold(
          backgroundColor: const Color(0xFF1C1C2E),
          body: WrapReverse(),
        ),
      ),
    );
    // Consume any overflow errors so the golden is still captured.
    tester.takeException();
    await expectLater(
      find.byType(MaterialApp),
      matchesGoldenFile('goldens/wrap_reverse.png'),
    );
  });
}
