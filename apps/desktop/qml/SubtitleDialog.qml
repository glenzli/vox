pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Dialogs as Native

Dialog {
    id: dialog
    objectName: "subtitleDialog"
    title: qsTr("Subtitles")
    width: Math.min(760, parent.width - 48)
    height: Math.min(700, parent.height - 56)
    anchors.centerIn: parent
    modal: true
    padding: 16
    closePolicy: delivery.aligning ? Popup.NoAutoClose : Popup.CloseOnEscape
    readonly property var delivery: backend.delivery
    readonly property var languageKeys: backend.languages.filter(item => item.key !== "auto").map(item => item.key)
    readonly property var candidateIDs: (delivery.subtitleCandidates || []).map(item => item.id)
    property string exportFormat: "srt"
    property int reviewGeneration: 0
    property int exportGeneration: -1
    property string exportTrackID: ""
    property string exportAudioID: ""
    function openForAudio(id) {
        reviewGeneration += 1
        confirmed.checked = false
        delivery.openSubtitles(id || backend.selectedAudio)
        if (delivery.subtitleActive) open()
    }
    function time(ms) {
        const value = Math.max(0, Math.round(ms || 0))
        return Math.floor(value / 60000).toString().padStart(2, "0") + ":" + Math.floor(value / 1000 % 60).toString().padStart(2, "0") + "." + (value % 1000).toString().padStart(3, "0")
    }
    onClosed: { reviewGeneration += 1; delivery.closeSubtitles() }
    background: Rectangle { color: Theme.panel; radius: 10; border.color: Theme.border }
    contentItem: ColumnLayout {
        spacing: 8
        RowLayout {
            Layout.fillWidth: true
            Label { objectName: "subtitleSourceLabel"; text: { const language = preferences.language; return delivery.subtitleSourceLabel(delivery.subtitleTrack.source || "") } font.weight: Font.DemiBold; font.pixelSize: 12; Layout.fillWidth: true }
            Label { text: qsTr("%1 cues").arg((delivery.subtitleTrack.cues || []).length); color: Theme.secondary; font.pixelSize: 11 }
            IconButton { objectName: "subtitlePlayAudioButton"; symbol: "play"; hint: qsTr("Listen to this audio"); enabled: !!delivery.audioID && !delivery.aligning; onClicked: { backend.selectAudio(delivery.audioID); backend.playSelected() } }
        }
        Label { text: qsTr("Measured timings follow the assembled segments. Aligned timings are model estimates; review them before accepting."); Layout.fillWidth: true; wrapMode: Text.WordWrap; color: Theme.secondary; font.pixelSize: 11 }
        RowLayout {
            Layout.fillWidth: true
            CompactComboBox {
                id: candidateChoice
                objectName: "subtitleCandidateChoice"
                Layout.fillWidth: true
                model: [qsTr("Current timings")].concat((dialog.delivery.subtitleCandidates || []).map((item, index) => qsTr("Candidate %1 · %2").arg(index + 1).arg(backend.languageLabel(item.language || ""))))
                selectionIndex: dialog.delivery.candidateID.length ? dialog.candidateIDs.indexOf(dialog.delivery.candidateID) + 1 : 0
                enabled: !backend.busy && !dialog.delivery.subtitleLoading
                onActivated: index => dialog.delivery.previewSubtitleCandidate(index > 0 ? dialog.candidateIDs[index - 1] : "")
            }
            ActionButton { objectName: "acceptSubtitleCandidateButton"; text: qsTr("Accept timings"); symbol: "check"; primary: true; visible: dialog.delivery.candidateID.length > 0; enabled: !backend.busy && !dialog.delivery.subtitleLoading && (dialog.delivery.subtitleTrack.cues || []).length > 0; onClicked: dialog.delivery.acceptSubtitleCandidate() }
            IconButton { objectName: "discardSubtitleCandidateButton"; symbol: "x"; hint: qsTr("Discard candidate timings"); visible: dialog.delivery.candidateID.length > 0; enabled: !backend.busy && !dialog.delivery.subtitleLoading; onClicked: dialog.delivery.discardSubtitleCandidate() }
        }
        Rectangle {
            Layout.fillWidth: true
            Layout.fillHeight: true
            Layout.minimumHeight: 105
            color: Theme.inset; radius: 5; border.color: Theme.border
            ListView {
                id: cueList
                objectName: "subtitleCueList"
                anchors.fill: parent; anchors.margins: 8
                clip: true; spacing: 7
                model: dialog.delivery.subtitleTrack.cues || []
                ScrollBar.vertical: ScrollBar { }
                delegate: RowLayout {
                    id: cue
                    required property var modelData
                    required property int index
                    width: cueList.width; spacing: 12
                    Label { text: dialog.time(cue.modelData.start_ms) + " – " + dialog.time(cue.modelData.end_ms); Layout.alignment: Qt.AlignTop; font.family: "monospace"; font.pixelSize: 10; color: Theme.secondary }
                    TextEdit { objectName: "subtitleCueText-" + cue.index; text: cue.modelData.text; readOnly: true; selectByMouse: true; wrapMode: TextEdit.Wrap; color: Theme.text; font.pixelSize: 12; Layout.fillWidth: true }
                }
            }
            Label { anchors.centerIn: parent; visible: !cueList.count; text: dialog.delivery.subtitleLoading ? qsTr("Loading timings…") : qsTr("No current timings for this audio"); color: Theme.muted; font.pixelSize: 11 }
        }
        Label { text: qsTr("Exact spoken transcript"); font.weight: Font.DemiBold; font.pixelSize: 12 }
        Label { text: qsTr("Check these words against the audio. After Echo cuts or edits, update the transcript to match exactly. This does not change the script."); Layout.fillWidth: true; wrapMode: Text.WordWrap; color: Theme.secondary; font.pixelSize: 11 }
        ScrollView {
            Layout.fillWidth: true; Layout.preferredHeight: 100
            TextArea {
                id: transcript
                objectName: "subtitleTranscript"
                text: dialog.delivery.transcript
                color: Theme.text; selectionColor: Theme.selected; selectedTextColor: Theme.text
                wrapMode: TextEdit.Wrap; selectByMouse: true; font.pixelSize: 12
                readOnly: backend.busy || dialog.delivery.subtitleLoading
                placeholderText: qsTr("Enter the words actually spoken in this audio")
                onTextChanged: { if (dialog.delivery.transcript !== text) { dialog.delivery.transcript = text; confirmed.checked = false } }
                background: Rectangle { color: Theme.inset; radius: 5; border.color: Theme.border }
            }
        }
        RowLayout {
            Layout.fillWidth: true; spacing: 8
            CompactComboBox {
                id: language
                objectName: "subtitleLanguage"
                Layout.preferredWidth: 160
                model: backend.languages.filter(item => item.key !== "auto").map(item => item.label)
                selectionIndex: dialog.languageKeys.indexOf(dialog.delivery.language)
                displayText: currentIndex < 0 ? qsTr("Choose language") : currentText
                enabled: !backend.busy && !dialog.delivery.subtitleLoading
                onActivated: index => { dialog.delivery.language = dialog.languageKeys[index]; confirmed.checked = false }
            }
            CheckBox { id: confirmed; objectName: "subtitleTranscriptConfirmed"; text: qsTr("These words match this audio"); font.pixelSize: 11; Layout.fillWidth: true; enabled: !backend.busy && !dialog.delivery.subtitleLoading }
            ActionButton { objectName: "alignSubtitlesButton"; text: qsTr("Realign"); symbol: "refresh"; enabled: dialog.delivery.acceptedAudio && !backend.busy && !dialog.delivery.subtitleLoading && backend.runtimeReady && confirmed.checked && transcript.text.trim().length > 0 && dialog.delivery.language.length > 0; onClicked: dialog.delivery.alignSubtitles(confirmed.checked) }
        }
        ProgressBar { Layout.fillWidth: true; visible: dialog.delivery.aligning; from: 0; to: Math.max(1, backend.total); value: backend.completed; indeterminate: backend.completed <= 0 }
        RowLayout {
            Layout.fillWidth: true
            Label { objectName: "subtitleStatusMessage"; text: dialog.delivery.subtitleLoading ? qsTr("Loading timings…") : dialog.delivery.subtitleMessage || (!dialog.delivery.acceptedAudio ? qsTr("Accept this audio version before realigning.") : !backend.runtimeReady ? qsTr("Connect Runtime to realign. Saved timings can still be exported.") : ""); Layout.fillWidth: true; wrapMode: Text.WordWrap; color: Theme.secondary; font.pixelSize: 11 }
            ActionButton { objectName: "stopSubtitleAlignmentButton"; text: qsTr("Stop alignment"); symbol: "stop"; visible: dialog.delivery.aligning; onClicked: dialog.delivery.cancelAlignment() }
        }
    }
    footer: Item {
        implicitHeight: 52
        RowLayout {
            anchors.fill: parent; anchors.margins: 12; spacing: 8
            CompactComboBox { id: format; objectName: "subtitleFormat"; Layout.preferredWidth: 146; model: [qsTr("SubRip (.srt)"), qsTr("WebVTT (.vtt)")]; enabled: !backend.busy }
            Item { Layout.fillWidth: true }
            ActionButton { text: qsTr("Close"); enabled: !dialog.delivery.aligning; onClicked: dialog.close() }
            ActionButton { objectName: "saveSubtitlesButton"; text: qsTr("Export subtitles…"); symbol: "export"; primary: true; enabled: dialog.delivery.canExportSubtitles && !backend.busy; onClicked: { dialog.exportFormat = format.currentIndex === 1 ? "vtt" : "srt"; dialog.exportGeneration = dialog.reviewGeneration; dialog.exportAudioID = dialog.delivery.audioID; dialog.exportTrackID = dialog.delivery.subtitleTrack.alignment_revision || "measured_segments"; subtitleFile.open() } }
        }
    }
    Native.FileDialog {
        id: subtitleFile
        title: qsTr("Export subtitles")
        fileMode: Native.FileDialog.SaveFile
        defaultSuffix: dialog.exportFormat
        nameFilters: dialog.exportFormat === "vtt" ? [qsTr("WebVTT subtitles (*.vtt)")] : [qsTr("SubRip subtitles (*.srt)")]
        onAccepted: { if (dialog.exportGeneration === dialog.reviewGeneration && dialog.exportAudioID === dialog.delivery.audioID && dialog.exportTrackID === (dialog.delivery.subtitleTrack.alignment_revision || "measured_segments") && dialog.delivery.canExportSubtitles) dialog.delivery.exportSubtitles(selectedFile, dialog.exportFormat) }
    }
    Connections {
        target: dialog.delivery
        function onSubtitleChanged() { if (dialog.opened && !dialog.delivery.subtitleActive) dialog.close() }
        function onFormChanged() { if (transcript.text !== dialog.delivery.transcript) transcript.text = dialog.delivery.transcript; confirmed.checked = false }
    }
}
