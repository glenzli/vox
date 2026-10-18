import QtQuick
import QtQuick.Controls
SpinBox {
    id: control
    implicitWidth: 100
    implicitHeight: 28
    leftPadding: 28
    rightPadding: 28
    font.pixelSize: 12
    contentItem: TextInput {
        text: control.displayText
        font: control.font
        color: control.enabled ? Theme.text : Theme.secondary
        selectionColor: Theme.selected
        selectedTextColor: Theme.text
        horizontalAlignment: Text.AlignHCenter
        verticalAlignment: Text.AlignVCenter
        readOnly: !control.editable
        validator: control.validator
        inputMethodHints: control.inputMethodHints
        selectByMouse: true
        clip: true
    }
    up.indicator: Rectangle {
        x: control.mirrored ? 0 : control.width - width
        width: 26
        height: control.height
        radius: 5
        color: control.up.pressed ? Theme.selected : control.up.hovered ? Theme.hover : "transparent"
        Text { anchors.centerIn: parent; text: "+"; font.pixelSize: 16; color: control.up.enabled ? Theme.text : Theme.muted }
    }
    down.indicator: Rectangle {
        x: control.mirrored ? control.width - width : 0
        width: 26
        height: control.height
        radius: 5
        color: control.down.pressed ? Theme.selected : control.down.hovered ? Theme.hover : "transparent"
        Text { anchors.centerIn: parent; text: "−"; font.pixelSize: 16; color: control.down.enabled ? Theme.text : Theme.muted }
    }
    background: Rectangle {
        color: Theme.panel
        radius: 5
        border.color: control.activeFocus ? Theme.secondary : Theme.border
    }
}
