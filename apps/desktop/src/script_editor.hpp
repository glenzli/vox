#pragma once

#include <QFont>
#include <QPointer>
#include <QQuickPaintedItem>
#include <QQuickTextDocument>
#include <QSyntaxHighlighter>
#include <QVariantMap>
#include <QtQml/qqmlregistration.h>

// Undo and redo belong to one project's editor session, even when another
// project happens to load identical source text.
class ScriptDocumentState : public QObject {
  Q_OBJECT
  QML_ELEMENT
  Q_PROPERTY(QQuickTextDocument *sourceDocument READ sourceDocument WRITE
                 setSourceDocument NOTIFY sourceDocumentChanged)
  Q_PROPERTY(QString sourceIdentity READ sourceIdentity WRITE setSourceIdentity
                 NOTIFY sourceIdentityChanged)
public:
  explicit ScriptDocumentState(QObject *parent = nullptr) : QObject(parent) {}
  QQuickTextDocument *sourceDocument() const { return m_quickDocument; }
  void setSourceDocument(QQuickTextDocument *document);
  QString sourceIdentity() const { return m_sourceIdentity; }
  void setSourceIdentity(const QString &identity);

signals:
  void sourceDocumentChanged();
  void sourceIdentityChanged();

private:
  void resetHistory();
  QPointer<QQuickTextDocument> m_quickDocument;
  QString m_sourceIdentity;
};

// Formatting is a view of the plain-text document. Rust remains the grammar
// owner.
class ScriptHighlighter : public QSyntaxHighlighter {
  Q_OBJECT
  QML_ELEMENT
  Q_PROPERTY(QQuickTextDocument *textDocument READ textDocument WRITE
                 setTextDocument NOTIFY textDocumentChanged)
  Q_PROPERTY(bool productionScript READ productionScript WRITE
                 setProductionScript NOTIFY productionScriptChanged)
  Q_PROPERTY(
      QVariantMap colors READ colors WRITE setColors NOTIFY colorsChanged)
public:
  explicit ScriptHighlighter(QObject *parent = nullptr);
  QQuickTextDocument *textDocument() const { return m_quickDocument; }
  void setTextDocument(QQuickTextDocument *document);
  bool productionScript() const { return m_productionScript; }
  void setProductionScript(bool enabled);
  QVariantMap colors() const { return m_colors; }
  void setColors(const QVariantMap &colors);

signals:
  void textDocumentChanged();
  void productionScriptChanged();
  void colorsChanged();

protected:
  void highlightBlock(const QString &text) override;

private:
  QPointer<QQuickTextDocument> m_quickDocument;
  QVariantMap m_colors;
  bool m_productionScript = false;
};

class LineNumberGutter : public QQuickPaintedItem {
  Q_OBJECT
  QML_ELEMENT
  // Decoration must not expose TextEdit's textDocument/cursorPosition property
  // pair: Qt accessibility would treat it as an editable text control.
  Q_PROPERTY(QQuickTextDocument *sourceDocument READ sourceDocument WRITE
                 setSourceDocument NOTIFY sourceDocumentChanged)
  Q_PROPERTY(QFont font READ font WRITE setFont NOTIFY fontChanged)
  Q_PROPERTY(
      QVariantMap colors READ colors WRITE setColors NOTIFY colorsChanged)
  Q_PROPERTY(qreal scrollY READ scrollY WRITE setScrollY NOTIFY scrollYChanged)
  Q_PROPERTY(qreal textTopPadding READ textTopPadding WRITE setTextTopPadding
                 NOTIFY textTopPaddingChanged)
  Q_PROPERTY(int activePosition READ cursorPosition WRITE setCursorPosition
                 NOTIFY cursorPositionChanged)
  Q_PROPERTY(int lineCount READ lineCount NOTIFY lineCountChanged)
public:
  explicit LineNumberGutter(QQuickItem *parent = nullptr);
  QQuickTextDocument *sourceDocument() const { return m_quickDocument; }
  void setSourceDocument(QQuickTextDocument *document);
  QFont font() const { return m_font; }
  void setFont(const QFont &font);
  QVariantMap colors() const { return m_colors; }
  void setColors(const QVariantMap &colors);
  qreal scrollY() const { return m_scrollY; }
  void setScrollY(qreal value);
  qreal textTopPadding() const { return m_textTopPadding; }
  void setTextTopPadding(qreal value);
  int cursorPosition() const { return m_cursorPosition; }
  void setCursorPosition(int position);
  int lineCount() const;
  void paint(QPainter *painter) override;

signals:
  void sourceDocumentChanged();
  void fontChanged();
  void colorsChanged();
  void scrollYChanged();
  void textTopPaddingChanged();
  void cursorPositionChanged();
  void lineCountChanged();

private:
  void bindDocument();
  void refreshWidth();
  void scheduleLayout();
  void updateLines();
  void geometryChange(const QRectF &newGeometry,
                      const QRectF &oldGeometry) override;
  QFont numberFont() const;
  struct Line {
    int number;
    qreal top, height, baseline;
  };
  QVector<Line> m_lines;
  int m_activeLine = 1;
  bool m_layoutPending = false;
  QPointer<QQuickTextDocument> m_quickDocument;
  QPointer<QTextDocument> m_document;
  QFont m_font;
  QVariantMap m_colors;
  qreal m_scrollY = 0;
  qreal m_textTopPadding = 0;
  int m_cursorPosition = 0;
};
