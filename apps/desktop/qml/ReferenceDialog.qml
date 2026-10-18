import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Dialogs as Native
Dialog {
    id: dialog
    objectName: "referenceDialog"
    title: qsTr("Add reference voice")
    modal: true
    closePolicy: backend.rendering ? Popup.NoAutoClose : Popup.CloseOnEscape
    width: Math.min(760, parent.width - 48)
    height: Math.min(650, parent.height - 48)
    anchors.centerIn: parent
    padding: 24
    property string operationError: ""
    footer: Item {
        implicitHeight: 70
        RowLayout {
            anchors.fill: parent; anchors.margins: 18
            Label { text: qsTr("Saved only in this Vox project."); color: Theme.secondary; Layout.fillWidth: true }
            ActionButton { text: qsTr("Close"); enabled: !backend.rendering; onClicked: dialog.close() }
            ActionButton { objectName: "saveReferenceButton"; text: qsTr("Save reference voice"); symbol: "check"; primary: true; enabled: dialog.projectPath === backend.path && !backend.rendering && !recorder.recording && !recorder.requesting && dialog.sourceFile.toString().length > 0 && voiceName.text.trim().length > 0 && transcript.text.trim().length > 0 && consent.checked; onClicked: backend.importReference(voiceName.text, transcript.text, dialog.sourceFile, consent.checked, recorder.file.toString().length > 0) }
        }
    }
    property url importedFile: ""
    property string projectPath: ""
    readonly property url sourceFile: recorder.file.toString().length ? recorder.file : importedFile
    background: Rectangle { color: Theme.panel; radius: 12; border.color: Theme.border }
    onOpened: { operationError = ""; projectPath = backend.path; importedFile = ""; recorder.cancel(); voiceName.text = ""; transcript.text = ""; consent.checked = false }
    onClosed: recorder.cancel()
    ScrollView {
        anchors.fill: parent
        clip: true
        contentWidth: availableWidth
        ColumnLayout {
            width: parent.width
            spacing: 16
            Label { text: qsTr("A voice from a recording"); font.pixelSize: 21; font.weight: Font.DemiBold }
            Label { text: qsTr("Record clear speech or import a mono/stereo PCM16 WAV. Include the exact words spoken; this is a reference for new speech, not model training."); Layout.fillWidth: true; wrapMode: Text.WordWrap; color: Theme.secondary }
            TextField { id: voiceName; objectName: "referenceName"; Layout.fillWidth: true; placeholderText: qsTr("Voice name"); maximumLength: 100; enabled: !backend.rendering }
            RowLayout {
                ComboBox { id: microphone; objectName: "microphoneChoice"; Layout.fillWidth: true; model: recorder.devices; displayText: count ? currentText : qsTr("No microphone"); enabled: !recorder.recording && !recorder.requesting && !backend.rendering }
                ActionButton { objectName: "recordReferenceButton"; symbol: recorder.recording ? "stop" : "waveform"; text: recorder.recording ? qsTr("Stop recording") : qsTr("Record"); enabled: !backend.rendering && !recorder.requesting && microphone.count > 0; onClicked: { if (recorder.recording) recorder.stop(); else { dialog.importedFile = ""; recorder.start(microphone.currentIndex) } } }
                ActionButton { symbol: "import"; text: qsTr("Import WAV"); enabled: !recorder.recording && !recorder.requesting && !backend.rendering; onClicked: referenceFile.open() }
            }
            RowLayout {
                visible: recorder.recording || recorder.requesting || recorder.milliseconds > 0
                Label { text: recorder.requesting ? qsTr("Waiting for microphone access…") : qsTr("%1 s · up to 60 s").arg((recorder.milliseconds / 1000).toFixed(1)); color: Theme.secondary }
                ProgressBar { Layout.fillWidth: true; from: 0; to: 1; value: recorder.level; visible: recorder.recording; Accessible.name: qsTr("Microphone level") }
            }
            Label { visible: recorder.error.length > 0; text: recorder.error; Layout.fillWidth: true; wrapMode: Text.WordWrap; color: Theme.text }
            RowLayout {
                visible: dialog.sourceFile.toString().length > 0
                Label { Layout.fillWidth: true; text: dialog.sourceFile.toString().split('/').pop(); elide: Text.ElideMiddle; color: Theme.secondary }
                IconButton { symbol: "play"; hint: qsTr("Listen to reference"); enabled: !recorder.recording && !backend.rendering; onClicked: backend.previewFile(dialog.sourceFile) }
            }
            Label { text: qsTr("Exact words in the reference"); font.weight: Font.DemiBold }
            ScrollView {
                Layout.fillWidth: true; Layout.preferredHeight: 155
                TextArea { id: transcript; objectName: "referenceTranscript"; wrapMode: TextEdit.Wrap; placeholderText: qsTr("Enter every spoken word from the reference, in its original language."); enabled: !backend.rendering; selectByMouse: true }
            }
            ActionButton { text: qsTr("Use sample text to read aloud"); symbol: "file"; enabled: !recorder.recording && !backend.rendering; onClicked: transcript.text = qsTr("Welcome to Vox. A quiet morning, a familiar voice, and a story waiting to be told.") }
            CheckBox { id: consent; objectName: "referenceConsent"; Layout.fillWidth: true; text: qsTr("I own this voice or have permission to use it for local, disclosed synthetic speech."); enabled: !backend.rendering; contentItem: Label { text: consent.text; wrapMode: Text.WordWrap; leftPadding: consent.indicator.width + consent.spacing; verticalAlignment: Text.AlignVCenter; color: Theme.text } }
            Label { visible: dialog.operationError.length > 0; text: dialog.operationError; Layout.fillWidth: true; wrapMode: Text.WordWrap; color: Theme.text }

        }
    }
    Native.FileDialog { id: referenceFile; title: qsTr("Import reference voice"); nameFilters: [qsTr("PCM16 WAV audio (*.wav)")]; onAccepted: { recorder.cancel(); dialog.importedFile = selectedFile } }
    Connections { target: backend; function onRequestFinished(action, ok) { if (dialog.opened && action === "import_reference") { if (ok) dialog.close(); else dialog.operationError = backend.message } } }
}
