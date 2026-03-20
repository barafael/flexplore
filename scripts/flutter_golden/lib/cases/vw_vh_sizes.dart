// AUTO-GENERATED — do not edit. Run tool/generate_cases.py to regenerate.
import 'package:flutter/material.dart';

class VwVhSizes extends StatelessWidget {
  const VwVhSizes({super.key});

  @override
  Widget build(BuildContext context) {
  return   Container(
    width: double.infinity,
    padding: EdgeInsets.all(12.0),
    margin: EdgeInsets.all(0.0),
    child:     Wrap(
      direction: Axis.vertical,
      spacing: 8.0,
      runSpacing: 8.0,
      children: [
        Container(
          width: MediaQuery.of(context).size.width * 0.500,
          height: MediaQuery.of(context).size.height * 0.200,
          padding: EdgeInsets.all(8.0),
          margin: EdgeInsets.all(0.0),
          color: Color.fromRGBO(251, 180, 174, 1.0),
          alignment: Alignment.center,
          child: Text('50vw x 20vh',
            style: TextStyle(fontSize: 26, color: Color.fromRGBO(13, 13, 26, 0.85)),
          ),
        )
        ,
        Container(
          width: MediaQuery.of(context).size.width * 0.750,
          height: MediaQuery.of(context).size.height * 0.300,
          padding: EdgeInsets.all(8.0),
          margin: EdgeInsets.all(0.0),
          color: Color.fromRGBO(179, 205, 227, 1.0),
          alignment: Alignment.center,
          child: Text('75vw x 30vh',
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
