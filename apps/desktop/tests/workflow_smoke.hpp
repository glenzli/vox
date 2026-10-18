#pragma once
#include <QString>
class QGuiApplication;
class QQmlApplicationEngine;
class UiPreferences;
class DesktopBackend;
void startWorkflowSmoke(QGuiApplication &, QQmlApplicationEngine &,
                        UiPreferences &, DesktopBackend &, const QString &path);
void startAudioSmoke(QGuiApplication &, QQmlApplicationEngine &,
                     DesktopBackend &, const QString &path);
void startAppearanceSmoke(QGuiApplication &, QQmlApplicationEngine &,
                          UiPreferences &, DesktopBackend &,
                          const QString &path);
void startEditorSmoke(QGuiApplication &, QQmlApplicationEngine &,
                      UiPreferences &, DesktopBackend &, const QString &path);

void startStudioSmoke(QGuiApplication &, QQmlApplicationEngine &,
                      UiPreferences &, DesktopBackend &, const QString &);
void startReadingSmoke(QGuiApplication &, QQmlApplicationEngine &,
                       UiPreferences &, DesktopBackend &, const QString &);

void startShapeTransferSmoke(QGuiApplication &, QQmlApplicationEngine &,
                             DesktopBackend &, const QString &output);
