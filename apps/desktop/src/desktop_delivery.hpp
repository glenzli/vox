#pragma once
#include <QJsonArray>
#include <QJsonObject>
#include <QObject>
#include <QUrl>
#include <QVariantList>
#include <QVariantMap>

class DesktopBackend;

// Owns subtitle review and compilation export lifecycles. Durable authority
// remains with the shared worker and vox-core.
class DesktopDelivery final : public QObject {
  Q_OBJECT
  Q_PROPERTY(bool subtitleActive READ subtitleActive NOTIFY subtitleChanged)
  Q_PROPERTY(bool subtitleLoading READ subtitleLoading NOTIFY subtitleChanged)
  Q_PROPERTY(bool acceptedAudio READ acceptedAudio NOTIFY subtitleChanged)
  Q_PROPERTY(bool aligning READ aligning NOTIFY subtitleChanged)
  Q_PROPERTY(QString audioID READ audioID NOTIFY subtitleChanged)
  Q_PROPERTY(QString candidateID READ candidateID NOTIFY subtitleChanged)
  Q_PROPERTY(
      QVariantMap subtitleTrack READ subtitleTrack NOTIFY subtitleChanged)
  Q_PROPERTY(QVariantList subtitleCandidates READ subtitleCandidates NOTIFY
                 subtitleChanged)
  Q_PROPERTY(QVariantList subtitleDraftCues READ subtitleDraftCues NOTIFY
                 subtitleChanged)
  Q_PROPERTY(QVariantMap selectedSubtitleCue READ selectedSubtitleCue NOTIFY
                 subtitleChanged)
  Q_PROPERTY(int subtitleCueIndex READ subtitleCueIndex NOTIFY subtitleChanged)
  Q_PROPERTY(
      bool subtitleDraftDirty READ subtitleDraftDirty NOTIFY subtitleChanged)
  Q_PROPERTY(
      QString subtitleDraftIssue READ subtitleDraftIssue NOTIFY subtitleChanged)
  Q_PROPERTY(bool canSaveSubtitleDraft READ canSaveSubtitleDraft NOTIFY
                 subtitleChanged)
  Q_PROPERTY(
      qint64 subtitleDurationMs READ subtitleDurationMs NOTIFY subtitleChanged)
  Q_PROPERTY(QString subtitleReviewToken READ subtitleReviewToken NOTIFY
                 subtitleChanged)
  Q_PROPERTY(bool subtitleDiscardPending READ subtitleDiscardPending NOTIFY
                 subtitleChanged)
  Q_PROPERTY(
      QString transcript READ transcript WRITE setTranscript NOTIFY formChanged)
  Q_PROPERTY(
      QString language READ language WRITE setLanguage NOTIFY formChanged)
  Q_PROPERTY(
      QString subtitleMessage READ subtitleMessage NOTIFY subtitleChanged)
  Q_PROPERTY(
      bool canExportSubtitles READ canExportSubtitles NOTIFY subtitleChanged)
  Q_PROPERTY(
      bool compilationActive READ compilationActive NOTIFY compilationChanged)
  Q_PROPERTY(
      bool compilationLoading READ compilationLoading NOTIFY compilationChanged)
  Q_PROPERTY(bool compiling READ compiling NOTIFY compilationChanged)
  Q_PROPERTY(QVariantMap compilationPreflight READ compilationPreflight NOTIFY
                 compilationChanged)
  Q_PROPERTY(QString compilationMessage READ compilationMessage NOTIFY
                 compilationChanged)
  Q_PROPERTY(QString compilationDestination READ compilationDestination NOTIFY
                 compilationChanged)
  Q_PROPERTY(QVariantMap chapterStatusLabels READ chapterStatusLabels NOTIFY
                 compilationChanged)
public:
  explicit DesktopDelivery(DesktopBackend &backend);
  bool subtitleActive() const { return !m_audioID.isEmpty(); }
  bool subtitleLoading() const { return m_subtitleLoading; }
  bool acceptedAudio() const { return m_acceptedAudio; }
  bool aligning() const { return m_aligning; }
  QString audioID() const { return m_audioID; }
  QString candidateID() const { return m_candidateID; }
  QVariantMap subtitleTrack() const { return m_track.toVariantMap(); }
  QVariantList subtitleCandidates() const {
    return m_candidates.toVariantList();
  }
  QVariantList subtitleDraftCues() const { return m_draftCues.toVariantList(); }
  QVariantMap selectedSubtitleCue() const;
  int subtitleCueIndex() const { return m_cueIndex; }
  bool subtitleDraftDirty() const { return m_draftCues != m_savedCues; }
  QString subtitleDraftIssue() const;
  bool canSaveSubtitleDraft() const;
  qint64 subtitleDurationMs() const { return m_durationMs; }
  QString subtitleReviewToken() const {
    return QString::number(m_subtitleSequence);
  }
  bool subtitleDiscardPending() const {
    return !m_pendingSubtitleAction.isEmpty();
  }
  QString transcript() const { return m_transcript; }
  QString language() const { return m_language; }
  QString subtitleMessage() const;
  bool canExportSubtitles() const;
  bool compilationActive() const { return !m_compilationPath.isEmpty(); }
  bool compilationLoading() const { return m_compilationLoading; }
  bool compiling() const { return m_compiling; }
  QVariantMap compilationPreflight() const {
    return m_preflight.toVariantMap();
  }
  QString compilationMessage() const;
  QString compilationDestination() const { return m_compilationDestination; }
  QVariantMap chapterStatusLabels() const;
  void setTranscript(const QString &value);
  void setLanguage(const QString &value);
  void retranslate();
  Q_INVOKABLE void openSubtitles(const QString &audioID);
  Q_INVOKABLE void closeSubtitles();
  Q_INVOKABLE void requestCloseSubtitles();
  Q_INVOKABLE void resolveSubtitleDiscard(bool discard);
  Q_INVOKABLE void previewSubtitleCandidate(const QString &id);
  Q_INVOKABLE void selectSubtitleCue(int index);
  Q_INVOKABLE void updateSubtitleCue(const QString &startTime,
                                     const QString &endTime,
                                     const QString &text);
  Q_INVOKABLE void splitSubtitleCue(int cursorPosition,
                                    const QString &splitTime);
  Q_INVOKABLE void mergeNextSubtitleCue();
  Q_INVOKABLE void addSubtitleCue();
  Q_INVOKABLE void removeSubtitleCue();
  Q_INVOKABLE void resetSubtitleDraft();
  Q_INVOKABLE void saveSubtitleDraft();
  Q_INVOKABLE void requestSubtitleImport();
  Q_INVOKABLE void importSubtitles(const QUrl &file, const QString &format,
                                   const QString &reviewToken);
  Q_INVOKABLE void auditionSubtitleCue();
  Q_INVOKABLE void alignSubtitles(bool exactTranscriptConfirmed);
  Q_INVOKABLE void acceptSubtitleCandidate();
  Q_INVOKABLE void discardSubtitleCandidate();
  Q_INVOKABLE void cancelAlignment();
  Q_INVOKABLE void exportSubtitles(const QUrl &destination,
                                   const QString &format);
  Q_INVOKABLE void openCompilation();
  Q_INVOKABLE void closeCompilation();
  Q_INVOKABLE void configureCompilation(const QVariantList &chapterIDs,
                                        int gapMs, const QString &audioMode,
                                        const QString &subtitles);
  Q_INVOKABLE void checkCompilation();
  Q_INVOKABLE void exportCompilation(const QUrl &parent,
                                     const QString &folderName);
  Q_INVOKABLE QString issueText(const QString &code) const;
  Q_INVOKABLE QString subtitleSourceLabel(const QString &source) const;
signals:
  void subtitleChanged();
  void subtitleDiscardRequested();
  void subtitleImportRequested();
  void subtitleCloseResolved(bool closed);
  void formChanged();
  void compilationChanged();

private:
  DesktopBackend &m_backend;
  QString m_subtitlePath, m_subtitleProject, m_audioID, m_candidateID;
  QString m_transcript, m_language, m_subtitleCode;
  QJsonObject m_track;
  QJsonArray m_candidates, m_draftCues, m_savedCues;
  int m_cueIndex = -1;
  qint64 m_durationMs = 0;
  QString m_pendingSubtitleAction, m_pendingSubtitleValue;
  quint64 m_subtitleEpoch = 0, m_subtitleSequence = 0;
  bool m_subtitleLoading = false, m_acceptedAudio = false, m_aligning = false;
  QString m_compilationPath, m_compilationProject, m_compilationCode;
  QString m_compilationDestination;
  quint64 m_compilationEpoch = 0, m_compilationSequence = 0;
  QJsonObject m_compilationFields, m_preflight;
  QJsonArray m_reviewedChapters;
  bool m_compilationLoading = false, m_compiling = false;
  bool subtitleIdentity() const;
  bool compilationIdentity() const;
  void syncIdentity();
  void loadSubtitles(const QString &candidateID, bool initialize = false);
  bool subtitleEditable() const;
  bool confirmSubtitleDiscard(const QString &action, const QString &value = {});
  void resetSubtitleCues();
  void createSubtitleCandidate(const QString &action, QJsonObject fields);
  void changeSubtitleCandidate(bool accept);
  void invalidateCompilation();
};
