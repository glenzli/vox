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
#include <QTimer>
#include <QUuid>
#include <memory>

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
        if (*stage == 1 && action == "save" && !ok) {
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
          qInfo() << "Vox UI, localization, appearance, persistence and "
                     "queued-project fence passed";
          app.quit();
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
