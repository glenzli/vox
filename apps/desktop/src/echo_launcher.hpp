#pragma once
#include <QObject>
#include <QProcess>
#include <QTemporaryDir>
#include <QTimer>

// Admits only Echo's independent-editor protocol, then launches its human
// editor.
class EchoLauncher final : public QObject {
  Q_OBJECT
public:
  explicit EchoLauncher(QObject *parent = nullptr);
  ~EchoLauncher() override;
  bool busy() const { return m_process.state() != QProcess::NotRunning; }
  void open(const QString &binary, const QString &workingCopy);
signals:
  void opened();
  void failed(const QString &code);

private:
  QProcess m_process;
  QTimer m_deadline;
  QTemporaryDir m_probeDirectory;
  QString m_binary, m_source;
  bool m_timedOut = false;
};
