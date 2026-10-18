import QtQuick
import QtQuick.Controls
Button {
    id: control
    property string symbol: ""
    property bool primary: false
    readonly property color foregroundColor: !control.enabled ? Theme.secondary : control.primary ? Theme.accentText : Theme.text
    implicitHeight: 28
    leftPadding: 10
    rightPadding: 10
    topPadding: 4
    bottomPadding: 4
    spacing: 6
    icon.source: symbol.length ? "qrc:/vox/icons/" + symbol + ".svg" : ""
    icon.width: 16
    icon.height: 16
    icon.color: foregroundColor
    palette.buttonText: foregroundColor
    palette.highlightedText: foregroundColor
    font.pixelSize: 12
    background: Rectangle {
        radius: 5
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
