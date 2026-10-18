<?xml version="1.0" encoding="utf-8"?>
<!DOCTYPE TS>
<TS version="2.1" language="zh_CN">
<context>
    <name>AudioPanel</name>
    <message>
        <location filename="../qml/AudioPanel.qml" line="37"/>
        <source>Takes</source>
        <translation>录音版本</translation>
    </message>
    <message>
        <location filename="../qml/AudioPanel.qml" line="40"/>
        <source>Accept</source>
        <translation>采用</translation>
    </message>
    <message>
        <location filename="../qml/AudioPanel.qml" line="41"/>
        <source>Discard take</source>
        <translation>丢弃候选</translation>
    </message>
    <message>
        <location filename="../qml/AudioPanel.qml" line="43"/>
        <source>Edit in Echo</source>
        <translation>在 Echo 中编辑</translation>
    </message>
    <message>
        <location filename="../qml/AudioPanel.qml" line="44"/>
        <source>Return edited WAV from Echo</source>
        <translation>取回 Echo 编辑后的 WAV</translation>
    </message>
    <message>
        <location filename="../qml/AudioPanel.qml" line="45"/>
        <location filename="../qml/AudioPanel.qml" line="115"/>
        <source>Export WAV</source>
        <translation>导出 WAV</translation>
    </message>
    <message>
        <location filename="../qml/AudioPanel.qml" line="64"/>
        <source>Accepted version</source>
        <translation>已采用的版本</translation>
    </message>
    <message>
        <location filename="../qml/AudioPanel.qml" line="64"/>
        <source>Candidate take</source>
        <translation>候选录音</translation>
    </message>
    <message>
        <location filename="../qml/AudioPanel.qml" line="68"/>
        <source>Play take</source>
        <translation>试听版本</translation>
    </message>
    <message>
        <location filename="../qml/AudioPanel.qml" line="71"/>
        <source>Echo edit</source>
        <translation>Echo 编辑版</translation>
    </message>
    <message>
        <location filename="../qml/AudioPanel.qml" line="71"/>
        <source>Take %1</source>
        <translation>录音 %1</translation>
    </message>
    <message>
        <location filename="../qml/AudioPanel.qml" line="72"/>
        <source>Accepted</source>
        <translation>已采用</translation>
    </message>
    <message>
        <location filename="../qml/AudioPanel.qml" line="72"/>
        <source>For review</source>
        <translation>待审核</translation>
    </message>
    <message>
        <location filename="../qml/AudioPanel.qml" line="76"/>
        <source>Generate a take to listen, compare and keep your preferred version.</source>
        <translation>生成配音后，即可试听、比较并采用喜欢的版本。</translation>
    </message>
    <message>
        <location filename="../qml/AudioPanel.qml" line="84"/>
        <source>Pause</source>
        <translation>暂停</translation>
    </message>
    <message>
        <location filename="../qml/AudioPanel.qml" line="84"/>
        <source>Play selected take</source>
        <translation>试听选中的录音</translation>
    </message>
    <message>
        <location filename="../qml/AudioPanel.qml" line="88"/>
        <source>Stop playback</source>
        <translation>停止播放</translation>
    </message>
    <message>
        <location filename="../qml/AudioPanel.qml" line="112"/>
        <source>Unmute</source>
        <translation>取消静音</translation>
    </message>
    <message>
        <location filename="../qml/AudioPanel.qml" line="112"/>
        <source>Mute</source>
        <translation>静音</translation>
    </message>
    <message>
        <location filename="../qml/AudioPanel.qml" line="115"/>
        <location filename="../qml/AudioPanel.qml" line="131"/>
        <source>WAV audio (*.wav)</source>
        <translation>WAV 音频 (*.wav)</translation>
    </message>
    <message>
        <location filename="../qml/AudioPanel.qml" line="118"/>
        <source>Return from Echo</source>
        <translation>从 Echo 取回</translation>
    </message>
    <message>
        <location filename="../qml/AudioPanel.qml" line="126"/>
        <source>Export your finished edit from Echo as WAV. Vox will add it as a candidate and keep the original take.</source>
        <translation>在 Echo 中将完成的编辑导出为 WAV。Vox 会将它加入候选，并保留原配音。</translation>
    </message>
    <message>
        <location filename="../qml/AudioPanel.qml" line="127"/>
        <source>Allow 24-bit to 16-bit PCM conversion</source>
        <translation>允许将 24 位 PCM 转为 16 位</translation>
    </message>
    <message>
        <location filename="../qml/AudioPanel.qml" line="131"/>
        <source>Choose Echo WAV export</source>
        <translation>选择 Echo 导出的 WAV</translation>
    </message>
</context>
<context>
    <name>DesktopBackend</name>
    <message>
        <location filename="../src/desktop_backend.cpp" line="392"/>
        <source>[production: dialogue]
[role: Host]
[role: Guest]

[speaker: Host]
Welcome. What would you like to share?
[pause: 0.4s]
[speaker: Guest]
Every story begins with a voice.</source>
        <translation>[production: dialogue]
[role: 主持人]
[role: 来宾]

[speaker: 主持人]
欢迎。今天你想和大家分享什么？
[pause: 0.4s]
[speaker: 来宾]
每个故事，都从一个声音开始。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="397"/>
        <source>[production: listening]
[role: Narrator]
[cue: turn; sound: chime]

[speaker: Narrator]
Listen, then listen once more.
[audio: turn]
[repeat: 2; gap: 2s]
A quiet morning. A new beginning.
[end-repeat]</source>
        <translation>[production: listening]
[role: 旁白]
[cue: 提示; sound: chime]

[speaker: 旁白]
请听下面的材料，稍后将重复播放。
[audio: 提示]
[repeat: 2; gap: 2s]
安静的清晨，一个新的开始。
[end-repeat]</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="401"/>
        <source>[production: narration]
[role: Narrator]

[speaker: Narrator]
Every story begins with a voice.
[pause: 0.8s]
Give your words a moment to breathe.</source>
        <translation>[production: narration]
[role: 旁白]

[speaker: 旁白]
每个故事，都从一个声音开始。
[pause: 0.8s]
给文字一点时间，让它慢慢呼吸。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="543"/>
        <source>All changes saved</source>
        <translation>所有修改已保存</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="545"/>
        <source>Project created</source>
        <translation>工程已创建</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="547"/>
        <source>Project opened</source>
        <translation>工程已打开</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="549"/>
        <source>Your take is ready. Listen, then accept the version you want to keep.</source>
        <translation>配音已生成。请试听，再采用想要保留的版本。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="552"/>
        <source>Audio version accepted</source>
        <translation>已采用音频版本</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="554"/>
        <source>Take discarded</source>
        <translation>已丢弃候选录音</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="556"/>
        <source>WAV exported</source>
        <translation>WAV 已导出</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="558"/>
        <source>Stopping after the current segment…</source>
        <translation>完成当前段落后停止…</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="560"/>
        <source>Voice sample · synthesized audio</source>
        <translation>声音试听 · 合成音频</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="562"/>
        <source>Echo is open. Export your edit as WAV, then return it here for review.</source>
        <translation>Echo 已打开。编辑后导出 WAV，再回到这里取回并审核。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="565"/>
        <source>Echo edit is ready for review. Accept it to keep a new version.</source>
        <translation>Echo 编辑版已加入候选。采用后将保存为新版本。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="573"/>
        <source>Use a pause between 0 and 120 seconds, for example [pause: 0.8s].</source>
        <translation>请使用 0 到 120 秒之间的停顿，例如 [pause: 0.8s]。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="579"/>
        <source>Check the repeat block: use one start and end, spoken content, and no nested repeat or scene boundary.</source>
        <translation>请检查重复块：起止指令要配对，并包含配音内容；不要嵌套重复或跨越场景。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="582"/>
        <source>Declare the role before using it, then choose its speaker voice.</source>
        <translation>请先声明角色，再使用它并选择相应声音。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="585"/>
        <source>Choose a speaker before the spoken passage.</source>
        <translation>请在配音段落前指定说话角色。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="587"/>
        <source>Declare this sound cue before using it and choose a sound or skip it.</source>
        <translation>请先声明音效，再选择声音或明确跳过。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="590"/>
        <source>Place production, role and cue declarations before the spoken content.</source>
        <translation>请把制作形式、角色和音效的声明放在配音内容之前。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="593"/>
        <source>Choose narration, dialogue or listening as the production form.</source>
        <translation>制作形式请选择旁白、对话或听力材料。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="596"/>
        <source>Choose neutral, clear, warm or lively delivery.</source>
        <translation>演绎方式请选择自然、清晰、温暖或轻快。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="598"/>
        <source>Choose a supported speech language or automatic detection.</source>
        <translation>请选择支持的配音语言，或使用自动识别。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="600"/>
        <source>Shorten this script or split it into separate projects.</source>
        <translation>请缩短脚本，或拆分为多个工程。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="603"/>
        <source>Use a supported script instruction on its own line. Open the script guide for examples.</source>
        <translation>请使用支持的脚本指令，并单独放在一行。示例可在脚本指南中查看。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="606"/>
        <source>Add a spoken passage to the script.</source>
        <translation>请在脚本中加入需要配音的段落。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="608"/>
        <source>Check the script, choose a voice for every used role and resolve its sound cues.</source>
        <translation>请检查脚本，为使用中的每个角色选择声音，并处理音效提示。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="611"/>
        <source>Connect Vox to Infer Runtime with its own credential in Settings.</source>
        <translation>请在设置中使用 Vox 自己的凭据连接 Infer Runtime。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="614"/>
        <source>This project changed. Reopen it to review the current version.</source>
        <translation>工程已有变化。请重新打开，查看当前版本。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="616"/>
        <source>The file could not be read or written. Check its location and permissions.</source>
        <translation>无法读写文件，请检查文件位置与权限。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="619"/>
        <source>That location already exists. Choose a new name.</source>
        <translation>该位置已存在，请使用新的名称。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="621"/>
        <source>Synthesis stopped. Existing audio is kept.</source>
        <translation>合成已停止，已有音频仍会保留。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="623"/>
        <source>Sound cues need a 24 kHz mono, 16-bit PCM WAV file, up to 8 MB.</source>
        <translation>音效需要 24 kHz、单声道、16 位 PCM WAV 文件，最大 8 MB。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="626"/>
        <source>Allow conversion from 24-bit to 16-bit WAV to review this Echo export.</source>
        <translation>请允许将 24 位 WAV 转为 16 位，以审核这份 Echo 导出。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="629"/>
        <source>Choose the Echo executable in Settings, then try again.</source>
        <translation>请先在设置中选择 Echo 可执行文件，然后重试。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="631"/>
        <source>This Echo build does not support independent editing. Choose a current Echo build in Settings.</source>
        <translation>这个 Echo 构建不支持独立编辑，请在设置中选择支持该功能的新版本。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="634"/>
        <source>Echo is opening. Wait a moment, then try again.</source>
        <translation>正在打开 Echo，请稍等片刻。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="636"/>
        <source>Accept this take before opening it in Echo.</source>
        <translation>请先采用这个候选，再用 Echo 打开。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="638"/>
        <source>Write or import a script first.</source>
        <translation>请先编写或导入脚本。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="640"/>
        <source>A voice is being rendered. Wait for it to finish or stop it.</source>
        <translation>配音正在生成，请等待完成或停止生成。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="642"/>
        <source>Choose a UTF-8 text file.</source>
        <translation>请选择 UTF-8 编码的文本文件。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="644"/>
        <source>Use the .vox extension for this project.</source>
        <translation>请使用 .vox 作为工程扩展名。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="646"/>
        <source>This audio format is unsupported. Export a PCM WAV from Echo.</source>
        <translation>暂不支持这个音频格式，请在 Echo 中导出 PCM WAV。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="649"/>
        <source>Infer Runtime is unavailable. You can keep editing and listening to saved audio.</source>
        <translation>Infer Runtime 暂不可用。你仍然可以编辑脚本、试听已保存的音频。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="651"/>
        <source>The operation could not be completed (%1).</source>
        <translation>操作未能完成（%1）。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="655"/>
        <source>Vivian · Bright</source>
        <translation>Vivian · 明亮</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="657"/>
        <source>Serena · Warm</source>
        <translation>Serena · 温暖</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="659"/>
        <source>Uncle Fu · Mature</source>
        <translation>Uncle Fu · 成熟</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="661"/>
        <source>Dylan · Beijing</source>
        <translation>Dylan · 北京口音</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="663"/>
        <source>Eric · Sichuan</source>
        <translation>Eric · 四川口音</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="665"/>
        <source>Ryan · Dynamic</source>
        <translation>Ryan · 生动</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="667"/>
        <source>Aiden · Warm</source>
        <translation>Aiden · 温暖</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="669"/>
        <source>Ono Anna · Bright</source>
        <translation>Ono Anna · 明亮</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="671"/>
        <source>Sohee · Warm</source>
        <translation>Sohee · 温暖</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="676"/>
        <source>Automatic</source>
        <translation>自动识别</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="678"/>
        <source>Chinese</source>
        <translation>中文</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="680"/>
        <source>English</source>
        <translation>英语</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="682"/>
        <source>Japanese</source>
        <translation>日语</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="684"/>
        <source>Korean</source>
        <translation>韩语</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="686"/>
        <source>German</source>
        <translation>德语</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="688"/>
        <source>French</source>
        <translation>法语</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="690"/>
        <source>Russian</source>
        <translation>俄语</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="692"/>
        <source>Portuguese</source>
        <translation>葡萄牙语</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="694"/>
        <source>Spanish</source>
        <translation>西班牙语</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="696"/>
        <source>Italian</source>
        <translation>意大利语</translation>
    </message>
</context>
<context>
    <name>Main</name>
    <message>
        <location filename="../qml/Main.qml" line="84"/>
        <source>Script-led voice production</source>
        <translation>以脚本为中心的配音制作</translation>
    </message>
    <message>
        <location filename="../qml/Main.qml" line="85"/>
        <source>Saving…</source>
        <translation>保存中…</translation>
    </message>
    <message>
        <location filename="../qml/Main.qml" line="85"/>
        <source>Saved</source>
        <translation>已保存</translation>
    </message>
    <message>
        <location filename="../qml/Main.qml" line="86"/>
        <location filename="../qml/Main.qml" line="222"/>
        <source>New project</source>
        <translation>新建工程</translation>
    </message>
    <message>
        <location filename="../qml/Main.qml" line="87"/>
        <location filename="../qml/Main.qml" line="223"/>
        <source>Open project</source>
        <translation>打开工程</translation>
    </message>
    <message>
        <location filename="../qml/Main.qml" line="88"/>
        <source>Save project</source>
        <translation>保存工程</translation>
    </message>
    <message>
        <location filename="../qml/Main.qml" line="89"/>
        <source>Settings</source>
        <translation>设置</translation>
    </message>
    <message>
        <location filename="../qml/Main.qml" line="113"/>
        <source>The script</source>
        <translation>脚本</translation>
    </message>
    <message>
        <location filename="../qml/Main.qml" line="115"/>
        <source>Import text or script</source>
        <translation>导入文稿或脚本</translation>
    </message>
    <message>
        <location filename="../qml/Main.qml" line="116"/>
        <location filename="../qml/Main.qml" line="241"/>
        <source>Script guide</source>
        <translation>脚本指南</translation>
    </message>
    <message>
        <location filename="../qml/Main.qml" line="123"/>
        <source>Plain narration</source>
        <translation>普通旁白</translation>
    </message>
    <message>
        <location filename="../qml/Main.qml" line="123"/>
        <source>Production script</source>
        <translation>制作脚本</translation>
    </message>
    <message>
        <location filename="../qml/Main.qml" line="133"/>
        <source>Line numbers</source>
        <translation>行号</translation>
    </message>
    <message>
        <location filename="../qml/Main.qml" line="141"/>
        <source>Write</source>
        <translation>编写</translation>
    </message>
    <message>
        <location filename="../qml/Main.qml" line="142"/>
        <source>Read</source>
        <translation>阅读</translation>
    </message>
    <message>
        <location filename="../qml/Main.qml" line="176"/>
        <source>Pause · %1 s</source>
        <translation>停顿 · %1 秒</translation>
    </message>
    <message>
        <location filename="../qml/Main.qml" line="177"/>
        <source>Sound · %1</source>
        <translation>音效 · %1</translation>
    </message>
    <message>
        <location filename="../qml/Main.qml" line="178"/>
        <source>Scene · %1</source>
        <translation>场景 · %1</translation>
    </message>
    <message>
        <location filename="../qml/Main.qml" line="179"/>
        <source>Repeat %1 times · same recording</source>
        <translation>重复 %1 次 · 重播同一录音</translation>
    </message>
    <message>
        <location filename="../qml/Main.qml" line="180"/>
        <source>End repeat</source>
        <translation>重复结束</translation>
    </message>
    <message>
        <location filename="../qml/Main.qml" line="192"/>
        <source>Line %1 · %2</source>
        <translation>第 %1 行 · %2</translation>
    </message>
    <message>
        <location filename="../qml/Main.qml" line="196"/>
        <source>%1 characters</source>
        <translation>%1 个字符</translation>
    </message>
    <message>
        <location filename="../qml/Main.qml" line="198"/>
        <source>%1 / %2 segments</source>
        <translation>%1 / %2 段</translation>
    </message>
    <message>
        <location filename="../qml/Main.qml" line="198"/>
        <source>Preparing…</source>
        <translation>准备中…</translation>
    </message>
    <message>
        <location filename="../qml/Main.qml" line="198"/>
        <source>Ready to render</source>
        <translation>可以生成配音</translation>
    </message>
    <message>
        <location filename="../qml/Main.qml" line="198"/>
        <source>Review voices and script</source>
        <translation>请检查脚本与角色声音</translation>
    </message>
    <message>
        <location filename="../qml/Main.qml" line="199"/>
        <source>Stop after current segment</source>
        <translation>完成当前段落后停止</translation>
    </message>
    <message>
        <location filename="../qml/Main.qml" line="200"/>
        <source>Rendering…</source>
        <translation>生成中…</translation>
    </message>
    <message>
        <location filename="../qml/Main.qml" line="200"/>
        <source>Render voice</source>
        <translation>生成配音</translation>
    </message>
    <message>
        <location filename="../qml/Main.qml" line="218"/>
        <source>Give your words a voice.</source>
        <translation>让文字，拥有声音。</translation>
    </message>
    <message>
        <location filename="../qml/Main.qml" line="219"/>
        <source>A quiet space for narration, dialogue and listening materials.
Write the script. Direct the voices. Keep the take.</source>
        <translation>专注旁白、对话与听力材料的创作空间。
写下脚本，安排声音，留下满意的录音。</translation>
    </message>
    <message>
        <location filename="../qml/Main.qml" line="235"/>
        <source>Your project stays on this device.</source>
        <translation>工程保存在这台设备上。</translation>
    </message>
    <message>
        <location filename="../qml/Main.qml" line="250"/>
        <source>A voice is still rendering</source>
        <translation>配音仍在生成</translation>
    </message>
    <message>
        <location filename="../qml/Main.qml" line="255"/>
        <source>Stop after the current segment, then close Vox?</source>
        <translation>完成当前段落后停止生成，然后关闭 Vox？</translation>
    </message>
    <message>
        <location filename="../qml/Main.qml" line="258"/>
        <source>New Vox project</source>
        <translation>新建 Vox 工程</translation>
    </message>
    <message>
        <location filename="../qml/Main.qml" line="258"/>
        <location filename="../qml/Main.qml" line="259"/>
        <source>Vox project (*.vox)</source>
        <translation>Vox 工程 (*.vox)</translation>
    </message>
    <message>
        <location filename="../qml/Main.qml" line="259"/>
        <source>Open Vox project</source>
        <translation>打开 Vox 工程</translation>
    </message>
    <message>
        <location filename="../qml/Main.qml" line="260"/>
        <source>Import script</source>
        <translation>导入脚本</translation>
    </message>
    <message>
        <location filename="../qml/Main.qml" line="260"/>
        <source>Text files (*.txt *.md)</source>
        <translation>文本文件 (*.txt *.md)</translation>
    </message>
    <message>
        <location filename="../qml/Main.qml" line="260"/>
        <source>All files (*)</source>
        <translation>所有文件 (*)</translation>
    </message>
</context>
<context>
    <name>ProjectSidebar</name>
    <message>
        <location filename="../qml/ProjectSidebar.qml" line="11"/>
        <source>PRODUCTION</source>
        <translation>配音制作</translation>
    </message>
    <message>
        <location filename="../qml/ProjectSidebar.qml" line="13"/>
        <location filename="../qml/ProjectSidebar.qml" line="14"/>
        <source>Script</source>
        <translation>脚本</translation>
    </message>
    <message>
        <location filename="../qml/ProjectSidebar.qml" line="16"/>
        <source>Start with a form</source>
        <translation>从一种形式开始</translation>
    </message>
    <message>
        <location filename="../qml/ProjectSidebar.qml" line="17"/>
        <source>Narration</source>
        <translation>旁白</translation>
    </message>
    <message>
        <location filename="../qml/ProjectSidebar.qml" line="18"/>
        <source>Dialogue</source>
        <translation>对话</translation>
    </message>
    <message>
        <location filename="../qml/ProjectSidebar.qml" line="19"/>
        <source>Listening</source>
        <translation>听力材料</translation>
    </message>
    <message>
        <location filename="../qml/ProjectSidebar.qml" line="23"/>
        <source>SCRIPT HISTORY</source>
        <translation>脚本历史</translation>
    </message>
    <message>
        <location filename="../qml/ProjectSidebar.qml" line="26"/>
        <source>A script version is kept each time you render a revision.</source>
        <translation>每次生成配音时，都会保存对应的脚本版本。</translation>
    </message>
    <message>
        <location filename="../qml/ProjectSidebar.qml" line="39"/>
        <source>Script %1</source>
        <translation>脚本 %1</translation>
    </message>
    <message>
        <location filename="../qml/ProjectSidebar.qml" line="44"/>
        <source>Restore this script into the working draft</source>
        <translation>将这个脚本版本恢复为工作草稿</translation>
    </message>
    <message>
        <location filename="../qml/ProjectSidebar.qml" line="51"/>
        <source>Local voice ready</source>
        <translation>本地语音已就绪</translation>
    </message>
    <message>
        <location filename="../qml/ProjectSidebar.qml" line="51"/>
        <source>Voice engine offline</source>
        <translation>语音引擎未连接</translation>
    </message>
    <message>
        <location filename="../qml/ProjectSidebar.qml" line="53"/>
        <source>Vox · Development preview</source>
        <translation>Vox · 开发预览</translation>
    </message>
</context>
<context>
    <name>ScriptEditor</name>
    <message>
        <location filename="../qml/ScriptEditor.qml" line="34"/>
        <source>Start with your words.

Write a narration, import an existing script, or choose a form on the left.</source>
        <translation>从你的文字开始。

写一段旁白，导入已有的脚本，或从左侧选择一种形式。</translation>
    </message>
    <message>
        <location filename="../qml/ScriptEditor.qml" line="41"/>
        <source>Script editor</source>
        <translation>脚本编辑器</translation>
    </message>
</context>
<context>
    <name>SettingsDialog</name>
    <message>
        <location filename="../qml/SettingsDialog.qml" line="7"/>
        <source>Settings</source>
        <translation>设置</translation>
    </message>
    <message>
        <location filename="../qml/SettingsDialog.qml" line="16"/>
        <source>Appearance</source>
        <translation>外观</translation>
    </message>
    <message>
        <location filename="../qml/SettingsDialog.qml" line="18"/>
        <source>Theme</source>
        <translation>主题</translation>
    </message>
    <message>
        <location filename="../qml/SettingsDialog.qml" line="21"/>
        <location filename="../qml/SettingsDialog.qml" line="30"/>
        <source>Follow system</source>
        <translation>跟随系统</translation>
    </message>
    <message>
        <location filename="../qml/SettingsDialog.qml" line="21"/>
        <source>Light</source>
        <translation>亮色</translation>
    </message>
    <message>
        <location filename="../qml/SettingsDialog.qml" line="21"/>
        <source>Dark</source>
        <translation>暗色</translation>
    </message>
    <message>
        <location filename="../qml/SettingsDialog.qml" line="27"/>
        <source>Language</source>
        <translation>界面语言</translation>
    </message>
    <message>
        <location filename="../qml/SettingsDialog.qml" line="36"/>
        <source>Voice synthesis</source>
        <translation>语音合成</translation>
    </message>
    <message>
        <location filename="../qml/SettingsDialog.qml" line="38"/>
        <source>Infer Runtime is connected</source>
        <translation>Infer Runtime 已连接</translation>
    </message>
    <message>
        <location filename="../qml/SettingsDialog.qml" line="38"/>
        <source>Infer Runtime needs a connection</source>
        <translation>需要连接 Infer Runtime</translation>
    </message>
    <message>
        <location filename="../qml/SettingsDialog.qml" line="45"/>
        <source>Vox uses its own local authorization. Import the credential issued for Vox in Infer Console.</source>
        <translation>Vox 使用独立的本地授权。请导入 Infer Console 为 Vox 签发的凭据。</translation>
    </message>
    <message>
        <location filename="../qml/SettingsDialog.qml" line="48"/>
        <source>Import credential</source>
        <translation>导入凭据</translation>
    </message>
    <message>
        <location filename="../qml/SettingsDialog.qml" line="49"/>
        <source>Check connection</source>
        <translation>检查连接</translation>
    </message>
    <message>
        <location filename="../qml/SettingsDialog.qml" line="52"/>
        <source>Edit in Echo</source>
        <translation>在 Echo 中编辑</translation>
    </message>
    <message>
        <location filename="../qml/SettingsDialog.qml" line="53"/>
        <source>Open accepted audio in Echo’s independent editor. Return its WAV export to review a new take.</source>
        <translation>将已采用的音频送到 Echo 独立编辑。导出 WAV 后返回这里，作为新的候选版本审核。</translation>
    </message>
    <message>
        <location filename="../qml/SettingsDialog.qml" line="56"/>
        <location filename="../qml/SettingsDialog.qml" line="60"/>
        <source>Choose Echo executable</source>
        <translation>选择 Echo 可执行文件</translation>
    </message>
    <message>
        <location filename="../qml/SettingsDialog.qml" line="59"/>
        <source>Choose Vox credential</source>
        <translation>选择 Vox 凭据</translation>
    </message>
</context>
<context>
    <name>VoicePanel</name>
    <message>
        <location filename="../qml/VoicePanel.qml" line="27"/>
        <source>VOICES &amp; DIRECTION</source>
        <translation>声音与演绎</translation>
    </message>
    <message>
        <location filename="../qml/VoicePanel.qml" line="28"/>
        <source>Default voice</source>
        <translation>默认声音</translation>
    </message>
    <message>
        <location filename="../qml/VoicePanel.qml" line="40"/>
        <source>Audition this voice</source>
        <translation>试听这个声音</translation>
    </message>
    <message>
        <location filename="../qml/VoicePanel.qml" line="42"/>
        <source>Language</source>
        <translation>配音语言</translation>
    </message>
    <message>
        <location filename="../qml/VoicePanel.qml" line="51"/>
        <source>Pace</source>
        <translation>语速</translation>
    </message>
    <message>
        <location filename="../qml/VoicePanel.qml" line="61"/>
        <source>Speech pace</source>
        <translation>配音语速</translation>
    </message>
    <message>
        <location filename="../qml/VoicePanel.qml" line="64"/>
        <source>Roles</source>
        <translation>角色</translation>
    </message>
    <message>
        <location filename="../qml/VoicePanel.qml" line="66"/>
        <source>Declare roles in your script, then choose a voice for each one.</source>
        <translation>在脚本中声明角色，再为每个角色选择声音。</translation>
    </message>
    <message>
        <location filename="../qml/VoicePanel.qml" line="84"/>
        <source>Choose a voice</source>
        <translation>选择声音</translation>
    </message>
    <message>
        <location filename="../qml/VoicePanel.qml" line="88"/>
        <source>Audition role voice</source>
        <translation>试听角色声音</translation>
    </message>
    <message>
        <location filename="../qml/VoicePanel.qml" line="92"/>
        <source>Sound cues</source>
        <translation>音效提示</translation>
    </message>
    <message>
        <location filename="../qml/VoicePanel.qml" line="104"/>
        <source>From script</source>
        <translation>使用脚本声明</translation>
    </message>
    <message>
        <location filename="../qml/VoicePanel.qml" line="104"/>
        <source>Chime</source>
        <translation>提示音</translation>
    </message>
    <message>
        <location filename="../qml/VoicePanel.qml" line="104"/>
        <source>Skip</source>
        <translation>跳过</translation>
    </message>
    <message>
        <location filename="../qml/VoicePanel.qml" line="104"/>
        <source>Imported WAV</source>
        <translation>导入的 WAV</translation>
    </message>
    <message>
        <location filename="../qml/VoicePanel.qml" line="109"/>
        <source>Import cue WAV</source>
        <translation>导入音效 WAV</translation>
    </message>
    <message>
        <location filename="../qml/VoicePanel.qml" line="114"/>
        <source>Generated speech is labeled as synthetic. Your accepted takes stay available when you revise the script.</source>
        <translation>生成的配音会标为合成音频。修改脚本后，已采用的录音版本仍会保留。</translation>
    </message>
    <message>
        <location filename="../qml/VoicePanel.qml" line="117"/>
        <source>Import sound cue</source>
        <translation>导入音效</translation>
    </message>
    <message>
        <location filename="../qml/VoicePanel.qml" line="117"/>
        <source>WAV audio (*.wav)</source>
        <translation>WAV 音频 (*.wav)</translation>
    </message>
</context>
</TS>
