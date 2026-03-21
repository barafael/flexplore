Widget build(BuildContext context) {
  return   Container(
    width: double.infinity,
    height: double.infinity,
    padding: EdgeInsets.all(12.0),
    margin: EdgeInsets.all(0.0),
    child:     Wrap(
      direction: Axis.horizontal,
      spacing: 8.0,
      runSpacing: 8.0,
      children: [
        Container(
          width: 80.0,
          height: 80.0,
          padding: EdgeInsets.all(8.0),
          margin: EdgeInsets.all(0.0),
          color: Color.fromRGBO(251, 180, 174, 1.0),
          alignment: Alignment.center,
          child: Text('visible',
            style: TextStyle(fontSize: 26, color: Color.fromRGBO(13, 13, 26, 0.85)),
          ),
        )
        ,
        Visibility(
          visible: false,
          maintainSize: true,
          maintainAnimation: true,
          maintainState: true,
          child:           Container(
            width: 80.0,
            height: 80.0,
            padding: EdgeInsets.all(8.0),
            margin: EdgeInsets.all(0.0),
            color: Color.fromRGBO(179, 205, 227, 1.0),
            alignment: Alignment.center,
            child: Text('hidden',
              style: TextStyle(fontSize: 26, color: Color.fromRGBO(13, 13, 26, 0.85)),
            ),
          )
        )
        ,
      ],
    )
  )
;
}
