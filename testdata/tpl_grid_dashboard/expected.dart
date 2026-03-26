Widget build(BuildContext context) {
  return   Container(
    width: double.infinity,
    height: double.infinity,
    child:     // CSS Grid layout — use GridView.count or a custom grid widget
    Wrap(
      // grid-template-columns / rows not directly supported in Flutter
      // grid-template-columns: 1.0fr 1.0fr 1.0fr
      // grid-template-rows: 60px 1.0fr 40px
      spacing: 0.0,
      runSpacing: 0.0,
      children: [
        Container(
          padding: EdgeInsets.all(8.0),
          color: Color.fromRGBO(251, 180, 174, 1.0),
          alignment: Alignment.center,
          child: Text('header',
            style: TextStyle(fontSize: 26, color: Color.fromRGBO(13, 13, 26, 0.85)),
          ),
        )
        ,
        Container(
          padding: EdgeInsets.all(8.0),
          color: Color.fromRGBO(179, 205, 227, 1.0),
          alignment: Alignment.center,
          child: Text('sidebar',
            style: TextStyle(fontSize: 26, color: Color.fromRGBO(13, 13, 26, 0.85)),
          ),
        )
        ,
        Container(
          padding: EdgeInsets.all(8.0),
          color: Color.fromRGBO(204, 235, 197, 1.0),
          alignment: Alignment.center,
          child: Text('main',
            style: TextStyle(fontSize: 26, color: Color.fromRGBO(13, 13, 26, 0.85)),
          ),
        )
        ,
        Container(
          padding: EdgeInsets.all(8.0),
          color: Color.fromRGBO(222, 203, 228, 1.0),
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
