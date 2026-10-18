import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Dialogs as Native
Dialog {
    id: dialog
    title: qsTr("Settings")
    modal: true
    width: 510
    padding: 24
    standardButtons: Dialog.Close
    background: Rectangle { color: Theme.panel; radius: 12; border.color: Theme.border }
    ColumnLayout {
        anchors.fill: parent
        spacing: 18
        Label { text: qsTr("Appearance"); font.weight: Font.DemiBold }
        RowLayout {
            Label { text: qsTr("Theme"); Layout.fillWidth: true }
            ComboBox {
                objectName: "themeChoice"
                model: [qsTr("Follow system"), qsTr("Light"), qsTr("Dark")]
                currentIndex: ["system", "light", "dark"].indexOf(preferences.theme)
                onActivated: preferences.theme = ["system", "light", "dark"][currentIndex]
            }
        }
        RowLayout {
            Label { text: qsTr("Language"); Layout.fillWidth: true }
            ComboBox {
                objectName: "languageChoice"
                model: [qsTr("Follow system"), "English", "简体中文"]
                currentIndex: ["system", "en", "zh_CN"].indexOf(preferences.language)
                onActivated: preferences.language = ["system", "en", "zh_CN"][currentIndex]
            }
        }
        Rectangle { Layout.fillWidth: true; height: 1; color: Theme.border }
        Label { text: qsTr("Voice synthesis"); font.weight: Font.DemiBold }
        Label {
            text: backend.runtimeReady ? qsTr("Infer Runtime is connected") : qsTr("Infer Runtime needs a connection")
            color: Theme.secondary
        }
        Label {
            Layout.fillWidth: true
            wrapMode: Text.WordWrap
            color: Theme.secondary
            text: qsTr("Vox uses its own local authorization. Import the credential issued for Vox in Infer Console.")
        }
        RowLayout {
            ActionButton { text: qsTr("Import credential"); symbol: "import"; onClicked: credentialFile.open() }
            IconButton { symbol: "refresh"; hint: qsTr("Check connection"); onClicked: backend.probeRuntime() }
        }
        Rectangle { Layout.fillWidth: true; height: 1; color: Theme.border }
        Label { text: qsTr("Edit in Echo"); font.weight: Font.DemiBold }
        Label { text: qsTr("Open accepted audio in Echo’s independent editor. Return its WAV export to review a new take."); wrapMode: Text.WordWrap; Layout.fillWidth: true; color: Theme.secondary }
        RowLayout {
            Label { text: backend.echoBinary(); elide: Text.ElideMiddle; Layout.fillWidth: true; color: Theme.secondary; font.pixelSize: 11 }
            IconButton { symbol: "folder"; hint: qsTr("Choose Echo executable"); onClicked: echoFile.open() }
        }
    }
    Native.FileDialog { id: credentialFile; title: qsTr("Choose Vox credential"); onAccepted: backend.installCredential(selectedFile) }
    Native.FileDialog { id: echoFile; title: qsTr("Choose Echo executable"); onAccepted: backend.setEchoBinary(selectedFile) }
}
