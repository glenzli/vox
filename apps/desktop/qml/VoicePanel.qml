import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Dialogs as Native
Rectangle {
    id: panel
    color: Theme.panel
    property string cueLabel: ""
    property var languages: ["auto", "Chinese", "English", "Japanese", "Korean", "German", "French", "Russian", "Portuguese", "Spanish", "Italian"]
    function voiceIndex(alias) {
        for (let i = 0; i < backend.voices.length; ++i) if (backend.voices[i].alias === alias) return i
        return -1
    }
    function voiceNames() {
        let result = []
        for (let v of backend.voices) result.push(backend.voiceLabel(v.key))
        return result
    }
    ScrollView {
        anchors.fill: parent
        anchors.margins: 20
        clip: true
        contentWidth: availableWidth
        ColumnLayout {
            width: parent.width
            spacing: 16
            Label { text: qsTr("VOICES & DIRECTION"); font.pixelSize: 10; font.letterSpacing: 1.5; color: Theme.secondary }
            Label { text: qsTr("Default voice"); font.weight: Font.DemiBold }
            RowLayout {
                Layout.fillWidth: true
                ComboBox {
                    id: defaultVoice
                    objectName: "defaultVoice"
                    Layout.fillWidth: true
                    model: { const localeDependency = preferences.language; return panel.voiceNames() }
                    currentIndex: panel.voiceIndex(backend.document.voice || "")
                    enabled: !backend.rendering
                    onActivated: backend.setVoice(backend.voices[currentIndex].alias)
                }
                IconButton { symbol: "play"; hint: qsTr("Audition this voice"); enabled: !backend.rendering && defaultVoice.currentIndex >= 0; onClicked: backend.auditionVoice(backend.voices[defaultVoice.currentIndex].alias) }
            }
            Label { text: qsTr("Language"); color: Theme.secondary }
            ComboBox {
                Layout.fillWidth: true
                model: { const localeDependency = preferences.language; return panel.languages.map(k => backend.languageLabel(k)) }
                currentIndex: panel.languages.indexOf(backend.document.language || "auto")
                enabled: !backend.rendering
                onActivated: backend.setLanguage(panel.languages[currentIndex])
            }
            RowLayout {
                Label { text: qsTr("Pace"); color: Theme.secondary; Layout.fillWidth: true }
                Label { text: ((backend.document.speed_milli || 1000) / 1000).toFixed(2) + "×"; font.pixelSize: 12 }
            }
            Slider {
                id: pace
                Layout.fillWidth: true
                from: 500; to: 2000; stepSize: 50
                value: backend.document.speed_milli || 1000
                enabled: !backend.rendering
                onMoved: backend.setSpeed(Math.round(value))
                Accessible.name: qsTr("Speech pace")
            }
            Rectangle { Layout.fillWidth: true; height: 1; color: Theme.border }
            Label { text: qsTr("Roles"); font.weight: Font.DemiBold; visible: backend.document.format === "production_script" }
            Label {
                text: qsTr("Declare roles in your script, then choose a voice for each one.")
                visible: backend.document.format === "production_script" && !(backend.analysis.roles || []).length
                wrapMode: Text.WordWrap; Layout.fillWidth: true; color: Theme.secondary; font.pixelSize: 12
            }
            Repeater {
                model: backend.document.format === "production_script" ? (backend.analysis.roles || []) : []
                ColumnLayout {
                    id: roleRow
                    required property string modelData
                    Layout.fillWidth: true
                    spacing: 7
                    Label { text: roleRow.modelData; font.pixelSize: 12; font.weight: Font.Medium }
                    RowLayout {
                        ComboBox {
                            id: roleChoice
                            Layout.fillWidth: true
                            model: { const localeDependency = preferences.language; return panel.voiceNames() }
                            currentIndex: panel.voiceIndex((((backend.document.options || {}).roles || {})[roleRow.modelData] || {}).alias || "")
                            displayText: currentIndex < 0 ? qsTr("Choose a voice") : currentText
                            enabled: !backend.rendering
                            onActivated: backend.bindRole(roleRow.modelData, backend.voices[currentIndex].alias)
                        }
                        IconButton { symbol: "play"; hint: qsTr("Audition role voice"); enabled: !backend.rendering && roleChoice.currentIndex >= 0; onClicked: backend.auditionVoice(backend.voices[roleChoice.currentIndex].alias) }
                    }
                }
            }
            Label { text: qsTr("Sound cues"); font.weight: Font.DemiBold; visible: (backend.analysis.cues || []).length > 0 && backend.document.format === "production_script" }
            Repeater {
                model: backend.document.format === "production_script" ? (backend.analysis.cues || []) : []
                ColumnLayout {
                    id: cueRow
                    required property string modelData
                    Layout.fillWidth: true
                    spacing: 7
                    Label { text: cueRow.modelData; font.pixelSize: 12 }
                    RowLayout {
                        ComboBox {
                            Layout.fillWidth: true
                            model: [qsTr("From script"), qsTr("Chime"), qsTr("Skip"), qsTr("Imported WAV")]
                            currentIndex: { let a = (((backend.document.options || {}).cues || {})[cueRow.modelData] || {}).action || "unresolved"; return ["unresolved", "chime", "skip", "audio"].indexOf(a) }
                            enabled: !backend.rendering
                            onActivated: { if (currentIndex === 3) { panel.cueLabel = cueRow.modelData; cueFile.open() } else backend.bindCue(cueRow.modelData, ["unresolved", "chime", "skip"][currentIndex]) }
                        }
                        IconButton { symbol: "import"; hint: qsTr("Import cue WAV"); enabled: !backend.rendering; onClicked: { panel.cueLabel = cueRow.modelData; cueFile.open() } }
                    }
                }
            }
            Item { Layout.preferredHeight: 12 }
            Label { Layout.fillWidth: true; text: qsTr("Generated speech is labeled as synthetic. Your accepted takes stay available when you revise the script."); wrapMode: Text.WordWrap; color: Theme.muted; font.pixelSize: 11; lineHeight: 1.35 }
        }
    }
    Native.FileDialog { id: cueFile; title: qsTr("Import sound cue"); nameFilters: [qsTr("WAV audio (*.wav)")]; onAccepted: backend.importCue(panel.cueLabel, selectedFile) }
}
