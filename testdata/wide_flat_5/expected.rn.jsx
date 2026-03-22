import React from 'react';
import { View, Text } from 'react-native';

export default function FlexLayout() {
  return (
    <View style={{
      flexDirection: 'row',
      flexWrap: 'wrap',
      alignItems: 'flex-start',
      rowGap: 8,
      columnGap: 8,
      flexGrow: 1,
      flexShrink: 1,
      width: '100.0%',
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
        width: 60,
        height: 60,
        padding: 8,
        backgroundColor: 'rgb(251, 180, 174)',
      }}>
        <Text style={{ color: 'rgba(13, 13, 26, 0.85)', fontSize: 26 }}>item-0</Text>
      </View>
      <View style={{
        flexDirection: 'row',
        flexWrap: 'wrap',
        justifyContent: 'center',
        alignItems: 'center',
        rowGap: 4,
        columnGap: 4,
        flexShrink: 1,
        width: 60,
        height: 60,
        padding: 8,
        backgroundColor: 'rgb(179, 205, 227)',
      }}>
        <Text style={{ color: 'rgba(13, 13, 26, 0.85)', fontSize: 26 }}>item-1</Text>
      </View>
      <View style={{
        flexDirection: 'row',
        flexWrap: 'wrap',
        justifyContent: 'center',
        alignItems: 'center',
        rowGap: 4,
        columnGap: 4,
        flexShrink: 1,
        width: 60,
        height: 60,
        padding: 8,
        backgroundColor: 'rgb(204, 235, 197)',
      }}>
        <Text style={{ color: 'rgba(13, 13, 26, 0.85)', fontSize: 26 }}>item-2</Text>
      </View>
      <View style={{
        flexDirection: 'row',
        flexWrap: 'wrap',
        justifyContent: 'center',
        alignItems: 'center',
        rowGap: 4,
        columnGap: 4,
        flexShrink: 1,
        width: 60,
        height: 60,
        padding: 8,
        backgroundColor: 'rgb(222, 203, 228)',
      }}>
        <Text style={{ color: 'rgba(13, 13, 26, 0.85)', fontSize: 26 }}>item-3</Text>
      </View>
      <View style={{
        flexDirection: 'row',
        flexWrap: 'wrap',
        justifyContent: 'center',
        alignItems: 'center',
        rowGap: 4,
        columnGap: 4,
        flexShrink: 1,
        width: 60,
        height: 60,
        padding: 8,
        backgroundColor: 'rgb(254, 217, 166)',
      }}>
        <Text style={{ color: 'rgba(13, 13, 26, 0.85)', fontSize: 26 }}>item-4</Text>
      </View>
    </View>
  );
}
