import QtQuick
import QtQuick.Controls
TextField {
    id: control
    implicitHeight: 28
    leftPadding: 9
    rightPadding: 9
    topPadding: 4
    bottomPadding: 4
    font.pixelSize: 12
    color: Theme.text
    placeholderTextColor: Theme.secondary
    background: Rectangle {
        color: Theme.panel
        radius: 5
        border.color: control.activeFocus ? Theme.secondary : Theme.border
    }
}
