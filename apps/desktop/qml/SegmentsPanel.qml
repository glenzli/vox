pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

ColumnLayout {
    id: panel
    objectName: "segmentsPanel"
    spacing: 12
    property string readinessHint: ""
    readonly property bool current: !backend.dirty && !!backend.segmentPlan.document_digest
    readonly property var segments: current ? (backend.segmentPlan.segments || []) : []
    readonly property bool ready: current && !!backend.segmentPlan.ready
    readonly property var selectedTake: (backend.project.candidates || []).concat(backend.project.audio || []).find(take => take.id === backend.selectedAudio) || null
    signal editLineRequested(int line)
    signal pronunciationRequested()
    signal subtitleRequested()

    RowLayout {
        Layout.fillWidth: true
        Label { text: qsTr("Speech segments"); font.pixelSize: 17; font.weight: Font.DemiBold; Layout.fillWidth: true }
        ActionButton { objectName: "segmentPronunciationButton"; text: qsTr("Pronunciation"); enabled: !backend.busy; onClicked: panel.pronunciationRequested() }
    }
    Label {
        Layout.fillWidth: true
        text: qsTr("Regenerate a passage, compare its alternatives, then accept the one to use. Assemble accepted segments into a take for final review.")
        color: Theme.secondary
        font.pixelSize: 12
        wrapMode: Text.WordWrap
    }
    Label {
        objectName: "segmentsReadiness"
        Layout.fillWidth: true
        text: !panel.current ? qsTr("Checking script…") : !panel.ready ? panel.readinessHint : backend.segmentPlan.all_accepted ? qsTr("All segments accepted") : qsTr("%1 segments · accept one alternative for each passage").arg(panel.segments.length)
        color: Theme.secondary
        wrapMode: Text.WordWrap
        font.pixelSize: 11
    }
    ScrollView {
        id: scroll
        Layout.fillWidth: true
        Layout.fillHeight: true
        contentWidth: availableWidth
        clip: true
        ColumnLayout {
            width: scroll.availableWidth
            spacing: 10
            Label {
                visible: panel.current && !panel.segments.length
                Layout.fillWidth: true
                text: qsTr("Write spoken text to create segments.")
                color: Theme.muted
                wrapMode: Text.WordWrap
            }
            Repeater {
                model: panel.segments
                Rectangle {
                    id: segment
                    required property var modelData
                    required property int index
                    objectName: "segmentCard-" + index
                    Layout.fillWidth: true
                    implicitHeight: content.implicitHeight + 24
                    radius: 7
                    color: Theme.inset
                    border.color: Theme.border
                    property string selectedId: ""
                    property int candidateCount: 0
                    // Time-ordered segment IDs keep take numbers stable when acceptance
                    // moves a recording from candidates to accepted storage.
                    readonly property var alternatives: (modelData.candidates || []).slice().sort((a, b) => a.id < b.id ? -1 : a.id > b.id ? 1 : 0)
                    readonly property int selectedIndex: alternatives.findIndex(candidate => candidate.id === selectedId)
                    readonly property var selectedCandidate: selectedIndex >= 0 ? alternatives[selectedIndex] : null
                    function syncSelection() {
                        if (alternatives.length > candidateCount || selectedIndex < 0) {
                            const pending = alternatives.filter(candidate => !candidate.accepted)
                            selectedId = pending.length ? pending[pending.length - 1].id
                                : modelData.accepted_id || (alternatives.length ? alternatives[alternatives.length - 1].id : "")
                        }
                        candidateCount = alternatives.length
                    }
                    onModelDataChanged: syncSelection()
                    Component.onCompleted: syncSelection()
                    ColumnLayout {
                        id: content
                        anchors.fill: parent
                        anchors.margins: 12
                        spacing: 8
                        RowLayout {
                            Layout.fillWidth: true
                            Flow {
                                Layout.fillWidth: true
                                Layout.minimumWidth: 0
                                spacing: 8
                                Label { width: Math.min(implicitWidth, parent.width); text: segment.modelData.role || qsTr("Narration"); textFormat: Text.PlainText; font.weight: Font.DemiBold; font.pixelSize: 12; wrapMode: Text.Wrap }
                                Label { width: Math.min(implicitWidth, parent.width); text: (backend.languages.find(language => language.key === (segment.modelData.language || "auto")) || {}).label || segment.modelData.language; font.pixelSize: 11; color: Theme.secondary; wrapMode: Text.Wrap }
                                Label { width: Math.min(implicitWidth, parent.width); text: segment.modelData.accepted_id ? qsTr("Accepted") : qsTr("Needs a take"); font.pixelSize: 11; color: Theme.secondary; wrapMode: Text.Wrap }
                            }
                            IconButton { objectName: "segmentSource-" + segment.index; symbol: "edit"; hint: qsTr("Edit line %1").arg(segment.modelData.line); implicitWidth: 24; implicitHeight: 24; padding: 4; onClicked: panel.editLineRequested(segment.modelData.line) }
                            Label { text: qsTr("Line %1").arg(segment.modelData.line); color: Theme.muted; font.pixelSize: 10 }
                        }
                        TextEdit {
                            objectName: "segmentText-" + segment.index
                            Layout.fillWidth: true
                            Layout.preferredHeight: contentHeight
                            text: segment.modelData.text || ""
                            textFormat: TextEdit.PlainText
                            readOnly: true
                            selectByMouse: true
                            wrapMode: TextEdit.Wrap
                            color: Theme.text
                            selectionColor: Theme.selected
                            selectedTextColor: Theme.text
                            font.pixelSize: 15
                            Accessible.name: qsTr("Source passage")
                        }
                        Label {
                            objectName: "segmentSpoken-" + segment.index
                            visible: segment.modelData.spoken_text !== segment.modelData.text
                            Layout.fillWidth: true
                            text: qsTr("Spoken: %1").arg(segment.modelData.spoken_text || "")
                            textFormat: Text.PlainText
                            wrapMode: Text.WordWrap
                            font.pixelSize: 12
                            color: Theme.secondary
                        }
                        RowLayout {
                            Layout.fillWidth: true
                            spacing: 6
                            CompactComboBox {
                                objectName: "segmentAlternative-" + segment.index
                                Layout.fillWidth: true
                                Layout.minimumWidth: 0
                                visible: segment.alternatives.length > 0
                                model: segment.alternatives.map((candidate, i) => qsTr("Take %1 · %2 s%3").arg(i + 1).arg((candidate.duration_millis / 1000).toFixed(1)).arg(candidate.id === segment.modelData.accepted_id ? " · " + qsTr("In use") : ""))
                                selectionIndex: segment.selectedIndex
                                enabled: !backend.busy
                                onActivated: segment.selectedId = segment.alternatives[currentIndex].id
                            }
                            Item { visible: !segment.alternatives.length; Layout.fillWidth: true }
                            IconButton { objectName: "segmentPlay-" + segment.index; symbol: "play"; hint: qsTr("Listen to this alternative"); visible: !!segment.selectedCandidate; enabled: !backend.busy; onClicked: backend.playSegment(segment.selectedId) }
                            ActionButton { objectName: "segmentAccept-" + segment.index; text: qsTr("Accept"); symbol: "check"; visible: !!segment.selectedCandidate && segment.selectedId !== segment.modelData.accepted_id; enabled: !backend.busy && panel.current && !!segment.selectedCandidate && !!backend.segmentPlan.adopted && (segment.selectedCandidate.accepted || segment.selectedCandidate.source_revision === backend.segmentPlan.source_revision); onClicked: backend.acceptSegment(segment.selectedId) }
                            ActionButton { objectName: "segmentGenerate-" + segment.index; text: segment.alternatives.length ? qsTr("Regenerate") : qsTr("Generate"); symbol: "waveform"; enabled: !backend.busy && backend.runtimeReady && panel.ready; onClicked: backend.renderSegment(segment.modelData.key) }
                        }
                        Label {
                            visible: !!segment.selectedCandidate && !segment.selectedCandidate.accepted && segment.selectedCandidate.source_revision !== backend.segmentPlan.source_revision
                            Layout.fillWidth: true
                            text: qsTr("This alternative belongs to an earlier script. Regenerate to accept a take for the current source.")
                            font.pixelSize: 11
                            color: Theme.muted
                            wrapMode: Text.WordWrap
                        }
                    }
                }
            }
            Item { Layout.preferredHeight: 2 }
        }
    }
    RowLayout {
        Layout.fillWidth: true
        Label { Layout.fillWidth: true; Layout.minimumWidth: 0; text: qsTr("Earlier accepted takes stay available."); color: Theme.muted; font.pixelSize: 11; wrapMode: Text.WordWrap }
        ActionButton { objectName: "exportSubtitlesButton"; text: qsTr("Subtitles"); symbol: "export"; visible: !!panel.selectedTake && !!panel.selectedTake.segment_assembly; enabled: !backend.busy; onClicked: panel.subtitleRequested() }
        ActionButton { objectName: "assembleSegmentsButton"; text: qsTr("Assemble take"); symbol: "waveform"; primary: true; enabled: !backend.busy && panel.ready && !!backend.segmentPlan.all_accepted; onClicked: backend.assembleSegments() }
    }
}
