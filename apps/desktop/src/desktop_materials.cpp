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
  if (m_rendering || m_path.isEmpty())
    return;
  m_rendering = true;
  emit renderingChanged();
  saveThen([this, id, text] {
    command("reference_preview", {{"id", id}, {"text", text}},
            [this](const QJsonObject &r) { finishRender(r, true); });
  });
}
void DesktopBackend::materialCommand(const QString &action,
                                     const QJsonObject &fields, bool generate) {
  if (m_rendering || m_path.isEmpty()) {
    messageCode("render_busy");
    return;
  }
  m_rendering = true;
  emit renderingChanged();
  saveThen([this, action, fields, generate] {
    command(action, fields, [this, generate](const QJsonObject &r) {
      m_rendering = false;
      emit renderingChanged();
      if (r["ok"].toBool()) {
        apply(r["result"].toObject(), m_editSequence);
        messageCode(generate ? "sound_generated" : "material_saved");
      }
    });
  });
}
void DesktopBackend::addBuiltin(const QString &key) {
  materialCommand("add_builtin", {{"key", key}});
}
void DesktopBackend::previewBuiltin(const QString &key) {
  if (m_rendering)
    return;
  command(
      "builtin_preview", {{"key", key}},
      [this](const QJsonObject &r) {
        if (r["ok"].toBool())
          emit playFile(
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
  if (!m_rendering && file.isLocalFile() &&
      QFileInfo(file.toLocalFile()).isFile())
    emit playFile(file);
}
void DesktopBackend::playMaterial(const QString &kind, const QString &id) {
  if (m_rendering)
    return;
  command("material_file", {{"kind", kind}, {"id", id}},
          [this](const QJsonObject &r) {
            if (r["ok"].toBool())
              emit playFile(QUrl::fromLocalFile(
                  r["result"].toObject()["file"].toString()));
          });
}
void DesktopBackend::exportMaterial(const QString &kind, const QString &id,
                                    const QUrl &destination) {
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
  if (m_rendering || m_echoOpening || m_echoLauncher.busy())
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
