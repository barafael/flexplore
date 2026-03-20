Widget build(BuildContext context) {
  return   Container(
    width: double.infinity,
    padding: EdgeInsets.all(12.0),
    margin: EdgeInsets.all(0.0),
    child:     Wrap(
      direction: Axis.horizontal,
      spacing: 8.0,
      runSpacing: 8.0,
      children: [
        Container(
          width: 60.0,
          height: 60.0,
          padding: EdgeInsets.all(8.0),
          margin: EdgeInsets.all(0.0),
          color: Color.fromRGBO(251, 180, 174, 1.0),
          alignment: Alignment.center,
          child: Text('item-0',
            style: TextStyle(fontSize: 26, color: Color.fromRGBO(13, 13, 26, 0.85)),
          ),
        )
        ,
        Container(
          width: 60.0,
          height: 60.0,
          padding: EdgeInsets.all(8.0),
          margin: EdgeInsets.all(0.0),
          color: Color.fromRGBO(179, 205, 227, 1.0),
          alignment: Alignment.center,
          child: Text('item-1',
            style: TextStyle(fontSize: 26, color: Color.fromRGBO(13, 13, 26, 0.85)),
          ),
        )
        ,
        Container(
          width: 60.0,
          height: 60.0,
          padding: EdgeInsets.all(8.0),
          margin: EdgeInsets.all(0.0),
          color: Color.fromRGBO(204, 235, 197, 1.0),
          alignment: Alignment.center,
          child: Text('item-2',
            style: TextStyle(fontSize: 26, color: Color.fromRGBO(13, 13, 26, 0.85)),
          ),
        )
        ,
        Container(
          width: 60.0,
          height: 60.0,
          padding: EdgeInsets.all(8.0),
          margin: EdgeInsets.all(0.0),
          color: Color.fromRGBO(222, 203, 228, 1.0),
          alignment: Alignment.center,
          child: Text('item-3',
            style: TextStyle(fontSize: 26, color: Color.fromRGBO(13, 13, 26, 0.85)),
          ),
        )
        ,
        Container(
          width: 60.0,
          height: 60.0,
          padding: EdgeInsets.all(8.0),
          margin: EdgeInsets.all(0.0),
          color: Color.fromRGBO(254, 217, 166, 1.0),
          alignment: Alignment.center,
          child: Text('item-4',
            style: TextStyle(fontSize: 26, color: Color.fromRGBO(13, 13, 26, 0.85)),
          ),
        )
        ,
      ],
    )
  )
;
}
