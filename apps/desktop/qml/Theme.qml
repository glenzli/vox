pragma Singleton
import QtQuick
QtObject {
    readonly property bool dark: preferences.theme === "dark" || (preferences.theme === "system" && preferences.systemDark)
    readonly property color window: dark ? "#161616" : "#f7f7f6"
    readonly property color panel: dark ? "#1d1d1d" : "#ffffff"
    readonly property color inset: dark ? "#141414" : "#f2f2f1"
    readonly property color hover: dark ? "#303030" : "#eaeae8"
    readonly property color selected: dark ? "#363636" : "#e4e4e1"
    readonly property color border: dark ? "#353535" : "#dededb"
    readonly property color text: dark ? "#f1f1ee" : "#242423"
    readonly property color secondary: dark ? "#a5a5a2" : "#72726e"
    readonly property color muted: dark ? "#777774" : "#969691"
    readonly property color accent: dark ? "#eeeeeb" : "#282827"
    readonly property color accentText: dark ? "#1a1a1a" : "#ffffff"
    readonly property color scriptCommand: dark ? "#9cb4ce" : "#49647f"
    readonly property color scriptIdentity: dark ? "#d2b48c" : "#78572f"
}
