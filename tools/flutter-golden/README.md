# flutter_golden

Golden-screenshot harness for flexplore's Flutter code generator. Every
`lib/cases/*.dart` widget is generated from `testdata/<case>/expected.dart`
(run `cargo run -p build-overview -- --generate-only` from the repository root
to refresh them; do not edit by hand), and `test/golden_test.dart` renders each
one at 400x300 with `flutter test --update-goldens`, writing
`test/goldens/<case>.png`. `cargo run -p build-overview -- --backend flutter`
drives the whole thing and copies the PNGs to `testdata/<case>/rendered_flutter.png`
for the cross-backend comparison in `testdata/overview.html`. The package is
not published; it only exists to be run from this repository.
