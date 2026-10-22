#include "workflow_smoke.hpp"
#include "desktop_backend.hpp"
#include "ui_preferences.hpp"
#include <QAudioOutput>
#include <QDebug>
#include <QFile>
#include <QFileInfo>
#include <QGuiApplication>
#include <QJsonDocument>
#include <QMediaPlayer>
#include <QQmlApplicationEngine>
#include <QQuickWindow>
#include <QStyleHints>
#include <QTemporaryDir>
#include <QTimer>
#include <QUuid>
#include <memory>

namespace {
// A protocol fixture makes the callback/edit ordering deterministic. The real
// worker persistence path is exercised separately below.
class BackendWorkflowSmoke final : public QObject {
public:
  explicit BackendWorkflowSmoke(QGuiApplication &app)
      : QObject(&app), app_(app) {
    QFile worker(directory_.filePath("worker.py"));
    if (!worker.open(QIODevice::WriteOnly) ||
        worker.write(R"PY(#!/usr/bin/python3
import json, sys
project = None
path = ''
for line in sys.stdin:
    request = json.loads(line)
    action = request['action']
    ok = True
    result = {}
    error = ''
    if action == 'create':
        path = request['path']
        project = {'id': path, 'version': 0, 'draft': {'text': '', 'format': 'plain'},
                   'candidates': [{'id': 'first'}, {'id': 'second'}], 'audio': []}
    elif action == 'save':
        if request['document']['text'] == 'force save failure':
            ok, error = False, 'file_error'
        else:
            project['draft'] = request['document']
            project['version'] += 1
    elif action == 'accept':
        project['accepted_id'] = request['audio_id']
        project['version'] += 1
    elif action == 'discard':
        project['discarded_id'] = request['audio_id']
        project['version'] += 1
    elif action in ('render', 'adopt', 'voice_preview'):
        project = dict(project or {}, unexpected_generation=action)
    if action in ('create', 'open', 'save', 'accept', 'discard', 'adopt', 'render'):
        result = {'path': path, 'project': project,
                  'analysis': {'ready': bool(project['draft']['text']) and
                               'missing' not in project['draft']['text']}}
    elif action == 'playback':
        result = {'file': '/nonexistent/' + request['audio_id'] + '.wav', 'peaks': [0.25]}
    elif action == 'material_file':
        result = {'file': '/nonexistent/material.wav'}
    print(json.dumps({'request_id': request['request_id'], 'ok': ok,
                      'error': error, 'result': result}), flush=True)
)PY") < 0) {
      fail("Could not create the worker workflow fixture");
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
    connect(backend_.get(), &DesktopBackend::requestFinished, this,
            [this](const QString &action, bool ok) { finished(action, ok); });
    connect(backend_.get(), &DesktopBackend::playFile, this,
            [this](const QUrl &) { ++played_; });
    backend_->auditionVoice("ryan");
    backend_->cancel();
  }

private:
  void fail(const char *message) {
    qCritical() << message;
    app_.exit(13);
  }
  void finished(const QString &action, bool ok) {
    auto &b = *backend_;
    if (action == "capabilities" || action == "playback")
      return;
    if (stage_ == 0 && action == "voice_preview") {
      if (ok || b.busy() || b.project().contains("unexpected_generation")) {
        fail("A queued preview was dispatched after cancellation");
        return;
      }
      stage_ = 1;
      b.createProject(QUrl::fromLocalFile(directory_.filePath("first.vox")));
      if (!b.busy() || b.rendering())
        fail("Project replacement did not expose its own busy state");
      b.editText("This edit must be rejected while replacing the project.");
      return;
    }
    if (stage_ == 7 && action == "save" && !ok) {
      if (b.busy() || b.pending() || !b.dirty() ||
          b.document()["text"] != "force save failure" ||
          b.path() != directory_.filePath("first.vox")) {
        fail("A failed save lost the draft or stranded project switching");
        return;
      }
      qInfo() << "Desktop async draft, take selection, cancellation, readiness "
                 "and failed-switch fences passed";
      app_.quit();
      return;
    }
    if (!ok) {
      fail(qPrintable("Backend workflow failed: " + action + ": " +
                      b.message()));
      return;
    }
    if (stage_ == 1 && action == "create") {
      if (b.busy() || b.dirty() || !b.document()["text"].toString().isEmpty()) {
        fail("Project replacement accepted an edit from the previous identity");
        return;
      }
      stage_ = 2;
      b.selectAudio("first");
      b.editText("Draft saved before accepting.");
      b.save();
      b.acceptSelected();
      b.selectAudio("second");
      return;
    }
    if (stage_ == 2 && action == "save") {
      // saveThen has already queued accept with the originally clicked id.
      b.editText("New input while accepting.");
      return;
    }
    if (stage_ == 2 && action == "accept") {
      if (b.project()["accepted_id"] != "first" ||
          b.selectedAudio() != "second" || !b.dirty() ||
          b.document()["text"] != "New input while accepting.") {
        fail("Accept changed its target or erased newer input");
        return;
      }
      stage_ = 3;
      b.discardSelected();
      b.selectAudio("first");
      b.editText("New input while discarding.");
      return;
    }
    if (stage_ == 3 && action == "discard") {
      if (b.project()["discarded_id"] != "second" ||
          b.selectedAudio() != "first" || !b.dirty() ||
          b.document()["text"] != "New input while discarding.") {
        fail("Discard changed the later selection or erased newer input");
        return;
      }
      stage_ = 4;
      b.render();
      b.cancel();
      return;
    }
    if (stage_ == 4 && action == "save") {
      if (b.busy() || b.project().contains("unexpected_generation")) {
        fail("Cancellation during save still adopted or rendered a script");
        return;
      }
      stage_ = 5;
      b.editText("missing role");
      b.render();
      return;
    }
    if (stage_ == 5 && action == "save") {
      if (b.busy() || b.project().contains("unexpected_generation")) {
        fail("An unready script was adopted before validation");
        return;
      }
      stage_ = 6;
      b.playMaterial("sound", "fixture");
      b.selectAudio("second");
      return;
    }
    if (stage_ == 6 && action == "material_file") {
      if (played_ != 0 || b.selectedAudio() != "second") {
        fail("A late material preview replaced the user's later audio choice");
        return;
      }
      b.previewFile(QUrl::fromLocalFile(directory_.filePath("worker.py")));
      if (played_ != 1 || !b.selectedAudio().isEmpty() ||
          !b.waveform().isEmpty()) {
        fail("An independent preview retained the selected take identity");
        return;
      }
      b.editText("force save failure");
      stage_ = 7;
      b.save();
      b.createProject(QUrl::fromLocalFile(directory_.filePath("next.vox")));
    }
  }
  QGuiApplication &app_;
  QTemporaryDir directory_;
  std::unique_ptr<DesktopBackend> backend_;
  int stage_ = 0, played_ = 0;
};
} // namespace

void startWorkflowSmoke(QGuiApplication &app, QQmlApplicationEngine &engine,
                        UiPreferences &preferences, DesktopBackend &backend,
                        const QString &path) {
  const auto requested = path;
  const auto directory = QFileInfo(requested).absolutePath() + "/" +
                         QFileInfo(requested).completeBaseName() + "-" +
                         QUuid::createUuid().toString(QUuid::WithoutBraces) +
                         ".vox";
  const auto stage = std::make_shared<int>(0);
  QTimer::singleShot(20000, &app, [&] {
    qCritical() << "Vox smoke timed out";
    app.exit(3);
  });
  QObject::connect(
      &backend, &DesktopBackend::requestFinished, &app,
      [&, directory, stage](const QString &action, bool ok) {
        if (*stage == 1 && action == "create" && ok)
          return;
        if (*stage == 1 && action == "playback" && !ok) {
          QFile oldFile(directory + "/project.json"),
              newFile(directory + ".next.vox/project.json");
          if (!oldFile.open(QIODevice::ReadOnly) ||
              !newFile.open(QIODevice::ReadOnly) ||
              QJsonDocument::fromJson(oldFile.readAll())
                      .object()["draft"]
                      .toObject()["text"]
                      .toString() !=
                  QString::fromUtf8(
                      "Vox keeps this exact script.\n你好，Vox。\n") ||
              !QJsonDocument::fromJson(newFile.readAll())
                   .object()["draft"]
                   .toObject()["text"]
                   .toString()
                   .isEmpty()) {
            qCritical() << "Queued mutation crossed project identity";
            app.exit(13);
            return;
          }
          if (backend.busy() || backend.dirty() ||
              !backend.selectedAudio().isEmpty()) {
            qCritical()
                << "Project replacement retained stale draft/audio state";
            app.exit(13);
            return;
          }
          qInfo() << "Vox UI, localization, appearance, persistence and "
                     "queued-project fence passed";
          *stage = 2;
          new BackendWorkflowSmoke(app);
          return;
        }
        if (!ok) {
          qCritical() << "Vox smoke operation failed" << action;
          app.exit(4);
          return;
        }
        auto *root = engine.rootObjects().first();
        if (action == "create") {
          auto *editor = root->findChild<QObject *>("scriptEditor");
          auto *render = root->findChild<QObject *>("renderButton");
          auto *settings = root->findChild<QObject *>("settingsButton");
          if (!editor || !render || !settings) {
            qCritical() << "Missing packaged UI controls";
            app.exit(5);
            return;
          }
          const auto originalLanguage = preferences.language();
          const auto originalTheme = preferences.theme();
          preferences.setLanguage("zh_CN");
          if (render->property("text").toString() !=
              QString::fromUtf8("生成配音")) {
            qCritical() << "Live Chinese translation failed"
                        << render->property("text");
            app.exit(6);
            return;
          }
          preferences.setLanguage("en");
          if (render->property("text").toString() != "Render voice") {
            qCritical() << "Live English translation failed";
            app.exit(7);
            return;
          }
          preferences.setTheme("dark");
          if (root->property("color").value<QColor>() != QColor("#161616")) {
            qCritical() << "Dark appearance failed";
            app.exit(9);
            return;
          }
          preferences.setTheme("light");
          if (root->property("color").value<QColor>() != QColor("#f7f7f6")) {
            qCritical() << "Light appearance failed";
            app.exit(9);
            return;
          }
          preferences.setTheme("system");
          QGuiApplication::styleHints()->setColorScheme(Qt::ColorScheme::Dark);
          QCoreApplication::processEvents();
          if (QGuiApplication::styleHints()->colorScheme() ==
                  Qt::ColorScheme::Dark &&
              root->property("color").value<QColor>() != QColor("#161616")) {
            qCritical() << "System appearance change failed";
            app.exit(9);
            return;
          }
          QGuiApplication::styleHints()->unsetColorScheme();
          preferences.setLanguage(originalLanguage);
          preferences.setTheme(originalTheme);
          editor->setProperty("text",
                              "Vox keeps this exact script.\n你好，Vox。\n");
          backend.save();
        } else if (action == "save") {
          QFile saved(directory + "/project.json");
          if (!saved.open(QIODevice::ReadOnly) ||
              QJsonDocument::fromJson(saved.readAll())
                      .object()["draft"]
                      .toObject()["text"]
                      .toString() !=
                  QString::fromUtf8(
                      "Vox keeps this exact script.\n你好，Vox。\n")) {
            qCritical() << "UI-to-worker persistence failed";
            app.exit(8);
            return;
          }
          *stage = 1;
          backend.createProject(QUrl::fromLocalFile(directory + ".next.vox"));
          backend.selectAudio("old-project-only");
          backend.editText("This queued edit belongs to the previous project.");
          backend.save();
        }
      });
  QTimer::singleShot(50, &backend, [&backend, directory] {
    backend.createProject(QUrl::fromLocalFile(directory));
  });
}

void startAudioSmoke(QGuiApplication &app, QQmlApplicationEngine &engine,
                     DesktopBackend &backend, const QString &path) {
  auto *player =
      engine.rootObjects().first()->findChild<QMediaPlayer *>("audioPlayer");
  if (!player || !player->audioOutput()) {
    qCritical() << "Packaged audio player is unavailable";
    app.exit(10);
    return;
  }
  player->audioOutput()->setMuted(true);
  QObject::connect(player, &QMediaPlayer::positionChanged, &app,
                   [&app, player](qint64 position) {
                     if (position > 100 && player->duration() > 0) {
                       qInfo() << "Real packaged audition decoded and advanced"
                               << player->duration() << "ms";
                       player->stop();
                       app.quit();
                     }
                   });
  QObject::connect(player, &QMediaPlayer::errorOccurred, &app,
                   [&app](QMediaPlayer::Error, const QString &message) {
                     qCritical() << "Audition failed" << message;
                     app.exit(11);
                   });
  QObject::connect(&backend, &DesktopBackend::requestFinished, &app,
                   [&backend, player](const QString &action, bool ok) {
                     qInfo() << "Audition request" << action << ok
                             << backend.selectedAudio() << player->source();
                     if (action == "open" && ok)
                       backend.playSelected();
                   });
  QObject::connect(player, &QMediaPlayer::mediaStatusChanged, &app,
                   [player](QMediaPlayer::MediaStatus status) {
                     qInfo() << "Audition media status" << status
                             << player->source();
                   });
  QTimer::singleShot(15000, &app, [&app] {
    qCritical() << "Audition deadline exceeded";
    app.exit(12);
  });
  backend.openProject(QUrl::fromLocalFile(path));
}
