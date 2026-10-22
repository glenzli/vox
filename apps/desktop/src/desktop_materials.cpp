#include "desktop_backend.hpp"
#include <QFileInfo>

void DesktopBackend::importReference(const QString &name,
                                     const QString &transcript,
                                     const QUrl &file, bool consent,
                                     bool recorded) {
  materialCommand("import_reference", {{"name", name},
                                       {"transcript", transcript},
                                       {"file", file.toLocalFile()},
                                       {"consent", consent},
                                       {"recorded", recorded}});
}
void DesktopBackend::bindReference(const QString &role, const QString &id) {
  const auto library = m_project["voice_library"].toArray();
  for (const auto &item : library) {
    const auto voice = item.toObject();
    if (voice["id"].toString() != id)
      continue;
    mutate([&](auto &d) {
      d.insert("speed_milli", 1000);
      if (role.isEmpty())
        d.insert("reference_voice", voice["reference"]);
      else {
        auto options = d["options"].toObject();
        auto roles = options["roles"].toObject();
        auto refs = options["reference_roles"].toObject();
        roles.remove(role);
        refs.insert(role, voice["reference"]);
        options.insert("roles", roles);
        options.insert("reference_roles", refs);
        d.insert("options", options);
      }
    });
    return;
  }
  messageCode("reference_missing");
}
void DesktopBackend::auditionReference(const QString &id, const QString &text) {
  if (busy() || m_path.isEmpty())
    return;
  setRendering(true);
  const auto playback = ++m_playbackSequence;
  saveThen(
      [this, id, text, playback] {
        if (m_cancelRequested) {
          setRendering(false);
          messageCode("speech_cancelled");
          return;
        }
        command("reference_preview", {{"id", id}, {"text", text}},
                [this, playback](const QJsonObject &r) {
                  if (playback == m_playbackSequence)
                    finishRender(r, true);
                  else
                    setRendering(false);
                });
      },
      [this] { setRendering(false); });
}
void DesktopBackend::materialCommand(const QString &action,
                                     const QJsonObject &fields, bool generate) {
  if (busy() || m_path.isEmpty()) {
    messageCode("render_busy");
    return;
  }
  if (generate)
    setRendering(true);
  saveThen(
      [this, action, fields, generate] {
        if (generate && m_cancelRequested) {
          setRendering(false);
          messageCode("generation_cancelled");
          emit requestFinished(action, false);
          return;
        }
        const auto edit = m_editSequence;
        command(action, fields, [this, generate, edit](const QJsonObject &r) {
          if (generate)
            setRendering(false);
          if (r["ok"].toBool()) {
            const auto result = r["result"].toObject();
            apply(result, edit);
            const auto batch = result["sound_batch"].toObject();
            if (!batch.isEmpty()) {
              const auto state = batch["state"].toString();
              messageCode(state == "cancelled" ? "sound_batch_cancelled"
                          : state == "partial_failed"
                              ? "sound_batch_partial_failed"
                              : "sound_batch_completed");
            } else {
              messageCode(generate ? "sound_generated" : "material_saved");
            }
          }
        });
      },
      [this, action, generate] {
        if (generate)
          setRendering(false);
        emit requestFinished(action, false);
      });
}
void DesktopBackend::addBuiltin(const QString &key) {
  materialCommand("add_builtin", {{"key", key}});
}
void DesktopBackend::previewBuiltin(const QString &key) {
  if (busy())
    return;
  const auto playback = ++m_playbackSequence;
  command(
      "builtin_preview", {{"key", key}},
      [this, playback](const QJsonObject &r) {
        if (r["ok"].toBool() && playback == m_playbackSequence)
          playPreview(
              QUrl::fromLocalFile(r["result"].toObject()["file"].toString()));
      },
      false);
}
void DesktopBackend::importSound(const QUrl &url) {
  materialCommand("import_sound",
                  {{"file", url.toLocalFile()},
                   {"name", QFileInfo(url.toLocalFile()).completeBaseName()}});
}
void DesktopBackend::generateSound(const QString &name, const QString &prompt,
                                   int seconds, int seed, bool music) {
  materialCommand("generate_sound",
                  {{"name", name},
                   {"spec", QJsonObject{{"prompt", prompt},
                                        {"seconds", seconds},
                                        {"seed", seed},
                                        {"music", music}}}},
                  true);
}
void DesktopBackend::generateSoundBatch(const QString &name,
                                        const QString &prompt, int seconds,
                                        int seed, const QString &category,
                                        int variations) {
  materialCommand("generate_sound_batch",
                  {{"name", name},
                   {"category", category},
                   {"variations", variations},
                   {"spec", QJsonObject{{"prompt", prompt},
                                        {"seconds", seconds},
                                        {"seed", seed},
                                        {"music", category == "short_music"}}}},
                  true);
}
void DesktopBackend::acceptSound(const QString &id) {
  materialCommand("accept_sound", {{"id", id}});
}
void DesktopBackend::discardSound(const QString &id) {
  materialCommand("discard_sound", {{"id", id}});
}
void DesktopBackend::bindSound(const QString &label, const QString &id) {
  for (const auto &item : m_project["sound_library"].toArray()) {
    const auto sound = item.toObject();
    if (sound["id"].toString() != id || !sound["content"].isObject())
      continue;
    mutate([&](auto &d) {
      auto options = d["options"].toObject();
      auto cues = options["cues"].toObject();
      cues.insert(label, QJsonObject{{"action", "audio"},
                                     {"content", sound["content"]}});
      options.insert("cues", cues);
      d.insert("options", options);
    });
    return;
  }
  messageCode("material_not_accepted");
}
void DesktopBackend::previewFile(const QUrl &file) {
  if (!busy() && file.isLocalFile() && QFileInfo(file.toLocalFile()).isFile())
    playPreview(file);
}
void DesktopBackend::playMaterial(const QString &kind, const QString &id) {
  if (busy() || m_path.isEmpty())
    return;
  const auto playback = ++m_playbackSequence;
  command("material_file", {{"kind", kind}, {"id", id}},
          [this, playback](const QJsonObject &r) {
            if (r["ok"].toBool() && playback == m_playbackSequence)
              playPreview(QUrl::fromLocalFile(
                  r["result"].toObject()["file"].toString()));
          });
}
void DesktopBackend::exportMaterial(const QString &kind, const QString &id,
                                    const QUrl &destination) {
  if (m_path.isEmpty() || m_projectChanging)
    return;
  command(
      "export_material",
      {{"kind", kind}, {"id", id}, {"destination", destination.toLocalFile()}},
      [this](const QJsonObject &r) {
        if (r["ok"].toBool())
          messageCode("exported");
      });
}
void DesktopBackend::editMaterialInEcho(const QString &kind,
                                        const QString &id) {
  if (busy() || m_path.isEmpty() || m_echoOpening || m_echoLauncher.busy())
    return;
  m_echoOpening = true;
  command("material_echo", {{"kind", kind}, {"id", id}},
          [this](const QJsonObject &r) {
            if (!r["ok"].toBool()) {
              m_echoOpening = false;
              return;
            }
            m_echoLauncher.open(echoBinary(),
                                r["result"].toObject()["file"].toString());
          });
}
QString DesktopBackend::soundLabel(const QString &key) const {
  if (key == "soft_chime")
    return tr("Soft chime");
  if (key == "double_beep")
    return tr("Double beep");
  if (key == "click")
    return tr("Click");
  if (key == "wood_block")
    return tr("Wood block");
  if (key == "bell")
    return tr("Bell");
  if (key == "success")
    return tr("Success");
  if (key == "whoosh")
    return tr("Whoosh");
  if (key == "countdown")
    return tr("Countdown");
  return key;
}

QVariantList DesktopBackend::voices() const {
  QVariantList result;
  for (const auto &value : m_voices) {
    auto voice = value.toObject();
    voice.insert("label", voiceLabel(voice["key"].toString()));
    result.append(voice.toVariantMap());
  }
  return result;
}
QVariantList DesktopBackend::builtinSounds() const {
  QVariantList result;
  for (const auto &key :
       QStringList{"soft_chime", "double_beep", "click", "wood_block", "bell",
                   "success", "whoosh", "countdown"})
    result.append(QVariantMap{{"key", key}, {"label", soundLabel(key)}});
  return result;
}
QVariantList DesktopBackend::languages() const {
  QVariantList result;
  for (const auto &key :
       QStringList{"auto", "Chinese", "English", "Japanese", "Korean", "German",
                   "French", "Russian", "Portuguese", "Spanish", "Italian"})
    result.append(QVariantMap{{"key", key}, {"label", languageLabel(key)}});
  return result;
}
