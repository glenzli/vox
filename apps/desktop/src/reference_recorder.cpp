#include "reference_recorder.hpp"
#include <QCoreApplication>
#include <QDataStream>
#include <QFile>
#include <QPermissions>
#include <QtEndian>
#include <algorithm>
#include <cmath>
#include <cstring>

ReferenceRecorder::ReferenceRecorder(QObject *parent) : QObject(parent) {
  connect(&m_devices, &QMediaDevices::audioInputsChanged, this,
          &ReferenceRecorder::devicesChanged);
  m_timer.setInterval(80);
  connect(&m_timer, &QTimer::timeout, this, [this] {
    if (!m_source)
      return;
    drain();
    m_milliseconds = int(m_elapsed.elapsed());
    emit changed();
    if (m_milliseconds >= 60000 || m_pcm.size() > 24 * 1024 * 1024)
      stop();
  });
}
ReferenceRecorder::~ReferenceRecorder() { cancel(); }
QVariantList ReferenceRecorder::devices() const {
  QVariantList out;
  for (const auto &device : QMediaDevices::audioInputs())
    out.append(device.description());
  return out;
}
void ReferenceRecorder::start(int index) {
  if (recording() || m_requesting)
    return;
  cancel();
  const auto generation = ++m_generation;
  QMicrophonePermission permission;
  const auto status = qApp->checkPermission(permission);
  if (status == Qt::PermissionStatus::Denied) {
    m_error = tr("Microphone access is denied. Enable Vox in System Settings, "
                 "then try again.");
    emit changed();
    return;
  }
  if (status == Qt::PermissionStatus::Undetermined) {
    m_requesting = true;
    emit changed();
    qApp->requestPermission(
        permission, this, [this, index, generation](const QPermission &result) {
          if (generation != m_generation)
            return;
          m_requesting = false;
          if (result.status() != Qt::PermissionStatus::Granted) {
            m_error = tr("Microphone access was not granted.");
            emit changed();
            return;
          }
          begin(index, generation);
        });
  } else
    begin(index, generation);
}
void ReferenceRecorder::begin(int index, quint64 generation) {
  if (generation != m_generation)
    return;
  const auto devices = QMediaDevices::audioInputs();
  if (index < 0 || index >= devices.size() || !m_directory.isValid()) {
    m_error = tr("No microphone is available.");
    emit changed();
    return;
  }
  const auto device = devices[index];
  m_format = device.preferredFormat();
  QAudioFormat desired;
  desired.setSampleRate(24000);
  desired.setChannelCount(1);
  desired.setSampleFormat(QAudioFormat::Int16);
  if (device.isFormatSupported(desired))
    m_format = desired;
  if (m_format.channelCount() > 2 ||
      m_format.sampleFormat() == QAudioFormat::Unknown) {
    m_error = tr("This microphone format is not supported.");
    emit changed();
    return;
  }
  m_source = std::make_unique<QAudioSource>(device, m_format);
  m_input = m_source->start();
  if (!m_input || m_source->error() != QAudio::NoError) {
    m_source.reset();
    m_error = tr("Could not start the microphone.");
    emit changed();
    return;
  }
  connect(m_input, &QIODevice::readyRead, this, &ReferenceRecorder::drain);
  connect(m_source.get(), &QAudioSource::stateChanged, this,
          [this](QAudio::State state) {
            if (state == QAudio::StoppedState && m_source &&
                m_source->error() != QAudio::NoError) {
              cancel();
              m_error = tr("Microphone recording stopped unexpectedly.");
              emit changed();
            }
          });
  m_elapsed.start();
  m_timer.start();
  emit changed();
}
void ReferenceRecorder::drain() {
  if (!m_input)
    return;
  m_partial += m_input->readAll();
  const int width = m_format.bytesPerSample();
  const int count = m_partial.size() / width;
  double peak = 0;
  for (int i = 0; i < count; ++i) {
    const auto *p = m_partial.constData() + i * width;
    double value = 0;
    switch (m_format.sampleFormat()) {
    case QAudioFormat::UInt8:
      value = (int(static_cast<unsigned char>(*p)) - 128) / 128.0;
      break;
    case QAudioFormat::Int16: {
      qint16 v;
      std::memcpy(&v, p, 2);
      value = v / 32768.0;
      break;
    }
    case QAudioFormat::Int32: {
      qint32 v;
      std::memcpy(&v, p, 4);
      value = v / 2147483648.0;
      break;
    }
    case QAudioFormat::Float: {
      float v;
      std::memcpy(&v, p, 4);
      value = std::isfinite(v) ? v : 0;
      break;
    }
    default:
      break;
    }
    peak = std::max(peak, std::abs(value));
    const auto sample = qToLittleEndian<qint16>(
        qint16(std::clamp(std::round(value * 32767.0), -32768.0, 32767.0)));
    m_pcm.append(reinterpret_cast<const char *>(&sample), 2);
  }
  m_partial.remove(0, count * width);
  m_level = std::clamp(peak, 0.0, 1.0);
}
void ReferenceRecorder::stop() {
  if (!m_source)
    return;
  m_timer.stop();
  drain();
  disconnect(m_input, nullptr, this, nullptr);
  disconnect(m_source.get(), nullptr, this, nullptr);
  m_source->stop();
  m_source.reset();
  m_input = nullptr;
  m_level = 0;
  finalizeCapture();
}
void ReferenceRecorder::finalizeCapture() {
  const auto frameBytes = m_format.channelCount() * 2;
  m_pcm.truncate((m_pcm.size() / frameBytes) * frameBytes);
  const qint64 frames = m_pcm.size() / frameBytes;
  m_milliseconds = int(frames * 1000 / m_format.sampleRate());
  if (m_milliseconds < 500) {
    m_pcm.clear();
    m_error = tr("Record at least half a second of clear speech.");
    emit changed();
    return;
  }
  const auto path = m_directory.filePath("reference.wav");
  QFile output(path);
  if (!output.open(QIODevice::WriteOnly)) {
    m_error = tr("Could not save the recording.");
    emit changed();
    return;
  }
  QDataStream out(&output);
  out.setByteOrder(QDataStream::LittleEndian);
  out.writeRawData("RIFF", 4);
  out << quint32(36 + m_pcm.size());
  out.writeRawData("WAVEfmt ", 8);
  out << quint32(16) << quint16(1) << quint16(m_format.channelCount())
      << quint32(m_format.sampleRate())
      << quint32(m_format.sampleRate() * frameBytes) << quint16(frameBytes)
      << quint16(16);
  out.writeRawData("data", 4);
  out << quint32(m_pcm.size());
  out.writeRawData(m_pcm.constData(), m_pcm.size());
  if (out.status() != QDataStream::Ok) {
    m_error = tr("Could not save the recording.");
    output.close();
    output.remove();
  } else {
    output.close();
    m_file = QUrl::fromLocalFile(path);
  }
  m_pcm.clear();
  emit changed();
}
void ReferenceRecorder::cancel() {
  ++m_generation;
  m_requesting = false;
  m_timer.stop();
  if (m_source) {
    disconnect(m_source.get(), nullptr, this, nullptr);
    if (m_input)
      disconnect(m_input, nullptr, this, nullptr);
    m_source->stop();
    m_source.reset();
  }
  m_input = nullptr;
  m_pcm.clear();
  m_partial.clear();
  m_milliseconds = 0;
  m_level = 0;
  m_error.clear();
  if (m_file.isLocalFile())
    QFile::remove(m_file.toLocalFile());
  m_file = QUrl{};
  emit changed();
}
