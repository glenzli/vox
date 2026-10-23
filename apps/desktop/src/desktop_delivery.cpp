#include "desktop_delivery.hpp"
#include "desktop_backend.hpp"
#include <QDir>
#include <QFileInfo>

DesktopDelivery::DesktopDelivery(DesktopBackend &backend)
    : QObject(&backend), m_backend(backend) {
  connect(&backend, &DesktopBackend::stateChanged, this,
          &DesktopDelivery::syncIdentity);
  connect(&backend, &DesktopBackend::documentChanged, this,
          &DesktopDelivery::syncIdentity);
  connect(&backend, &DesktopBackend::selectionChanged, this, [this] {
    if (subtitleActive() && m_audioID != m_backend.selectedAudio())
      closeSubtitles();
  });
}

bool DesktopDelivery::subtitleIdentity() const {
  return subtitleActive() && m_subtitlePath == m_backend.m_path &&
         m_subtitleProject == m_backend.m_project["id"].toString() &&
         m_subtitleEpoch == m_backend.m_projectEpoch;
}
bool DesktopDelivery::compilationIdentity() const {
  return compilationActive() &&
         m_compilationPath == m_backend.m_collectionPath &&
         m_compilationProject ==
             m_backend.m_collectionProject["id"].toString() &&
         m_compilationEpoch == m_backend.m_collectionEpoch;
}
void DesktopDelivery::syncIdentity() {
  if (subtitleActive() && !subtitleIdentity())
    closeSubtitles();
  if (compilationActive() && !compilationIdentity()) {
    closeCompilation();
  } else if (!m_compiling && !m_preflight.isEmpty() &&
             (m_backend.dirty() ||
              m_preflight["project_version"] !=
                  m_backend.m_collectionProject["version"] ||
              m_reviewedChapters != m_backend.m_chapters)) {
    invalidateCompilation();
  }
}
void DesktopDelivery::retranslate() {
  emit subtitleChanged();
  emit compilationChanged();
}
QVariantMap DesktopDelivery::chapterStatusLabels() const {
  QVariantMap labels;
  for (const auto &value : m_backend.m_chapters) {
    const auto chapter = value.toObject();
    labels.insert(chapter["id"].toString(),
                  m_backend.productionStatus(chapter["status"].toString()));
  }
  return labels;
}
void DesktopDelivery::setTranscript(const QString &value) {
  if (m_transcript == value)
    return;
  m_transcript = value;
  emit formChanged();
}
void DesktopDelivery::setLanguage(const QString &value) {
  if (m_language == value)
    return;
  m_language = value;
  emit formChanged();
}
void DesktopDelivery::openSubtitles(const QString &id) {
  if (m_backend.busy() || m_backend.m_path.isEmpty() || id.isEmpty())
    return;
  closeSubtitles();
  m_subtitlePath = m_backend.m_path;
  m_subtitleProject = m_backend.m_project["id"].toString();
  m_subtitleEpoch = m_backend.m_projectEpoch;
  m_audioID = id;
  loadSubtitles({}, true);
}
void DesktopDelivery::closeSubtitles() {
  ++m_subtitleSequence;
  m_subtitlePath.clear();
  m_subtitleProject.clear();
  m_audioID.clear();
  m_candidateID.clear();
  m_track = {};
  m_candidates = {};
  m_acceptedAudio = false;
  m_subtitleLoading = false;
  m_subtitleCode.clear();
  m_transcript.clear();
  m_language.clear();
  emit subtitleChanged();
  emit formChanged();
}
void DesktopDelivery::loadSubtitles(const QString &candidateID,
                                    bool initialize) {
  if (!subtitleIdentity())
    return;
  const auto sequence = ++m_subtitleSequence;
  const auto edit = m_backend.m_editSequence;
  m_subtitleLoading = true;
  m_candidateID = candidateID;
  m_track = {};
  emit subtitleChanged();
  QJsonObject fields{{"audio_id", m_audioID}};
  if (!candidateID.isEmpty())
    fields.insert("candidate_id", candidateID);
  m_backend.command("subtitle_preview", fields,
                    [this, sequence, edit, initialize](const QJsonObject &r) {
                      if (sequence != m_subtitleSequence || !subtitleIdentity())
                        return;
                      m_subtitleLoading = false;
                      if (!r["ok"].toBool()) {
                        m_subtitleCode = r["error"].toString();
                        emit subtitleChanged();
                        return;
                      }
                      const auto result = r["result"].toObject();
                      m_backend.apply(result, edit);
                      if (sequence != m_subtitleSequence || !subtitleIdentity())
                        return;
                      m_track = result["track"].toObject();
                      m_candidates = result["candidates"].toArray();
                      m_acceptedAudio = result["accepted_audio"].toBool();
                      if (initialize) {
                        m_transcript = result["transcript"].toString();
                        m_language = result["language"].toString();
                        if (m_language == "auto")
                          m_language.clear();
                        if (!result["transcript_error"].toString().isEmpty())
                          m_subtitleCode =
                              result["transcript_error"].toString();
                        emit formChanged();
                      }
                      if (m_subtitleCode.isEmpty() && m_track.isEmpty())
                        m_subtitleCode = result["track_error"].toString();
                      emit subtitleChanged();
                    });
}
void DesktopDelivery::previewSubtitleCandidate(const QString &id) {
  if (m_backend.busy() || m_subtitleLoading || !subtitleIdentity())
    return;
  m_subtitleCode.clear();
  loadSubtitles(id);
}
void DesktopDelivery::alignSubtitles(bool exactTranscriptConfirmed) {
  if (m_backend.busy() || m_subtitleLoading || !subtitleIdentity())
    return;
  if (!m_acceptedAudio || !exactTranscriptConfirmed ||
      m_transcript.trimmed().isEmpty() || m_language.isEmpty() ||
      m_language == "auto") {
    m_subtitleCode = "subtitle_transcript_required";
    emit subtitleChanged();
    return;
  }
  const auto epoch = m_backend.m_projectEpoch;
  const auto sequence = m_subtitleSequence;
  const auto audio = m_audioID;
  const QJsonObject spec{{"transcript", m_transcript},
                         {"language", m_language}};
  m_aligning = true;
  m_subtitleCode = "subtitle_aligning";
  m_backend.m_completed = 0;
  m_backend.m_total = 1;
  m_backend.setRendering(true);
  emit m_backend.progressChanged();
  emit subtitleChanged();
  const auto stopped = [this, epoch, sequence](const QString &code) {
    m_aligning = false;
    m_backend.setRendering(false);
    m_backend.messageCode({});
    if (epoch == m_backend.m_projectEpoch && sequence == m_subtitleSequence) {
      m_subtitleCode = code;
    }
    emit subtitleChanged();
    emit m_backend.requestFinished("align_subtitles", false);
  };
  m_backend.saveThen(
      [this, epoch, sequence, audio, spec, stopped] {
        if (epoch != m_backend.m_projectEpoch ||
            sequence != m_subtitleSequence || m_backend.m_cancelRequested) {
          stopped("subtitle_cancelled");
          return;
        }
        const auto edit = m_backend.m_editSequence;
        m_backend.command(
            "align_subtitles", {{"audio_id", audio}, {"spec", spec}},
            [this, epoch, sequence, edit](const QJsonObject &r) {
              m_aligning = false;
              m_backend.setRendering(false);
              m_backend.messageCode({});
              emit subtitleChanged();
              if (epoch != m_backend.m_projectEpoch)
                return;
              if (r["ok"].toBool())
                m_backend.apply(r["result"].toObject(), edit);
              if (sequence != m_subtitleSequence || !subtitleIdentity())
                return;
              QString candidate;
              if (!r["ok"].toBool()) {
                m_subtitleCode = r["error"].toString();
              } else {
                const auto outcome =
                    r["result"].toObject()["subtitle_outcome"].toObject();
                candidate = outcome["candidate_id"].toString();
                const auto state = outcome["state"].toString();
                m_subtitleCode =
                    state == "cancelled" ? candidate.isEmpty()
                                               ? "subtitle_cancelled"
                                               : "subtitle_cancelled_candidate"
                    : state == "failed"  ? outcome["error"].toString()
                                         : "subtitle_candidate_ready";
              }
              emit subtitleChanged();
              loadSubtitles(candidate);
            });
      },
      [stopped] { stopped("subtitle_prepare_failed"); });
}
void DesktopDelivery::changeSubtitleCandidate(bool accept) {
  if (m_backend.busy() || m_subtitleLoading || !subtitleIdentity() ||
      m_candidateID.isEmpty())
    return;
  const auto candidate = m_candidateID;
  const auto sequence = m_subtitleSequence;
  const auto epoch = m_backend.m_projectEpoch;
  m_subtitleLoading = true;
  emit subtitleChanged();
  m_backend.saveThen(
      [this, candidate, sequence, epoch, accept] {
        if (sequence != m_subtitleSequence || !subtitleIdentity())
          return;
        const auto edit = m_backend.m_editSequence;
        m_backend.command(
            accept ? "accept_subtitles" : "discard_subtitles",
            {{"id", candidate}},
            [this, sequence, epoch, edit, accept](const QJsonObject &r) {
              if (epoch == m_backend.m_projectEpoch && r["ok"].toBool())
                m_backend.apply(r["result"].toObject(), edit);
              if (sequence != m_subtitleSequence || !subtitleIdentity())
                return;
              m_subtitleLoading = false;
              if (r["ok"].toBool()) {
                m_subtitleCode =
                    accept ? "subtitle_accepted" : "subtitle_discarded";
                loadSubtitles({});
              } else {
                m_subtitleCode = r["error"].toString();
                emit subtitleChanged();
              }
            });
      },
      [this, sequence] {
        if (sequence == m_subtitleSequence) {
          m_subtitleLoading = false;
          m_subtitleCode = "subtitle_prepare_failed";
          emit subtitleChanged();
        }
      });
}
void DesktopDelivery::acceptSubtitleCandidate() {
  changeSubtitleCandidate(true);
}
void DesktopDelivery::discardSubtitleCandidate() {
  changeSubtitleCandidate(false);
}
void DesktopDelivery::cancelAlignment() {
  if (m_aligning) {
    m_subtitleCode = "subtitle_stopping";
    m_backend.cancel();
    emit subtitleChanged();
  }
}
bool DesktopDelivery::canExportSubtitles() const {
  return subtitleIdentity() && m_candidateID.isEmpty() && !m_track.isEmpty() &&
         !m_subtitleLoading && !m_aligning;
}
void DesktopDelivery::exportSubtitles(const QUrl &destination,
                                      const QString &format) {
  if (canExportSubtitles())
    m_backend.exportSubtitles(m_audioID, destination, format);
}

void DesktopDelivery::openCompilation() {
  if (m_backend.busy() || m_backend.m_collectionPath.isEmpty())
    return;
  closeCompilation();
  m_compilationPath = m_backend.m_collectionPath;
  m_compilationProject = m_backend.m_collectionProject["id"].toString();
  m_compilationEpoch = m_backend.m_collectionEpoch;
  emit compilationChanged();
}
void DesktopDelivery::closeCompilation() {
  ++m_compilationSequence;
  m_compilationPath.clear();
  m_compilationProject.clear();
  m_compilationFields = {};
  m_preflight = {};
  m_compilationLoading = false;
  m_compilationCode.clear();
  m_compilationDestination.clear();
  emit compilationChanged();
}
void DesktopDelivery::invalidateCompilation() {
  ++m_compilationSequence;
  m_preflight = {};
  m_compilationLoading = false;
  m_compilationDestination.clear();
  m_compilationCode = "compilation_review_required";
  emit compilationChanged();
}
void DesktopDelivery::configureCompilation(const QVariantList &chapterIDs,
                                           int gapMs, const QString &audioMode,
                                           const QString &subtitles) {
  if (!compilationIdentity() || m_compiling)
    return;
  QJsonArray ids;
  for (const auto &id : chapterIDs)
    ids.append(id.toString());
  QJsonObject fields{{"chapter_ids", ids},
                     {"options", QJsonObject{{"gap_ms", gapMs},
                                             {"audio_mode", audioMode},
                                             {"subtitles", subtitles}}}};
  if (m_compilationFields == fields)
    return;
  m_compilationFields = fields;
  invalidateCompilation();
}
void DesktopDelivery::checkCompilation() {
  if (!compilationIdentity() || m_backend.busy() || m_compilationLoading)
    return;
  if (m_backend.dirty() ||
      m_compilationFields["chapter_ids"].toArray().isEmpty()) {
    m_compilationCode = m_backend.dirty() ? "compilation_save_required"
                                          : "compilation_selection_required";
    emit compilationChanged();
    return;
  }
  const auto sequence = ++m_compilationSequence;
  m_preflight = {};
  m_compilationLoading = true;
  m_compilationCode.clear();
  emit compilationChanged();
  m_backend.collectionCommand(
      "chapter_compilation_preflight", m_compilationFields,
      [this, sequence](const QJsonObject &r) {
        if (sequence != m_compilationSequence || !compilationIdentity())
          return;
        m_compilationLoading = false;
        if (r["ok"].toBool()) {
          m_preflight = r["result"].toObject();
          m_reviewedChapters = m_backend.m_chapters;
          m_compilationCode = m_preflight["ready"].toBool()
                                  ? "compilation_ready"
                                  : "compilation_issues";
          syncIdentity();
        } else {
          m_compilationCode = r["error"].toString();
        }
        emit compilationChanged();
      });
}
void DesktopDelivery::exportCompilation(const QUrl &parent,
                                        const QString &folderName) {
  if (!compilationIdentity() || m_backend.busy() || m_backend.dirty() ||
      m_compilationLoading || !m_preflight["ready"].toBool() ||
      m_preflight["fingerprint"].toString().isEmpty())
    return;
  const auto name = folderName.trimmed();
  if (!parent.isLocalFile() || name.isEmpty() || name == "." || name == ".." ||
      name.contains('/') || name.contains('\\')) {
    m_compilationCode = "compilation_folder_required";
    emit compilationChanged();
    return;
  }
  const auto sequence = m_compilationSequence;
  const auto destination = QDir(parent.toLocalFile()).filePath(name);
  auto fields = m_compilationFields;
  fields.insert("destination", destination);
  fields.insert("fingerprint", m_preflight["fingerprint"]);
  m_compiling = true;
  m_compilationCode = "compilation_exporting";
  m_backend.setProjectChanging(true);
  emit compilationChanged();
  m_backend.collectionCommand(
      "chapter_compile", fields,
      [this, sequence, destination](const QJsonObject &r) {
        m_compiling = false;
        m_backend.setProjectChanging(false);
        emit compilationChanged();
        if (sequence != m_compilationSequence || !compilationIdentity())
          return;
        if (r["ok"].toBool()) {
          m_compilationDestination = destination;
          m_compilationCode = "compilation_exported";
        } else {
          m_compilationCode = r["error"].toString();
          if (m_compilationCode == "compilation_preflight_changed")
            m_preflight = {};
        }
        emit compilationChanged();
      });
}

QString DesktopDelivery::subtitleSourceLabel(const QString &source) const {
  if (source == "measured_segments")
    return tr("Measured segments");
  if (source == "forced_alignment")
    return tr("Aligned estimate");
  return tr("No current timings");
}
QString DesktopDelivery::subtitleMessage() const {
  if (m_subtitleCode == "subtitle_aligning")
    return tr("Aligning the confirmed transcript to this audio…");
  if (m_subtitleCode == "subtitle_stopping")
    return tr("Waiting for the current alignment request to finish…");
  if (m_subtitleCode == "subtitle_candidate_ready")
    return tr(
        "Candidate timings are ready. Review them, then accept explicitly.");
  if (m_subtitleCode == "subtitle_cancelled_candidate")
    return tr("Alignment stopped. The completed candidate is kept for review.");
  if (m_subtitleCode == "subtitle_cancelled" ||
      m_subtitleCode == "request_cancelled")
    return tr("Alignment stopped. Existing timings are kept.");
  if (m_subtitleCode == "subtitle_accepted")
    return tr("Timings accepted for this audio version.");
  if (m_subtitleCode == "subtitle_discarded")
    return tr("Candidate timings discarded.");
  if (m_subtitleCode.isEmpty())
    return {};
  return issueText(m_subtitleCode);
}
QString DesktopDelivery::compilationMessage() const {
  if (m_compilationCode == "compilation_review_required")
    return tr("Check this selection before exporting.");
  if (m_compilationCode == "compilation_ready")
    return tr("Ready to export the reviewed chapter versions.");
  if (m_compilationCode == "compilation_issues")
    return tr("Resolve the issues below, then check again.");
  if (m_compilationCode == "compilation_exporting")
    return tr("Exporting the compilation…");
  if (m_compilationCode == "compilation_exported")
    return tr("Compilation exported to a new folder.");
  if (m_compilationCode.isEmpty())
    return {};
  return issueText(m_compilationCode);
}
QString DesktopDelivery::issueText(const QString &code) const {
  if (code == "subtitle_transcript_required" ||
      code == "invalid_alignment_spec")
    return tr("Enter the exact spoken transcript, choose its language and "
              "confirm it matches this audio.");
  if (code == "subtitles_unavailable" || code == "subtitle_track_unavailable")
    return tr("This audio has no current timings. Align a confirmed transcript "
              "to create them.");
  if (code == "subtitle_prepare_failed" || code == "compilation_save_required")
    return tr("Save the current script before continuing.");
  if (code == "subtitle_missing" || code == "subtitle_candidate_missing" ||
      code == "subtitle_audio_changed" || code == "stale_audio")
    return tr("These timings no longer match the audio. Reopen subtitles to "
              "review it.");
  if (code == "audio_not_accepted" || code == "subtitle_audio_unaccepted" ||
      code == "accepted_audio_missing")
    return tr("Accept this audio version before realigning subtitles.");
  if (code == "compilation_selection_required" || code == "compilation_empty")
    return tr("Select at least one chapter.");
  if (code == "compilation_folder_required")
    return tr("Choose a parent folder and a new folder name.");
  if (code == "compilation_preflight_changed")
    return tr("A chapter changed after review. Check the selection again.");
  if (code == "compilation_format_mismatch")
    return tr("The audio formats differ. Explicitly choose mono 24 kHz "
              "conversion, or use matching audio.");
  if (code == "compilation_subtitles_missing")
    return tr("A chapter needs current subtitles. Align and accept them, or "
              "choose audio only.");
  if (code == "compilation_gap_invalid" ||
      code == "invalid_compilation_options")
    return tr("Choose a gap from 0 to 600000 milliseconds and supported export "
              "options.");
  if (code == "compilation_not_ready")
    return tr("Check the selection and resolve every issue before exporting.");
  if (code == "compilation_conversion_channels")
    return tr("Mono conversion supports one or two input channels. Prepare "
              "this chapter in Echo first.");
  if (code == "compilation_input_too_large" || code == "compilation_too_large")
    return tr("This compilation exceeds the 512 MB audio limit. Export fewer "
              "chapters at a time.");
  if (code == "compilation_too_long")
    return tr(
        "This compilation exceeds 24 hours. Export fewer chapters at a time.");
  if (code == "compilation_empty_audio")
    return tr(
        "A selected chapter has empty audio. Accept a playable version first.");
  if (code == "compilation_subtitle_resolution")
    return tr("A subtitle cue is too short for the output audio. Revise its "
              "timing or choose audio only.");
  if (code == "compilation_size_mismatch" || code == "corrupt_audio" ||
      code == "corrupt_object")
    return tr("Audio or source material failed its integrity check. Restore "
              "the project material before exporting.");
  if (code == "invalid_chapter_selection")
    return tr(
        "Select each available chapter once in the desired export order.");
  if (code == "invalid_chapter_owner" || code == "nested_chapter_project")
    return tr("A chapter does not belong to this collection. Refresh the "
              "chapter list.");
  if (code == "invalid_destination")
    return tr("Choose a new export folder outside the project.");
  if (code == "invalid_subtitles" || code == "invalid_alignment_response" ||
      code == "invalid_alignment_receipt" || code == "invalid_alignment" ||
      code == "invalid_provenance")
    return tr("Subtitle timing evidence is invalid. Realign this audio and "
              "review a new candidate.");
  if (code == "alignment_audio_too_large")
    return tr("Alignment supports audio up to 128 MiB and 30 minutes. Use a "
              "shorter accepted version.");
  if (code == "alignment_transcript_mismatch")
    return tr("The alignment could not match these words. Review the exact "
              "transcript against the audio, then try again.");
  return m_backend.errorText(code);
}
