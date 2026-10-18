import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
ScrollView {
    id: reader
    objectName: "scriptReader"
    signal editLineRequested(int line)
    readonly property bool production: backend.document.format === "production_script"
    readonly property var preview: backend.analysis.reading || {}
    readonly property bool current: !production || preview.source_text === (backend.document.text || "")
    readonly property var blocks: production ? (current ? preview.blocks || [] : []) : (backend.document.text || "").trim().length ? [{kind:"speech", line:1, text:backend.document.text, role:"", language:backend.document.language || "auto", delivery:"neutral"}] : []
    readonly property var forms: ({narration: qsTr("Narration"), dialogue: qsTr("Dialogue"), listening: qsTr("Listening")})
    clip: true
    contentWidth: availableWidth
    ScrollBar.horizontal.policy: ScrollBar.AlwaysOff
    ColumnLayout {
        width: reader.availableWidth
        spacing: 10
        RowLayout {
            Layout.fillWidth: true
            Layout.bottomMargin: 2
            Label { text: reader.forms[reader.preview.production || "narration"]; font.pixelSize: 12; font.weight: Font.DemiBold }
            Label { text: reader.production ? qsTr("Passages %1 · Roles %2").arg(backend.analysis.spoken_passages || 0).arg((backend.analysis.roles || []).length) : qsTr("Passages %1").arg(backend.analysis.spoken_passages || 0); color: Theme.secondary; font.pixelSize: 11 }
            Item { Layout.fillWidth: true }
            Label { text: qsTr("Reading preview"); color: Theme.secondary; font.pixelSize: 10 }
        }
        Label { objectName: "readingEmptyState"; visible: !reader.blocks.length; Layout.fillWidth: true; text: reader.current ? qsTr("Your spoken passages will appear here as you write.") : qsTr("Updating preview…"); wrapMode: Text.WordWrap; color: Theme.secondary; font.pixelSize: 12 }
        Repeater {
            model: reader.blocks
            Loader {
                id: blockLoader
                required property var modelData
                Layout.fillWidth: true
                sourceComponent: modelData.kind === "repeat" ? repeatGroup : leaf
                Component {
                    id: leaf
                    ReadingEntry { block: blockLoader.modelData; onEditLineRequested: line => reader.editLineRequested(line) }
                }
                Component {
                    id: repeatGroup
                    Rectangle {
                        objectName: "readingRepeat-" + blockLoader.modelData.line
                        implicitHeight: repeated.implicitHeight + 24
                        color: "transparent"
                        border.color: Theme.border
                        radius: 8
                        ColumnLayout {
                            id: repeated
                            anchors.fill: parent
                            anchors.margins: 12
                            spacing: 8
                            RowLayout {
                                Layout.fillWidth: true
                                spacing: 6
                                IconButton { symbol: "refresh"; enabled: false; implicitWidth: 20; implicitHeight: 20; icon.width: 14; icon.height: 14; padding: 3; Accessible.ignored: true }
                                Label { objectName: "readingRepeatTitle-" + blockLoader.modelData.line; text: qsTr("Repeat %1×").arg(blockLoader.modelData.count); font.pixelSize: 12; font.weight: Font.DemiBold }
                                Label { text: qsTr("Same recording"); color: Theme.secondary; font.pixelSize: 10 }
                                Item { Layout.fillWidth: true }
                                IconButton { objectName: "readingEdit-" + blockLoader.modelData.line; symbol: "edit"; hint: qsTr("Edit line %1").arg(blockLoader.modelData.line); implicitWidth: 24; implicitHeight: 24; padding: 4; onClicked: reader.editLineRequested(blockLoader.modelData.line) }
                            }
                            Label {
                                Layout.fillWidth: true
                                wrapMode: Text.WordWrap
                                textFormat: Text.PlainText
                                text: qsTr("%1 s between repeats").arg(blockLoader.modelData.gap_ms / 1000) + (blockLoader.modelData.between_cue ? " · " + qsTr("Sound · %1").arg(blockLoader.modelData.between_cue) : "")
                                color: Theme.secondary
                                font.pixelSize: 11
                            }
                            Repeater {
                                model: blockLoader.modelData.blocks || []
                                ReadingEntry {
                                    required property var modelData
                                    Layout.fillWidth: true
                                    block: modelData
                                    onEditLineRequested: line => reader.editLineRequested(line)
                                }
                            }
                            Repeater {
                                model: blockLoader.modelData.issues || []
                                Label { required property string modelData; text: qsTr("Needs attention") + " · " + backend.errorText(modelData); Layout.fillWidth: true; wrapMode: Text.WordWrap; font.pixelSize: 11; color: Theme.secondary }
                            }
                        }
                    }
                }
            }
        }
        Item { Layout.preferredHeight: 10 }
    }
}
