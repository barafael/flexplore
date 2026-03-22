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
        width: 80,
        height: 80,
        padding: 8,
        backgroundColor: 'rgb(251, 180, 174)',
      }}>
        <Text style={{ color: 'rgba(13, 13, 26, 0.85)', fontSize: 26 }}>A</Text>
      </View>
      <View style={{
        flexWrap: 'wrap',
        alignItems: 'flex-start',
        rowGap: 8,
        columnGap: 8,
        flexGrow: 1,
        flexShrink: 1,
        width: 200,
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
          width: 40,
          height: 40,
          padding: 8,
          backgroundColor: 'rgb(179, 205, 227)',
        }}>
          <Text style={{ color: 'rgba(13, 13, 26, 0.85)', fontSize: 26 }}>X</Text>
        </View>
        <View style={{
          flexDirection: 'row',
          flexWrap: 'wrap',
          justifyContent: 'center',
          alignItems: 'center',
          rowGap: 4,
          columnGap: 4,
          flexShrink: 1,
          width: 40,
          height: 40,
          padding: 8,
          backgroundColor: 'rgb(204, 235, 197)',
        }}>
          <Text style={{ color: 'rgba(13, 13, 26, 0.85)', fontSize: 26 }}>Y</Text>
        </View>
      </View>
      <View style={{
        flexDirection: 'row',
        flexWrap: 'wrap',
        justifyContent: 'center',
        alignItems: 'center',
        rowGap: 4,
        columnGap: 4,
        flexShrink: 1,
        width: 80,
        height: 80,
        padding: 8,
        backgroundColor: 'rgb(222, 203, 228)',
      }}>
        <Text style={{ color: 'rgba(13, 13, 26, 0.85)', fontSize: 26 }}>B</Text>
      </View>
    </View>
  );
}
