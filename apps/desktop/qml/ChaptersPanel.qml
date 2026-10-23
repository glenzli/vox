pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Dialogs as Native

Rectangle {
    id: panel
    color: Theme.window
    readonly property var activeChapters: backend.chapters.filter(chapter => !chapter.archived)
    readonly property var interrupted: activeChapters.filter(chapter => chapter.interrupted_unknown)
    readonly property int acceptedCount: activeChapters.filter(chapter => chapter.status === "accepted").length
    property string dialogIdentity: ""
    function openAdd() {
        chapterDialog.chapterId = ""
        chapterTitle.text = qsTr("Chapter %1").arg(activeChapters.length + 1)
        dialogIdentity = backend.collectionPath
        chapterDialog.open()
    }
    function openRename(chapter) {
        chapterDialog.chapterId = chapter.id
        chapterTitle.text = chapter.title
        dialogIdentity = backend.collectionPath
        chapterDialog.open()
    }
    ColumnLayout {
        anchors.fill: parent; anchors.margins: 12; spacing: 8
        RowLayout {
            Label { objectName: "chaptersHeading"; text: qsTr("CHAPTERS"); color: Theme.secondary; font.pixelSize: 10; font.letterSpacing: 1.3; Layout.fillWidth: true }
            IconButton { objectName: "addChapterButton"; symbol: "add"; hint: qsTr("Add chapter"); enabled: !backend.busy; onClicked: panel.openAdd() }
            IconButton { objectName: "importChaptersButton"; symbol: "import"; hint: qsTr("Import text as chapters") + "\n" + qsTr("Markdown uses repeated # headings, or repeated ## headings when needed. Code fences are ignored. Long text splits at paragraphs."); enabled: !backend.busy; onClicked: { panel.dialogIdentity = backend.collectionPath; importFiles.open() } }
            IconButton { objectName: "refreshChaptersButton"; symbol: "refresh"; hint: qsTr("Refresh chapters"); enabled: !backend.busy; onClicked: backend.refreshChapters() }
        }
        Label { text: qsTr("%1 chapters · %2 accepted").arg(panel.activeChapters.length).arg(panel.acceptedCount); font.pixelSize: 10; color: Theme.muted }
        Label { objectName: "chapterStateError"; visible: backend.chapterError.length > 0; text: { const language = preferences.language; return backend.chapterError } Layout.fillWidth: true; wrapMode: Text.WordWrap; color: Theme.secondary; font.pixelSize: 11 }
        ListView {
            id: chapterList; objectName: "chapterList"
            Layout.fillWidth: true; Layout.fillHeight: true
            clip: true; spacing: 5; model: panel.activeChapters
            ScrollBar.vertical: ScrollBar { }
            delegate: Rectangle {
                id: chapter
                required property var modelData
                required property int index
                readonly property bool selected: backend.selectedChapter === modelData.id
                readonly property bool activeJob: backend.batchingChapters && backend.chapterProgress.chapter_id === modelData.id
                width: chapterList.width; height: details.implicitHeight + 16
                color: selected ? Theme.selected : Theme.inset; radius: 6; border.color: selected ? Theme.secondary : Theme.border
                Accessible.role: Accessible.Button
                Accessible.name: modelData.title
                Accessible.selected: selected
                Accessible.onPressAction: backend.selectChapter(modelData.id)
                MouseArea { anchors.fill: parent; enabled: !backend.busy; onClicked: backend.selectChapter(chapter.modelData.id); onDoubleClicked: panel.openRename(chapter.modelData) }
                ColumnLayout {
                    id: details
                    anchors.fill: parent; anchors.margins: 8; spacing: 4
                    Label { objectName: "chapterTitle-" + chapter.index; text: (chapter.index + 1) + ". " + chapter.modelData.title; Layout.fillWidth: true; wrapMode: Text.WordWrap; maximumLineCount: 2; elide: Text.ElideRight; font.pixelSize: 12; font.weight: chapter.selected ? Font.DemiBold : Font.Normal }
                    Label { objectName: "chapterStatus-" + chapter.index; text: { const language = preferences.language; return backend.productionStatus(backend.chapterError.length ? "unavailable" : chapter.activeJob ? backend.chapterProgress.status : chapter.modelData.status) } Layout.fillWidth: true; wrapMode: Text.WordWrap; font.pixelSize: 10; color: Theme.secondary }
                    ProgressBar { visible: chapter.activeJob; Layout.fillWidth: true; from: 0; to: Math.max(1,backend.chapterProgress.total_segments || 0); value: backend.chapterProgress.completed_segments || 0; indeterminate: !backend.chapterProgress.total_segments }
                    Label { visible: !chapter.activeJob && chapter.modelData.total_segments > 0; text: qsTr("%1 / %2 segments").arg(chapter.modelData.completed_segments).arg(chapter.modelData.total_segments); font.pixelSize: 10; color: Theme.muted }
                    RowLayout {
                        visible: chapter.selected; Layout.fillWidth: true; spacing: 3
                        IconButton { objectName: "renameChapterButton"; symbol: "edit"; hint: qsTr("Rename chapter"); enabled: !backend.busy; onClicked: panel.openRename(chapter.modelData) }
                        IconButton { objectName: "moveChapterUpButton"; symbol: "arrow-up"; hint: qsTr("Move chapter up"); enabled: !backend.busy && chapter.index > 0; onClicked: backend.moveChapter(chapter.modelData.id, -1) }
                        IconButton { objectName: "moveChapterDownButton"; symbol: "arrow-down"; hint: qsTr("Move chapter down"); enabled: !backend.busy && chapter.index + 1 < panel.activeChapters.length; onClicked: backend.moveChapter(chapter.modelData.id, 1) }
                        Item { Layout.fillWidth: true }
                        IconButton { objectName: "removeChapterButton"; symbol: "x"; hint: qsTr("Remove chapter from this set"); enabled: !backend.busy; onClicked: backend.archiveChapter(chapter.modelData.id) }
                    }
                }
            }
            Label { anchors.centerIn: parent; width: parent.width; visible: !panel.activeChapters.length && !backend.chapterError.length; text: qsTr("Import Markdown sections as chapters, or split long text at paragraph boundaries."); horizontalAlignment: Text.AlignHCenter; wrapMode: Text.WordWrap; color: Theme.muted; font.pixelSize: 11 }
        }
        Rectangle { Layout.fillWidth: true; Layout.preferredHeight: 1; color: Theme.border }
        ActionButton { objectName: "shareChapterSettingsButton"; Layout.fillWidth: true; text: qsTr("Share chapter voices…"); symbol: "users"; enabled: !backend.busy && backend.selectedChapter.length > 0; onClicked: shareDialog.open() }
        Label { text: qsTr("Shared voices, pronunciation and cues apply to the next generation. Chapter text stays separate."); Layout.fillWidth: true; wrapMode: Text.WordWrap; font.pixelSize: 10; color: Theme.muted }
        ActionButton { objectName: "generateChaptersButton"; Layout.fillWidth: true; text: qsTr("Generate pending"); symbol: "waveform"; primary: true; enabled: !backend.busy && backend.runtimeReady && !backend.chapterError.length && panel.activeChapters.length > 0; onClicked: backend.renderChapterBatch(false) }
        ActionButton { objectName: "resumeChaptersButton"; visible: panel.interrupted.length > 0; Layout.fillWidth: true; text: qsTr("Resume interrupted"); symbol: "refresh"; enabled: !backend.busy && backend.runtimeReady; onClicked: backend.renderChapterBatch(true) }
        ColumnLayout {
            visible: backend.batchingChapters; Layout.fillWidth: true
            Label { text: qsTr("Chapter %1 / %2").arg(backend.chapterProgress.chapter_id ? (backend.chapterProgress.chapter_index || 0) + 1 : 0).arg(backend.chapterProgress.total_chapters || panel.activeChapters.length); color: Theme.secondary; font.pixelSize: 11 }
            ActionButton { objectName: "stopChaptersButton"; Layout.fillWidth: true; text: qsTr("Stop after current segment"); symbol: "stop"; onClicked: backend.cancel() }
        }
        ActionButton { objectName: "exportChaptersButton"; Layout.fillWidth: true; text: qsTr("Export accepted set…"); symbol: "export"; enabled: !backend.busy && !backend.dirty && !backend.chapterError.length && panel.acceptedCount > 0; onClicked: { panel.dialogIdentity = backend.collectionPath; exportFolder.open() } }
        ActionButton { objectName: "compileChaptersButton"; Layout.fillWidth: true; text: qsTr("Compile chapters…"); symbol: "export"; enabled: !backend.busy && !backend.dirty && !backend.chapterError.length && panel.activeChapters.length > 0; onClicked: compilationDialog.openForCollection() }
        Label { text: qsTr("Matching segments are reused. Changed chapters receive new candidates for review."); Layout.fillWidth: true; wrapMode: Text.WordWrap; font.pixelSize: 10; color: Theme.muted }
    }
    Dialog {
        id: chapterDialog; objectName: "chapterTitleDialog"
        property string chapterId: ""
        title: chapterId.length ? qsTr("Rename chapter") : qsTr("Add chapter")
        anchors.centerIn: Overlay.overlay; modal: true; width: 400
        background: Rectangle { color: Theme.panel; radius: 8; border.color: Theme.border }
        ColumnLayout {
            anchors.fill: parent; spacing: 12
            CompactTextField { id: chapterTitle; objectName: "newChapterTitle"; Layout.fillWidth: true; maximumLength: 120; placeholderText: qsTr("Chapter title"); onAccepted: if (text.trim().length) confirmChapter.clicked() }
            RowLayout {
                Item { Layout.fillWidth: true }
                ActionButton { text: qsTr("Cancel"); onClicked: chapterDialog.close() }
                ActionButton { id: confirmChapter; objectName: "confirmChapterTitle"; text: chapterDialog.chapterId.length ? qsTr("Rename") : qsTr("Add chapter"); primary: true; enabled: !backend.busy && chapterTitle.text.trim().length > 0; onClicked: { if (panel.dialogIdentity === backend.collectionPath) { if (chapterDialog.chapterId.length) backend.renameChapter(chapterDialog.chapterId, chapterTitle.text); else backend.addChapter(chapterTitle.text, "", "plain") } chapterDialog.close() } }
            }
        }
        onOpened: { chapterTitle.forceActiveFocus(); chapterTitle.selectAll() }
    }
    Dialog {
        id: shareDialog
        title: qsTr("Share voices and pronunciation")
        anchors.centerIn: Overlay.overlay; modal: true; width: 440
        standardButtons: Dialog.Apply | Dialog.Cancel
        background: Rectangle { color: Theme.panel; radius: 8; border.color: Theme.border }
        Label { width: parent.width; text: qsTr("Use this chapter’s default voice, role choices, pronunciation rules and cue bindings for all chapters on their next generation?"); wrapMode: Text.WordWrap; color: Theme.text }
        onOpened: { standardButton(Dialog.Apply).text = qsTr("Use for all chapters"); standardButton(Dialog.Cancel).text = qsTr("Cancel") }
        onApplied: { backend.publishChapterSettings(); close() }
    }
    Native.FileDialog { id: importFiles; title: qsTr("Import text as chapters"); fileMode: Native.FileDialog.OpenFiles; nameFilters: [qsTr("Text files (*.txt *.md)"),qsTr("All files (*)")]; onAccepted: if (panel.dialogIdentity === backend.collectionPath) backend.importChapters(selectedFiles) }
    Native.FileDialog { id: exportFolder; title: qsTr("Export accepted chapter set"); fileMode: Native.FileDialog.SaveFile; onAccepted: if (panel.dialogIdentity === backend.collectionPath) backend.exportChapters(selectedFile) }
    CompilationDialog { id: compilationDialog; anchors.centerIn: Overlay.overlay }
}
