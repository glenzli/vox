#include "desktop_backend.hpp"
#include <QFileInfo>
#include <algorithm>

void DesktopBackend::applyCollection(const QJsonObject &result) {
  const auto parent = result["parent_project"].toObject();
  if (!parent.isEmpty()) {
    m_collectionProject = parent;
    const auto path = result["parent_path"].toString();
    if (!path.isEmpty())
      m_collectionPath = path;
  } else if (result["path"].toString() == m_collectionPath &&
             !result["project"].toObject().isEmpty()) {
    m_collectionProject = result["project"].toObject();
  }
  m_chaptersError = result["chapters_error"].toString();
  if (!m_chaptersError.isEmpty()) {
    if (m_chapters.isEmpty())
      m_chapters = m_collectionProject["chapters"].toArray();
  } else if (result["chapters"].isArray())
    m_chapters = result["chapters"].toArray();
  if (m_chapterID.isEmpty() && m_path == m_collectionPath) {
    m_project = m_collectionProject;
    if (result["path"].toString() == m_collectionPath) {
      m_analysis = result["analysis"].toObject();
      m_segmentPlan = result["segment_plan"].toObject();
    }
    if (!m_dirty && m_draft != m_collectionProject["draft"].toObject()) {
      m_draft = m_collectionProject["draft"].toObject();
      emit documentChanged();
    }
  }
  emit stateChanged();
}

void DesktopBackend::collectionCommand(const QString &action,
                                       const QJsonObject &fields,
                                       WorkerClient::Callback done) {
  if (m_collectionPath.isEmpty())
    return;
  const auto path = m_collectionPath;
  const auto id = m_collectionProject["id"].toString();
  const auto epoch = m_collectionEpoch;
  const bool cancellable = action == "chapter_batch";
  ++m_pendingRequests;
  emit pendingChanged();
  m_worker.request(
      [this, action, fields, path, id, epoch] {
        if (epoch != m_collectionEpoch || path != m_collectionPath ||
            id != m_collectionProject["id"].toString())
          return QJsonObject{{"action", "capabilities"}};
        auto request = fields;
        request.insert("action", action);
        request.insert("path", path);
        request.insert("version", m_collectionProject["version"]);
        return request;
      },
      [this, action, done, path, id, epoch](const QJsonObject &response) {
        --m_pendingRequests;
        emit pendingChanged();
        if (epoch != m_collectionEpoch || path != m_collectionPath ||
            id != m_collectionProject["id"].toString()) {
          if (done)
            done({{"ok", false}, {"error", "stale_project"}});
          emit requestFinished(action, false);
          return;
        }
        auto r = response;
        if (r["error"].toString() == "request_cancelled")
          r.insert("error", "chapters_cancelled");
        const bool ok = r["ok"].toBool();
        if (!ok)
          messageCode(r["error"].toString());
        if (done)
          done(r);
        else if (ok)
          applyCollection(r["result"].toObject());
        emit requestFinished(action, ok);
      },
      cancellable);
}

void DesktopBackend::selectChapter(const QString &id) {
  if (id.isEmpty() || id == m_chapterID)
    return;
  if (busy()) {
    messageCode("render_busy");
    return;
  }
  const auto epoch = m_collectionEpoch;
  setProjectChanging(true);
  saveThen(
      [this, id, epoch] {
        if (epoch != m_collectionEpoch)
          return;
        collectionCommand("chapter_open", {{"chapter_id", id}},
                          [this, id, epoch](const QJsonObject &response) {
                            setProjectChanging(false);
                            if (epoch != m_collectionEpoch ||
                                !response["ok"].toBool())
                              return;
                            const auto result = response["result"].toObject();
                            applyCollection(result);
                            m_chapterID = id;
                            ++m_editSequence;
                            apply(result, m_editSequence, true);
                          });
      },
      [this] { setProjectChanging(false); });
}

void DesktopBackend::showMainScript() {
  if (m_chapterID.isEmpty())
    return;
  if (busy()) {
    messageCode("render_busy");
    return;
  }
  const auto epoch = m_collectionEpoch;
  const auto path = m_collectionPath;
  setProjectChanging(true);
  saveThen(
      [this, epoch, path] {
        command(
            "open", {{"path", path}},
            [this, epoch](const QJsonObject &r) {
              setProjectChanging(false);
              if (epoch == m_collectionEpoch && r["ok"].toBool()) {
                m_chapterID.clear();
                ++m_editSequence;
                apply(r["result"].toObject(), m_editSequence, true);
              }
            },
            false);
      },
      [this] { setProjectChanging(false); });
}

void DesktopBackend::refreshChapters() {
  if (!m_collectionPath.isEmpty() && !m_rendering)
    collectionCommand("chapter_list", {});
}

void DesktopBackend::addChapter(const QString &title, const QString &text,
                                const QString &format) {
  if (busy() || m_collectionPath.isEmpty() || title.trimmed().isEmpty())
    return;
  const auto epoch = m_collectionEpoch;
  saveThen([this, title, text, format, epoch] {
    if (epoch != m_collectionEpoch)
      return;
    collectionCommand(
        "chapter_add", {{"title", title}, {"text", text}, {"format", format}},
        [this, epoch](const QJsonObject &r) {
          if (epoch != m_collectionEpoch || !r["ok"].toBool())
            return;
          applyCollection(r["result"].toObject());
          if (!m_chapters.isEmpty())
            selectChapter(m_chapters.last().toObject()["id"].toString());
        });
  });
}

void DesktopBackend::importChapters(const QList<QUrl> &files) {
  if (busy() || m_collectionPath.isEmpty() || files.isEmpty())
    return;
  QJsonArray paths;
  for (const auto &file : files)
    if (file.isLocalFile())
      paths.append(file.toLocalFile());
  if (paths.isEmpty())
    return;
  const auto epoch = m_collectionEpoch;
  saveThen([this, paths, epoch] {
    if (epoch == m_collectionEpoch)
      collectionCommand("chapter_import", {{"files", paths}});
  });
}

void DesktopBackend::renameChapter(const QString &id, const QString &title) {
  if (!busy() && !id.isEmpty() && !title.trimmed().isEmpty())
    collectionCommand("chapter_rename", {{"chapter_id", id}, {"title", title}});
}

void DesktopBackend::moveChapter(const QString &id, int offset) {
  if (busy() || (offset != -1 && offset != 1))
    return;
  QJsonArray ids;
  int index = -1;
  for (const auto &value : m_chapters) {
    const auto chapter = value.toObject();
    if (chapter["archived"].toBool())
      continue;
    if (chapter["id"].toString() == id)
      index = ids.size();
    ids.append(chapter["id"]);
  }
  const auto destination = index + offset;
  if (index < 0 || destination < 0 || destination >= ids.size())
    return;
  const QJsonValue value = ids.at(index);
  ids[index] = ids.at(destination);
  ids[destination] = value;
  collectionCommand("chapter_reorder", {{"chapter_ids", ids}});
}

void DesktopBackend::archiveChapter(const QString &id) {
  if (busy() || id.isEmpty())
    return;
  const auto epoch = m_collectionEpoch;
  saveThen([this, id, epoch] {
    if (epoch != m_collectionEpoch)
      return;
    collectionCommand("chapter_archive", {{"chapter_id", id}},
                      [this, id](const QJsonObject &r) {
                        if (!r["ok"].toBool())
                          return;
                        applyCollection(r["result"].toObject());
                        if (id == m_chapterID)
                          showMainScript();
                      });
  });
}

void DesktopBackend::publishChapterSettings() {
  if (busy() || m_chapterID.isEmpty())
    return;
  const auto epoch = m_collectionEpoch;
  const auto chapter = m_chapterID;
  saveThen([this, epoch, chapter] {
    if (epoch == m_collectionEpoch)
      collectionCommand("chapter_shared_settings", {{"chapter_id", chapter}},
                        [this](const QJsonObject &r) {
                          if (r["ok"].toBool()) {
                            applyCollection(r["result"].toObject());
                            messageCode("chapter_settings_shared");
                          }
                        });
  });
}

void DesktopBackend::renderChapterBatch(bool retryInterrupted) {
  if (busy() || m_collectionPath.isEmpty()) {
    messageCode("render_busy");
    return;
  }
  const auto epoch = m_collectionEpoch;
  const auto chapter = m_chapterID;
  m_batchingChapters = true;
  m_chapterProgress = {};
  m_completed = 0;
  m_total = 0;
  setRendering(true);
  emit progressChanged();
  const auto failed = [this, epoch] {
    if (epoch != m_collectionEpoch)
      return;
    setRendering(false);
    m_batchingChapters = false;
    emit requestFinished("chapter_batch", false);
  };
  saveThen(
      [this, epoch, chapter, retryInterrupted, failed] {
        if (epoch != m_collectionEpoch)
          return;
        if (m_cancelRequested) {
          messageCode("chapters_cancelled");
          failed();
          return;
        }
        collectionCommand(
            "chapter_batch",
            {{"retry_interrupted", retryInterrupted}, {"changed_only", true}},
            [this, epoch, chapter](const QJsonObject &r) {
              if (epoch != m_collectionEpoch)
                return;
              setRendering(false);
              m_batchingChapters = false;
              if (r["ok"].toBool()) {
                const auto result = r["result"].toObject();
                applyCollection(result);
                const auto batch = result["chapter_batch"].toObject();
                const auto state = batch["status"].toString();
                const auto items = batch["chapters"].toArray();
                const bool allReused =
                    !items.isEmpty() &&
                    std::all_of(items.begin(), items.end(),
                                [](const QJsonValue &item) {
                                  return item.toObject()["status"].toString() ==
                                         "skipped";
                                });
                messageCode(state == "cancelled" ? "chapters_cancelled"
                            : state == "partial_failed" || state == "failed"
                                ? "chapters_partial_failed"
                            : state == "completed" && allReused
                                ? "chapters_reused"
                                : "chapters_completed");
              } else {
                refreshChapters();
              }
              if (!chapter.isEmpty() && chapter == m_chapterID) {
                m_chapterID.clear();
                selectChapter(chapter);
              }
            });
      },
      failed);
}

void DesktopBackend::exportChapters(const QUrl &destination) {
  QJsonArray accepted;
  for (const auto &value : m_chapters) {
    const auto chapter = value.toObject();
    if (!chapter["archived"].toBool() &&
        chapter["status"].toString() == "accepted")
      accepted.append(chapter["id"]);
  }
  if (!busy() && destination.isLocalFile() && !accepted.isEmpty()) {
    const auto epoch = m_collectionEpoch;
    saveThen([this, destination, accepted, epoch] {
      if (epoch == m_collectionEpoch)
        collectionCommand("chapter_export",
                          {{"destination", destination.toLocalFile()},
                           {"chapter_ids", accepted}},
                          [this](const QJsonObject &r) {
                            if (r["ok"].toBool())
                              messageCode("chapters_exported");
                          });
    });
  }
}

QString DesktopBackend::productionStatus(const QString &status) const {
  if (status == "unavailable")
    return tr("Unavailable");
  if (status == "running" || status == "rendering")
    return tr("Generating");
  if (status == "completed" || status == "candidate_ready" ||
      status == "ready_for_review" || status == "review")
    return tr("For review");
  if (status == "accepted")
    return tr("Accepted");
  if (status == "skipped")
    return tr("Reused");
  if (status == "cancelled")
    return tr("Stopped");
  if (status == "partial_failed" || status == "partial")
    return tr("Partly completed");
  if (status == "failed")
    return tr("Failed");
  if (status == "interrupted_unknown" || status == "interrupted")
    return tr("Interrupted · review before retry");
  if (status == "changed" || status == "needs_generation" ||
      status == "stale" || status == "settings_changed")
    return tr("Changed");
  if (status == "needs_script")
    return tr("Needs script");
  if (status == "archived")
    return tr("Removed");
  return tr("Pending");
}

QString DesktopBackend::chapterError() const {
  return m_chaptersError.isEmpty()
             ? QString{}
             : tr("Chapter state is unavailable. The chapter list is kept. "
                  "Check the project files, then refresh.");
}

void DesktopBackend::acceptChapterSelected() {
  const auto id = m_selected;
  const auto chapter = m_chapterID;
  const auto epoch = m_projectEpoch;
  saveThen([this, id, chapter, epoch] {
    if (epoch != m_projectEpoch || chapter != m_chapterID)
      return;
    const auto edit = m_editSequence;
    collectionCommand("chapter_accept",
                      {{"chapter_id", chapter}, {"audio_id", id}},
                      [this, chapter, epoch, edit](const QJsonObject &r) {
                        if (!r["ok"].toBool())
                          return;
                        const auto result = r["result"].toObject();
                        applyCollection(result);
                        if (epoch == m_projectEpoch && chapter == m_chapterID) {
                          apply(result, edit);
                          messageCode("accepted");
                        }
                      });
  });
}

void DesktopBackend::beginChapterEcho(const QString &directory) {
  const auto chapter = m_chapterID;
  const auto id = m_selected;
  const auto project = m_project["id"].toString();
  const auto epoch = m_projectEpoch;
  const auto edit = m_editSequence;
  collectionCommand(
      "chapter_begin_echo",
      {{"chapter_id", chapter}, {"audio_id", id}, {"destination", directory}},
      [this, chapter, project, epoch, edit](const QJsonObject &r) {
        if (!r["ok"].toBool()) {
          m_echoOpening = false;
          return;
        }
        const auto result = r["result"].toObject();
        applyCollection(result);
        m_settings.setValue("echo/ticket/" + project,
                            result["handoff_file"].toString());
        if (epoch == m_projectEpoch && chapter == m_chapterID)
          apply(result, edit);
        emit stateChanged();
        m_echoLauncher.open(echoBinary(), result["source_file"].toString());
      });
}

void DesktopBackend::reviewChapterEcho(const QUrl &url, bool allowPCM24) {
  const auto chapter = m_chapterID;
  const auto ticket =
      m_settings.value("echo/ticket/" + m_project["id"].toString()).toString();
  const auto epoch = m_projectEpoch;
  const auto edit = m_editSequence;
  const auto selection = m_selectionSequence;
  collectionCommand(
      "chapter_review_echo",
      {{"chapter_id", chapter},
       {"handoff", ticket},
       {"file", url.toLocalFile()},
       {"allow_pcm24", allowPCM24}},
      [this, chapter, epoch, edit, selection](const QJsonObject &r) {
        if (!r["ok"].toBool())
          return;
        const auto result = r["result"].toObject();
        applyCollection(result);
        if (epoch != m_projectEpoch || chapter != m_chapterID)
          return;
        apply(result, edit);
        const auto candidates = m_project["candidates"].toArray();
        if (!candidates.isEmpty() && selection == m_selectionSequence)
          selectAudio(candidates.last().toObject()["id"].toString());
        messageCode("echo_returned");
      });
}
