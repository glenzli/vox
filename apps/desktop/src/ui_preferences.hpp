#pragma once
#include <QEvent>
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
signals:
  void changed();
  void languageChanged();
  void lineNumbersChanged();

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
