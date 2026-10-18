import QtQuick
import QtQuick.Controls

Item {
    id: root
    readonly property alias text: textArea.text
    property bool loadingSource: false
    property string observedText: ""
    signal textEdited(string value)
    function loadText(value) {
        observedText = value
        if (textArea.text === value) return
        loadingSource = true
        textArea.text = value
        loadingSource = false
    }
    function focusLine(line) {
        let position = 0
        for (let current = 1; current < line; ++current) {
            let next = textArea.text.indexOf("\n", position)
            if (next < 0) break
            position = next + 1
        }
        textArea.cursorPosition = position
        textArea.forceActiveFocus()
    }
    property bool productionScript: false
    property bool readOnly: false
    property bool showLineNumbers: true
    required property string proseFontFamily
    readonly property alias lineCount: gutter.lineCount

    ScrollView {
        id: scroll
        objectName: "scriptScrollView"
        anchors.fill: parent
        clip: true
        leftPadding: root.showLineNumbers ? gutter.implicitWidth + 12 : 0
        contentWidth: availableWidth
        ScrollBar.horizontal.policy: ScrollBar.AlwaysOff

        TextArea {
            id: textArea
            objectName: "scriptEditor"
            textFormat: TextEdit.PlainText
            wrapMode: TextEdit.Wrap
            selectByMouse: true
            persistentSelection: true
            readOnly: root.readOnly
            font.family: preferences.editorFontFamily
            font.pixelSize: preferences.editorFontSize
            color: Theme.text
            selectionColor: Theme.selected
            selectedTextColor: Theme.text
            placeholderText: qsTr("Start with your words.\n\nWrite a narration, import an existing script, or choose a form on the left.")
            placeholderTextColor: Theme.muted
            leftPadding: 0
            rightPadding: 8
            topPadding: 10
            bottomPadding: 30
            background: Item { }
            Accessible.name: qsTr("Script editor")
            onTextChanged: {
                // QSyntaxHighlighter also emits TextEdit notifications when
                // only formatting changes. Persist actual text changes only.
                if (textArea.text === root.observedText) return
                root.observedText = textArea.text
                if (!root.loadingSource) root.textEdited(textArea.text)
            }
        }
    }

    ScriptHighlighter {
        objectName: "scriptHighlighter"
        textDocument: textArea.textDocument
        productionScript: root.productionScript
        colors: ({
            text: Theme.text,
            command: Theme.scriptCommand,
            identity: Theme.scriptIdentity,
            secondary: Theme.secondary
        })
    }

    LineNumberGutter {
        id: gutter
        objectName: "lineNumberGutter"
        anchors.left: parent.left
        anchors.top: parent.top
        anchors.topMargin: textArea.topPadding
        anchors.bottom: parent.bottom
        anchors.bottomMargin: textArea.bottomPadding
        width: implicitWidth
        visible: root.showLineNumbers
        clip: true
        sourceDocument: textArea.textDocument
        font: textArea.font
        activePosition: textArea.cursorPosition
        scrollY: scroll.contentItem ? scroll.contentItem.contentY : 0
        textTopPadding: 0
        colors: ({
            text: Theme.text,
            secondary: Theme.secondary,
            background: Theme.panel,
            currentBackground: Theme.inset,
            border: Theme.border
        })
        Accessible.ignored: true
    }
}
