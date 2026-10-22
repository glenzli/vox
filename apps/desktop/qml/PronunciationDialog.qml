pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Dialog {
    id: dialog
    objectName: "pronunciationDialog"
    title: qsTr("Pronunciation dictionary")
    modal: true
    width: Math.min(720, parent.width - 48)
    height: Math.min(690, parent.height - 48)
    anchors.centerIn: parent
    padding: 18
    property var rules: []
    property bool changed: false
    property string projectPath: ""
    property string projectId: ""
    readonly property var languageChoices: backend.languages.filter(language => language.key !== "auto")
    readonly property bool sameProject: projectPath === backend.path && projectId === (backend.project.id || "")
    readonly property bool previewCurrent: sameProject && !changed && !backend.dirty && !!backend.segmentPlan.document_digest
    readonly property string selectedLanguage: languageChoice.currentIndex > 0 ? languageChoices[languageChoice.currentIndex - 1].key : ""
    readonly property string ruleProblem: {
        if (utf8Length(source.text) > 256) return qsTr("Use a shorter written phrase.")
        if (utf8Length(spoken.text) > 1024) return qsTr("Use a shorter pronunciation.")
        if (utf8Length(role.text.trim()) > 128) return qsTr("Use a shorter role name.")
        if (rules.some(rule => rule.text === source.text && (rule.language || "") === selectedLanguage && (rule.role || "") === role.text.trim())) return qsTr("A rule already exists for this text and filters. Remove it before adding a replacement.")
        return ""
    }
    function utf8Length(value) {
        let bytes = 0
        for (let i = 0; i < value.length; ++i) {
            const code = value.charCodeAt(i)
            if (code < 0x80) ++bytes
            else if (code < 0x800) bytes += 2
            else if (code >= 0xd800 && code <= 0xdbff && i + 1 < value.length && value.charCodeAt(i + 1) >= 0xdc00 && value.charCodeAt(i + 1) <= 0xdfff) { bytes += 4; ++i }
            else bytes += 3
        }
        return bytes
    }
    function loadRules() {
        rules = (backend.document.pronunciation_rules || []).map(rule => ({text: rule.text, spoken: rule.spoken, language: rule.language || "", role: rule.role || ""}))
        changed = false
    }
    function addRule() {
        if (!source.text.trim().length || !spoken.text.trim().length) return
        rules = rules.concat([{text: source.text, spoken: spoken.text, language: selectedLanguage, role: role.text.trim()}])
        changed = true
        source.clear()
        spoken.clear()
        source.forceActiveFocus()
    }
    function removeRule(index) {
        rules = rules.filter((_, candidateIndex) => candidateIndex !== index)
        changed = true
    }
    onOpened: {
        projectPath = backend.path
        projectId = backend.project.id || ""
        loadRules()
        source.clear()
        spoken.clear()
        role.clear()
        languageChoice.currentIndex = 0
    }
    background: Rectangle { color: Theme.panel; radius: 10; border.color: Theme.border }
    footer: Item {
        implicitHeight: 52
        RowLayout {
            anchors.fill: parent
            anchors.margins: 12
            Label { Layout.fillWidth: true; Layout.minimumWidth: 0; text: backend.dirty ? qsTr("Saving…") : dialog.changed ? qsTr("Save rules to refresh the preview.") : backend.message; font.pixelSize: 11; color: Theme.secondary; elide: Text.ElideRight }
            ActionButton { text: qsTr("Close"); onClicked: dialog.close() }
            ActionButton { objectName: "savePronunciationButton"; text: qsTr("Save rules"); primary: true; enabled: dialog.sameProject && dialog.changed && !backend.busy; onClicked: { backend.setPronunciationRules(dialog.rules); dialog.changed = false; backend.save() } }
        }
    }
    ColumnLayout {
        anchors.fill: parent
        spacing: 12
        Label {
            Layout.fillWidth: true
            text: qsTr("Rules change the text sent to the voice engine. Your authored script stays unchanged. Match an exact word or phrase, with optional language and role filters.")
            color: Theme.secondary
            font.pixelSize: 12
            wrapMode: Text.WordWrap
        }
        RowLayout {
            Layout.fillWidth: true
            spacing: 8
            CompactTextField { id: source; objectName: "pronunciationSource"; Layout.fillWidth: true; placeholderText: qsTr("Written text"); maximumLength: 256; enabled: !backend.busy }
            Label { text: "→"; color: Theme.secondary }
            CompactTextField { id: spoken; objectName: "pronunciationSpoken"; Layout.fillWidth: true; placeholderText: qsTr("Read as"); maximumLength: 1024; enabled: !backend.busy; onAccepted: if (add.enabled) dialog.addRule() }
        }
        RowLayout {
            Layout.fillWidth: true
            spacing: 8
            CompactComboBox { id: languageChoice; objectName: "pronunciationLanguage"; Layout.fillWidth: true; Layout.minimumWidth: 0; model: [qsTr("All languages")].concat(dialog.languageChoices.map(language => language.label)); enabled: !backend.busy }
            CompactTextField { id: role; objectName: "pronunciationRole"; Layout.fillWidth: true; placeholderText: qsTr("All roles (or enter a role)"); maximumLength: 128; enabled: !backend.busy }
            ActionButton { id: add; objectName: "addPronunciationRule"; text: qsTr("Add rule"); symbol: "add"; enabled: !backend.busy && source.text.trim().length > 0 && spoken.text.trim().length > 0 && !dialog.ruleProblem.length && dialog.rules.length < 128; onClicked: dialog.addRule() }
        }
        Label { visible: !!dialog.ruleProblem; text: dialog.ruleProblem; color: Theme.secondary; font.pixelSize: 11; Layout.fillWidth: true; wrapMode: Text.WordWrap }
        ScrollView {
            id: scroll
            Layout.fillWidth: true
            Layout.fillHeight: true
            contentWidth: availableWidth
            clip: true
            ColumnLayout {
                width: scroll.availableWidth
                spacing: 10
                Label { visible: !dialog.rules.length; text: qsTr("No pronunciation rules yet."); Layout.fillWidth: true; color: Theme.muted; font.pixelSize: 12 }
                Repeater {
                    model: dialog.rules
                    Rectangle {
                        id: rule
                        required property var modelData
                        required property int index
                        Layout.fillWidth: true
                        implicitHeight: ruleContent.implicitHeight + 20
                        radius: 6
                        color: Theme.inset
                        border.color: Theme.border
                        RowLayout {
                            id: ruleContent
                            anchors.fill: parent
                            anchors.margins: 10
                            spacing: 8
                            ColumnLayout {
                                Layout.fillWidth: true
                                Layout.minimumWidth: 0
                                spacing: 4
                                Label { Layout.fillWidth: true; text: rule.modelData.text + " → " + rule.modelData.spoken; textFormat: Text.PlainText; wrapMode: Text.WordWrap; font.pixelSize: 13 }
                                Label { Layout.fillWidth: true; text: (rule.modelData.language ? backend.languageLabel(rule.modelData.language) : qsTr("All languages")) + " · " + (rule.modelData.role || qsTr("All roles")); textFormat: Text.PlainText; wrapMode: Text.WordWrap; font.pixelSize: 11; color: Theme.secondary }
                            }
                            IconButton { objectName: "removePronunciation-" + rule.index; symbol: "x"; hint: qsTr("Remove rule"); enabled: !backend.busy; onClicked: dialog.removeRule(rule.index) }
                        }
                    }
                }
                Rectangle { Layout.fillWidth: true; height: 1; color: Theme.border; Layout.topMargin: 6 }
                Label { text: qsTr("Effective spoken text"); font.pixelSize: 14; font.weight: Font.DemiBold }
                Label { visible: !dialog.previewCurrent; Layout.fillWidth: true; text: dialog.changed ? qsTr("Save rules to refresh the preview.") : qsTr("Checking script…"); color: Theme.secondary; font.pixelSize: 12; wrapMode: Text.WordWrap }
                Repeater {
                    model: dialog.previewCurrent ? (backend.segmentPlan.segments || []) : []
                    ColumnLayout {
                        id: preview
                        required property var modelData
                        required property int index
                        Layout.fillWidth: true
                        spacing: 4
                        Label { text: qsTr("Line %1 · %2").arg(preview.modelData.line).arg(preview.modelData.role || qsTr("Narration")); textFormat: Text.PlainText; font.pixelSize: 10; color: Theme.muted; Layout.fillWidth: true; wrapMode: Text.WordWrap }
                        Label { objectName: "pronunciationPreview-" + preview.index; text: preview.modelData.spoken_text; textFormat: Text.PlainText; font.pixelSize: 13; Layout.fillWidth: true; wrapMode: Text.WordWrap }
                    }
                }
            }
        }
    }
    Connections {
        target: backend
        function onStateChanged() { if (dialog.opened && !dialog.sameProject) dialog.close() }
        function onRequestFinished(action, ok) { if (dialog.opened && action === "save" && !ok && dialog.sameProject) dialog.changed = true }
    }
}
