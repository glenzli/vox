#include "desktop_backend.hpp"
#include "ui_preferences.hpp"
#include "workflow_smoke.hpp"
#include <QAudioOutput>
#include <QDataStream>
#include <QDeadlineTimer>
#include <QDir>
#include <QFile>
#include <QGuiApplication>
#include <QMediaPlayer>
#include <QQmlApplicationEngine>
#include <QQmlComponent>
#include <QQmlContext>
#include <QTemporaryDir>
#include <QTimer>
#include <functional>
#include <limits>
#include <memory>

namespace {
class PlaybackSmoke final : public QObject {
public:
  PlaybackSmoke(QGuiApplication &app, QQmlApplicationEngine &mainEngine,
                UiPreferences &preferences)
      : QObject(mainEngine.rootObjects().first()), app_(app) {
    engine_.setImportPathList(mainEngine.importPathList());
    engine_.rootContext()->setContextProperty("preferences", &preferences);
    QQmlComponent fixture(&engine_);
    fixture.setData(R"QML(
import QtQuick
import QtMultimedia
import Vox.Desktop
Item {
    id: fixtureRoot
    Component { id: disposableRange; AudioRangePlayback { mediaPlayer: transport } }
    property QtObject transport: QtObject {
        objectName: "delayedTransport"
        property url source
        property int mediaStatus: MediaPlayer.NoMedia
        property bool seekable: false
        property double duration: 0
        property double position: 0
        property double requestedPosition: -1
        property int playbackState: MediaPlayer.StoppedState
        property int playCalls: 0
        property bool playedFromZero: false
        property bool volumeLeaked: false
        property bool nativeClockInitialized: false
        property bool positionEndSent: false
        property bool naturalEndSent: false
        property bool zeroHeadStarted: false
        property QtObject audioOutput: QtObject {
            property real volume: 0.37
            property bool muted: true
        }
        signal errorOccurred(int error, string errorString)
        onSourceChanged: {
            seekable = false
            duration = 0
            position = 0
            nativeClockInitialized = false
            positionEndSent = false
            naturalEndSent = false
            zeroHeadStarted = false
            mediaStatus = source.toString().length ? MediaPlayer.LoadingMedia : MediaPlayer.NoMedia
        }
        function setPosition(ms) {
            requestedPosition = ms
            if (nativeClockInitialized) position = ms
        }
        function play() {
            ++playCalls
            playedFromZero = playedFromZero || (position < range.startMs && audioOutput.volume > 0)
            volumeLeaked = volumeLeaked || (range.phase === "priming" && audioOutput.volume !== 0)
            playbackState = MediaPlayer.PlayingState
            if (!nativeClockInitialized && range.startMs > 0) {
                let playingSource = source.toString()
                Qt.callLater(function() {
                    if (transport.source.toString() !== playingSource || transport.playbackState !== MediaPlayer.PlayingState) return
                    transport.volumeLeaked = transport.volumeLeaked || transport.audioOutput.volume !== 0
                    transport.nativeClockInitialized = true
                    // A native backend can replace the optimistic seek with
                    // its first real clock update from the recording's head.
                    if (playingSource.endsWith("fractional.wav")) transport.mediaStatus = MediaPlayer.EndOfMedia
                    else transport.position = 0
                })
            }
            if (range.phase === "playing") {
                mediaStatus = MediaPlayer.BufferedMedia
                let finalSource = source.toString()
                if (range.startMs === 0)
                    zeroHeadStarted = !range.holdingVolume && position === 0 && audioOutput.volume === 0.37 && !audioOutput.muted
                if (finalSource.endsWith("second.wav") || finalSource.endsWith("fractional.wav") || finalSource.endsWith("short.wav")) {
                    Qt.callLater(function() {
                        if (transport.source.toString() !== finalSource || transport.playbackState !== MediaPlayer.PlayingState) return
                        if (finalSource.endsWith("second.wav")) {
                            transport.positionEndSent = true
                            transport.position = 1700
                        } else {
                            transport.naturalEndSent = true
                            transport.mediaStatus = MediaPlayer.EndOfMedia
                        }
                    })
                }
            }
        }
        function pause() { playbackState = MediaPlayer.PausedState }
        function stop() { playbackState = MediaPlayer.StoppedState; position = 0 }
        function loaded() { duration = 3000; mediaStatus = MediaPlayer.LoadedMedia }
    }
    AudioRangePlayback { id: range; objectName: "delayedRange"; mediaPlayer: transport }
    function delayedSource() {
        range.request("file:///first.wav", 800, 1200)
        if (transport.playCalls !== 0) return false
        transport.loaded()
        if (transport.playCalls !== 0) return false
        transport.seekable = true
        if (transport.requestedPosition !== 800 || transport.playCalls !== 0) return false
        transport.position = 800
        // Supersede the queued post-seek start before returning to the loop.
        range.request("file:///second.wav", 1500, 1700)
        return transport.playCalls === 0
    }
    function newerSourceStillWaiting() {
        return transport.playCalls === 0 && range.phase === "loading"
    }
    function positionNewSource() {
        transport.loaded()
        transport.seekable = true
        if (transport.playCalls !== 0 || transport.requestedPosition !== 1500) return false
        transport.position = 1500
        return transport.playCalls === 0
    }
    function finishNewSource() {
        if (transport.playCalls !== 2 || transport.playedFromZero || transport.volumeLeaked
                || !transport.positionEndSent || transport.audioOutput.volume !== 0.37 || !transport.audioOutput.muted) return false
        return !range.active && transport.playbackState === MediaPlayer.StoppedState
    }
    function finiteBounds() {
        return !range.request("file:///invalid.wav", NaN, 1000)
            && !range.request("file:///invalid.wav", 0, Infinity)
            && !range.request("file:///invalid.wav", -1, 1000)
            && !range.request("file:///invalid.wav", 1000, 1000)
            && !range.request("file:///invalid.wav", 0.5, 1000)
            && !range.active && transport.audioOutput.volume === 0.37 && transport.audioOutput.muted
    }
    function startDeadline() {
        range.request("file:///deadline.wav", 800, 900)
        transport.loaded()
        transport.seekable = true
        transport.position = 800
        return true
    }
    function deadlineStopped() {
        return !range.active && transport.playCalls === 4
            && transport.playbackState === MediaPlayer.StoppedState && !transport.playedFromZero
            && !transport.volumeLeaked && transport.audioOutput.volume === 0.37 && transport.audioOutput.muted
    }
    function startRoundedLastCue() {
        range.request("file:///fractional.wav", 900, 1001)
        transport.loaded()
        transport.duration = 1000
        transport.seekable = true
        transport.position = 900
        return range.active && transport.playCalls === 4
    }
    function finishRoundedLastCue() {
        return transport.playCalls === 6 && transport.naturalEndSent
            && !range.active && transport.playbackState === MediaPlayer.StoppedState
    }
    function startZeroCue() {
        transport.audioOutput.muted = false
        range.request("file:///short.wav", 0, 1)
        transport.loaded()
        transport.duration = 1
        transport.seekable = true
        return range.active && transport.playCalls === 6
    }
    function finishZeroCue() {
        if (transport.playCalls !== 7 || !transport.zeroHeadStarted || range.holdingVolume
                || transport.audioOutput.volume !== 0.37 || transport.audioOutput.muted) return false
        return transport.naturalEndSent && !range.active && transport.playbackState === MediaPlayer.StoppedState
            && transport.audioOutput.volume === 0.37 && !transport.audioOutput.muted && !transport.volumeLeaked
    }
    function restorationPaths() {
        range.request("file:///cancel.wav", 100, 200)
        if (transport.audioOutput.volume !== 0) return false
        range.stop()
        if (transport.audioOutput.volume !== 0.37 || transport.audioOutput.muted) return false
        range.request("file:///error.wav", 100, 200)
        transport.errorOccurred(1, "injected media error")
        if (range.active || transport.audioOutput.volume !== 0.37 || transport.audioOutput.muted) return false
        range.request("file:///old-source.wav", 100, 200)
        transport.source = "file:///new-source.wav"
        if (range.active || transport.audioOutput.volume !== 0.37 || transport.audioOutput.muted) return false
        let disposable = disposableRange.createObject(fixtureRoot)
        disposable.request("file:///destroy.wav", 100, 200)
        if (transport.audioOutput.volume !== 0) return false
        disposable.destroy()
        return true
    }
    function destructionRestored() {
        return transport.audioOutput.volume === 0.37 && !transport.audioOutput.muted
            && transport.playbackState === MediaPlayer.StoppedState && transport.playCalls === 7
    }
}
)QML",
                    QUrl("qrc:/playback-transport-fixture.qml"));
    delayed_.reset(fixture.create());
    if (!delayed_) {
      qCritical() << fixture.errors();
      fail("Could not load the packaged range-playback helper");
      return;
    }
    transport_ = delayed_->findChild<QObject *>("delayedTransport");
    delayedRange_ = delayed_->findChild<QObject *>("delayedRange");
    fakeOutput_ = transport_
                      ? transport_->property("audioOutput").value<QObject *>()
                      : nullptr;
    if (!transport_ || !delayedRange_ || !fakeOutput_) {
      fail("Could not inspect the delayed transport fixture");
      return;
    }
    if (!invoke("delayedSource"))
      return;
    QTimer::singleShot(0, this, [this] {
      if (!invoke("newerSourceStillWaiting") || !invoke("positionNewSource"))
        return;
      waitFor(
          [this] {
            return transport_->property("playCalls").toInt() >= 2 &&
                   !delayedRange_->property("active").toBool();
          },
          [this] {
            if (!invoke("finishNewSource") || !invoke("finiteBounds") ||
                !invoke("startDeadline"))
              return;
            waitFor(
                [this] { return !delayedRange_->property("active").toBool(); },
                [this] {
                  if (!invoke("deadlineStopped") ||
                      !invoke("startRoundedLastCue"))
                    return;
                  waitFor(
                      [this] {
                        return transport_->property("playCalls").toInt() >= 6 &&
                               !delayedRange_->property("active").toBool();
                      },
                      [this] {
                        if (!invoke("finishRoundedLastCue") ||
                            !invoke("startZeroCue"))
                          return;
                        waitFor(
                            [this] {
                              return transport_->property("playCalls")
                                             .toInt() >= 7 &&
                                     !delayedRange_->property("active")
                                          .toBool();
                            },
                            [this] {
                              if (!invoke("finishZeroCue") ||
                                  !invoke("restorationPaths"))
                                return;
                              waitFor(
                                  [this] {
                                    return fakeOutput_->property("volume")
                                               .toDouble() == 0.37;
                                  },
                                  [this] {
                                    if (invoke("destructionRestored"))
                                      startBackend();
                                  },
                                  "Destroying a pending range did not restore "
                                  "its volume");
                            },
                            "A zero-start cue did not play directly from its "
                            "head");
                      },
                      "Rounded final cue never reached its positioned start");
                },
                "The range deadline did not stop the delayed transport");
          },
          "The delayed transport never reached its positioned start");
    });
    QTimer::singleShot(20000, this,
                       [this] { fail("Playback smoke deadline exceeded"); });
  }

private:
  bool invoke(const char *method) {
    QVariant result;
    if (!QMetaObject::invokeMethod(delayed_.get(), method,
                                   Q_RETURN_ARG(QVariant, result)) ||
        !result.toBool()) {
      qCritical() << "Range transport assertion failed:" << method;
      fail("Delayed source, seek, stale start, bounds or deadline failed");
      return false;
    }
    return true;
  }
  void fail(const char *message) {
    if (failed_)
      return;
    failed_ = true;
    qCritical() << message;
    qCritical() << "Playback diagnostics" << "platform"
                << QGuiApplication::platformName() << "test phase" << phase_
                << "range started" << rangeStarted_ << "advanced"
                << rangeAdvanced_ << "dispatches" << rangeDispatches_
                << "maximum position" << maximumPosition_;
    if (transport_ && delayedRange_)
      qCritical() << "Delayed transport" << "phase"
                  << delayedRange_->property("phase") << "active"
                  << delayedRange_->property("active") << "play calls"
                  << transport_->property("playCalls") << "position"
                  << transport_->property("position") << "requested position"
                  << transport_->property("requestedPosition");
    if (player_)
      qCritical() << "Actual player" << "status" << player_->mediaStatus()
                  << "state" << player_->playbackState() << "position"
                  << player_->position() << "duration" << player_->duration()
                  << "seekable" << player_->isSeekable() << "source"
                  << player_->source() << "error" << player_->errorString()
                  << "range phase"
                  << (range_ ? range_->property("phase") : QVariant{})
                  << "range active" << backend_->rangePlaybackActive();
    app_.exit(35);
  }
  void waitFor(std::function<bool()> condition, std::function<void()> next,
               const char *message,
               QDeadlineTimer deadline = QDeadlineTimer(3000)) {
    if (failed_)
      return;
    if (condition()) {
      next();
      return;
    }
    if (deadline.hasExpired()) {
      fail(message);
      return;
    }
    QTimer::singleShot(20, this, [this, condition, next, message, deadline] {
      waitFor(condition, next, message, deadline);
    });
  }
  void afterIdle(std::function<void()> next) {
    QTimer::singleShot(20, this, [this, next = std::move(next)] {
      if (failed_)
        return;
      if (backend_->pending())
        afterIdle(next);
      else
        next();
    });
  }
  bool writeWav() {
    QFile wav(directory_.filePath("fixture.wav"));
    if (!wav.open(QIODevice::WriteOnly))
      return false;
    QDataStream stream(&wav);
    stream.setByteOrder(QDataStream::LittleEndian);
    stream.writeRawData("RIFF", 4);
    stream << quint32(36 + 72000 * 2);
    stream.writeRawData("WAVEfmt ", 8);
    stream << quint32(16) << quint16(1) << quint16(1) << quint32(24000)
           << quint32(48000) << quint16(2) << quint16(16);
    stream.writeRawData("data", 4);
    stream << quint32(72000 * 2);
    for (int frame = 0; frame < 72000; ++frame)
      stream << qint16(frame % 48 < 24 ? 1200 : -1200);
    return stream.status() == QDataStream::Ok;
  }
  void startBackend() {
    if (!writeWav()) {
      fail("Could not create the isolated playback WAV");
      return;
    }
    QFile worker(directory_.filePath("worker.py"));
    if (!worker.open(QIODevice::WriteOnly) ||
        worker.write(R"PY(#!/usr/bin/python3
import json, os, sys, time
project = None
path = ''
for line in sys.stdin:
    request = json.loads(line)
    action = request['action']
    result = {}
    if action == 'create':
        path = request['path']
        project = {'id':path,'version':0,'draft':{'text':'','format':'plain'},
                   'candidates':[],'audio_head':'first','audio':[
                       {'id':name,'origin':'echo_file_return','contract':{
                           'frame_count':72000,'sample_rate_hz':24000,'channels':1}}
                       for name in ['first','second']]}
        project['audio'].append({'id':'fractional','origin':'echo_file_return',
            'contract':{'frame_count':24013,'sample_rate_hz':24000,'channels':1}})
        result = {'path':path,'project':project,'analysis':{},'segment_plan':{}}
    elif action == 'playback':
        time.sleep(0.12)
        result = {'file':os.path.join(os.path.dirname(__file__),'fixture.wav'),'peaks':[0.2,0.4]}
    print(json.dumps({'request_id':request['request_id'],'ok':True,'result':result}),flush=True)
)PY") < 0) {
      fail("Could not create delayed playback protocol fixture");
      return;
    }
    worker.close();
    worker.setPermissions(QFileDevice::ReadOwner | QFileDevice::WriteOwner |
                          QFileDevice::ExeOwner);
    const auto original = qgetenv("VOX_CLI_BINARY");
    qputenv("VOX_CLI_BINARY", worker.fileName().toUtf8());
    backend_ = std::make_unique<DesktopBackend>();
    if (original.isNull())
      qunsetenv("VOX_CLI_BINARY");
    else
      qputenv("VOX_CLI_BINARY", original);
    engine_.rootContext()->setContextProperty("backend", backend_.get());
    QQmlComponent panel(&engine_);
    panel.loadFromModule("Vox.Desktop", "AudioPanel");
    panel_.reset(panel.create());
    if (!panel_) {
      qCritical() << panel.errors();
      fail("Could not load the packaged AudioPanel");
      return;
    }
    player_ = panel_->findChild<QMediaPlayer *>("audioPlayer");
    range_ = panel_->findChild<QObject *>("audioRangePlayback");
    if (!player_ || !player_->audioOutput() ||
        panel_->findChildren<QMediaPlayer *>().size() != 1) {
      fail("AudioPanel must retain exactly one audio player");
      return;
    }
    player_->audioOutput()->setMuted(true);
    player_->audioOutput()->setVolume(0.37);
    connect(player_, &QMediaPlayer::errorOccurred, this,
            [this](QMediaPlayer::Error, const QString &error) {
              qCritical() << error;
              fail("Packaged WAV range playback failed");
            });
    connect(backend_.get(), &DesktopBackend::playRangeFile, this,
            [this](const QUrl &, qint64, qint64) { ++rangeDispatches_; });
    connect(backend_.get(), &DesktopBackend::requestFinished, this,
            [this](const QString &action, bool ok) {
              if (action == "create" && ok && projectReady_) {
                auto next = std::move(projectReady_);
                projectReady_ = {};
                next();
              }
            });
    connect(
        player_, &QMediaPlayer::playbackStateChanged, this,
        [this](QMediaPlayer::PlaybackState state) {
          if (state != QMediaPlayer::PlayingState)
            return;
          if (phase_ == "range") {
            if (range_->property("phase").toString() == "priming") {
              if (player_->audioOutput()->volume() != 0 ||
                  !player_->audioOutput()->isMuted())
                fail("Native startup released audio before the final range "
                     "seek");
              else
                primedSilently_ = true;
              return;
            }
            if (range_->property("phase").toString() != "playing" ||
                player_->position() != 800 || !player_->isSeekable() ||
                !qFuzzyCompare(player_->audioOutput()->volume(), float(0.37)) ||
                !player_->audioOutput()->isMuted()) {
              fail("Range began before its source was seekable and positioned");
              return;
            }
            rangeStarted_ = true;
          } else if (phase_ != "whole") {
            fail("A cancelled or stale range began playback");
          }
        });
    connect(player_, &QMediaPlayer::positionChanged, this, [this](qint64 pos) {
      if (phase_ == "range") {
        maximumPosition_ = qMax(maximumPosition_, pos);
        if (pos > 800 && range_->property("phase").toString() == "playing")
          rangeAdvanced_ = true;
      }
    });
    connect(player_, &QMediaPlayer::mediaStatusChanged, this,
            [this](QMediaPlayer::MediaStatus status) {
              if (phase_ == "range")
                qInfo() << "Range WAV status" << status << "position"
                        << player_->position() << "duration"
                        << player_->duration() << "seekable"
                        << player_->isSeekable() << "range phase"
                        << (range_ ? range_->property("phase") : QVariant{});
            });
    connect(backend_.get(), &DesktopBackend::rangePlaybackChanged, this,
            [this] {
              if (phase_ == "range" && !backend_->rangePlaybackActive()) {
                phase_ = "range-ended";
                QTimer::singleShot(0, this, [this] { checkRangeEnded(); });
              }
            });
    projectReady_ = [this] { boundsAndSelection(); };
    backend_->createProject(
        QUrl::fromLocalFile(directory_.filePath("first.vox")));
  }
  void boundsAndSelection() {
    auto &b = *backend_;
    if (b.playAudioRange("first", -1, 1000) ||
        b.playAudioRange("first", 1000, 1000) ||
        b.playAudioRange("first", 1001, 1000) ||
        b.playAudioRange("first", 0, 3001) ||
        b.playAudioRange("first", 0, std::numeric_limits<qint64>::max()) ||
        b.playAudioRange("second", 800, 1600) ||
        b.playAudioRange("missing", 800, 1600) ||
        !b.playAudioRange("first", 0, 3000) ||
        !b.playAudioRange("first", 800, 1600) || !b.rangePlaybackActive() ||
        rangeDispatches_) {
      fail("Range admission ignored exact duration, selected audio or bounds");
      return;
    }
    b.selectAudio("fractional");
    if (b.playAudioRange("fractional", 0, 1002) ||
        !b.playAudioRange("fractional", 0, 1001)) {
      fail("Range admission disagreed with captions' rounded PCM duration");
      return;
    }
    b.selectAudio("second");
    if (b.rangePlaybackActive()) {
      fail("Changing audio did not cancel a pending range");
      return;
    }
    afterIdle([this] {
      if (rangeDispatches_ ||
          player_->playbackState() != QMediaPlayer::StoppedState) {
        fail("A delayed range escaped the audio-selection fence");
        return;
      }
      backend_->selectAudio("first");
      backend_->playAudioRange("first", 800, 1600);
      backend_->stopRangePlayback();
      afterIdle([this] { projectFence(); });
    });
  }
  void projectFence() {
    if (rangeDispatches_ || backend_->rangePlaybackActive()) {
      fail("Explicit cancellation left a delayed range request alive");
      return;
    }
    backend_->selectAudio("second");
    backend_->playAudioRange("second", 800, 1600);
    projectReady_ = [this] {
      afterIdle([this] {
        if (rangeDispatches_ || backend_->rangePlaybackActive()) {
          fail("A delayed range escaped the project-transition fence");
          return;
        }
        if (QGuiApplication::platformName() == "offscreen") {
          verifyContextReset(false);
          return;
        }
        phase_ = "range";
        if (!backend_->playAudioRange("first", 800, 1600))
          fail("Prepared verified playback did not admit a valid range");
      });
    };
    backend_->createProject(
        QUrl::fromLocalFile(directory_.filePath("second.vox")));
  }
  void checkRangeEnded() {
    if (failed_)
      return;
    if (!rangeStarted_ || !rangeAdvanced_ || !primedSilently_ ||
        rangeDispatches_ != 1 ||
        player_->playbackState() != QMediaPlayer::StoppedState ||
        backend_->rangePlaybackActive() ||
        !qFuzzyCompare(player_->audioOutput()->volume(), float(0.37)) ||
        !player_->audioOutput()->isMuted()) {
      fail("The actual WAV did not advance and stop at its bounded range");
      return;
    }
    phase_ = "whole";
    panel_->setProperty("loopPreview", true);
    backend_->playSelected();
    waitFor(
        [this] {
          return player_->playbackState() == QMediaPlayer::PlayingState &&
                 player_->position() > 0;
        },
        [this] {
          if (backend_->rangePlaybackActive() ||
              player_->loops() != QMediaPlayer::Infinite) {
            fail("Whole playback or ordinary sound looping retained range "
                 "state");
            return;
          }
          verifyContextReset(true);
        },
        "Whole WAV playback did not start and advance");
  }
  void verifyContextReset(bool nativeVerified) {
    player_->stop();
    phase_ = "context";
    if (!backend_->playAudioRange("first", 1800, 2200)) {
      fail("Context reset did not admit its valid pending range");
      return;
    }
    panel_->setProperty("playbackContext", "different-workspace");
    waitFor(
        [this] {
          return !backend_->rangePlaybackActive() &&
                 player_->playbackState() == QMediaPlayer::StoppedState &&
                 player_->source().isEmpty() &&
                 player_->mediaStatus() == QMediaPlayer::NoMedia &&
                 qFuzzyCompare(player_->audioOutput()->volume(), float(0.37)) &&
                 player_->audioOutput()->isMuted();
        },
        [this, nativeVerified] {
          if (nativeVerified)
            qInfo() << "Range playback passed delayed load/seek, stale queued "
                       "start, finite bounds, deadline, rounded PCM duration, "
                       "selection/project fences, actual WAV advancement, stop "
                       "and ordinary loop reset";
          else
            qInfo()
                << "Offscreen playback contract passed delayed load/seek, "
                   "stale queued start, finite bounds, deadline, rounded PCM "
                   "duration, selection/project fences and context reset. "
                   "Actual WAV decode, advancement, range stop and ordinary "
                   "loop checks were not run; Cocoa --smoke-playback is "
                   "required separately.";
          app_.quit();
        },
        "A pending range leaked across the playback-context change");
  }
  QGuiApplication &app_;
  QTemporaryDir directory_;
  QQmlEngine engine_;
  std::unique_ptr<DesktopBackend> backend_;
  std::unique_ptr<QObject> delayed_, panel_;
  QMediaPlayer *player_ = nullptr;
  QObject *transport_ = nullptr, *delayedRange_ = nullptr, *range_ = nullptr,
          *fakeOutput_ = nullptr;
  std::function<void()> projectReady_;
  QString phase_ = "identity";
  int rangeDispatches_ = 0;
  qint64 maximumPosition_ = 0;
  bool rangeStarted_ = false, rangeAdvanced_ = false, primedSilently_ = false,
       failed_ = false;
};
} // namespace

void startPlaybackSmoke(QGuiApplication &app, QQmlApplicationEngine &engine,
                        UiPreferences &preferences, DesktopBackend &,
                        const QString &) {
  new PlaybackSmoke(app, engine, preferences);
}
