#import <AppKit/AppKit.h>

#include <QDebug>
#include <QQuickItem>
#include <QQuickWindow>
#include <array>

bool verifyMacTitleBar(QQuickWindow *window) {
  NSView *view = reinterpret_cast<NSView *>(window->winId());
  NSWindow *native = view.window;
  auto *toolbar = window->findChild<QQuickItem *>("titleToolBar");
  auto *content = window->findChild<QQuickItem *>("titleBarContent");
  if (!native || !toolbar || !content ||
      !(native.styleMask & NSWindowStyleMaskFullSizeContentView) ||
      native.titleVisibility != NSWindowTitleHidden ||
      !native.titlebarAppearsTransparent ||
      qAbs(toolbar->mapToScene(QPointF()).y()) > 1) {
    qCritical() << "Native titlebar is not integrated with QML content";
    return false;
  }
  for (const auto type : std::array<NSWindowButton, 3>{
           NSWindowCloseButton, NSWindowMiniaturizeButton,
           NSWindowZoomButton}) {
    NSButton *button = [native standardWindowButton:type];
    if (!button || !button.superview || button.hidden) {
      qCritical() << "Missing native traffic-light button";
      return false;
    }
    NSRect inWindow = [button convertRect:button.bounds toView:nil];
    NSRect onScreen = [native convertRectToScreen:inWindow];
    const auto centerFromTop = NSMaxY(native.frame) - NSMidY(onScreen);
    if (qAbs(centerFromTop - toolbar->height() / 2) > 1.5 ||
        NSMaxX(inWindow) + 10 > content->mapToScene(QPointF()).x()) {
      qCritical() << "Native controls overlap or drift from the toolbar"
                  << centerFromTop;
      return false;
    }
  }
  return true;
}
