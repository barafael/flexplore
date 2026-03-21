Widget build(BuildContext context) {
  return   Container(
    width: double.infinity,
    height: double.infinity,
    padding: EdgeInsets.all(12.0),
    margin: EdgeInsets.all(0.0),
    child:     Wrap(
      direction: Axis.horizontal,
      runAlignment: WrapAlignment.spaceBetween,
      spacing: 8.0,
      runSpacing: 8.0,
      children: [
        Container(
          width: 170.0,
          height: 60.0,
          padding: EdgeInsets.all(8.0),
          margin: EdgeInsets.all(0.0),
          color: Color.fromRGBO(251, 180, 174, 1.0),
          alignment: Alignment.center,
          child: Text('A',
            style: TextStyle(fontSize: 26, color: Color.fromRGBO(13, 13, 26, 0.85)),
          ),
        )
        ,
        Container(
          width: 170.0,
          height: 60.0,
          padding: EdgeInsets.all(8.0),
          margin: EdgeInsets.all(0.0),
          color: Color.fromRGBO(179, 205, 227, 1.0),
          alignment: Alignment.center,
          child: Text('B',
            style: TextStyle(fontSize: 26, color: Color.fromRGBO(13, 13, 26, 0.85)),
          ),
        )
        ,
        Container(
          width: 170.0,
          height: 60.0,
          padding: EdgeInsets.all(8.0),
          margin: EdgeInsets.all(0.0),
          color: Color.fromRGBO(204, 235, 197, 1.0),
          alignment: Alignment.center,
          child: Text('C',
            style: TextStyle(fontSize: 26, color: Color.fromRGBO(13, 13, 26, 0.85)),
          ),
        )
        ,
        Container(
          width: 170.0,
          height: 60.0,
          padding: EdgeInsets.all(8.0),
          margin: EdgeInsets.all(0.0),
          color: Color.fromRGBO(222, 203, 228, 1.0),
          alignment: Alignment.center,
          child: Text('D',
            style: TextStyle(fontSize: 26, color: Color.fromRGBO(13, 13, 26, 0.85)),
          ),
        )
        ,
        Container(
          width: 170.0,
          height: 60.0,
          padding: EdgeInsets.all(8.0),
          margin: EdgeInsets.all(0.0),
          color: Color.fromRGBO(254, 217, 166, 1.0),
          alignment: Alignment.center,
          child: Text('E',
            style: TextStyle(fontSize: 26, color: Color.fromRGBO(13, 13, 26, 0.85)),
          ),
        )
        ,
        Container(
          width: 170.0,
          height: 60.0,
          padding: EdgeInsets.all(8.0),
          margin: EdgeInsets.all(0.0),
          color: Color.fromRGBO(255, 255, 204, 1.0),
          alignment: Alignment.center,
          child: Text('F',
            style: TextStyle(fontSize: 26, color: Color.fromRGBO(13, 13, 26, 0.85)),
          ),
        )
        ,
      ],
    )
  )
;
}
