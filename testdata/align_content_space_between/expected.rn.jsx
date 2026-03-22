import React from 'react';
import { View, Text } from 'react-native';

export default function FlexLayout() {
  return (
    <View style={{
      flexDirection: 'row',
      flexWrap: 'wrap',
      alignItems: 'flex-start',
      alignContent: 'space-between',
      rowGap: 8,
      columnGap: 8,
      flexGrow: 1,
      flexShrink: 1,
      width: '100.0%',
      height: 280,
      minHeight: 0,
      padding: 12,
      backgroundColor: 'rgba(28, 28, 43, 1)',
    }}>
      <View style={{
        flexDirection: 'row',
        flexWrap: 'wrap',
        justifyContent: 'center',
        alignItems: 'center',
        rowGap: 4,
        columnGap: 4,
        flexShrink: 1,
        width: 170,
        height: 60,
        padding: 8,
        backgroundColor: 'rgb(251, 180, 174)',
      }}>
        <Text style={{ color: 'rgba(13, 13, 26, 0.85)', fontSize: 26 }}>A</Text>
      </View>
      <View style={{
        flexDirection: 'row',
        flexWrap: 'wrap',
        justifyContent: 'center',
        alignItems: 'center',
        rowGap: 4,
        columnGap: 4,
        flexShrink: 1,
        width: 170,
        height: 60,
        padding: 8,
        backgroundColor: 'rgb(179, 205, 227)',
      }}>
        <Text style={{ color: 'rgba(13, 13, 26, 0.85)', fontSize: 26 }}>B</Text>
      </View>
      <View style={{
        flexDirection: 'row',
        flexWrap: 'wrap',
        justifyContent: 'center',
        alignItems: 'center',
        rowGap: 4,
        columnGap: 4,
        flexShrink: 1,
        width: 170,
        height: 60,
        padding: 8,
        backgroundColor: 'rgb(204, 235, 197)',
      }}>
        <Text style={{ color: 'rgba(13, 13, 26, 0.85)', fontSize: 26 }}>C</Text>
      </View>
      <View style={{
        flexDirection: 'row',
        flexWrap: 'wrap',
        justifyContent: 'center',
        alignItems: 'center',
        rowGap: 4,
        columnGap: 4,
        flexShrink: 1,
        width: 170,
        height: 60,
        padding: 8,
        backgroundColor: 'rgb(222, 203, 228)',
      }}>
        <Text style={{ color: 'rgba(13, 13, 26, 0.85)', fontSize: 26 }}>D</Text>
      </View>
      <View style={{
        flexDirection: 'row',
        flexWrap: 'wrap',
        justifyContent: 'center',
        alignItems: 'center',
        rowGap: 4,
        columnGap: 4,
        flexShrink: 1,
        width: 170,
        height: 60,
        padding: 8,
        backgroundColor: 'rgb(254, 217, 166)',
      }}>
        <Text style={{ color: 'rgba(13, 13, 26, 0.85)', fontSize: 26 }}>E</Text>
      </View>
      <View style={{
        flexDirection: 'row',
        flexWrap: 'wrap',
        justifyContent: 'center',
        alignItems: 'center',
        rowGap: 4,
        columnGap: 4,
        flexShrink: 1,
        width: 170,
        height: 60,
        padding: 8,
        backgroundColor: 'rgb(255, 255, 204)',
      }}>
        <Text style={{ color: 'rgba(13, 13, 26, 0.85)', fontSize: 26 }}>F</Text>
      </View>
    </View>
  );
}
