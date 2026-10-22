#include "worker_client.hpp"
#include <QCoreApplication>
#include <QDir>
#include <QJsonDocument>
#include <QTimer>

WorkerClient::WorkerClient(QObject *parent) : QObject(parent) {
  const auto override = qEnvironmentVariable("VOX_CLI_BINARY");
  m_process.setProgram(override.isEmpty()
                           ? QCoreApplication::applicationDirPath() + "/vox-cli"
                           : override);
  m_process.setArguments({"serve"});
  connect(&m_process, &QProcess::readyReadStandardOutput, this,
          &WorkerClient::receive);
  connect(&m_process, &QProcess::errorOccurred, this,
          [this](QProcess::ProcessError) { failed(); });
  connect(&m_process, &QProcess::finished, this,
          [this](int, QProcess::ExitStatus) { failed(); });
  connect(&m_process, &QProcess::started, this, &WorkerClient::pump);
  m_process.start();
}
WorkerClient::~WorkerClient() {
  disconnect(&m_process, nullptr, this, nullptr);
  cancel();
  m_queue.clear();
  if (m_process.state() != QProcess::NotRunning) {
    m_process.closeWriteChannel();
    if (!m_process.waitForFinished(1000)) {
      m_process.terminate();
      if (!m_process.waitForFinished(1000)) {
        m_process.kill();
        m_process.waitForFinished(1000);
      }
    }
  }
}
void WorkerClient::request(std::function<QJsonObject()> build, Callback done,
                           bool cancellable) {
  m_queue.enqueue({std::move(build), std::move(done),
                   QString::number(++m_sequence), cancellable});
  if (m_process.state() == QProcess::NotRunning) {
    failed();
    return;
  }
  pump();
}
void WorkerClient::pump() {
  if (m_running || m_queue.isEmpty() || m_process.state() != QProcess::Running)
    return;
  if (m_queue.head().cancelled) {
    auto current = m_queue.dequeue();
    current.done(QJsonObject{{"ok", false}, {"error", "request_cancelled"}});
    QTimer::singleShot(0, this, &WorkerClient::pump);
    return;
  }
  auto request = m_queue.head().build();
  request.insert("request_id", m_queue.head().id);
  m_running = true;
  m_process.write(QJsonDocument(request).toJson(QJsonDocument::Compact) + '\n');
}
void WorkerClient::cancel() {
  for (auto &request : m_queue)
    if (request.cancellable)
      request.cancelled = true;
  if (m_running && !m_queue.isEmpty() && m_queue.head().cancellable)
    m_process.write("{\"action\":\"cancel\",\"request_id\":\"cancel\"}\n");
}
void WorkerClient::receive() {
  m_buffer += m_process.readAllStandardOutput();
  if (m_buffer.size() > 20 * 1024 * 1024) {
    m_buffer.clear();
    failed();
    return;
  }
  while (m_buffer.contains('\n')) {
    const auto index = m_buffer.indexOf('\n');
    const auto line = m_buffer.left(index);
    m_buffer.remove(0, index + 1);
    QJsonParseError error;
    const auto value = QJsonDocument::fromJson(line, &error).object();
    if (error.error != QJsonParseError::NoError) {
      emit diagnostic("worker_protocol");
      continue;
    }
    if (value["request_id"].toString() == "cancel")
      continue;
    if (m_queue.isEmpty() ||
        value["request_id"].toString() != m_queue.head().id)
      continue;
    if (value["event"].toString() == "progress") {
      emit progress(value["completed"].toInt(), value["total"].toInt());
      continue;
    }
    auto current = m_queue.dequeue();
    m_running = false;
    current.done(value);
    QTimer::singleShot(0, this, &WorkerClient::pump);
  }
}
void WorkerClient::failed() {
  m_running = false;
  const auto pending = std::exchange(m_queue, {});
  for (const auto &request : pending)
    request.done(QJsonObject{{"ok", false}, {"error", "worker_unavailable"}});
  emit diagnostic("worker_unavailable");
}
