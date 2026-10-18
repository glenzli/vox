// Adapted from Shape, MIT; see docs/SHAPE_PROVENANCE.json.
#import <AppKit/AppKit.h>

#include "mac_titlebar.hpp"

#include <QEvent>
#include <QGuiApplication>
#include <QObject>
#include <QPointer>
#include <QTimer>
#include <QWindow>

#include <array>

namespace {

class MacTitleBarAlignment final : public QObject {
public:
  MacTitleBarAlignment(QWindow *window, const int title_bar_height)
      : QObject(window), window_(window), title_bar_height_(title_bar_height) {
    window_->installEventFilter(this);
    settled_alignment_.setSingleShot(true);
    settled_alignment_.setInterval(80);
    connect(&settled_alignment_, &QTimer::timeout, this,
            [this] { alignNativeButtons(); });
    connect(window_, &QWindow::screenChanged, this,
            [this] { scheduleAlignment(); });
    connect(window_, &QWindow::activeChanged, this,
            [this] { scheduleAlignment(); });
    scheduleAlignment();
  }

protected:
  bool eventFilter(QObject *watched, QEvent *event) override {
    if (watched == window_) {
      switch (event->type()) {
      case QEvent::Show:
      case QEvent::Expose:
      case QEvent::Resize:
      case QEvent::WindowStateChange:
      case QEvent::ThemeChange:
        scheduleAlignment();
        break;
      default:
        break;
      }
    }
    return QObject::eventFilter(watched, event);
  }

private:
  void scheduleAlignment() {
    // Cocoa can lay out the standard buttons again after Qt's queued update.
    // Reapply once after the window has settled, coalescing resize/appearance
    // events.
    settled_alignment_.start();
    if (alignment_pending_) {
      return;
    }
    alignment_pending_ = true;
    QTimer::singleShot(0, this, [this] {
      alignment_pending_ = false;
      alignNativeButtons();
    });
  }

  void alignNativeButtons() const {
    if (!window_ || !window_->isVisible() ||
        window_->visibility() == QWindow::FullScreen) {
      return;
    }
    NSView *const qt_view = reinterpret_cast<NSView *>(window_->winId());
    NSWindow *const native_window = qt_view.window;
    if (native_window == nil) {
      return;
    }
    native_window.titleVisibility = NSWindowTitleHidden;
    native_window.titlebarAppearsTransparent = YES;

    const NSRect window_frame = native_window.frame;
    const NSPoint target_in_screen = NSMakePoint(
        NSMidX(window_frame),
        NSMaxY(window_frame) - static_cast<CGFloat>(title_bar_height_) / 2.0);
    const NSPoint target_in_window =
        [native_window convertPointFromScreen:target_in_screen];

    constexpr std::array<NSWindowButton, 3> button_types = {
        NSWindowCloseButton,
        NSWindowMiniaturizeButton,
        NSWindowZoomButton,
    };
    for (const NSWindowButton button_type : button_types) {
      NSButton *const button = [native_window standardWindowButton:button_type];
      NSView *const container = button.superview;
      if (button == nil || container == nil) {
        continue;
      }
      const NSPoint target_in_container =
          [container convertPoint:target_in_window fromView:nil];
      NSRect frame = button.frame;
      frame.origin.y = target_in_container.y - NSHeight(frame) / 2.0;
      [button setFrameOrigin:frame.origin];
    }
  }

  QPointer<QWindow> window_;
  int title_bar_height_ = 0;
  bool alignment_pending_ = false;
  QTimer settled_alignment_;
};

} // namespace

void installMacTitleBarAlignment(QWindow *window, const int title_bar_height) {
  if (window == nullptr || title_bar_height <= 0 ||
      QGuiApplication::platformName() != QStringLiteral("cocoa")) {
    return;
  }
  new MacTitleBarAlignment(window, title_bar_height);
}
