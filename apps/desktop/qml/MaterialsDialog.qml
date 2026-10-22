import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Dialogs as Native
Dialog {
    id: dialog
    objectName: "materialsDialog"
    title: qsTr("Voice & sound library")
    modal: true
    width: Math.min(860, parent.width - 48)
    height: Math.min(730, parent.height - 48)
    anchors.centerIn: parent
    padding: 18
    footer: Item {
        implicitHeight: 54
        RowLayout {
            anchors.fill: parent; anchors.margins: 12
            Label { text: backend.message; color: Theme.secondary; Layout.fillWidth: true; maximumLineCount: 2; wrapMode: Text.WordWrap; elide: Text.ElideRight }
            ActionButton { text: qsTr("Close"); onClicked: dialog.close() }
        }
    }
    property int initialTab: 0
    property string exportKind: "sound"
    property string exportId: ""
    property string projectPath: ""
    property string projectId: ""
    onOpened: { tabs.currentIndex = initialTab; projectPath = backend.path; projectId = backend.project.id || "" }
    function exportItem(kind, id) { exportKind = kind; exportId = id; exportFile.open() }
    background: Rectangle { color: Theme.panel; radius: 12; border.color: Theme.border }
    ColumnLayout {
        anchors.fill: parent
        spacing: 12
        TabBar {
            id: tabs
            Layout.fillWidth: true
            TabButton { id: voiceTab; implicitHeight: 30; text: qsTr("Reference voices"); contentItem: Label { text: voiceTab.text; color: Theme.text; horizontalAlignment: Text.AlignHCenter; verticalAlignment: Text.AlignVCenter }
                background: Rectangle { color: voiceTab.checked ? Theme.selected : Theme.panel; border.color: Theme.border; radius: 6 } }
            TabButton { id: soundTab; implicitHeight: 30; text: qsTr("Sound effects"); contentItem: Label { text: soundTab.text; color: Theme.text; horizontalAlignment: Text.AlignHCenter; verticalAlignment: Text.AlignVCenter }
                background: Rectangle { color: soundTab.checked ? Theme.selected : Theme.panel; border.color: Theme.border; radius: 6 } }
        }
        RowLayout {
            visible: backend.rendering
            BusyIndicator { running: visible; Layout.preferredWidth: 24; Layout.preferredHeight: 24 }
            Label { text: qsTr("Working locally…"); Layout.fillWidth: true }
            ActionButton { text: qsTr("Stop after current generation"); symbol: "stop"; onClicked: backend.cancel() }
        }
        ScrollView {
            Layout.fillWidth: true; Layout.fillHeight: true
            visible: tabs.currentIndex === 0
            clip: true; contentWidth: availableWidth
            ColumnLayout {
                width: parent.width
                spacing: 12
                RowLayout {
                    Label { text: qsTr("Your reference voices"); font.pixelSize: 17; font.weight: Font.DemiBold; Layout.fillWidth: true }
                    ActionButton { objectName: "addReferenceButton"; text: qsTr("Add voice"); symbol: "add"; enabled: !backend.busy; onClicked: reference.open() }
                }
                Label { text: qsTr("Record or import a voice, then choose it for narration or any role. References stay in this project."); wrapMode: Text.WordWrap; Layout.fillWidth: true; color: Theme.secondary }
                Label { visible: !(backend.project.voice_library || []).length; text: qsTr("No reference voices yet. Add a recording to begin."); color: Theme.muted }
                Repeater {
                    model: backend.project.voice_library || []
                    Rectangle {
                        id: voiceCard
                        required property var modelData
                        Layout.fillWidth: true; implicitHeight: voiceContent.implicitHeight + 28
                        color: Theme.inset; radius: 8; border.color: Theme.border
                        ColumnLayout {
                            id: voiceContent
                            anchors.fill: parent; anchors.margins: 14
                            spacing: 8
                            RowLayout {
                                Label { text: voiceCard.modelData.name; font.weight: Font.DemiBold; Layout.fillWidth: true; elide: Text.ElideRight }
                                IconButton { symbol: "play"; hint: qsTr("Listen to reference"); enabled: !backend.busy; onClicked: backend.playMaterial("voice", voiceCard.modelData.id) }
                                IconButton { symbol: "echo"; hint: qsTr("Edit a copy in Echo"); enabled: !backend.busy; onClicked: backend.editMaterialInEcho("voice", voiceCard.modelData.id) }
                                IconButton { symbol: "export"; hint: qsTr("Export original WAV"); enabled: !backend.busy; onClicked: dialog.exportItem("voice", voiceCard.modelData.id) }
                            }
                            Label { text: voiceCard.modelData.reference.transcript; Layout.fillWidth: true; wrapMode: Text.WordWrap; maximumLineCount: 3; elide: Text.ElideRight; color: Theme.secondary }
                            RowLayout {
                                Label { text: voiceCard.modelData.origin === "designed" ? qsTr("%1 s · synthetic voice").arg((voiceCard.modelData.contract.frame_count / voiceCard.modelData.contract.sample_rate_hz).toFixed(1)) : qsTr("%1 s · local reference").arg((voiceCard.modelData.contract.frame_count / voiceCard.modelData.contract.sample_rate_hz).toFixed(1)); font.pixelSize: 11; color: Theme.muted; Layout.fillWidth: true }
                                ActionButton { text: qsTr("Synthesize a preview"); symbol: "waveform"; enabled: !backend.busy && backend.runtimeReady; onClicked: backend.auditionReference(voiceCard.modelData.id, previewWords.text) }
                            }
                            Label { visible: !!voiceCard.modelData.design; text: voiceCard.modelData.design ? qsTr("Designed voice · Runtime job %1").arg(voiceCard.modelData.design.generation.job_id) : ""; Layout.fillWidth: true; wrapMode: Text.WrapAnywhere; font.pixelSize: 10; color: Theme.muted }
                        }
                    }
                }
                CompactTextField { id: previewWords; objectName: "referencePreviewText"; Layout.fillWidth: true; text: qsTr("Hello. Welcome to Vox. Every story begins with a voice."); maximumLength: 200; placeholderText: qsTr("Words for the synthesized preview"); enabled: !backend.busy }
                Label { text: qsTr("After editing in Echo, import its exported WAV as a new reference and confirm the spoken words."); Layout.fillWidth: true; wrapMode: Text.WordWrap; color: Theme.muted; font.pixelSize: 11 }
            }
        }
        SoundWorkspace {
            id: sounds
            Layout.fillWidth: true; Layout.fillHeight: true
            visible: tabs.currentIndex === 1
            showLoopControl: false
            onCategoryRequested: value => category = value
        }
    }
    Connections { target: backend; function onStateChanged() { if (dialog.opened && (dialog.projectPath !== backend.path || dialog.projectId !== (backend.project.id || ""))) dialog.close() } }
    ReferenceDialog { id: reference; parent: dialog.parent }
    Native.FileDialog { id: exportFile; title: qsTr("Export original WAV"); fileMode: Native.FileDialog.SaveFile; defaultSuffix: "wav"; nameFilters: [qsTr("WAV audio (*.wav)")]; onAccepted: { if (dialog.projectPath === backend.path && dialog.projectId === (backend.project.id || "")) backend.exportMaterial(dialog.exportKind, dialog.exportId, selectedFile) } }
}
