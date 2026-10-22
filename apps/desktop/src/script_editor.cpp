#include "script_editor.hpp"

#include <QAbstractTextDocumentLayout>
#include <QFontMetricsF>
#include <QPainter>
#include <QRegularExpression>
#include <QTextBlock>
#include <QTextLayout>
#include <QTimer>
#include <algorithm>
#include <cmath>

namespace {
QColor color(const QVariantMap &colors, const char *key) {
  return colors.value(QLatin1String(key)).value<QColor>();
}
QTextCharFormat textFormat(const QColor &foreground, bool bold = false,
                           bool italic = false) {
  QTextCharFormat format;
  format.setForeground(foreground);
  format.setFontWeight(bold ? QFont::DemiBold : QFont::Normal);
  format.setFontItalic(italic);
  return format;
}
bool identityValue(const QString &key) {
  return QStringList{"role",       "定义角色", "speaker", "角色", "cue",
                     "定义提示音", "audio",    "提示音",  "音效"}
      .contains(key);
}
} // namespace

void ScriptDocumentState::setSourceDocument(QQuickTextDocument *document) {
  if (m_quickDocument == document)
    return;
  if (m_quickDocument)
    disconnect(m_quickDocument, nullptr, this, nullptr);
  m_quickDocument = document;
  if (document)
    connect(document, &QQuickTextDocument::textDocumentChanged, this,
            &ScriptDocumentState::resetHistory);
  resetHistory();
  emit sourceDocumentChanged();
}

void ScriptDocumentState::setSourceIdentity(const QString &identity) {
  if (m_sourceIdentity == identity)
    return;
  m_sourceIdentity = identity;
  resetHistory();
  emit sourceIdentityChanged();
}

void ScriptDocumentState::resetHistory() {
  if (m_quickDocument && m_quickDocument->textDocument())
    m_quickDocument->textDocument()->clearUndoRedoStacks();
}

ScriptHighlighter::ScriptHighlighter(QObject *parent)
    : QSyntaxHighlighter(parent) {}

void ScriptHighlighter::setTextDocument(QQuickTextDocument *document) {
  if (m_quickDocument == document)
    return;
  if (m_quickDocument)
    disconnect(m_quickDocument, nullptr, this, nullptr);
  m_quickDocument = document;
  if (document)
    connect(document, &QQuickTextDocument::textDocumentChanged, this, [this] {
      setDocument(m_quickDocument ? m_quickDocument->textDocument() : nullptr);
    });
  setDocument(document ? document->textDocument() : nullptr);
  emit textDocumentChanged();
}

void ScriptHighlighter::setProductionScript(bool enabled) {
  if (m_productionScript == enabled)
    return;
  m_productionScript = enabled;
  rehighlight();
  emit productionScriptChanged();
}

void ScriptHighlighter::setColors(const QVariantMap &colors) {
  if (m_colors == colors)
    return;
  m_colors = colors;
  rehighlight();
  emit colorsChanged();
}

void ScriptHighlighter::highlightBlock(const QString &text) {
  if (!m_productionScript || m_colors.isEmpty())
    return;
  const auto trimmed = text.trimmed();
  const auto offset = text.indexOf(trimmed);
  const auto command = textFormat(color(m_colors, "command"), true);
  const auto identity = textFormat(color(m_colors, "identity"), true);
  const auto marker = textFormat(color(m_colors, "secondary"));
  const auto note = textFormat(color(m_colors, "secondary"), false, true);
  static const QRegularExpression heading("^#+\\s");
  if (heading.match(trimmed).hasMatch()) {
    setFormat(offset, trimmed.size(),
              textFormat(color(m_colors, "text"), true));
    return;
  }
  // This is a lexical view, including an unfinished directive while typing.
  // It never claims a command is valid or replaces the shared parser's errors.
  if (!trimmed.startsWith('['))
    return;
  const auto end =
      trimmed.endsWith(']') ? offset + trimmed.size() - 1 : text.size();
  setFormat(offset, 1, marker);
  if (trimmed.endsWith(']'))
    setFormat(end, 1, marker);
  const auto directive = text.mid(offset + 1, end - offset - 1);
  if (directive == "end-repeat" || directive == QStringLiteral("结束重复")) {
    setFormat(offset + 1, directive.size(), command);
    return;
  }
  int start = offset + 1;
  bool first = true;
  static const QRegularExpression number("^[0-9]+(?:\\.[0-9]+)?(?:s|秒|次)?$");
  for (int i = start; i <= end; ++i) {
    if (i != end && text.at(i) != ';' && text.at(i) != QChar(0xff1b))
      continue;
    const auto part = text.mid(start, i - start);
    int colon = part.indexOf(':');
    const int fullColon = part.indexOf(QChar(0xff1a));
    if (colon < 0 || (fullColon >= 0 && fullColon < colon))
      colon = fullColon;
    const auto key = (colon < 0 ? part : part.left(colon)).trimmed();
    if (first && (key == "note" || key == QStringLiteral("备注"))) {
      setFormat(offset, trimmed.size(), note);
      return;
    }
    if (!key.isEmpty())
      setFormat(start + part.indexOf(key), key.size(), command);
    if (colon >= 0) {
      setFormat(start + colon, 1, marker);
      const auto value = part.mid(colon + 1).trimmed();
      const auto valueStart =
          start + colon + 1 + part.mid(colon + 1).indexOf(value);
      if (!value.isEmpty() && identityValue(key))
        setFormat(valueStart, value.size(), identity);
      else if (number.match(value).hasMatch())
        setFormat(valueStart, value.size(),
                  textFormat(color(m_colors, "text"), true));
    }
    if (i < end)
      setFormat(i, 1, marker);
    start = i + 1;
    first = false;
  }
}

LineNumberGutter::LineNumberGutter(QQuickItem *parent)
    : QQuickPaintedItem(parent) {
  setAntialiasing(true);
  refreshWidth();
}

void LineNumberGutter::setSourceDocument(QQuickTextDocument *document) {
  if (m_quickDocument == document)
    return;
  if (m_quickDocument)
    disconnect(m_quickDocument, nullptr, this, nullptr);
  m_quickDocument = document;
  if (document)
    connect(document, &QQuickTextDocument::textDocumentChanged, this,
            &LineNumberGutter::bindDocument);
  bindDocument();
  emit sourceDocumentChanged();
}

void LineNumberGutter::bindDocument() {
  if (m_document) {
    disconnect(m_document, nullptr, this, nullptr);
    disconnect(m_document->documentLayout(), nullptr, this, nullptr);
  }
  m_document = m_quickDocument ? m_quickDocument->textDocument() : nullptr;
  if (m_document) {
    connect(m_document, &QTextDocument::blockCountChanged, this, [this] {
      refreshWidth();
      emit lineCountChanged();
    });
    connect(m_document, &QTextDocument::contentsChanged, this,
            [this] { scheduleLayout(); });
    connect(m_document->documentLayout(),
            &QAbstractTextDocumentLayout::documentSizeChanged, this,
            [this] { scheduleLayout(); });
  }
  refreshWidth();
  scheduleLayout();
  emit lineCountChanged();
}

QFont LineNumberGutter::numberFont() const {
  auto font = m_font;
  if (font.pixelSize() > 0)
    font.setPixelSize(std::max(10, font.pixelSize() - 2));
  return font;
}

void LineNumberGutter::refreshWidth() {
  const int digits = std::max(2, int(QString::number(lineCount()).size()));
  setImplicitWidth(
      std::ceil(QFontMetricsF(numberFont())
                    .horizontalAdvance(QString(digits, QLatin1Char('9')))) +
      24);
}

int LineNumberGutter::lineCount() const {
  return m_document ? m_document->blockCount() : 1;
}

void LineNumberGutter::setFont(const QFont &font) {
  if (m_font == font)
    return;
  m_font = font;
  refreshWidth();
  scheduleLayout();
  emit fontChanged();
}

void LineNumberGutter::setColors(const QVariantMap &colors) {
  if (m_colors == colors)
    return;
  m_colors = colors;
  update();
  emit colorsChanged();
}

void LineNumberGutter::setScrollY(qreal value) {
  if (qFuzzyCompare(m_scrollY, value))
    return;
  m_scrollY = value;
  scheduleLayout();
  emit scrollYChanged();
}

void LineNumberGutter::setTextTopPadding(qreal value) {
  if (qFuzzyCompare(m_textTopPadding, value))
    return;
  m_textTopPadding = value;
  scheduleLayout();
  emit textTopPaddingChanged();
}

void LineNumberGutter::setCursorPosition(int position) {
  if (m_cursorPosition == position)
    return;
  m_cursorPosition = position;
  scheduleLayout();
  emit cursorPositionChanged();
}

void LineNumberGutter::geometryChange(const QRectF &geometry,
                                      const QRectF &oldGeometry) {
  QQuickPaintedItem::geometryChange(geometry, oldGeometry);
  scheduleLayout();
}

void LineNumberGutter::scheduleLayout() {
  if (m_layoutPending)
    return;
  m_layoutPending = true;
  QTimer::singleShot(0, this, [this] {
    m_layoutPending = false;
    updateLines();
    update();
  });
}

void LineNumberGutter::updateLines() {
  // The document's lazy layout stays on the GUI thread. Painting consumes only
  // this viewport snapshot, including with Qt's threaded Metal renderer.
  m_lines.clear();
  if (!m_document)
    return;
  const auto *layout = m_document->documentLayout();
  m_activeLine = m_document->findBlock(m_cursorPosition).blockNumber() + 1;
  const auto position =
      layout->hitTest(QPointF(m_document->documentMargin(),
                              std::max(0.0, m_scrollY - m_textTopPadding)),
                      Qt::FuzzyHit);
  auto block = m_document->findBlock(std::max(0, position));
  while (block.isValid()) {
    const auto rect = layout->blockBoundingRect(block);
    const auto top = rect.top() + m_textTopPadding - m_scrollY;
    if (top > height())
      break;
    if (top + rect.height() >= 0) {
      const auto *textLayout = block.layout();
      if (textLayout && textLayout->lineCount() > 0) {
        const auto firstLine = textLayout->lineAt(0);
        m_lines.append({block.blockNumber() + 1, top, rect.height(),
                        top + firstLine.y() + firstLine.ascent()});
      }
    }
    block = block.next();
  }
}

void LineNumberGutter::paint(QPainter *painter) {
  painter->fillRect(boundingRect(), color(m_colors, "background"));
  painter->setPen(color(m_colors, "border"));
  painter->drawLine(QPointF(width() - 1, 0), QPointF(width() - 1, height()));
  painter->setFont(numberFont());
  for (const auto &line : m_lines) {
    const bool active = line.number == m_activeLine;
    if (active)
      painter->fillRect(QRectF(0, line.top, width() - 1, line.height),
                        color(m_colors, "currentBackground"));
    painter->setPen(color(m_colors, active ? "text" : "secondary"));
    const auto number = QString::number(line.number);
    painter->drawText(
        QPointF(width() - 12 - painter->fontMetrics().horizontalAdvance(number),
                line.baseline),
        number);
  }
}
