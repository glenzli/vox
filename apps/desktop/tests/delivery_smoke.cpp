#include "desktop_backend.hpp"
#include "desktop_delivery.hpp"
#include "ui_preferences.hpp"
#include "workflow_smoke.hpp"
#include <QDir>
#include <QEventLoop>
#include <QFile>
#include <QGuiApplication>
#include <QJsonDocument>
#include <QKeyEvent>
#include <QKeySequence>
#include <QMouseEvent>
#include <QQmlApplicationEngine>
#include <QQuickItem>
#include <QQuickWindow>
#include <QTimer>
#include <QUuid>

QString prepareDeliverySmoke(const QString &output) {
  QDir().mkpath(output);
  QFile::remove(output + "/subtitle-mutations.jsonl");
  QFile worker(output + "/delivery-worker.py");
  if (!worker.open(QIODevice::WriteOnly) || worker.write(R"PY(#!/usr/bin/python3
import json, os, select, sys
path, project, candidate, accepted = '', {}, None, None
chapters = [{'id':'one','title':'First chapter','status':'accepted','archived':False},
            {'id':'two','title':'Second chapter','status':'accepted','archived':False}]
def snapshot():
    return {'path':path,'project':project,'analysis':{'ready':True},'segment_plan':{},'chapters':chapters}
def track(value):
    source = value.get('source','forced_alignment')
    return {'audio_revision':'audio-main','audio_digest':'pinned-audio','source_revision':'pinned-source',
            'source':source,'revision_id':value['id'],
            'alignment_revision':value['id'] if source == 'forced_alignment' else None,
            'language':value.get('language'),
            'cues':value.get('cues',[{'start_ms':100,'end_ms':900,'text':value['transcript']}])}
for line in sys.stdin:
    r = json.loads(line)
    action, result, error = r['action'], {}, ''
    if action == 'create':
        path = r['path']
        project = {'id':path,'version':0,'title':'Delivery fixture',
                   'draft':{'text':'The authored script stays exact.','format':'plain','language':'English','voice':'ryan','speed_milli':1000},
                   'audio_head':'audio-main','candidates':[],
                   'audio':[{'id':name,'origin':'echo_file_return','contract':{'frame_count':48000,'sample_rate_hz':48000,'channels':2}}
                            for name in ['audio-main','other-audio']]}
        result = snapshot()
    elif action == 'capabilities': result = {'voices':[]}
    elif action == 'probe': result = {'ready':True}
    elif action == 'save':
        project['draft'] = r['document']; project['version'] += 1
        result = snapshot()
    elif action == 'open':
        path = r['path']; project['id'] = path
        result = snapshot()
    elif action == 'chapter_list': result = snapshot()
    elif action == 'playback': result = {'file':'/nonexistent/delivery-audio.wav','peaks':[]}
    elif action == 'subtitle_preview':
        result = snapshot()
        shown = candidate if r.get('candidate_id') and candidate else accepted
        result.update({'audio_id':r['audio_id'],'track':track(shown) if shown else None,
                       'track_error':None if shown else 'subtitles_unavailable',
                       'transcript':'Pinned source words before the Echo cut.','language':'English',
                       'accepted_audio':True,'duration_ms':1000,'candidates':[candidate] if candidate else [],'runs':[]})
    elif action == 'align_subtitles':
        print(json.dumps({'request_id':r['request_id'],'event':'progress','completed':0,'total':1}),flush=True)
        cancelled = False
        if select.select([sys.stdin],[],[],0.25)[0]:
            stop = json.loads(sys.stdin.readline())
            cancelled = stop['action'] == 'cancel'
            print(json.dumps({'request_id':stop['request_id'],'ok':True,'result':{}}),flush=True)
        candidate = {'id':'candidate-2' if accepted else 'candidate-1',
                     'audio_revision':'audio-main','source':'forced_alignment',
                     'transcript':r['spec']['transcript'],'language':r['spec']['language']}
        project['version'] += 1
        result = snapshot()
        result['subtitle_outcome'] = {'state':'cancelled' if cancelled else 'completed','candidate_id':candidate['id'],'run_id':'run'}
    elif action in ['subtitle_edit','subtitle_import']:
        if action == 'subtitle_edit':
            cues = r['cues']; source = 'manual_revision'
        else:
            with open(r['file'],'rb') as f: data = f.read()
            words = data.decode('utf-8-sig').replace('\r','').strip().split('\n')[2:]
            cues = [{'start_ms':150,'end_ms':850,'text':'\n'.join(words)}]
            source = 'imported_subtitles'
        candidate = {'id':'manual-1' if action == 'subtitle_edit' else 'imported-1',
                     'audio_revision':r['audio_id'],'source':source,'cues':cues,
                     'language':r.get('language'),'transcript':'\n'.join(c['text'] for c in cues)}
        project['version'] += 1
        result = snapshot(); result['subtitle_candidate_id'] = candidate['id']
        with open(os.path.join(os.path.dirname(path),'subtitle-mutations.jsonl'),'a') as f: f.write(json.dumps(r)+'\n')
    elif action in ['accept_subtitles','discard_subtitles']:
        if not candidate or r['id'] != candidate['id']: error = 'subtitle_candidate_missing'
        else:
            if action == 'accept_subtitles': accepted = candidate
            candidate = None; project['version'] += 1
        result = snapshot()
    elif action == 'export_subtitles':
        if not accepted: error = 'subtitles_unavailable'
        else:
            with open(r['destination'],'w') as f: f.write('1\n00:00:00,100 --> 00:00:00,900\n'+accepted['transcript']+'\n')
            result = {'exported':True}
    elif action == 'chapter_compilation_preflight':
        options = r['options']; issues = []
        if options['audio_mode'] == 'preserve': issues.append('compilation_format_mismatch')
        if options['subtitles'] == 'required': issues.append('subtitles_unavailable')
        result = {'schema':'vox.chapter-compilation-preflight/1','project_id':project['id'],
                  'project_version':project['version'],'ready':not issues,'fingerprint':'reviewed-fingerprint',
                  'issues':issues,'options':options,
                  'chapters':[{'chapter_id':id,'title':next(c['title'] for c in chapters if c['id']==id),
                               'issues':issues if id=='two' else []} for id in r['chapter_ids']],
                  'output':{'duration_ms':2250,'sample_rate_hz':24000,'channels':1,'frames':54000,'subtitle_cues':0} if not issues else None}
    elif action == 'chapter_compile':
        if r.get('fingerprint') != 'reviewed-fingerprint': error = 'compilation_preflight_changed'
        elif os.path.exists(r['destination']): error = 'destination_exists'
        else:
            os.mkdir(r['destination'])
            with open(os.path.join(r['destination'],'request.json'),'w') as f: json.dump(r,f)
            result = {'exported':True,'destination':r['destination'],'manifest':{}}
    print(json.dumps({'request_id':r['request_id'],'ok':not error,'error':error,'result':result}),flush=True)
)PY") < 0)
    return {};
  worker.close();
  worker.setPermissions(QFileDevice::ReadOwner | QFileDevice::WriteOwner |
                        QFileDevice::ExeOwner);
  return worker.fileName();
}

namespace {
class DeliverySmoke final : public QObject {
public:
  DeliverySmoke(QGuiApplication &app, QQmlApplicationEngine &engine,
                UiPreferences &preferences, DesktopBackend &backend,
                const QString &output)
      : QObject(&app), app_(app), engine_(engine), preferences_(preferences),
        backend_(backend),
        delivery_(*static_cast<DesktopDelivery *>(backend.delivery())),
        output_(output),
        window_(qobject_cast<QQuickWindow *>(engine.rootObjects().first())) {
    preferences_.setLanguage("en");
    preferences_.setTheme("dark");
    connect(&backend_, &DesktopBackend::requestFinished, this,
            [this](const QString &action, bool ok) { finished(action, ok); });
    connect(&backend_, &DesktopBackend::progressChanged, this, [this] {
      if (stage_ == 6 && delivery_.aligning() && ++cancellationProgress_ > 1)
        delivery_.cancelAlignment();
    });
    connect(&backend_, &DesktopBackend::playRangeFile, this,
            [this](const QUrl &, qint64 start, qint64 end) {
              rangeStart_ = start;
              rangeEnd_ = end;
              ++rangeRequests_;
            });
    backend_.createProject(QUrl::fromLocalFile(
        output_ + "/fixture-" +
        QUuid::createUuid().toString(QUuid::WithoutBraces) + ".vox"));
    QTimer::singleShot(28000, this,
                       [this] { fail("Delivery smoke timed out"); });
  }

private:
  void fail(const char *message) {
    window_->grabWindow().save(output_ + "/failure.png");
    qCritical() << message << "stage" << stage_ << delivery_.subtitleMessage()
                << delivery_.compilationMessage();
    app_.exit(32);
  }
  QQuickItem *item(const char *name) const {
    QList<QQuickItem *> pending{window_->contentItem()};
    while (!pending.isEmpty()) {
      auto *current = pending.takeLast();
      if (current->objectName() == QLatin1String(name) && current->isVisible())
        return current;
      pending.append(current->childItems());
    }
    return nullptr;
  }
  QObject *dialog(const char *name) const {
    return window_->findChild<QObject *>(name);
  }
  bool click(const char *name) {
    auto *control = item(name);
    return control && control->isEnabled() &&
           QMetaObject::invokeMethod(control, "clicked");
  }
  bool pointerClick(const char *name) {
    auto *control = item(name);
    if (!control || !control->isEnabled())
      return false;
    const auto position = control->mapToScene(
        QPointF(control->width() / 2, control->height() / 2));
    const auto global = window_->mapToGlobal(position.toPoint());
    QMouseEvent press(QEvent::MouseButtonPress, position, position, global,
                      Qt::LeftButton, Qt::LeftButton, Qt::NoModifier);
    QMouseEvent release(QEvent::MouseButtonRelease, position, position, global,
                        Qt::LeftButton, Qt::NoButton, Qt::NoModifier);
    QCoreApplication::sendEvent(window_, &press);
    QCoreApplication::sendEvent(window_, &release);
    return true;
  }
  void saveShortcut(bool focusEditor = true) {
    if (focusEditor) {
      const auto active = [this] {
        return window_->isActive() && QGuiApplication::focusWindow() == window_;
      };
      QEventLoop activation;
      QTimer timeout;
      timeout.setSingleShot(true);
      connect(&timeout, &QTimer::timeout, &activation, &QEventLoop::quit);
      connect(window_, &QWindow::activeChanged, &activation, [&] {
        if (active())
          activation.quit();
      });
      connect(&app_, &QGuiApplication::focusWindowChanged, &activation,
              [&](QWindow *) {
                if (active())
                  activation.quit();
              });
      window_->requestActivate();
      if (!active()) {
        timeout.start(2000);
        activation.exec();
      }
      if (!active()) {
        qCritical() << "Save key activation" << QGuiApplication::platformName()
                    << "active" << window_->isActive() << "focusWindow"
                    << QGuiApplication::focusWindow();
        fail("The native save-key smoke requires an active foreground window");
        return;
      }
      item("subtitleCueEditor")->forceActiveFocus(Qt::ShortcutFocusReason);
    }
    const auto key = QKeySequence(QKeySequence::Save)[0];
    QKeyEvent press(QEvent::KeyPress, int(key.key()), key.keyboardModifiers(),
                    QStringLiteral("s"));
    QKeyEvent release(QEvent::KeyRelease, int(key.key()),
                      key.keyboardModifiers(), QStringLiteral("s"));
    QCoreApplication::sendEvent(window_, &press);
    QCoreApplication::sendEvent(window_, &release);
  }
  void later(std::function<void()> next) {
    QTimer::singleShot(100, this, std::move(next));
  }
  bool editorFits() const {
    for (const auto *name :
         {"subtitleCueStart", "subtitleCueEnd", "subtitleCueEditor",
          "subtitleCueSplitTime", "splitSubtitleCueButton",
          "saveSubtitleDraftButton", "saveSubtitlesButton"}) {
      auto *control = item(name);
      if (!control || control->width() < 20 || control->height() < 20)
        return false;
      const auto top = control->mapToScene(QPointF{});
      if (top.x() < 0 || top.y() < 0 ||
          top.x() + control->width() > window_->width() ||
          top.y() + control->height() > window_->height())
        return false;
    }
    return item("subtitleCueEditor")->height() >= 64 &&
           item("subtitleCueSplitTime")->width() >= 100;
  }
  void refineCues() {
    if (!click("subtitleCuesTab")) {
      fail("Cue editing entry is missing");
      return;
    }
    savedTrack_ = delivery_.subtitleTrack();
    originalPath_ = backend_.path();
    window_->resize(1080, 740);
    item("subtitleCueStart")->setProperty("text", "00:00:00.120");
    item("subtitleCueEnd")->setProperty("text", "00:00:00.050");
    if (!delivery_.subtitleDraftDirty() ||
        delivery_.subtitleDraftIssue().isEmpty() ||
        delivery_.canSaveSubtitleDraft() || delivery_.canExportSubtitles()) {
      fail("Invalid draft cue times bypassed save or export guards");
      return;
    }
    item("subtitleCueEnd")->setProperty("text", "00:00:00.880");
    item("subtitleCueEditor")
        ->setProperty("text", "Manual cue words stay literal <Alice> & Bob.");
    item("subtitleCueEditor")->setProperty("cursorPosition", 10);
    item("subtitleCueSplitTime")->setProperty("text", "00:00:00.500");
    if (!click("splitSubtitleCueButton") ||
        delivery_.subtitleDraftCues().size() != 2 ||
        delivery_.subtitleDraftCues()[0].toMap()["end_time"] !=
            "00:00:00.500" ||
        delivery_.subtitleDraftCues()[1].toMap()["start_time"] !=
            "00:00:00.500" ||
        !click("mergeSubtitleCueButton") ||
        delivery_.subtitleDraftCues().size() != 1) {
      fail("Split or merge lost the explicit timing boundary");
      return;
    }
    manualText_ = delivery_.selectedSubtitleCue()["text"].toString();
    if (!manualText_.contains("<Alice> & Bob.") ||
        !manualText_.contains('\n') || !click("addSubtitleCueButton") ||
        delivery_.subtitleCueIndex() != 1 ||
        delivery_.subtitleDraftIssue().isEmpty()) {
      fail("Literal cue text or blank added-cue validation was lost");
      return;
    }
    item("subtitleCueEditor")->setProperty("text", "Added in the final gap.");
    if (!delivery_.canSaveSubtitleDraft() ||
        !click("removeSubtitleCueButton") ||
        delivery_.subtitleDraftCues().size() != 1 ||
        delivery_.selectedSubtitleCue()["text"] != manualText_ ||
        delivery_.subtitleTrack() != savedTrack_) {
      fail("Add/remove changed the saved subtitle track");
      return;
    }
    const auto before = rangeRequests_;
    if (!click("auditionSubtitleCueButton") || rangeRequests_ != before + 1 ||
        rangeStart_ != 120 || rangeEnd_ != 880) {
      fail("Selected-cue audition did not use the edited cue range");
      return;
    }
    backend_.stopRangePlayback();
    preferences_.setLanguage("en");
    preferences_.setTheme("light");
    later([this] {
      if (!editorFits()) {
        fail("Cue inspector clips at the minimum English window size");
        return;
      }
      window_->grabWindow().save(output_ + "/subtitle-refinement-light-en.png");
      preferences_.setLanguage("zh_CN");
      preferences_.setTheme("dark");
      later([this] {
        if (!editorFits() ||
            item("saveSubtitleDraftButton")->property("text") !=
                QString::fromUtf8("保存为候选") ||
            item("subtitleDraftState")->property("text") !=
                QString::fromUtf8("有未保存的修改") ||
            delivery_.selectedSubtitleCue()["text"] != manualText_) {
          fail("Live Chinese translation clipped or replaced the cue draft");
          return;
        }
        window_->grabWindow().save(output_ +
                                   "/subtitle-refinement-dark-zh.png");
        guardTransition(0);
      });
    });
  }
  void guardTransition(int scenario) {
    if (scenario == 8) {
      preferences_.setLanguage("en");
      preferences_.setTheme("dark");
      later([this] {
        saveShortcut();
        later([this] {
          if (delivery_.subtitleDraftDirty() && !delivery_.subtitleLoading()) {
            auto *shortcut = dialog("subtitleSaveShortcut");
            qCritical() << "Popup Save routing"
                        << QGuiApplication::platformName()
                        << QKeySequence(QKeySequence::Save).toString()
                        << "enabled"
                        << (shortcut && shortcut->property("enabled").toBool())
                        << "owner" << (shortcut ? shortcut->parent() : nullptr)
                        << "focus" << window_->activeFocusItem() << "active"
                        << window_->isActive() << "focusWindow"
                        << QGuiApplication::focusWindow();
            fail("The popup save key did not create a correction candidate");
          }
        });
      });
      return;
    }
    if (scenario == 0)
      click("closeSubtitleDialogButton");
    else if (scenario == 1)
      delivery_.previewSubtitleCandidate({});
    else if (scenario == 2)
      click("resetSubtitleDraftButton");
    else if (scenario == 3)
      backend_.selectAudio("other-audio");
    else if (scenario == 4 || scenario == 5)
      QMetaObject::invokeMethod(
          window_, "beginProjectAction",
          Q_ARG(QVariant, QVariant(scenario == 4 ? QStringLiteral("new")
                                                 : QStringLiteral("open"))));
    else if (scenario == 6)
      window_->close();
    else
      backend_.createProject(
          QUrl::fromLocalFile(output_ + "/cancelled-new.vox"));
    later([this, scenario] {
      if (!delivery_.subtitleDiscardPending() ||
          !dialog("discardSubtitleDraftDialog")->property("visible").toBool() ||
          backend_.selectedAudio() != "audio-main" ||
          backend_.path() != originalPath_ || !window_->isVisible()) {
        fail("An exit, selection or project change bypassed draft protection");
        return;
      }
      auto *shortcut = dialog("subtitleSaveShortcut");
      if (!shortcut || shortcut->property("enabled").toBool()) {
        fail("Save stayed active over the draft discard confirmation");
        return;
      }
      saveShortcut(false);
      if (delivery_.subtitleLoading() || !delivery_.subtitleDraftDirty() ||
          !click("keepSubtitleDraftButton")) {
        fail("A confirmation key saved a draft or bypassed Keep editing");
        return;
      }
      later([this, scenario] {
        if (!delivery_.subtitleDraftDirty() ||
            delivery_.subtitleDiscardPending() ||
            delivery_.selectedSubtitleCue()["text"] != manualText_ ||
            delivery_.subtitleTrack() != savedTrack_ ||
            backend_.path() != originalPath_) {
          fail("Keep editing failed to retain the exact correction draft");
          return;
        }
        guardTransition(scenario + 1);
      });
    });
  }
  bool completeNativeImport() {
    auto *file = dialog("subtitleImportFileDialog");
    if (!file || !file->property("visible").toBool())
      return false;
    auto *shortcut = dialog("subtitleSaveShortcut");
    if (!shortcut || shortcut->property("enabled").toBool())
      return false;
    file->setProperty("selectedFile", QUrl::fromLocalFile(importPath_));
    const auto invoked = QMetaObject::invokeMethod(file, "accepted");
    file->setProperty("visible", false);
    return invoked;
  }
  void finished(const QString &action, bool ok) {
    if (!ok && action != "probe") {
      fail("A delivery protocol action failed");
      return;
    }
    if (stage_ == 0 && action == "create") {
      stage_ = 1;
      later([this] {
        if (!click("takeSubtitlesButton"))
          fail("Echo subtitle review entry is missing");
      });
    } else if (stage_ == 1 && action == "subtitle_preview") {
      stage_ = 2;
      later([this] {
        if (!click("subtitleAlignmentTab")) {
          fail("Transcript realignment entry is missing");
          return;
        }
        if (!dialog("subtitleDialog") || delivery_.canExportSubtitles() ||
            item("subtitleTranscript")->property("text") !=
                "Pinned source words before the Echo cut.") {
          fail("Subtitle source was taken from the current draft or stale "
               "timings exported");
          return;
        }
        item("subtitleTranscript")->setProperty("text", edited_);
        delivery_.previewSubtitleCandidate({});
      });
    } else if (stage_ == 2 && action == "subtitle_preview") {
      stage_ = 3;
      preferences_.setLanguage("zh_CN");
      preferences_.setTheme("light");
      later([this] {
        if (delivery_.transcript() != edited_ ||
            item("subtitleTranscript")->property("text") != edited_ ||
            dialog("subtitleDialog")->property("title") !=
                QString::fromUtf8("字幕")) {
          fail("Refresh or live translation replaced the typed transcript");
          return;
        }
        window_->grabWindow().save(output_ + "/subtitles-light-zh.png");
        preferences_.setLanguage("en");
        preferences_.setTheme("dark");
        item("subtitleTranscriptConfirmed")->setProperty("checked", true);
        if (!click("alignSubtitlesButton"))
          fail("Confirmed realignment could not be started");
      });
    } else if (stage_ == 3 && action == "subtitle_preview") {
      stage_ = 4;
      later([this] {
        if (delivery_.candidateID().isEmpty() ||
            delivery_.canExportSubtitles() ||
            delivery_.subtitleTrack()["cues"]
                    .toList()
                    .first()
                    .toMap()["text"] != edited_) {
          fail(
              "Candidate timing review lost exact text or bypassed acceptance");
          return;
        }
        window_->grabWindow().save(output_ + "/subtitle-candidate-dark-en.png");
        if (!click("acceptSubtitleCandidateButton"))
          fail("Explicit timing acceptance is missing");
      });
    } else if (stage_ == 4 && action == "subtitle_preview") {
      stage_ = 5;
      if (!delivery_.canExportSubtitles() ||
          !delivery_.candidateID().isEmpty() ||
          backend_.document()["text"] != "The authored script stays exact.") {
        fail(
            "Accepted timings were unavailable or changed the authored script");
        return;
      }
      delivery_.exportSubtitles(QUrl::fromLocalFile(output_ + "/accepted.srt"),
                                "srt");
    } else if (stage_ == 5 && action == "export_subtitles") {
      QFile file(output_ + "/accepted.srt");
      if (!file.open(QIODevice::ReadOnly) ||
          !file.readAll().contains(edited_.toUtf8())) {
        fail("Subtitle export lost the accepted edited wording");
        return;
      }
      stage_ = 6;
      delivery_.setTranscript("Completed candidate after stop.");
      delivery_.alignSubtitles(true);
    } else if (stage_ == 6 && action == "subtitle_preview") {
      stage_ = 7;
      if (delivery_.aligning() || delivery_.candidateID().isEmpty() ||
          delivery_.canExportSubtitles() ||
          !delivery_.subtitleMessage().contains("completed candidate")) {
        fail("Stop lost its completed candidate or exported it before "
             "acceptance");
        return;
      }
      delivery_.discardSubtitleCandidate();
    } else if (stage_ == 7 && action == "subtitle_preview") {
      stage_ = 20;
      later([this] { refineCues(); });
    } else if (stage_ == 20 && action == "subtitle_preview") {
      stage_ = 21;
      if (delivery_.candidateID() != "manual-1" ||
          delivery_.subtitleDraftDirty() || delivery_.canExportSubtitles() ||
          delivery_.subtitleTrack()["source"] != "manual_revision" ||
          delivery_.subtitleTrack()["revision_id"] != "manual-1" ||
          !delivery_.subtitleTrack()["alignment_revision"].isNull() ||
          delivery_.selectedSubtitleCue()["text"] != manualText_ ||
          backend_.document()["text"] != "The authored script stays exact.") {
        fail(
            "Manual correction did not create a distinct unaccepted candidate");
        return;
      }
      later([this] {
        if (!pointerClick("acceptSubtitleCandidateButton"))
          fail("Manual subtitle candidate has no explicit acceptance action");
      });
    } else if (stage_ == 21 && action == "subtitle_preview") {
      stage_ = 22;
      manualRevision_ = delivery_.subtitleTrack()["revision_id"].toString();
      if (!delivery_.canExportSubtitles() ||
          delivery_.selectedSubtitleCue()["text"] != manualText_) {
        fail("Accepted manual corrections did not become the saved track");
        return;
      }
      importPath_ = output_ + "/literal-import.srt";
      QFile file(importPath_);
      if (!file.open(QIODevice::WriteOnly) ||
          file.write("1\n00:00:00,150 --> 00:00:00,850\n"
                     "Imported literal <Alice> & Bob.\n") < 0) {
        fail("Could not prepare the isolated subtitle import fixture");
        return;
      }
      file.close();
      later([this] {
        if (!click("importSubtitleButton")) {
          fail("Native subtitle import action is missing");
          return;
        }
        delivery_.previewSubtitleCandidate({});
      });
    } else if (stage_ == 22 && action == "subtitle_preview") {
      stage_ = 23;
      later([this] {
        if (!completeNativeImport() || delivery_.subtitleLoading() ||
            !delivery_.candidateID().isEmpty() ||
            delivery_.subtitleTrack()["revision_id"] != manualRevision_ ||
            !delivery_.subtitleMessage().contains("changed")) {
          fail("A stale native import callback changed the reviewed track");
          return;
        }
        later([this] {
          if (!click("importSubtitleButton")) {
            fail("A fresh native subtitle import could not be opened");
            return;
          }
          later([this] {
            if (!completeNativeImport())
              fail("The native file selection did not reach subtitle import");
          });
        });
      });
    } else if (stage_ == 23 && action == "subtitle_preview") {
      stage_ = 24;
      if (delivery_.candidateID() != "imported-1" ||
          delivery_.canExportSubtitles() || delivery_.subtitleDraftDirty() ||
          delivery_.subtitleTrack()["source"] != "imported_subtitles" ||
          delivery_.selectedSubtitleCue()["text"] !=
              "Imported literal <Alice> & Bob.") {
        fail("Native import lost literal wording or implicitly accepted cues");
        return;
      }
      later([this] {
        window_->grabWindow().save(output_ +
                                   "/subtitle-import-candidate-dark-en.png");
        if (!click("discardSubtitleCandidateButton"))
          fail("Imported subtitle candidate could not be discarded");
      });
    } else if (stage_ == 24 && action == "subtitle_preview") {
      stage_ = 8;
      if (!delivery_.canExportSubtitles() ||
          delivery_.subtitleTrack()["revision_id"] != manualRevision_ ||
          delivery_.selectedSubtitleCue()["text"] != manualText_) {
        fail("Discarding an import replaced the previously accepted subtitles");
        return;
      }
      QFile mutations(output_ + "/subtitle-mutations.jsonl");
      if (!mutations.open(QIODevice::ReadOnly)) {
        fail("Subtitle candidate requests were not recorded");
        return;
      }
      int edits = 0, imports = 0;
      for (const auto &line : mutations.readAll().split('\n')) {
        if (line.isEmpty())
          continue;
        const auto request = QJsonDocument::fromJson(line).object();
        edits += request["action"] == "subtitle_edit";
        imports += request["action"] == "subtitle_import";
        if (request["audio_id"] != "audio-main" ||
            request["path"] != originalPath_) {
          fail("A subtitle mutation escaped its project/audio identity");
          return;
        }
      }
      if (edits != 1 || imports != 1) {
        fail("A protected or stale transition submitted an extra mutation");
        return;
      }
      delivery_.previewSubtitleCandidate({});
      backend_.selectAudio("other-audio");
    } else if (stage_ == 8 && action == "subtitle_preview") {
      stage_ = 9;
      later([this] {
        if (delivery_.subtitleActive() ||
            dialog("subtitleDialog")->property("visible").toBool() ||
            backend_.selectedAudio() != "other-audio") {
          fail("A stale subtitle reply replaced a later audio identity");
          return;
        }
        if (!click("productionMode-chapters"))
          fail("Chapter workspace entry is missing");
        later([this] {
          if (!click("compileChaptersButton"))
            fail("Compilation entry is missing");
          later([this] {
            if (!click("compileChapterDown-0") ||
                !click("checkCompilationButton"))
              fail("Compilation order or preflight action is unavailable");
          });
        });
      });
    } else if (stage_ == 9 && action == "chapter_compilation_preflight") {
      stage_ = 10;
      if (delivery_.compilationPreflight()["ready"].toBool()) {
        fail("Mixed formats and missing captions bypassed preflight");
        return;
      }
      preferences_.setLanguage("zh_CN");
      preferences_.setTheme("light");
      later([this] {
        if (!dialog("compilationDialog") ||
            dialog("compilationDialog")->property("title") !=
                QString::fromUtf8("合编章节") ||
            dialog("compilationDialog")->property("width").toInt() < 650 ||
            item("compileChapterStatus-0")->property("text") !=
                QString::fromUtf8("已采用") ||
            !item("compilationIssues")
                 ->property("text")
                 .toString()
                 .contains(QString::fromUtf8("音频格式不同"))) {
          fail("Compilation issues or live Chinese messages are missing");
          return;
        }
        window_->grabWindow().save(output_ +
                                   "/compilation-issues-light-zh.png");
        auto *compile = dialog("compilationDialog");
        compile->setProperty("audioMode", 1);
        compile->setProperty("captionsMode", 1);
        compile->setProperty("gapMs", 250);
        QMetaObject::invokeMethod(compile, "configure");
        if (!delivery_.compilationPreflight().isEmpty()) {
          fail("Changed export options retained a reviewed preflight");
          return;
        }
        if (!click("checkCompilationButton"))
          fail("Explicit conversion could not be checked");
      });
    } else if (stage_ == 10 && action == "chapter_compilation_preflight") {
      stage_ = 11;
      later([this] {
        if (!delivery_.compilationPreflight()["ready"].toBool()) {
          fail("Explicit normalized audio-only compilation failed preflight");
          return;
        }
        auto *compile = dialog("compilationDialog");
        compile->setProperty("parentFolder", QUrl::fromLocalFile(output_));
        exportName_ =
            "compiled-" + QUuid::createUuid().toString(QUuid::WithoutBraces);
        item("compilationFolderName")->setProperty("text", exportName_);
        preferences_.setLanguage("en");
        preferences_.setTheme("dark");
        later([this] {
          if (item("compileChapterStatus-0")->property("text") != "Accepted" ||
              item("compilationAudioMode")->property("currentIndex").toInt() !=
                  1 ||
              item("compilationCaptionsMode")
                      ->property("currentIndex")
                      .toInt() != 1) {
            fail(
                "Compilation live translation lost status or explicit options");
            return;
          }
          window_->grabWindow().save(output_ +
                                     "/compilation-ready-dark-en.png");
          if (!click("exportCompilationButton"))
            fail("Reviewed compilation could not be exported");
        });
      });
    } else if (stage_ == 11 && action == "chapter_compile") {
      QFile file(output_ + "/" + exportName_ + "/request.json");
      if (!file.open(QIODevice::ReadOnly)) {
        fail("Compilation output was not written");
        return;
      }
      const auto request = QJsonDocument::fromJson(file.readAll()).object();
      if (request["fingerprint"] != "reviewed-fingerprint" ||
          request["chapter_ids"].toArray() != QJsonArray{"two", "one"} ||
          request["options"].toObject() !=
              QJsonObject{{"gap_ms", 250},
                          {"audio_mode", "normalize_mono24000"},
                          {"subtitles", "audio_only"}}) {
        fail("Compilation export changed the reviewed order, options or "
             "fingerprint");
        return;
      }
      stage_ = 30;
      delivery_.closeCompilation();
      backend_.selectAudio("audio-main");
      delivery_.openSubtitles("audio-main");
      QMetaObject::invokeMethod(dialog("subtitleDialog"), "open");
    } else if (stage_ == 30 && action == "subtitle_preview") {
      stage_ = 31;
      later([this] {
        item("subtitleCueEditor")
            ->setProperty("text", "Unsaved before native open.");
        nativeTarget_ = output_ + "/native-open.vox";
        backend_.openProject(QUrl::fromLocalFile(nativeTarget_));
        backend_.openProject(
            QUrl::fromLocalFile(output_ + "/ignored-open.vox"));
        later([this] {
          if (!delivery_.subtitleDiscardPending() ||
              backend_.path() != originalPath_ ||
              !click("confirmDiscardSubtitleDraftButton"))
            fail("Native open did not wait for explicit draft discard");
        });
      });
    } else if (stage_ == 31 && action == "open") {
      if (backend_.path() != nativeTarget_ || delivery_.subtitleActive() ||
          delivery_.subtitleDraftDirty()) {
        fail("Native open lost its captured target or kept a stale cue draft");
        return;
      }
      qInfo() << "Delivery packaged UI: pinned transcript, explicit candidate "
                 "acceptance/export, stopped alignment, manual cue text/times, "
                 "split/merge/add/remove, popup save key, selected-range "
                 "audition, minimum "
                 "English/Chinese geometry, protected close/candidate/audio/"
                 "project transitions, native import identity fences, stale "
                 "audio callbacks and reviewed ordered compilation passed";
      app_.quit();
    }
  }
  QGuiApplication &app_;
  QQmlApplicationEngine &engine_;
  UiPreferences &preferences_;
  DesktopBackend &backend_;
  DesktopDelivery &delivery_;
  QString output_, exportName_, originalPath_, manualText_, importPath_,
      manualRevision_, nativeTarget_;
  QVariantMap savedTrack_;
  QQuickWindow *window_;
  const QString edited_ = "Exact words after the Echo cut.";
  int stage_ = 0, cancellationProgress_ = 0, rangeRequests_ = 0;
  qint64 rangeStart_ = -1, rangeEnd_ = -1;
};
} // namespace
void startDeliverySmoke(QGuiApplication &app, QQmlApplicationEngine &engine,
                        UiPreferences &preferences, DesktopBackend &backend,
                        const QString &output) {
  new DeliverySmoke(app, engine, preferences, backend, output);
}
