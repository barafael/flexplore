export default function FlexLayout() {
  return (
    <div style={{
      display: 'flex',
      flexDirection: 'column',
      alignContent: 'flex-start',
      flexGrow: 1,
      width: '100.0%',
      height: '100.0%',
      minHeight: '0.0px',
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
        flexShrink: 0,
        height: '60.0px',
        padding: '8.0px',
        background: 'rgb(251, 180, 174)',
        boxSizing: 'border-box',
        color: 'rgba(13, 13, 26, 0.85)',
        fontSize: 26,
      }}>header</div>
      <div style={{
        display: 'flex',
        alignContent: 'flex-start',
        flexGrow: 1,
        minHeight: '0.0px',
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
          flexShrink: 0,
          width: '200.0px',
          padding: '8.0px',
          background: 'rgb(179, 205, 227)',
          boxSizing: 'border-box',
          color: 'rgba(13, 13, 26, 0.85)',
          fontSize: 26,
        }}>sidebar-left</div>
        <div style={{
          display: 'flex',
          flexWrap: 'wrap',
          justifyContent: 'center',
          alignItems: 'center',
          alignContent: 'flex-start',
          rowGap: '4.0px',
          columnGap: '4.0px',
          flexGrow: 1,
          padding: '8.0px',
          background: 'rgb(204, 235, 197)',
          boxSizing: 'border-box',
          color: 'rgba(13, 13, 26, 0.85)',
          fontSize: 26,
        }}>content</div>
        <div style={{
          display: 'flex',
          flexWrap: 'wrap',
          justifyContent: 'center',
          alignItems: 'center',
          alignContent: 'flex-start',
          rowGap: '4.0px',
          columnGap: '4.0px',
          flexShrink: 0,
          width: '200.0px',
          padding: '8.0px',
          background: 'rgb(222, 203, 228)',
          boxSizing: 'border-box',
          color: 'rgba(13, 13, 26, 0.85)',
          fontSize: 26,
        }}>sidebar-right</div>
      </div>
      <div style={{
        display: 'flex',
        flexWrap: 'wrap',
        justifyContent: 'center',
        alignItems: 'center',
        alignContent: 'flex-start',
        rowGap: '4.0px',
        columnGap: '4.0px',
        flexShrink: 0,
        height: '60.0px',
        padding: '8.0px',
        background: 'rgb(254, 217, 166)',
        boxSizing: 'border-box',
        color: 'rgba(13, 13, 26, 0.85)',
        fontSize: 26,
      }}>footer</div>
    </div>
  );
}
