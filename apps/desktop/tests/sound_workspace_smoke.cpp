#include "desktop_backend.hpp"
#include "ui_preferences.hpp"
#include "workflow_smoke.hpp"
#include <QDir>
#include <QFile>
#include <QGuiApplication>
#include <QJsonArray>
#include <QJsonDocument>
#include <QMouseEvent>
#include <QQmlApplicationEngine>
#include <QQuickItem>
#include <QQuickWindow>
#include <QTimer>
#include <QUuid>

namespace {
class SoundWorkspaceSmoke final : public QObject {
public:
  SoundWorkspaceSmoke(QGuiApplication &app, QQmlApplicationEngine &engine,
                      UiPreferences &preferences, DesktopBackend &backend,
                      const QString &output)
      : QObject(engine.rootObjects().first()), app_(app),
        preferences_(preferences), backend_(backend), output_(output),
        window_(qobject_cast<QQuickWindow *>(engine.rootObjects().first())) {
    QDir().mkpath(output_);
    path_ = output_ + "/fixture-" +
            QUuid::createUuid().toString(QUuid::WithoutBraces) + ".vox";
    export_ = output_ + "/original-" +
              QUuid::createUuid().toString(QUuid::WithoutBraces) + ".wav";
    connect(&backend_, &DesktopBackend::playFile, this,
            [this](const QUrl &file) { auditioned_ = file.isLocalFile(); });
    connect(
        &backend_, &DesktopBackend::requestFinished, this,
        [this](const QString &action, bool ok) {
          if (action == "probe" || action == "capabilities" || action == "save")
            return;
          if (!ok) {
            fail(qPrintable("Sound operation failed: " + action + " " +
                            backend_.message()));
            return;
          }
          if (action == "create" && stage_ == 0) {
            stage_ = 1;
            preferences_.setLanguage("en");
            preferences_.setTheme("light");
            window_->setProperty("productionMode", "sfx");
            QTimer::singleShot(200, this, [this] { inspectForm(); });
          } else if (action == "add_builtin" && stage_ == 2) {
            stage_ = 3;
            const auto sounds = manifest()["sound_library"].toArray();
            if (sounds.size() != 1 ||
                sounds.first().toObject()["content"].isObject()) {
              fail("A built-in sound did not enter review as a candidate");
              return;
            }
            sound_ = sounds.first().toObject();
            QByteArray digest;
            for (const auto value :
                 sound_["original"].toObject()["digest"].toArray())
              digest.append(static_cast<char>(value.toInt()));
            if (digest.size() != 32) {
              fail("Sound original digest is not a 32-byte content identity");
              return;
            }
            original_ = bytes(path_ + "/objects/" +
                              QString::fromLatin1(digest.toHex()));
            if (original_.isEmpty()) {
              fail("Sound original object could not be read");
              return;
            }
            QTimer::singleShot(150, this, [this] {
              if (!click("soundCandidatePreviewButton"))
                fail("Candidate audition control failed");
            });
          } else if (action == "material_file" && stage_ == 3) {
            stage_ = 4;
            if (!auditioned_ || !click("soundCandidateAcceptButton"))
              fail("Candidate audition or explicit acceptance control failed");
          } else if (action == "accept_sound" && stage_ == 4) {
            stage_ = 5;
            const auto sound =
                manifest()["sound_library"].toArray().first().toObject();
            if (!sound["content"].isObject() ||
                sound["original"] != sound_["original"]) {
              fail("Sound acceptance changed the original source");
              return;
            }
            backend_.exportMaterial("sound", sound["id"].toString(),
                                    QUrl::fromLocalFile(export_));
          } else if (action == "export_material" && stage_ == 5) {
            if (bytes(export_) != original_) {
              fail("Original-quality export changed WAV bytes");
              return;
            }
            stage_ = 6;
            backend_.openProject(QUrl::fromLocalFile(path_));
          } else if (action == "open" && stage_ == 6) {
            stage_ = 7;
            window_->setProperty("productionMode", "short_music");
            QTimer::singleShot(200, this, [this] { inspectReopen(); });
          }
        });
    QTimer::singleShot(20000, this,
                       [this] { fail("Sound workspace check timed out"); });
    backend_.createProject(QUrl::fromLocalFile(path_));
  }

private:
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
    auto *target = item(name);
    if (!target || !target->isEnabled())
      return false;
    const auto position =
        target->mapToScene(QPointF(target->width() / 2, target->height() / 2));
    if (!QRectF(0, 0, window_->width(), window_->height()).contains(position))
      return false;
    const QPointF global = window_->mapToGlobal(position.toPoint());
    QMouseEvent press(QEvent::MouseButtonPress, position, global,
                      Qt::LeftButton, Qt::LeftButton, Qt::NoModifier);
    QMouseEvent release(QEvent::MouseButtonRelease, position, global,
                        Qt::LeftButton, Qt::NoButton, Qt::NoModifier);
    QCoreApplication::sendEvent(window_, &press);
    QCoreApplication::sendEvent(window_, &release);
    return true;
  }
  static QByteArray bytes(const QString &path) {
    QFile file(path);
    return file.open(QIODevice::ReadOnly) ? file.readAll() : QByteArray{};
  }
  QJsonObject manifest() const {
    return QJsonDocument::fromJson(bytes(path_ + "/project.json")).object();
  }
  void fail(const char *message) {
    if (finished_)
      return;
    finished_ = true;
    window_->grabWindow().save(output_ + "/failure.png");
    qCritical() << message << "stage" << stage_;
    app_.exit(19);
  }
  bool categorySelectionMatches(const QString &category) const {
    for (const auto &key : {QStringLiteral("sfx"), QStringLiteral("ambience"),
                            QStringLiteral("short_music")}) {
      const auto name = ("soundCategory_" + key).toLatin1();
      auto *button = item(name.constData());
      if (!button ||
          button->property("categorySelected").toBool() != (category == key))
        return false;
    }
    return true;
  }
  void inspectForm() {
    auto *workspace = item("soundWorkspace");
    auto *seconds = item("soundSeconds");
    auto *variations = item("soundVariations");
    if (!workspace || !seconds || !variations ||
        !categorySelectionMatches("sfx") ||
        seconds->property("from").toInt() != 1 ||
        seconds->property("to").toInt() != 30 ||
        variations->property("to").toInt() != 4) {
      fail("Sound generation bounds are missing");
      return;
    }
    item("soundName")->setProperty("text", "Rain candidates");
    item("soundPrompt")
        ->setProperty("text", "Gentle rain, no speech or music.");
    variations->setProperty("value", 3);
    if (item("generateSoundButton")->property("text").toString() !=
            "Generate 3 candidates" ||
        !click("soundCategory_short_music") ||
        window_->property("productionMode").toString() != "short_music" ||
        !categorySelectionMatches("short_music")) {
      fail("Sound category or variation interaction failed");
      return;
    }
    if (!click("soundLoopAudition") ||
        !workspace->property("loopAudition").toBool() ||
        !click("soundCategory_ambience") ||
        workspace->property("loopAudition").toBool() ||
        !categorySelectionMatches("ambience")) {
      fail("Loop audition/category lifecycle failed");
      return;
    }
    preferences_.setLanguage("zh_CN");
    if (!categorySelectionMatches("ambience") ||
        item("soundCategory_ambience")->property("text").toString() !=
            QString::fromUtf8("环境声") ||
        item("generateSoundButton")->property("text").toString() !=
            QString::fromUtf8("生成 3 个候选")) {
      fail("Sound workspace live Chinese translation failed");
      return;
    }
    window_->grabWindow().save(output_ + "/ambience-light-zh.png");
    stage_ = 2;
    if (!click("soundBuiltinAddButton"))
      fail("Built-in sound action is unavailable");
  }
  void inspectReopen() {
    const auto sounds = manifest()["sound_library"].toArray();
    auto *workspace = item("soundWorkspace");
    if (sounds.size() != 1 ||
        !sounds.first().toObject()["content"].isObject() || !workspace ||
        workspace->property("loopAudition").toBool() ||
        !item("soundCandidateList") ||
        item("soundCandidateList")->property("count").toInt() != 1) {
      fail("Accepted sound did not reopen across categories");
      return;
    }
    preferences_.setLanguage("en");
    preferences_.setTheme("dark");
    QTimer::singleShot(200, this, [this] {
      if (!categorySelectionMatches("short_music")) {
        fail("Sound category selection changed after English retranslation");
        return;
      }
      window_->grabWindow().save(output_ + "/short-music-dark-en.png");
      qInfo() << "Sound category controls, bounded variations, loop audition, "
                 "live language, local candidate acceptance, reopen and exact "
                 "original export passed";
      finished_ = true;
      app_.quit();
    });
  }
  QGuiApplication &app_;
  UiPreferences &preferences_;
  DesktopBackend &backend_;
  QString output_, path_, export_;
  QQuickWindow *window_;
  QJsonObject sound_;
  QByteArray original_;
  int stage_ = 0;
  bool auditioned_ = false, finished_ = false;
};
} // namespace

void startSoundWorkspaceSmoke(QGuiApplication &app,
                              QQmlApplicationEngine &engine,
                              UiPreferences &preferences,
                              DesktopBackend &backend, const QString &output) {
  new SoundWorkspaceSmoke(app, engine, preferences, backend, output);
}
