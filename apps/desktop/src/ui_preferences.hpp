#pragma once
#include <QEvent>
#include <QFontDatabase>
#include <QGuiApplication>
#include <QObject>
#include <QPalette>
#include <QSettings>
#include <QStyleHints>
class UiPreferences final : public QObject {
  Q_OBJECT
  Q_PROPERTY(QString theme READ theme WRITE setTheme NOTIFY changed)
  Q_PROPERTY(
      QString language READ language WRITE setLanguage NOTIFY languageChanged)
  Q_PROPERTY(bool systemDark READ systemDark NOTIFY changed)
  Q_PROPERTY(bool lineNumbers READ lineNumbers WRITE setLineNumbers NOTIFY
                 lineNumbersChanged)
  Q_PROPERTY(QString editorFontFamily READ editorFontFamily WRITE
                 setEditorFontFamily NOTIFY editorFontChanged)
  Q_PROPERTY(int editorFontSize READ editorFontSize WRITE setEditorFontSize
                 NOTIFY editorFontChanged)
  Q_PROPERTY(QStringList fontFamilies READ fontFamilies CONSTANT)
public:
  explicit UiPreferences(QObject *parent = nullptr) : QObject(parent) {
    connect(QGuiApplication::styleHints(), &QStyleHints::colorSchemeChanged,
            this, [this] { emit changed(); });
    QGuiApplication::instance()->installEventFilter(this);
    applyTheme();
  }
  QString theme() const {
    return m_settings.value("appearance/theme", "system").toString();
  }
  QString language() const {
    return m_settings.value("appearance/language", "system").toString();
  }
  void setTheme(const QString &value) {
    if (!QStringList{"system", "light", "dark"}.contains(value))
      return;
    m_settings.setValue("appearance/theme", value);
    applyTheme();
    emit changed();
  }
  void setLanguage(const QString &value) {
    if (!QStringList{"system", "en", "zh_CN"}.contains(value))
      return;
    m_settings.setValue("appearance/language", value);
    emit languageChanged();
  }
  bool systemDark() const {
    const auto scheme = QGuiApplication::styleHints()->colorScheme();
    return scheme == Qt::ColorScheme::Dark ||
           (scheme == Qt::ColorScheme::Unknown &&
            QGuiApplication::palette().color(QPalette::Window).lightness() <
                128);
  }
  bool lineNumbers() const {
    return m_settings.value("editor/lineNumbers", true).toBool();
  }
  void setLineNumbers(bool visible) {
    if (lineNumbers() == visible)
      return;
    m_settings.setValue("editor/lineNumbers", visible);
    emit lineNumbersChanged();
  }
  QStringList fontFamilies() const { return QFontDatabase::families(); }
  QString editorFontFamily() const {
    const auto selected = m_settings.value("editor/fontFamily").toString();
    return fontFamilies().contains(selected)
               ? selected
               : QFontDatabase::systemFont(QFontDatabase::FixedFont).family();
  }
  int editorFontSize() const {
    return qBound(11, m_settings.value("editor/fontSize", 16).toInt(), 32);
  }
  void setEditorFontFamily(const QString &family) {
    if (!fontFamilies().contains(family) || family == editorFontFamily())
      return;
    m_settings.setValue("editor/fontFamily", family);
    emit editorFontChanged();
  }
  void setEditorFontSize(int size) {
    if (size < 11 || size > 32 || size == editorFontSize())
      return;
    m_settings.setValue("editor/fontSize", size);
    emit editorFontChanged();
  }
  Q_INVOKABLE void resetEditorFont() {
    m_settings.remove("editor/fontFamily");
    m_settings.remove("editor/fontSize");
    emit editorFontChanged();
  }
signals:
  void changed();
  void languageChanged();
  void lineNumbersChanged();
  void editorFontChanged();

private:
  QSettings m_settings;
  void applyTheme() {
    QGuiApplication::styleHints()->setColorScheme(
        theme() == "dark"    ? Qt::ColorScheme::Dark
        : theme() == "light" ? Qt::ColorScheme::Light
                             : Qt::ColorScheme::Unknown);
  }
  bool eventFilter(QObject *, QEvent *event) override {
    if (event->type() == QEvent::ApplicationPaletteChange)
      emit changed();
    return false;
  }
};
