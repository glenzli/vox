#include "desktop_backend.hpp"
#include "reference_recorder.hpp"
#include "ui_preferences.hpp"
#include "workflow_smoke.hpp"
#include <QBuffer>
#include <QDir>
#include <QFile>
#include <QGuiApplication>
#include <QJsonArray>
#include <QQmlApplicationEngine>
#include <QQuickWindow>
#include <QTimer>
#include <QUuid>
#include <cmath>

// Exercise capture conversion and disposable-file ownership without recording a
// person.
class ReferenceRecorderSmoke {
public:
  static bool fixture(const QString &path) {
    ReferenceRecorder recorder;
    recorder.m_format.setSampleRate(48000);
    recorder.m_format.setChannelCount(2);
    recorder.m_format.setSampleFormat(QAudioFormat::Float);
    QByteArray input;
    for (int i = 0; i < 48000; ++i) {
      const float v = 0.2f * std::sin(i * 0.03f);
      for (int ch = 0; ch < 2; ++ch)
        input.append(reinterpret_cast<const char *>(&v), 4);
    }
    QBuffer buffer(&input);
    buffer.open(QIODevice::ReadOnly);
    recorder.m_input = &buffer;
    recorder.drain();
    recorder.m_input = nullptr;
    recorder.finalizeCapture();
    const auto temporary = recorder.file().toLocalFile();
    QFile file(temporary);
    if (recorder.milliseconds() != 1000 || !file.open(QIODevice::ReadOnly))
      return false;
    const auto bytes = file.readAll();
    file.close();
    if (bytes.size() != 44 + 48000 * 4 ||
        bytes.mid(20, 4) != QByteArray::fromHex("01000200") ||
        !QFile::copy(temporary, path))
      return false;
    recorder.cancel();
    return !QFile::exists(temporary) && recorder.file().isEmpty() &&
           recorder.milliseconds() == 0;
  }
};
namespace {
class StudioSmoke final : public QObject {
public:
  StudioSmoke(QGuiApplication &app, QQmlApplicationEngine &engine,
              UiPreferences &preferences, DesktopBackend &backend,
              const QString &output)
      : QObject(engine.rootObjects().first()), app_(app),
        preferences_(preferences), backend_(backend),
        window_(qobject_cast<QQuickWindow *>(engine.rootObjects().first())),
        output_(output) {
    QDir().mkpath(output);
    file_ = output + "/capture-" +
            QUuid::createUuid().toString(QUuid::WithoutBraces) + ".wav";
    if (!ReferenceRecorderSmoke::fixture(file_)) {
      fail("Capture PCM conversion/cleanup failed");
      return;
    }
    connect(&backend_, &DesktopBackend::requestFinished, this,
            [this](const QString &action, bool ok) {
              if (action == "probe" || action == "capabilities")
                return;
              if (!ok) {
                fail(qPrintable("Studio operation failed: " + action + ": " +
                                backend_.message()));
                return;
              }
              if (action == "create" && stage_ == 0) {
                stage_ = 1;
                backend_.importReference(
                    "Fixture voice", "Exact local reference words.",
                    QUrl::fromLocalFile(file_), true, true);
              } else if (action == "import_reference" && stage_ == 1) {
                stage_ = 2;
                reference_ = backend_.project()["voice_library"]
                                 .toList()
                                 .first()
                                 .toMap()["id"]
                                 .toString();
                backend_.addBuiltin("bell");
              } else if (action == "add_builtin" && stage_ == 2) {
                stage_ = 3;
                sound_ = backend_.project()["sound_library"]
                             .toList()
                             .first()
                             .toMap()["id"]
                             .toString();
                backend_.acceptSound(sound_);
              } else if (action == "accept_sound" && stage_ == 3) {
                stage_ = 4;
                backend_.bindReference("Guide", reference_);
                backend_.setFormat("production_script");
                backend_.editText(
                    "[role: Guide]\n[cue: turn; sound: external]\n[speaker: "
                    "Guide]\nHello.\n[audio: turn]");
                backend_.bindSound("turn", sound_);
                backend_.save();
              } else if (action == "save" && stage_ == 4) {
                stage_ = 5;
                QTimer::singleShot(200, this, [this] { inspect(); });
              }
            });
    backend_.createProject(QUrl::fromLocalFile(
        output + "/studio-" +
        QUuid::createUuid().toString(QUuid::WithoutBraces) + ".vox"));
    QTimer::singleShot(20000, this,
                       [this] { fail("Studio native check timed out"); });
  }

private:
  void fail(const char *s) {
    qCritical() << s;
    app_.exit(16);
  }
  bool click(const char *name) {
    auto *item = window_->findChild<QObject *>(name);
    return item && QMetaObject::invokeMethod(item, "clicked");
  }
  void inspect() {
    if (backend_.dirty() || !backend_.analysis()["ready"].toBool()) {
      fail("Native material bindings were not saved/ready");
      return;
    }
    preferences_.setLanguage("zh_CN");
    preferences_.setLanguage("en");
    if (backend_.builtinSounds().first().toMap()["label"].toString() !=
        "Soft chime") {
      fail("English sound projection failed");
      return;
    }
    preferences_.setLanguage("zh_CN");
    if (backend_.builtinSounds().first().toMap()["label"].toString() !=
            "轻柔提示音" ||
        backend_.languages().first().toMap()["label"].toString() !=
            "自动识别") {
      fail("Live Chinese sound/language projection failed");
      return;
    }
    preferences_.setTheme("light");
    if (!click("voiceLibraryButton")) {
      fail("Voice library trigger missing");
      return;
    }
    QTimer::singleShot(250, this, [this] {
      auto *library = window_->findChild<QObject *>("materialsDialog");
      if (!library || !library->property("visible").toBool()) {
        fail("Library did not open");
        return;
      }
      window_->grabWindow().save(output_ + "/voices-light.png");
      if (!click("addReferenceButton")) {
        fail("Reference trigger missing");
        return;
      }
      QTimer::singleShot(250, this, [this] {
        auto *dialog = window_->findChild<QObject *>("referenceDialog");
        auto *save = window_->findChild<QObject *>("saveReferenceButton");
        if (!dialog || !dialog->property("visible").toBool() || !save ||
            save->property("enabled").toBool()) {
          fail("Reference form admission failed");
          return;
        }
        window_->grabWindow().save(output_ + "/recording-form-light.png");
        QMetaObject::invokeMethod(dialog, "close");
        auto *library = window_->findChild<QObject *>("materialsDialog");
        QMetaObject::invokeMethod(library, "close");
        preferences_.setTheme("dark");
        click("soundLibraryButton");
        QTimer::singleShot(250, this, [this] {
          window_->grabWindow().save(output_ + "/sounds-dark.png");
          qInfo()
              << "Native capture encoding, cleanup, reference import, sound "
                 "adoption, role/cue binding and library forms passed";
          app_.quit();
        });
      });
    });
  }
  QGuiApplication &app_;
  UiPreferences &preferences_;
  DesktopBackend &backend_;
  QQuickWindow *window_;
  QString output_, file_, reference_, sound_;
  int stage_ = 0;
};
} // namespace
void startStudioSmoke(QGuiApplication &app, QQmlApplicationEngine &engine,
                      UiPreferences &preferences, DesktopBackend &backend,
                      const QString &output) {
  new StudioSmoke(app, engine, preferences, backend, output);
}
