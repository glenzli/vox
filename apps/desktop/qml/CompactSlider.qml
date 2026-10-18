import QtQuick
import QtQuick.Controls
Slider {
    id: control
    implicitWidth: 160
    implicitHeight: 24
    leftPadding: 6
    rightPadding: 6
    background: Rectangle {
        x: control.leftPadding
        y: (control.height - height) / 2
        width: control.availableWidth
        height: 3
        radius: 1.5
        color: Theme.border
        Rectangle {
            width: control.visualPosition * parent.width
            height: parent.height
            radius: parent.radius
            color: control.enabled ? Theme.text : Theme.secondary
        }
    }
    handle: Rectangle {
        x: control.leftPadding + control.visualPosition * (control.availableWidth - width)
        y: (control.height - height) / 2
        implicitWidth: 12
        implicitHeight: 12
        radius: 6
        color: control.pressed ? Theme.selected : Theme.panel
        border.color: Theme.secondary
        Rectangle {
            anchors.fill: parent
            anchors.margins: -3
            radius: 9
            color: "transparent"
            border.color: Theme.secondary
            visible: control.visualFocus
        }
    }
}
