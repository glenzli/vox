#include "desktop_backend.hpp"
#include "desktop_delivery.hpp"
#include "ui_preferences.hpp"
#include "workflow_smoke.hpp"
#include <QDir>
#include <QFile>
#include <QGuiApplication>
#include <QJsonDocument>
#include <QQmlApplicationEngine>
#include <QQuickItem>
#include <QQuickWindow>
#include <QTimer>
#include <QUuid>

QString prepareDeliverySmoke(const QString &output) {
  QDir().mkpath(output);
  QFile worker(output + "/delivery-worker.py");
  if (!worker.open(QIODevice::WriteOnly) || worker.write(R"PY(#!/usr/bin/python3
import json, os, select, sys
path, project, candidate, accepted = '', {}, None, None
chapters = [{'id':'one','title':'First chapter','status':'accepted','archived':False},
            {'id':'two','title':'Second chapter','status':'accepted','archived':False}]
def snapshot():
    return {'path':path,'project':project,'analysis':{'ready':True},'segment_plan':{},'chapters':chapters}
def track(value):
    return {'audio_revision':'audio-main','audio_digest':'pinned-audio','source_revision':'pinned-source',
            'source':'forced_alignment','alignment_revision':value['id'],'language':value['language'],
            'cues':[{'start_ms':100,'end_ms':900,'text':value['transcript']}]}
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
    elif action in ['open','chapter_list']: result = snapshot()
    elif action == 'playback': result = {'file':'/nonexistent/delivery-audio.wav','peaks':[]}
    elif action == 'subtitle_preview':
        result = snapshot()
        shown = candidate if r.get('candidate_id') and candidate else accepted
        result.update({'audio_id':r['audio_id'],'track':track(shown) if shown else None,
                       'track_error':None if shown else 'subtitles_unavailable',
                       'transcript':'Pinned source words before the Echo cut.','language':'English',
                       'accepted_audio':True,'candidates':[candidate] if candidate else [],'runs':[]})
    elif action == 'align_subtitles':
        print(json.dumps({'request_id':r['request_id'],'event':'progress','completed':0,'total':1}),flush=True)
        cancelled = False
        if select.select([sys.stdin],[],[],0.25)[0]:
            stop = json.loads(sys.stdin.readline())
            cancelled = stop['action'] == 'cancel'
            print(json.dumps({'request_id':stop['request_id'],'ok':True,'result':{}}),flush=True)
        candidate = {'id':'candidate-2' if accepted else 'candidate-1',
                     'audio_revision':'audio-main','transcript':r['spec']['transcript'],'language':r['spec']['language']}
        project['version'] += 1
        result = snapshot()
        result['subtitle_outcome'] = {'state':'cancelled' if cancelled else 'completed','candidate_id':candidate['id'],'run_id':'run'}
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
    backend_.createProject(QUrl::fromLocalFile(
        output_ + "/fixture-" +
        QUuid::createUuid().toString(QUuid::WithoutBraces) + ".vox"));
    QTimer::singleShot(24000, this,
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
  void later(std::function<void()> next) {
    QTimer::singleShot(100, this, std::move(next));
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
      stage_ = 8;
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
      qInfo() << "Delivery packaged UI: pinned Echo transcript, "
                 "refresh/translation preservation, candidate acceptance, "
                 "export, stopped candidate retention, stale audio fences, "
                 "ordered compilation, explicit conversion/audio-only options "
                 "and reviewed fingerprint passed";
      app_.quit();
    }
  }
  QGuiApplication &app_;
  QQmlApplicationEngine &engine_;
  UiPreferences &preferences_;
  DesktopBackend &backend_;
  DesktopDelivery &delivery_;
  QString output_, exportName_;
  QQuickWindow *window_;
  const QString edited_ = "Exact words after the Echo cut.";
  int stage_ = 0, cancellationProgress_ = 0;
};
} // namespace
void startDeliverySmoke(QGuiApplication &app, QQmlApplicationEngine &engine,
                        UiPreferences &preferences, DesktopBackend &backend,
                        const QString &output) {
  new DeliverySmoke(app, engine, preferences, backend, output);
}
