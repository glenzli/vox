import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
Rectangle {
    id: entry
    required property var block
    signal editLineRequested(int line)
    readonly property bool speech: block.kind === "speech"
    readonly property bool rejected: block.kind === "issue"
    readonly property bool card: speech || rejected
    readonly property var issues: block.issues || []
    readonly property var deliveries: ({clear: qsTr("Clear"), warm: qsTr("Warm"), lively: qsTr("Lively")})
    objectName: "readingEntry-" + block.line
    implicitHeight: content.implicitHeight + (card ? 24 : 8)
    color: card ? Theme.inset : "transparent"
    border.color: card ? Theme.border : "transparent"
    radius: 7
    ColumnLayout {
        id: content
        anchors.fill: parent
        anchors.margins: entry.card ? 12 : 4
        spacing: 8
        RowLayout {
            visible: entry.card
            Layout.fillWidth: true
            spacing: 8
            Rectangle {
                implicitWidth: identity.implicitWidth + 12
                implicitHeight: 20
                radius: 4
                color: Theme.selected
                Label {
                    id: identity
                    anchors.centerIn: parent
                    text: entry.rejected ? (entry.issues.indexOf("missing_speaker") >= 0 ? qsTr("Choose a role") : qsTr("Needs attention")) : (entry.block.role || qsTr("Narration"))
                    textFormat: Text.PlainText
                    font.pixelSize: 11
                    font.weight: Font.DemiBold
                }
            }
            Label {
                visible: entry.speech
                text: { let key = entry.block.language || "auto"; return (backend.languages.find(v => v.key === key) || {}).label || key }
                font.pixelSize: 11
                color: Theme.secondary
            }
            Label { visible: entry.speech && entry.block.delivery !== "neutral"; text: entry.deliveries[entry.block.delivery] || ""; font.pixelSize: 11; color: Theme.secondary }
            Item { Layout.fillWidth: true }
            Label { text: qsTr("Line %1").arg(entry.block.line); font.pixelSize: 10; color: Theme.secondary }
            IconButton {
                objectName: "readingEdit-" + entry.block.line
                symbol: "edit"
                hint: qsTr("Edit line %1").arg(entry.block.line)
                implicitWidth: 24; implicitHeight: 24; padding: 4
                onClicked: entry.editLineRequested(entry.block.line)
            }
        }
        TextEdit {
            objectName: "readingText-" + entry.block.line
            visible: entry.card
            Layout.fillWidth: true
            Layout.preferredHeight: contentHeight
            text: entry.card ? entry.block.text || "" : ""
            textFormat: TextEdit.PlainText
            readOnly: true
            selectByMouse: true
            wrapMode: TextEdit.Wrap
            font.pixelSize: 16
            color: Theme.text
            selectionColor: Theme.selected
            selectedTextColor: Theme.text
            Accessible.name: entry.rejected ? qsTr("Source needing review") : qsTr("Spoken passage")
        }
        RowLayout {
            visible: !entry.card
            Layout.fillWidth: true
            spacing: 5
            IconButton {
                visible: entry.block.kind === "pause" || entry.block.kind === "cue"
                symbol: entry.block.kind === "pause" ? "clock" : "headphones"
                enabled: false
                implicitWidth: 20; implicitHeight: 20; icon.width: 13; icon.height: 13; padding: 3
                Accessible.ignored: true
            }
            Label {
                Layout.fillWidth: true
                wrapMode: Text.WordWrap
                textFormat: Text.PlainText
                color: entry.block.kind === "scene" || entry.block.kind === "heading" ? Theme.text : Theme.secondary
                font.pixelSize: entry.block.kind === "scene" || entry.block.kind === "heading" ? 14 : 11
                font.weight: entry.block.kind === "scene" || entry.block.kind === "heading" ? Font.DemiBold : Font.Normal
                text: {
                    if (entry.block.kind === "pause") return qsTr("Pause %1 s").arg(entry.block.milliseconds / 1000)
                    if (entry.block.kind === "cue") return qsTr("Sound · %1").arg(entry.block.label)
                    if (entry.block.kind === "scene") return qsTr("Scene · %1").arg(entry.block.label)
                    if (entry.block.kind === "note") return qsTr("Note · %1").arg(entry.block.text)
                    return entry.block.text || ""
                }
            }
        }
        Repeater {
            model: entry.issues
            Label {
                required property string modelData
                Layout.fillWidth: true
                wrapMode: Text.WordWrap
                text: qsTr("Needs attention") + " · " + backend.errorText(modelData)
                font.pixelSize: 11
                color: Theme.secondary
            }
        }
    }
}
