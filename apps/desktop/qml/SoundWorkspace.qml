import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Dialogs as Native

Item {
    id: workspace
    objectName: "soundWorkspace"
    property string category: "sfx"
    property bool loopAudition: false
    property bool showLoopControl: true
    property bool showAllSounds: false
    property string selectedId: ""
    property string projectIdentity: backend.path + ":" + (backend.project.id || "")
    property string fileProjectIdentity: ""
    property string exportId: ""
    signal categoryRequested(string category)
    readonly property var filteredSounds: (backend.project.sound_library || []).slice().reverse().filter(sound => {
        if (showAllSounds || !sound.generation) return true
        const kind = sound.metadata ? sound.metadata.category : sound.generation.spec.music ? "short_music" : "sfx"
        return kind === category
    })
    onCategoryChanged: { loopAudition = false; selectedId = "" }
    onProjectIdentityChanged: { loopAudition = false; selectedId = ""; if (soundName) soundName.clear(); if (soundPrompt) soundPrompt.clear() }
    function categoryLabel(value) {
        if (value === "ambience") return qsTr("Ambience")
        if (value === "short_music") return qsTr("Short music")
        return qsTr("Sound effects")
    }
    function displayName(sound) {
        const locale = preferences.language
        return sound.origin === "builtin" ? ((backend.builtinSounds.find(v => v.key === sound.name) || {}).label || sound.name) : sound.name
    }
    function sourceLabel(sound) {
        if (sound.origin === "builtin") return qsTr("Built-in")
        if (sound.origin === "imported") return qsTr("Imported")
        if (sound.origin === "shape_import") return qsTr("Shape import")
        return categoryLabel(sound.metadata ? sound.metadata.category : sound.generation && sound.generation.spec.music ? "short_music" : "sfx")
    }
    function openImport() { fileProjectIdentity = projectIdentity; soundFile.open() }
    function exportSound(id) { fileProjectIdentity = projectIdentity; exportId = id; exportFile.open() }
    function audition(id) { selectedId = id; backend.playMaterial("sound", id) }

    ColumnLayout {
        anchors.fill: parent
        spacing: 10
        RowLayout {
            Layout.fillWidth: true
            RowLayout {
                id: categories
                objectName: "soundCategoryTabs"
                Layout.fillWidth: true
                spacing: 4
                Repeater {
                    model: ["sfx", "ambience", "short_music"]
                    TabButton {
                        id: categoryTab
                        required property string modelData
                        readonly property bool categorySelected: workspace.category === modelData
                        objectName: "soundCategory_" + modelData
                        Layout.fillWidth: true
                        checkable: false
                        Accessible.selected: categorySelected
                        text: { const language = preferences.language; return workspace.categoryLabel(modelData) }
                        implicitHeight: 30
                        onClicked: workspace.categoryRequested(modelData)
                        contentItem: Label { text: categoryTab.text; color: Theme.text; horizontalAlignment: Text.AlignHCenter; verticalAlignment: Text.AlignVCenter }
                        background: Rectangle { color: categoryTab.categorySelected ? Theme.selected : Theme.panel; radius: 5; border.color: Theme.border }
                    }
                }
            }
            ActionButton { symbol: "import"; text: qsTr("Import WAV"); enabled: !backend.busy; onClicked: workspace.openImport() }
        }
        Rectangle {
            Layout.fillWidth: true
            implicitHeight: generationForm.implicitHeight + 24
            color: Theme.inset; radius: 7; border.color: Theme.border
            ColumnLayout {
                id: generationForm
                anchors.fill: parent; anchors.margins: 12
                spacing: 8
                RowLayout {
                    Label { text: workspace.categoryLabel(workspace.category); font.weight: Font.DemiBold; font.pixelSize: 15 }
                    Label { text: qsTr("1–30 seconds"); color: Theme.muted; font.pixelSize: 11 }
                    Item { Layout.fillWidth: true }
                    Label { text: qsTr("Local generation"); color: Theme.secondary; font.pixelSize: 11 }
                }
                Label {
                    Layout.fillWidth: true
                    wrapMode: Text.WordWrap; color: Theme.secondary; font.pixelSize: 12
                    text: workspace.category === "short_music" ? qsTr("Short instrumental music. No vocals or singing are requested; audition each result.")
                        : workspace.category === "ambience" ? qsTr("Describe an environmental bed. Loop audition helps you hear how it repeats.")
                        : qsTr("Describe a single effect or a short sequence in English or Chinese.")
                }
                CompactTextField { id: soundName; objectName: "soundName"; Layout.fillWidth: true; placeholderText: qsTr("Sound name"); maximumLength: 100; enabled: !backend.busy }
                ScrollView {
                    Layout.fillWidth: true; Layout.preferredHeight: 76
                    TextArea {
                        id: soundPrompt; objectName: "soundPrompt"
                        wrapMode: TextEdit.Wrap; selectByMouse: true; enabled: !backend.busy
                        color: Theme.text; selectionColor: Theme.selected; selectedTextColor: Theme.text
                        placeholderText: workspace.category === "short_music" ? qsTr("Sparse piano and soft strings, a calm instrumental ending.")
                            : workspace.category === "ambience" ? qsTr("Steady rain in a quiet forest, distant leaves moving, no speech or music.")
                            : qsTr("A soft bell, then two quiet wooden knocks. No speech or music.")
                        onTextChanged: { if (length > 1800) remove(1800, length) }
                        background: Rectangle { color: Theme.panel; radius: 5; border.color: soundPrompt.activeFocus ? Theme.secondary : Theme.border }
                    }
                }
                RowLayout {
                    Label { text: qsTr("Seconds"); font.pixelSize: 12 }
                    CompactSpinBox { id: seconds; objectName: "soundSeconds"; from: 1; to: 30; value: 5; editable: true; enabled: !backend.busy; Layout.preferredWidth: 90 }
                    Label { text: qsTr("Seed"); font.pixelSize: 12 }
                    CompactTextField { id: seed; objectName: "soundSeed"; text: "42"; Layout.preferredWidth: 90; validator: IntValidator { bottom: 0; top: 2147483647 }
                        enabled: !backend.busy; Accessible.name: qsTr("Generation seed") }
                    Label { text: qsTr("Variations"); font.pixelSize: 12 }
                    CompactSpinBox { id: variations; objectName: "soundVariations"; from: 1; to: 4; value: 1; enabled: !backend.busy; Layout.preferredWidth: 82 }
                    Item { Layout.fillWidth: true }
                    ActionButton {
                        objectName: "generateSoundButton"
                        text: variations.value > 1 ? qsTr("Generate %1 candidates").arg(variations.value) : qsTr("Generate candidate")
                        symbol: "waveform"; primary: true
                        enabled: !backend.busy && backend.runtimeReady && soundName.text.trim().length > 0 && soundPrompt.text.trim().length > 0 && seed.acceptableInput
                        onClicked: backend.generateSoundBatch(soundName.text, soundPrompt.text, seconds.value, Number(seed.text), workspace.category, variations.value)
                    }
                }
                RowLayout {
                    visible: backend.rendering
                    BusyIndicator { running: visible; Layout.preferredWidth: 20; Layout.preferredHeight: 20 }
                    Label { text: qsTr("Completed variations are kept as candidates when you stop."); Layout.fillWidth: true; wrapMode: Text.WordWrap; font.pixelSize: 11; color: Theme.secondary }
                    ActionButton { text: qsTr("Stop after current generation"); symbol: "stop"; onClicked: backend.cancel() }
                }
            }
        }
        RowLayout {
            Layout.fillWidth: true
            Label { text: qsTr("Project sounds"); font.weight: Font.DemiBold; font.pixelSize: 13 }
            Label { text: workspace.filteredSounds.length; color: Theme.muted; font.pixelSize: 11 }
            Item { Layout.fillWidth: true }
            ActionButton { text: qsTr("All project sounds"); checkable: true; checked: workspace.showAllSounds; primary: checked; onClicked: workspace.showAllSounds = checked }
            ActionButton { objectName: "soundLoopAudition"; text: qsTr("Loop audition"); symbol: "refresh"; visible: workspace.showLoopControl; checkable: true; checked: workspace.loopAudition; primary: checked; onClicked: workspace.loopAudition = checked; ToolTip.visible: hovered; ToolTip.text: qsTr("Repeats playback; it does not create a seamless loop.") }
        }
        ListView {
            id: soundList; objectName: "soundCandidateList"
            Layout.fillWidth: true; Layout.fillHeight: true
            Layout.minimumHeight: 80
            clip: true; spacing: 6
            model: workspace.filteredSounds
            ScrollBar.vertical: ScrollBar { }
            delegate: Rectangle {
                id: card
                required property var modelData
                property bool detailsOpen: false
                width: soundList.width; height: soundContent.implicitHeight + 20
                radius: 6; color: workspace.selectedId === modelData.id ? Theme.selected : Theme.inset; border.color: Theme.border
                ColumnLayout {
                    id: soundContent
                    anchors.fill: parent; anchors.margins: 10
                    spacing: 5
                    RowLayout {
                        Layout.fillWidth: true
                        IconButton { objectName: "soundCandidatePreviewButton"; symbol: "play"; hint: qsTr("Preview sound"); enabled: !backend.busy; onClicked: workspace.audition(card.modelData.id) }
                        Label { text: workspace.displayName(card.modelData); Layout.fillWidth: true; elide: Text.ElideRight; font.weight: Font.Medium }
                        Label { text: card.modelData.content ? qsTr("Accepted") : qsTr("Candidate"); color: Theme.secondary; font.pixelSize: 11 }
                        IconButton { symbol: "help"; hint: qsTr("Sound details"); checkable: true; checked: card.detailsOpen; onClicked: card.detailsOpen = checked }
                        IconButton { symbol: "echo"; hint: qsTr("Edit a copy in Echo"); enabled: !backend.busy; onClicked: backend.editMaterialInEcho("sound", card.modelData.id) }
                        IconButton { symbol: "export"; hint: qsTr("Export original WAV"); enabled: !backend.busy; onClicked: workspace.exportSound(card.modelData.id) }
                        IconButton { symbol: "x"; hint: qsTr("Discard candidate"); visible: !card.modelData.content; enabled: !backend.busy; onClicked: backend.discardSound(card.modelData.id) }
                        ActionButton { objectName: "soundCandidateAcceptButton"; text: qsTr("Accept"); symbol: "check"; visible: !card.modelData.content; enabled: !backend.busy; onClicked: backend.acceptSound(card.modelData.id) }
                    }
                    RowLayout {
                        Layout.fillWidth: true
                        Label { text: workspace.sourceLabel(card.modelData) + " · " + qsTr("%1 s").arg((card.modelData.contract.frame_count / card.modelData.contract.sample_rate_hz).toFixed(1)); font.pixelSize: 11; color: Theme.muted }
                        Label { visible: !!card.modelData.generation; text: card.modelData.generation ? (card.modelData.metadata ? qsTr("Variation %1 of %2 · seed %3").arg(card.modelData.metadata.variation).arg(card.modelData.metadata.variations).arg(card.modelData.generation.spec.seed) : qsTr("Seed %1").arg(card.modelData.generation.spec.seed)) : ""; font.pixelSize: 11; color: Theme.muted; Layout.fillWidth: true; elide: Text.ElideRight }
                    }
                    Label { visible: !!card.modelData.generation; text: card.modelData.metadata ? card.modelData.metadata.authored_prompt : card.modelData.generation ? card.modelData.generation.spec.prompt : ""; Layout.fillWidth: true; wrapMode: Text.WordWrap; maximumLineCount: card.detailsOpen ? 100 : 2; elide: Text.ElideRight; color: Theme.secondary; font.pixelSize: 12 }
                    Label { visible: card.detailsOpen && !!card.modelData.generation; text: card.modelData.generation ? qsTr("Generation prompt: %1").arg(card.modelData.generation.prepared.effective_prompt) : ""; Layout.fillWidth: true; wrapMode: Text.WordWrap; color: Theme.muted; font.pixelSize: 11 }
                    Label { visible: card.detailsOpen; text: qsTr("Original WAV is retained. Accepting also prepares a copy for script cues."); Layout.fillWidth: true; wrapMode: Text.WordWrap; color: Theme.muted; font.pixelSize: 11 }
                }
            }
            Label { anchors.centerIn: parent; width: Math.min(parent.width - 20, 420); horizontalAlignment: Text.AlignHCenter; wrapMode: Text.WordWrap; visible: workspace.filteredSounds.length === 0; text: qsTr("Generate or import sounds, then listen and accept the candidates you want to keep."); color: Theme.muted; font.pixelSize: 12 }
        }
        RowLayout {
            Layout.fillWidth: true
            Label { text: qsTr("Built-in sounds"); font.pixelSize: 11; color: Theme.secondary }
            CompactComboBox { id: builtinChoice; objectName: "builtinSoundChoice"; Layout.fillWidth: true; model: backend.builtinSounds.map(v => v.label); enabled: !backend.busy }
            IconButton { symbol: "play"; hint: qsTr("Preview built-in sound"); enabled: !backend.busy; onClicked: backend.previewBuiltin(backend.builtinSounds[builtinChoice.currentIndex].key) }
            IconButton { objectName: "soundBuiltinAddButton"; symbol: "add"; hint: qsTr("Add to sound library"); enabled: !backend.busy; onClicked: backend.addBuiltin(backend.builtinSounds[builtinChoice.currentIndex].key) }
        }
        Label { text: qsTr("Export keeps the original quality. Use Echo for trimming, fades and mixing; import its WAV as a new candidate."); Layout.fillWidth: true; wrapMode: Text.WordWrap; color: Theme.muted; font.pixelSize: 11 }
    }
    Native.FileDialog {
        id: soundFile
        title: qsTr("Import sound WAV"); nameFilters: [qsTr("PCM16 WAV audio (*.wav)")]
        onAccepted: { if (workspace.fileProjectIdentity === workspace.projectIdentity) backend.importSound(selectedFile) }
    }
    Native.FileDialog {
        id: exportFile
        title: qsTr("Export original WAV"); fileMode: Native.FileDialog.SaveFile; defaultSuffix: "wav"; nameFilters: [qsTr("WAV audio (*.wav)")]
        onAccepted: { if (workspace.fileProjectIdentity === workspace.projectIdentity) backend.exportMaterial("sound", workspace.exportId, selectedFile) }
    }
}
