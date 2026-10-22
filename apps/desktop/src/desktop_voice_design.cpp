#include "desktop_backend.hpp"

void DesktopBackend::generateVoiceDesign(const QString &description,
                                         const QString &text,
                                         const QString &language,
                                         int variations) {
  if (busy() || m_path.isEmpty()) {
    messageCode("render_busy");
    return;
  }
  if (description.trimmed().isEmpty() || text.trimmed().isEmpty() ||
      variations < 1 || variations > 4) {
    messageCode("invalid_voice_design");
    return;
  }
  const auto epoch = m_projectEpoch;
  m_designingVoice = true;
  m_completed = 0;
  m_total = variations;
  setRendering(true);
  emit progressChanged();
  const auto failed = [this, epoch] {
    if (epoch != m_projectEpoch)
      return;
    setRendering(false);
    m_designingVoice = false;
    emit requestFinished("generate_voice_design_batch", false);
  };
  saveThen(
      [this, description, text, language, variations, epoch, failed] {
        if (epoch != m_projectEpoch)
          return;
        if (m_cancelRequested) {
          messageCode("voice_design_cancelled");
          failed();
          return;
        }
        const auto edit = m_editSequence;
        command("generate_voice_design_batch",
                {{"spec", QJsonObject{{"description", description},
                                      {"text", text},
                                      {"language", language}}},
                 {"variations", variations}},
                [this, epoch, edit](const QJsonObject &response) {
                  if (epoch != m_projectEpoch)
                    return;
                  setRendering(false);
                  m_designingVoice = false;
                  if (!response["ok"].toBool()) {
                    command("open", {});
                    return;
                  }
                  const auto result = response["result"].toObject();
                  apply(result, edit);
                  const auto batch = result["voice_design_batch"].toObject();
                  const auto state = batch["state"].toString();
                  messageCode(state == "cancelled" ? "voice_design_cancelled"
                              : state == "partial_failed" || state == "failed"
                                  ? "voice_design_partial_failed"
                                  : "voice_design_completed");
                });
      },
      failed);
}

void DesktopBackend::acceptVoiceDesign(const QString &id, const QString &name) {
  if (busy() || m_path.isEmpty() || id.isEmpty() || name.trimmed().isEmpty())
    return;
  const auto epoch = m_projectEpoch;
  saveThen([this, id, name, epoch] {
    if (epoch != m_projectEpoch)
      return;
    const auto edit = m_editSequence;
    command("accept_voice_design", {{"id", id}, {"name", name}},
            [this, epoch, edit](const QJsonObject &response) {
              if (epoch == m_projectEpoch && response["ok"].toBool()) {
                apply(response["result"].toObject(), edit);
                messageCode("voice_design_saved");
              }
            });
  });
}

void DesktopBackend::discardVoiceDesign(const QString &id) {
  materialCommand("discard_voice_design", {{"id", id}});
}
