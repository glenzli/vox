import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
Rectangle {
    id: sidebar
    color: Theme.window
    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 14
        spacing: 8
        Label { text: qsTr("PRODUCTION"); color: Theme.secondary; font.pixelSize: 10; font.letterSpacing: 1.5 }
        RowLayout {
            IconButton { symbol: "file"; hint: qsTr("Script"); enabled: false }
            Label { text: qsTr("Script"); font.weight: Font.Medium }
        }
        Label { text: qsTr("Start with a form"); color: Theme.secondary; visible: backend.path.length > 0 && !(backend.document.text || "").length; font.pixelSize: 12 }
        ActionButton { Layout.fillWidth: true; text: qsTr("Narration"); symbol: "file"; enabled: !backend.rendering && !(backend.document.text || "").length && backend.path.length > 0; onClicked: backend.useTemplate("narration") }
        ActionButton { Layout.fillWidth: true; text: qsTr("Dialogue"); symbol: "users"; enabled: !backend.rendering && !(backend.document.text || "").length && backend.path.length > 0; onClicked: backend.useTemplate("dialogue") }
        ActionButton { Layout.fillWidth: true; text: qsTr("Listening"); symbol: "headphones"; enabled: !backend.rendering && !(backend.document.text || "").length && backend.path.length > 0; onClicked: backend.useTemplate("listening") }
        Item { Layout.preferredHeight: 6 }
        Rectangle { Layout.fillWidth: true; height: 1; color: Theme.border }
        RowLayout {
            Label { text: qsTr("SCRIPT HISTORY"); color: Theme.secondary; font.pixelSize: 10; font.letterSpacing: 1.2; Layout.fillWidth: true }
            Label { text: (backend.project.sources || []).length; color: Theme.muted; font.pixelSize: 10 }
        }
        Label { text: qsTr("A script version is kept each time you render a revision."); visible: !(backend.project.sources || []).length; Layout.fillWidth: true; wrapMode: Text.WordWrap; color: Theme.muted; font.pixelSize: 11; lineHeight: 1.3 }
        ListView {
            Layout.fillWidth: true
            Layout.fillHeight: true
            spacing: 5
            clip: true
            model: (backend.project.sources || []).slice().reverse()
            delegate: ItemDelegate {
                id: history
                required property var modelData
                required property int index
                width: ListView.view.width
                height: 32
                text: qsTr("Script %1").arg((backend.project.sources || []).length-index)
                font.pixelSize: 12
                enabled: !backend.rendering
                highlighted: modelData.id === backend.project.source_head
                ToolTip.visible: hovered
                ToolTip.text: qsTr("Restore this script into the working draft")
                onClicked: backend.restoreSource(modelData.id)
                background: Rectangle { radius: 6; color: history.highlighted ? Theme.selected : history.hovered ? Theme.hover : "transparent" }
            }
        }
        RowLayout {
            Rectangle { width: 6; height: 6; radius: 3; color: backend.runtimeReady ? Theme.text : Theme.muted }
            Label { text: backend.runtimeReady ? qsTr("Local voice ready") : qsTr("Voice engine offline"); color: Theme.secondary; font.pixelSize: 10; Layout.fillWidth: true }
        }
        Label { text: qsTr("Vox · Development preview"); color: Theme.muted; font.pixelSize: 10 }
    }
}
