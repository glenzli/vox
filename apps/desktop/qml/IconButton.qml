import QtQuick
import QtQuick.Controls
ToolButton {
    id: control
    property string symbol: "add"
    property string hint: ""
    property bool primary: false
    readonly property color foregroundColor: !control.enabled ? Theme.secondary : control.primary ? Theme.accentText : Theme.text
    implicitWidth: 28
    implicitHeight: 28
    padding: 6
    icon.source: "qrc:/vox/icons/" + symbol + ".svg"
    icon.width: 16
    icon.height: 16
    icon.color: foregroundColor
    display: AbstractButton.IconOnly
    focusPolicy: Qt.StrongFocus
    Accessible.name: hint
    ToolTip.visible: hovered && hint.length > 0
    ToolTip.text: hint
    ToolTip.delay: 500
    background: Rectangle {
        radius: 5
        color: control.primary ? (control.enabled ? Theme.accent : Theme.selected) : control.down || control.checked ? Theme.selected : control.hovered || control.visualFocus ? Theme.hover : "transparent"
        border.color: control.visualFocus ? Theme.secondary : "transparent"
    }
}
