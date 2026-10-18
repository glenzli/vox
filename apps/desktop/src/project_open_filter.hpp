#pragma once
#include "desktop_backend.hpp"
#include <QFileOpenEvent>
#include <QTimer>
class ProjectOpenFilter final : public QObject {
public:
  explicit ProjectOpenFilter(DesktopBackend &backend) : m_backend(backend) {}

protected:
  bool eventFilter(QObject *, QEvent *event) override {
    if (event->type() != QEvent::FileOpen)
      return false;
    const auto url = static_cast<QFileOpenEvent *>(event)->url();
    QTimer::singleShot(0, &m_backend,
                       [this, url] { m_backend.openProject(url); });
    return true;
  }

private:
  DesktopBackend &m_backend;
};
