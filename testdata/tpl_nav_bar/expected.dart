Widget build(BuildContext context) {
  return   Container(
    width: double.infinity,
    height: 56.0,
    padding: EdgeInsets.all(12.0),
    margin: EdgeInsets.all(0.0),
    child:     Row(
      mainAxisAlignment: MainAxisAlignment.spaceBetween,
      crossAxisAlignment: CrossAxisAlignment.center,
      children: [
        Flexible(
          fit: FlexFit.loose,
          child:           Container(
            width: 48.0,
            height: 48.0,
            padding: EdgeInsets.all(8.0),
            margin: EdgeInsets.all(0.0),
            color: Color.fromRGBO(251, 180, 174, 1.0),
            alignment: Alignment.center,
            child: Text('logo',
              style: TextStyle(fontSize: 26, color: Color.fromRGBO(13, 13, 26, 0.85)),
            ),
          )
        ),
        Flexible(
          fit: FlexFit.loose,
          child:           Container(
            padding: EdgeInsets.all(0.0),
            margin: EdgeInsets.all(0.0),
            child:             Row(
              mainAxisAlignment: MainAxisAlignment.start,
              crossAxisAlignment: CrossAxisAlignment.center,
              children: [
                Flexible(
                  fit: FlexFit.loose,
                  child:                   Container(
                    width: 80.0,
                    height: 36.0,
                    padding: EdgeInsets.all(8.0),
                    margin: EdgeInsets.all(0.0),
                    color: Color.fromRGBO(179, 205, 227, 1.0),
                    alignment: Alignment.center,
                    child: Text('link-1',
                      style: TextStyle(fontSize: 26, color: Color.fromRGBO(13, 13, 26, 0.85)),
                    ),
                  )
                ),
                Flexible(
                  fit: FlexFit.loose,
                  child:                   Container(
                    width: 80.0,
                    height: 36.0,
                    padding: EdgeInsets.all(8.0),
                    margin: EdgeInsets.all(0.0),
                    color: Color.fromRGBO(204, 235, 197, 1.0),
                    alignment: Alignment.center,
                    child: Text('link-2',
                      style: TextStyle(fontSize: 26, color: Color.fromRGBO(13, 13, 26, 0.85)),
                    ),
                  )
                ),
                Flexible(
                  fit: FlexFit.loose,
                  child:                   Container(
                    width: 80.0,
                    height: 36.0,
                    padding: EdgeInsets.all(8.0),
                    margin: EdgeInsets.all(0.0),
                    color: Color.fromRGBO(222, 203, 228, 1.0),
                    alignment: Alignment.center,
                    child: Text('link-3',
                      style: TextStyle(fontSize: 26, color: Color.fromRGBO(13, 13, 26, 0.85)),
                    ),
                  )
                ),
              ],
            )
          )
        ),
        Flexible(
          fit: FlexFit.loose,
          child:           Container(
            padding: EdgeInsets.all(0.0),
            margin: EdgeInsets.all(0.0),
            child:             Row(
              mainAxisAlignment: MainAxisAlignment.start,
              crossAxisAlignment: CrossAxisAlignment.center,
              children: [
                Flexible(
                  fit: FlexFit.loose,
                  child:                   Container(
                    width: 36.0,
                    height: 36.0,
                    padding: EdgeInsets.all(8.0),
                    margin: EdgeInsets.all(0.0),
                    color: Color.fromRGBO(254, 217, 166, 1.0),
                    alignment: Alignment.center,
                    child: Text('btn-1',
                      style: TextStyle(fontSize: 26, color: Color.fromRGBO(13, 13, 26, 0.85)),
                    ),
                  )
                ),
                Flexible(
                  fit: FlexFit.loose,
                  child:                   Container(
                    width: 36.0,
                    height: 36.0,
                    padding: EdgeInsets.all(8.0),
                    margin: EdgeInsets.all(0.0),
                    color: Color.fromRGBO(255, 255, 204, 1.0),
                    alignment: Alignment.center,
                    child: Text('btn-2',
                      style: TextStyle(fontSize: 26, color: Color.fromRGBO(13, 13, 26, 0.85)),
                    ),
                  )
                ),
              ],
            )
          )
        ),
      ],
    )
  )
;
}
