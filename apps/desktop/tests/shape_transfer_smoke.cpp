#include "desktop_backend.hpp"
#include "workflow_smoke.hpp"
#include <QDir>
#include <QGuiApplication>
#include <QQmlApplicationEngine>
#include <QQuickItem>
#include <QQuickWindow>
#include <QTimer>

namespace {
class ShapeTransferSmoke final : public QObject {
public:
  ShapeTransferSmoke(QGuiApplication &app, QQmlApplicationEngine &engine,
                     DesktopBackend &backend, const QString &output)
      : QObject(engine.rootObjects().first()), app_(app), backend_(backend),
        output_(output),
        window_(qobject_cast<QQuickWindow *>(engine.rootObjects().first())) {
    QDir().mkpath(output_);
    connect(&backend_, &DesktopBackend::requestFinished, this,
            [this](const QString &action, bool ok) {
              if (action != "import_shape_transfer" &&
                  action != "restore_shape_draft")
                return;
              if (!ok) {
                fail("Shape transfer operation failed");
                return;
              }
              QTimer::singleShot(120, this, [this] { inspect(); });
            });
    QTimer::singleShot(15000, this,
                       [this] { fail("Shape transfer opening timed out"); });
  }

private:
  QQuickItem *item(const QString &name) const {
    QList<QQuickItem *> pending{window_->contentItem()};
    while (!pending.isEmpty()) {
      auto *current = pending.takeLast();
      if (current->objectName() == name)
        return current;
      pending.append(current->childItems());
    }
    return nullptr;
  }
  void fail(const char *message) {
    window_->grabWindow().save(output_ + "/shape-transfer-failure.png");
    qCritical() << message << backend_.message();
    app_.exit(23);
  }
  void inspect() {
    const auto project = backend_.project();
    const auto receipt = project["shape_import"].toMap();
    const auto drafts = receipt["drafts"].toList();
    if (stage_ == 0) {
      if (backend_.path().isEmpty() || receipt.isEmpty() || drafts.isEmpty()) {
        fail("The native Shape import did not open its independent project");
        return;
      }
      sourceHead_ = project["source_head"].toString();
      audioHead_ = project["audio_head"].toString();
      for (const auto &draft : drafts) {
        const auto value = draft.toMap();
        const auto kind = value["kind"].toString();
        if (value["status"].toString() != "unaccepted" ||
            !item("shapeDraft-" + kind)) {
          fail("A preserved draft has no human-visible restore action");
          return;
        }
        if (kind == "speech")
          speech_ = value["document"].toMap();
        if (kind == "authoring")
          authoring_ = value["document"].toMap();
      }
      window_->grabWindow().save(output_ + "/shape-import-open.png");
      if (speech_.isEmpty()) {
        app_.quit();
        return;
      }
      stage_ = 1;
      if (!QMetaObject::invokeMethod(item("shapeDraft-speech"), "clicked"))
        fail("Speech draft restore action did not activate");
      return;
    }
    if (backend_.dirty() || project["source_head"].toString() != sourceHead_ ||
        project["audio_head"].toString() != audioHead_) {
      fail("Restoring a Shape draft changed accepted history");
      return;
    }
    if (stage_ == 1) {
      if (backend_.document() != speech_ ||
          backend_.importedDraftKind() != "speech") {
        fail("Speech source, role/cue settings or active-draft presentation "
             "changed");
        return;
      }
      window_->grabWindow().save(output_ + "/shape-speech-restored.png");
      if (authoring_.isEmpty()) {
        app_.quit();
        return;
      }
      stage_ = 2;
      if (!QMetaObject::invokeMethod(item("shapeDraft-authoring"), "clicked"))
        fail("Writing draft restore action did not activate");
    } else {
      if (backend_.document() != authoring_ ||
          backend_.importedDraftKind() != "authoring") {
        fail("Exact writing draft or active-draft presentation changed");
        return;
      }
      window_->grabWindow().save(output_ + "/shape-writing-restored.png");
      qInfo() << "Native Shape import/open and explicit draft restoration "
                 "preserved exact settings and accepted history";
      app_.quit();
    }
  }
  QGuiApplication &app_;
  DesktopBackend &backend_;
  QString output_, sourceHead_, audioHead_;
  QQuickWindow *window_;
  QVariantMap speech_, authoring_;
  int stage_ = 0;
};
} // namespace
void startShapeTransferSmoke(QGuiApplication &app,
                             QQmlApplicationEngine &engine,
                             DesktopBackend &backend, const QString &output) {
  new ShapeTransferSmoke(app, engine, backend, output);
}
