#include "desktop_backend.hpp"
#include "ui_preferences.hpp"
#include "workflow_smoke.hpp"
#include <QDir>
#include <QFile>
#include <QGuiApplication>
#include <QJsonDocument>
#include <QQmlApplicationEngine>
#include <QQuickItem>
#include <QQuickWindow>
#include <QTemporaryDir>
#include <QTimer>
#include <QUuid>
#include <memory>

namespace {
// Exercise callback order deterministically; the UI check below uses the real
// packaged worker and its project store.
class ProductionBackendSmoke final : public QObject {
public:
  explicit ProductionBackendSmoke(QGuiApplication &app)
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
    ok, error, result = True, '', {}
    if action == 'create':
        path = request['path']
        project = {'id':path, 'version':0, 'draft':{'text':'', 'format':'plain'},
                   'candidates':[], 'audio':[]}
    elif action == 'save':
        if request['document']['text'] == 'force save failure':
            ok, error = False, 'file_error'
        else:
            project['draft'] = request['document']
            project['version'] += 1
    elif action == 'accept_segment':
        project['accepted_segment'] = request['segment_id']
        project['version'] += 1
    elif action in ('adopt', 'render_segment', 'assemble_segments'):
        project['unexpected_generation'] = action
    if action in ('create','open','save','adopt','accept_segment','render_segment','assemble_segments'):
        result = {'path':path, 'project':project, 'analysis':{'ready':True},
                  'segment_plan':{'document_digest':str(project['version']),
                      'ready':True,'adopted':True,'segments':[]}}
    elif action == 'segment_playback':
        result = {'file':'/nonexistent/' + request['segment_id'] + '.wav'}
    print(json.dumps({'request_id':request['request_id'], 'ok':ok,
                      'error':error, 'result':result}), flush=True)
)PY") < 0) {
      fail("Could not create the production protocol fixture");
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
    backend_->createProject(
        QUrl::fromLocalFile(directory_.filePath("fixture.vox")));
  }

private:
  void fail(const char *message) {
    qCritical() << message;
    app_.exit(19);
  }
  void finished(const QString &action, bool ok) {
    auto &b = *backend_;
    if (action == "capabilities")
      return;
    if (stage_ == 0 && action == "create" && ok) {
      stage_ = 1;
      b.editText("Saved before segment generation.");
      b.renderSegment("segment");
      b.cancel();
    } else if (stage_ == 1 && action == "render_segment") {
      if (ok || b.busy() || b.project().contains("unexpected_generation")) {
        fail("Cancelled segment preparation still adopted or generated");
        return;
      }
      stage_ = 10;
      QTimer::singleShot(0, this, [this] {
        stage_ = 2;
        backend_->editText("Saved before segment acceptance.");
        backend_->acceptSegment("clicked-alternative");
      });
    } else if (stage_ == 2 && action == "save" && ok) {
      b.editText("New text while segment acceptance is pending.");
    } else if (stage_ == 2 && action == "accept_segment" && ok) {
      if (b.project()["accepted_segment"] != "clicked-alternative" ||
          !b.dirty() ||
          b.document()["text"] !=
              "New text while segment acceptance is pending.") {
        fail("Segment acceptance lost its target or cleared concurrent input");
        return;
      }
      stage_ = 3;
      b.playSegment("older-audition");
      b.selectAudio("later-selection");
    } else if (stage_ == 3 && action == "segment_playback" && ok) {
      if (played_ || b.selectedAudio() != "later-selection") {
        fail("A stale segment audition replaced the later audio selection");
        return;
      }
      stage_ = 4;
      b.editText("force save failure");
      b.renderSegment("segment");
    } else if (stage_ == 4 && action == "render_segment") {
      if (ok || b.busy() || !b.dirty() ||
          b.document()["text"] != "force save failure" ||
          b.project().contains("unexpected_generation")) {
        fail("Failed segment preparation stranded state or erased the draft");
        return;
      }
      qInfo() << "Production UI, pronunciation persistence, source navigation, "
                 "cancelled preparation, acceptance edits and audition fences "
                 "passed";
      app_.quit();
    }
  }
  QGuiApplication &app_;
  QTemporaryDir directory_;
  std::unique_ptr<DesktopBackend> backend_;
  int stage_ = 0, played_ = 0;
};

class ProductionSmoke final : public QObject {
public:
  ProductionSmoke(QGuiApplication &app, QQmlApplicationEngine &engine,
                  UiPreferences &preferences, DesktopBackend &backend,
                  const QString &output)
      : QObject(engine.rootObjects().first()), app_(app),
        preferences_(preferences), backend_(backend), output_(output),
        window_(qobject_cast<QQuickWindow *>(engine.rootObjects().first())) {
    QDir().mkpath(output_);
    path_ = output_ + "/fixture-" +
            QUuid::createUuid().toString(QUuid::WithoutBraces) + ".vox";
    connect(&backend_, &DesktopBackend::requestFinished, this,
            [this](const QString &action, bool ok) {
              if ((action == "create" || action == "save") && !ok) {
                fail(qPrintable("Production persistence failed: " +
                                backend_.message()));
                return;
              }
              if (stage_ == 0 && action == "create" && ok) {
                stage_ = 1;
                backend_.setLanguage("English");
                backend_.editText(source_);
                backend_.save();
              } else if (stage_ == 1 && action == "save" && ok &&
                         !backend_.dirty()) {
                stage_ = 2;
                preferences_.setLanguage("en");
                preferences_.setTheme("dark");
                window_->setProperty("segments", true);
                QTimer::singleShot(150, this, [this] { inspectInitial(); });
              } else if (stage_ == 3 && action == "save" && ok &&
                         !backend_.dirty()) {
                stage_ = 4;
                QTimer::singleShot(150, this, [this] { inspectRule(); });
              } else if (stage_ == 5 && action == "save" && ok &&
                         !backend_.dirty()) {
                stage_ = 6;
                QTimer::singleShot(150, this, [this] { inspectRemoval(); });
              } else if (stage_ == 7 && action == "save" && ok &&
                         !backend_.dirty()) {
                stage_ = 8;
                QTimer::singleShot(150, this, [this] { inspectNavigation(); });
              }
            });
    backend_.createProject(QUrl::fromLocalFile(path_));
    QTimer::singleShot(20000, this,
                       [this] { fail("Production smoke timed out"); });
  }

private:
  void fail(const char *message) {
    window_->grabWindow().save(output_ + "/failure.png");
    qCritical() << message;
    app_.exit(18);
  }
  QQuickItem *item(const char *name) const {
    QList<QQuickItem *> pending{window_->contentItem()};
    while (!pending.isEmpty()) {
      auto *current = pending.takeLast();
      if (current->objectName() == QLatin1String(name))
        return current;
      pending.append(current->childItems());
    }
    return nullptr;
  }
  QObject *dialog() const {
    return window_->findChild<QObject *>("pronunciationDialog");
  }
  bool click(const char *name) {
    auto *control = item(name);
    return control && control->isEnabled() &&
           QMetaObject::invokeMethod(control, "clicked");
  }
  QJsonObject persistedDraft() const {
    QFile file(path_ + "/project.json");
    return file.open(QIODevice::ReadOnly)
               ? QJsonDocument::fromJson(file.readAll())
                     .object()["draft"]
                     .toObject()
               : QJsonObject{};
  }
  void inspectInitial() {
    auto *panel = item("planPanel");
    auto *text = item("segmentText-0");
    auto *assemble = item("assembleSegmentsButton");
    if (!panel || !panel->property("current").toBool() || !text ||
        text->property("text").toString() != source_ || !assemble ||
        assemble->isEnabled() || !click("segmentPronunciationButton")) {
      fail("Segments did not project source or required acceptance");
      return;
    }
    QTimer::singleShot(100, this, [this] {
      if (!dialog() || !dialog()->property("visible").toBool() ||
          !item("pronunciationSource") || !item("pronunciationSpoken")) {
        fail("Pronunciation entry point did not open its form");
        return;
      }
      item("pronunciationSource")->setProperty("text", "Vox");
      item("pronunciationSpoken")->setProperty("text", "Voks");
      if (!click("addPronunciationRule") ||
          backend_.document().contains("pronunciation_rules")) {
        fail("Uncommitted rules mutated the document");
        return;
      }
      stage_ = 3;
      if (!click("savePronunciationButton"))
        fail("Pronunciation save action is missing");
    });
  }
  void inspectRule() {
    const auto saved = persistedDraft();
    const auto rules = saved["pronunciation_rules"].toArray();
    auto *preview = item("pronunciationPreview-0");
    if (saved["text"].toString() != source_ || rules.size() != 1 ||
        rules.first().toObject()["text"].toString() != "Vox" || !preview ||
        preview->property("text").toString() !=
            "Voks keeps the authored words.") {
      fail("Pronunciation changed source or missed effective spoken text");
      return;
    }
    window_->grabWindow().save(output_ + "/pronunciation-dark-en.png");
    preferences_.setLanguage("zh_CN");
    preferences_.setTheme("light");
    QTimer::singleShot(100, this, [this] {
      if (!dialog() ||
          dialog()->property("title").toString() !=
              QString::fromUtf8("发音词典") ||
          item("pronunciationPreview-0")->property("text").toString() !=
              "Voks keeps the authored words.") {
        fail("Live pronunciation translation changed user-authored text");
        return;
      }
      window_->grabWindow().save(output_ + "/pronunciation-light-zh.png");
      if (!click("removePronunciation-0")) {
        fail("Pronunciation removal action is missing");
        return;
      }
      stage_ = 5;
      if (!click("savePronunciationButton"))
        fail("Removed pronunciation rules could not be saved");
    });
  }
  void inspectRemoval() {
    if (persistedDraft().contains("pronunciation_rules") ||
        !item("pronunciationPreview-0") ||
        item("pronunciationPreview-0")->property("text").toString() !=
            source_) {
      fail("Removing pronunciation did not restore literal speech");
      return;
    }
    QMetaObject::invokeMethod(dialog(), "close");
    preferences_.setLanguage("en");
    backend_.editText("New unsaved passage.");
    auto *panel = item("planPanel");
    if (!panel || panel->property("current").toBool() ||
        item("assembleSegmentsButton")->isEnabled()) {
      fail("Unsaved input retained a stale segment action");
      return;
    }
    stage_ = 7;
    backend_.save();
  }
  void inspectNavigation() {
    if (!click("segmentSource-0")) {
      fail("Segment source navigation action is missing");
      return;
    }
    QTimer::singleShot(100, this, [this] {
      auto *editor = item("scriptEditor");
      if (window_->property("segments").toBool() ||
          window_->property("reading").toBool() || !editor ||
          editor->property("cursorPosition").toInt() != 0 ||
          !editor->hasActiveFocus() || backend_.dirty() ||
          backend_.document()["text"] != "New unsaved passage.") {
        fail("Segment navigation lost source position, focus or exact text");
        return;
      }
      new ProductionBackendSmoke(app_);
    });
  }
  QGuiApplication &app_;
  UiPreferences &preferences_;
  DesktopBackend &backend_;
  QString output_, path_;
  const QString source_ = "Vox keeps the authored words.";
  QQuickWindow *window_;
  int stage_ = 0;
};
} // namespace

void startProductionSmoke(QGuiApplication &app, QQmlApplicationEngine &engine,
                          UiPreferences &preferences, DesktopBackend &backend,
                          const QString &output) {
  new ProductionSmoke(app, engine, preferences, backend, output);
}
