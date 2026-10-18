import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Dialogs as Native
import QtMultimedia
Rectangle {
    id: panel
    color: Theme.panel
    property var takes: {
        let items = []
        for (let r of (backend.project.candidates || []).slice().reverse()) items.push({record:r, accepted:false})
        for (let r of (backend.project.audio || []).slice().reverse()) items.push({record:r, accepted:true})
        return items
    }
    property var selected: {
        for (let take of takes) if (take.record.id === backend.selectedAudio) return take
        return null
    }
    function time(ms) {
        let s = Math.floor(ms / 1000)
        return Math.floor(s / 60) + ":" + (s % 60).toString().padStart(2, "0")
    }
    MediaPlayer { id: player; objectName: "audioPlayer"; audioOutput: AudioOutput { id: output; volume: 0.8 } }
    Connections {
        target: backend
        function onPlayFile(file) { player.stop(); player.source = file; player.play() }
        function onSelectionChanged() { player.stop(); player.source = "" }
        function onWaveformChanged() { waveform.requestPaint() }
    }
    Connections { target: Theme; function onDarkChanged() { waveform.requestPaint() } }
    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 18
        spacing: 12
        RowLayout {
            Layout.fillWidth: true
            Label { text: qsTr("Takes"); font.weight: Font.DemiBold; font.pixelSize: 14 }
            Label { text: panel.takes.length; color: Theme.secondary; font.pixelSize: 11 }
            Item { Layout.fillWidth: true }
            ActionButton { text: qsTr("Accept"); symbol: "check"; primary: true; enabled: panel.selected && !panel.selected.accepted && !backend.rendering; onClicked: backend.acceptSelected() }
            IconButton { symbol: "x"; hint: qsTr("Discard take"); enabled: panel.selected && !panel.selected.accepted && !backend.rendering; onClicked: backend.discardSelected() }
            Rectangle { width: 1; height: 20; color: Theme.border }
            IconButton { objectName: "echoEditButton"; symbol: "echo"; hint: qsTr("Edit in Echo"); enabled: panel.selected && panel.selected.accepted && !backend.rendering; onClicked: backend.editInEcho() }
            IconButton { symbol: "import"; hint: qsTr("Return edited WAV from Echo"); enabled: backend.hasEchoHandoff && !backend.rendering; onClicked: echoReturnDialog.open() }
            IconButton { objectName: "exportButton"; symbol: "export"; hint: qsTr("Export WAV"); enabled: panel.selected !== null; onClicked: exportFile.open() }
        }
        ListView {
            id: takeList
            Layout.fillWidth: true
            Layout.preferredHeight: 82
            orientation: ListView.Horizontal
            spacing: 10
            clip: true
            model: panel.takes
            ScrollBar.horizontal: ScrollBar { }
            delegate: Rectangle {
                id: take
                required property var modelData
                required property int index
                width: 218; height: 78; radius: 8
                color: backend.selectedAudio === modelData.record.id ? Theme.selected : Theme.inset
                border.color: backend.selectedAudio === modelData.record.id ? Theme.secondary : Theme.border
                Accessible.role: Accessible.Button
                Accessible.name: (modelData.accepted ? qsTr("Accepted version") : qsTr("Candidate take")) + " " + (index+1)
                MouseArea { anchors.fill: parent; onClicked: backend.selectAudio(take.modelData.record.id) }
                RowLayout {
                    anchors.fill: parent; anchors.margins: 12; spacing: 10
                    IconButton { symbol: "play"; hint: qsTr("Play take"); onClicked: { backend.selectAudio(take.modelData.record.id); backend.playSelected() } }
                    ColumnLayout {
                        Layout.fillWidth: true; spacing: 5
                        Label { text: take.modelData.record.origin === "echo_file_return" ? qsTr("Echo edit") : qsTr("Take %1").arg(take.index + 1); font.weight: Font.Medium; font.pixelSize: 12 }
                        Label { text: (take.modelData.accepted ? qsTr("Accepted") : qsTr("For review")) + " · " + panel.time(take.modelData.record.contract.frame_count * 1000 / take.modelData.record.contract.sample_rate_hz); color: Theme.secondary; font.pixelSize: 11 }
                    }
                }
            }
            Label { anchors.centerIn: parent; visible: panel.takes.length === 0; text: qsTr("Generate a take to listen, compare and keep your preferred version."); color: Theme.muted; font.pixelSize: 12 }
        }
        RowLayout {
            Layout.fillWidth: true
            IconButton {
                objectName: "playButton"
                symbol: player.playbackState === MediaPlayer.PlayingState ? "pause" : "play"
                primary: true
                hint: player.playbackState === MediaPlayer.PlayingState ? qsTr("Pause") : qsTr("Play selected take")
                enabled: panel.selected !== null || player.source.toString().length > 0
                onClicked: { if (player.playbackState === MediaPlayer.PlayingState) player.pause(); else if (player.source.toString().length > 0) player.play(); else backend.playSelected() }
            }
            IconButton { symbol: "stop"; hint: qsTr("Stop playback"); onClicked: player.stop(); enabled: player.source.toString().length > 0 }
            Item {
                Layout.fillWidth: true
                Layout.preferredHeight: 36
                Canvas {
                    id: waveform
                    anchors.fill: parent
                    onWidthChanged: requestPaint()
                    onPaint: {
                        let ctx = getContext("2d"); ctx.reset()
                        let peaks = backend.waveform
                        ctx.fillStyle = Theme.secondary
                        if (peaks.length === 0) { ctx.fillStyle = Theme.border; ctx.fillRect(0, height/2, width, 1); return }
                        let step = width/peaks.length
                        for (let i=0; i<peaks.length; ++i) {
                            let h=Math.max(2,peaks[i]*height)
                            ctx.fillRect(i*step,(height-h)/2,Math.max(1,step-1),h)
                        }
                    }
                }
                Rectangle { height: parent.height; width: 1; color: Theme.text; x: player.duration > 0 ? player.position / player.duration * parent.width : 0; visible: player.duration > 0 }
                MouseArea { anchors.fill: parent; enabled: player.seekable; onClicked: mouse => player.setPosition(Math.round(mouse.x/width*player.duration)) }
            }
            Label { text: panel.time(player.position) + " / " + panel.time(player.duration > 0 ? player.duration : panel.selected ? panel.selected.record.contract.frame_count * 1000 / panel.selected.record.contract.sample_rate_hz : 0); font.pixelSize: 11; color: Theme.secondary; Layout.preferredWidth: 88 }
            IconButton { symbol: "volume"; hint: output.muted ? qsTr("Unmute") : qsTr("Mute"); onClicked: output.muted = !output.muted; opacity: output.muted ? 0.4 : 1 }
        }
    }
    Native.FileDialog { id: exportFile; title: qsTr("Export WAV"); fileMode: Native.FileDialog.SaveFile; defaultSuffix: "wav"; nameFilters: [qsTr("WAV audio (*.wav)")]; onAccepted: backend.exportSelected(selectedFile) }
    Dialog {
        id: echoReturnDialog
        title: qsTr("Return from Echo")
        anchors.centerIn: Overlay.overlay
        width: 440
        modal: true
        standardButtons: Dialog.Open | Dialog.Cancel
        background: Rectangle { color: Theme.panel; radius: 10; border.color: Theme.border }
        ColumnLayout {
            anchors.fill: parent
            Label { text: qsTr("Export your finished edit from Echo as WAV. Vox will add it as a candidate and keep the original take."); Layout.fillWidth: true; wrapMode: Text.WordWrap }
            CheckBox { id: pcm24; text: qsTr("Allow 24-bit to 16-bit PCM conversion"); checked: false }
        }
        onAccepted: returnFile.open()
    }
    Native.FileDialog { id: returnFile; title: qsTr("Choose Echo WAV export"); nameFilters: [qsTr("WAV audio (*.wav)")]; onAccepted: backend.returnFromEcho(selectedFile, pcm24.checked) }
}
