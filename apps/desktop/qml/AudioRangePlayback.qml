import QtQuick
import QtMultimedia

// A range controls AudioPanel's existing player. Cue audio is released only
// after loading, native-clock initialization and the final seek to its start.
Item {
    id: range
    visible: false
    property var mediaPlayer
    readonly property bool active: phase !== "idle"
    property string phase: "idle"
    property url requestedSource
    property double startMs: 0
    property double endMs: 0
    property int generation: 0
    property bool loaded: false
    property bool primed: false
    property var heldOutput
    property real heldVolume: 1
    property bool holdingVolume: false
    signal finished()

    function matchesSource() {
        return mediaPlayer && mediaPlayer.source.toString() === requestedSource.toString()
    }
    function stop() {
        let wasActive = active
        phase = "idle"
        ++generation
        loadDeadline.stop()
        endDeadline.stop()
        if (wasActive && mediaPlayer) mediaPlayer.stop()
        releaseVolume()
    }
    function releaseVolume() {
        if (!holdingVolume) return
        holdingVolume = false
        if (heldOutput) heldOutput.volume = heldVolume
        heldOutput = null
    }
    function complete() {
        if (!active) return
        stop()
        finished()
    }
    function request(file, begin, end) {
        stop()
        if (!mediaPlayer || !mediaPlayer.audioOutput || !isFinite(begin) || !isFinite(end) || begin < 0 || end <= begin
                || Math.floor(begin) !== begin || Math.floor(end) !== end
                || file.toString().length === 0) {
            finished()
            return false
        }
        requestedSource = file
        startMs = begin
        endMs = end
        loaded = false
        primed = begin === 0
        if (!primed) {
            heldOutput = mediaPlayer.audioOutput
            heldVolume = heldOutput.volume
            holdingVolume = true
            heldOutput.volume = 0
        }
        phase = "setting"
        mediaPlayer.stop()
        // Clearing also makes a second cue on the same file wait for this load,
        // instead of consuming readiness left over from the preceding request.
        mediaPlayer.source = ""
        phase = "loading"
        mediaPlayer.source = file
        loadDeadline.start()
        return true
    }
    function ready() {
        return matchesSource() && loaded && mediaPlayer.seekable
                && (mediaPlayer.mediaStatus === MediaPlayer.LoadedMedia
                    || mediaPlayer.mediaStatus === MediaPlayer.BufferedMedia
                    || (primed && mediaPlayer.mediaStatus === MediaPlayer.EndOfMedia))
    }
    function prepare() {
        if (!active || !ready()) return
        if (mediaPlayer.duration <= 0 && endMs > 1) return
        // Captions round the PCM duration to the nearest millisecond; a media
        // backend may truncate it. EndOfMedia stops the final cue naturally.
        if (mediaPlayer.duration + 1 < endMs) {
            complete()
            return
        }
        if (phase === "loading" || phase === "repositioning") {
            phase = "seeking"
            mediaPlayer.setPosition(startMs)
        }
        if (phase !== "seeking" || mediaPlayer.position !== startMs) return
        phase = "ready"
        let captured = generation
        Qt.callLater(function() {
            if (!range.active || captured !== range.generation || range.phase !== "ready") return
            if (!range.ready()) {
                range.phase = "seeking"
                range.prepare()
                return
            }
            if (range.primed && range.mediaPlayer.position !== range.startMs) {
                range.phase = "repositioning"
                range.prepare()
                return
            }
            if (range.primed) {
                range.phase = "playing"
                range.releaseVolume()
                loadDeadline.stop()
            } else {
                // Native media can reset an optimistic pre-play seek to zero
                // on its first clock update. Prime silently, then seek again
                // on the initialized clock before releasing any cue audio.
                range.phase = "priming"
            }
            range.mediaPlayer.play()
        })
    }
    function nativeClockReady() {
        if (phase !== "priming") return
        phase = "initializing"
        let captured = generation
        Qt.callLater(function() {
            if (!range.active || captured !== range.generation || range.phase !== "initializing") return
            if (!range.matchesSource() || !range.mediaPlayer.seekable) { range.complete(); return }
            range.mediaPlayer.pause()
            range.primed = true
            range.phase = "repositioning"
            range.prepare()
        })
    }
    function positionChanged() {
        if (!active || !matchesSource()) return
        if (phase === "priming") {
            if (mediaPlayer.playbackState === MediaPlayer.PlayingState && mediaPlayer.position !== startMs) nativeClockReady()
            return
        }
        if (phase !== "playing") {
            prepare()
            return
        }
        if (mediaPlayer.position < startMs || mediaPlayer.position >= endMs) complete()
        else armEnd()
    }
    function armEnd() {
        if (phase !== "playing" || mediaPlayer.playbackState !== MediaPlayer.PlayingState) return
        endDeadline.interval = Math.max(1, Math.ceil(endMs - mediaPlayer.position))
        endDeadline.restart()
    }
    Connections {
        target: range.mediaPlayer
        function onSourceChanged() {
            if (range.active && range.phase !== "setting" && !range.matchesSource()) range.complete()
        }
        function onMediaStatusChanged() {
            if (!range.active || !range.matchesSource()) return
            if (range.phase === "priming" && range.mediaPlayer.mediaStatus === MediaPlayer.EndOfMedia) {
                range.nativeClockReady()
                return
            }
            if (range.mediaPlayer.mediaStatus === MediaPlayer.InvalidMedia
                    || (range.phase === "playing" && range.mediaPlayer.mediaStatus === MediaPlayer.EndOfMedia)) {
                range.complete()
                return
            }
            if (range.mediaPlayer.mediaStatus === MediaPlayer.LoadedMedia
                    || range.mediaPlayer.mediaStatus === MediaPlayer.BufferedMedia) range.loaded = true
            range.prepare()
        }
        function onSeekableChanged() { range.prepare() }
        function onDurationChanged() { range.prepare() }
        function onPositionChanged() { range.positionChanged() }
        function onPlaybackStateChanged() {
            if (range.mediaPlayer.playbackState === MediaPlayer.PlayingState) range.armEnd()
            else endDeadline.stop()
        }
        function onErrorOccurred() { range.complete() }
    }
    Timer { id: loadDeadline; interval: 10000; onTriggered: range.complete() }
    Timer { id: endDeadline; onTriggered: range.complete() }
    Component.onDestruction: stop()
}
