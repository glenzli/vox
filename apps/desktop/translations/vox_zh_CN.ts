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
    <message>
        <source>Export subtitles</source>
        <translation>导出字幕</translation>
    </message>
</context>
<context>
    <name>DesktopBackend</name>
    <message>
        <location filename="../src/desktop_backend.cpp" line="398"/>
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
        <location filename="../src/desktop_backend.cpp" line="403"/>
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
        <location filename="../src/desktop_backend.cpp" line="407"/>
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
        <location filename="../src/desktop_backend.cpp" line="549"/>
        <source>Material saved in this project</source>
        <translation>素材已保存在此项目中</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="551"/>
        <source>Sound candidate is ready. Listen, then accept it for script use.</source>
        <translation>音效候选已生成，试听后可采用到脚本中。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="554"/>
        <source>All changes saved</source>
        <translation>所有修改已保存</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="556"/>
        <source>Project created</source>
        <translation>工程已创建</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="558"/>
        <source>Project opened</source>
        <translation>工程已打开</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="560"/>
        <source>Your take is ready. Listen, then accept the version you want to keep.</source>
        <translation>配音已生成。请试听，再采用想要保留的版本。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="563"/>
        <source>Audio version accepted</source>
        <translation>已采用音频版本</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="565"/>
        <source>Take discarded</source>
        <translation>已丢弃候选录音</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="567"/>
        <source>WAV exported</source>
        <translation>WAV 已导出</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="569"/>
        <source>Stopping after the current segment…</source>
        <translation>完成当前段落后停止…</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="571"/>
        <source>Voice sample · synthesized audio</source>
        <translation>声音试听 · 合成音频</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="573"/>
        <source>Echo is open. Export your edit as WAV, then return it here for review.</source>
        <translation>Echo 已打开。编辑后导出 WAV，再回到这里取回并审核。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="576"/>
        <source>Echo edit is ready for review. Accept it to keep a new version.</source>
        <translation>Echo 编辑版已加入候选。采用后将保存为新版本。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="584"/>
        <source>Confirm that you may use this voice for local synthesis.</source>
        <translation>请确认你有权使用此声音进行本机合成。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="586"/>
        <source>Name the voice and enter the exact words spoken in its reference recording.</source>
        <translation>请填写音色名称和参考录音中准确的台词。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="589"/>
        <source>Use a mono or stereo WAV reference between 0.5 and 120 seconds.</source>
        <translation>请选择 0.5 至 120 秒的单声道或立体声 WAV 参考录音。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="592"/>
        <source>The reference voice is missing or changed. Import it again.</source>
        <translation>参考音色缺失或已更改，请重新导入。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="594"/>
        <source>Reference voices currently require 1× pace and neutral delivery.</source>
        <translation>音色模仿目前支持 1× 语速与自然平稳的演绎。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="597"/>
        <source>Listen and accept this sound before using it in a script.</source>
        <translation>请先试听并采用此音效，再用于脚本。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="599"/>
        <source>This material is missing or invalid.</source>
        <translation>此素材缺失或无效。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="601"/>
        <source>Describe a sound in at most 2000 bytes and choose 1–30 seconds.</source>
        <translation>声音描述最多 2000 字节，时长可选 1 至 30 秒。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="604"/>
        <source>The sound description could not be prepared faithfully. Revise it or try again.</source>
        <translation>声音描述未能准确转换，请修改描述或重试。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="607"/>
        <source>Use a mono or stereo sound of at most 120 seconds.</source>
        <translation>请选择不超过 120 秒的单声道或立体声音效。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="609"/>
        <source>Sound generation cancelled. No candidate was added.</source>
        <translation>音效生成已取消，未添加候选。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="612"/>
        <source>Use a pause between 0 and 120 seconds, for example [pause: 0.8s].</source>
        <translation>请使用 0 到 120 秒之间的停顿，例如 [pause: 0.8s]。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="618"/>
        <source>Check the repeat block: use one start and end, spoken content, and no nested repeat or scene boundary.</source>
        <translation>请检查重复块：起止指令要配对，并包含配音内容；不要嵌套重复或跨越场景。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="621"/>
        <source>Declare the role before using it, then choose its speaker voice.</source>
        <translation>请先声明角色，再使用它并选择相应声音。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="624"/>
        <source>Choose a speaker before the spoken passage.</source>
        <translation>请在配音段落前指定说话角色。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="626"/>
        <source>Declare this sound cue before using it and choose a sound or skip it.</source>
        <translation>请先声明音效，再选择声音或明确跳过。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="629"/>
        <source>Place production, role and cue declarations before the spoken content.</source>
        <translation>请把制作形式、角色和音效的声明放在配音内容之前。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="632"/>
        <source>Choose narration, dialogue or listening as the production form.</source>
        <translation>制作形式请选择旁白、对话或听力材料。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="635"/>
        <source>Choose neutral, clear, warm or lively delivery.</source>
        <translation>演绎方式请选择自然、清晰、温暖或轻快。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="637"/>
        <source>Choose a supported speech language or automatic detection.</source>
        <translation>请选择支持的配音语言，或使用自动识别。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="639"/>
        <source>Shorten this script or split it into separate projects.</source>
        <translation>请缩短脚本，或拆分为多个工程。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="642"/>
        <source>Use a supported script instruction on its own line. Open the script guide for examples.</source>
        <translation>请使用支持的脚本指令，并单独放在一行。示例可在脚本指南中查看。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="645"/>
        <source>Add a spoken passage to the script.</source>
        <translation>请在脚本中加入需要配音的段落。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="647"/>
        <source>Check the script, choose a voice for every used role and resolve its sound cues.</source>
        <translation>请检查脚本，为使用中的每个角色选择声音，并处理音效提示。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="650"/>
        <source>Connect Vox to Infer Runtime with its own credential in Settings.</source>
        <translation>请在设置中使用 Vox 自己的凭据连接 Infer Runtime。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="653"/>
        <source>This project changed. Reopen it to review the current version.</source>
        <translation>工程已有变化。请重新打开，查看当前版本。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="655"/>
        <source>The file could not be read or written. Check its location and permissions.</source>
        <translation>无法读写文件，请检查文件位置与权限。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="658"/>
        <source>That location already exists. Choose a new name.</source>
        <translation>该位置已存在，请使用新的名称。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="660"/>
        <source>Synthesis stopped. Existing audio is kept.</source>
        <translation>合成已停止，已有音频仍会保留。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="662"/>
        <source>Sound cues need a 24 kHz mono, 16-bit PCM WAV file, up to 8 MB.</source>
        <translation>音效需要 24 kHz、单声道、16 位 PCM WAV 文件，最大 8 MB。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="665"/>
        <source>Allow conversion from 24-bit to 16-bit WAV to review this Echo export.</source>
        <translation>请允许将 24 位 WAV 转为 16 位，以审核这份 Echo 导出。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="668"/>
        <source>Choose the Echo executable in Settings, then try again.</source>
        <translation>请先在设置中选择 Echo 可执行文件，然后重试。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="670"/>
        <source>This Echo build does not support independent editing. Choose a current Echo build in Settings.</source>
        <translation>这个 Echo 构建不支持独立编辑，请在设置中选择支持该功能的新版本。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="673"/>
        <source>Echo is opening. Wait a moment, then try again.</source>
        <translation>正在打开 Echo，请稍等片刻。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="675"/>
        <source>Accept this take before opening it in Echo.</source>
        <translation>请先采用这个候选，再用 Echo 打开。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="677"/>
        <source>Write or import a script first.</source>
        <translation>请先编写或导入脚本。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="679"/>
        <source>A voice is being rendered. Wait for it to finish or stop it.</source>
        <translation>配音正在生成，请等待完成或停止生成。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="681"/>
        <source>Choose a UTF-8 text file.</source>
        <translation>请选择 UTF-8 编码的文本文件。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="683"/>
        <source>Use the .vox extension for this project.</source>
        <translation>请使用 .vox 作为工程扩展名。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="685"/>
        <source>This audio format is unsupported. Export a PCM WAV from Echo.</source>
        <translation>暂不支持这个音频格式，请在 Echo 中导出 PCM WAV。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="688"/>
        <source>Infer Runtime is unavailable. You can keep editing and listening to saved audio.</source>
        <translation>Infer Runtime 暂不可用。你仍然可以编辑脚本、试听已保存的音频。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="690"/>
        <source>The operation could not be completed (%1).</source>
        <translation>操作未能完成（%1）。</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="694"/>
        <source>Vivian · Bright</source>
        <translation>Vivian · 明亮</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="696"/>
        <source>Serena · Warm</source>
        <translation>Serena · 温暖</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="698"/>
        <source>Uncle Fu · Mature</source>
        <translation>Uncle Fu · 成熟</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="700"/>
        <source>Dylan · Beijing</source>
        <translation>Dylan · 北京口音</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="702"/>
        <source>Eric · Sichuan</source>
        <translation>Eric · 四川口音</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="704"/>
        <source>Ryan · Dynamic</source>
        <translation>Ryan · 生动</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="706"/>
        <source>Aiden · Warm</source>
        <translation>Aiden · 温暖</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="708"/>
        <source>Ono Anna · Bright</source>
        <translation>Ono Anna · 明亮</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="710"/>
        <source>Sohee · Warm</source>
        <translation>Sohee · 温暖</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="715"/>
        <source>Automatic</source>
        <translation>自动识别</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="717"/>
        <source>Chinese</source>
        <translation>中文</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="719"/>
        <source>English</source>
        <translation>英语</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="721"/>
        <source>Japanese</source>
        <translation>日语</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="723"/>
        <source>Korean</source>
        <translation>韩语</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="725"/>
        <source>German</source>
        <translation>德语</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="727"/>
        <source>French</source>
        <translation>法语</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="729"/>
        <source>Russian</source>
        <translation>俄语</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="731"/>
        <source>Portuguese</source>
        <translation>葡萄牙语</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="733"/>
        <source>Spanish</source>
        <translation>西班牙语</translation>
    </message>
    <message>
        <location filename="../src/desktop_backend.cpp" line="735"/>
        <source>Italian</source>
        <translation>意大利语</translation>
    </message>
    <message>
        <location filename="../src/desktop_materials.cpp" line="163"/>
        <source>Soft chime</source>
        <translation>轻柔提示音</translation>
    </message>
    <message>
        <location filename="../src/desktop_materials.cpp" line="165"/>
        <source>Double beep</source>
        <translation>双提示音</translation>
    </message>
    <message>
        <location filename="../src/desktop_materials.cpp" line="167"/>
        <source>Click</source>
        <translation>轻点</translation>
    </message>
    <message>
        <location filename="../src/desktop_materials.cpp" line="169"/>
        <source>Wood block</source>
        <translation>木鱼敲击</translation>
    </message>
    <message>
        <location filename="../src/desktop_materials.cpp" line="171"/>
        <source>Bell</source>
        <translation>铃声</translation>
    </message>
    <message>
        <location filename="../src/desktop_materials.cpp" line="173"/>
        <source>Success</source>
        <translation>完成提示</translation>
    </message>
    <message>
        <location filename="../src/desktop_materials.cpp" line="175"/>
        <source>Whoosh</source>
        <translation>掠过</translation>
    </message>
    <message>
        <location filename="../src/desktop_materials.cpp" line="177"/>
        <source>Countdown</source>
        <translation>倒计时</translation>
    </message>
    <message>
        <source>Shape material imported. Review preserved drafts and source receipts before continuing.</source>
        <translation>Shape 素材已导入。请先检查保留的草稿与来源回执。</translation>
    </message>
    <message>
        <source>Some imported voice settings need review. Select supported voices or reimport missing references before rendering.</source>
        <translation>部分导入的配音设置需要检查。请选择支持的音色，或重新导入缺失的参考录音，再生成配音。</translation>
    </message>
    <message>
        <source>This Shape transfer is incomplete or changed. Export it again from Shape.</source>
        <translation>此 Shape 迁移包不完整或已改变。请从 Shape 重新导出。</translation>
    </message>
    <message>
        <source>This project belongs to another transfer. Choose a new .vox location.</source>
        <translation>此项目属于另一个迁移包。请选择新的 .vox 位置。</translation>
    </message>
    <message>
        <source>Shape import cancelled. No project was created.</source>
        <translation>Shape 导入已取消，未创建项目。</translation>
    </message>
    <message>
        <source>Segment candidate ready. Listen and choose the take to use.</source>
        <translation>分段候选已生成。试听并选择要采用的版本。</translation>
    </message>
    <message>
        <source>Segment selected for assembly</source>
        <translation>已选定用于拼装的分段录音</translation>
    </message>
    <message>
        <source>Assembled take ready for review. Earlier audio is kept.</source>
        <translation>拼装录音已生成，等待审阅。旧录音仍保留。</translation>
    </message>
    <message>
        <source>Subtitles exported</source>
        <translation>字幕已导出</translation>
    </message>
    <message>
        <source>Sound variations are ready. Listen and keep your preferred candidates.</source>
        <translation>声音变体已生成，试听并保留喜欢的候选。</translation>
    </message>
    <message>
        <source>Generation stopped. Completed sound candidates are kept.</source>
        <translation>生成已停止，已完成的声音候选仍保留。</translation>
    </message>
    <message>
        <source>Some variations could not be generated. Completed candidates are kept.</source>
        <translation>部分变体生成失败，已完成的候选仍保留。</translation>
    </message>
    <message>
        <source>Choose an accepted take for every segment before assembly.</source>
        <translation>请先为每个分段采用一个录音，再进行拼装。</translation>
    </message>
    <message>
        <source>This segment changed. Review the current script and generate a new alternative.</source>
        <translation>此分段已改变，请检查当前脚本并重新生成候选。</translation>
    </message>
    <message>
        <source>Check pronunciation rules for empty text, duplicate matches or unsupported filters.</source>
        <translation>请检查发音规则中的空文本、重复匹配或不支持的筛选条件。</translation>
    </message>
    <message>
        <source>Assemble a take from segments before exporting measured subtitles.</source>
        <translation>请先将分段拼装为录音，再导出带实际时长的字幕。</translation>
    </message>
</context>
<context>
    <name>Main</name>
    <message>
        <source>Keep open</source>
        <translation>保持打开</translation>
    </message>
    <message>
        <source>Stop and close</source>
        <translation>停止并关闭</translation>
    </message>
    <message>
        <source>Review the script issues above.</source>
        <translation>请处理上方的脚本问题。</translation>
    </message>
    <message>
        <source>Resolve sound cues: %1</source>
        <translation>请设置音效：%1</translation>
    </message>
    <message>
        <source>Choose voices: %1</source>
        <translation>请选择角色声音：%1</translation>
    </message>
    <message>
        <source>Checking script…</source>
        <translation>正在检查脚本…</translation>
    </message>
    <message>
        <source>Write or import a script to begin.</source>
        <translation>编写或导入脚本以开始。</translation>
    </message>
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
        <location filename="../qml/Main.qml" line="194"/>
        <source>New project</source>
        <translation>新建工程</translation>
    </message>
    <message>
        <location filename="../qml/Main.qml" line="87"/>
        <location filename="../qml/Main.qml" line="195"/>
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
        <location filename="../qml/Main.qml" line="114"/>
        <source>The script</source>
        <translation>脚本</translation>
    </message>
    <message>
        <location filename="../qml/Main.qml" line="139"/>
        <source>Import text or script</source>
        <translation>导入文稿或脚本</translation>
    </message>
    <message>
        <location filename="../qml/Main.qml" line="140"/>
        <location filename="../qml/Main.qml" line="213"/>
        <source>Script guide</source>
        <translation>脚本指南</translation>
    </message>
    <message>
        <location filename="../qml/Main.qml" line="118"/>
        <source>Plain narration</source>
        <translation>普通旁白</translation>
    </message>
    <message>
        <location filename="../qml/Main.qml" line="118"/>
        <source>Production script</source>
        <translation>制作脚本</translation>
    </message>
    <message>
        <location filename="../qml/Main.qml" line="128"/>
        <source>Line numbers</source>
        <translation>行号</translation>
    </message>
    <message>
        <location filename="../qml/Main.qml" line="136"/>
        <source>Write</source>
        <translation>编写</translation>
    </message>
    <message>
        <location filename="../qml/Main.qml" line="137"/>
        <source>Read</source>
        <translation>阅读</translation>
    </message>
    <message>
        <location filename="../qml/Main.qml" line="164"/>
        <source>Line %1 · %2</source>
        <translation>第 %1 行 · %2</translation>
    </message>
    <message>
        <location filename="../qml/Main.qml" line="168"/>
        <source>%1 characters</source>
        <translation>%1 个字符</translation>
    </message>
    <message>
        <location filename="../qml/Main.qml" line="170"/>
        <source>%1 / %2 segments</source>
        <translation>%1 / %2 段</translation>
    </message>
    <message>
        <location filename="../qml/Main.qml" line="170"/>
        <source>Preparing…</source>
        <translation>准备中…</translation>
    </message>
    <message>
        <location filename="../qml/Main.qml" line="170"/>
        <source>Ready to render</source>
        <translation>可以生成配音</translation>
    </message>
    <message>
        <location filename="../qml/Main.qml" line="170"/>
        <source>Review voices and script</source>
        <translation>请检查脚本与角色声音</translation>
    </message>
    <message>
        <location filename="../qml/Main.qml" line="171"/>
        <source>Stop after current segment</source>
        <translation>完成当前段落后停止</translation>
    </message>
    <message>
        <location filename="../qml/Main.qml" line="172"/>
        <source>Rendering…</source>
        <translation>生成中…</translation>
    </message>
    <message>
        <location filename="../qml/Main.qml" line="172"/>
        <source>Render voice</source>
        <translation>生成配音</translation>
    </message>
    <message>
        <location filename="../qml/Main.qml" line="190"/>
        <source>Give your words a voice.</source>
        <translation>让文字，拥有声音。</translation>
    </message>
    <message>
        <location filename="../qml/Main.qml" line="191"/>
        <source>A quiet space for narration, dialogue and listening materials.
Write the script. Direct the voices. Keep the take.</source>
        <translation>专注旁白、对话与听力材料的创作空间。
写下脚本，安排声音，留下满意的录音。</translation>
    </message>
    <message>
        <location filename="../qml/Main.qml" line="207"/>
        <source>Your project stays on this device.</source>
        <translation>工程保存在这台设备上。</translation>
    </message>
    <message>
        <location filename="../qml/Main.qml" line="222"/>
        <source>Audio is still being generated</source>
        <translation>音频仍在生成中</translation>
    </message>
    <message>
        <location filename="../qml/Main.qml" line="227"/>
        <source>Stop after the current generation, then close Vox?</source>
        <translation>在当前生成结束后停止，并关闭 Vox？</translation>
    </message>
    <message>
        <location filename="../qml/Main.qml" line="230"/>
        <source>New Vox project</source>
        <translation>新建 Vox 工程</translation>
    </message>
    <message>
        <location filename="../qml/Main.qml" line="230"/>
        <location filename="../qml/Main.qml" line="231"/>
        <source>Vox project (*.vox)</source>
        <translation>Vox 工程 (*.vox)</translation>
    </message>
    <message>
        <location filename="../qml/Main.qml" line="231"/>
        <source>Open Vox project</source>
        <translation>打开 Vox 工程</translation>
    </message>
    <message>
        <location filename="../qml/Main.qml" line="232"/>
        <source>Import script</source>
        <translation>导入脚本</translation>
    </message>
    <message>
        <location filename="../qml/Main.qml" line="232"/>
        <source>Text files (*.txt *.md)</source>
        <translation>文本文件 (*.txt *.md)</translation>
    </message>
    <message>
        <location filename="../qml/Main.qml" line="232"/>
        <source>All files (*)</source>
        <translation>所有文件 (*)</translation>
    </message>
    <message>
        <source>Segments</source>
        <translation>分段</translation>
    </message>
    <message>
        <source>Pronunciation dictionary</source>
        <translation>发音词典</translation>
    </message>
</context>
<context>
    <name>MaterialsDialog</name>
    <message>
        <location filename="../qml/MaterialsDialog.qml" line="8"/>
        <source>Voice &amp; sound library</source>
        <translation>音色与音效库</translation>
    </message>
    <message>
        <location filename="../qml/MaterialsDialog.qml" line="19"/>
        <source>Close</source>
        <translation>关闭</translation>
    </message>
    <message>
        <location filename="../qml/MaterialsDialog.qml" line="34"/>
        <source>Reference voices</source>
        <translation>参考音色</translation>
    </message>
    <message>
        <location filename="../qml/MaterialsDialog.qml" line="36"/>
        <source>Sound effects</source>
        <translation>音效</translation>
    </message>
    <message>
        <location filename="../qml/MaterialsDialog.qml" line="42"/>
        <source>Working locally…</source>
        <translation>正在本机处理…</translation>
    </message>
    <message>
        <location filename="../qml/MaterialsDialog.qml" line="43"/>
        <source>Stop after current generation</source>
        <translation>当前生成结束后停止</translation>
    </message>
    <message>
        <location filename="../qml/MaterialsDialog.qml" line="53"/>
        <source>Your reference voices</source>
        <translation>我的参考音色</translation>
    </message>
    <message>
        <location filename="../qml/MaterialsDialog.qml" line="54"/>
        <source>Add voice</source>
        <translation>添加音色</translation>
    </message>
    <message>
        <location filename="../qml/MaterialsDialog.qml" line="56"/>
        <source>Record or import a voice, then choose it for narration or any role. References stay in this project.</source>
        <translation>录制或导入参考音色后，可用于旁白或任意角色。素材保存在当前项目中。</translation>
    </message>
    <message>
        <location filename="../qml/MaterialsDialog.qml" line="57"/>
        <source>No reference voices yet. Add a recording to begin.</source>
        <translation>还没有参考音色，添加一段录音即可开始。</translation>
    </message>
    <message>
        <location filename="../qml/MaterialsDialog.qml" line="71"/>
        <source>Listen to reference</source>
        <translation>试听参考录音</translation>
    </message>
    <message>
        <location filename="../qml/MaterialsDialog.qml" line="72"/>
        <location filename="../qml/MaterialsDialog.qml" line="148"/>
        <source>Edit a copy in Echo</source>
        <translation>在 Echo 中编辑副本</translation>
    </message>
    <message>
        <location filename="../qml/MaterialsDialog.qml" line="73"/>
        <location filename="../qml/MaterialsDialog.qml" line="149"/>
        <location filename="../qml/MaterialsDialog.qml" line="168"/>
        <source>Export original WAV</source>
        <translation>导出原始 WAV</translation>
    </message>
    <message>
        <location filename="../qml/MaterialsDialog.qml" line="77"/>
        <source>%1 s · local reference</source>
        <translation>%1 秒 · 本机参考音色</translation>
    </message>
    <message>
        <location filename="../qml/MaterialsDialog.qml" line="78"/>
        <source>Synthesize a preview</source>
        <translation>合成试听</translation>
    </message>
    <message>
        <location filename="../qml/MaterialsDialog.qml" line="83"/>
        <source>Hello. Welcome to Vox. Every story begins with a voice.</source>
        <translation>你好，欢迎使用 Vox。每一个故事，从声音开始。</translation>
    </message>
    <message>
        <location filename="../qml/MaterialsDialog.qml" line="83"/>
        <source>Words for the synthesized preview</source>
        <translation>填写合成试听的台词</translation>
    </message>
    <message>
        <location filename="../qml/MaterialsDialog.qml" line="84"/>
        <source>After editing in Echo, import its exported WAV as a new reference and confirm the spoken words.</source>
        <translation>在 Echo 剪辑后，将导出的 WAV 作为新参考音色导入，并确认录音台词。</translation>
    </message>
    <message>
        <location filename="../qml/MaterialsDialog.qml" line="95"/>
        <source>Built-in sounds</source>
        <translation>内置音效</translation>
    </message>
    <message>
        <location filename="../qml/MaterialsDialog.qml" line="106"/>
        <location filename="../qml/MaterialsDialog.qml" line="147"/>
        <source>Preview sound</source>
        <translation>试听音效</translation>
    </message>
    <message>
        <location filename="../qml/MaterialsDialog.qml" line="107"/>
        <source>Add to sound library</source>
        <translation>添加到音效库</translation>
    </message>
    <message>
        <location filename="../qml/MaterialsDialog.qml" line="112"/>
        <source>Generate a sound</source>
        <translation>生成声音</translation>
    </message>
    <message>
        <location filename="../qml/MaterialsDialog.qml" line="113"/>
        <source>Describe a sound in English or Chinese. Listen to the candidate before accepting it.</source>
        <translation>支持中英文描述。生成后先试听，再决定是否采用。</translation>
    </message>
    <message>
        <location filename="../qml/MaterialsDialog.qml" line="114"/>
        <source>Sound name</source>
        <translation>音效名称</translation>
    </message>
    <message>
        <location filename="../qml/MaterialsDialog.qml" line="115"/>
        <source>A soft bell, followed by gentle rain. No music or speech.</source>
        <translation>一声轻柔的铃声，随后是细雨声。不含音乐或人声。</translation>
    </message>
    <message>
        <location filename="../qml/MaterialsDialog.qml" line="117"/>
        <source>Sound effect</source>
        <translation>音效</translation>
    </message>
    <message>
        <location filename="../qml/MaterialsDialog.qml" line="117"/>
        <source>Short music</source>
        <translation>短音乐</translation>
    </message>
    <message>
        <location filename="../qml/MaterialsDialog.qml" line="118"/>
        <source>Seconds</source>
        <translation>秒</translation>
    </message>
    <message>
        <location filename="../qml/MaterialsDialog.qml" line="120"/>
        <source>Seed</source>
        <translation>种子</translation>
    </message>
    <message>
        <location filename="../qml/MaterialsDialog.qml" line="122"/>
        <source>Generation seed</source>
        <translation>生成种子</translation>
    </message>
    <message>
        <location filename="../qml/MaterialsDialog.qml" line="125"/>
        <location filename="../qml/MaterialsDialog.qml" line="167"/>
        <source>Import sound WAV</source>
        <translation>导入音效 WAV</translation>
    </message>
    <message>
        <location filename="../qml/MaterialsDialog.qml" line="127"/>
        <source>Generate candidate</source>
        <translation>生成候选</translation>
    </message>
    <message>
        <location filename="../qml/MaterialsDialog.qml" line="130"/>
        <source>Project sounds</source>
        <translation>项目音效</translation>
    </message>
    <message>
        <location filename="../qml/MaterialsDialog.qml" line="131"/>
        <source>Accepting prepares a script-compatible copy and keeps the original. Choose accepted sounds beside each cue in your script.</source>
        <translation>采用时会生成脚本可用的副本，并保留原始音频。之后可在脚本各提示音旁选择已采用的音效。</translation>
    </message>
    <message>
        <location filename="../qml/MaterialsDialog.qml" line="146"/>
        <source>Accepted</source>
        <translation>已采用</translation>
    </message>
    <message>
        <location filename="../qml/MaterialsDialog.qml" line="146"/>
        <source>Candidate</source>
        <translation>候选</translation>
    </message>
    <message>
        <location filename="../qml/MaterialsDialog.qml" line="154"/>
        <source>%1 s</source>
        <translation>%1 秒</translation>
    </message>
    <message>
        <location filename="../qml/MaterialsDialog.qml" line="155"/>
        <source>Discard candidate</source>
        <translation>丢弃候选</translation>
    </message>
    <message>
        <location filename="../qml/MaterialsDialog.qml" line="156"/>
        <source>Accept for scripts</source>
        <translation>采用为音效</translation>
    </message>
    <message>
        <location filename="../qml/MaterialsDialog.qml" line="161"/>
        <source>For trimming, fades and mixing, open a copy in Echo. Import its WAV export here as a new sound candidate.</source>
        <translation>裁剪、淡入淡出和混音可交给 Echo；将其导出的 WAV 导入此处，即可作为新的音效候选。</translation>
    </message>
    <message>
        <location filename="../qml/MaterialsDialog.qml" line="167"/>
        <source>PCM16 WAV audio (*.wav)</source>
        <translation>PCM16 WAV 音频 (*.wav)</translation>
    </message>
    <message>
        <location filename="../qml/MaterialsDialog.qml" line="168"/>
        <source>WAV audio (*.wav)</source>
        <translation>WAV 音频 (*.wav)</translation>
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
    <message>
        <source>Restore Shape speech draft</source>
        <translation>恢复 Shape 配音草稿</translation>
    </message>
    <message>
        <source>Restore Shape writing draft</source>
        <translation>恢复 Shape 写作草稿</translation>
    </message>
    <message>
        <source>Working draft: Shape speech</source>
        <translation>当前工作稿：Shape 配音草稿</translation>
    </message>
    <message>
        <source>Working draft: Shape writing</source>
        <translation>当前工作稿：Shape 写作草稿</translation>
    </message>
    <message>
        <source>Working draft: your changes</source>
        <translation>当前工作稿：你的修改</translation>
    </message>
    <message>
        <source>Restore changes only the working draft. Accepted audio stays available.</source>
        <translation>恢复仅改变工作稿，已采用的录音仍然保留。</translation>
    </message>
    <message>
        <source>Voice</source>
        <translation>配音</translation>
    </message>
    <message>
        <source>Sound effects</source>
        <translation>音效</translation>
    </message>
    <message>
        <source>Ambience</source>
        <translation>环境声</translation>
    </message>
    <message>
        <source>Short music</source>
        <translation>短配乐</translation>
    </message>
</context>
<context>
    <name>ReadingEntry</name>
    <message>
        <location filename="../qml/ReadingEntry.qml" line="12"/>
        <source>Clear</source>
        <translation>清晰平稳</translation>
    </message>
    <message>
        <location filename="../qml/ReadingEntry.qml" line="12"/>
        <source>Warm</source>
        <translation>温暖亲切</translation>
    </message>
    <message>
        <location filename="../qml/ReadingEntry.qml" line="12"/>
        <source>Lively</source>
        <translation>轻快活泼</translation>
    </message>
    <message>
        <location filename="../qml/ReadingEntry.qml" line="35"/>
        <source>Choose a role</source>
        <translation>请选择角色</translation>
    </message>
    <message>
        <location filename="../qml/ReadingEntry.qml" line="35"/>
        <location filename="../qml/ReadingEntry.qml" line="107"/>
        <source>Needs attention</source>
        <translation>需要检查</translation>
    </message>
    <message>
        <location filename="../qml/ReadingEntry.qml" line="35"/>
        <source>Narration</source>
        <translation>旁白</translation>
    </message>
    <message>
        <location filename="../qml/ReadingEntry.qml" line="49"/>
        <source>Line %1</source>
        <translation>第 %1 行</translation>
    </message>
    <message>
        <location filename="../qml/ReadingEntry.qml" line="53"/>
        <source>Edit line %1</source>
        <translation>编辑第 %1 行</translation>
    </message>
    <message>
        <location filename="../qml/ReadingEntry.qml" line="72"/>
        <source>Source needing review</source>
        <translation>需要检查的原文</translation>
    </message>
    <message>
        <location filename="../qml/ReadingEntry.qml" line="72"/>
        <source>Spoken passage</source>
        <translation>配音台词</translation>
    </message>
    <message>
        <location filename="../qml/ReadingEntry.qml" line="93"/>
        <source>Pause %1 s</source>
        <translation>停顿 %1 秒</translation>
    </message>
    <message>
        <location filename="../qml/ReadingEntry.qml" line="94"/>
        <source>Sound · %1</source>
        <translation>音效 · %1</translation>
    </message>
    <message>
        <location filename="../qml/ReadingEntry.qml" line="95"/>
        <source>Scene · %1</source>
        <translation>片段 · %1</translation>
    </message>
    <message>
        <location filename="../qml/ReadingEntry.qml" line="96"/>
        <source>Note · %1</source>
        <translation>备注 · %1</translation>
    </message>
</context>
<context>
    <name>ReferenceDialog</name>
    <message>
        <location filename="../qml/ReferenceDialog.qml" line="8"/>
        <source>Add reference voice</source>
        <translation>添加参考音色</translation>
    </message>
    <message>
        <location filename="../qml/ReferenceDialog.qml" line="21"/>
        <source>Close</source>
        <translation>关闭</translation>
    </message>
    <message>
        <location filename="../qml/ReferenceDialog.qml" line="38"/>
        <source>A voice from a recording</source>
        <translation>从录音中采样音色</translation>
    </message>
    <message>
        <location filename="../qml/ReferenceDialog.qml" line="39"/>
        <source>Record clear speech or import a mono/stereo PCM16 WAV. Include the exact words spoken; this is a reference for new speech, not model training.</source>
        <translation>录制清晰的语音，或导入单声道、立体声 PCM16 WAV，并填写准确台词。参考录音用于合成新语音，无需训练模型。</translation>
    </message>
    <message>
        <location filename="../qml/ReferenceDialog.qml" line="40"/>
        <source>Voice name</source>
        <translation>音色名称</translation>
    </message>
    <message>
        <location filename="../qml/ReferenceDialog.qml" line="42"/>
        <source>No microphone</source>
        <translation>未发现麦克风</translation>
    </message>
    <message>
        <location filename="../qml/ReferenceDialog.qml" line="43"/>
        <source>Stop recording</source>
        <translation>停止录音</translation>
    </message>
    <message>
        <location filename="../qml/ReferenceDialog.qml" line="43"/>
        <source>Record</source>
        <translation>录音</translation>
    </message>
    <message>
        <location filename="../qml/ReferenceDialog.qml" line="44"/>
        <source>Import WAV</source>
        <translation>导入 WAV</translation>
    </message>
    <message>
        <location filename="../qml/ReferenceDialog.qml" line="48"/>
        <source>Waiting for microphone access…</source>
        <translation>正在等待麦克风授权…</translation>
    </message>
    <message>
        <location filename="../qml/ReferenceDialog.qml" line="48"/>
        <source>%1 s · up to 60 s</source>
        <translation>%1 秒 · 最长 60 秒</translation>
    </message>
    <message>
        <location filename="../qml/ReferenceDialog.qml" line="49"/>
        <source>Microphone level</source>
        <translation>麦克风音量</translation>
    </message>
    <message>
        <location filename="../qml/ReferenceDialog.qml" line="55"/>
        <source>Listen to reference</source>
        <translation>试听参考录音</translation>
    </message>
    <message>
        <location filename="../qml/ReferenceDialog.qml" line="57"/>
        <source>Exact words in the reference</source>
        <translation>参考录音的准确台词</translation>
    </message>
    <message>
        <location filename="../qml/ReferenceDialog.qml" line="60"/>
        <source>Enter every spoken word from the reference, in its original language.</source>
        <translation>按录音原语言填写完整、准确的台词。</translation>
    </message>
    <message>
        <location filename="../qml/ReferenceDialog.qml" line="62"/>
        <source>Use sample text to read aloud</source>
        <translation>使用朗读示例</translation>
    </message>
    <message>
        <location filename="../qml/ReferenceDialog.qml" line="62"/>
        <source>Welcome to Vox. A quiet morning, a familiar voice, and a story waiting to be told.</source>
        <translation>欢迎使用 Vox。一个安静的早晨，一段熟悉的声音，一个等待讲述的故事。</translation>
    </message>
    <message>
        <location filename="../qml/ReferenceDialog.qml" line="63"/>
        <source>I own this voice or have permission to use it for local, disclosed synthetic speech.</source>
        <translation>这是我的声音，或我已获准在本机使用，并标记为合成语音。</translation>
    </message>
    <message>
        <location filename="../qml/ReferenceDialog.qml" line="20"/>
        <source>Saved only in this Vox project.</source>
        <translation>仅保存在此 Vox 项目中。</translation>
    </message>
    <message>
        <location filename="../qml/ReferenceDialog.qml" line="22"/>
        <source>Save reference voice</source>
        <translation>保存参考音色</translation>
    </message>
    <message>
        <location filename="../qml/ReferenceDialog.qml" line="68"/>
        <source>Import reference voice</source>
        <translation>导入参考音色</translation>
    </message>
    <message>
        <location filename="../qml/ReferenceDialog.qml" line="68"/>
        <source>PCM16 WAV audio (*.wav)</source>
        <translation>PCM16 WAV 音频 (*.wav)</translation>
    </message>
</context>
<context>
    <name>ReferenceRecorder</name>
    <message>
        <location filename="../src/reference_recorder.cpp" line="40"/>
        <source>Microphone access is denied. Enable Vox in System Settings, then try again.</source>
        <translation>麦克风访问被拒绝，请在系统设置中为 Vox 开启权限后重试。</translation>
    </message>
    <message>
        <location filename="../src/reference_recorder.cpp" line="54"/>
        <source>Microphone access was not granted.</source>
        <translation>尚未获得麦克风权限。</translation>
    </message>
    <message>
        <location filename="../src/reference_recorder.cpp" line="68"/>
        <source>No microphone is available.</source>
        <translation>没有可用的麦克风。</translation>
    </message>
    <message>
        <location filename="../src/reference_recorder.cpp" line="82"/>
        <source>This microphone format is not supported.</source>
        <translation>暂不支持此麦克风的录音格式。</translation>
    </message>
    <message>
        <location filename="../src/reference_recorder.cpp" line="90"/>
        <source>Could not start the microphone.</source>
        <translation>无法启动麦克风。</translation>
    </message>
    <message>
        <location filename="../src/reference_recorder.cpp" line="100"/>
        <source>Microphone recording stopped unexpectedly.</source>
        <translation>麦克风录音意外停止。</translation>
    </message>
    <message>
        <location filename="../src/reference_recorder.cpp" line="171"/>
        <source>Record at least half a second of clear speech.</source>
        <translation>请至少录制半秒清晰的语音。</translation>
    </message>
    <message>
        <location filename="../src/reference_recorder.cpp" line="178"/>
        <location filename="../src/reference_recorder.cpp" line="195"/>
        <source>Could not save the recording.</source>
        <translation>无法保存录音。</translation>
    </message>
</context>
<context>
    <name>ScriptEditor</name>
    <message>
        <location filename="../qml/ScriptEditor.qml" line="55"/>
        <source>Start with your words.

Write a narration, import an existing script, or choose a form on the left.</source>
        <translation>从你的文字开始。

写一段旁白，导入已有的脚本，或从左侧选择一种形式。</translation>
    </message>
    <message>
        <location filename="../qml/ScriptEditor.qml" line="62"/>
        <source>Script editor</source>
        <translation>脚本编辑器</translation>
    </message>
</context>
<context>
    <name>ScriptReader</name>
    <message>
        <location filename="../qml/ScriptReader.qml" line="12"/>
        <source>Narration</source>
        <translation>旁白</translation>
    </message>
    <message>
        <location filename="../qml/ScriptReader.qml" line="12"/>
        <source>Dialogue</source>
        <translation>对话</translation>
    </message>
    <message>
        <location filename="../qml/ScriptReader.qml" line="12"/>
        <source>Listening</source>
        <translation>听力材料</translation>
    </message>
    <message>
        <location filename="../qml/ScriptReader.qml" line="23"/>
        <source>Passages %1 · Roles %2</source>
        <translation>台词 %1 段 · 角色 %2 位</translation>
    </message>
    <message>
        <location filename="../qml/ScriptReader.qml" line="23"/>
        <source>Passages %1</source>
        <translation>台词 %1 段</translation>
    </message>
    <message>
        <location filename="../qml/ScriptReader.qml" line="25"/>
        <source>Reading preview</source>
        <translation>配音预览</translation>
    </message>
    <message>
        <location filename="../qml/ScriptReader.qml" line="27"/>
        <source>Your spoken passages will appear here as you write.</source>
        <translation>编写脚本后，配音台词会显示在这里。</translation>
    </message>
    <message>
        <location filename="../qml/ScriptReader.qml" line="27"/>
        <source>Updating preview…</source>
        <translation>正在更新预览…</translation>
    </message>
    <message>
        <location filename="../qml/ScriptReader.qml" line="56"/>
        <source>Repeat %1×</source>
        <translation>重复 %1×</translation>
    </message>
    <message>
        <location filename="../qml/ScriptReader.qml" line="57"/>
        <source>Same recording</source>
        <translation>重播同一录音</translation>
    </message>
    <message>
        <location filename="../qml/ScriptReader.qml" line="59"/>
        <source>Edit line %1</source>
        <translation>编辑第 %1 行</translation>
    </message>
    <message>
        <location filename="../qml/ScriptReader.qml" line="65"/>
        <source>%1 s between repeats</source>
        <translation>每次间隔 %1 秒</translation>
    </message>
    <message>
        <location filename="../qml/ScriptReader.qml" line="65"/>
        <source>Sound · %1</source>
        <translation>音效 · %1</translation>
    </message>
    <message>
        <location filename="../qml/ScriptReader.qml" line="80"/>
        <source>Needs attention</source>
        <translation>需要检查</translation>
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
        <location filename="../qml/SettingsDialog.qml" line="21"/>
        <source>Appearance</source>
        <translation>外观</translation>
    </message>
    <message>
        <location filename="../qml/SettingsDialog.qml" line="23"/>
        <source>Theme</source>
        <translation>主题</translation>
    </message>
    <message>
        <location filename="../qml/SettingsDialog.qml" line="24"/>
        <location filename="../qml/SettingsDialog.qml" line="28"/>
        <source>Follow system</source>
        <translation>跟随系统</translation>
    </message>
    <message>
        <location filename="../qml/SettingsDialog.qml" line="24"/>
        <source>Light</source>
        <translation>亮色</translation>
    </message>
    <message>
        <location filename="../qml/SettingsDialog.qml" line="24"/>
        <source>Dark</source>
        <translation>暗色</translation>
    </message>
    <message>
        <location filename="../qml/SettingsDialog.qml" line="27"/>
        <source>Language</source>
        <translation>界面语言</translation>
    </message>
    <message>
        <location filename="../qml/SettingsDialog.qml" line="31"/>
        <source>Script editor</source>
        <translation>脚本编辑区</translation>
    </message>
    <message>
        <location filename="../qml/SettingsDialog.qml" line="32"/>
        <source>Default font for scripts only. Interface text and exported audio are unaffected.</source>
        <translation>默认字体仅用于脚本编辑区，不影响界面文字和导出音频。</translation>
    </message>
    <message>
        <location filename="../qml/SettingsDialog.qml" line="40"/>
        <source>Script font</source>
        <translation>脚本字体</translation>
    </message>
    <message>
        <location filename="../qml/SettingsDialog.qml" line="42"/>
        <source>Script font size</source>
        <translation>脚本字号</translation>
    </message>
    <message>
        <location filename="../qml/SettingsDialog.qml" line="50"/>
        <source>Show line numbers</source>
        <translation>显示行号</translation>
    </message>
    <message>
        <location filename="../qml/SettingsDialog.qml" line="51"/>
        <source>Reset font</source>
        <translation>恢复默认字体</translation>
    </message>
    <message>
        <location filename="../qml/SettingsDialog.qml" line="54"/>
        <source>Voice synthesis</source>
        <translation>语音合成</translation>
    </message>
    <message>
        <location filename="../qml/SettingsDialog.qml" line="55"/>
        <source>Infer Runtime is connected</source>
        <translation>Infer Runtime 已连接</translation>
    </message>
    <message>
        <location filename="../qml/SettingsDialog.qml" line="55"/>
        <source>Infer Runtime needs a connection</source>
        <translation>需要连接 Infer Runtime</translation>
    </message>
    <message>
        <location filename="../qml/SettingsDialog.qml" line="56"/>
        <source>Vox uses its own local authorization. Import the credential issued for Vox in Infer Console.</source>
        <translation>Vox 使用独立的本地授权。请导入 Infer Console 为 Vox 签发的凭据。</translation>
    </message>
    <message>
        <location filename="../qml/SettingsDialog.qml" line="58"/>
        <source>Import credential</source>
        <translation>导入凭据</translation>
    </message>
    <message>
        <location filename="../qml/SettingsDialog.qml" line="59"/>
        <source>Check connection</source>
        <translation>检查连接</translation>
    </message>
    <message>
        <location filename="../qml/SettingsDialog.qml" line="62"/>
        <source>Edit in Echo</source>
        <translation>在 Echo 中编辑</translation>
    </message>
    <message>
        <location filename="../qml/SettingsDialog.qml" line="63"/>
        <source>Open accepted audio in Echo’s independent editor. Return its WAV export to review a new take.</source>
        <translation>将已采用的音频送到 Echo 独立编辑。导出 WAV 后返回这里，作为新的候选版本审核。</translation>
    </message>
    <message>
        <location filename="../qml/SettingsDialog.qml" line="66"/>
        <location filename="../qml/SettingsDialog.qml" line="71"/>
        <source>Choose Echo executable</source>
        <translation>选择 Echo 可执行文件</translation>
    </message>
    <message>
        <location filename="../qml/SettingsDialog.qml" line="70"/>
        <source>Choose Vox credential</source>
        <translation>选择 Vox 凭据</translation>
    </message>
</context>
<context>
    <name>VoicePanel</name>
    <message>
        <location filename="../qml/VoicePanel.qml" line="37"/>
        <source>Audio / open library…</source>
        <translation>音频 / 打开音效库…</translation>
    </message>
    <message>
        <location filename="../qml/VoicePanel.qml" line="56"/>
        <source>VOICES &amp; DIRECTION</source>
        <translation>声音与演绎</translation>
    </message>
    <message>
        <location filename="../qml/VoicePanel.qml" line="57"/>
        <source>Voice library</source>
        <translation>音色库</translation>
    </message>
    <message>
        <location filename="../qml/VoicePanel.qml" line="58"/>
        <source>Sounds</source>
        <translation>音效库</translation>
    </message>
    <message>
        <location filename="../qml/VoicePanel.qml" line="63"/>
        <source>Default voice</source>
        <translation>默认声音</translation>
    </message>
    <message>
        <location filename="../qml/VoicePanel.qml" line="76"/>
        <source>Audition this voice</source>
        <translation>试听这个声音</translation>
    </message>
    <message>
        <location filename="../qml/VoicePanel.qml" line="82"/>
        <source>Language</source>
        <translation>配音语言</translation>
    </message>
    <message>
        <location filename="../qml/VoicePanel.qml" line="95"/>
        <source>Pace</source>
        <translation>语速</translation>
    </message>
    <message>
        <location filename="../qml/VoicePanel.qml" line="105"/>
        <source>Speech pace</source>
        <translation>配音语速</translation>
    </message>
    <message>
        <location filename="../qml/VoicePanel.qml" line="107"/>
        <source>Reference voices use 1× pace and neutral delivery.</source>
        <translation>参考音色支持 1× 语速与自然平稳的演绎。</translation>
    </message>
    <message>
        <location filename="../qml/VoicePanel.qml" line="114"/>
        <source>Roles</source>
        <translation>角色</translation>
    </message>
    <message>
        <location filename="../qml/VoicePanel.qml" line="116"/>
        <source>Declare roles in your script, then choose a voice for each one.</source>
        <translation>在脚本中声明角色，再为每个角色选择声音。</translation>
    </message>
    <message>
        <location filename="../qml/VoicePanel.qml" line="139"/>
        <source>Choose a voice</source>
        <translation>选择声音</translation>
    </message>
    <message>
        <location filename="../qml/VoicePanel.qml" line="143"/>
        <source>Audition role voice</source>
        <translation>试听角色声音</translation>
    </message>
    <message>
        <location filename="../qml/VoicePanel.qml" line="151"/>
        <source>Sound cues</source>
        <translation>音效提示</translation>
    </message>
    <message>
        <location filename="../qml/VoicePanel.qml" line="173"/>
        <source>Open sound library</source>
        <translation>打开音效库</translation>
    </message>
    <message>
        <location filename="../qml/VoicePanel.qml" line="37"/>
        <source>From script</source>
        <translation>使用脚本声明</translation>
    </message>
    <message>
        <location filename="../qml/VoicePanel.qml" line="15"/>
        <source>Reference · %1</source>
        <translation>参考 · %1</translation>
    </message>
    <message>
        <location filename="../qml/VoicePanel.qml" line="33"/>
        <source>Hello. Welcome to Vox. Every story begins with a voice.</source>
        <translation>你好，欢迎使用 Vox。每一个故事，从声音开始。</translation>
    </message>
    <message>
        <location filename="../qml/VoicePanel.qml" line="37"/>
        <source>Chime</source>
        <translation>提示音</translation>
    </message>
    <message>
        <location filename="../qml/VoicePanel.qml" line="37"/>
        <source>Skip</source>
        <translation>跳过</translation>
    </message>
    <message>
        <location filename="../qml/VoicePanel.qml" line="177"/>
        <source>Generated speech is labeled as synthetic. Your accepted takes stay available when you revise the script.</source>
        <translation>生成的配音会标为合成音频。修改脚本后，已采用的录音版本仍会保留。</translation>
    </message>
    <message>
        <location filename="../qml/VoicePanel.qml" line="181"/>
        <source>Import sound cue</source>
        <translation>导入音效</translation>
    </message>
    <message>
        <location filename="../qml/VoicePanel.qml" line="181"/>
        <source>WAV audio (*.wav)</source>
        <translation>WAV 音频 (*.wav)</translation>
    </message>
</context>
<context>
    <name>PronunciationDialog</name>
    <message>
        <source>Pronunciation dictionary</source>
        <translation>发音词典</translation>
    </message>
    <message>
        <source>Saving…</source>
        <translation>保存中…</translation>
    </message>
    <message>
        <source>Save rules to refresh the preview.</source>
        <translation>保存规则后更新预览。</translation>
    </message>
    <message>
        <source>Close</source>
        <translation>关闭</translation>
    </message>
    <message>
        <source>Save rules</source>
        <translation>保存规则</translation>
    </message>
    <message>
        <source>Rules change the text sent to the voice engine. Your authored script stays unchanged. Match an exact word or phrase, with optional language and role filters.</source>
        <translation>规则只改变送入语音引擎的文本，脚本原文保持不变。匹配指定文字，并可限定语言和角色。</translation>
    </message>
    <message>
        <source>Written text</source>
        <translation>原文</translation>
    </message>
    <message>
        <source>Read as</source>
        <translation>读作</translation>
    </message>
    <message>
        <source>All languages</source>
        <translation>所有语言</translation>
    </message>
    <message>
        <source>All roles (or enter a role)</source>
        <translation>所有角色（或输入角色名）</translation>
    </message>
    <message>
        <source>Add rule</source>
        <translation>添加规则</translation>
    </message>
    <message>
        <source>No pronunciation rules yet.</source>
        <translation>尚未添加发音规则。</translation>
    </message>
    <message>
        <source>All roles</source>
        <translation>所有角色</translation>
    </message>
    <message>
        <source>Remove rule</source>
        <translation>移除规则</translation>
    </message>
    <message>
        <source>Effective spoken text</source>
        <translation>实际朗读文本</translation>
    </message>
    <message>
        <source>Checking script…</source>
        <translation>检查脚本中…</translation>
    </message>
    <message>
        <source>Line %1 · %2</source>
        <translation>第 %1 行 · %2</translation>
    </message>
    <message>
        <source>Narration</source>
        <translation>旁白</translation>
    </message>
    <message>
        <source>Use a shorter written phrase.</source>
        <translation>请缩短要匹配的原文。</translation>
    </message>
    <message>
        <source>Use a shorter pronunciation.</source>
        <translation>请缩短读法文本。</translation>
    </message>
    <message>
        <source>Use a shorter role name.</source>
        <translation>请缩短角色名称。</translation>
    </message>
    <message>
        <source>A rule already exists for this text and filters. Remove it before adding a replacement.</source>
        <translation>已有相同文字和筛选条件的规则，请先移除旧规则再替换。</translation>
    </message>
</context>
<context>
    <name>SegmentsPanel</name>
    <message>
        <source>Speech segments</source>
        <translation>配音分段</translation>
    </message>
    <message>
        <source>Pronunciation</source>
        <translation>发音词典</translation>
    </message>
    <message>
        <source>Regenerate a passage, compare its alternatives, then accept the one to use. Assemble accepted segments into a take for final review.</source>
        <translation>单独生成或重录一段，比较候选并采用。随后将选定的分段拼装为录音，进行最终审阅。</translation>
    </message>
    <message>
        <source>Checking script…</source>
        <translation>检查脚本中…</translation>
    </message>
    <message>
        <source>All segments accepted</source>
        <translation>所有分段已采用</translation>
    </message>
    <message>
        <source>%1 segments · accept one alternative for each passage</source>
        <translation>共 %1 段 · 请为每段采用一个候选</translation>
    </message>
    <message>
        <source>Write spoken text to create segments.</source>
        <translation>输入配音文本后显示分段。</translation>
    </message>
    <message>
        <source>Narration</source>
        <translation>旁白</translation>
    </message>
    <message>
        <source>Accepted</source>
        <translation>已采用</translation>
    </message>
    <message>
        <source>Needs a take</source>
        <translation>待生成录音</translation>
    </message>
    <message>
        <source>Edit line %1</source>
        <translation>编辑第 %1 行</translation>
    </message>
    <message>
        <source>Line %1</source>
        <translation>第 %1 行</translation>
    </message>
    <message>
        <source>Source passage</source>
        <translation>原文段落</translation>
    </message>
    <message>
        <source>Spoken: %1</source>
        <translation>读作：%1</translation>
    </message>
    <message>
        <source>Take %1 · %2 s%3</source>
        <translation>录音 %1 · %2 秒%3</translation>
    </message>
    <message>
        <source>Listen to this alternative</source>
        <translation>试听此候选</translation>
    </message>
    <message>
        <source>Accept</source>
        <translation>采用</translation>
    </message>
    <message>
        <source>Regenerate</source>
        <translation>重新生成</translation>
    </message>
    <message>
        <source>Generate</source>
        <translation>生成</translation>
    </message>
    <message>
        <source>This alternative belongs to an earlier script. Regenerate to accept a take for the current source.</source>
        <translation>此候选属于较早的脚本版本，请重新生成后用于当前版本。</translation>
    </message>
    <message>
        <source>Earlier accepted takes stay available.</source>
        <translation>此前采用的录音仍然保留。</translation>
    </message>
    <message>
        <source>Subtitles</source>
        <translation>字幕</translation>
    </message>
    <message>
        <source>Assemble take</source>
        <translation>拼装录音</translation>
    </message>
    <message>
        <source>In use</source>
        <translation>正在使用</translation>
    </message>
</context>
<context>
    <name>SoundWorkspace</name>
    <message>
        <source>Ambience</source>
        <translation>环境声</translation>
    </message>
    <message>
        <source>Short music</source>
        <translation>短配乐</translation>
    </message>
    <message>
        <source>Sound effects</source>
        <translation>音效</translation>
    </message>
    <message>
        <source>Built-in</source>
        <translation>内置</translation>
    </message>
    <message>
        <source>Imported</source>
        <translation>已导入</translation>
    </message>
    <message>
        <source>Shape import</source>
        <translation>来自 Shape</translation>
    </message>
    <message>
        <source>Import WAV</source>
        <translation>导入 WAV</translation>
    </message>
    <message>
        <source>1–30 seconds</source>
        <translation>1–30 秒</translation>
    </message>
    <message>
        <source>Local generation</source>
        <translation>本地生成</translation>
    </message>
    <message>
        <source>Short instrumental music. No vocals or singing are requested; audition each result.</source>
        <translation>生成短器乐，提示要求不含人声或歌唱；请逐个试听确认。</translation>
    </message>
    <message>
        <source>Describe an environmental bed. Loop audition helps you hear how it repeats.</source>
        <translation>描述一段环境氛围。循环试听可帮助判断重复播放的效果。</translation>
    </message>
    <message>
        <source>Describe a single effect or a short sequence in English or Chinese.</source>
        <translation>用中文或英文描述单个音效或一小段声音序列。</translation>
    </message>
    <message>
        <source>Sound name</source>
        <translation>声音名称</translation>
    </message>
    <message>
        <source>Sparse piano and soft strings, a calm instrumental ending.</source>
        <translation>稀疏的钢琴与柔和弦乐，一段平静的器乐结尾。</translation>
    </message>
    <message>
        <source>Steady rain in a quiet forest, distant leaves moving, no speech or music.</source>
        <translation>安静森林中的持续雨声，远处树叶轻轻摇动，不要说话或音乐。</translation>
    </message>
    <message>
        <source>A soft bell, then two quiet wooden knocks. No speech or music.</source>
        <translation>轻柔的铃声，接着两次轻轻的木头敲击。不要说话或音乐。</translation>
    </message>
    <message>
        <source>Seconds</source>
        <translation>秒数</translation>
    </message>
    <message>
        <source>Seed</source>
        <translation>种子</translation>
    </message>
    <message>
        <source>Generation seed</source>
        <translation>生成种子</translation>
    </message>
    <message>
        <source>Variations</source>
        <translation>变体数</translation>
    </message>
    <message>
        <source>Generate %1 candidates</source>
        <translation>生成 %1 个候选</translation>
    </message>
    <message>
        <source>Generate candidate</source>
        <translation>生成候选</translation>
    </message>
    <message>
        <source>Completed variations are kept as candidates when you stop.</source>
        <translation>停止后，已完成的变体仍保留为候选。</translation>
    </message>
    <message>
        <source>Stop after current generation</source>
        <translation>当前生成完成后停止</translation>
    </message>
    <message>
        <source>Project sounds</source>
        <translation>项目声音</translation>
    </message>
    <message>
        <source>All project sounds</source>
        <translation>所有项目声音</translation>
    </message>
    <message>
        <source>Loop audition</source>
        <translation>循环试听</translation>
    </message>
    <message>
        <source>Repeats playback; it does not create a seamless loop.</source>
        <translation>重复播放，不会生成无缝循环。</translation>
    </message>
    <message>
        <source>Preview sound</source>
        <translation>试听声音</translation>
    </message>
    <message>
        <source>Accepted</source>
        <translation>已采用</translation>
    </message>
    <message>
        <source>Candidate</source>
        <translation>候选</translation>
    </message>
    <message>
        <source>Sound details</source>
        <translation>声音详情</translation>
    </message>
    <message>
        <source>Edit a copy in Echo</source>
        <translation>在 Echo 中编辑副本</translation>
    </message>
    <message>
        <source>Export original WAV</source>
        <translation>导出原始 WAV</translation>
    </message>
    <message>
        <source>Discard candidate</source>
        <translation>丢弃候选</translation>
    </message>
    <message>
        <source>Accept</source>
        <translation>采用</translation>
    </message>
    <message>
        <source>%1 s</source>
        <translation>%1 秒</translation>
    </message>
    <message>
        <source>Variation %1 of %2 · seed %3</source>
        <translation>变体 %1/%2 · 种子 %3</translation>
    </message>
    <message>
        <source>Seed %1</source>
        <translation>种子 %1</translation>
    </message>
    <message>
        <source>Generation prompt: %1</source>
        <translation>生成提示词：%1</translation>
    </message>
    <message>
        <source>Original WAV is retained. Accepting also prepares a copy for script cues.</source>
        <translation>保留原始 WAV。采用时还会准备用于脚本音效提示的副本。</translation>
    </message>
    <message>
        <source>Generate or import sounds, then listen and accept the candidates you want to keep.</source>
        <translation>生成或导入声音，试听后采用希望保留的候选。</translation>
    </message>
    <message>
        <source>Built-in sounds</source>
        <translation>内置声音</translation>
    </message>
    <message>
        <source>Preview built-in sound</source>
        <translation>试听内置声音</translation>
    </message>
    <message>
        <source>Add to sound library</source>
        <translation>加入声音库</translation>
    </message>
    <message>
        <source>Export keeps the original quality. Use Echo for trimming, fades and mixing; import its WAV as a new candidate.</source>
        <translation>导出保留原始音质。可在 Echo 中裁剪、淡入淡出或混音，再将其 WAV 导入为新候选。</translation>
    </message>
    <message>
        <source>Import sound WAV</source>
        <translation>导入声音 WAV</translation>
    </message>
    <message>
        <source>PCM16 WAV audio (*.wav)</source>
        <translation>PCM16 WAV 音频 (*.wav)</translation>
    </message>
    <message>
        <source>WAV audio (*.wav)</source>
        <translation>WAV 音频 (*.wav)</translation>
    </message>
</context>
<context>
    <name>SubtitleDialog</name>
    <message>
        <source>Export subtitles</source>
        <translation>导出字幕</translation>
    </message>
    <message>
        <source>Subtitle timings come from the measured audio of this assembled take. Text follows the authored script.</source>
        <translation>字幕时间来自此拼装录音的实际分段时长，字幕内容采用脚本原文。</translation>
    </message>
    <message>
        <source>SubRip (.srt)</source>
        <translation>SubRip (.srt)</translation>
    </message>
    <message>
        <source>WebVTT (.vtt)</source>
        <translation>WebVTT (.vtt)</translation>
    </message>
    <message>
        <source>Cancel</source>
        <translation>取消</translation>
    </message>
    <message>
        <source>Choose file…</source>
        <translation>选择文件…</translation>
    </message>
    <message>
        <source>WebVTT subtitles (*.vtt)</source>
        <translation>WebVTT 字幕 (*.vtt)</translation>
    </message>
    <message>
        <source>SubRip subtitles (*.srt)</source>
        <translation>SubRip 字幕 (*.srt)</translation>
    </message>
</context>
</TS>
