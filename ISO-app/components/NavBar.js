// JS file for the NavBar component
// Path: ISO-app\components\NavBar.js
import GLOBALS from '../globals';

import React, { Component } from 'react';
import { Text, View } from 'react-native';

import { createMaterialBottomTabNavigator } from '@react-navigation/material-bottom-tabs';
import Icon from 'react-native-vector-icons/Ionicons';

import FeedScreen from '../pages/Feed';
import MessagesScreen from '../pages/Messages';
import ProfileScreen from '../pages/Profile';

const Tab = createMaterialBottomTabNavigator();

function NavBar() {
    return (
        <Tab.Navigator
            initialRouteName="Feed"
            activeColor='white'
            inactiveColor='gray'
            barStyle={{ backgroundColor: GLOBALS.COLORS.THEME }}
        >
            <Tab.Screen 
                name="Feed" 
                component={FeedScreen} 
                options={{
                tabBarLabel: 'Feed',
                tabBarIcon: ({ color }) => (
                    <Icon name="home" color={color} size={26} />
                )}}
            />
            <Tab.Screen 
                name="Messages" 
                component={MessagesScreen} 
                options={{
                tabBarLabel: 'Messages',
                tabBarIcon: ({ color }) => (
                    <Icon name="mail" color={color} size={26} />
                )}}
            />
            <Tab.Screen 
                name="Profile"
                component={ProfileScreen}
                options={{
                tabBarLabel: 'Profile',
                tabBarIcon: ({ color }) => (
                    <Icon name="person" color={color} size={26} />
                )}}
            />
        </Tab.Navigator>
    );
}

export default NavBar;