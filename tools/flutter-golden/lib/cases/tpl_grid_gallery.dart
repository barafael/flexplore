// AUTO-GENERATED — do not edit. Run `cargo run -p build-overview` to regenerate.
import 'package:flutter/material.dart';

class TplGridGallery extends StatelessWidget {
  const TplGridGallery({super.key});

  @override
  Widget build(BuildContext context) {
  return   Container(
    width: double.infinity,
    height: double.infinity,
    padding: EdgeInsets.all(8.0),
    child:     // CSS Grid layout — use GridView.count or a custom grid widget
    Wrap(
      // grid-template-columns / rows not directly supported in Flutter
      // grid-template-columns: 1.0fr 1.0fr 1.0fr 1.0fr
      // grid-auto-rows: 120px
      spacing: 8.0,
      runSpacing: 8.0,
      children: [
        Container(
          padding: EdgeInsets.all(8.0),
          color: Color.fromRGBO(251, 180, 174, 1.0),
          alignment: Alignment.center,
          child: Text('wide',
            style: TextStyle(fontSize: 26, color: Color.fromRGBO(13, 13, 26, 0.85)),
          ),
        )
        ,
        Container(
          padding: EdgeInsets.all(8.0),
          color: Color.fromRGBO(179, 205, 227, 1.0),
          alignment: Alignment.center,
          child: Text('img-1',
            style: TextStyle(fontSize: 26, color: Color.fromRGBO(13, 13, 26, 0.85)),
          ),
        )
        ,
        Container(
          padding: EdgeInsets.all(8.0),
          color: Color.fromRGBO(204, 235, 197, 1.0),
          alignment: Alignment.center,
          child: Text('img-2',
            style: TextStyle(fontSize: 26, color: Color.fromRGBO(13, 13, 26, 0.85)),
          ),
        )
        ,
        Container(
          padding: EdgeInsets.all(8.0),
          color: Color.fromRGBO(222, 203, 228, 1.0),
          alignment: Alignment.center,
          child: Text('tall',
            style: TextStyle(fontSize: 26, color: Color.fromRGBO(13, 13, 26, 0.85)),
          ),
        )
        ,
        Container(
          padding: EdgeInsets.all(8.0),
          color: Color.fromRGBO(254, 217, 166, 1.0),
          alignment: Alignment.center,
          child: Text('img-3',
            style: TextStyle(fontSize: 26, color: Color.fromRGBO(13, 13, 26, 0.85)),
          ),
        )
        ,
        Container(
          padding: EdgeInsets.all(8.0),
          color: Color.fromRGBO(255, 255, 204, 1.0),
          alignment: Alignment.center,
          child: Text('img-4',
            style: TextStyle(fontSize: 26, color: Color.fromRGBO(13, 13, 26, 0.85)),
          ),
        )
        ,
      ],
    )
  )
;
}
}
