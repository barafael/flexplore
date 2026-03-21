// AUTO-GENERATED — do not edit. Run tool/generate_cases.py to regenerate.
import 'package:flutter/material.dart';

class AlignItemsStretch extends StatelessWidget {
  const AlignItemsStretch({super.key});

  @override
  Widget build(BuildContext context) {
  return   Container(
    width: double.infinity,
    height: 300.0,
    padding: EdgeInsets.all(12.0),
    margin: EdgeInsets.all(0.0),
    child:     Row(
      mainAxisAlignment: MainAxisAlignment.start,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Flexible(
          fit: FlexFit.loose,
          child:           Container(
            width: 100.0,
            padding: EdgeInsets.all(8.0),
            margin: EdgeInsets.all(0.0),
            color: Color.fromRGBO(251, 180, 174, 1.0),
            alignment: Alignment.center,
            child: Text('A',
              style: TextStyle(fontSize: 26, color: Color.fromRGBO(13, 13, 26, 0.85)),
            ),
          )
        ),
        Flexible(
          fit: FlexFit.loose,
          child:           Container(
            width: 80.0,
            padding: EdgeInsets.all(8.0),
            margin: EdgeInsets.all(0.0),
            color: Color.fromRGBO(179, 205, 227, 1.0),
            alignment: Alignment.center,
            child: Text('B',
              style: TextStyle(fontSize: 26, color: Color.fromRGBO(13, 13, 26, 0.85)),
            ),
          )
        ),
        Flexible(
          fit: FlexFit.loose,
          child:           Container(
            width: 60.0,
            padding: EdgeInsets.all(8.0),
            margin: EdgeInsets.all(0.0),
            color: Color.fromRGBO(204, 235, 197, 1.0),
            alignment: Alignment.center,
            child: Text('C',
              style: TextStyle(fontSize: 26, color: Color.fromRGBO(13, 13, 26, 0.85)),
            ),
          )
        ),
      ],
    )
  )
;
}
}
