#include "desktop_backend.hpp"
#include "ui_preferences.hpp"
#include "workflow_smoke.hpp"
#include <QDir>
#include <QFile>
#include <QGuiApplication>
#include <QQmlApplicationEngine>
#include <QQuickItem>
#include <QQuickWindow>
#include <QTimer>
#include <QUuid>

namespace {
class ReadingSmoke final : public QObject {
public:
  ReadingSmoke(QGuiApplication &app, QQmlApplicationEngine &engine,
               UiPreferences &preferences, DesktopBackend &backend,
               const QString &output)
      : QObject(engine.rootObjects().first()), app_(app),
        preferences_(preferences), backend_(backend), output_(output),
        window_(qobject_cast<QQuickWindow *>(engine.rootObjects().first())) {
    QDir().mkpath(output_);
    path_ = output_ + "/fixture-" +
            QUuid::createUuid().toString(QUuid::WithoutBraces) + ".vox";
    source_ = QString::fromUtf8(
        "[production: dialogue]\n[role: Guide]\n[role: 旁白]\n"
        "[language: Chinese]\n你好，Vox。\n[pause: 0.3s]\n"
        "[repeat: 2; gap: 0.4s]\n[speaker: Guide]\n[language: English]\n"
        "Every story begins with a voice.\n[end-repeat]\n"
        "[note: speaker: keep this authored note]");
    connect(&backend_, &DesktopBackend::requestFinished, this,
            [this](const QString &action, bool ok) {
              if (action == "create") {
                if (!ok) {
                  fail("Reading fixture creation failed");
                  return;
                }
                backend_.setFormat("production_script");
                backend_.setVoice(
                    backend_.voices().at(5).toMap()["alias"].toString());
                backend_.editText(source_);
                backend_.save();
              } else if (action == "save" && !ok) {
                fail(qPrintable("Reading fixture save failed: " +
                                backend_.message()));
              } else if (action == "save" && ok && !backend_.dirty()) {
                if (stage_ == 0) {
                  stage_ = 1;
                  before_ = manifest();
                  preferences_.setLanguage("zh_CN");
                  preferences_.setTheme("light");
                  window_->setProperty("reading", true);
                  QTimer::singleShot(250, this, [this] { inspectLight(); });
                } else if (stage_ == 4) {
                  stage_ = 5;
                  before_ = manifest();
                  window_->setProperty("reading", true);
                  QTimer::singleShot(250, this, [this] { inspectPlain(); });
                }
              }
            });
    backend_.createProject(QUrl::fromLocalFile(path_));
    QTimer::singleShot(20000, this,
                       [this] { fail("Reading check timed out"); });
  }

private:
  void fail(const char *message) {
    window_->grabWindow().save(output_ + "/failure.png");
    qCritical() << message;
    app_.exit(17);
  }
  QByteArray manifest() const {
    QFile file(path_ + "/project.json");
    return file.open(QIODevice::ReadOnly) ? file.readAll() : QByteArray{};
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
  bool preserved() const {
    return !before_.isEmpty() && before_ == manifest() && !backend_.dirty() &&
           backend_.document()["text"].toString() == source_;
  }
  void inspectLight() {
    auto *rejected = item("readingText-5");
    auto *spoken = item("readingText-10");
    auto *group = item("readingRepeat-7");
    auto *title = item("readingRepeatTitle-7");
    if (!preserved() || !rejected || !spoken || !group || !title ||
        item("formatChoice")->property("currentIndex").toInt() != 1 ||
        item("defaultVoice")->property("currentIndex").toInt() != 5 ||
        rejected->property("text").toString() !=
            QString::fromUtf8("你好，Vox。") ||
        spoken->property("text").toString() !=
            "Every story begins with a voice." ||
        title->property("text").toString() != QString::fromUtf8("重复 2×") ||
        group->height() < 100 || group->height() > 240 ||
        !group->mapRectToScene(group->boundingRect())
             .contains(spoken->mapRectToScene(spoken->boundingRect())) ||
        item("readingEntry-4") || item("readingEntry-8") ||
        item("readingEntry-9") || !item("readingEntry-12")) {
      qCritical() << "Reading state" << preserved() << rejected << spoken
                  << group << title
                  << (rejected ? rejected->property("text") : QVariant{})
                  << (title ? title->property("text") : QVariant{})
                  << (group ? group->height() : 0) << item("readingEntry-12");
      fail("Grouped reading, rejected source, authored notes or geometry "
           "failed");
      return;
    }
    window_->grabWindow().save(output_ + "/reading-light-zh.png");
    preferences_.setLanguage("en");
    preferences_.setTheme("dark");
    QTimer::singleShot(200, this, [this] {
      if (!preserved() ||
          item("formatChoice")->property("currentIndex").toInt() != 1 ||
          item("defaultVoice")->property("currentIndex").toInt() != 5 ||
          item("readingRepeatTitle-7")->property("text").toString() !=
              "Repeat 2×") {
        fail("Live reading translation changed source or stayed stale");
        return;
      }
      window_->grabWindow().save(output_ + "/reading-dark-en.png");
      auto *edit = item("readingEdit-5");
      if (!edit || !QMetaObject::invokeMethod(edit, "clicked")) {
        fail("Reading source navigation is missing");
        return;
      }
      QTimer::singleShot(150, this, [this] { inspectNavigation(); });
    });
  }
  void inspectNavigation() {
    auto *editor = item("scriptEditor");
    const auto expected = source_.indexOf(QString::fromUtf8("你好，Vox。"));
    if (!preserved() || window_->property("reading").toBool() || !editor ||
        editor->property("cursorPosition").toInt() != expected ||
        !editor->hasActiveFocus()) {
      fail("Reading-to-source line navigation changed data or missed its "
           "target");
      return;
    }
    stage_ = 4;
    source_ = "<b>Literal plain narration</b>\nA second paragraph.";
    backend_.editText(source_);
    window_->setProperty("reading", true);
    QTimer::singleShot(0, this, [this] {
      if (item("scriptReader")->property("current").toBool() ||
          (item("readingText-10") && item("readingText-10")->isVisible())) {
        fail("A prior-source preview remained visible after editing");
        return;
      }
      backend_.setFormat("plain");
      backend_.save();
    });
  }
  void inspectPlain() {
    auto *text = item("readingText-1");
    if (!preserved() || !text || text->property("text").toString() != source_ ||
        text->property("textFormat").toInt() != Qt::PlainText) {
      fail("Plain reading did not retain literal authored text");
      return;
    }
    window_->grabWindow().save(output_ + "/reading-plain.png");
    qInfo() << "Grouped reading, invalid-source visibility, authored notes, "
               "live themes/languages, source navigation and exact persistence "
               "passed";
    app_.quit();
  }
  QGuiApplication &app_;
  UiPreferences &preferences_;
  DesktopBackend &backend_;
  QString output_, path_, source_;
  QByteArray before_;
  QQuickWindow *window_;
  int stage_ = 0;
};
} // namespace
void startReadingSmoke(QGuiApplication &app, QQmlApplicationEngine &engine,
                       UiPreferences &preferences, DesktopBackend &backend,
                       const QString &output) {
  new ReadingSmoke(app, engine, preferences, backend, output);
}
