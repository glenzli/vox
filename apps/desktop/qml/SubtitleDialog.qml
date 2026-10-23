pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Dialogs as Native

Dialog {
    id: dialog
    objectName: "subtitleDialog"
    title: qsTr("Subtitles")
    width: Math.min(880, parent.width - 48)
    height: Math.min(720, parent.height - 40)
    anchors.centerIn: parent
    modal: true
    padding: 16
    closePolicy: Popup.NoAutoClose
    readonly property var delivery: backend.delivery
    readonly property bool locked: backend.busy || delivery.subtitleLoading || delivery.subtitleDiscardPending
    readonly property var languageKeys: backend.languages.filter(item => item.key !== "auto").map(item => item.key)
    readonly property var candidateIDs: (delivery.subtitleCandidates || []).map(item => item.id)
    property bool alignmentMode: false
    property bool syncingCue: true
    property bool cueReady: false
    property string inspectorKey: ""
    property string inspectorRange: ""
    property string lastCandidateID: ""
    property string exportFormat: "srt"
    property string exportToken: ""
    property string exportTrackID: ""
    property string exportAudioID: ""
    property string importToken: ""

    function openForAudio(id) {
        confirmed.checked = false
        alignmentMode = false
        delivery.openSubtitles(id || backend.selectedAudio)
        if (delivery.subtitleActive) open()
    }
    function time(ms) {
        const value = Math.max(0, Math.round(ms || 0))
        return Math.floor(value / 3600000).toString().padStart(2, "0") + ":" + Math.floor(value / 60000 % 60).toString().padStart(2, "0") + ":" + Math.floor(value / 1000 % 60).toString().padStart(2, "0") + "." + (value % 1000).toString().padStart(3, "0")
    }
    function milliseconds(value) {
        const match = /^(\d{1,4}):([0-5]\d):([0-5]\d)\.(\d{3})$/.exec(value || "")
        return match ? Number(match[1]) * 3600000 + Number(match[2]) * 60000 + Number(match[3]) * 1000 + Number(match[4]) : -1
    }
    function syncCue() {
        if (!cueReady) return
        const cue = delivery.selectedSubtitleCue
        const key = delivery.subtitleReviewToken + ":" + delivery.subtitleCueIndex
        const range = (cue.start_time || "") + ":" + (cue.end_time || "")
        syncingCue = true
        if (cueStart.text !== (cue.start_time || "")) cueStart.text = cue.start_time || ""
        if (cueEnd.text !== (cue.end_time || "")) cueEnd.text = cue.end_time || ""
        if (cueText.text !== (cue.text || "")) cueText.text = cue.text || ""
        if (key !== inspectorKey) cueText.cursorPosition = cueText.length
        if (key !== inspectorKey || range !== inspectorRange) {
            const start = milliseconds(cue.start_time)
            const end = milliseconds(cue.end_time)
            cueSplitTime.text = start >= 0 && end > start ? time(Math.floor((start + end) / 2)) : ""
        }
        inspectorKey = key
        inspectorRange = range
        syncingCue = false
    }
    function updateCue() {
        if (cueReady && !syncingCue && delivery.subtitleCueIndex >= 0)
            delivery.updateSubtitleCue(cueStart.text, cueEnd.text, cueText.text)
    }
    function trackIdentity() {
        return delivery.subtitleTrack.revision_id || delivery.subtitleTrack.alignment_revision || "measured_segments"
    }
    Component.onCompleted: { cueReady = true; syncCue() }
    onClosed: {
        backend.stopRangePlayback()
        if (delivery.subtitleActive) {
            if (delivery.subtitleDraftDirty) {
                Qt.callLater(() => dialog.open())
                delivery.requestCloseSubtitles()
            } else delivery.closeSubtitles()
        }
    }
    background: Rectangle { color: Theme.panel; radius: 10; border.color: Theme.border }
    contentItem: ColumnLayout {
        spacing: 6
        Shortcut {
            sequence: "Esc"
            enabled: dialog.opened && !discardDialog.opened && !subtitleFile.visible && !importFile.visible
            onActivated: dialog.delivery.requestCloseSubtitles()
        }
        Shortcut {
            objectName: "subtitleSaveShortcut"
            sequences: [StandardKey.Save]
            enabled: dialog.opened && !dialog.locked && dialog.delivery.canSaveSubtitleDraft && !discardDialog.visible && !subtitleFile.visible && !importFile.visible
            onActivated: dialog.delivery.saveSubtitleDraft()
        }
        RowLayout {
            Layout.fillWidth: true
            Label {
                objectName: "subtitleSourceLabel"
                text: { const language = preferences.language; return delivery.subtitleSourceLabel(delivery.subtitleTrack.source || "") }
                font.weight: Font.DemiBold; font.pixelSize: 12
            }
            Label {
                objectName: "subtitleDraftState"
                text: dialog.delivery.subtitleDraftDirty ? qsTr("Unsaved changes") : dialog.delivery.candidateID.length ? qsTr("Candidate · not accepted") : (dialog.delivery.subtitleTrack.cues || []).length ? qsTr("Current saved track") : qsTr("No saved subtitles")
                color: Theme.secondary; font.pixelSize: 11; Layout.fillWidth: true
            }
            Label { text: qsTr("%1 cues · %2").arg(dialog.delivery.subtitleDraftCues.length).arg(dialog.time(dialog.delivery.subtitleDurationMs)); color: Theme.secondary; font.pixelSize: 11 }
            IconButton { objectName: "subtitlePlayAudioButton"; symbol: "play"; hint: qsTr("Listen to this audio"); enabled: !!delivery.audioID && !dialog.locked; onClicked: { backend.stopRangePlayback(); backend.selectAudio(delivery.audioID); backend.playSelected() } }
            IconButton { objectName: "subtitleStopRangeButton"; symbol: "stop"; hint: qsTr("Stop cue audition"); visible: backend.rangePlaybackActive; onClicked: backend.stopRangePlayback() }
        }
        Label {
            objectName: "subtitleProvenance"
            text: dialog.delivery.subtitleTrack.source === "manual_revision" ? qsTr("Manually corrected words and times. Review against this audio before accepting.") : dialog.delivery.subtitleTrack.source === "imported_subtitles" ? qsTr("Words and times from an imported file. Review against this audio before accepting.") : qsTr("Measured timings follow the assembled segments. Aligned timings are model estimates; review them before accepting.")
            Layout.fillWidth: true; wrapMode: Text.WordWrap; color: Theme.secondary; font.pixelSize: 11
        }
        RowLayout {
            Layout.fillWidth: true
            CompactComboBox {
                id: candidateChoice
                objectName: "subtitleCandidateChoice"
                Layout.fillWidth: true
                model: [qsTr("Current subtitles")].concat((dialog.delivery.subtitleCandidates || []).map((item, index) => qsTr("Candidate %1 · %2").arg(index + 1).arg(dialog.delivery.subtitleSourceLabel(item.source || "forced_alignment"))))
                selectionIndex: dialog.delivery.candidateID.length ? dialog.candidateIDs.indexOf(dialog.delivery.candidateID) + 1 : 0
                enabled: !dialog.locked
                onActivated: index => dialog.delivery.previewSubtitleCandidate(index > 0 ? dialog.candidateIDs[index - 1] : "")
            }
            ActionButton { objectName: "importSubtitleButton"; text: qsTr("Import…"); symbol: "import"; enabled: dialog.delivery.acceptedAudio && !dialog.locked; onClicked: dialog.delivery.requestSubtitleImport() }
            ActionButton { objectName: "acceptSubtitleCandidateButton"; text: qsTr("Accept subtitles"); symbol: "check"; primary: true; visible: dialog.delivery.candidateID.length > 0; enabled: !dialog.locked && !dialog.delivery.subtitleDraftDirty && dialog.delivery.subtitleDraftCues.length > 0; onClicked: dialog.delivery.acceptSubtitleCandidate() }
            IconButton { objectName: "discardSubtitleCandidateButton"; symbol: "x"; hint: qsTr("Discard candidate subtitles"); visible: dialog.delivery.candidateID.length > 0; enabled: !dialog.locked && !dialog.delivery.subtitleDraftDirty; onClicked: dialog.delivery.discardSubtitleCandidate() }
        }
        RowLayout {
            Layout.fillWidth: true
            ActionButton { objectName: "subtitleCuesTab"; text: qsTr("Edit cues"); symbol: "edit"; primary: !dialog.alignmentMode; onClicked: dialog.alignmentMode = false }
            ActionButton { objectName: "subtitleAlignmentTab"; text: qsTr("Realign from transcript"); symbol: "refresh"; primary: dialog.alignmentMode; onClicked: dialog.alignmentMode = true }
            Item { Layout.fillWidth: true }
            Label { visible: !dialog.alignmentMode; text: dialog.delivery.subtitleCueIndex >= 0 ? qsTr("Cue %1 of %2").arg(dialog.delivery.subtitleCueIndex + 1).arg(dialog.delivery.subtitleDraftCues.length) : qsTr("Cue inspector"); color: Theme.secondary; font.pixelSize: 11 }
            IconButton { objectName: "auditionSubtitleCueButton"; visible: !dialog.alignmentMode; symbol: "headphones"; hint: qsTr("Listen to selected cue"); enabled: !dialog.locked && dialog.delivery.subtitleCueIndex >= 0 && dialog.milliseconds(cueStart.text) >= 0 && dialog.milliseconds(cueEnd.text) > dialog.milliseconds(cueStart.text) && dialog.milliseconds(cueEnd.text) <= dialog.delivery.subtitleDurationMs; onClicked: dialog.delivery.auditionSubtitleCue() }
            IconButton { objectName: "addSubtitleCueButton"; visible: !dialog.alignmentMode; symbol: "add"; hint: qsTr("Add a cue in the next gap"); enabled: !dialog.locked && dialog.delivery.acceptedAudio && (dialog.delivery.subtitleTrack.cues || []).length > 0; onClicked: dialog.delivery.addSubtitleCue() }
            IconButton { objectName: "removeSubtitleCueButton"; visible: !dialog.alignmentMode; symbol: "x"; hint: qsTr("Remove selected cue"); enabled: !dialog.locked && dialog.delivery.subtitleCueIndex >= 0; onClicked: dialog.delivery.removeSubtitleCue() }
        }
        RowLayout {
            visible: !dialog.alignmentMode
            Layout.fillWidth: true; Layout.fillHeight: true; spacing: 12
            Rectangle {
                Layout.preferredWidth: 284; Layout.fillHeight: true
                color: Theme.inset; radius: 5; border.color: Theme.border
                ListView {
                    id: cueList
                    objectName: "subtitleCueList"
                    anchors.fill: parent; anchors.margins: 5
                    clip: true; spacing: 2
                    model: dialog.delivery.subtitleDraftCues
                    currentIndex: dialog.delivery.subtitleCueIndex
                    ScrollBar.vertical: ScrollBar { }
                    delegate: ItemDelegate {
                        id: cue
                        required property var modelData
                        required property int index
                        objectName: "subtitleCueSelect-" + index
                        width: cueList.width; height: 66; padding: 7
                        enabled: !dialog.locked
                        highlighted: dialog.delivery.subtitleCueIndex === index
                        onClicked: dialog.delivery.selectSubtitleCue(index)
                        contentItem: ColumnLayout {
                            spacing: 4
                            Label { text: (cue.index + 1) + "  " + cue.modelData.start_time + " – " + cue.modelData.end_time; Layout.fillWidth: true; font.family: "monospace"; font.pixelSize: 10; color: Theme.secondary; elide: Text.ElideRight }
                            Label { objectName: "subtitleCueText-" + cue.index; text: cue.modelData.text || qsTr("Empty cue"); Layout.fillWidth: true; Layout.fillHeight: true; maximumLineCount: 2; elide: Text.ElideRight; wrapMode: Text.WordWrap; color: Theme.text; font.pixelSize: 12 }
                        }
                        background: Rectangle { radius: 4; color: cue.highlighted ? Theme.selected : cue.hovered ? Theme.hover : "transparent" }
                    }
                }
                Label { anchors.centerIn: parent; width: parent.width - 28; visible: !cueList.count; text: dialog.delivery.subtitleLoading ? qsTr("Loading timings…") : qsTr("Import subtitles or realign a confirmed transcript."); horizontalAlignment: Text.AlignHCenter; wrapMode: Text.WordWrap; color: Theme.muted; font.pixelSize: 11 }
            }
            ColumnLayout {
                Layout.fillWidth: true; Layout.fillHeight: true; spacing: 6
                GridLayout {
                    columns: 2; rowSpacing: 4; columnSpacing: 10; Layout.fillWidth: true
                    Label { text: qsTr("Start"); color: Theme.secondary; font.pixelSize: 11 }
                    Label { text: qsTr("End"); color: Theme.secondary; font.pixelSize: 11 }
                    CompactTextField { id: cueStart; objectName: "subtitleCueStart"; Layout.fillWidth: true; font.family: "monospace"; font.pixelSize: 12; placeholderText: "00:00:00.000"; selectByMouse: true; enabled: !dialog.locked && dialog.delivery.subtitleCueIndex >= 0; onTextChanged: dialog.updateCue(); Accessible.name: qsTr("Cue start time") }
                    CompactTextField { id: cueEnd; objectName: "subtitleCueEnd"; Layout.fillWidth: true; font.family: "monospace"; font.pixelSize: 12; placeholderText: "00:00:00.000"; selectByMouse: true; enabled: !dialog.locked && dialog.delivery.subtitleCueIndex >= 0; onTextChanged: dialog.updateCue(); Accessible.name: qsTr("Cue end time") }
                }
                ScrollView {
                    id: cueScroll
                    Layout.fillWidth: true; Layout.fillHeight: true; Layout.minimumHeight: 64
                    TextArea {
                        id: cueText
                        objectName: "subtitleCueEditor"
                        width: cueScroll.availableWidth
                        height: Math.max(implicitHeight, cueScroll.availableHeight)
                        color: Theme.text; selectionColor: Theme.selected; selectedTextColor: Theme.text
                        wrapMode: TextEdit.Wrap; selectByMouse: true; font.pixelSize: 13
                        readOnly: dialog.locked || dialog.delivery.subtitleCueIndex < 0
                        placeholderText: qsTr("Select a cue to edit its text")
                        onTextChanged: dialog.updateCue()
                        Accessible.name: qsTr("Selected cue text")
                        background: Rectangle { color: Theme.inset; radius: 5; border.color: Theme.border }
                    }
                }
                RowLayout {
                    Layout.fillWidth: true
                    Label { text: qsTr("Split time"); color: Theme.secondary; font.pixelSize: 11 }
                    CompactTextField { id: cueSplitTime; objectName: "subtitleCueSplitTime"; Layout.fillWidth: true; font.family: "monospace"; font.pixelSize: 11; selectByMouse: true; enabled: !dialog.locked && dialog.delivery.subtitleCueIndex >= 0; Accessible.name: qsTr("Split boundary time") }
                    ActionButton { objectName: "splitSubtitleCueButton"; text: qsTr("Split at cursor"); enabled: !dialog.locked && dialog.delivery.subtitleCueIndex >= 0 && cueText.cursorPosition > 0 && cueText.cursorPosition < cueText.length; onClicked: dialog.delivery.splitSubtitleCue(cueText.cursorPosition, cueSplitTime.text); ToolTip.visible: hovered; ToolTip.text: qsTr("Split uses the text cursor and the time above. Merge joins the next cue, including any gap.") }
                }
                Label { text: qsTr("hh:mm:ss.mmm · millisecond precision"); color: Theme.muted; font.pixelSize: 10 }
                Label { objectName: "subtitleDraftIssue"; text: dialog.delivery.subtitleDraftDirty ? dialog.delivery.subtitleDraftIssue : ""; visible: text.length > 0; Layout.fillWidth: true; wrapMode: Text.WordWrap; color: Theme.secondary; font.pixelSize: 11 }
                RowLayout {
                    Layout.fillWidth: true
                    ActionButton { objectName: "mergeSubtitleCueButton"; text: qsTr("Merge with next"); enabled: !dialog.locked && dialog.delivery.subtitleCueIndex >= 0 && dialog.delivery.subtitleCueIndex + 1 < dialog.delivery.subtitleDraftCues.length; onClicked: dialog.delivery.mergeNextSubtitleCue() }
                    ActionButton { objectName: "saveSubtitleDraftButton"; text: qsTr("Save as candidate"); symbol: "save"; primary: true; enabled: dialog.delivery.canSaveSubtitleDraft && !dialog.locked; onClicked: dialog.delivery.saveSubtitleDraft() }
                    IconButton { objectName: "resetSubtitleDraftButton"; hint: qsTr("Reset changes"); symbol: "refresh"; enabled: dialog.delivery.subtitleDraftDirty && !dialog.locked; onClicked: dialog.delivery.resetSubtitleDraft() }
                    Item { Layout.fillWidth: true }
                }
            }
        }
        ColumnLayout {
            visible: dialog.alignmentMode
            Layout.fillWidth: true; Layout.fillHeight: true; spacing: 8
            Label { text: qsTr("Exact spoken transcript"); font.weight: Font.DemiBold; font.pixelSize: 12 }
            Label { text: qsTr("Check these words against the audio. After Echo cuts or edits, update the transcript to match exactly. This does not change the script."); Layout.fillWidth: true; wrapMode: Text.WordWrap; color: Theme.secondary; font.pixelSize: 11 }
            ScrollView {
                Layout.fillWidth: true; Layout.fillHeight: true; Layout.minimumHeight: 100
                TextArea {
                    id: transcript
                    objectName: "subtitleTranscript"
                    text: dialog.delivery.transcript
                    color: Theme.text; selectionColor: Theme.selected; selectedTextColor: Theme.text
                    wrapMode: TextEdit.Wrap; selectByMouse: true; font.pixelSize: 12
                    readOnly: dialog.locked
                    placeholderText: qsTr("Enter the words actually spoken in this audio")
                    onTextChanged: { if (dialog.delivery.transcript !== text) { dialog.delivery.transcript = text; confirmed.checked = false } }
                    background: Rectangle { color: Theme.inset; radius: 5; border.color: Theme.border }
                }
            }
            RowLayout {
                Layout.fillWidth: true; spacing: 8
                CompactComboBox {
                    id: language
                    objectName: "subtitleLanguage"
                    Layout.preferredWidth: 160
                    model: backend.languages.filter(item => item.key !== "auto").map(item => item.label)
                    selectionIndex: dialog.languageKeys.indexOf(dialog.delivery.language)
                    displayText: currentIndex < 0 ? qsTr("Choose language") : currentText
                    enabled: !dialog.locked
                    onActivated: index => { dialog.delivery.language = dialog.languageKeys[index]; confirmed.checked = false }
                }
                CheckBox { id: confirmed; objectName: "subtitleTranscriptConfirmed"; text: qsTr("These words match this audio"); font.pixelSize: 11; Layout.fillWidth: true; enabled: !dialog.locked }
                ActionButton { objectName: "alignSubtitlesButton"; text: qsTr("Realign"); symbol: "refresh"; enabled: dialog.delivery.acceptedAudio && !dialog.locked && backend.runtimeReady && confirmed.checked && transcript.text.trim().length > 0 && dialog.delivery.language.length > 0; onClicked: dialog.delivery.alignSubtitles(confirmed.checked) }
            }
        }
        Label { text: qsTr("Corrections and imports create a new candidate. Accept it separately to use it for export. Audio and script stay unchanged."); Layout.fillWidth: true; wrapMode: Text.WordWrap; color: Theme.secondary; font.pixelSize: 11 }
        ProgressBar { Layout.fillWidth: true; visible: dialog.delivery.aligning; from: 0; to: Math.max(1, backend.total); value: backend.completed; indeterminate: backend.completed <= 0 }
        RowLayout {
            Layout.fillWidth: true
            Label { objectName: "subtitleStatusMessage"; text: dialog.delivery.subtitleLoading ? qsTr("Loading timings…") : dialog.delivery.subtitleMessage || (!dialog.delivery.acceptedAudio ? qsTr("Accept this audio version before realigning.") : dialog.alignmentMode && !backend.runtimeReady ? qsTr("Connect Runtime to realign. Saved timings can still be exported.") : ""); Layout.fillWidth: true; wrapMode: Text.WordWrap; color: Theme.secondary; font.pixelSize: 11 }
            ActionButton { objectName: "stopSubtitleAlignmentButton"; text: qsTr("Stop alignment"); symbol: "stop"; visible: dialog.delivery.aligning; onClicked: dialog.delivery.cancelAlignment() }
        }
    }
    footer: Item {
        implicitHeight: 52
        RowLayout {
            anchors.fill: parent; anchors.margins: 12; spacing: 8
            CompactComboBox { id: format; objectName: "subtitleFormat"; Layout.preferredWidth: 146; model: [qsTr("SubRip (.srt)"), qsTr("WebVTT (.vtt)")]; enabled: !dialog.locked }
            Item { Layout.fillWidth: true }
            ActionButton { objectName: "closeSubtitleDialogButton"; text: qsTr("Close"); enabled: !dialog.locked; onClicked: dialog.delivery.requestCloseSubtitles() }
            ActionButton { objectName: "saveSubtitlesButton"; text: qsTr("Export subtitles…"); symbol: "export"; primary: true; enabled: dialog.delivery.canExportSubtitles && !dialog.locked; onClicked: { dialog.exportFormat = format.currentIndex === 1 ? "vtt" : "srt"; dialog.exportToken = dialog.delivery.subtitleReviewToken; dialog.exportAudioID = dialog.delivery.audioID; dialog.exportTrackID = dialog.trackIdentity(); subtitleFile.open() } }
        }
    }
    Dialog {
        id: discardDialog
        objectName: "discardSubtitleDraftDialog"
        title: qsTr("Discard unsaved subtitle changes?")
        anchors.centerIn: parent
        width: Math.min(430, dialog.width - 40)
        implicitHeight: 184
        modal: true
        closePolicy: Popup.CloseOnEscape
        padding: 16
        background: Rectangle { color: Theme.panel; radius: 8; border.color: Theme.border }
        contentItem: Item {
            implicitHeight: 56
            Label { anchors.fill: parent; text: qsTr("Your cue edits have not been saved as a candidate. Discard them to continue?"); wrapMode: Text.WordWrap; verticalAlignment: Text.AlignVCenter; color: Theme.text; font.pixelSize: 12 }
        }
        footer: Item {
            implicitHeight: 44
            RowLayout {
                anchors.fill: parent; anchors.margins: 8; spacing: 8
                Item { Layout.fillWidth: true }
                ActionButton { objectName: "keepSubtitleDraftButton"; text: qsTr("Keep editing"); onClicked: discardDialog.reject() }
                ActionButton { objectName: "confirmDiscardSubtitleDraftButton"; text: qsTr("Discard changes"); primary: true; onClicked: discardDialog.accept() }
            }
        }
        onAccepted: dialog.delivery.resolveSubtitleDiscard(true)
        onRejected: dialog.delivery.resolveSubtitleDiscard(false)
        onClosed: { if (dialog.delivery.subtitleDiscardPending) dialog.delivery.resolveSubtitleDiscard(false) }
    }
    Native.FileDialog {
        id: subtitleFile
        objectName: "subtitleExportFileDialog"
        title: qsTr("Export subtitles")
        fileMode: Native.FileDialog.SaveFile
        defaultSuffix: dialog.exportFormat
        nameFilters: dialog.exportFormat === "vtt" ? [qsTr("WebVTT subtitles (*.vtt)")] : [qsTr("SubRip subtitles (*.srt)")]
        onAccepted: { if (dialog.exportToken === dialog.delivery.subtitleReviewToken && dialog.exportAudioID === dialog.delivery.audioID && dialog.exportTrackID === dialog.trackIdentity() && dialog.delivery.canExportSubtitles) dialog.delivery.exportSubtitles(selectedFile, dialog.exportFormat) }
    }
    Native.FileDialog {
        id: importFile
        objectName: "subtitleImportFileDialog"
        title: qsTr("Import subtitles as a candidate")
        fileMode: Native.FileDialog.OpenFile
        nameFilters: [qsTr("Subtitle files (*.srt *.vtt)")]
        onAccepted: { const format = selectedFile.toString().toLowerCase().endsWith(".vtt") ? "vtt" : "srt"; dialog.delivery.importSubtitles(selectedFile, format, dialog.importToken) }
    }
    Connections {
        target: dialog.delivery
        function onSubtitleChanged() {
            dialog.syncCue()
            if (dialog.opened && !dialog.delivery.subtitleActive) dialog.close()
            if (!dialog.delivery.subtitleDiscardPending && discardDialog.opened) discardDialog.close()
            if (dialog.delivery.candidateID.length && dialog.lastCandidateID !== dialog.delivery.candidateID && !dialog.delivery.subtitleLoading) dialog.alignmentMode = false
            if (!dialog.delivery.subtitleLoading) dialog.lastCandidateID = dialog.delivery.candidateID
        }
        function onSubtitleDiscardRequested() { discardDialog.open() }
        function onSubtitleImportRequested() { dialog.importToken = dialog.delivery.subtitleReviewToken; importFile.open() }
        function onFormChanged() { if (transcript.text !== dialog.delivery.transcript) transcript.text = dialog.delivery.transcript; confirmed.checked = false }
    }
}
