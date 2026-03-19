Widget build(BuildContext context) {
  return   Container(
    width: 100.0 /* 100% — use FractionallySizedBox */,
    height: 100.0 /* 100% — use FractionallySizedBox */,
    padding: EdgeInsets.all(0.0),
    margin: EdgeInsets.all(0.0),
    child:     Row(
      mainAxisAlignment: MainAxisAlignment.start,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Container(
          width: 250.0,
          padding: EdgeInsets.all(8.0),
          margin: EdgeInsets.all(0.0),
          child:           Column(
            mainAxisAlignment: MainAxisAlignment.start,
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              Container(
                height: 44.0,
                padding: EdgeInsets.all(8.0),
                margin: EdgeInsets.all(0.0),
                color: Color.fromRGBO(251, 180, 174, 1.0),
                alignment: Alignment.center,
                child: Text('nav-1',
                  style: TextStyle(fontSize: 26, color: Color.fromRGBO(13, 13, 26, 0.85)),
                ),
              )
              ,
              Container(
                height: 44.0,
                padding: EdgeInsets.all(8.0),
                margin: EdgeInsets.all(0.0),
                color: Color.fromRGBO(179, 205, 227, 1.0),
                alignment: Alignment.center,
                child: Text('nav-2',
                  style: TextStyle(fontSize: 26, color: Color.fromRGBO(13, 13, 26, 0.85)),
                ),
              )
              ,
              Container(
                height: 44.0,
                padding: EdgeInsets.all(8.0),
                margin: EdgeInsets.all(0.0),
                color: Color.fromRGBO(204, 235, 197, 1.0),
                alignment: Alignment.center,
                child: Text('nav-3',
                  style: TextStyle(fontSize: 26, color: Color.fromRGBO(13, 13, 26, 0.85)),
                ),
              )
              ,
            ],
          )
        )
        ,
        Expanded(
          flex: 1,
          child:           Container(
            padding: EdgeInsets.all(8.0),
            margin: EdgeInsets.all(0.0),
            color: Color.fromRGBO(222, 203, 228, 1.0),
            alignment: Alignment.center,
            child: Text('content',
              style: TextStyle(fontSize: 26, color: Color.fromRGBO(13, 13, 26, 0.85)),
            ),
          )
        ),
      ],
    )
  )
;
}
