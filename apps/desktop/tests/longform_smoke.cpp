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
class ProductionLifecycleSmoke final : public QObject {
public:
  ProductionLifecycleSmoke(QGuiApplication &app, UiPreferences &preferences)
      : QObject(&app), app_(app), preferences_(preferences) {
    QFile worker(directory_.filePath("worker.py"));
    if (!worker.open(QIODevice::WriteOnly) ||
        worker.write(R"PY(#!/usr/bin/python3
import json, sys
project = None
path = ''
batch_count = 0
for line in sys.stdin:
    request = json.loads(line)
    action = request['action']
    if action == 'create':
        path = request['path']
        project = {'id':path,'version':0,'draft':{'text':'','format':'plain'},
                   'candidates':[],'audio':[],'voice_library':[]}
    elif action == 'save':
        project['draft'] = request['document']
        project['version'] += 1
    elif action == 'generate_voice_design_batch':
        project['unexpected_generation'] = True
    elif action == 'accept_voice_design':
        project['accepted_design'] = request['id']
        project['version'] += 1
    result = {}
    if action in ('create','save','open','accept_voice_design','generate_voice_design_batch','chapter_batch'):
        result = {'path':path,'project':project,'analysis':{'ready':True},'segment_plan':{}}
    if action == 'chapter_batch':
        batch_count += 1
        state = 'completed' if batch_count <= 2 else 'cancelled' if batch_count == 3 else 'failed'
        second = 'completed' if batch_count == 2 else 'skipped'
        result['parent_path'], result['parent_project'] = path, project
        result['chapters'] = []
        result['chapter_batch'] = {'status':state,'chapters':[
            {'chapter_id':'first','status':'skipped'},
            {'chapter_id':'second','status':second}]}
        print(json.dumps({'event':'progress','request_id':request['request_id'],
            'chapter_id':'first','chapter_index':0,'total_chapters':2,
            'completed_segments':1,'total_segments':1,'completed':1,'total':1,
            'status':'skipped'}),flush=True)
    elif action == 'material_file':
        result = {'file':'/nonexistent/voice-design.wav','peaks':[0.5]}
    print(json.dumps({'request_id':request['request_id'],'ok':True,'result':result}),flush=True)
)PY") < 0) {
      fail("Could not create design lifecycle fixture");
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
    connect(backend_.get(), &DesktopBackend::progressChanged, this, [this] {
      if (backend_->batchingChapters() &&
          backend_->chapterProgress()["status"] == "skipped")
        ++skippedProgress_;
    });
    backend_->createProject(
        QUrl::fromLocalFile(directory_.filePath("fixture.vox")));
  }

private:
  void fail(const char *message) {
    qCritical() << message;
    app_.exit(31);
  }
  void finished(const QString &action, bool ok) {
    auto &b = *backend_;
    if (stage_ == 0 && action == "create" && ok) {
      stage_ = 1;
      b.editText("Save before design preparation.");
      b.generateVoiceDesign("Warm voice", "Exact sample.", "English", 2);
      b.cancel();
    } else if (stage_ == 1 && action == "generate_voice_design_batch") {
      if (ok || b.busy() || b.project().contains("unexpected_generation") ||
          !b.project()["voice_library"].toList().isEmpty()) {
        fail("Cancelled design preparation dispatched or saved a voice");
        return;
      }
      stage_ = 2;
      QTimer::singleShot(0, this, [this] {
        backend_->editText("Save before explicit design acceptance.");
        backend_->acceptVoiceDesign("clicked-design", "Named synthetic voice");
      });
    } else if (stage_ == 2 && action == "save" && ok) {
      b.editText("Concurrent authored text remains exact.");
    } else if (stage_ == 2 && action == "accept_voice_design" && ok) {
      if (b.project()["accepted_design"] != "clicked-design" || !b.dirty() ||
          b.document()["text"] != "Concurrent authored text remains exact.") {
        fail("Design acceptance changed its target or erased concurrent text");
        return;
      }
      stage_ = 3;
      b.playMaterial("voice_design", "older-design");
      b.selectAudio("later-audio-selection");
    } else if (stage_ == 3 && action == "material_file" && ok) {
      if (played_ || b.selectedAudio() != "later-audio-selection" ||
          !b.waveform().isEmpty()) {
        fail("Stale design audition replaced a later selection");
        return;
      }
      stage_ = 4;
      b.renderChapterBatch();
    } else if (stage_ == 4 && action == "chapter_batch" && ok) {
      if (b.message() !=
              "All chapters were reused. No new candidates were generated." ||
          b.productionStatus("skipped") != "Reused" || !skippedProgress_) {
        fail("An all-reused chapter batch claimed new candidates or pending "
             "progress");
        return;
      }
      preferences_.setLanguage("zh_CN");
      b.retranslate();
      if (b.message() !=
              QString::fromUtf8("所有章节均已复用，未生成新候选。") ||
          b.productionStatus("skipped") != QString::fromUtf8("已复用")) {
        fail("Reused chapter completion or progress missed its Chinese "
             "translation");
        return;
      }
      preferences_.setLanguage("en");
      b.retranslate();
      stage_ = 5;
      b.renderChapterBatch();
    } else if (stage_ == 5 && action == "chapter_batch" && ok) {
      if (b.message() != "Chapter candidates are ready. Review and accept each "
                         "chapter before export.") {
        fail("A partly reused batch hid its newly generated candidates");
        return;
      }
      stage_ = 6;
      b.renderChapterBatch();
    } else if (stage_ == 6 && action == "chapter_batch" && ok) {
      if (b.message() !=
          "Chapter generation stopped. Completed candidates are kept.") {
        fail("Reused chapter entries hid the batch cancellation");
        return;
      }
      stage_ = 7;
      b.renderChapterBatch();
    } else if (stage_ == 7 && action == "chapter_batch" && ok) {
      if (b.message() !=
          "Some chapters need attention. Completed candidates are kept.") {
        fail("Reused chapter entries hid the batch failure");
        return;
      }
      qInfo()
          << "Long-form UI persistence, chapter identity, live translations, "
             "design cancellation, explicit-save targeting, audition fences "
             "and reused/mixed/cancelled/failed chapter batch statuses passed";
      app_.quit();
    }
  }
  QGuiApplication &app_;
  UiPreferences &preferences_;
  QTemporaryDir directory_;
  std::unique_ptr<DesktopBackend> backend_;
  int stage_ = 0, played_ = 0, skippedProgress_ = 0;
};

class LongformSmoke final : public QObject {
public:
  LongformSmoke(QGuiApplication &app, QQmlApplicationEngine &engine,
                UiPreferences &preferences, DesktopBackend &backend,
                const QString &output)
      : QObject(engine.rootObjects().first()), app_(app),
        preferences_(preferences), backend_(backend), output_(output),
        window_(qobject_cast<QQuickWindow *>(engine.rootObjects().first())) {
    QDir().mkpath(output_);
    path_ = output_ + "/fixture-" +
            QUuid::createUuid().toString(QUuid::WithoutBraces) + ".vox";
    preferences_.setLanguage("en");
    preferences_.setTheme("dark");
    connect(&backend_, &DesktopBackend::requestFinished, this,
            [this](const QString &action, bool ok) { finished(action, ok); });
    backend_.createProject(QUrl::fromLocalFile(path_));
    QTimer::singleShot(24000, this,
                       [this] { fail("Long-form smoke timed out"); });
  }

private:
  void fail(const char *message) {
    window_->grabWindow().save(output_ + "/failure.png");
    qCritical() << message << "stage" << stage_ << backend_.message();
    app_.exit(30);
  }
  QQuickItem *item(const char *name) const {
    QList<QQuickItem *> pending{window_->contentItem()};
    while (!pending.isEmpty()) {
      auto *current = pending.takeLast();
      if (current->objectName() == QLatin1String(name) && current->isVisible())
        return current;
      pending.append(current->childItems());
    }
    return nullptr;
  }
  bool click(const char *name) {
    auto *control = item(name);
    return control && control->isEnabled() &&
           QMetaObject::invokeMethod(control, "clicked");
  }
  void add(const QString &title) {
    if (!click("addChapterButton")) {
      fail("Chapter add entry is unavailable");
      return;
    }
    QTimer::singleShot(80, this, [this, title] {
      auto *field = item("newChapterTitle");
      if (!field) {
        fail("Chapter title dialog did not open");
        return;
      }
      field->setProperty("text", title);
      if (!click("confirmChapterTitle"))
        fail("Chapter title confirmation is unavailable");
    });
  }
  void finished(const QString &action, bool ok) {
    if (!ok && action != "probe" && action != "capabilities") {
      fail("A long-form persistence action failed");
      return;
    }
    if (stage_ == 0 && action == "create") {
      stage_ = 1;
      if (!click("productionMode-chapters")) {
        fail("Chapters workspace entry is missing");
        return;
      }
      QTimer::singleShot(80, this, [this] { add("First chapter"); });
    } else if (stage_ == 1 && action == "chapter_open") {
      if (backend_.selectedChapter().isEmpty() || backend_.path() == path_ ||
          backend_.collectionPath() != path_) {
        fail("Chapter selection did not retain its collection identity");
        return;
      }
      first_ = backend_.selectedChapter();
      stage_ = 2;
      backend_.editText("First chapter text stays exact.");
      backend_.save();
    } else if (stage_ == 2 && action == "save") {
      stage_ = 3;
      QTimer::singleShot(80, this, [this] { add("Second chapter"); });
    } else if (stage_ == 3 && action == "chapter_open") {
      second_ = backend_.selectedChapter();
      if (first_ == second_ || backend_.chapters().size() != 2) {
        fail("New chapters reused an existing identity");
        return;
      }
      stage_ = 4;
      backend_.editText("Second chapter text stays independent.");
      backend_.save();
    } else if (stage_ == 4 && action == "save") {
      stage_ = 5;
      QTimer::singleShot(80, this, [this] {
        if (!click("renameChapterButton")) {
          fail("Chapter rename entry is missing");
          return;
        }
        QTimer::singleShot(80, this, [this] {
          item("newChapterTitle")->setProperty("text", "Renamed second");
          if (!click("confirmChapterTitle"))
            fail("Chapter rename confirmation failed");
        });
      });
    } else if (stage_ == 5 && action == "chapter_rename") {
      stage_ = 6;
      QTimer::singleShot(80, this, [this] {
        if (!click("moveChapterUpButton"))
          fail("Chapter reorder entry is missing");
      });
    } else if (stage_ == 6 && action == "chapter_reorder") {
      const auto chapters = backend_.chapters();
      if (chapters.first().toMap()["id"] != second_ ||
          chapters.first().toMap()["title"] != "Renamed second") {
        fail("Chapter order or title did not persist");
        return;
      }
      stage_ = 7;
      backend_.selectChapter(first_);
    } else if (stage_ == 7 && action == "chapter_open") {
      if (backend_.document()["text"] != "First chapter text stays exact.") {
        fail("Switching chapters replaced the wrong script");
        return;
      }
      stage_ = 8;
      backend_.editText("First chapter saved before switching.");
      backend_.selectChapter(second_);
    } else if (stage_ == 8 && action == "chapter_open") {
      if (backend_.document()["text"] !=
              "Second chapter text stays independent." ||
          backend_.dirty() ||
          backend_.collection()["draft"].toMap()["text"] != "") {
        fail("Chapter switching lost its save or mutated the shared draft");
        return;
      }
      stage_ = 9;
      window_->grabWindow().save(output_ + "/chapters-dark-en.png");
      preferences_.setLanguage("zh_CN");
      preferences_.setTheme("light");
      QTimer::singleShot(100, this, [this] {
        auto *heading = item("chaptersHeading");
        if (!heading ||
            heading->property("text").toString() != QString::fromUtf8("章节") ||
            !item("chapterStatus-0") ||
            item("chapterStatus-0")->property("text").toString() !=
                QString::fromUtf8("待处理") ||
            backend_.document()["text"] !=
                "Second chapter text stays independent.") {
          fail("Chapter translation changed authored text or missed live "
               "labels");
          return;
        }
        window_->grabWindow().save(output_ + "/chapters-light-zh.png");
        window_->resize(1080, 740);
        QTimer::singleShot(100, this, [this] {
          window_->grabWindow().save(output_ +
                                     "/chapters-minimum-light-zh.png");
          window_->resize(1320, 900);
          if (!click("productionMode-voice_design"))
            fail("Voice design workspace entry is missing");
        });
      });
    } else if (stage_ == 9 && action == "open") {
      if (!backend_.selectedChapter().isEmpty() || backend_.path() != path_) {
        fail("Voice design did not return to the collection library");
        return;
      }
      stage_ = 10;
      QTimer::singleShot(100, this, [this] {
        auto *description = item("voiceDesignDescription");
        auto *sample = item("voiceDesignSample");
        auto *count = item("voiceDesignCount");
        if (!description || !sample || !count ||
            count->property("to").toInt() != 4) {
          fail("Voice design form or bounded candidate count is missing");
          return;
        }
        description->setProperty("text", "Warm synthetic narrator");
        sample->setProperty("text", "Authored sample words.");
        window_->grabWindow().save(output_ + "/voice-design-light-zh.png");
        preferences_.setLanguage("en");
        preferences_.setTheme("dark");
        QTimer::singleShot(100, this, [this] {
          if (item("voiceDesignDescription")->property("text") !=
                  "Warm synthetic narrator" ||
              item("voiceDesignSample")->property("text") !=
                  "Authored sample words." ||
              backend_.dirty() ||
              !(backend_.document()["text"].toString()).isEmpty()) {
            fail("Voice design form adopted a script or translated authored "
                 "input");
            return;
          }
          window_->grabWindow().save(output_ + "/voice-design-dark-en.png");
          if (!click("productionMode-chapters")) {
            fail("Chapter return entry is missing");
            return;
          }
          stage_ = 11;
          backend_.selectChapter(second_);
        });
      });
    } else if (stage_ == 11 && action == "chapter_open") {
      stage_ = 12;
      QTimer::singleShot(80, this, [this] {
        if (!click("removeChapterButton"))
          fail("Chapter removal entry is missing");
      });
    } else if (stage_ == 12 && action == "open") {
      const auto chapters = backend_.chapters();
      int active = 0;
      for (const auto &chapter : chapters)
        if (!chapter.toMap()["archived"].toBool())
          ++active;
      if (active != 1 || !backend_.selectedChapter().isEmpty()) {
        fail("Chapter removal did not preserve the active collection");
        return;
      }
      stage_ = 13;
      backend_.selectChapter(first_);
    } else if (stage_ == 13 && action == "chapter_open") {
      if (backend_.document()["text"] !=
          "First chapter saved before switching.") {
        fail("Saved chapter text was lost after archive and reopen");
        return;
      }
      stage_ = 14;
      heldChapter_ = backend_.path();
      if (!QDir().rename(heldChapter_, heldChapter_ + ".held")) {
        fail("Could not isolate the missing-chapter fixture");
        return;
      }
      backend_.refreshChapters();
    } else if (stage_ == 14 && action == "chapter_list") {
      if (backend_.chapterError().isEmpty() ||
          backend_.chapters().size() != 2) {
        fail("Unavailable chapter state erased the known chapter list");
        return;
      }
      if (!QDir().rename(heldChapter_ + ".held", heldChapter_)) {
        fail("Could not restore the isolated chapter fixture");
        return;
      }
      stage_ = 15;
      backend_.refreshChapters();
    } else if (stage_ == 15 && action == "chapter_list") {
      if (!backend_.chapterError().isEmpty() ||
          backend_.chapters().size() != 2) {
        fail("Restored chapter state did not clear its visible error");
        return;
      }
      stage_ = 16;
      new ProductionLifecycleSmoke(app_, preferences_);
    }
  }
  QGuiApplication &app_;
  UiPreferences &preferences_;
  DesktopBackend &backend_;
  QString output_, path_, first_, second_, heldChapter_;
  QQuickWindow *window_;
  int stage_ = 0;
};
} // namespace

void startLongformSmoke(QGuiApplication &app, QQmlApplicationEngine &engine,
                        UiPreferences &preferences, DesktopBackend &backend,
                        const QString &output) {
  new LongformSmoke(app, engine, preferences, backend, output);
}
