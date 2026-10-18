import QtQuick
import QtQuick.Controls
ComboBox {
    id: control
    implicitWidth: Math.max(112, implicitContentWidth + leftPadding + rightPadding)
    implicitHeight: 28
    leftPadding: mirrored ? 28 : 9
    rightPadding: mirrored ? 9 : 28
    topPadding: 4
    bottomPadding: 4
    font.pixelSize: 12
    hoverEnabled: true
    focusPolicy: Qt.StrongFocus
    ToolTip.visible: hovered && contentItem.truncated
    ToolTip.text: displayText
    ToolTip.delay: 600
    contentItem: Text {
        text: control.displayText
        font: control.font
        color: control.enabled ? Theme.text : Theme.secondary
        verticalAlignment: Text.AlignVCenter
        elide: Text.ElideRight
    }
    indicator: Canvas {
        x: control.mirrored ? 8 : control.width - width - 8
        y: (control.height - height) / 2
        width: 12
        height: 12
        property color ink: control.enabled ? Theme.secondary : Theme.muted
        onInkChanged: requestPaint()
        onPaint: {
            let ctx = getContext("2d")
            ctx.reset()
            ctx.strokeStyle = ink
            ctx.lineWidth = 1.5
            ctx.lineCap = "round"
            ctx.lineJoin = "round"
            ctx.beginPath()
            ctx.moveTo(3, 4.5)
            ctx.lineTo(6, 7.5)
            ctx.lineTo(9, 4.5)
            ctx.stroke()
        }
    }
    background: Rectangle {
        radius: 5
        color: control.down ? Theme.selected : control.hovered ? Theme.hover : Theme.panel
        border.color: control.visualFocus ? Theme.secondary : Theme.border
    }
    delegate: ItemDelegate {
        id: option
        required property var modelData
        required property int index
        width: ListView.view.width
        height: 28
        leftPadding: 9
        rightPadding: 9
        text: String(modelData)
        font: control.font
        highlighted: control.highlightedIndex === index
        hoverEnabled: control.hoverEnabled
        contentItem: Text {
            text: option.text
            font: option.font
            color: Theme.text
            verticalAlignment: Text.AlignVCenter
            elide: Text.ElideRight
        }
        background: Rectangle {
            radius: 3
            color: option.highlighted ? Theme.selected : option.hovered ? Theme.hover : "transparent"
        }
    }
    popup: Popup {
        y: control.height + 3
        width: control.width
        implicitHeight: Math.min(contentItem.implicitHeight, 280) + topPadding + bottomPadding
        topMargin: 8
        bottomMargin: 8
        padding: 4
        font: control.font
        contentItem: ListView {
            clip: true
            implicitHeight: contentHeight
            model: control.delegateModel
            currentIndex: control.highlightedIndex
            highlightMoveDuration: 0
            ScrollBar.vertical: ScrollBar { policy: ScrollBar.AsNeeded }
        }
        background: Rectangle { color: Theme.panel; radius: 6; border.color: Theme.border }
    }
}
