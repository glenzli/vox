pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Dialogs as Native

Dialog {
    id: dialog
    objectName: "compilationDialog"
    title: qsTr("Compile chapters")
    parent: Overlay.overlay
    width: Math.min(710, parent.width - 48)
    height: Math.min(690, parent.height - 56)
    anchors.centerIn: parent
    modal: true
    padding: 16
    closePolicy: delivery.compiling ? Popup.NoAutoClose : Popup.CloseOnEscape
    readonly property var delivery: backend.delivery
    property var chapterRows: []
    property int gapMs: 0
    property int audioMode: 0
    property int captionsMode: 0
    property url parentFolder: ""
    readonly property var selectedIDs: chapterRows.filter(item => item.chosen).map(item => item.id)
    readonly property var output: delivery.compilationPreflight.output || ({})
    readonly property var issueLines: {
        const language = preferences.language
        const rows = delivery.compilationPreflight.chapters || []
        let issues = [], codes = []
        for (const chapter of rows) {
            for (const code of (chapter.issues || [])) {
                issues.push(chapter.title + ": " + delivery.issueText(code))
                codes.push(code)
            }
        }
        for (const code of (delivery.compilationPreflight.issues || [])) {
            if (codes.indexOf(code) < 0) issues.push(delivery.issueText(code))
        }
        return issues
    }
    function configure() {
        delivery.configureCompilation(selectedIDs, gapMs, audioMode === 1 ? "normalize_mono24000" : "preserve", captionsMode === 1 ? "audio_only" : "required")
    }
    function openForCollection() {
        delivery.openCompilation()
        if (!delivery.compilationActive) return
        chapterRows = backend.chapters.filter(item => !item.archived).map(item => ({ id: item.id, title: item.title, status: item.status, chosen: true }))
        gapMs = 0; audioMode = 0; captionsMode = 0; parentFolder = ""
        folderName.text = qsTr("Compilation")
        configure()
        open()
    }
    function choose(index, chosen) {
        let rows = chapterRows.slice()
        rows[index] = Object.assign({}, rows[index], {chosen: chosen})
        chapterRows = rows
        configure()
    }
    function move(index, offset) {
        let rows = chapterRows.slice()
        const target = index + offset
        if (target < 0 || target >= rows.length) return
        const item = rows[index]; rows[index] = rows[target]; rows[target] = item
        chapterRows = rows
        configure()
    }
    function duration(ms) {
        const seconds = Math.round((ms || 0) / 1000)
        return Math.floor(seconds / 60) + ":" + (seconds % 60).toString().padStart(2, "0")
    }
    onClosed: delivery.closeCompilation()
    background: Rectangle { color: Theme.panel; radius: 10; border.color: Theme.border }
    contentItem: ColumnLayout {
        spacing: 9
        Label { text: qsTr("Select chapters and arrange their export order. Only current accepted audio is included."); Layout.fillWidth: true; wrapMode: Text.WordWrap; color: Theme.secondary; font.pixelSize: 11 }
        RowLayout {
            Layout.fillWidth: true
            Label { objectName: "compilationSelectionCount"; text: qsTr("%1 of %2 chapters selected").arg(dialog.selectedIDs.length).arg(dialog.chapterRows.length); font.pixelSize: 11; color: Theme.secondary; Layout.fillWidth: true }
            ActionButton { text: qsTr("Select all"); enabled: !backend.busy; onClicked: { dialog.chapterRows = dialog.chapterRows.map(item => Object.assign({}, item, {chosen: true})); dialog.configure() } }
            ActionButton { text: qsTr("Clear"); enabled: !backend.busy; onClicked: { dialog.chapterRows = dialog.chapterRows.map(item => Object.assign({}, item, {chosen: false})); dialog.configure() } }
        }
        Rectangle {
            Layout.fillWidth: true; Layout.fillHeight: true; Layout.minimumHeight: 130
            color: Theme.inset; radius: 5; border.color: Theme.border
            ListView {
                id: chapterList
                objectName: "compilationChapterList"
                anchors.fill: parent; anchors.margins: 5
                model: dialog.chapterRows; clip: true; spacing: 2
                ScrollBar.vertical: ScrollBar { }
                delegate: RowLayout {
                    id: row
                    required property var modelData
                    required property int index
                    width: chapterList.width; spacing: 5
                    CheckBox { objectName: "compileChapterCheck-" + row.index; checked: row.modelData.chosen; enabled: !backend.busy; onToggled: dialog.choose(row.index, checked); Accessible.name: row.modelData.title }
                    Label { text: (row.index + 1) + ". " + row.modelData.title; Layout.fillWidth: true; elide: Text.ElideRight; font.pixelSize: 12; color: row.modelData.chosen ? Theme.text : Theme.secondary }
                    Label { objectName: "compileChapterStatus-" + row.index; text: dialog.delivery.chapterStatusLabels[row.modelData.id] || ""; font.pixelSize: 10; color: Theme.secondary }
                    IconButton { objectName: "compileChapterUp-" + row.index; symbol: "arrow-up"; hint: qsTr("Move earlier in this export"); enabled: !backend.busy && row.index > 0; onClicked: dialog.move(row.index, -1) }
                    IconButton { objectName: "compileChapterDown-" + row.index; symbol: "arrow-down"; hint: qsTr("Move later in this export"); enabled: !backend.busy && row.index + 1 < dialog.chapterRows.length; onClicked: dialog.move(row.index, 1) }
                }
            }
        }
        GridLayout {
            columns: 2; columnSpacing: 12; rowSpacing: 7; Layout.fillWidth: true
            Label { text: qsTr("Between chapters"); font.pixelSize: 11; color: Theme.secondary }
            RowLayout {
                CompactSpinBox { objectName: "compilationGap"; from: 0; to: 600000; stepSize: 100; editable: true; value: dialog.gapMs; enabled: !backend.busy; onValueModified: { dialog.gapMs = value; dialog.configure() } }
                Label { text: qsTr("ms silence"); color: Theme.secondary; font.pixelSize: 11 }
            }
            Label { text: qsTr("Audio format"); font.pixelSize: 11; color: Theme.secondary }
            CompactComboBox { objectName: "compilationAudioMode"; Layout.fillWidth: true; model: [qsTr("Keep matching formats"), qsTr("Convert all to mono · 24 kHz · PCM16")]; selectionIndex: dialog.audioMode; enabled: !backend.busy; onActivated: index => { dialog.audioMode = index; dialog.configure() } }
            Label { text: qsTr("Subtitles"); font.pixelSize: 11; color: Theme.secondary }
            CompactComboBox { objectName: "compilationCaptionsMode"; Layout.fillWidth: true; model: [qsTr("Require current subtitles"), qsTr("Audio only")]; selectionIndex: dialog.captionsMode; enabled: !backend.busy; onActivated: index => { dialog.captionsMode = index; dialog.configure() } }
        }
        Label { visible: dialog.audioMode === 1; text: qsTr("Conversion downmixes and resamples every chapter. Original project audio is kept."); Layout.fillWidth: true; wrapMode: Text.WordWrap; font.pixelSize: 10; color: Theme.secondary }
        Rectangle { Layout.fillWidth: true; height: 1; color: Theme.border }
        RowLayout {
            Layout.fillWidth: true
            Label { objectName: "compilationStatusMessage"; text: dialog.delivery.compilationLoading ? qsTr("Checking accepted versions…") : dialog.delivery.compilationMessage; Layout.fillWidth: true; wrapMode: Text.WordWrap; font.pixelSize: 11; color: Theme.secondary }
            ActionButton { objectName: "checkCompilationButton"; text: qsTr("Check selection"); symbol: "check"; enabled: !backend.busy && !backend.dirty && !dialog.delivery.compilationLoading && dialog.selectedIDs.length > 0; onClicked: { dialog.configure(); dialog.delivery.checkCompilation() } }
        }
        Label { objectName: "compilationOutputSummary"; visible: !!dialog.delivery.compilationPreflight.ready; text: qsTr("%1 · %2 Hz · %3 channels · %4 subtitle cues").arg(dialog.duration(dialog.output.duration_ms)).arg(dialog.output.sample_rate_hz || 0).arg(dialog.output.channels || 0).arg(dialog.output.subtitle_cues || 0); Layout.fillWidth: true; wrapMode: Text.WordWrap; font.pixelSize: 11; color: Theme.text }
        ScrollView {
            visible: dialog.issueLines.length > 0
            Layout.fillWidth: true; Layout.preferredHeight: Math.min(100, issueText.implicitHeight)
            TextArea { id: issueText; objectName: "compilationIssues"; text: dialog.issueLines.join("\n"); readOnly: true; selectByMouse: true; wrapMode: TextEdit.Wrap; color: Theme.secondary; font.pixelSize: 11; background: Item { } }
        }
        RowLayout {
            Layout.fillWidth: true
            Label { text: qsTr("New folder"); font.pixelSize: 11; color: Theme.secondary }
            CompactTextField { id: folderName; objectName: "compilationFolderName"; Layout.fillWidth: true; maximumLength: 120; enabled: !backend.busy; placeholderText: qsTr("Folder name") }
            ActionButton { objectName: "chooseCompilationParentButton"; text: qsTr("Choose parent…"); symbol: "folder"; enabled: !backend.busy; onClicked: parentPicker.open() }
        }
        Label { objectName: "compilationParentPath"; text: dialog.delivery.compilationDestination || (dialog.parentFolder.toString().length ? decodeURIComponent(dialog.parentFolder.toString().replace(/^file:\/\//, "")) : qsTr("Choose an existing parent folder. The new export folder must not already exist.")); Layout.fillWidth: true; elide: Text.ElideMiddle; color: Theme.muted; font.pixelSize: 10 }
        ProgressBar { visible: dialog.delivery.compiling; indeterminate: true; Layout.fillWidth: true }
    }
    footer: Item {
        implicitHeight: 52
        RowLayout {
            anchors.fill: parent; anchors.margins: 12; spacing: 8
            Label { text: qsTr("WAV · chapter markers · source manifest"); color: Theme.muted; font.pixelSize: 10; Layout.fillWidth: true; elide: Text.ElideRight }
            ActionButton { text: qsTr("Close"); enabled: !dialog.delivery.compiling; onClicked: dialog.close() }
            ActionButton { objectName: "exportCompilationButton"; text: qsTr("Export compilation"); symbol: "export"; primary: true; enabled: !backend.busy && !backend.dirty && !!dialog.delivery.compilationPreflight.ready && dialog.parentFolder.toString().length > 0 && folderName.text.trim().length > 0; onClicked: dialog.delivery.exportCompilation(dialog.parentFolder, folderName.text) }
        }
    }
    Native.FolderDialog { id: parentPicker; title: qsTr("Choose parent folder for compilation"); onAccepted: if (dialog.delivery.compilationActive) dialog.parentFolder = selectedFolder }
    Connections { target: dialog.delivery; function onCompilationChanged() { if (dialog.opened && !dialog.delivery.compilationActive) dialog.close() } }
}
