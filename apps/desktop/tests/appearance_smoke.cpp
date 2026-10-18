#include "desktop_backend.hpp"
#include "ui_preferences.hpp"
#include "workflow_smoke.hpp"
#include <QDebug>
#include <QDir>
#include <QGuiApplication>
#include <QImage>
#include <QQmlApplicationEngine>
#include <QQuickItem>
#include <QQuickWindow>
#include <QTimer>
#include <QUuid>
#include <cmath>

#ifdef Q_OS_MACOS
bool verifyMacTitleBar(QQuickWindow *window);
#endif

namespace {
double luminance(const QColor &color) {
  auto linear = [](double value) {
    return value <= 0.04045 ? value / 12.92
                            : std::pow((value + 0.055) / 1.055, 2.4);
  };
  return 0.2126 * linear(color.redF()) + 0.7152 * linear(color.greenF()) +
         0.0722 * linear(color.blueF());
}

double contrast(const QColor &a, const QColor &b) {
  const auto first = luminance(a), second = luminance(b);
  return (std::max(first, second) + 0.05) / (std::min(first, second) + 0.05);
}

bool visibleForeground(QQuickWindow *window, const QImage &image,
                       QQuickItem *item, const QColor &background,
                       double minimum) {
  if (!item || !item->isVisible())
    return false;
  const auto bounds = item->mapRectToScene(item->boundingRect());
  const auto scale = double(image.width()) / window->width();
  const auto pixels = QRectF(bounds.x() * scale, bounds.y() * scale,
                             bounds.width() * scale, bounds.height() * scale)
                          .toAlignedRect()
                          .intersected(image.rect());
  int visible = 0;
  for (int y = pixels.top(); y <= pixels.bottom(); ++y)
    for (int x = pixels.left(); x <= pixels.right(); ++x)
      if (contrast(image.pixelColor(x, y), background) >= minimum)
        ++visible;
  return visible >= 5;
}

class AppearanceSmoke final : public QObject {
public:
  AppearanceSmoke(QGuiApplication &app, QQmlApplicationEngine &engine,
                  UiPreferences &preferences, DesktopBackend &backend,
                  const QString &output)
      : QObject(engine.rootObjects().first()), app_(app),
        preferences_(preferences), backend_(backend), output_(output),
        window_(qobject_cast<QQuickWindow *>(engine.rootObjects().first())) {
    if (!window_ || !QDir().mkpath(output_)) {
      fail("Appearance check has no window or output directory");
      return;
    }
    connect(&backend_, &DesktopBackend::requestFinished, this,
            [this](const QString &action, bool ok) {
              if (action != "create")
                return;
              if (!ok) {
                fail("Appearance fixture creation failed");
                return;
              }
              auto *editor = window_->findChild<QObject *>("scriptEditor");
              if (!editor) {
                fail("Missing script editor in appearance fixture");
                return;
              }
              editor->setProperty("text", "A visible voice.\n你好，Vox。\n");
              advance();
            });
    QTimer::singleShot(20000, this,
                       [this] { fail("Appearance check timed out"); });
    backend_.createProject(QUrl::fromLocalFile(
        output_ + "/fixture-" +
        QUuid::createUuid().toString(QUuid::WithoutBraces) + ".vox"));
  }

private:
  void fail(const char *message) {
    qCritical() << message << "stage" << stage_;
    app_.exit(14);
  }

  void advance() {
    if (stage_ == 8) {
      qInfo()
          << "Rendered button text/icons passed light/dark, active/inactive, "
             "disabled and resized-window checks";
      app_.quit();
      return;
    }
    const bool dark = stage_ == 2 || stage_ == 3 || stage_ == 5;
    const bool inactive = stage_ == 1 || stage_ == 3;
    const bool disabled = stage_ == 4 || stage_ == 5;
    preferences_.setTheme(dark ? "dark" : "light");
    preferences_.setLanguage(dark ? "en" : "zh_CN");
    for (const auto *name :
         {"writeButton", "readButton", "renderButton", "playButton"}) {
      auto *button = window_->findChild<QObject *>(name);
      if (!button) {
        fail("Missing button in appearance fixture");
        return;
      }
      button->setProperty("enabled", !disabled);
    }
    if (stage_ == 6)
      window_->showMaximized();
    else if (stage_ == 7) {
      window_->showNormal();
      window_->resize(1160, 780);
    }
    if (native()) {
      if (inactive) {
        decoy_.setTitle("Vox appearance check");
        decoy_.resize(120, 80);
        decoy_.show();
        decoy_.requestActivate();
      } else {
        decoy_.hide();
        window_->requestActivate();
      }
    }
    QTimer::singleShot(450, this, [this, dark, inactive, disabled] {
      if (native() && window_->isActive() == inactive) {
        fail("Could not exercise the requested window activation state");
        return;
      }
      auto image = window_->grabWindow();
      if (image.isNull()) {
        fail("No rendered appearance image");
        return;
      }
      image.save(output_ + "/stage-" + QString::number(stage_) + ".png");
      for (const auto *name : {"writeButton", "renderButton", "playButton"}) {
        auto *button = window_->findChild<QObject *>(name);
        auto *background = button->property("background").value<QQuickItem *>();
        auto *content = button->property("contentItem").value<QQuickItem *>();
        if (!background || !content) {
          fail("Missing rendered button contents");
          return;
        }
        const auto color = background->property("color").value<QColor>();
        const auto threshold = disabled ? 3.0 : 4.5;
        auto *label =
            button->property("text").toString().isEmpty() ? nullptr : content;
        if (label &&
            (contrast(label->property("color").value<QColor>(), color) <
                 threshold ||
             !visibleForeground(window_, image, label, color, threshold))) {
          qCritical() << "Unreadable rendered button label" << name << dark
                      << "foreground" << button->property("foregroundColor")
                      << "label" << label->property("color") << "background"
                      << color;
          fail("Button text contrast failed");
          return;
        }
        QQuickItem *symbol = nullptr;
        for (auto *child : content->findChildren<QQuickItem *>())
          if (child->inherits("QQuickIconImage")) {
            symbol = child;
            break;
          }
        if (button->property("icon").isValid() && !symbol && label == nullptr) {
          fail("Button icon was not instantiated");
          return;
        }
        if (symbol && symbol->isVisible() &&
            !visibleForeground(window_, image, symbol, color,
                               disabled ? 3.0 : 4.5)) {
          qCritical() << "Unreadable rendered button symbol" << name << dark;
          fail("Button icon contrast failed");
          return;
        }
      }
#ifdef Q_OS_MACOS
      if (native() && !verifyMacTitleBar(window_)) {
        fail("Native titlebar geometry failed");
        return;
      }
#endif
      ++stage_;
      advance();
    });
  }

  bool native() const { return QGuiApplication::platformName() == "cocoa"; }
  QGuiApplication &app_;
  UiPreferences &preferences_;
  DesktopBackend &backend_;
  QString output_;
  QQuickWindow *window_;
  QWindow decoy_;
  int stage_ = 0;
};
} // namespace

void startAppearanceSmoke(QGuiApplication &app, QQmlApplicationEngine &engine,
                          UiPreferences &preferences, DesktopBackend &backend,
                          const QString &path) {
  new AppearanceSmoke(app, engine, preferences, backend, path);
}
