// AUTO-GENERATED — do not edit. Run tool/generate_cases.py to regenerate.
import 'package:flutter/material.dart';

class TplHolyGrail extends StatelessWidget {
  const TplHolyGrail({super.key});

  @override
  Widget build(BuildContext context) {
  return   Container(
    width: double.infinity,
    height: double.infinity,
    padding: EdgeInsets.all(0.0),
    margin: EdgeInsets.all(0.0),
    child:     Column(
      mainAxisAlignment: MainAxisAlignment.start,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Container(
          height: 60.0,
          padding: EdgeInsets.all(8.0),
          margin: EdgeInsets.all(0.0),
          color: Color.fromRGBO(251, 180, 174, 1.0),
          alignment: Alignment.center,
          child: Text('header',
            style: TextStyle(fontSize: 26, color: Color.fromRGBO(13, 13, 26, 0.85)),
          ),
        )
        ,
        Expanded(
          flex: 1,
          child:           Container(
            padding: EdgeInsets.all(0.0),
            margin: EdgeInsets.all(0.0),
            child:             Row(
              mainAxisAlignment: MainAxisAlignment.start,
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                Container(
                  width: 200.0,
                  padding: EdgeInsets.all(8.0),
                  margin: EdgeInsets.all(0.0),
                  color: Color.fromRGBO(179, 205, 227, 1.0),
                  alignment: Alignment.center,
                  child: Text('sidebar-left',
                    style: TextStyle(fontSize: 26, color: Color.fromRGBO(13, 13, 26, 0.85)),
                  ),
                )
                ,
                Expanded(
                  flex: 1,
                  child:                   Container(
                    padding: EdgeInsets.all(8.0),
                    margin: EdgeInsets.all(0.0),
                    color: Color.fromRGBO(204, 235, 197, 1.0),
                    alignment: Alignment.center,
                    child: Text('content',
                      style: TextStyle(fontSize: 26, color: Color.fromRGBO(13, 13, 26, 0.85)),
                    ),
                  )
                ),
                Container(
                  width: 200.0,
                  padding: EdgeInsets.all(8.0),
                  margin: EdgeInsets.all(0.0),
                  color: Color.fromRGBO(222, 203, 228, 1.0),
                  alignment: Alignment.center,
                  child: Text('sidebar-right',
                    style: TextStyle(fontSize: 26, color: Color.fromRGBO(13, 13, 26, 0.85)),
                  ),
                )
                ,
              ],
            )
          )
        ),
        Container(
          height: 60.0,
          padding: EdgeInsets.all(8.0),
          margin: EdgeInsets.all(0.0),
          color: Color.fromRGBO(254, 217, 166, 1.0),
          alignment: Alignment.center,
          child: Text('footer',
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
