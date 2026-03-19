Widget build(BuildContext context) {
  return   Container(
    width: 100.0 /* 100% — use FractionallySizedBox */,
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
          padding: EdgeInsets.all(20.0),
          margin: EdgeInsets.all(10.0),
          color: Color.fromRGBO(251, 180, 174, 1.0),
          alignment: Alignment.center,
          child: Text('spaced',
            style: TextStyle(fontSize: 26, color: Color.fromRGBO(13, 13, 26, 0.85)),
          ),
        )
        ,
      ],
    )
  )
;
}
