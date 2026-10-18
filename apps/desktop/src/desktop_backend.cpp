#include "desktop_backend.hpp"
#include <QCoreApplication>
#include <QDir>
#include <QFile>
#include <QFileInfo>
#include <QJsonDocument>
#include <QStandardPaths>
#include <QUuid>

DesktopBackend::DesktopBackend(QObject *parent)
    : QObject(parent), m_worker(this), m_probeWorker(this),
      m_echoLauncher(this) {
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
}
void DesktopBackend::command(const QString &action, const QJsonObject &fields,
                             WorkerClient::Callback done, bool includeProject) {
  const auto path = m_path;
  const auto projectID = m_project["id"].toString();
  const auto edit = m_editSequence;
  m_worker.request(
      [this, action, fields, includeProject, path, projectID] {
        if (includeProject &&
            (path != m_path || projectID != m_project["id"].toString()))
          return QJsonObject{{"action", "capabilities"}};
        auto request = fields;
        request.insert("action", action);
        if (includeProject) {
          request.insert("path", path);
          request.insert("version", m_project["version"]);
        }
        return request;
      },
      [this, action, done, edit, path, includeProject,
       projectID](const QJsonObject &r) {
        if (includeProject &&
            (path != m_path || projectID != m_project["id"].toString())) {
          if (done)
            done(QJsonObject{{"ok", false}, {"error", "stale_project"}});
          emit requestFinished(action, false);
          return;
        }
        const bool ok = r["ok"].toBool();
        if (!ok)
          messageCode(r["error"].toString());
        if (done)
          done(r);
        else if (ok && r["result"].toObject().contains("project"))
          apply(r["result"].toObject(), edit);
        emit requestFinished(action, ok);
      });
}
void DesktopBackend::apply(const QJsonObject &result, quint64 capturedEdit,
                           bool reset) {
  const auto project = result["project"].toObject();
  if (project.isEmpty())
    return;
  m_project = project;
  m_path = result["path"].toString();
  m_analysis = result["analysis"].toObject();
  if (reset || capturedEdit == m_editSequence) {
    m_draft = project["draft"].toObject();
    m_dirty = false;
    emit documentChanged();
  }
  if (reset) {
    m_selected.clear();
    m_preparedID.clear();
    m_waveform.clear();
    emit selectionChanged();
    emit waveformChanged();
    selectAudio(project["audio_head"].toString());
  }
  QStringList recent = m_settings.value("recentProjects").toStringList();
  recent.removeAll(m_path);
  recent.prepend(m_path);
  while (recent.size() > 12)
    recent.removeLast();
  m_settings.setValue("recentProjects", recent);
  emit stateChanged();
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
  if (m_rendering)
    return;
  saveThen([this, url] {
    const auto path = url.toLocalFile();
    command(
        "create",
        {{"path", path}, {"title", QFileInfo(path).completeBaseName()}},
        [this](const QJsonObject &r) {
          if (r["ok"].toBool()) {
            ++m_editSequence;
            apply(r["result"].toObject(), m_editSequence, true);
            messageCode("project_created");
          }
        },
        false);
  });
}
void DesktopBackend::openProject(const QUrl &url) {
  if (m_rendering)
    return;
  saveThen([this, url] {
    command(
        "open", {{"path", url.toLocalFile()}},
        [this](const QJsonObject &r) {
          if (r["ok"].toBool()) {
            ++m_editSequence;
            apply(r["result"].toObject(), m_editSequence, true);
            messageCode("project_opened");
          }
        },
        false);
  });
}
void DesktopBackend::mutate(
    const std::function<void(QJsonObject &)> &mutation) {
  if (m_rendering || m_path.isEmpty())
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
  if (!m_dirty || m_path.isEmpty() || m_rendering)
    return;
  saveThen([] {});
}
void DesktopBackend::saveThen(std::function<void()> continuation) {
  if (m_path.isEmpty() || !m_dirty) {
    continuation();
    return;
  }
  if (m_savePending) {
    QTimer::singleShot(40, this,
                       [this, continuation] { saveThen(continuation); });
    return;
  }
  m_saveTimer.stop();
  m_savePending = true;
  const auto edit = m_editSequence;
  const auto draft = m_draft;
  command("save", {{"document", draft}},
          [this, edit, continuation](const QJsonObject &r) {
            m_savePending = false;
            if (r["ok"].toBool()) {
              apply(r["result"].toObject(), edit);
              if (m_dirty)
                saveThen(continuation);
              else {
                messageCode("saved");
                continuation();
              }
            } else {
              m_rendering = false;
              emit renderingChanged();
            }
          });
}
void DesktopBackend::render() {
  if (m_rendering || m_path.isEmpty())
    return;
  m_rendering = true;
  m_completed = 0;
  m_total = 0;
  emit renderingChanged();
  emit progressChanged();
  saveThen([this] {
    command("adopt", {}, [this](const QJsonObject &r) {
      if (!r["ok"].toBool()) {
        m_rendering = false;
        emit renderingChanged();
        return;
      }
      apply(r["result"].toObject(), m_editSequence);
      command("render", {},
              [this](const QJsonObject &r) { finishRender(r, false); });
    });
  });
}
void DesktopBackend::cancel() {
  if (m_rendering) {
    m_worker.cancel();
    messageCode("stop_requested");
  }
}
void DesktopBackend::finishRender(const QJsonObject &r, bool sample) {
  m_rendering = false;
  emit renderingChanged();
  if (r["ok"].toBool()) {
    if (sample) {
      selectAudio("");
      const auto result = r["result"].toObject();
      m_waveform = result["peaks"].toArray().toVariantList();
      emit waveformChanged();
      emit playFile(QUrl::fromLocalFile(result["file"].toString()));
      messageCode("voice_sample");
    } else {
      apply(r["result"].toObject(), m_editSequence);
      auto candidates = m_project["candidates"].toArray();
      if (!candidates.isEmpty())
        selectAudio(candidates.last().toObject()["id"].toString());
      messageCode("rendered");
    }
  } else if (!sample) {
    command("open", {});
  }
}
void DesktopBackend::auditionVoice(const QString &alias) {
  if (m_rendering)
    return;
  m_rendering = true;
  emit renderingChanged();
  m_completed = 0;
  m_total = 1;
  emit progressChanged();
  command(
      "voice_preview", {{"voice", alias}},
      [this](const QJsonObject &r) { finishRender(r, true); }, false);
}
void DesktopBackend::selectAudio(const QString &id) {
  if (m_selected == id)
    return;
  m_selected = id;
  m_waveform.clear();
  m_preparedID.clear();
  emit selectionChanged();
  emit waveformChanged();
  if (id.isEmpty())
    return;
  command("playback", {{"audio_id", id}}, [this, id](const QJsonObject &r) {
    if (r["ok"].toBool() && id == m_selected) {
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
  if (m_preparedID == m_selected) {
    emit playFile(m_preparedFile);
    return;
  }
  const auto id = m_selected;
  command("playback", {{"audio_id", id}}, [this, id](const QJsonObject &r) {
    if (r["ok"].toBool() && id == m_selected)
      emit playFile(
          QUrl::fromLocalFile(r["result"].toObject()["file"].toString()));
  });
}
void DesktopBackend::acceptSelected() {
  if (m_rendering || m_selected.isEmpty())
    return;
  saveThen([this] {
    command("accept", {{"audio_id", m_selected}}, [this](const QJsonObject &r) {
      if (r["ok"].toBool()) {
        apply(r["result"].toObject(), m_editSequence);
        messageCode("accepted");
      }
    });
  });
}
void DesktopBackend::discardSelected() {
  if (m_rendering || m_selected.isEmpty())
    return;
  command("discard", {{"audio_id", m_selected}}, [this](const QJsonObject &r) {
    if (r["ok"].toBool()) {
      apply(r["result"].toObject(), m_editSequence);
      selectAudio(m_project["audio_head"].toString());
      messageCode("discarded");
    }
  });
}
void DesktopBackend::exportSelected(const QUrl &url) {
  if (m_selected.isEmpty())
    return;
  command("export",
          {{"audio_id", m_selected}, {"destination", url.toLocalFile()}},
          [this](const QJsonObject &r) {
            if (r["ok"].toBool())
              messageCode("exported");
          });
}
void DesktopBackend::importScript(const QUrl &url) {
  saveThen(
      [this, url] { command("import_script", {{"file", url.toLocalFile()}}); });
}
void DesktopBackend::importCue(const QString &label, const QUrl &url) {
  saveThen([this, label, url] {
    command("import_cue", {{"label", label}, {"file", url.toLocalFile()}});
  });
}
void DesktopBackend::restoreSource(const QString &id) {
  if (m_rendering)
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
  if (m_rendering || m_selected.isEmpty())
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
  command("begin_echo", {{"audio_id", m_selected}, {"destination", directory}},
          [this](const QJsonObject &r) {
            if (!r["ok"].toBool()) {
              m_echoOpening = false;
              return;
            }
            const auto result = r["result"].toObject();
            apply(result, m_editSequence);
            m_settings.setValue("echo/ticket/" + m_project["id"].toString(),
                                result["handoff_file"].toString());
            emit stateChanged();
            m_echoLauncher.open(echoBinary(), result["source_file"].toString());
          });
}
void DesktopBackend::returnFromEcho(const QUrl &url, bool allowPCM24) {
  if (m_rendering)
    return;
  const auto ticket =
      m_settings.value("echo/ticket/" + m_project["id"].toString()).toString();
  command("review_echo",
          {{"handoff", ticket},
           {"file", url.toLocalFile()},
           {"allow_pcm24", allowPCM24}},
          [this](const QJsonObject &r) {
            if (r["ok"].toBool()) {
              apply(r["result"].toObject(), m_editSequence);
              const auto c = m_project["candidates"].toArray();
              if (!c.isEmpty())
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
  if (m_messageCode == "rendered")
    return tr("Your take is ready. Listen, then accept the version you want to "
              "keep.");
  if (m_messageCode == "accepted")
    return tr("Audio version accepted");
  if (m_messageCode == "discarded")
    return tr("Take discarded");
  if (m_messageCode == "exported")
    return tr("WAV exported");
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
