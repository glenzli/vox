#include "desktop_backend.hpp"
#include "script_editor.hpp"
#include "ui_preferences.hpp"
#include "workflow_smoke.hpp"
#include <QAbstractTextDocumentLayout>
#include <QDir>
#include <QFile>
#include <QGuiApplication>
#include <QJsonDocument>
#include <QQmlApplicationEngine>
#include <QQuickWindow>
#include <QTextBlock>
#include <QTextCursor>
#include <QTextLayout>
#include <QTimer>
#include <QUuid>

namespace {
QTextCharFormat formatAt(QTextDocument *document, int position) {
  const auto block = document->findBlock(position);
  for (const auto &range : block.layout()->formats())
    if (position - block.position() >= range.start &&
        position - block.position() < range.start + range.length)
      return range.format;
  return {};
}

class EditorSmoke final : public QObject {
public:
  EditorSmoke(QGuiApplication &app, QQmlApplicationEngine &engine,
              UiPreferences &preferences, DesktopBackend &backend,
              const QString &output)
      : QObject(engine.rootObjects().first()), app_(app),
        preferences_(preferences), backend_(backend), output_(output),
        window_(qobject_cast<QQuickWindow *>(engine.rootObjects().first())) {
    editor_ = window_->findChild<QQuickItem *>("scriptEditor");
    gutter_ = window_->findChild<LineNumberGutter *>("lineNumberGutter");
    highlighter_ = window_->findChild<ScriptHighlighter *>("scriptHighlighter");
    if (!editor_ || !gutter_ || !highlighter_ || !QDir().mkpath(output_)) {
      fail("Editor components not packaged");
      return;
    }
    document_ = highlighter_->document();
    script_ =
        QString::fromUtf8(
            "[production: dialogue]\n[role: Guide; language: auto]\n"
            "[定义角色：旁白；语言：自动]\n[备注：此行不朗读]\n# A quiet "
            "scene\n"
            "[speaker: Guide]\n") +
        QString::fromUtf8("这是一个用于检查自动换行的长段落。 ").repeated(12) +
        "\n[pause: 0.4s]\n[repeat: 2; gap: 1s]\nHello, Vox.\n[end-repeat]\n"
        "Literal \\[text\\]. 😀\n";
    for (int i = 0; i < 120; ++i)
      script_ += QString("A quiet sentence %1.\n").arg(i + 1);
    connect(&backend_, &DesktopBackend::requestFinished, this,
            [this](const QString &action, bool ok) {
              if (action == "create") {
                if (reopening_ && ok) {
                  backend_.openProject(QUrl::fromLocalFile(path_));
                  return;
                }
                if (!ok) {
                  fail("Editor fixture creation failed");
                  return;
                }
                backend_.setFormat("production_script");
                editor_->setProperty("text", script_);
                preferences_.setTheme("light");
                preferences_.setLanguage("zh_CN");
                preferences_.setLineNumbers(true);
                QTimer::singleShot(350, this, [this] { firstStage(); });
              } else if (saving_ && action == "save" && ok &&
                         !backend_.dirty()) {
                finish();
              } else if (reopening_ && action == "open" && ok) {
                QTimer::singleShot(1200, this, [this] { finish(); });
              }
            });
    path_ = output_ + "/fixture-" +
            QUuid::createUuid().toString(QUuid::WithoutBraces) + ".vox";
    backend_.createProject(QUrl::fromLocalFile(path_));
    QTimer::singleShot(20000, this, [this] { fail("Editor check timed out"); });
  }

private:
  void finish() {
    QFile manifest(path_ + "/project.json");
    if (!manifest.open(QIODevice::ReadOnly) ||
        QJsonDocument::fromJson(manifest.readAll())
                .object()["draft"]
                .toObject()["text"]
                .toString() != script_) {
      fail("Editor persistence changed source bytes");
      return;
    }
    if (!reopening_) {
      reopening_ = true;
      saving_ = false;
      backend_.createProject(QUrl::fromLocalFile(path_ + ".empty.vox"));
      return;
    }
    if (editor_->property("text").toString() != script_ || backend_.dirty()) {
      fail("Loading an existing script fed back an edit");
      return;
    }
    qInfo() << "Script highlighting, wrapped/scrolling line numbers, theme "
               "changes, plain mode, selection, undo, exact persistence and "
               "reopen passed";
    app_.quit();
  }
  void fail(const char *message) {
    qCritical() << message;
    app_.exit(15);
  }

  bool sourceAndFormats() {
    const auto colors = highlighter_->colors();
    return document_->toPlainText() == script_ &&
           gutter_->lineCount() == script_.count('\n') + 1 &&
           formatAt(document_, script_.indexOf("production"))
                   .foreground()
                   .color() == colors["command"].value<QColor>() &&
           formatAt(document_, script_.indexOf("定义角色"))
                   .foreground()
                   .color() == colors["command"].value<QColor>() &&
           formatAt(document_, script_.indexOf("Guide")).foreground().color() ==
               colors["identity"].value<QColor>() &&
           formatAt(document_, script_.indexOf("备注")).fontItalic() &&
           !formatAt(document_, script_.indexOf("Hello"))
                .hasProperty(QTextFormat::ForegroundBrush) &&
           !formatAt(document_, script_.indexOf("Literal"))
                .hasProperty(QTextFormat::ForegroundBrush);
  }

  bool gutterAt(const QImage &image, int number, bool expectInk = true,
                int visualRow = 0) {
    const auto block = document_->findBlockByNumber(number - 1);
    QRectF caret;
    QMetaObject::invokeMethod(editor_, "positionToRectangle",
                              Q_RETURN_ARG(QRectF, caret),
                              Q_ARG(int, block.position()));
    const auto textTop =
        editor_->mapToScene(caret.topLeft()).y() + visualRow * caret.height();
    const auto origin = gutter_->mapToScene(QPointF());
    const auto predictedTop =
        origin.y() +
        document_->documentLayout()->blockBoundingRect(block).top() +
        gutter_->textTopPadding() - gutter_->scrollY();
    if (visualRow == 0 && qAbs(textTop - predictedTop) > 1.5) {
      qCritical() << "Line geometry differs from caret" << number << textTop
                  << predictedTop;
      return false;
    }
    const auto scale = double(image.width()) / window_->width();
    const auto rect =
        QRectF((origin.x() + 5) * scale, textTop * scale,
               (gutter_->width() - 11) * scale, caret.height() * scale)
            .toAlignedRect()
            .intersected(image.rect());
    const auto ink = gutter_->colors()["secondary"].value<QColor>();
    int count = 0;
    for (int y = rect.top(); y <= rect.bottom(); ++y)
      for (int x = rect.left(); x <= rect.right(); ++x) {
        const auto pixel = image.pixelColor(x, y);
        if (qAbs(pixel.redF() - ink.redF()) < .12 &&
            qAbs(pixel.greenF() - ink.greenF()) < .12 &&
            qAbs(pixel.blueF() - ink.blueF()) < .12)
          ++count;
      }
    if ((count >= 3) != expectInk)
      qCritical() << "Gutter ink mismatch" << number << visualRow << count;
    return (count >= 3) == expectInk;
  }

  void firstStage() {
    auto image = window_->grabWindow();
    image.save(output_ + "/light.png");
    const auto wrapped = document_->findBlockByNumber(6);
    if (!sourceAndFormats() || wrapped.layout()->lineCount() < 2 ||
        !gutterAt(image, 2) || !gutterAt(image, 7, false, 1) ||
        !gutterAt(image, 8)) {
      fail("Script colors or wrapped source line numbers failed");
      return;
    }
    QMetaObject::invokeMethod(editor_, "selectAll");
    if (editor_->property("selectedText").toString() != script_) {
      fail("Selection includes formatting or line numbers");
      return;
    }
    QMetaObject::invokeMethod(editor_, "deselect");
    document_->clearUndoRedoStacks();
    highlighter_->setProductionScript(false);
    highlighter_->rehighlight();
    if (document_->isUndoAvailable() ||
        !document_->firstBlock().layout()->formats().isEmpty()) {
      fail("Plain narration retained script formatting or changed undo");
      return;
    }
    highlighter_->setProductionScript(true);
    QTextCursor cursor(document_);
    cursor.movePosition(QTextCursor::End);
    cursor.insertText("\nA temporary edit.");
    document_->undo();
    preferences_.setLineNumbers(false);
    if (gutter_->isVisible() || UiPreferences().lineNumbers()) {
      fail("Line-number preference did not persist");
      return;
    }
    preferences_.setLineNumbers(true);
    preferences_.setTheme("dark");
    editor_->setProperty("cursorPosition", 0);
    QTimer::singleShot(350, this, [this] {
      const auto image = window_->grabWindow();
      image.save(output_ + "/dark.png");
      if (!sourceAndFormats() || !gutterAt(image, 2)) {
        fail("Live theme or undo altered script formatting/text");
        return;
      }
      auto *scroll = window_->findChild<QObject *>("scriptScrollView");
      auto *flickable = scroll->property("contentItem").value<QQuickItem *>();
      const auto block = document_->findBlockByNumber(79);
      flickable->setProperty(
          "contentY",
          document_->documentLayout()->blockBoundingRect(block).top());
      QTimer::singleShot(350, this, [this] {
        const auto image = window_->grabWindow();
        image.save(output_ + "/scrolled.png");
        if (!gutterAt(image, 80) || !sourceAndFormats()) {
          fail("Line numbers drifted after scrolling");
          return;
        }
        saving_ = true;
        if (backend_.dirty())
          backend_.save();
        else
          finish();
      });
    });
  }

  QGuiApplication &app_;
  UiPreferences &preferences_;
  DesktopBackend &backend_;
  QString output_, path_, script_;
  QQuickWindow *window_;
  QQuickItem *editor_ = nullptr;
  LineNumberGutter *gutter_ = nullptr;
  ScriptHighlighter *highlighter_ = nullptr;
  QTextDocument *document_ = nullptr;
  bool saving_ = false;
  bool reopening_ = false;
};
} // namespace

void startEditorSmoke(QGuiApplication &app, QQmlApplicationEngine &engine,
                      UiPreferences &preferences, DesktopBackend &backend,
                      const QString &path) {
  new EditorSmoke(app, engine, preferences, backend, path);
}
