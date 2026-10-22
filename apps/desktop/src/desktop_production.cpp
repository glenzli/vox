#include "desktop_backend.hpp"

void DesktopBackend::productionCommand(const QString &action,
                                       const QJsonObject &fields,
                                       bool generate) {
  if (busy() || m_path.isEmpty()) {
    messageCode("render_busy");
    return;
  }
  const bool prepareSource =
      action == "render_segment" || action == "assemble_segments";
  const bool active = generate || prepareSource;
  const auto epoch = m_projectEpoch;
  if (active) {
    setRendering(true);
    m_completed = 0;
    m_total = generate ? 1 : 0;
    emit progressChanged();
  }
  const auto fail = [this, action, active, epoch](const QString &code) {
    if (epoch != m_projectEpoch)
      return;
    if (active)
      setRendering(false);
    if (!code.isEmpty())
      messageCode(code);
    emit requestFinished(action, false);
  };
  saveThen(
      [this, action, fields, active, prepareSource, epoch, fail] {
        if (epoch != m_projectEpoch)
          return;
        if (active && m_cancelRequested) {
          fail("speech_cancelled");
          return;
        }
        const auto dispatch = [this, action, fields, active, epoch] {
          const auto edit = m_editSequence;
          const auto selection = m_selectionSequence;
          command(
              action, fields,
              [this, action, active, epoch, edit,
               selection](const QJsonObject &r) {
                if (epoch != m_projectEpoch)
                  return;
                if (active)
                  setRendering(false);
                if (!r["ok"].toBool()) {
                  if (active)
                    command("open", {});
                  return;
                }
                apply(r["result"].toObject(), edit);
                if (action == "assemble_segments") {
                  const auto candidates = m_project["candidates"].toArray();
                  if (!candidates.isEmpty() && selection == m_selectionSequence)
                    selectAudio(candidates.last().toObject()["id"].toString());
                  messageCode("segments_assembled");
                } else if (action == "accept_segment") {
                  messageCode("segment_accepted");
                } else {
                  messageCode("segment_rendered");
                }
              });
        };
        if (!prepareSource) {
          dispatch();
          return;
        }
        if (!m_analysis["ready"].toBool()) {
          fail("script_not_ready");
          return;
        }
        const auto edit = m_editSequence;
        command(
            "adopt", {},
            [this, action, epoch, edit, dispatch, fail](const QJsonObject &r) {
              if (epoch != m_projectEpoch)
                return;
              if (!r["ok"].toBool()) {
                fail({});
                return;
              }
              apply(r["result"].toObject(), edit);
              if (m_cancelRequested) {
                fail("speech_cancelled");
                return;
              }
              if (!m_segmentPlan["ready"].toBool() ||
                  !m_segmentPlan["adopted"].toBool()) {
                fail("script_not_ready");
                return;
              }
              if (action == "assemble_segments" &&
                  !m_segmentPlan["can_assemble"].toBool()) {
                fail("segments_not_accepted");
                return;
              }
              dispatch();
            });
      },
      [fail] { fail({}); });
}

void DesktopBackend::renderSegment(const QString &key) {
  if (key.isEmpty())
    return;
  productionCommand("render_segment", {{"segment_key", key}}, true);
}

void DesktopBackend::acceptSegment(const QString &id) {
  if (!id.isEmpty())
    productionCommand("accept_segment", {{"segment_id", id}});
}

void DesktopBackend::assembleSegments() {
  productionCommand("assemble_segments", {});
}

void DesktopBackend::playSegment(const QString &id) {
  if (busy() || m_path.isEmpty() || id.isEmpty())
    return;
  const auto epoch = m_projectEpoch;
  const auto playback = ++m_playbackSequence;
  command("segment_playback", {{"segment_id", id}},
          [this, epoch, playback](const QJsonObject &r) {
            if (r["ok"].toBool() && epoch == m_projectEpoch &&
                playback == m_playbackSequence) {
              const auto result = r["result"].toObject();
              playPreview(QUrl::fromLocalFile(result["file"].toString()),
                          result["peaks"].toArray().toVariantList());
            }
          });
}

void DesktopBackend::setPronunciationRules(const QVariantList &rules) {
  QJsonArray values;
  for (const auto &value : rules) {
    const auto rule = value.toMap();
    QJsonObject item{{"text", rule["text"].toString()},
                     {"spoken", rule["spoken"].toString()}};
    for (const auto &key :
         {QStringLiteral("language"), QStringLiteral("role")}) {
      const auto filter = rule[key].toString();
      if (!filter.isEmpty())
        item.insert(key, filter);
    }
    values.append(item);
  }
  mutate([&values](QJsonObject &document) {
    if (values.isEmpty())
      document.remove("pronunciation_rules");
    else
      document.insert("pronunciation_rules", values);
  });
}

void DesktopBackend::exportSubtitles(const QString &audioID, const QUrl &url,
                                     const QString &format) {
  if (busy() || m_path.isEmpty() || audioID.isEmpty() || !url.isLocalFile() ||
      (format != "srt" && format != "vtt"))
    return;
  const auto epoch = m_projectEpoch;
  saveThen([this, audioID, url, format, epoch] {
    if (epoch != m_projectEpoch)
      return;
    command("export_subtitles",
            {{"audio_id", audioID},
             {"destination", url.toLocalFile()},
             {"format", format}},
            [this, epoch](const QJsonObject &r) {
              if (r["ok"].toBool() && epoch == m_projectEpoch)
                messageCode("subtitles_exported");
            });
  });
}
