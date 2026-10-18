#include "desktop_backend.hpp"
#include "project_open_filter.hpp"
#include "ui_preferences.hpp"
#include "workflow_smoke.hpp"
#ifdef Q_OS_MACOS
#include "mac_titlebar.hpp"
#endif
#include <QDebug>
#include <QDir>
#include <QFile>
#include <QGuiApplication>
#include <QIcon>
#include <QJsonDocument>
#include <QQmlApplicationEngine>
#include <QQmlContext>
#include <QQuickStyle>
#include <QQuickWindow>
#include <QTimer>
#include <QTranslator>

int main(int argc, char **argv) {
  QGuiApplication app(argc, argv);
  app.setApplicationName("Vox");
  app.setOrganizationName("Vox");
  app.setOrganizationDomain("glenzli.dev");
  app.setApplicationVersion("0.1.0");
  app.setWindowIcon(QIcon(":/vox/branding/app-icon.png"));
  QQuickStyle::setStyle("Basic");
  if (app.arguments().contains("--smoke") ||
      app.arguments().contains("--smoke-exit") ||
      app.arguments().contains("--smoke-appearance") ||
      app.arguments().contains("--smoke-editor") ||
      app.arguments().contains("--smoke-audio")) {
    QSettings::setDefaultFormat(QSettings::IniFormat);
    QSettings::setPath(QSettings::IniFormat, QSettings::UserScope,
                       QDir::tempPath() + "/vox-ui-tests-" +
                           QUuid::createUuid().toString(QUuid::WithoutBraces));
  }
  UiPreferences preferences;
  for (const auto &option : QStringList{"--language", "--theme"}) {
    const auto index = app.arguments().indexOf(option);
    if (index >= 0 && index + 1 < app.arguments().size()) {
      if (option == "--language")
        preferences.setLanguage(app.arguments()[index + 1]);
      else
        preferences.setTheme(app.arguments()[index + 1]);
    }
  }
  QTranslator translator;
  DesktopBackend backend;
  const auto echoIndex = app.arguments().indexOf("--echo-binary");
  if (echoIndex >= 0 && echoIndex + 1 < app.arguments().size())
    backend.setEchoBinary(QUrl::fromLocalFile(app.arguments()[echoIndex + 1]));
  ProjectOpenFilter fileEvents(backend);
  app.installEventFilter(&fileEvents);
  QQmlApplicationEngine engine;
  auto translate = [&] {
    app.removeTranslator(&translator);
    auto language = preferences.language();
    if (language == "system")
      language = QLocale::system().name();
    if (language.startsWith("zh") &&
        translator.load(":/translations/vox_zh_CN.qm"))
      app.installTranslator(&translator);
    engine.retranslate();
  };
  QObject::connect(&preferences, &UiPreferences::languageChanged, &app,
                   translate);
  QObject::connect(&preferences, &UiPreferences::languageChanged, &backend,
                   &DesktopBackend::messageChanged);
  translate();
  engine.rootContext()->setContextProperty("backend", &backend);
  engine.rootContext()->setContextProperty("preferences", &preferences);
  QObject::connect(
      &engine, &QQmlApplicationEngine::objectCreationFailed, &app,
      [] { QCoreApplication::exit(1); }, Qt::QueuedConnection);
  engine.loadFromModule("Vox.Desktop", "Main");
  if (engine.rootObjects().isEmpty())
    return 1;
#ifdef Q_OS_MACOS
  auto *titleBar =
      engine.rootObjects().first()->findChild<QObject *>("titleToolBar");
  installMacTitleBarAlignment(
      qobject_cast<QQuickWindow *>(engine.rootObjects().first()),
      titleBar ? qRound(titleBar->property("height").toReal()) : 44);
#endif
  const auto arguments = app.arguments();
  const int projectIndex = arguments.indexOf("--project");
  if (projectIndex >= 0 && projectIndex + 1 < arguments.size())
    QTimer::singleShot(0, &backend,
                       [&backend, path = arguments[projectIndex + 1]] {
                         backend.openProject(QUrl::fromLocalFile(path));
                       });
  else if (arguments.size() > 1 && arguments[1].endsWith(".vox"))
    QTimer::singleShot(0, &backend, [&backend, path = arguments[1]] {
      backend.openProject(
          QUrl::fromLocalFile(QFileInfo(path).absoluteFilePath()));
    });
  const int snapshotIndex = arguments.indexOf("--snapshot");
  if (snapshotIndex >= 0 && snapshotIndex + 1 < arguments.size())
    QTimer::singleShot(2000, &app, [&] {
      auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
      if (!window || !window->grabWindow().save(arguments[snapshotIndex + 1]))
        app.exit(2);
    });
  if (arguments.contains("--smoke-exit"))
    QTimer::singleShot(snapshotIndex >= 0 ? 2500 : 900, &app,
                       &QCoreApplication::quit);
  const int smokeIndex = arguments.indexOf("--smoke");
  if (smokeIndex >= 0 && smokeIndex + 1 < arguments.size()) {
    startWorkflowSmoke(app, engine, preferences, backend,
                       arguments[smokeIndex + 1]);
  }
  const int audioSmokeIndex = arguments.indexOf("--smoke-audio");
  if (audioSmokeIndex >= 0 && audioSmokeIndex + 1 < arguments.size())
    QTimer::singleShot(0, &backend, [&] {
      startAudioSmoke(app, engine, backend, arguments[audioSmokeIndex + 1]);
    });
  const int appearanceIndex = arguments.indexOf("--smoke-appearance");
  if (appearanceIndex >= 0 && appearanceIndex + 1 < arguments.size())
    startAppearanceSmoke(app, engine, preferences, backend,
                         arguments[appearanceIndex + 1]);
  const int editorIndex = arguments.indexOf("--smoke-editor");
  if (editorIndex >= 0 && editorIndex + 1 < arguments.size())
    startEditorSmoke(app, engine, preferences, backend,
                     arguments[editorIndex + 1]);
  return app.exec();
}
