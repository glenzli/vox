pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Dialogs as Native

Dialog {
    id: dialog
    objectName: "subtitleDialog"
    title: qsTr("Export subtitles")
    width: Math.min(460, parent.width - 48)
    anchors.centerIn: parent
    modal: true
    padding: 18
    property string audioId: ""
    property string projectPath: ""
    property string projectId: ""
    property string exportFormat: "srt"
    readonly property bool sameProject: projectPath === backend.path && projectId === (backend.project.id || "")
    function openForAudio(id) {
        audioId = id || backend.selectedAudio
        projectPath = backend.path
        projectId = backend.project.id || ""
        open()
    }
    background: Rectangle { color: Theme.panel; radius: 10; border.color: Theme.border }
    contentItem: ColumnLayout {
        spacing: 12
        Label { Layout.fillWidth: true; text: qsTr("Subtitle timings come from the measured audio of this assembled take. Text follows the authored script."); wrapMode: Text.WordWrap; color: Theme.secondary }
        CompactComboBox { id: format; objectName: "subtitleFormat"; Layout.fillWidth: true; model: [qsTr("SubRip (.srt)"), qsTr("WebVTT (.vtt)")] }
    }
    footer: Item {
        implicitHeight: 52
        RowLayout {
            anchors.fill: parent
            anchors.margins: 12
            Item { Layout.fillWidth: true }
            ActionButton { text: qsTr("Cancel"); onClicked: dialog.close() }
            ActionButton { objectName: "saveSubtitlesButton"; text: qsTr("Choose file…"); symbol: "export"; primary: true; enabled: dialog.sameProject && !!dialog.audioId && !backend.busy; onClicked: { dialog.exportFormat = format.currentIndex === 1 ? "vtt" : "srt"; subtitleFile.open() } }
        }
    }
    Native.FileDialog {
        id: subtitleFile
        title: qsTr("Export subtitles")
        fileMode: Native.FileDialog.SaveFile
        defaultSuffix: dialog.exportFormat
        nameFilters: dialog.exportFormat === "vtt" ? [qsTr("WebVTT subtitles (*.vtt)")] : [qsTr("SubRip subtitles (*.srt)")]
        onAccepted: { if (dialog.sameProject) backend.exportSubtitles(dialog.audioId, selectedFile, dialog.exportFormat); dialog.close() }
    }
    Connections { target: backend; function onStateChanged() { if (dialog.opened && !dialog.sameProject) dialog.close() } }
}
