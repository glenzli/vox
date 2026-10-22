cmake_minimum_required(VERSION 3.28)
project(Vox VERSION 0.1.0 LANGUAGES CXX)
include(CTest)
if(CMAKE_BINARY_DIR STREQUAL CMAKE_SOURCE_DIR OR CMAKE_BINARY_DIR MATCHES "^${CMAKE_SOURCE_DIR}/")
    message(FATAL_ERROR "Build Vox outside the source checkout")
endif()
find_package(Qt6 6.9 REQUIRED COMPONENTS Quick QuickControls2 Multimedia Svg LinguistTools)
qt_standard_project_setup(REQUIRES 6.9)
qt_policy(SET QTP0004 NEW)
find_program(VOX_CARGO cargo REQUIRED)
set(VOX_CARGO_TARGET_DIR "${CMAKE_BINARY_DIR}/cargo" CACHE PATH "External Cargo output")
if(VOX_CARGO_TARGET_DIR MATCHES "^${CMAKE_SOURCE_DIR}/")
    message(FATAL_ERROR "Cargo output must stay outside the checkout")
endif()
set(VOX_WORKER "${VOX_CARGO_TARGET_DIR}/debug/vox-cli${CMAKE_EXECUTABLE_SUFFIX}")
file(GLOB_RECURSE VOX_RUST_SOURCES CONFIGURE_DEPENDS crates/*.rs crates/*/Cargo.toml apps/vox-cli/*.rs apps/vox-cli/Cargo.toml)
add_custom_command(OUTPUT "${VOX_WORKER}"
    COMMAND ${CMAKE_COMMAND} -E env "CARGO_TARGET_DIR=${VOX_CARGO_TARGET_DIR}" ${VOX_CARGO} build --locked --offline --package vox-cli
    WORKING_DIRECTORY "${CMAKE_SOURCE_DIR}"
    DEPENDS ${VOX_RUST_SOURCES} Cargo.toml Cargo.lock
    VERBATIM)
add_custom_target(vox-worker-build DEPENDS "${VOX_WORKER}")
qt_add_executable(vox-desktop
    apps/desktop/src/main.cpp
    apps/desktop/src/desktop_backend.cpp apps/desktop/src/desktop_backend.hpp
    apps/desktop/src/worker_client.cpp apps/desktop/src/worker_client.hpp
    apps/desktop/src/echo_launcher.cpp apps/desktop/src/echo_launcher.hpp
    apps/desktop/src/desktop_materials.cpp apps/desktop/src/desktop_production.cpp
    apps/desktop/src/desktop_voice_design.cpp
    apps/desktop/src/desktop_longform.cpp
    apps/desktop/src/reference_recorder.cpp apps/desktop/src/reference_recorder.hpp
    apps/desktop/src/ui_preferences.hpp
    apps/desktop/src/script_editor.cpp apps/desktop/src/script_editor.hpp
    apps/desktop/tests/workflow_smoke.cpp apps/desktop/tests/workflow_smoke.hpp
    apps/desktop/tests/appearance_smoke.cpp apps/desktop/tests/editor_smoke.cpp apps/desktop/tests/studio_smoke.cpp
    apps/desktop/tests/reading_smoke.cpp apps/desktop/tests/shape_transfer_smoke.cpp
    apps/desktop/tests/production_smoke.cpp apps/desktop/tests/sound_workspace_smoke.cpp
    apps/desktop/tests/longform_smoke.cpp)
target_include_directories(vox-desktop PRIVATE apps/desktop/src apps/desktop/tests)
set_source_files_properties(apps/desktop/qml/Theme.qml PROPERTIES QT_QML_SINGLETON_TYPE TRUE)
qt_add_qml_module(vox-desktop URI Vox.Desktop VERSION 1.0
    QML_FILES apps/desktop/qml/Main.qml apps/desktop/qml/Theme.qml
    apps/desktop/qml/IconButton.qml apps/desktop/qml/ActionButton.qml
    apps/desktop/qml/CompactComboBox.qml apps/desktop/qml/CompactSlider.qml
    apps/desktop/qml/CompactSpinBox.qml apps/desktop/qml/CompactTextField.qml
    apps/desktop/qml/MaterialsDialog.qml apps/desktop/qml/ReferenceDialog.qml
    apps/desktop/qml/SoundWorkspace.qml apps/desktop/qml/SegmentsPanel.qml
    apps/desktop/qml/VoiceDesignWorkspace.qml
    apps/desktop/qml/ChaptersPanel.qml
    apps/desktop/qml/PronunciationDialog.qml apps/desktop/qml/SubtitleDialog.qml
    apps/desktop/qml/ScriptEditor.qml
    apps/desktop/qml/ScriptReader.qml apps/desktop/qml/ReadingEntry.qml
    apps/desktop/qml/VoicePanel.qml apps/desktop/qml/AudioPanel.qml
    apps/desktop/qml/ProjectSidebar.qml apps/desktop/qml/SettingsDialog.qml)
file(GLOB VOX_ICONS CONFIGURE_DEPENDS "${CMAKE_SOURCE_DIR}/apps/desktop/icons/*.svg")
qt_add_resources(vox-desktop vox-icons PREFIX "/vox" BASE apps/desktop FILES ${VOX_ICONS})
qt_add_resources(vox-desktop vox-branding PREFIX "/vox" BASE assets FILES assets/branding/app-icon.png)
qt_add_resources(vox-desktop vox-docs PREFIX "/vox" BASE "${CMAKE_SOURCE_DIR}" FILES docs/SPEECH_SCRIPT.md)
qt_add_translations(vox-desktop TS_FILES apps/desktop/translations/vox_zh_CN.ts RESOURCE_PREFIX "/translations" LUPDATE_OPTIONS -no-obsolete LRELEASE_OPTIONS -nounfinished)
target_link_libraries(vox-desktop PRIVATE Qt6::Quick Qt6::QuickControls2 Qt6::Multimedia Qt6::Svg)
target_compile_features(vox-desktop PRIVATE cxx_std_20)
target_compile_definitions(vox-desktop PRIVATE VOX_DEV_ECHO_BINARY="${CMAKE_SOURCE_DIR}/../.echo-local-build/current-debug/Echo.app/Contents/MacOS/Echo")
set_target_properties(vox-desktop PROPERTIES OUTPUT_NAME Vox MACOSX_BUNDLE TRUE
    MACOSX_BUNDLE_BUNDLE_NAME Vox MACOSX_BUNDLE_GUI_IDENTIFIER dev.vox.desktop
    MACOSX_BUNDLE_INFO_PLIST "${CMAKE_SOURCE_DIR}/apps/desktop/Info.plist.in"
    WIN32_EXECUTABLE TRUE)
if(APPLE)
    enable_language(OBJCXX)
    target_sources(vox-desktop PRIVATE apps/desktop/src/mac_titlebar.mm apps/desktop/src/mac_titlebar.hpp
        apps/desktop/tests/mac_titlebar_smoke.mm)
    target_link_libraries(vox-desktop PRIVATE "-framework AppKit")
    target_sources(vox-desktop PRIVATE assets/branding/app-icon.icns)
    set_source_files_properties(assets/branding/app-icon.icns PROPERTIES MACOSX_PACKAGE_LOCATION Resources)
endif()
add_dependencies(vox-desktop vox-worker-build)
add_custom_target(vox-package ALL
    COMMAND ${CMAKE_COMMAND} -E copy_if_different "${VOX_WORKER}" "$<TARGET_FILE_DIR:vox-desktop>/vox-cli${CMAKE_EXECUTABLE_SUFFIX}"
    DEPENDS vox-desktop vox-worker-build VERBATIM)
if(BUILD_TESTING)
    add_test(NAME vox-packaged-startup COMMAND $<TARGET_FILE:vox-desktop> --smoke-exit)
    add_test(NAME vox-packaged-project COMMAND $<TARGET_FILE:vox-desktop> --smoke "${CMAKE_BINARY_DIR}/smoke-${PROJECT_VERSION}.vox")
    add_test(NAME vox-packaged-appearance COMMAND $<TARGET_FILE:vox-desktop> --smoke-appearance "${CMAKE_BINARY_DIR}/appearance-smoke")
    add_test(NAME vox-packaged-editor COMMAND $<TARGET_FILE:vox-desktop> --smoke-editor "${CMAKE_BINARY_DIR}/editor-smoke")
    add_test(NAME vox-packaged-studio COMMAND $<TARGET_FILE:vox-desktop> --smoke-studio "${CMAKE_BINARY_DIR}/studio-smoke")
    add_test(NAME vox-packaged-reading COMMAND $<TARGET_FILE:vox-desktop> --smoke-reading "${CMAKE_BINARY_DIR}/reading-smoke")
    add_test(NAME vox-packaged-production COMMAND $<TARGET_FILE:vox-desktop> --smoke-production "${CMAKE_BINARY_DIR}/production-smoke")
    add_test(NAME vox-packaged-sound-workspace COMMAND $<TARGET_FILE:vox-desktop> --smoke-sound-workspace "${CMAKE_BINARY_DIR}/sound-workspace-smoke")
    add_test(NAME vox-packaged-longform COMMAND $<TARGET_FILE:vox-desktop> --smoke-longform "${CMAKE_BINARY_DIR}/longform-smoke")
    set_tests_properties(vox-packaged-longform vox-packaged-production vox-packaged-sound-workspace vox-packaged-reading vox-packaged-studio vox-packaged-startup vox-packaged-project vox-packaged-appearance vox-packaged-editor PROPERTIES TIMEOUT 30
        ENVIRONMENT "QT_QPA_PLATFORM=offscreen;QT_QUICK_BACKEND=software;XDG_CONFIG_HOME=${CMAKE_BINARY_DIR}/test-config"
        FAIL_REGULAR_EXPRESSION "QQml;ReferenceError;TypeError;Binding loop;failed to load;Unable to assign")
endif()
