import React from 'react';
import { View, Text, Dimensions } from 'react-native';

export default function FlexLayout() {
  return (
    <View style={{
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
        width: Dimensions.get('window').width * 0.50,
        height: Dimensions.get('window').height * 0.20,
        padding: 8,
        backgroundColor: 'rgb(251, 180, 174)',
      }}>
        <Text style={{ color: 'rgba(13, 13, 26, 0.85)', fontSize: 26 }}>50vw x 20vh</Text>
      </View>
      <View style={{
        flexDirection: 'row',
        flexWrap: 'wrap',
        justifyContent: 'center',
        alignItems: 'center',
        rowGap: 4,
        columnGap: 4,
        flexShrink: 1,
        width: Dimensions.get('window').width * 0.75,
        height: Dimensions.get('window').height * 0.30,
        padding: 8,
        backgroundColor: 'rgb(179, 205, 227)',
      }}>
        <Text style={{ color: 'rgba(13, 13, 26, 0.85)', fontSize: 26 }}>75vw x 30vh</Text>
      </View>
    </View>
  );
}
