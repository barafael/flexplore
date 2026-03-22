import React from 'react';
import { View, Text } from 'react-native';

export default function FlexLayout() {
  return (
    <View style={{
      flexDirection: 'row',
      justifyContent: 'space-between',
      alignItems: 'center',
      rowGap: 8,
      flexGrow: 1,
      flexShrink: 1,
      width: '100.0%',
      height: 56,
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
        width: 48,
        height: 48,
        padding: 8,
        backgroundColor: 'rgb(251, 180, 174)',
      }}>
        <Text style={{ color: 'rgba(13, 13, 26, 0.85)', fontSize: 26 }}>logo</Text>
      </View>
      <View style={{
        flexDirection: 'row',
        alignItems: 'center',
        columnGap: 8,
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
          flexShrink: 1,
          width: 80,
          height: 36,
          padding: 8,
          backgroundColor: 'rgb(179, 205, 227)',
        }}>
          <Text style={{ color: 'rgba(13, 13, 26, 0.85)', fontSize: 26 }}>link-1</Text>
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
          height: 36,
          padding: 8,
          backgroundColor: 'rgb(204, 235, 197)',
        }}>
          <Text style={{ color: 'rgba(13, 13, 26, 0.85)', fontSize: 26 }}>link-2</Text>
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
          height: 36,
          padding: 8,
          backgroundColor: 'rgb(222, 203, 228)',
        }}>
          <Text style={{ color: 'rgba(13, 13, 26, 0.85)', fontSize: 26 }}>link-3</Text>
        </View>
      </View>
      <View style={{
        flexDirection: 'row',
        alignItems: 'center',
        columnGap: 8,
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
          flexShrink: 1,
          width: 36,
          height: 36,
          padding: 8,
          backgroundColor: 'rgb(254, 217, 166)',
        }}>
          <Text style={{ color: 'rgba(13, 13, 26, 0.85)', fontSize: 26 }}>btn-1</Text>
        </View>
        <View style={{
          flexDirection: 'row',
          flexWrap: 'wrap',
          justifyContent: 'center',
          alignItems: 'center',
          rowGap: 4,
          columnGap: 4,
          flexShrink: 1,
          width: 36,
          height: 36,
          padding: 8,
          backgroundColor: 'rgb(255, 255, 204)',
        }}>
          <Text style={{ color: 'rgba(13, 13, 26, 0.85)', fontSize: 26 }}>btn-2</Text>
        </View>
      </View>
    </View>
  );
}
