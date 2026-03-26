import React from 'react';
import { View, Text } from 'react-native';

export default function FlexLayout() {
  return (
    {/* CSS Grid — approximated with flexWrap */}
    <View style={{
      flexDirection: 'row',
      flexWrap: 'wrap',
      alignItems: 'flex-start',
      flexGrow: 1,
      flexShrink: 1,
      width: '100.0%',
      height: '100.0%',
      minHeight: 0,
      backgroundColor: 'rgba(28, 28, 43, 1)',
    }}>
      {/* Grid children — each sized to ~1/3 of container width */}
      {/* grid-column: 1 / span 3 — not supported in RN */}
      <View style={{
        flexDirection: 'row',
        flexWrap: 'wrap',
        justifyContent: 'center',
        alignItems: 'center',
        rowGap: 4,
        columnGap: 4,
        flexShrink: 1,
        padding: 8,
        backgroundColor: 'rgb(251, 180, 174)',
      }}>
        <Text style={{ color: 'rgba(13, 13, 26, 0.85)', fontSize: 26 }}>header</Text>
      </View>
      <View style={{
        flexDirection: 'row',
        flexWrap: 'wrap',
        justifyContent: 'center',
        alignItems: 'center',
        rowGap: 4,
        columnGap: 4,
        flexShrink: 1,
        padding: 8,
        backgroundColor: 'rgb(179, 205, 227)',
      }}>
        <Text style={{ color: 'rgba(13, 13, 26, 0.85)', fontSize: 26 }}>sidebar</Text>
      </View>
      {/* grid-column: span 2 — not supported in RN */}
      <View style={{
        flexDirection: 'row',
        flexWrap: 'wrap',
        justifyContent: 'center',
        alignItems: 'center',
        rowGap: 4,
        columnGap: 4,
        flexShrink: 1,
        padding: 8,
        backgroundColor: 'rgb(204, 235, 197)',
      }}>
        <Text style={{ color: 'rgba(13, 13, 26, 0.85)', fontSize: 26 }}>main</Text>
      </View>
      {/* grid-column: 1 / span 3 — not supported in RN */}
      <View style={{
        flexDirection: 'row',
        flexWrap: 'wrap',
        justifyContent: 'center',
        alignItems: 'center',
        rowGap: 4,
        columnGap: 4,
        flexShrink: 1,
        padding: 8,
        backgroundColor: 'rgb(222, 203, 228)',
      }}>
        <Text style={{ color: 'rgba(13, 13, 26, 0.85)', fontSize: 26 }}>footer</Text>
      </View>
    </View>
  );
}
