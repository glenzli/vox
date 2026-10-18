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
    padding: 24
    footer: Item {
        implicitHeight: 64
        RowLayout {
            anchors.fill: parent; anchors.margins: 18
            Label { text: backend.message; color: Theme.secondary; Layout.fillWidth: true; maximumLineCount: 2; wrapMode: Text.WordWrap; elide: Text.ElideRight }
            ActionButton { text: qsTr("Close"); onClicked: dialog.close() }
        }
    }
    property int initialTab: 0
    property string exportKind: "sound"
    property string exportId: ""
    onOpened: tabs.currentIndex = initialTab
    function exportItem(kind, id) { exportKind = kind; exportId = id; exportFile.open() }
    background: Rectangle { color: Theme.panel; radius: 12; border.color: Theme.border }
    ColumnLayout {
        anchors.fill: parent
        spacing: 16
        TabBar {
            id: tabs
            Layout.fillWidth: true
            TabButton { id: voiceTab; text: qsTr("Reference voices"); contentItem: Label { text: voiceTab.text; color: Theme.text; horizontalAlignment: Text.AlignHCenter; verticalAlignment: Text.AlignVCenter }
                background: Rectangle { color: voiceTab.checked ? Theme.selected : Theme.panel; border.color: Theme.border; radius: 6 } }
            TabButton { id: soundTab; text: qsTr("Sound effects"); contentItem: Label { text: soundTab.text; color: Theme.text; horizontalAlignment: Text.AlignHCenter; verticalAlignment: Text.AlignVCenter }
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
                spacing: 14
                RowLayout {
                    Label { text: qsTr("Your reference voices"); font.pixelSize: 20; font.weight: Font.DemiBold; Layout.fillWidth: true }
                    ActionButton { objectName: "addReferenceButton"; text: qsTr("Add voice"); symbol: "add"; enabled: !backend.rendering; onClicked: reference.open() }
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
                                IconButton { symbol: "play"; hint: qsTr("Listen to reference"); enabled: !backend.rendering; onClicked: backend.playMaterial("voice", voiceCard.modelData.id) }
                                IconButton { symbol: "echo"; hint: qsTr("Edit a copy in Echo"); enabled: !backend.rendering; onClicked: backend.editMaterialInEcho("voice", voiceCard.modelData.id) }
                                IconButton { symbol: "export"; hint: qsTr("Export original WAV"); enabled: !backend.rendering; onClicked: dialog.exportItem("voice", voiceCard.modelData.id) }
                            }
                            Label { text: voiceCard.modelData.reference.transcript; Layout.fillWidth: true; wrapMode: Text.WordWrap; maximumLineCount: 3; elide: Text.ElideRight; color: Theme.secondary }
                            RowLayout {
                                Label { text: qsTr("%1 s · local reference").arg((voiceCard.modelData.contract.frame_count / voiceCard.modelData.contract.sample_rate_hz).toFixed(1)); font.pixelSize: 11; color: Theme.muted; Layout.fillWidth: true }
                                ActionButton { text: qsTr("Synthesize a preview"); symbol: "waveform"; enabled: !backend.rendering && backend.runtimeReady; onClicked: backend.auditionReference(voiceCard.modelData.id, previewWords.text) }
                            }
                        }
                    }
                }
                TextField { id: previewWords; objectName: "referencePreviewText"; Layout.fillWidth: true; text: qsTr("Hello. Welcome to Vox. Every story begins with a voice."); maximumLength: 200; placeholderText: qsTr("Words for the synthesized preview"); enabled: !backend.rendering }
                Label { text: qsTr("After editing in Echo, import its exported WAV as a new reference and confirm the spoken words."); Layout.fillWidth: true; wrapMode: Text.WordWrap; color: Theme.muted; font.pixelSize: 11 }
            }
        }
        ScrollView {
            id: soundScroll
            Layout.fillWidth: true; Layout.fillHeight: true
            visible: tabs.currentIndex === 1
            clip: true; contentWidth: availableWidth
            ColumnLayout {
                width: parent.width
                spacing: 14
                Label { text: qsTr("Built-in sounds"); font.pixelSize: 20; font.weight: Font.DemiBold }
                GridLayout {
                    Layout.fillWidth: true; columns: 2; columnSpacing: 12; rowSpacing: 6
                    Repeater {
                        model: backend.builtinSounds
                        RowLayout {
                            id: builtin
                            required property var modelData
                            Layout.fillWidth: true
                            Label { text: builtin.modelData.label
                                Layout.fillWidth: true }
                            IconButton { symbol: "play"; hint: qsTr("Preview sound"); enabled: !backend.rendering; onClicked: backend.previewBuiltin(builtin.modelData.key) }
                            IconButton { symbol: "add"; hint: qsTr("Add to sound library"); enabled: !backend.rendering; onClicked: backend.addBuiltin(builtin.modelData.key) }
                        }
                    }
                }
                Rectangle { height: 1; color: Theme.border; Layout.fillWidth: true }
                Label { text: qsTr("Generate a sound"); font.pixelSize: 18; font.weight: Font.DemiBold }
                Label { text: qsTr("Describe a sound in English or Chinese. Listen to the candidate before accepting it."); Layout.fillWidth: true; wrapMode: Text.WordWrap; color: Theme.secondary }
                TextField { id: soundName; Layout.fillWidth: true; placeholderText: qsTr("Sound name"); maximumLength: 100; enabled: !backend.rendering }
                ScrollView { Layout.fillWidth: true; Layout.preferredHeight: 95; TextArea { id: soundPrompt; objectName: "soundPrompt"; wrapMode: TextEdit.Wrap; placeholderText: qsTr("A soft bell, followed by gentle rain. No music or speech."); enabled: !backend.rendering; selectByMouse: true } }
                RowLayout {
                    ComboBox { id: soundKind; model: [qsTr("Sound effect"), qsTr("Short music")]; enabled: !backend.rendering; Layout.fillWidth: true }
                    Label { text: qsTr("Seconds") }
                    SpinBox { id: seconds; from: 1; to: 30; value: 5; editable: true; enabled: !backend.rendering }
                    Label { text: qsTr("Seed") }
                    TextField { id: seed; text: "42"; Layout.preferredWidth: 100; validator: IntValidator { bottom: 0; top: 2147483647 }
                        enabled: !backend.rendering; Accessible.name: qsTr("Generation seed") }
                }
                RowLayout {
                    ActionButton { symbol: "import"; text: qsTr("Import sound WAV"); enabled: !backend.rendering; onClicked: soundFile.open() }
                    Item { Layout.fillWidth: true }
                    ActionButton { objectName: "generateSoundButton"; text: qsTr("Generate candidate"); symbol: "waveform"; primary: true; enabled: !backend.rendering && backend.runtimeReady && soundName.text.trim().length > 0 && soundPrompt.text.trim().length > 0 && seed.acceptableInput; onClicked: backend.generateSound(soundName.text, soundPrompt.text, seconds.value, Number(seed.text), soundKind.currentIndex === 1) }
                }
                Rectangle { height: 1; color: Theme.border; Layout.fillWidth: true }
                Label { text: qsTr("Project sounds"); font.pixelSize: 18; font.weight: Font.DemiBold }
                Label { text: qsTr("Accepting prepares a script-compatible copy and keeps the original. Choose accepted sounds beside each cue in your script."); Layout.fillWidth: true; wrapMode: Text.WordWrap; color: Theme.secondary }
                Repeater {
                    model: backend.project.sound_library || []
                    Rectangle {
                        id: soundCard
                        required property var modelData
                        Layout.fillWidth: true; implicitHeight: soundContent.implicitHeight + 24
                        color: Theme.inset; radius: 8; border.color: Theme.border
                        ColumnLayout {
                            id: soundContent
                            anchors.fill: parent; anchors.margins: 12
                            spacing: 8
                            RowLayout {
                                Label { Layout.fillWidth: true; text: {const locale=preferences.language; return soundCard.modelData.origin === "builtin" ? ((backend.builtinSounds.find(v => v.key === soundCard.modelData.name) || {}).label || soundCard.modelData.name) : soundCard.modelData.name}
                                    font.weight: Font.DemiBold; elide: Text.ElideRight }
                                Label { text: soundCard.modelData.content ? qsTr("Accepted") : qsTr("Candidate"); color: Theme.secondary; font.pixelSize: 11 }
                                IconButton { symbol: "play"; hint: qsTr("Preview sound"); enabled: !backend.rendering; onClicked: backend.playMaterial("sound", soundCard.modelData.id) }
                                IconButton { symbol: "echo"; hint: qsTr("Edit a copy in Echo"); enabled: !backend.rendering; onClicked: backend.editMaterialInEcho("sound", soundCard.modelData.id) }
                                IconButton { symbol: "export"; hint: qsTr("Export original WAV"); enabled: !backend.rendering; onClicked: dialog.exportItem("sound", soundCard.modelData.id) }
                            }
                            Label { visible: !!soundCard.modelData.generation; text: soundCard.modelData.generation ? soundCard.modelData.generation.spec.prompt : ""; Layout.fillWidth: true; wrapMode: Text.WordWrap; color: Theme.secondary }
                            Label { visible: !!soundCard.modelData.generation && soundCard.modelData.generation.prepared.original_prompt !== soundCard.modelData.generation.prepared.effective_prompt; text: soundCard.modelData.generation ? soundCard.modelData.generation.prepared.effective_prompt : ""; Layout.fillWidth: true; wrapMode: Text.WordWrap; color: Theme.muted; font.pixelSize: 11 }
                            RowLayout {
                                Label { text: qsTr("%1 s").arg((soundCard.modelData.contract.frame_count / soundCard.modelData.contract.sample_rate_hz).toFixed(1)); color: Theme.muted; Layout.fillWidth: true; font.pixelSize: 11 }
                                IconButton { symbol: "x"; hint: qsTr("Discard candidate"); visible: !soundCard.modelData.content; enabled: !backend.rendering; onClicked: backend.discardSound(soundCard.modelData.id) }
                                ActionButton { text: qsTr("Accept for scripts"); symbol: "check"; visible: !soundCard.modelData.content; enabled: !backend.rendering; onClicked: backend.acceptSound(soundCard.modelData.id) }
                            }
                        }
                    }
                }
                Label { text: qsTr("For trimming, fades and mixing, open a copy in Echo. Import its WAV export here as a new sound candidate."); Layout.fillWidth: true; wrapMode: Text.WordWrap; color: Theme.muted; font.pixelSize: 11 }
            }
        }
    }
    Connections { target: backend; function onRequestFinished(action, ok) { if (action === "generate_sound" && ok && dialog.opened) Qt.callLater(() => { soundScroll.contentItem.contentY = Math.max(0, soundScroll.contentItem.contentHeight - soundScroll.height) }) } }
    ReferenceDialog { id: reference; parent: dialog.parent }
    Native.FileDialog { id: soundFile; title: qsTr("Import sound WAV"); nameFilters: [qsTr("PCM16 WAV audio (*.wav)")]; onAccepted: backend.importSound(selectedFile) }
    Native.FileDialog { id: exportFile; title: qsTr("Export original WAV"); fileMode: Native.FileDialog.SaveFile; defaultSuffix: "wav"; nameFilters: [qsTr("WAV audio (*.wav)")]; onAccepted: backend.exportMaterial(dialog.exportKind, dialog.exportId, selectedFile) }
}
