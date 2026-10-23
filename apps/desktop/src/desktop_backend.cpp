#include "desktop_backend.hpp"
#include "desktop_delivery.hpp"
#include <QCoreApplication>
#include <QDir>
#include <QFile>
#include <QFileInfo>
#include <QJsonDocument>
#include <QStandardPaths>
#include <QUuid>
#include <utility>

DesktopBackend::DesktopBackend(QObject *parent)
    : QObject(parent), m_worker(this), m_probeWorker(this),
      m_echoLauncher(this) {
  m_delivery = new DesktopDelivery(*this);
  connect(&m_echoLauncher, &EchoLauncher::opened, this, [this] {
    m_echoOpening = false;
    messageCode("echo_opened");
  });
  connect(&m_echoLauncher, &EchoLauncher::failed, this,
          [this](const QString &code) {
            m_echoOpening = false;
            messageCode(code);
          });
  m_saveTimer.setSingleShot(true);
  m_saveTimer.setInterval(700);
  connect(&m_saveTimer, &QTimer::timeout, this, &DesktopBackend::save);
  connect(&m_worker, &WorkerClient::progress, this,
          [this](int completed, int total) {
            m_completed = completed;
            m_total = total;
            emit progressChanged();
          });
  connect(&m_worker, &WorkerClient::progressDetail, this,
          [this](const QJsonObject &detail) {
            if (m_batchingChapters && detail.contains("chapter_id")) {
              m_chapterProgress = detail;
              emit progressChanged();
            }
          });
  command(
      "capabilities", {},
      [this](const QJsonObject &r) {
        if (r["ok"].toBool()) {
          m_voices = r["result"].toObject()["voices"].toArray();
          emit capabilitiesChanged();
        }
      },
      false);
  QTimer::singleShot(100, this, &DesktopBackend::probeRuntime);
  m_probeTimer.setInterval(30000);
  connect(&m_probeTimer, &QTimer::timeout, this, [this] {
    if (!m_rendering)
      probeRuntime();
  });
  m_probeTimer.start();
  connect(this, &DesktopBackend::requestFinished, this,
          [this](const QString &action, bool ok) {
            if (ok && !m_chapterID.isEmpty() && !busy() &&
                QStringList{"save", "adopt", "render", "render_segment",
                            "assemble_segments", "accept", "discard",
                            "echo_return", "accept_segment"}
                    .contains(action))
              refreshChapters();
          });
}
void DesktopBackend::command(const QString &action, const QJsonObject &fields,
                             WorkerClient::Callback done, bool includeProject) {
  const auto path = m_path;
  const auto projectID = m_project["id"].toString();
  const auto epoch = m_projectEpoch;
  const auto edit = m_editSequence;
  const bool cancellable = QStringList{"render",
                                       "render_segment",
                                       "voice_preview",
                                       "reference_preview",
                                       "generate_sound",
                                       "generate_sound_batch",
                                       "generate_voice_design_batch",
                                       "align_subtitles",
                                       "import_shape_transfer"}
                               .contains(action);
  ++m_pendingRequests;
  emit pendingChanged();
  m_worker.request(
      [this, action, fields, includeProject, path, projectID, epoch] {
        if (includeProject &&
            (path != m_path || projectID != m_project["id"].toString() ||
             epoch != m_projectEpoch))
          return QJsonObject{{"action", "capabilities"}};
        auto request = fields;
        request.insert("action", action);
        if (includeProject) {
          request.insert("path", path);
          request.insert("version", m_project["version"]);
        }
        return request;
      },
      [this, action, done, edit, path, includeProject, projectID, epoch,
       cancellable](const QJsonObject &response) {
        --m_pendingRequests;
        emit pendingChanged();
        if (includeProject &&
            (path != m_path || projectID != m_project["id"].toString() ||
             epoch != m_projectEpoch)) {
          if (done)
            done(QJsonObject{{"ok", false}, {"error", "stale_project"}});
          emit requestFinished(action, false);
          return;
        }
        auto r = response;
        if (cancellable && r["error"].toString() == "request_cancelled")
          r.insert("error", action == "generate_sound" ? "generation_cancelled"
                            : action == "import_shape_transfer"
                                ? "shape_transfer_cancelled"
                            : action == "align_subtitles" ? "subtitle_cancelled"
                                                          : "speech_cancelled");
        const bool ok = r["ok"].toBool();
        if (!ok)
          messageCode(r["error"].toString());
        if (done)
          done(r);
        else if (ok && r["result"].toObject().contains("project"))
          apply(r["result"].toObject(), edit);
        emit requestFinished(action, ok);
      },
      cancellable);
}
QObject *DesktopBackend::delivery() const { return m_delivery; }
void DesktopBackend::apply(const QJsonObject &result, quint64 capturedEdit,
                           bool reset) {
  const auto project = result["project"].toObject();
  if (project.isEmpty())
    return;
  const bool sameCollection = m_collectionPath == result["path"].toString();
  m_project = project;
  m_path = result["path"].toString();
  if (!result["parent_project"].toObject().isEmpty()) {
    applyCollection(result);
  } else if (m_chapterID.isEmpty() || reset) {
    if (reset) {
      ++m_collectionEpoch;
      m_chapterID.clear();
    }
    m_collectionProject = project;
    m_collectionPath = m_path;
    m_chaptersError = result["chapters_error"].toString();
    if (m_chaptersError.isEmpty())
      m_chapters = result["chapters"].toArray();
    else if (!sameCollection || m_chapters.isEmpty())
      m_chapters = project["chapters"].toArray();
  }
  m_analysis = result["analysis"].toObject();
  m_segmentPlan = result["segment_plan"].toObject();
  if (reset || capturedEdit == m_editSequence) {
    m_draft = project["draft"].toObject();
    m_dirty = false;
    emit documentChanged();
  }
  if (reset) {
    ++m_projectEpoch;
    ++m_selectionSequence;
    ++m_playbackSequence;
    m_selected.clear();
    m_preparedID.clear();
    m_waveform.clear();
    emit selectionChanged();
    emit waveformChanged();
    selectAudio(project["audio_head"].toString());
  }
  QStringList recent = m_settings.value("recentProjects").toStringList();
  recent.removeAll(m_collectionPath);
  recent.prepend(m_collectionPath);
  while (recent.size() > 12)
    recent.removeLast();
  m_settings.setValue("recentProjects", recent);
  emit stateChanged();
}
QString DesktopBackend::importedDraftKind() const {
  const auto drafts = m_project["shape_import"].toObject()["drafts"].toArray();
  for (const auto &draft : drafts) {
    const auto value = draft.toObject();
    if (value["document"].toObject() == m_draft)
      return value["kind"].toString();
  }
  return drafts.isEmpty() ? QString{} : QStringLiteral("modified");
}
QVariantList DesktopBackend::recentProjects() const {
  QVariantList out;
  for (const auto &path : m_settings.value("recentProjects").toStringList())
    if (QFileInfo::exists(path))
      out.append(QVariantMap{{"path", path},
                             {"name", QFileInfo(path).completeBaseName()}});
  return out;
}
void DesktopBackend::createProject(const QUrl &url) {
  if (busy())
    return;
  setProjectChanging(true);
  saveThen(
      [this, url] {
        const auto path = url.toLocalFile();
        command(
            "create",
            {{"path", path}, {"title", QFileInfo(path).completeBaseName()}},
            [this](const QJsonObject &r) {
              setProjectChanging(false);
              if (r["ok"].toBool()) {
                ++m_editSequence;
                apply(r["result"].toObject(), m_editSequence, true);
                messageCode("project_created");
              }
            },
            false);
      },
      [this] { setProjectChanging(false); });
}
void DesktopBackend::openProject(const QUrl &url) {
  if (busy())
    return;
  setProjectChanging(true);
  saveThen(
      [this, url] {
        command(
            "open", {{"path", url.toLocalFile()}},
            [this](const QJsonObject &r) {
              setProjectChanging(false);
              if (r["ok"].toBool()) {
                ++m_editSequence;
                apply(r["result"].toObject(), m_editSequence, true);
                messageCode("project_opened");
              }
            },
            false);
      },
      [this] { setProjectChanging(false); });
}
void DesktopBackend::importShapeTransfer(const QUrl &transfer,
                                         const QUrl &destination,
                                         const QString &expectedBlake3) {
  if (busy())
    return;
  setRendering(true);
  saveThen(
      [this, transfer, destination, expectedBlake3] {
        if (m_cancelRequested) {
          setRendering(false);
          messageCode("shape_transfer_cancelled");
          return;
        }
        command(
            "import_shape_transfer",
            {{"file", transfer.toLocalFile()},
             {"path", destination.toLocalFile()},
             {"expected_blake3", expectedBlake3.isEmpty()
                                     ? QJsonValue{}
                                     : QJsonValue{expectedBlake3}}},
            [this](const QJsonObject &r) {
              setRendering(false);
              if (r["ok"].toBool()) {
                ++m_editSequence;
                apply(r["result"].toObject(), m_editSequence, true);
                messageCode("shape_imported");
              }
            },
            false);
      },
      [this] { setRendering(false); });
}
void DesktopBackend::restoreShapeDraft(const QString &kind) {
  if (busy() || m_path.isEmpty())
    return;
  saveThen([this, kind] { command("restore_shape_draft", {{"kind", kind}}); });
}
void DesktopBackend::mutate(
    const std::function<void(QJsonObject &)> &mutation) {
  if (busy() || m_path.isEmpty())
    return;
  mutation(m_draft);
  ++m_editSequence;
  m_dirty = true;
  emit documentChanged();
  m_saveTimer.start();
}
void DesktopBackend::editText(const QString &text) {
  if (m_draft["text"].toString() != text)
    mutate([&](auto &d) { d.insert("text", text); });
}
void DesktopBackend::setFormat(const QString &format) {
  if (format == "plain" || format == "production_script")
    mutate([&](auto &d) { d.insert("format", format); });
}
void DesktopBackend::setVoice(const QString &alias) {
  mutate([&](auto &d) {
    d.insert("voice", alias);
    d.remove("voice_catalog_revision");
    d.remove("reference_voice");
  });
}
void DesktopBackend::setLanguage(const QString &language) {
  mutate([&](auto &d) { d.insert("language", language); });
}
void DesktopBackend::setSpeed(int speed) {
  mutate([&](auto &d) { d.insert("speed_milli", speed); });
}
void DesktopBackend::bindRole(const QString &role, const QString &alias) {
  mutate([&](auto &d) {
    auto options = d["options"].toObject();
    auto roles = options["roles"].toObject();
    auto references = options["reference_roles"].toObject();
    references.remove(role);
    options.insert("reference_roles", references);
    if (alias.isEmpty())
      roles.remove(role);
    else
      roles.insert(role,
                   QJsonObject{{"alias", alias},
                               {"catalog_revision",
                                "infer.speech.voice-aliases@20260922.1"}});
    options.insert("roles", roles);
    d.insert("options", options);
  });
}
void DesktopBackend::bindCue(const QString &label, const QString &action) {
  mutate([&](auto &d) {
    auto options = d["options"].toObject();
    auto cues = options["cues"].toObject();
    if (action == "unresolved")
      cues.remove(label);
    else
      cues.insert(label, QJsonObject{{"action", action}});
    options.insert("cues", cues);
    d.insert("options", options);
  });
}
void DesktopBackend::save() {
  if (!m_dirty || m_path.isEmpty() || busy())
    return;
  saveThen([] {});
}
void DesktopBackend::saveThen(std::function<void()> continuation,
                              std::function<void()> failed) {
  if (m_path.isEmpty() || !m_dirty) {
    continuation();
    return;
  }
  if (m_savePending) {
    m_saveContinuations.append(
        {m_projectEpoch, std::move(continuation), std::move(failed)});
    return;
  }
  m_saveTimer.stop();
  m_savePending = true;
  const auto edit = m_editSequence;
  const auto draft = m_draft;
  command("save", {{"document", draft}},
          [this, edit, continuation, failed](const QJsonObject &r) {
            m_savePending = false;
            const auto waiting = std::exchange(m_saveContinuations, {});
            if (r["ok"].toBool()) {
              apply(r["result"].toObject(), edit);
              if (m_dirty)
                saveThen(continuation, failed);
              else {
                messageCode("saved");
                continuation();
              }
              for (const auto &next : waiting) {
                if (next.epoch == m_projectEpoch)
                  saveThen(next.continuation, next.failed);
                else if (next.failed)
                  next.failed();
              }
            } else {
              if (failed)
                failed();
              for (const auto &next : waiting)
                if (next.failed)
                  next.failed();
            }
          });
}
void DesktopBackend::render() {
  if (busy() || m_path.isEmpty())
    return;
  setRendering(true);
  m_completed = 0;
  m_total = 0;
  emit progressChanged();
  saveThen(
      [this] {
        if (m_cancelRequested) {
          setRendering(false);
          messageCode("speech_cancelled");
          return;
        }
        if (!m_analysis["ready"].toBool()) {
          setRendering(false);
          messageCode("script_not_ready");
          return;
        }
        command("adopt", {}, [this](const QJsonObject &r) {
          if (!r["ok"].toBool()) {
            setRendering(false);
            return;
          }
          apply(r["result"].toObject(), m_editSequence);
          if (m_cancelRequested) {
            setRendering(false);
            messageCode("speech_cancelled");
            return;
          }
          const auto selection = m_selectionSequence;
          command("render", {}, [this, selection](const QJsonObject &r) {
            finishRender(r, false, selection);
          });
        });
      },
      [this] { setRendering(false); });
}
void DesktopBackend::cancel() {
  if (m_rendering) {
    m_cancelRequested = true;
    m_worker.cancel();
    messageCode("stop_requested");
  }
}
void DesktopBackend::finishRender(const QJsonObject &r, bool sample,
                                  quint64 selection) {
  setRendering(false);
  if (r["ok"].toBool()) {
    if (sample) {
      const auto result = r["result"].toObject();
      playPreview(QUrl::fromLocalFile(result["file"].toString()),
                  result["peaks"].toArray().toVariantList());
      messageCode("voice_sample");
    } else {
      apply(r["result"].toObject(), m_editSequence);
      auto candidates = m_project["candidates"].toArray();
      if (!candidates.isEmpty() &&
          (!selection || selection == m_selectionSequence))
        selectAudio(candidates.last().toObject()["id"].toString());
      messageCode("rendered");
    }
  } else if (!sample) {
    command("open", {});
  }
}
void DesktopBackend::auditionVoice(const QString &alias) {
  if (busy())
    return;
  setRendering(true);
  m_completed = 0;
  m_total = 1;
  emit progressChanged();
  const auto playback = ++m_playbackSequence;
  command(
      "voice_preview", {{"voice", alias}},
      [this, playback](const QJsonObject &r) {
        if (playback == m_playbackSequence)
          finishRender(r, true);
        else
          setRendering(false);
      },
      false);
}
void DesktopBackend::setRendering(bool rendering) {
  if (m_rendering == rendering)
    return;
  m_rendering = rendering;
  if (rendering)
    m_cancelRequested = false;
  emit renderingChanged();
  emit busyChanged();
}
void DesktopBackend::setProjectChanging(bool changing) {
  if (m_projectChanging == changing)
    return;
  m_projectChanging = changing;
  emit busyChanged();
}
void DesktopBackend::playPreview(const QUrl &file, const QVariantList &peaks) {
  m_selected.clear();
  ++m_selectionSequence;
  ++m_playbackSequence;
  m_preparedID.clear();
  m_preparedFile = QUrl{};
  m_waveform = peaks;
  emit selectionChanged();
  emit waveformChanged();
  emit playFile(file);
}
void DesktopBackend::selectAudio(const QString &id) {
  ++m_playbackSequence;
  if (m_selected == id && !id.isEmpty())
    return;
  m_selected = id;
  const auto selection = ++m_selectionSequence;
  m_waveform.clear();
  m_preparedID.clear();
  emit selectionChanged();
  emit waveformChanged();
  if (id.isEmpty())
    return;
  command("playback", {{"audio_id", id}},
          [this, id, selection](const QJsonObject &r) {
            if (r["ok"].toBool() && id == m_selected &&
                selection == m_selectionSequence) {
              const auto result = r["result"].toObject();
              m_preparedFile = QUrl::fromLocalFile(result["file"].toString());
              m_preparedID = id;
              m_waveform = result["peaks"].toArray().toVariantList();
              emit waveformChanged();
            }
          });
}
void DesktopBackend::playSelected() {
  if (m_selected.isEmpty() || m_path.isEmpty())
    return;
  const auto playback = ++m_playbackSequence;
  if (m_preparedID == m_selected) {
    emit playFile(m_preparedFile);
    return;
  }
  const auto id = m_selected;
  command("playback", {{"audio_id", id}},
          [this, id, playback](const QJsonObject &r) {
            if (r["ok"].toBool() && id == m_selected &&
                playback == m_playbackSequence)
              emit playFile(QUrl::fromLocalFile(
                  r["result"].toObject()["file"].toString()));
          });
}
void DesktopBackend::acceptSelected() {
  if (busy() || m_selected.isEmpty())
    return;
  if (!m_chapterID.isEmpty()) {
    acceptChapterSelected();
    return;
  }
  const auto id = m_selected;
  saveThen([this, id] {
    const auto edit = m_editSequence;
    command("accept", {{"audio_id", id}}, [this, edit](const QJsonObject &r) {
      if (r["ok"].toBool()) {
        apply(r["result"].toObject(), edit);
        messageCode("accepted");
      }
    });
  });
}
void DesktopBackend::discardSelected() {
  if (busy() || m_selected.isEmpty())
    return;
  const auto id = m_selected;
  const auto edit = m_editSequence;
  command("discard", {{"audio_id", id}},
          [this, id, edit](const QJsonObject &r) {
            if (r["ok"].toBool()) {
              apply(r["result"].toObject(), edit);
              if (m_selected == id)
                selectAudio(m_project["audio_head"].toString());
              messageCode("discarded");
            }
          });
}
void DesktopBackend::exportSelected(const QUrl &url) {
  exportAudio(m_selected, url);
}
void DesktopBackend::exportAudio(const QString &id, const QUrl &url) {
  if (id.isEmpty() || m_projectChanging)
    return;
  command("export", {{"audio_id", id}, {"destination", url.toLocalFile()}},
          [this](const QJsonObject &r) {
            if (r["ok"].toBool())
              messageCode("exported");
          });
}
void DesktopBackend::importScript(const QUrl &url) {
  if (busy() || m_path.isEmpty())
    return;
  saveThen(
      [this, url] { command("import_script", {{"file", url.toLocalFile()}}); });
}
void DesktopBackend::importCue(const QString &label, const QUrl &url) {
  if (busy() || m_path.isEmpty())
    return;
  saveThen([this, label, url] {
    command("import_cue", {{"label", label}, {"file", url.toLocalFile()}});
  });
}
void DesktopBackend::restoreSource(const QString &id) {
  if (busy() || m_path.isEmpty())
    return;
  saveThen([this, id] { command("restore_source", {{"source_id", id}}); });
}
void DesktopBackend::useTemplate(const QString &kind) {
  if (!m_draft["text"].toString().isEmpty())
    return;
  const auto text =
      kind == "dialogue" ? tr("[production: dialogue]\n[role: Host]\n[role: "
                              "Guest]\n\n[speaker: Host]\nWelcome. What would "
                              "you like to share?\n[pause: 0.4s]\n[speaker: "
                              "Guest]\nEvery story begins with a voice.")
      : kind == "listening"
          ? tr("[production: listening]\n[role: Narrator]\n[cue: turn; sound: "
               "chime]\n\n[speaker: Narrator]\nListen, then listen once "
               "more.\n[audio: turn]\n[repeat: 2; gap: 2s]\nA quiet morning. A "
               "new beginning.\n[end-repeat]")
          : tr("[production: narration]\n[role: Narrator]\n\n[speaker: "
               "Narrator]\nEvery story begins with a voice.\n[pause: "
               "0.8s]\nGive your words a moment to breathe.");
  mutate([&](auto &d) {
    d.insert("format", "production_script");
    d.insert("text", text);
  });
}
void DesktopBackend::probeRuntime() {
  if (m_probing)
    return;
  m_probing = true;
  m_probeWorker.request([] { return QJsonObject{{"action", "probe"}}; },
                        [this](const QJsonObject &r) {
                          m_probing = false;
                          const auto result = r["result"].toObject();
                          m_runtimeReady =
                              r["ok"].toBool() && result["ready"].toBool();
                          m_runtimeError = result["error"].toString();
                          if (!r["ok"].toBool())
                            m_runtimeError = r["error"].toString();
                          emit runtimeChanged();
                        });
}
void DesktopBackend::installCredential(const QUrl &url) {
  QFile input(url.toLocalFile());
  if (!input.open(QIODevice::ReadOnly) || input.size() > 4096) {
    messageCode("credential_unavailable");
    return;
  }
  const auto bytes = input.readAll().trimmed();
  if (bytes.isEmpty() || bytes.contains('\n') || bytes.contains('\r')) {
    messageCode("credential_unavailable");
    return;
  }
  const auto directory = QDir::homePath() + "/.config/vox";
  QDir().mkpath(directory);
  QFile::setPermissions(directory, QFileDevice::ReadOwner |
                                       QFileDevice::WriteOwner |
                                       QFileDevice::ExeOwner);
  const auto destination = directory + "/infer-credential";
  QFile output(destination);
  if (!output.open(QIODevice::WriteOnly | QIODevice::Truncate)) {
    messageCode("credential_unavailable");
    return;
  }
  output.setPermissions(QFileDevice::ReadOwner | QFileDevice::WriteOwner);
  if (output.write(bytes) != bytes.size()) {
    messageCode("credential_unavailable");
    return;
  }
  output.close();
  probeRuntime();
}
QString DesktopBackend::echoBinary() const {
  const auto custom =
      m_settings.value("echo/binary", qEnvironmentVariable("VOX_ECHO_BINARY"))
          .toString();
  if (!custom.isEmpty())
    return custom;
#ifdef VOX_DEV_ECHO_BINARY
  if (QFileInfo::exists(QStringLiteral(VOX_DEV_ECHO_BINARY)))
    return QStringLiteral(VOX_DEV_ECHO_BINARY);
#endif
#ifdef Q_OS_MACOS
  return "/Applications/Echo.app/Contents/MacOS/Echo";
#else
  return QStandardPaths::findExecutable("Echo");
#endif
}
void DesktopBackend::setEchoBinary(const QUrl &url) {
  m_settings.setValue("echo/binary", url.toLocalFile());
  emit stateChanged();
}
bool DesktopBackend::hasEchoHandoff() const {
  return !m_settings.value("echo/ticket/" + m_project["id"].toString())
              .toString()
              .isEmpty();
}
void DesktopBackend::editInEcho() {
  if (busy() || m_selected.isEmpty())
    return;
  if (m_echoOpening || m_echoLauncher.busy()) {
    messageCode("echo_busy");
    return;
  }
  if (!QFileInfo::exists(echoBinary())) {
    messageCode("echo_unavailable");
    return;
  }
  m_echoOpening = true;
  const auto parent =
      QStandardPaths::writableLocation(QStandardPaths::AppDataLocation) +
      "/handoffs";
  QDir().mkpath(parent);
  const auto directory =
      parent + "/" + QUuid::createUuid().toString(QUuid::WithoutBraces);
  if (!m_chapterID.isEmpty()) {
    beginChapterEcho(directory);
    return;
  }
  const auto edit = m_editSequence;
  command("begin_echo", {{"audio_id", m_selected}, {"destination", directory}},
          [this, edit](const QJsonObject &r) {
            if (!r["ok"].toBool()) {
              m_echoOpening = false;
              return;
            }
            const auto result = r["result"].toObject();
            apply(result, edit);
            m_settings.setValue("echo/ticket/" + m_project["id"].toString(),
                                result["handoff_file"].toString());
            emit stateChanged();
            m_echoLauncher.open(echoBinary(), result["source_file"].toString());
          });
}
void DesktopBackend::returnFromEcho(const QUrl &url, bool allowPCM24) {
  if (busy() || m_path.isEmpty())
    return;
  if (!m_chapterID.isEmpty()) {
    reviewChapterEcho(url, allowPCM24);
    return;
  }
  const auto ticket =
      m_settings.value("echo/ticket/" + m_project["id"].toString()).toString();
  const auto edit = m_editSequence;
  const auto selection = m_selectionSequence;
  command("review_echo",
          {{"handoff", ticket},
           {"file", url.toLocalFile()},
           {"allow_pcm24", allowPCM24}},
          [this, edit, selection](const QJsonObject &r) {
            if (r["ok"].toBool()) {
              apply(r["result"].toObject(), edit);
              const auto c = m_project["candidates"].toArray();
              if (!c.isEmpty() && selection == m_selectionSequence)
                selectAudio(c.last().toObject()["id"].toString());
              messageCode("echo_returned");
            }
          });
}
QString DesktopBackend::scriptRules() const {
  QFile f(":/vox/docs/SPEECH_SCRIPT.md");
  if (!f.open(QIODevice::ReadOnly))
    return {};
  return QString::fromUtf8(f.readAll());
}
void DesktopBackend::messageCode(const QString &code) {
  m_messageCode = code;
  emit messageChanged();
}
QString DesktopBackend::message() const {
  if (m_messageCode == "chapter_settings_shared")
    return tr("Shared voices, pronunciation and cue bindings updated. Changed "
              "chapters need generation.");
  if (m_messageCode == "chapters_completed")
    return tr("Chapter candidates are ready. Review and accept each chapter "
              "before export.");
  if (m_messageCode == "chapters_reused")
    return tr("All chapters were reused. No new candidates were generated.");
  if (m_messageCode == "chapters_cancelled")
    return tr("Chapter generation stopped. Completed candidates are kept.");
  if (m_messageCode == "chapters_partial_failed")
    return tr("Some chapters need attention. Completed candidates are kept.");
  if (m_messageCode == "chapters_exported")
    return tr("Accepted chapters exported with WAV, source metadata and "
              "available subtitles.");
  if (m_messageCode == "voice_design_completed")
    return tr("Voice candidates are ready. Listen, then save a named voice to "
              "reuse it.");
  if (m_messageCode == "voice_design_cancelled")
    return tr("Voice design stopped. Completed candidates are kept.");
  if (m_messageCode == "voice_design_partial_failed")
    return tr(
        "Some voices could not be generated. Completed candidates are kept.");
  if (m_messageCode == "voice_design_saved")
    return tr("Synthetic voice saved. Choose it for narration or a role in "
              "Voices & direction.");
  if (m_messageCode == "segment_rendered")
    return tr("Segment candidate ready. Listen and choose the take to use.");
  if (m_messageCode == "segment_accepted")
    return tr("Segment selected for assembly");
  if (m_messageCode == "segments_assembled")
    return tr("Assembled take ready for review. Earlier audio is kept.");
  if (m_messageCode == "subtitles_exported")
    return tr("Subtitles exported");
  if (m_messageCode == "sound_batch_completed")
    return tr("Sound variations are ready. Listen and keep your preferred "
              "candidates.");
  if (m_messageCode == "sound_batch_cancelled")
    return tr("Generation stopped. Completed sound candidates are kept.");
  if (m_messageCode == "sound_batch_partial_failed")
    return tr("Some variations could not be generated. Completed candidates "
              "are kept.");
  if (m_messageCode == "material_saved")
    return tr("Material saved in this project");
  if (m_messageCode == "sound_generated")
    return tr(
        "Sound candidate is ready. Listen, then accept it for script use.");
  if (m_messageCode == "saved")
    return tr("All changes saved");
  if (m_messageCode == "project_created")
    return tr("Project created");
  if (m_messageCode == "project_opened")
    return tr("Project opened");
  if (m_messageCode == "shape_imported")
    return tr("Shape material imported. Review preserved drafts and source "
              "receipts before continuing.");
  if (m_messageCode == "rendered")
    return tr("Your take is ready. Listen, then accept the version you want to "
              "keep.");
  if (m_messageCode == "accepted")
    return tr("Audio version accepted");
  if (m_messageCode == "discarded")
    return tr("Take discarded");
  if (m_messageCode == "exported")
    return tr("WAV exported");
  if (m_messageCode == "stop_requested" && m_designingVoice)
    return tr("Stopping after the current voice candidate…");
  if (m_messageCode == "stop_requested" && m_delivery->aligning())
    return tr("Waiting for the current alignment request to finish…");
  if (m_messageCode == "stop_requested")
    return tr("Stopping after the current segment…");
  if (m_messageCode == "voice_sample")
    return tr("Voice sample · synthesized audio");
  if (m_messageCode == "echo_opened")
    return tr("Echo is open. Export your edit as WAV, then return it here for "
              "review.");
  if (m_messageCode == "echo_returned")
    return tr(
        "Echo edit is ready for review. Accept it to keep a new version.");
  if (m_messageCode.isEmpty())
    return {};
  return errorText(m_messageCode);
}
QString DesktopBackend::errorText(const QString &code) const {
  if (code == "chapter_settings_changed")
    return tr(
        "This chapter uses different shared settings. Generate it with the "
        "current cast, or share this chapter’s voices before accepting it.");
  if (code == "chapter_missing" || code == "invalid_chapter")
    return tr("This chapter is unavailable. Refresh the chapter list.");
  if (code == "chapter_interrupted" || code == "chapter_retry_required")
    return tr("An earlier chapter run was interrupted. Review its state, then "
              "choose Resume interrupted.");
  if (code == "chapter_not_accepted" || code == "chapters_not_accepted" ||
      code == "chapter_audio_unaccepted")
    return tr(
        "Accept every selected chapter before exporting the chapter set.");
  if (code == "invalid_chapter_title" || code == "invalid_chapter_order")
    return tr("Give each chapter a title and keep every active chapter in the "
              "order.");
  if (code == "invalid_voice_design" || code == "invalid_voice_design_request")
    return tr("Describe a voice, enter sample words and choose one to four "
              "candidates.");
  if (code == "voice_design_missing")
    return tr(
        "This voice candidate is no longer available. Refresh the project.");
  if (code == "segments_not_accepted" || code == "segment_selection_missing")
    return tr("Choose an accepted take for every segment before assembly.");
  if (code == "segment_missing" || code == "segment_candidate_missing" ||
      code == "stale_segment")
    return tr("This segment changed. Review the current script and generate a "
              "new alternative.");
  if (code == "invalid_pronunciation")
    return tr("Check pronunciation rules for empty text, duplicate matches or "
              "unsupported filters.");
  if (code == "subtitles_unavailable")
    return tr(
        "Assemble a take from segments before exporting measured subtitles.");
  if (code == "shape_settings_incomplete")
    return tr("Some imported voice settings need review. Select supported "
              "voices or reimport missing references before rendering.");
  if (code == "invalid_shape_transfer" || code == "shape_transfer_path" ||
      code == "shape_transfer_hash" || code == "shape_transfer_contract")
    return tr("This Shape transfer is incomplete or changed. Export it again "
              "from Shape.");
  if (code == "shape_transfer_conflict")
    return tr("This project belongs to another transfer. Choose a new .vox "
              "location.");
  if (code == "shape_transfer_cancelled")
    return tr("Shape import cancelled. No project was created.");
  if (code == "voice_consent_required")
    return tr("Confirm that you may use this voice for local synthesis.");
  if (code == "reference_text_required")
    return tr("Name the voice and enter the exact words spoken in its "
              "reference recording.");
  if (code == "reference_duration")
    return tr(
        "Use a mono or stereo WAV reference between 0.5 and 120 seconds.");
  if (code == "reference_missing" || code == "invalid_reference")
    return tr("The reference voice is missing or changed. Import it again.");
  if (code == "clone_controls_unsupported")
    return tr(
        "Reference voices currently require 1× pace and neutral delivery.");
  if (code == "material_not_accepted")
    return tr("Listen and accept this sound before using it in a script.");
  if (code == "material_missing" || code == "invalid_material")
    return tr("This material is missing or invalid.");
  if (code == "invalid_sound_request")
    return tr(
        "Describe a sound in at most 2000 bytes and choose 1–30 seconds.");
  if (code == "prompt_preparation_failed")
    return tr("The sound description could not be prepared faithfully. Revise "
              "it or try again.");
  if (code == "sound_too_long")
    return tr("Use a mono or stereo sound of at most 120 seconds.");
  if (code == "generation_cancelled")
    return tr("Sound generation cancelled. No candidate was added.");

  if (code == "invalid_pause")
    return tr(
        "Use a pause between 0 and 120 seconds, for example [pause: 0.8s].");
  if (QStringList{"unclosed_repeat", "nested_repeat", "invalid_repeat",
                  "unexpected_repeat_end", "empty_repeat",
                  "scene_inside_repeat"}
          .contains(code))
    return tr("Check the repeat block: use one start and end, spoken content, "
              "and no nested repeat or scene boundary.");
  if (code == "undeclared_role" || code == "invalid_role")
    return tr(
        "Declare the role before using it, then choose its speaker voice.");
  if (code == "missing_speaker")
    return tr("Choose a speaker before the spoken passage.");
  if (code == "undeclared_cue" || code == "invalid_cue")
    return tr("Declare this sound cue before using it and choose a sound or "
              "skip it.");
  if (code == "declaration_position")
    return tr("Place production, role and cue declarations before the spoken "
              "content.");
  if (code == "invalid_production")
    return tr(
        "Choose narration, dialogue or listening as the production form.");
  if (code == "invalid_delivery")
    return tr("Choose neutral, clear, warm or lively delivery.");
  if (code == "unsupported_language")
    return tr("Choose a supported speech language or automatic detection.");
  if (code == "script_too_large" || code == "too_many_events")
    return tr("Shorten this script or split it into separate projects.");
  if (code == "unknown_directive" || code == "directive_line" ||
      code == "unsupported_markup")
    return tr("Use a supported script instruction on its own line. Open the "
              "script guide for examples.");
  if (code == "no_spoken_text")
    return tr("Add a spoken passage to the script.");
  if (code == "script_not_ready" || code == "invalid_speech_script")
    return tr("Check the script, choose a voice for every used role and "
              "resolve its sound cues.");
  if (code == "credential_unavailable")
    return tr(
        "Connect Vox to Infer Runtime with its own credential in Settings.");
  if (code == "stale_project" || code == "stale_source")
    return tr("This project changed. Reopen it to review the current version.");
  if (code == "file_error")
    return tr("The file could not be read or written. Check its location and "
              "permissions.");
  if (code == "destination_exists" || code == "project_exists")
    return tr("That location already exists. Choose a new name.");
  if (code == "speech_cancelled")
    return tr("Synthesis stopped. Existing audio is kept.");
  if (code == "cue_format")
    return tr(
        "Sound cues need a 24 kHz mono, 16-bit PCM WAV file, up to 8 MB.");
  if (code == "pcm24_confirmation_required")
    return tr("Allow conversion from 24-bit to 16-bit WAV to review this Echo "
              "export.");
  if (code == "echo_unavailable")
    return tr("Choose the Echo executable in Settings, then try again.");
  if (code == "echo_unsupported")
    return tr("This Echo build does not support independent editing. Choose a "
              "current Echo build in Settings.");
  if (code == "echo_busy")
    return tr("Echo is opening. Wait a moment, then try again.");
  if (code == "accept_before_echo")
    return tr("Accept this take before opening it in Echo.");
  if (code == "empty_script")
    return tr("Write or import a script first.");
  if (code == "render_busy")
    return tr("A voice is being rendered. Wait for it to finish or stop it.");
  if (code == "utf8_required")
    return tr("Choose a UTF-8 text file.");
  if (code == "project_extension")
    return tr("Use the .vox extension for this project.");
  if (code == "invalid_pcm24_return" || code == "invalid_audio_output")
    return tr("This audio format is unsupported. Export a PCM WAV from Echo.");
  if (code == "provider_unavailable" || code == "infer_unavailable" ||
      code == "infer_timeout" || code == "unavailable")
    return tr("Infer Runtime is unavailable. You can keep editing and "
              "listening to saved audio.");
  return tr("The operation could not be completed (%1).").arg(code);
}
QString DesktopBackend::voiceLabel(const QString &key) const {
  if (key == "vivian")
    return tr("Vivian · Bright");
  if (key == "serena")
    return tr("Serena · Warm");
  if (key == "uncle_fu")
    return tr("Uncle Fu · Mature");
  if (key == "dylan")
    return tr("Dylan · Beijing");
  if (key == "eric")
    return tr("Eric · Sichuan");
  if (key == "ryan")
    return tr("Ryan · Dynamic");
  if (key == "aiden")
    return tr("Aiden · Warm");
  if (key == "ono_anna")
    return tr("Ono Anna · Bright");
  if (key == "sohee")
    return tr("Sohee · Warm");
  return key;
}
QString DesktopBackend::languageLabel(const QString &key) const {
  if (key == "auto")
    return tr("Automatic");
  if (key == "Chinese")
    return tr("Chinese");
  if (key == "English")
    return tr("English");
  if (key == "Japanese")
    return tr("Japanese");
  if (key == "Korean")
    return tr("Korean");
  if (key == "German")
    return tr("German");
  if (key == "French")
    return tr("French");
  if (key == "Russian")
    return tr("Russian");
  if (key == "Portuguese")
    return tr("Portuguese");
  if (key == "Spanish")
    return tr("Spanish");
  if (key == "Italian")
    return tr("Italian");
  return key;
}
