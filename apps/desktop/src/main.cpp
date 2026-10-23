#include "desktop_backend.hpp"
#include "desktop_delivery.hpp"
#include "project_open_filter.hpp"
#include "reference_recorder.hpp"
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
      app.arguments().contains("--smoke-studio") ||
      app.arguments().contains("--smoke-reading") ||
      app.arguments().contains("--smoke-production") ||
      app.arguments().contains("--smoke-sound-workspace") ||
      app.arguments().contains("--smoke-longform") ||
      app.arguments().contains("--smoke-delivery") ||
      app.arguments().contains("--smoke-playback") ||
      app.arguments().contains("--smoke-audio") ||
      app.arguments().contains("--smoke-shape-transfer")) {
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
  const int preparedDeliveryIndex = app.arguments().indexOf("--smoke-delivery");
  if (preparedDeliveryIndex >= 0 &&
      preparedDeliveryIndex + 1 < app.arguments().size()) {
    const auto worker =
        prepareDeliverySmoke(app.arguments()[preparedDeliveryIndex + 1]);
    if (worker.isEmpty())
      return 32;
    qputenv("VOX_CLI_BINARY", worker.toUtf8());
  }
  DesktopBackend backend;
  ReferenceRecorder recorder;
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
    backend.retranslate();
    static_cast<DesktopDelivery *>(backend.delivery())->retranslate();
  };
  QObject::connect(&preferences, &UiPreferences::languageChanged, &app,
                   translate);
  translate();
  engine.rootContext()->setContextProperty("backend", &backend);
  engine.rootContext()->setContextProperty("recorder", &recorder);
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
  const int transferIndex = arguments.indexOf("--import-shape-transfer");
  const int expectedIndex = arguments.indexOf("--expected-blake3");
  if (transferIndex >= 0) {
    if (transferIndex + 1 >= arguments.size() || projectIndex < 0 ||
        projectIndex + 1 >= arguments.size() ||
        (expectedIndex >= 0 && expectedIndex + 1 >= arguments.size())) {
      qCritical()
          << "--import-shape-transfer requires TRANSFER and --project NEW.vox";
      return 64;
    }
    QTimer::singleShot(
        0, &backend,
        [&backend, transfer = arguments[transferIndex + 1],
         path = arguments[projectIndex + 1],
         expected = expectedIndex >= 0 && expectedIndex + 1 < arguments.size()
                        ? arguments[expectedIndex + 1]
                        : QString{}] {
          backend.importShapeTransfer(
              QUrl::fromLocalFile(QFileInfo(transfer).absoluteFilePath()),
              QUrl::fromLocalFile(QFileInfo(path).absoluteFilePath()),
              expected);
        });
  } else if (projectIndex >= 0 && projectIndex + 1 < arguments.size())
    QTimer::singleShot(0, &backend,
                       [&backend, path = arguments[projectIndex + 1]] {
                         backend.openProject(QUrl::fromLocalFile(path));
                       });
  else if (arguments.size() > 1 && arguments[1].endsWith(".vox"))
    QTimer::singleShot(0, &backend, [&backend, path = arguments[1]] {
      backend.openProject(
          QUrl::fromLocalFile(QFileInfo(path).absoluteFilePath()));
    });
  const int shapeSmokeIndex = arguments.indexOf("--smoke-shape-transfer");
  if (shapeSmokeIndex >= 0 && shapeSmokeIndex + 1 < arguments.size())
    startShapeTransferSmoke(app, engine, backend,
                            arguments[shapeSmokeIndex + 1]);
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
  const int studioIndex = arguments.indexOf("--smoke-studio");
  if (studioIndex >= 0 && studioIndex + 1 < arguments.size())
    startStudioSmoke(app, engine, preferences, backend,
                     arguments[studioIndex + 1]);
  const int readingIndex = arguments.indexOf("--smoke-reading");
  if (readingIndex >= 0 && readingIndex + 1 < arguments.size())
    startReadingSmoke(app, engine, preferences, backend,
                      arguments[readingIndex + 1]);
  const int productionIndex = arguments.indexOf("--smoke-production");
  if (productionIndex >= 0 && productionIndex + 1 < arguments.size())
    startProductionSmoke(app, engine, preferences, backend,
                         arguments[productionIndex + 1]);
  const int soundIndex = arguments.indexOf("--smoke-sound-workspace");
  if (soundIndex >= 0 && soundIndex + 1 < arguments.size())
    startSoundWorkspaceSmoke(app, engine, preferences, backend,
                             arguments[soundIndex + 1]);
  const int longformIndex = arguments.indexOf("--smoke-longform");
  if (longformIndex >= 0 && longformIndex + 1 < arguments.size())
    startLongformSmoke(app, engine, preferences, backend,
                       arguments[longformIndex + 1]);
  const int deliveryIndex = arguments.indexOf("--smoke-delivery");
  if (deliveryIndex >= 0 && deliveryIndex + 1 < arguments.size())
    startDeliverySmoke(app, engine, preferences, backend,
                       arguments[deliveryIndex + 1]);
  const int playbackIndex = arguments.indexOf("--smoke-playback");
  if (playbackIndex >= 0 && playbackIndex + 1 < arguments.size())
    startPlaybackSmoke(app, engine, preferences, backend,
                       arguments[playbackIndex + 1]);
  return app.exec();
}
