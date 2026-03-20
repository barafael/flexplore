export default function FlexLayout() {
  return (
    <div style={{
      display: 'flex',
      alignItems: 'flex-start',
      alignContent: 'flex-start',
      rowGap: '8.0px',
      columnGap: '8.0px',
      flexGrow: 1,
      width: '100.0%',
      minHeight: '0.0px',
      padding: '20.0px',
      background: 'rgba(28, 28, 43, 1)',
      boxSizing: 'border-box',
    }}>
      <div style={{
        display: 'flex',
        flexWrap: 'wrap',
        justifyContent: 'center',
        alignItems: 'center',
        alignContent: 'flex-start',
        rowGap: '4.0px',
        columnGap: '4.0px',
        width: '100.0px',
        height: '60.0px',
        padding: '8.0px',
        margin: '16.0px',
        background: 'rgb(251, 180, 174)',
        boxSizing: 'border-box',
        color: 'rgba(13, 13, 26, 0.85)',
        fontSize: 26,
      }}>A</div>
      <div style={{
        display: 'flex',
        flexWrap: 'wrap',
        justifyContent: 'center',
        alignItems: 'center',
        alignContent: 'flex-start',
        rowGap: '4.0px',
        columnGap: '4.0px',
        width: '100.0px',
        height: '60.0px',
        padding: '8.0px',
        margin: '16.0px',
        background: 'rgb(179, 205, 227)',
        boxSizing: 'border-box',
        color: 'rgba(13, 13, 26, 0.85)',
        fontSize: 26,
      }}>B</div>
      <div style={{
        display: 'flex',
        flexWrap: 'wrap',
        justifyContent: 'center',
        alignItems: 'center',
        alignContent: 'flex-start',
        rowGap: '4.0px',
        columnGap: '4.0px',
        width: '100.0px',
        height: '60.0px',
        padding: '8.0px',
        margin: '16.0px',
        background: 'rgb(204, 235, 197)',
        boxSizing: 'border-box',
        color: 'rgba(13, 13, 26, 0.85)',
        fontSize: 26,
      }}>C</div>
    </div>
  );
}
