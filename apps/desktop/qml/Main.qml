import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Dialogs as Native
import QtQuick.Window
ApplicationWindow {
    id: window
    objectName: "voxWindow"
    width: 1320
    height: 900
    minimumWidth: 1080
    minimumHeight: 740
    visible: true
    title: backend.path.length ? (backend.project.title || "Vox") + " — Vox" : "Vox"
    flags: Qt.platform.os === "osx" ? Qt.Window | Qt.ExpandedClientAreaHint | Qt.NoTitleBarBackgroundHint : Qt.Window
    topPadding: 0
    color: Theme.window
    palette.window: Theme.window
    palette.base: Theme.panel
    palette.alternateBase: Theme.inset
    palette.text: Theme.text
    palette.windowText: Theme.text
    palette.button: Theme.panel
    palette.buttonText: Theme.text
    palette.highlight: Theme.selected
    palette.highlightedText: Theme.text
    palette.mid: Theme.border
    palette.placeholderText: Theme.muted
    font.pixelSize: 13
    property bool reading: false
    property bool segments: false
    property string productionMode: "speech"
    property bool applyingText: false
    property bool exitWhenStopped: false
    readonly property bool renderReady: !backend.busy && !backend.dirty && !!backend.analysis.ready && (backend.document.text || "").trim().length > 0
    readonly property string readinessHint: {
        if (!(backend.document.text || "").trim().length) return qsTr("Write or import a script to begin.")
        if (backend.dirty) return qsTr("Checking script…")
        let readiness = backend.analysis.readiness || {}
        let missing = (readiness.missing_roles || []).map(role => role.name)
        let cues = (readiness.unresolved_cues || []).map(cue => cue.name)
        let hints = []
        if (missing.length) hints.push(qsTr("Choose voices: %1").arg(missing.join(", ")))
        if (cues.length) hints.push(qsTr("Resolve sound cues: %1").arg(cues.join(", ")))
        if (hints.length) return hints.join(" · ")
        if ((backend.analysis.issues || []).length) return qsTr("Review the script issues above.")
        return backend.analysis.ready ? qsTr("Ready to render") : qsTr("Review voices and script")
    }
    function finishClosing(ok) {
        if (!exitWhenStopped) return
        if (!ok && backend.dirty) { exitWhenStopped = false; return }
        if (backend.busy || backend.pending) return
        if (backend.dirty) { backend.save(); return }
        exitWhenStopped = false
        window.close()
    }
    onClosing: close => {
        if (backend.rendering) { close.accepted = false; stopDialog.open() }
        else if (backend.busy || backend.pending || backend.dirty) {
            close.accepted = false
            exitWhenStopped = true
            if (!backend.busy && !backend.pending) backend.save()
        }
    }
    Connections {
        target: backend
        function onDocumentChanged() {
            let next = backend.document.text || ""
            if (editor.text !== next) { window.applyingText = true; editor.loadText(next); window.applyingText = false }
        }
        function onRequestFinished(action, ok) {
            window.finishClosing(ok)
        }
    }
    Shortcut { sequences: [StandardKey.New]; onActivated: if (!backend.busy) newFile.open() }
    Shortcut { sequences: [StandardKey.Open]; onActivated: if (!backend.busy) openFile.open() }
    Shortcut { sequences: [StandardKey.Save]; onActivated: backend.save() }
    Shortcut { sequence: "Ctrl+Return"; onActivated: if (window.productionMode === "speech" && window.renderReady && !window.segments) backend.render() }
    ColumnLayout {
        anchors.fill: parent
        spacing: 0
        Rectangle {
            id: titleBar
            objectName: "titleToolBar"
            Layout.fillWidth: true
            Layout.preferredHeight: 44
            color: Theme.window
            Item {
                anchors.fill: parent
                DragHandler {
                    target: null
                    acceptedButtons: Qt.LeftButton
                    onActiveChanged: if (active) window.startSystemMove()
                }
                TapHandler {
                    acceptedButtons: Qt.LeftButton
                    onDoubleTapped: {
                        if (window.visibility === Window.Maximized) window.showNormal()
                        else if (window.visibility !== Window.FullScreen) window.showMaximized()
                    }
                }
            }
            RowLayout {
                objectName: "titleBarContent"
                anchors.fill: parent
                anchors.leftMargin: Math.max(titleBar.SafeArea.margins.left, Qt.platform.os === "osx" && window.visibility !== Window.FullScreen ? 96 : 18)
                anchors.rightMargin: Math.max(titleBar.SafeArea.margins.right, 18)
                spacing: 12
                Image { source: "qrc:/vox/branding/app-icon.png"; sourceSize.width: 48; sourceSize.height: 48; Layout.preferredWidth: 24; Layout.preferredHeight: 24 }
                Label { text: "Vox"; font.pixelSize: 16; font.weight: Font.DemiBold; font.letterSpacing: -0.6 }
                Rectangle { Layout.leftMargin: 4; Layout.rightMargin: 4; width: 1; height: 18; color: Theme.border }
                Label { text: backend.project.title || qsTr("Script-led voice production"); color: Theme.secondary; elide: Text.ElideRight; Layout.fillWidth: true }
                Label { text: backend.dirty ? qsTr("Saving…") : backend.path.length ? qsTr("Saved") : ""; font.pixelSize: 10; color: Theme.muted }
                IconButton { symbol: "add"; hint: qsTr("New project"); enabled: !backend.busy; onClicked: newFile.open() }
                IconButton { symbol: "folder"; hint: qsTr("Open project"); enabled: !backend.busy; onClicked: openFile.open() }
                IconButton { symbol: "save"; hint: qsTr("Save project"); enabled: backend.dirty && !backend.busy; onClicked: backend.save() }
                IconButton { objectName: "settingsButton"; symbol: "settings"; hint: qsTr("Settings"); onClicked: settings.open() }
            }
        }
        Rectangle { Layout.fillWidth: true; height: 1; color: Theme.border }
        Item {
            Layout.fillWidth: true
            Layout.fillHeight: true
            RowLayout {
                anchors.fill: parent
                visible: backend.path.length > 0
                spacing: 0
                ProjectSidebar {
                    Layout.preferredWidth: 176; Layout.fillHeight: true
                    productionMode: window.productionMode
                    onModeRequested: mode => window.productionMode = mode
                }
                Rectangle { width: 1; Layout.fillHeight: true; color: Theme.border }
                ColumnLayout {
                    Layout.fillWidth: true; Layout.fillHeight: true; spacing: 0
                    RowLayout {
                        Layout.fillWidth: true; Layout.fillHeight: true; spacing: 0
                        visible: window.productionMode === "speech"
                        Rectangle {
                            color: Theme.panel
                            Layout.fillHeight: true; Layout.fillWidth: true
                            ColumnLayout {
                                anchors.fill: parent; anchors.margins: 18; spacing: 12
                                RowLayout {
                                    Layout.fillWidth: true
                                    spacing: 6
                                    Label { text: window.segments ? qsTr("Segments") : qsTr("The script"); font.pixelSize: 17; font.weight: Font.DemiBold; Layout.rightMargin: 6 }
                                    CompactComboBox {
                                        id: formatChoice
                                        objectName: "formatChoice"
                                        visible: !window.segments
                                        model: [qsTr("Plain narration"), qsTr("Production script")]
                                        selectionIndex: backend.document.format === "production_script" ? 1 : 0
                                        enabled: !backend.busy
                                        onActivated: backend.setFormat(currentIndex === 0 ? "plain" : "production_script")
                                        implicitWidth: 154
                                    }
                                    Item { Layout.fillWidth: true }
                                    IconButton {
                                        objectName: "lineNumbersButton"
                                        symbol: "line-numbers"
                                        hint: qsTr("Line numbers")
                                        visible: !window.reading && !window.segments
                                        checkable: true
                                        checked: preferences.lineNumbers
                                        onToggled: preferences.lineNumbers = checked
                                    }
                                    RowLayout {
                                        spacing: 4
                                        ActionButton { objectName: "writeButton"; text: qsTr("Write"); primary: !window.reading && !window.segments; onClicked: { window.segments = false; window.reading = false } }
                                        ActionButton { objectName: "readButton"; text: qsTr("Read"); primary: window.reading && !window.segments; onClicked: { window.segments = false; window.reading = true } }
                                        ActionButton { objectName: "segmentsButton"; text: qsTr("Segments"); primary: window.segments; onClicked: window.segments = true }
                                    }
                                    IconButton { objectName: "pronunciationButton"; symbol: "language"; hint: qsTr("Pronunciation dictionary"); enabled: !backend.busy; onClicked: pronunciationDialog.open() }
                                    IconButton { objectName: "importScriptButton"; symbol: "import"; hint: qsTr("Import text or script"); enabled: !backend.busy; onClicked: scriptFile.open() }
                                    IconButton { symbol: "help"; hint: qsTr("Script guide"); onClicked: guide.open() }
                                }
                                StackLayout {
                                    Layout.fillWidth: true; Layout.fillHeight: true
                                    currentIndex: window.segments ? 2 : window.reading ? 1 : 0
                                    ScriptEditor {
                                        id: editor
                                        objectName: "scriptEditorSurface"
                                        sourceIdentity: backend.project.id || ""
                                        productionScript: backend.document.format === "production_script"
                                        readOnly: backend.busy
                                        showLineNumbers: preferences.lineNumbers
                                        proseFontFamily: window.font.family
                                        onTextEdited: value => { if (!window.applyingText) backend.editText(value) }
                                    }
                                    ScriptReader {
                                        onEditLineRequested: line => {
                                            window.reading = false
                                            window.segments = false
                                            Qt.callLater(() => editor.focusLine(line))
                                        }
                                    }
                                    SegmentsPanel {
                                        objectName: "planPanel"
                                        readinessHint: window.readinessHint
                                        onEditLineRequested: line => {
                                            window.segments = false; window.reading = false
                                            Qt.callLater(() => editor.focusLine(line))
                                        }
                                        onPronunciationRequested: pronunciationDialog.open()
                                        onSubtitleRequested: subtitleDialog.openForAudio()
                                    }
                                }
                                ColumnLayout {
                                    Layout.fillWidth: true; spacing: 5
                                    visible: !window.segments && (backend.analysis.issues || []).length > 0
                                    Repeater { model: (backend.analysis.issues || []).slice(0,3); Label { required property var modelData; text: qsTr("Line %1 · %2").arg(modelData.line).arg(backend.errorText(modelData.code)); Layout.fillWidth: true; wrapMode: Text.WordWrap; color: Theme.secondary; font.pixelSize: 11 } }
                                }
                                RowLayout {
                                    Layout.fillWidth: true
                                    visible: !window.segments || backend.rendering
                                    Label { text: qsTr("%1 characters").arg((backend.document.text || "").length); color: Theme.muted; font.pixelSize: 11 }
                                    Label {
                                        objectName: "renderReadiness"
                                        Layout.fillWidth: true
                                        Layout.minimumWidth: 0
                                        horizontalAlignment: Text.AlignRight
                                        text: backend.rendering ? (backend.total > 0 ? qsTr("%1 / %2 segments").arg(backend.completed).arg(backend.total) : qsTr("Preparing…")) : window.readinessHint
                                        color: Theme.secondary
                                        font.pixelSize: 11
                                        elide: Text.ElideRight
                                        HoverHandler { id: readinessHover }
                                        ToolTip.visible: readinessHover.hovered && truncated
                                        ToolTip.text: text
                                    }
                                    IconButton { visible: backend.rendering; symbol: "stop"; hint: qsTr("Stop after current segment"); onClicked: backend.cancel() }
                                    ActionButton { objectName: "renderButton"; text: backend.rendering ? qsTr("Rendering…") : qsTr("Render voice"); symbol: "waveform"; primary: true; enabled: window.renderReady; onClicked: backend.render() }
                                }
                                ProgressBar { Layout.fillWidth: true; visible: backend.rendering; from: 0; to: Math.max(1,backend.total); value: backend.completed; indeterminate: backend.total <= 0 }
                            }
                        }
                        Rectangle { width: 1; Layout.fillHeight: true; color: Theme.border }
                        VoicePanel { Layout.preferredWidth: 276; Layout.fillHeight: true }
                    }
                    SoundWorkspace {
                        id: soundWorkspace
                        objectName: "soundWorkspace"
                        Layout.fillWidth: true; Layout.fillHeight: true
                        Layout.margins: 18
                        visible: window.productionMode !== "speech"
                        category: window.productionMode === "speech" ? "sfx" : window.productionMode
                        onCategoryRequested: category => window.productionMode = category
                    }
                    Rectangle { Layout.fillWidth: true; height: 1; color: Theme.border }
                    AudioPanel {
                        Layout.fillWidth: true
                        Layout.preferredHeight: window.productionMode === "speech" ? 172 : 64
                        showTakes: window.productionMode === "speech"
                        playbackContext: window.productionMode
                        loopPreview: window.productionMode !== "speech" && soundWorkspace.loopAudition
                        onExportSubtitlesRequested: audioId => subtitleDialog.openForAudio(audioId)
                    }
                }
            }
            ColumnLayout {
                anchors.centerIn: parent
                width: 560
                spacing: 20
                visible: backend.path.length === 0
                Image { source: "qrc:/vox/branding/app-icon.png"; Layout.alignment: Qt.AlignHCenter; Layout.preferredWidth: 96; Layout.preferredHeight: 96; sourceSize.width: 192; sourceSize.height: 192 }
                Label { text: qsTr("Give your words a voice."); Layout.alignment: Qt.AlignHCenter; font.pixelSize: 32; font.weight: Font.Medium; font.letterSpacing: -1 }
                Label { text: qsTr("A quiet space for narration, dialogue and listening materials.\nWrite the script. Direct the voices. Keep the take."); horizontalAlignment: Text.AlignHCenter; Layout.fillWidth: true; color: Theme.secondary; font.pixelSize: 14; lineHeight: 1.5 }
                RowLayout {
                    Layout.alignment: Qt.AlignHCenter; spacing: 12
                    ActionButton { text: qsTr("New project"); symbol: "add"; primary: true; enabled: !backend.busy; onClicked: newFile.open() }
                    ActionButton { text: qsTr("Open project"); symbol: "folder"; enabled: !backend.busy; onClicked: openFile.open() }
                }
                Item { Layout.preferredHeight: 14 }
                Repeater {
                    model: backend.recentProjects.slice(0,4)
                    ItemDelegate { required property var modelData; Layout.fillWidth: true; text: modelData.name; icon.source: "qrc:/vox/icons/file.svg"; icon.color: Theme.secondary; enabled: !backend.busy; onClicked: backend.openProject("file://"+modelData.path) }
                }
            }
        }
        Rectangle { Layout.fillWidth: true; height: 1; color: Theme.border }
        Rectangle {
            Layout.fillWidth: true; Layout.preferredHeight: 26; color: Theme.window
            Label { anchors.fill: parent; anchors.leftMargin: 20; anchors.rightMargin: 20; verticalAlignment: Text.AlignVCenter; text: backend.message || qsTr("Your project stays on this device."); color: Theme.secondary; font.pixelSize: 11; elide: Text.ElideRight }
        }
    }
    SettingsDialog { id: settings; anchors.centerIn: Overlay.overlay }
    PronunciationDialog { id: pronunciationDialog; objectName: "pronunciationDialog"; anchors.centerIn: Overlay.overlay }
    SubtitleDialog { id: subtitleDialog; objectName: "subtitleDialog"; anchors.centerIn: Overlay.overlay }
    Dialog {
        id: guide
        title: qsTr("Script guide")
        anchors.centerIn: Overlay.overlay
        width: Math.min(window.width-100,820); height: Math.min(window.height-100,700)
        modal: true; standardButtons: Dialog.Close
        background: Rectangle { color: Theme.panel; radius: 10; border.color: Theme.border }
        ScrollView { anchors.fill: parent; TextArea { readOnly: true; selectByMouse: true; text: backend.scriptRules(); textFormat: TextEdit.MarkdownText; wrapMode: TextEdit.Wrap; color: Theme.text; background: Item {} } }
    }
    Dialog {
        id: stopDialog
        objectName: "stopDialog"
        title: qsTr("Audio is still being generated")
        anchors.centerIn: Overlay.overlay
        width: 440
        modal: true
        standardButtons: Dialog.Ok | Dialog.Cancel
        onOpened: {
            standardButton(Dialog.Ok).text = qsTr("Stop and close")
            standardButton(Dialog.Cancel).text = qsTr("Keep open")
        }
        Label { text: qsTr("Stop after the current generation, then close Vox?"); color: Theme.text }
        onAccepted: {
            window.exitWhenStopped = true
            if (backend.rendering) backend.cancel()
            else window.finishClosing(true)
        }
    }
    Native.FileDialog { id: newFile; title: qsTr("New Vox project"); fileMode: Native.FileDialog.SaveFile; defaultSuffix: "vox"; nameFilters: [qsTr("Vox project (*.vox)")]; onAccepted: backend.createProject(selectedFile) }
    Native.FileDialog { id: openFile; title: qsTr("Open Vox project"); nameFilters: [qsTr("Vox project (*.vox)")]; onAccepted: backend.openProject(selectedFile) }
    Native.FileDialog { id: scriptFile; title: qsTr("Import script"); nameFilters: [qsTr("Text files (*.txt *.md)"),qsTr("All files (*)")]; onAccepted: backend.importScript(selectedFile) }
}
