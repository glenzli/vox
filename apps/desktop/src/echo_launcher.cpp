#include "echo_launcher.hpp"
#include <QFile>
#include <QFileInfo>
#include <QJsonDocument>
#include <QJsonObject>
#include <QProcessEnvironment>
EchoLauncher::EchoLauncher(QObject *parent) : QObject(parent) {
  m_deadline.setSingleShot(true);
  m_deadline.setInterval(5000);
  connect(&m_deadline, &QTimer::timeout, this, [this] {
    m_timedOut = true;
    m_process.kill();
  });
  connect(&m_process, &QProcess::finished, this,
          [this](int code, QProcess::ExitStatus status) {
            m_deadline.stop();
            const auto bytes = m_process.readAllStandardOutput();
            const auto capability =
                bytes.size() <= 512 * 1024
                    ? QJsonDocument::fromJson(bytes).object()
                    : QJsonObject{};
            if (m_timedOut || status != QProcess::NormalExit || code != 0 ||
                !capability["ok"].toBool() ||
                capability["schema_version"].toInt() != 1 ||
                capability["command"].toString() != "capabilities" ||
                capability["session_owner"].toString() !=
                    "exclusive_tool_process" ||
                capability["discovery_creates_session"].toBool(true)) {
              emit failed("echo_unsupported");
              return;
            }
            if (QProcess::startDetached(m_binary, {"--edit", m_source},
                                        QFileInfo(m_source).absolutePath()))
              emit opened();
            else
              emit failed("echo_unavailable");
          });
  connect(&m_process, &QProcess::errorOccurred, this,
          [this](QProcess::ProcessError error) {
            if (error == QProcess::FailedToStart) {
              m_deadline.stop();
              emit failed("echo_unavailable");
            }
          });
}
EchoLauncher::~EchoLauncher() {
  disconnect(&m_process, nullptr, this, nullptr);
  if (busy()) {
    m_process.kill();
    m_process.waitForFinished(1000);
  }
}
void EchoLauncher::open(const QString &binary, const QString &workingCopy) {
  if (busy()) {
    emit failed("echo_busy");
    return;
  }
  QFile executable(binary);
  if (!QFileInfo(binary).isExecutable() ||
      !executable.open(QIODevice::ReadOnly) ||
      executable.size() > 512 * 1024 * 1024) {
    emit failed("echo_unavailable");
    return;
  }
  // Old Echo builds interpret unknown flags as Catalog paths. Never execute
  // that entry.
  bool marker = false;
  QByteArray tail;
  while (!executable.atEnd()) {
    auto bytes = tail + executable.read(64 * 1024);
    if (bytes.contains("--editor-stdio")) {
      marker = true;
      break;
    }
    tail = bytes.right(32);
  }
  if (!marker || !m_probeDirectory.isValid()) {
    emit failed("echo_unsupported");
    return;
  }
  m_binary = QFileInfo(binary).canonicalFilePath();
  m_source = workingCopy;
  m_timedOut = false;
  auto environment = QProcessEnvironment::systemEnvironment();
  environment.insert("QT_QPA_PLATFORM", "offscreen");
  m_process.setProcessEnvironment(environment);
  m_process.setWorkingDirectory(m_probeDirectory.path());
  m_process.setProgram(m_binary);
  m_process.setArguments({"--editor-stdio", "--capabilities"});
  m_process.start();
  m_deadline.start();
}
