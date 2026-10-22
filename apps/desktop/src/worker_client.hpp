#pragma once
#include <QJsonObject>
#include <QObject>
#include <QProcess>
#include <QQueue>
#include <functional>

// Owns the complete correlated subprocess request lifecycle; no project
// mutation policy.
class WorkerClient final : public QObject {
  Q_OBJECT
public:
  using Callback = std::function<void(const QJsonObject &)>;
  explicit WorkerClient(QObject *parent = nullptr);
  ~WorkerClient() override;
  void request(std::function<QJsonObject()> build, Callback done,
               bool cancellable = false);
  void cancel();
signals:
  void progress(int completed, int total);
  void progressDetail(const QJsonObject &detail);
  void diagnostic(const QString &code);

private:
  struct Request {
    std::function<QJsonObject()> build;
    Callback done;
    QString id;
    bool cancellable = false, cancelled = false;
  };
  QProcess m_process;
  QQueue<Request> m_queue;
  QByteArray m_buffer;
  bool m_running = false;
  quint64 m_sequence = 0;
  void pump();
  void receive();
  void failed();
};
