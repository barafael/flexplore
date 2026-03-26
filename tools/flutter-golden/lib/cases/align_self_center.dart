// AUTO-GENERATED — do not edit. Run `cargo run -p build-overview` to regenerate.
import 'package:flutter/material.dart';

class AlignSelfCenter extends StatelessWidget {
  const AlignSelfCenter({super.key});

  @override
  Widget build(BuildContext context) {
  return   Container(
    width: double.infinity,
    height: double.infinity,
    padding: EdgeInsets.all(12.0),
    child:     Row(
      mainAxisAlignment: MainAxisAlignment.start,
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Flexible(
          fit: FlexFit.loose,
          child:           Container(
            width: 100.0,
            height: 60.0,
            padding: EdgeInsets.all(8.0),
            color: Color.fromRGBO(251, 180, 174, 1.0),
            alignment: Alignment.center,
            child: Text('top',
              style: TextStyle(fontSize: 26, color: Color.fromRGBO(13, 13, 26, 0.85)),
            ),
          )
        ),
        Flexible(
          fit: FlexFit.loose,
          child: Align(
            alignment: Alignment.center,
            child:             Container(
              width: 120.0,
              height: 60.0,
              padding: EdgeInsets.all(8.0),
              color: Color.fromRGBO(179, 205, 227, 1.0),
              alignment: Alignment.center,
              child: Text('centered',
                style: TextStyle(fontSize: 26, color: Color.fromRGBO(13, 13, 26, 0.85)),
              ),
            )
          ),
        ),
        Flexible(
          fit: FlexFit.loose,
          child:           Container(
            width: 100.0,
            height: 60.0,
            padding: EdgeInsets.all(8.0),
            color: Color.fromRGBO(204, 235, 197, 1.0),
            alignment: Alignment.center,
            child: Text('top',
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
