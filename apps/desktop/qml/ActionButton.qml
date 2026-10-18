import QtQuick
import QtQuick.Controls
Button {
    id: control
    property string symbol: ""
    property bool primary: false
    readonly property color foregroundColor: !control.enabled ? Theme.secondary : control.primary ? Theme.accentText : Theme.text
    implicitHeight: 36
    leftPadding: 13
    rightPadding: 13
    icon.source: symbol.length ? "qrc:/vox/icons/" + symbol + ".svg" : ""
    icon.width: 16
    icon.height: 16
    icon.color: foregroundColor
    palette.buttonText: foregroundColor
    palette.highlightedText: foregroundColor
    font.pixelSize: 13
    background: Rectangle {
        radius: 7
        color: control.primary ? (control.enabled ? Theme.accent : Theme.selected) : control.hovered ? Theme.hover : Theme.panel
        border.color: control.primary ? "transparent" : Theme.border
        Rectangle {
            anchors.fill: parent
            anchors.margins: -2
            radius: parent.radius + 2
            color: "transparent"
            border.color: Theme.secondary
            visible: control.enabled && control.visualFocus
        }
    }
}
