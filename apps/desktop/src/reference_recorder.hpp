#pragma once
#include <QAudioSource>
#include <QElapsedTimer>
#include <QMediaDevices>
#include <QObject>
#include <QTemporaryDir>
#include <QTimer>
#include <QUrl>
#include <QVariantList>
#include <memory>

// Owns microphone permission, bounded local capture and disposable preview
// bytes.
class ReferenceRecorder final : public QObject {
  Q_OBJECT
  Q_PROPERTY(bool recording READ recording NOTIFY changed)
  Q_PROPERTY(bool requesting READ requesting NOTIFY changed)
  Q_PROPERTY(int milliseconds READ milliseconds NOTIFY changed)
  Q_PROPERTY(double level READ level NOTIFY changed)
  Q_PROPERTY(QUrl file READ file NOTIFY changed)
  Q_PROPERTY(QString error READ error NOTIFY changed)
  Q_PROPERTY(QVariantList devices READ devices NOTIFY devicesChanged)
public:
  explicit ReferenceRecorder(QObject *parent = nullptr);
  ~ReferenceRecorder() override;
  bool recording() const { return bool(m_source); }
  bool requesting() const { return m_requesting; }
  int milliseconds() const { return m_milliseconds; }
  double level() const { return m_level; }
  QUrl file() const { return m_file; }
  QString error() const { return m_error; }
  QVariantList devices() const;
  Q_INVOKABLE void start(int deviceIndex = 0);
  Q_INVOKABLE void stop();
  Q_INVOKABLE void cancel();
signals:
  void changed();
  void devicesChanged();

private:
  friend class ReferenceRecorderSmoke;
  void finalizeCapture();
  QMediaDevices m_devices;
  QTemporaryDir m_directory;
  std::unique_ptr<QAudioSource> m_source;
  QIODevice *m_input = nullptr;
  QAudioFormat m_format;
  QByteArray m_pcm, m_partial;
  QTimer m_timer;
  QElapsedTimer m_elapsed;
  QUrl m_file;
  QString m_error;
  int m_milliseconds = 0;
  double m_level = 0;
  quint64 m_generation = 0;
  bool m_requesting = false;
  void begin(int deviceIndex, quint64 generation);
  void drain();
};
