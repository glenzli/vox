import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Dialogs as Native
Rectangle {
    id: panel
    color: Theme.panel
    property string cueLabel: ""
    property var languages: backend.languages.map(v => v.key)
    readonly property var references: backend.project.voice_library || []
    readonly property bool usesReference: !!backend.document.reference_voice || (backend.document.format === "production_script" && Object.keys((backend.document.options || {}).reference_roles || {}).length > 0)
    function choices() {
        let items = []
        for (let voice of backend.voices) items.push({id: voice.alias, name: voice.label, reference: false})
        for (let voice of references) items.push({id: voice.id, name: qsTr("Reference · %1").arg(voice.name), reference: true})
        return items
    }
    function selected(role) {
        let options = backend.document.options || {}
        let reference = role.length ? (options.reference_roles || {})[role] : backend.document.reference_voice
        return reference ? reference.source_revision_id : role.length ? ((options.roles || {})[role] || {}).alias || "" : backend.document.voice || ""
    }
    function voiceIndex(id) { return choices().findIndex(v => v.id === id) }
    function voiceNames() { return choices().map(v => v.name) }
    function choose(role, index) {
        let item = choices()[index]
        if (item.reference) backend.bindReference(role, item.id)
        else if (role.length) backend.bindRole(role, item.id)
        else backend.setVoice(item.id)
    }
    function audition(index) {
        let item = choices()[index]
        if (item.reference) backend.auditionReference(item.id, qsTr("Hello. Welcome to Vox. Every story begins with a voice."))
        else backend.auditionVoice(item.id)
    }
    function sounds() { return (backend.project.sound_library || []).filter(s => !!s.content) }
    function cueNames() { return [qsTr("From script"), qsTr("Chime"), qsTr("Skip"), qsTr("Audio / open library…")].concat(sounds().map(s => s.origin === "builtin" ? ((backend.builtinSounds.find(v => v.key === s.name) || {}).label || s.name) : s.name)) }
    function cueIndex(label) {
        let cue = ((backend.document.options || {}).cues || {})[label] || {}
        if (cue.action !== "audio") return ["unresolved", "chime", "skip"].indexOf(cue.action || "unresolved")
        let index = sounds().findIndex(s => s.content.digest === cue.content.digest)
        return index < 0 ? 3 : 4 + index
    }
    function openLibrary(tab) { materials.initialTab = tab; materials.open() }
    ScrollView {
        anchors.fill: parent
        anchors.margins: 16
        clip: true
        contentWidth: availableWidth
        ColumnLayout {
            width: parent.width
            spacing: 14
            RowLayout {
                Layout.fillWidth: true
                spacing: 4
                Label { text: qsTr("VOICES & DIRECTION"); font.pixelSize: 10; font.letterSpacing: 1; color: Theme.secondary; Layout.fillWidth: true }
                IconButton { objectName: "voiceLibraryButton"; symbol: "users"; hint: qsTr("Voice library"); enabled: !backend.busy; onClicked: panel.openLibrary(0) }
                IconButton { objectName: "soundLibraryButton"; symbol: "headphones"; hint: qsTr("Sounds"); enabled: !backend.busy; onClicked: panel.openLibrary(1) }
            }
            ColumnLayout {
                Layout.fillWidth: true
                spacing: 6
                Label { text: qsTr("Default voice"); font.pixelSize: 11; color: Theme.secondary }
                RowLayout {
                    Layout.fillWidth: true
                    spacing: 6
                    CompactComboBox {
                        id: defaultVoice
                        objectName: "defaultVoice"
                        Layout.fillWidth: true
                        model: { const localeDependency = preferences.language; return panel.voiceNames() }
                        selectionIndex: panel.voiceIndex(panel.selected(""))
                        enabled: !backend.busy
                        onActivated: panel.choose("", currentIndex)
                    }
                    IconButton { symbol: "play"; hint: qsTr("Audition this voice"); enabled: !backend.busy && defaultVoice.currentIndex >= 0; onClicked: panel.audition(defaultVoice.currentIndex) }
                }
            }
            RowLayout {
                Layout.fillWidth: true
                spacing: 12
                Label { text: qsTr("Language"); color: Theme.secondary; font.pixelSize: 12 }
                CompactComboBox {
                    Layout.fillWidth: true
                    model: backend.languages.map(v => v.label)
                    selectionIndex: panel.languages.indexOf(backend.document.language || "auto")
                    enabled: !backend.busy
                    onActivated: backend.setLanguage(panel.languages[currentIndex])
                }
            }
            ColumnLayout {
                Layout.fillWidth: true
                spacing: 2
                RowLayout {
                    Label { text: qsTr("Pace"); color: Theme.secondary; font.pixelSize: 12; Layout.fillWidth: true }
                    Label { text: ((backend.document.speed_milli || 1000) / 1000).toFixed(2) + "×"; font.pixelSize: 11; color: Theme.secondary }
                }
                CompactSlider {
                    id: pace
                    Layout.fillWidth: true
                    from: 500; to: 2000; stepSize: 50
                    value: backend.document.speed_milli || 1000
                    enabled: !backend.busy && !panel.usesReference
                    onMoved: backend.setSpeed(Math.round(value))
                    Accessible.name: qsTr("Speech pace")
                }
                Label { visible: panel.usesReference; Layout.fillWidth: true; text: qsTr("Reference voices use 1× pace and neutral delivery."); wrapMode: Text.WordWrap; color: Theme.muted; font.pixelSize: 11 }
            }
            Rectangle { Layout.fillWidth: true; height: 1; color: Theme.border; visible: backend.document.format === "production_script" }
            ColumnLayout {
                Layout.fillWidth: true
                spacing: 8
                visible: backend.document.format === "production_script"
                Label { text: qsTr("Roles"); font.pixelSize: 12; font.weight: Font.DemiBold }
                Label {
                    text: qsTr("Declare roles in your script, then choose a voice for each one.")
                    visible: !(backend.analysis.roles || []).length
                    wrapMode: Text.WordWrap; Layout.fillWidth: true; color: Theme.secondary; font.pixelSize: 12
                }
                Repeater {
                    model: backend.document.format === "production_script" ? (backend.analysis.roles || []) : []
                    RowLayout {
                        id: roleRow
                        required property string modelData
                        Layout.fillWidth: true
                        spacing: 6
                        Label {
                            text: roleRow.modelData; font.pixelSize: 11
                            Layout.preferredWidth: 48; elide: Text.ElideRight
                            HoverHandler { id: roleHover }
                            ToolTip.visible: roleHover.hovered && truncated
                            ToolTip.text: text
                        }
                        CompactComboBox {
                            id: roleChoice
                            Layout.fillWidth: true
                            model: { const localeDependency = preferences.language; return panel.voiceNames() }
                            selectionIndex: panel.voiceIndex(panel.selected(roleRow.modelData))
                            displayText: currentIndex < 0 ? qsTr("Choose a voice") : currentText
                            enabled: !backend.busy
                            onActivated: panel.choose(roleRow.modelData, currentIndex)
                        }
                        IconButton { symbol: "play"; hint: qsTr("Audition role voice"); enabled: !backend.busy && roleChoice.currentIndex >= 0; onClicked: panel.audition(roleChoice.currentIndex) }
                    }
                }
            }
            ColumnLayout {
                Layout.fillWidth: true
                spacing: 8
                visible: (backend.analysis.cues || []).length > 0 && backend.document.format === "production_script"
                Label { text: qsTr("Sound cues"); font.pixelSize: 12; font.weight: Font.DemiBold }
                Repeater {
                    model: backend.document.format === "production_script" ? (backend.analysis.cues || []) : []
                    RowLayout {
                        id: cueRow
                        required property string modelData
                        Layout.fillWidth: true
                        spacing: 6
                        Label {
                            text: cueRow.modelData; font.pixelSize: 11
                            Layout.preferredWidth: 48; elide: Text.ElideRight
                            HoverHandler { id: cueHover }
                            ToolTip.visible: cueHover.hovered && truncated
                            ToolTip.text: text
                        }
                        CompactComboBox {
                            Layout.fillWidth: true
                            model: {const locale = preferences.language; return panel.cueNames()}
                            selectionIndex: panel.cueIndex(cueRow.modelData)
                            enabled: !backend.busy
                            onActivated: { if (currentIndex === 3) panel.openLibrary(1); else if (currentIndex >= 4) backend.bindSound(cueRow.modelData, panel.sounds()[currentIndex - 4].id); else backend.bindCue(cueRow.modelData, ["unresolved", "chime", "skip"][currentIndex]) }
                        }
                        IconButton { symbol: "import"; hint: qsTr("Open sound library"); enabled: !backend.busy; onClicked: panel.openLibrary(1) }
                    }
                }
            }
            Label { Layout.fillWidth: true; text: qsTr("Generated speech is labeled as synthetic. Your accepted takes stay available when you revise the script."); wrapMode: Text.WordWrap; color: Theme.muted; font.pixelSize: 11; lineHeight: 1.35 }
        }
    }
    MaterialsDialog { id: materials; parent: Overlay.overlay }
    Native.FileDialog { id: cueFile; title: qsTr("Import sound cue"); nameFilters: [qsTr("WAV audio (*.wav)")]; onAccepted: backend.importCue(panel.cueLabel, selectedFile) }
}
