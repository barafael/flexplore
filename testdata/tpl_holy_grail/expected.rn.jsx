import React from 'react';
import { View, Text } from 'react-native';

export default function FlexLayout() {
  return (
    <View style={{
      flexGrow: 1,
      flexShrink: 1,
      width: '100.0%',
      height: '100.0%',
      minHeight: 0,
      backgroundColor: 'rgba(28, 28, 43, 1)',
    }}>
      <View style={{
        flexDirection: 'row',
        flexWrap: 'wrap',
        justifyContent: 'center',
        alignItems: 'center',
        rowGap: 4,
        columnGap: 4,
        height: 60,
        padding: 8,
        backgroundColor: 'rgb(251, 180, 174)',
      }}>
        <Text style={{ color: 'rgba(13, 13, 26, 0.85)', fontSize: 26 }}>header</Text>
      </View>
      <View style={{
        flexDirection: 'row',
        flexGrow: 1,
        flexShrink: 1,
        minHeight: 0,
        backgroundColor: 'rgba(28, 28, 43, 1)',
      }}>
        <View style={{
          flexDirection: 'row',
          flexWrap: 'wrap',
          justifyContent: 'center',
          alignItems: 'center',
          rowGap: 4,
          columnGap: 4,
          width: 120,
          padding: 8,
          backgroundColor: 'rgb(179, 205, 227)',
        }}>
          <Text style={{ color: 'rgba(13, 13, 26, 0.85)', fontSize: 26 }}>sidebar-left</Text>
        </View>
        <View style={{
          flexDirection: 'row',
          flexWrap: 'wrap',
          justifyContent: 'center',
          alignItems: 'center',
          rowGap: 4,
          columnGap: 4,
          flexGrow: 1,
          flexShrink: 1,
          padding: 8,
          backgroundColor: 'rgb(204, 235, 197)',
        }}>
          <Text style={{ color: 'rgba(13, 13, 26, 0.85)', fontSize: 26 }}>content</Text>
        </View>
        <View style={{
          flexDirection: 'row',
          flexWrap: 'wrap',
          justifyContent: 'center',
          alignItems: 'center',
          rowGap: 4,
          columnGap: 4,
          width: 120,
          padding: 8,
          backgroundColor: 'rgb(222, 203, 228)',
        }}>
          <Text style={{ color: 'rgba(13, 13, 26, 0.85)', fontSize: 26 }}>sidebar-right</Text>
        </View>
      </View>
      <View style={{
        flexDirection: 'row',
        flexWrap: 'wrap',
        justifyContent: 'center',
        alignItems: 'center',
        rowGap: 4,
        columnGap: 4,
        height: 60,
        padding: 8,
        backgroundColor: 'rgb(254, 217, 166)',
      }}>
        <Text style={{ color: 'rgba(13, 13, 26, 0.85)', fontSize: 26 }}>footer</Text>
      </View>
    </View>
  );
}
