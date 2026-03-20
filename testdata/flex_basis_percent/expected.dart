Widget build(BuildContext context) {
  return   Container(
    width: double.infinity,
    padding: EdgeInsets.all(12.0),
    margin: EdgeInsets.all(0.0),
    child:     Row(
      mainAxisAlignment: MainAxisAlignment.start,
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Container(
          height: 80.0,
          padding: EdgeInsets.all(8.0),
          margin: EdgeInsets.all(0.0),
          color: Color.fromRGBO(251, 180, 174, 1.0),
          alignment: Alignment.center,
          child: Text('50%',
            style: TextStyle(fontSize: 26, color: Color.fromRGBO(13, 13, 26, 0.85)),
          ),
        )
        ,
        Container(
          height: 80.0,
          padding: EdgeInsets.all(8.0),
          margin: EdgeInsets.all(0.0),
          color: Color.fromRGBO(179, 205, 227, 1.0),
          alignment: Alignment.center,
          child: Text('25%',
            style: TextStyle(fontSize: 26, color: Color.fromRGBO(13, 13, 26, 0.85)),
          ),
        )
        ,
        Container(
          height: 80.0,
          padding: EdgeInsets.all(8.0),
          margin: EdgeInsets.all(0.0),
          color: Color.fromRGBO(204, 235, 197, 1.0),
          alignment: Alignment.center,
          child: Text('25%',
            style: TextStyle(fontSize: 26, color: Color.fromRGBO(13, 13, 26, 0.85)),
          ),
        )
        ,
      ],
    )
  )
;
}
