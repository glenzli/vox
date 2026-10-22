import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
Rectangle {
    id: sidebar
    color: Theme.window
    property string productionMode: "speech"
    signal modeRequested(string mode)
    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 14
        spacing: 8
        Label { text: qsTr("PRODUCTION"); color: Theme.secondary; font.pixelSize: 10; font.letterSpacing: 1.5 }
        Repeater {
            model: [
                {key: "speech", title: qsTr("Voice"), icon: "waveform"},
                {key: "sfx", title: qsTr("Sound effects"), icon: "headphones"},
                {key: "ambience", title: qsTr("Ambience"), icon: "volume"},
                {key: "short_music", title: qsTr("Short music"), icon: "play"}
            ]
            delegate: ActionButton {
                required property var modelData
                objectName: "productionMode-" + modelData.key
                Layout.fillWidth: true
                text: modelData.title; symbol: modelData.icon
                primary: sidebar.productionMode === modelData.key
                onClicked: sidebar.modeRequested(modelData.key)
            }
        }
        Label { text: qsTr("Start with a form"); color: Theme.secondary; visible: backend.path.length > 0 && !(backend.document.text || "").length; font.pixelSize: 12 }
        ActionButton { Layout.fillWidth: true; visible: sidebar.productionMode === "speech" && !(backend.document.text || "").length; text: qsTr("Narration"); symbol: "file"; enabled: !backend.busy && backend.path.length > 0; onClicked: backend.useTemplate("narration") }
        ActionButton { Layout.fillWidth: true; visible: sidebar.productionMode === "speech" && !(backend.document.text || "").length; text: qsTr("Dialogue"); symbol: "users"; enabled: !backend.busy && backend.path.length > 0; onClicked: backend.useTemplate("dialogue") }
        ActionButton { Layout.fillWidth: true; visible: sidebar.productionMode === "speech" && !(backend.document.text || "").length; text: qsTr("Listening"); symbol: "headphones"; enabled: !backend.busy && backend.path.length > 0; onClicked: backend.useTemplate("listening") }
        Label {
            visible: !!backend.project.shape_import
            Layout.fillWidth: true
            wrapMode: Text.WordWrap
            font.pixelSize: 11
            color: Theme.secondary
            text: backend.importedDraftKind === "speech" ? qsTr("Working draft: Shape speech") : backend.importedDraftKind === "authoring" ? qsTr("Working draft: Shape writing") : qsTr("Working draft: your changes")
        }
        Label {
            visible: !!backend.project.shape_import
            Layout.fillWidth: true
            wrapMode: Text.WordWrap
            font.pixelSize: 11
            color: Theme.muted
            text: qsTr("Restore changes only the working draft. Accepted audio stays available.")
        }
        Repeater {
            model: backend.project.shape_import ? backend.project.shape_import.drafts || [] : []
            delegate: ActionButton {
                required property var modelData
                objectName: "shapeDraft-" + modelData.kind
                Layout.fillWidth: true
                text: modelData.kind === "speech" ? qsTr("Restore Shape speech draft") : qsTr("Restore Shape writing draft")
                symbol: "file"
                enabled: !backend.busy
                onClicked: backend.restoreShapeDraft(modelData.kind)
            }
        }
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
                enabled: !backend.busy
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
