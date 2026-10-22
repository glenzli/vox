pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Dialogs as Native

Item {
    id: workspace
    property string projectIdentity: backend.path + ":" + (backend.project.id || "")
    property string selectedId: ""
    property string fileIdentity: ""
    property string exportId: ""
    readonly property var candidates: (backend.project.voice_design_candidates || []).slice().reverse()
    readonly property var runs: backend.project.voice_design_runs || []
    readonly property var lastRun: runs.length ? runs[runs.length - 1] : null
    onProjectIdentityChanged: {
        selectedId = ""
        if (description) description.clear()
        if (sample) sample.clear()
        if (saveDialog) saveDialog.close()
    }
    function openSave(id) {
        saveDialog.candidateId = id
        saveDialog.projectIdentity = projectIdentity
        voiceName.clear()
        saveDialog.open()
    }
    function exportCandidate(id) {
        exportId = id
        fileIdentity = projectIdentity
        exportFile.open()
    }
    ColumnLayout {
        anchors.fill: parent
        spacing: 12
        RowLayout {
            Layout.fillWidth: true
            Label { text: qsTr("Voice design"); font.pixelSize: 18; font.weight: Font.DemiBold }
            Item { Layout.fillWidth: true }
            ActionButton { objectName: "designVoiceLibraryButton"; text: qsTr("Voice library"); symbol: "users"; enabled: !backend.busy; onClicked: library.open() }
        }
        Label {
            Layout.fillWidth: true; wrapMode: Text.WordWrap
            text: qsTr("Describe a synthetic voice. Compare the samples, then save a named voice for narration or any role.")
            font.pixelSize: 12; color: Theme.secondary
        }
        RowLayout {
            Layout.fillWidth: true; Layout.fillHeight: true
            spacing: 14
            Rectangle {
                Layout.preferredWidth: Math.min(360, workspace.width * 0.44)
                Layout.fillHeight: true
                color: Theme.inset; radius: 7; border.color: Theme.border
                ScrollView {
                    anchors.fill: parent; anchors.margins: 14
                    contentWidth: availableWidth; clip: true
                    ColumnLayout {
                        width: parent.width; spacing: 10
                        Label { text: qsTr("Voice description"); font.pixelSize: 12; font.weight: Font.Medium }
                        ScrollView {
                            Layout.fillWidth: true; Layout.preferredHeight: 128
                            TextArea {
                                id: description; objectName: "voiceDesignDescription"
                                color: Theme.text; wrapMode: TextEdit.Wrap; selectByMouse: true
                                placeholderText: qsTr("A warm, low voice with clear diction and an unhurried pace.")
                                enabled: !backend.busy
                                onTextChanged: { if (length > 1800) remove(1800, length) }
                                background: Rectangle { color: Theme.panel; radius: 5; border.color: description.activeFocus ? Theme.secondary : Theme.border }
                            }
                        }
                        Label { text: qsTr("Sample words"); font.pixelSize: 12; font.weight: Font.Medium }
                        ScrollView {
                            Layout.fillWidth: true; Layout.preferredHeight: 116
                            TextArea {
                                id: sample; objectName: "voiceDesignSample"
                                color: Theme.text; wrapMode: TextEdit.Wrap; selectByMouse: true
                                placeholderText: qsTr("Enter the exact words each candidate should say.")
                                enabled: !backend.busy
                                onTextChanged: { if (length > 1800) remove(1800, length) }
                                background: Rectangle { color: Theme.panel; radius: 5; border.color: sample.activeFocus ? Theme.secondary : Theme.border }
                            }
                        }
                        RowLayout {
                            Layout.fillWidth: true
                            Label { text: qsTr("Language"); font.pixelSize: 12 }
                            CompactComboBox {
                                id: language; objectName: "voiceDesignLanguage"
                                Layout.fillWidth: true
                                model: backend.languages.map(v => v.label)
                                enabled: !backend.busy
                            }
                        }
                        RowLayout {
                            Label { text: qsTr("Candidates"); Layout.fillWidth: true; font.pixelSize: 12 }
                            CompactSpinBox { id: count; objectName: "voiceDesignCount"; from: 1; to: 4; value: 2; enabled: !backend.busy; Layout.preferredWidth: 82 }
                        }
                        ActionButton {
                            objectName: "generateVoiceDesignButton"
                            Layout.fillWidth: true
                            text: qsTr("Generate %1 candidates").arg(count.value)
                            symbol: "waveform"; primary: true
                            enabled: !backend.busy && backend.runtimeReady && description.text.trim().length > 0 && sample.text.trim().length > 0
                            onClicked: backend.generateVoiceDesign(description.text, sample.text, backend.languages[Math.max(0, language.currentIndex)].key, count.value)
                        }
                        ColumnLayout {
                            visible: backend.designingVoice; Layout.fillWidth: true
                            ProgressBar { Layout.fillWidth: true; from: 0; to: Math.max(1,backend.total); value: backend.completed; indeterminate: backend.total <= 0 }
                            Label { text: qsTr("%1 / %2 candidates completed").arg(backend.completed).arg(backend.total); font.pixelSize: 11; color: Theme.secondary }
                            ActionButton { objectName: "stopVoiceDesignButton"; Layout.fillWidth: true; text: qsTr("Stop after current candidate"); symbol: "stop"; onClicked: backend.cancel() }
                        }
                        Label { Layout.fillWidth: true; text: qsTr("Completed candidates remain available if generation stops. Each saved voice is labelled synthetic."); wrapMode: Text.WordWrap; color: Theme.muted; font.pixelSize: 11 }
                    }
                }
            }
            ColumnLayout {
                Layout.fillWidth: true; Layout.fillHeight: true; spacing: 8
                RowLayout {
                    Label { text: qsTr("Voice candidates"); font.pixelSize: 13; font.weight: Font.DemiBold }
                    Label { text: workspace.candidates.length; color: Theme.muted; font.pixelSize: 11 }
                    Item { Layout.fillWidth: true }
                    Label { objectName: "voiceDesignRunStatus"; visible: !!workspace.lastRun; text: { const language = preferences.language; return workspace.lastRun ? backend.productionStatus(workspace.lastRun.status) : "" } color: Theme.secondary; font.pixelSize: 11 }
                }
                ListView {
                    id: candidateList; objectName: "voiceDesignCandidateList"
                    Layout.fillWidth: true; Layout.fillHeight: true
                    spacing: 7; clip: true; model: workspace.candidates
                    ScrollBar.vertical: ScrollBar { }
                    delegate: Rectangle {
                        id: candidate
                        required property var modelData
                        required property int index
                        property bool detailsOpen: false
                        width: candidateList.width; height: contents.implicitHeight + 22
                        color: workspace.selectedId === modelData.id ? Theme.selected : Theme.inset
                        border.color: Theme.border; radius: 6
                        ColumnLayout {
                            id: contents
                            anchors.fill: parent; anchors.margins: 11; spacing: 6
                            RowLayout {
                                Layout.fillWidth: true
                                IconButton { objectName: "voiceDesignPlay-" + candidate.index; symbol: "play"; hint: qsTr("Listen to candidate"); enabled: !backend.busy; onClicked: { workspace.selectedId = candidate.modelData.id; backend.playMaterial("voice_design", candidate.modelData.id) } }
                                Label { text: qsTr("Candidate %1").arg(candidate.modelData.variation); font.weight: Font.Medium; Layout.fillWidth: true }
                                Label { objectName: "voiceDesignSynthetic-" + candidate.index; text: qsTr("Synthetic voice"); font.pixelSize: 10; color: Theme.secondary }
                                IconButton { symbol: "help"; hint: qsTr("Voice provenance"); checkable: true; checked: candidate.detailsOpen; onClicked: candidate.detailsOpen = checked }
                            }
                            Label { text: candidate.modelData.generation.spec.description; Layout.fillWidth: true; wrapMode: Text.WordWrap; maximumLineCount: candidate.detailsOpen ? 100 : 2; elide: Text.ElideRight; color: Theme.secondary; font.pixelSize: 12 }
                            Label { text: candidate.modelData.generation.spec.text; Layout.fillWidth: true; wrapMode: Text.WordWrap; maximumLineCount: candidate.detailsOpen ? 100 : 2; elide: Text.ElideRight; font.pixelSize: 12 }
                            Label { visible: candidate.detailsOpen; text: qsTr("%1 · %2 s · Runtime job %3").arg(backend.languageLabel(candidate.modelData.generation.spec.language)).arg((candidate.modelData.contract.frame_count / candidate.modelData.contract.sample_rate_hz).toFixed(1)).arg(candidate.modelData.generation.job_id); Layout.fillWidth: true; wrapMode: Text.WrapAnywhere; color: Theme.muted; font.pixelSize: 11 }
                            Label { visible: candidate.detailsOpen; text: qsTr("Source digest: %1").arg(candidate.modelData.generation.output_digest); Layout.fillWidth: true; wrapMode: Text.WrapAnywhere; color: Theme.muted; font.pixelSize: 10 }
                            RowLayout {
                                Layout.fillWidth: true
                                Label { visible: !!candidate.modelData.accepted_reference; text: qsTr("Saved in voice library"); color: Theme.secondary; font.pixelSize: 11; Layout.fillWidth: true }
                                Item { visible: !candidate.modelData.accepted_reference; Layout.fillWidth: true }
                                IconButton { symbol: "echo"; hint: qsTr("Edit a copy in Echo"); enabled: !backend.busy; onClicked: backend.editMaterialInEcho("voice_design", candidate.modelData.id) }
                                IconButton { symbol: "export"; hint: qsTr("Export original WAV"); enabled: !backend.busy; onClicked: workspace.exportCandidate(candidate.modelData.id) }
                                IconButton { symbol: "x"; hint: qsTr("Discard candidate"); visible: !candidate.modelData.accepted_reference; enabled: !backend.busy; onClicked: backend.discardVoiceDesign(candidate.modelData.id) }
                                ActionButton { objectName: "saveDesignedVoice-" + candidate.index; text: qsTr("Save voice…"); symbol: "check"; primary: true; visible: !candidate.modelData.accepted_reference; enabled: !backend.busy; onClicked: workspace.openSave(candidate.modelData.id) }
                            }
                        }
                    }
                    Label { anchors.centerIn: parent; width: Math.min(parent.width - 24, 360); visible: !workspace.candidates.length; text: qsTr("Describe a voice and generate samples to compare here."); wrapMode: Text.WordWrap; horizontalAlignment: Text.AlignHCenter; color: Theme.muted; font.pixelSize: 12 }
                }
            }
        }
    }
    MaterialsDialog { id: library; initialTab: 0; parent: Overlay.overlay }
    Dialog {
        id: saveDialog; objectName: "saveDesignedVoiceDialog"
        property string candidateId: ""
        property string projectIdentity: ""
        title: qsTr("Save synthetic voice")
        anchors.centerIn: Overlay.overlay; modal: true; width: 420
        background: Rectangle { color: Theme.panel; radius: 8; border.color: Theme.border }
        ColumnLayout {
            anchors.fill: parent; spacing: 12
            Label { text: qsTr("Give this voice a name. Its synthetic origin, sample words and Runtime receipt stay with the reference."); Layout.fillWidth: true; wrapMode: Text.WordWrap; color: Theme.secondary }
            CompactTextField { id: voiceName; objectName: "designedVoiceName"; Layout.fillWidth: true; placeholderText: qsTr("Voice name"); maximumLength: 100; onAccepted: if (text.trim().length) saveNamedVoice.clicked() }
            RowLayout {
                Item { Layout.fillWidth: true }
                ActionButton { text: qsTr("Cancel"); onClicked: saveDialog.close() }
                ActionButton { id: saveNamedVoice; objectName: "confirmSaveDesignedVoice"; text: qsTr("Save voice"); symbol: "check"; primary: true; enabled: !backend.busy && voiceName.text.trim().length > 0; onClicked: { if (saveDialog.projectIdentity === workspace.projectIdentity) backend.acceptVoiceDesign(saveDialog.candidateId, voiceName.text); saveDialog.close() } }
            }
        }
        onOpened: voiceName.forceActiveFocus()
    }
    Native.FileDialog { id: exportFile; title: qsTr("Export voice sample WAV"); fileMode: Native.FileDialog.SaveFile; defaultSuffix: "wav"; nameFilters: [qsTr("WAV audio (*.wav)")]; onAccepted: if (workspace.fileIdentity === workspace.projectIdentity) backend.exportMaterial("voice_design", workspace.exportId, selectedFile) }
}
