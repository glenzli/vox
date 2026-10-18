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
    property bool applyingText: false
    property bool exitWhenStopped: false
    onClosing: close => {
        if (backend.rendering) { close.accepted = false; stopDialog.open() }
        else if (backend.dirty) { close.accepted = false; backend.save(); exitWhenStopped = true }
    }
    Connections {
        target: backend
        function onDocumentChanged() {
            let next = backend.document.text || ""
            if (editor.text !== next) { window.applyingText = true; editor.loadText(next); window.applyingText = false }
        }
        function onRequestFinished(action, ok) {
            if (window.exitWhenStopped && ok && !backend.rendering && !backend.dirty) window.close()
        }
    }
    Shortcut { sequences: [StandardKey.New]; onActivated: if (!backend.rendering) newFile.open() }
    Shortcut { sequences: [StandardKey.Open]; onActivated: if (!backend.rendering) openFile.open() }
    Shortcut { sequences: [StandardKey.Save]; onActivated: backend.save() }
    Shortcut { sequence: "Ctrl+Return"; onActivated: backend.render() }
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
                IconButton { symbol: "add"; hint: qsTr("New project"); enabled: !backend.rendering; onClicked: newFile.open() }
                IconButton { symbol: "folder"; hint: qsTr("Open project"); enabled: !backend.rendering; onClicked: openFile.open() }
                IconButton { symbol: "save"; hint: qsTr("Save project"); enabled: backend.dirty && !backend.rendering; onClicked: backend.save() }
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
                ProjectSidebar { Layout.preferredWidth: 176; Layout.fillHeight: true }
                Rectangle { width: 1; Layout.fillHeight: true; color: Theme.border }
                ColumnLayout {
                    Layout.fillWidth: true; Layout.fillHeight: true; spacing: 0
                    RowLayout {
                        Layout.fillWidth: true; Layout.fillHeight: true; spacing: 0
                        Rectangle {
                            color: Theme.panel
                            Layout.fillHeight: true; Layout.fillWidth: true
                            ColumnLayout {
                                anchors.fill: parent; anchors.margins: 18; spacing: 12
                                RowLayout {
                                    Layout.fillWidth: true
                                    spacing: 6
                                    Label { text: qsTr("The script"); font.pixelSize: 17; font.weight: Font.DemiBold; Layout.rightMargin: 6 }
                                    CompactComboBox {
                                        id: formatChoice
                                        objectName: "formatChoice"
                                        model: [qsTr("Plain narration"), qsTr("Production script")]
                                        currentIndex: backend.document.format === "production_script" ? 1 : 0
                                        enabled: !backend.rendering
                                        onActivated: backend.setFormat(currentIndex === 0 ? "plain" : "production_script")
                                        implicitWidth: 154
                                    }
                                    Item { Layout.fillWidth: true }
                                    IconButton {
                                        objectName: "lineNumbersButton"
                                        symbol: "line-numbers"
                                        hint: qsTr("Line numbers")
                                        visible: !window.reading
                                        checkable: true
                                        checked: preferences.lineNumbers
                                        onToggled: preferences.lineNumbers = checked
                                    }
                                    RowLayout {
                                        spacing: 4
                                        ActionButton { objectName: "writeButton"; text: qsTr("Write"); primary: !window.reading; onClicked: window.reading = false }
                                        ActionButton { objectName: "readButton"; text: qsTr("Read"); primary: window.reading; onClicked: window.reading = true }
                                    }
                                    IconButton { objectName: "importScriptButton"; symbol: "import"; hint: qsTr("Import text or script"); enabled: !backend.rendering; onClicked: scriptFile.open() }
                                    IconButton { symbol: "help"; hint: qsTr("Script guide"); onClicked: guide.open() }
                                }
                                StackLayout {
                                    Layout.fillWidth: true; Layout.fillHeight: true
                                    currentIndex: window.reading ? 1 : 0
                                    ScriptEditor {
                                        id: editor
                                        objectName: "scriptEditorSurface"
                                        productionScript: backend.document.format === "production_script"
                                        readOnly: backend.rendering
                                        showLineNumbers: preferences.lineNumbers
                                        proseFontFamily: window.font.family
                                        onTextEdited: value => { if (!window.applyingText) backend.editText(value) }
                                    }
                                    ScrollView {
                                        clip: true
                                        contentWidth: availableWidth
                                        ColumnLayout {
                                            width: parent.width; spacing: 18
                                            Label { visible: backend.document.format === "plain"; text: backend.document.text || ""; Layout.fillWidth: true; wrapMode: Text.WordWrap; font.pixelSize: 19; lineHeight: 1.6 }
                                            Repeater {
                                                model: backend.document.format === "production_script" ? (backend.analysis.events || []) : []
                                                ColumnLayout {
                                                    id: eventRow
                                                    required property var modelData
                                                    Layout.fillWidth: true; spacing: 7
                                                    Label { visible: eventRow.modelData.kind === "speech" && (eventRow.modelData.role || "").length > 0; text: eventRow.modelData.role || ""; color: Theme.secondary; font.pixelSize: 11; font.weight: Font.DemiBold }
                                                    Label {
                                                        Layout.fillWidth: true; wrapMode: Text.WordWrap
                                                        font.pixelSize: eventRow.modelData.kind === "speech" ? 18 : 12
                                                        color: eventRow.modelData.kind === "speech" ? Theme.text : Theme.secondary
                                                        lineHeight: 1.5
                                                        text: {
                                                            let e=eventRow.modelData
                                                            if(e.kind === "pause")return qsTr("Pause · %1 s").arg(e.milliseconds / 1000)
                                                            if(e.kind === "cue")return qsTr("Sound · %1").arg(e.label)
                                                            if(e.kind === "scene")return qsTr("Scene · %1").arg(e.label)
                                                            if(e.kind === "repeat_start")return qsTr("Repeat %1 times · same recording").arg(e.count)
                                                            if(e.kind === "repeat_end")return qsTr("End repeat")
                                                            return e.text || ""
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                ColumnLayout {
                                    Layout.fillWidth: true; spacing: 5
                                    visible: (backend.analysis.issues || []).length > 0
                                    Repeater { model: (backend.analysis.issues || []).slice(0,3); Label { required property var modelData; text: qsTr("Line %1 · %2").arg(modelData.line).arg(backend.errorText(modelData.code)); Layout.fillWidth: true; wrapMode: Text.WordWrap; color: Theme.secondary; font.pixelSize: 11 } }
                                }
                                RowLayout {
                                    Layout.fillWidth: true
                                    Label { text: qsTr("%1 characters").arg((backend.document.text || "").length); color: Theme.muted; font.pixelSize: 11 }
                                    Item { Layout.fillWidth: true }
                                    Label { text: backend.rendering ? (backend.total > 0 ? qsTr("%1 / %2 segments").arg(backend.completed).arg(backend.total) : qsTr("Preparing…")) : (backend.analysis.ready ? qsTr("Ready to render") : qsTr("Review voices and script")); color: Theme.secondary; font.pixelSize: 11 }
                                    IconButton { visible: backend.rendering; symbol: "stop"; hint: qsTr("Stop after current segment"); onClicked: backend.cancel() }
                                    ActionButton { objectName: "renderButton"; text: backend.rendering ? qsTr("Rendering…") : qsTr("Render voice"); symbol: "waveform"; primary: true; enabled: !backend.rendering && (backend.document.text || "").trim().length > 0; onClicked: backend.render() }
                                }
                                ProgressBar { Layout.fillWidth: true; visible: backend.rendering; from: 0; to: Math.max(1,backend.total); value: backend.completed; indeterminate: backend.total <= 0 }
                            }
                        }
                        Rectangle { width: 1; Layout.fillHeight: true; color: Theme.border }
                        VoicePanel { Layout.preferredWidth: 276; Layout.fillHeight: true }
                    }
                    Rectangle { Layout.fillWidth: true; height: 1; color: Theme.border }
                    AudioPanel { Layout.fillWidth: true; Layout.preferredHeight: 172 }
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
                    ActionButton { text: qsTr("New project"); symbol: "add"; primary: true; onClicked: newFile.open() }
                    ActionButton { text: qsTr("Open project"); symbol: "folder"; onClicked: openFile.open() }
                }
                Item { Layout.preferredHeight: 14 }
                Repeater {
                    model: backend.recentProjects.slice(0,4)
                    ItemDelegate { required property var modelData; Layout.fillWidth: true; text: modelData.name; icon.source: "qrc:/vox/icons/file.svg"; icon.color: Theme.secondary; onClicked: backend.openProject("file://"+modelData.path) }
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
        title: qsTr("Audio is still being generated")
        anchors.centerIn: Overlay.overlay
        width: 440
        modal: true
        standardButtons: Dialog.Ok | Dialog.Cancel
        Label { text: qsTr("Stop after the current generation, then close Vox?"); color: Theme.text }
        onAccepted: { window.exitWhenStopped = true; backend.cancel() }
    }
    Native.FileDialog { id: newFile; title: qsTr("New Vox project"); fileMode: Native.FileDialog.SaveFile; defaultSuffix: "vox"; nameFilters: [qsTr("Vox project (*.vox)")]; onAccepted: backend.createProject(selectedFile) }
    Native.FileDialog { id: openFile; title: qsTr("Open Vox project"); nameFilters: [qsTr("Vox project (*.vox)")]; onAccepted: backend.openProject(selectedFile) }
    Native.FileDialog { id: scriptFile; title: qsTr("Import script"); nameFilters: [qsTr("Text files (*.txt *.md)"),qsTr("All files (*)")]; onAccepted: backend.importScript(selectedFile) }
}
