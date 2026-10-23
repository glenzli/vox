#pragma once
#include "echo_launcher.hpp"
#include "worker_client.hpp"
#include <QJsonArray>
#include <QSettings>
#include <QTimer>
#include <QUrl>
#include <QVariantMap>

class DesktopDelivery;

// Owns the live project/draft identity and routes all durable operations
// through vox-core.
class DesktopBackend final : public QObject {
  Q_OBJECT
  Q_PROPERTY(QVariantMap project READ project NOTIFY stateChanged)
  Q_PROPERTY(QVariantMap document READ document NOTIFY documentChanged)
  Q_PROPERTY(QVariantMap analysis READ analysis NOTIFY stateChanged)
  Q_PROPERTY(QVariantMap segmentPlan READ segmentPlan NOTIFY stateChanged)
  Q_PROPERTY(QVariantMap collection READ collection NOTIFY stateChanged)
  Q_PROPERTY(QVariantList chapters READ chapters NOTIFY stateChanged)
  Q_PROPERTY(QString collectionPath READ collectionPath NOTIFY stateChanged)
  Q_PROPERTY(QString selectedChapter READ selectedChapter NOTIFY stateChanged)
  Q_PROPERTY(QString chapterError READ chapterError NOTIFY stateChanged)
  Q_PROPERTY(
      bool batchingChapters READ batchingChapters NOTIFY renderingChanged)
  Q_PROPERTY(
      QVariantMap chapterProgress READ chapterProgress NOTIFY progressChanged)
  Q_PROPERTY(QVariantList voices READ voices NOTIFY capabilitiesChanged)
  Q_PROPERTY(
      QVariantList builtinSounds READ builtinSounds NOTIFY capabilitiesChanged)
  Q_PROPERTY(QVariantList languages READ languages NOTIFY capabilitiesChanged)
  Q_PROPERTY(
      QVariantList recentProjects READ recentProjects NOTIFY stateChanged)
  Q_PROPERTY(QString path READ path NOTIFY stateChanged)
  Q_PROPERTY(
      QString importedDraftKind READ importedDraftKind NOTIFY documentChanged)
  Q_PROPERTY(bool dirty READ dirty NOTIFY documentChanged)
  Q_PROPERTY(bool busy READ busy NOTIFY busyChanged)
  Q_PROPERTY(bool pending READ pending NOTIFY pendingChanged)
  Q_PROPERTY(bool rendering READ rendering NOTIFY renderingChanged)
  Q_PROPERTY(bool designingVoice READ designingVoice NOTIFY renderingChanged)
  Q_PROPERTY(bool runtimeReady READ runtimeReady NOTIFY runtimeChanged)
  Q_PROPERTY(QString runtimeError READ runtimeError NOTIFY runtimeChanged)
  Q_PROPERTY(QString message READ message NOTIFY messageChanged)
  Q_PROPERTY(int completed READ completed NOTIFY progressChanged)
  Q_PROPERTY(int total READ total NOTIFY progressChanged)
  Q_PROPERTY(QString selectedAudio READ selectedAudio WRITE selectAudio NOTIFY
                 selectionChanged)
  Q_PROPERTY(QVariantList waveform READ waveform NOTIFY waveformChanged)
  Q_PROPERTY(bool hasEchoHandoff READ hasEchoHandoff NOTIFY stateChanged)
  Q_PROPERTY(QObject *delivery READ delivery CONSTANT)
public:
  explicit DesktopBackend(QObject *parent = nullptr);
  QVariantMap project() const { return m_project.toVariantMap(); }
  QVariantMap document() const { return m_draft.toVariantMap(); }
  QVariantMap analysis() const { return m_analysis.toVariantMap(); }
  QVariantMap segmentPlan() const { return m_segmentPlan.toVariantMap(); }
  QVariantMap collection() const { return m_collectionProject.toVariantMap(); }
  QVariantList chapters() const { return m_chapters.toVariantList(); }
  QString collectionPath() const { return m_collectionPath; }
  QString selectedChapter() const { return m_chapterID; }
  QString chapterError() const;
  bool batchingChapters() const { return m_rendering && m_batchingChapters; }
  QVariantMap chapterProgress() const {
    return m_chapterProgress.toVariantMap();
  }
  QVariantList voices() const;
  QVariantList builtinSounds() const;
  QVariantList languages() const;
  void retranslate() {
    emit capabilitiesChanged();
    emit stateChanged();
    emit messageChanged();
  }
  QVariantList recentProjects() const;
  QString path() const { return m_path; }
  QString importedDraftKind() const;
  bool dirty() const { return m_dirty; }
  bool busy() const { return m_rendering || m_projectChanging; }
  bool pending() const { return m_pendingRequests > 0; }
  bool rendering() const { return m_rendering; }
  bool designingVoice() const { return m_rendering && m_designingVoice; }
  bool runtimeReady() const { return m_runtimeReady; }
  QString runtimeError() const { return m_runtimeError; }
  QString message() const;
  int completed() const { return m_completed; }
  int total() const { return m_total; }
  QString selectedAudio() const { return m_selected; }
  QVariantList waveform() const { return m_waveform; }
  bool hasEchoHandoff() const;
  QObject *delivery() const;
  Q_INVOKABLE void createProject(const QUrl &url);
  Q_INVOKABLE void openProject(const QUrl &url);
  Q_INVOKABLE void importShapeTransfer(const QUrl &transfer,
                                       const QUrl &destination,
                                       const QString &expectedBlake3 = {});
  Q_INVOKABLE void restoreShapeDraft(const QString &kind);
  Q_INVOKABLE void editText(const QString &text);
  Q_INVOKABLE void setFormat(const QString &format);
  Q_INVOKABLE void setVoice(const QString &alias);
  Q_INVOKABLE void setLanguage(const QString &language);
  Q_INVOKABLE void setSpeed(int speed);
  Q_INVOKABLE void bindRole(const QString &role, const QString &alias);
  Q_INVOKABLE void importReference(const QString &name,
                                   const QString &transcript, const QUrl &file,
                                   bool consent, bool recorded);
  Q_INVOKABLE void bindReference(const QString &role, const QString &id);
  Q_INVOKABLE void auditionReference(const QString &id, const QString &text);
  Q_INVOKABLE void addBuiltin(const QString &key);
  Q_INVOKABLE void previewBuiltin(const QString &key);
  Q_INVOKABLE void importSound(const QUrl &file);
  Q_INVOKABLE void generateSound(const QString &name, const QString &prompt,
                                 int seconds, int seed, bool music);
  Q_INVOKABLE void generateSoundBatch(const QString &name,
                                      const QString &prompt, int seconds,
                                      int seed, const QString &category,
                                      int variations);
  Q_INVOKABLE void generateVoiceDesign(const QString &description,
                                       const QString &text,
                                       const QString &language, int variations);
  Q_INVOKABLE void acceptVoiceDesign(const QString &id, const QString &name);
  Q_INVOKABLE void discardVoiceDesign(const QString &id);
  Q_INVOKABLE void selectChapter(const QString &id);
  Q_INVOKABLE void showMainScript();
  Q_INVOKABLE void refreshChapters();
  Q_INVOKABLE void addChapter(const QString &title, const QString &text,
                              const QString &format);
  Q_INVOKABLE void importChapters(const QList<QUrl> &files);
  Q_INVOKABLE void renameChapter(const QString &id, const QString &title);
  Q_INVOKABLE void moveChapter(const QString &id, int offset);
  Q_INVOKABLE void archiveChapter(const QString &id);
  Q_INVOKABLE void publishChapterSettings();
  Q_INVOKABLE void renderChapterBatch(bool retryInterrupted = false);
  Q_INVOKABLE void exportChapters(const QUrl &destination);
  Q_INVOKABLE QString productionStatus(const QString &status) const;
  Q_INVOKABLE void acceptSound(const QString &id);
  Q_INVOKABLE void discardSound(const QString &id);
  Q_INVOKABLE void bindSound(const QString &label, const QString &id);
  Q_INVOKABLE void previewFile(const QUrl &file);
  Q_INVOKABLE void playMaterial(const QString &kind, const QString &id);
  Q_INVOKABLE void exportMaterial(const QString &kind, const QString &id,
                                  const QUrl &destination);
  Q_INVOKABLE void editMaterialInEcho(const QString &kind, const QString &id);
  Q_INVOKABLE QString soundLabel(const QString &key) const;
  Q_INVOKABLE void bindCue(const QString &label, const QString &action);
  Q_INVOKABLE void importCue(const QString &label, const QUrl &url);
  Q_INVOKABLE void save();
  Q_INVOKABLE void render();
  Q_INVOKABLE void renderSegment(const QString &key);
  Q_INVOKABLE void acceptSegment(const QString &id);
  Q_INVOKABLE void playSegment(const QString &id);
  Q_INVOKABLE void assembleSegments();
  Q_INVOKABLE void setPronunciationRules(const QVariantList &rules);
  Q_INVOKABLE void exportSubtitles(const QString &audioID, const QUrl &url,
                                   const QString &format);
  Q_INVOKABLE void cancel();
  Q_INVOKABLE void auditionVoice(const QString &alias);
  Q_INVOKABLE void selectAudio(const QString &id);
  Q_INVOKABLE void playSelected();
  Q_INVOKABLE void acceptSelected();
  Q_INVOKABLE void discardSelected();
  Q_INVOKABLE void exportSelected(const QUrl &url);
  Q_INVOKABLE void exportAudio(const QString &id, const QUrl &url);
  Q_INVOKABLE void importScript(const QUrl &url);
  Q_INVOKABLE void useTemplate(const QString &kind);
  Q_INVOKABLE void restoreSource(const QString &id);
  Q_INVOKABLE void probeRuntime();
  Q_INVOKABLE void installCredential(const QUrl &url);
  Q_INVOKABLE void editInEcho();
  Q_INVOKABLE void returnFromEcho(const QUrl &url, bool allowPCM24);
  Q_INVOKABLE QString scriptRules() const;
  Q_INVOKABLE QString voiceLabel(const QString &key) const;
  Q_INVOKABLE QString languageLabel(const QString &key) const;
  Q_INVOKABLE QString errorText(const QString &code) const;
  Q_INVOKABLE QString echoBinary() const;
  Q_INVOKABLE void setEchoBinary(const QUrl &url);
signals:
  void stateChanged();
  void documentChanged();
  void capabilitiesChanged();
  void renderingChanged();
  void busyChanged();
  void pendingChanged();
  void runtimeChanged();
  void messageChanged();
  void progressChanged();
  void selectionChanged();
  void playFile(const QUrl &file);
  void requestFinished(const QString &action, bool ok);
  void waveformChanged();

private:
  friend class DesktopDelivery;
  DesktopDelivery *m_delivery = nullptr;
  WorkerClient m_worker;
  WorkerClient m_probeWorker;
  EchoLauncher m_echoLauncher;
  QSettings m_settings;
  QJsonObject m_project, m_draft, m_analysis, m_segmentPlan;
  QJsonObject m_collectionProject;
  QJsonObject m_chapterProgress;
  QJsonArray m_chapters;
  QString m_collectionPath, m_chapterID, m_chaptersError;
  quint64 m_collectionEpoch = 0;
  QJsonArray m_voices;
  QVariantList m_waveform;
  QUrl m_preparedFile;
  QString m_preparedID;
  QString m_path, m_selected, m_messageCode, m_runtimeError;
  bool m_dirty = false, m_rendering = false, m_runtimeReady = false,
       m_savePending = false;
  bool m_probing = false;
  bool m_designingVoice = false;
  bool m_batchingChapters = false;
  bool m_echoOpening = false;
  bool m_projectChanging = false, m_cancelRequested = false;
  quint64 m_editSequence = 0, m_projectEpoch = 0, m_selectionSequence = 0,
          m_playbackSequence = 0;
  int m_completed = 0, m_total = 0;
  int m_pendingRequests = 0;
  struct SaveContinuation {
    quint64 epoch;
    std::function<void()> continuation, failed;
  };
  QList<SaveContinuation> m_saveContinuations;
  QTimer m_saveTimer;
  QTimer m_probeTimer;
  void materialCommand(const QString &action, const QJsonObject &fields,
                       bool generate = false);
  void productionCommand(const QString &action, const QJsonObject &fields,
                         bool generate = false);
  void collectionCommand(const QString &action, const QJsonObject &fields,
                         WorkerClient::Callback done = {});
  void applyCollection(const QJsonObject &result);
  void acceptChapterSelected();
  void beginChapterEcho(const QString &directory);
  void reviewChapterEcho(const QUrl &url, bool allowPCM24);
  void mutate(const std::function<void(QJsonObject &)> &mutation);
  void saveThen(std::function<void()> continuation,
                std::function<void()> failed = {});
  void setRendering(bool rendering);
  void setProjectChanging(bool changing);
  void playPreview(const QUrl &file, const QVariantList &peaks = {});
  void apply(const QJsonObject &result, quint64 capturedEdit,
             bool reset = false);
  void command(const QString &action, const QJsonObject &fields,
               WorkerClient::Callback done = {}, bool includeProject = true);
  void messageCode(const QString &code);
  void finishRender(const QJsonObject &response, bool voiceSample,
                    quint64 selection = 0);
};
