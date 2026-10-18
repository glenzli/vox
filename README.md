# Vox

[English](README.en.md) · 简体中文

**脚本驱动的配音制作。**

Vox 是独立的 Qt 桌面应用，用于旁白、多角色对话和听力材料制作。从脚本编写、音色选择、
本地合成到试听、采用与导出，保留每一版脚本和已采用的录音。当前为开发预览，尚未发布正式版本；
最初验证的平台为 macOS Apple Silicon。

## 制作流程

1. 新建 `.vox` 项目，编写或导入 UTF-8 文本，选择旁白、对话或听力模板。
2. 为默认旁白和各角色选择预设音色或参考音色，设置配音语言、语速与音效提示。
3. 生成候选录音，试听、比较后明确采用，再导出 WAV。修改脚本后，旧录音仍然保留。
4. 需要裁剪、淡入淡出或混音时，在 [Echo](https://github.com/glenzli/echo) 中编辑副本，
   将导出的 WAV 返回 Vox，作为新候选试听和采用。

## 功能

| 功能 | 当前支持 |
| --- | --- |
| 脚本编辑 | 语法高亮、逻辑行号、自动换行、编写与阅读视图；脚本区字体和字号可独立设置、预览与重置 |
| 多角色配音 | 复用 Shape 的脚本语法，支持角色、演绎、场景、停顿、音效与重复；重复片段重放同一份 PCM，不重复调用模型 |
| 预设音色 | 九种预设音色，可自动识别或明确指定配音语言 |
| 音色模仿 | 现场录音或导入单声道/立体声 PCM16 WAV，填写逐字参考文本并确认使用权限；可分配给旁白或任意角色 |
| 音效库 | 八种程序生成的内置音效、WAV 导入，以及本地 Stable Audio 3 的音效和短音乐生成 |
| 版本与素材 | 独立项目、自动保存的工作草稿、不可变素材、明确采用的脚本与录音版本 |
| Echo 联动 | 对录音、参考音色和音效建立独立编辑副本，保留原始素材，手动导入编辑后的 WAV |
| 界面 | 紧凑的图标工具栏，黑白灰主色，中英文界面，亮色、暗色和跟随系统主题 |

参考音色的原始音频、逐字文本和授权确认保存在当前项目中；只有点击录音才申请麦克风权限。
当前 Base 音色模仿使用 1× 语速和中性演绎。自动化检查使用合成参考和注入的音频样本，
真实麦克风采集仍需设备和首次系统授权。

生成音效支持 1–30 秒、显式随机种子和中英文描述。中文描述经过本地提示词准备，
原文、实际提示词和 Runtime 回执一并保留。试听并采用后，Vox 准备用于脚本的音频副本，
同时保留原始 WAV 供导出或 Echo 编辑。

Echo 返回的音频先进入候选列表；24 位到 16 位 PCM 的转换需要明确选择。
两款应用通过文件交接，各自管理项目和数据库。

## 本地运行

需要 Rust 1.96+、CMake 3.28+、Ninja，以及包含 Quick、Quick Controls、Multimedia、SVG 和
Linguist Tools 的 Qt 6.9+。构建脚本使用离线 Cargo 依赖缓存，首次运行前需准备锁定的依赖。

```sh
./scripts/build_and_promote_debug.sh
./scripts/run_debug.sh
./scripts/run_debug.sh /absolute/path/MyProject.vox
```

构建产物和当前 Debug 应用默认放在仓库旁的 `.vox-local-build`。可使用 `VOX_LOCAL_BUILD_ROOT`、
`VOX_BUILD_DIR` 和 `VOX_CARGO_TARGET_DIR` 指定其他仓库外路径。应用包包含 `vox-cli`；
开发机器仍需安装 Qt。可分发打包、签名公证与正式发布另行进行。

### 连接 Infer Runtime

Vox 使用自己的 `vox` 应用授权，合成限定为本地、零费用且不回退的执行路径。
在 Infer Console 签发独立凭据后，可从 Vox 设置导入。操作辅助脚本见：

```sh
python3 scripts/provision_infer.py --help
```

默认凭据位置为 `~/.config/vox/infer-credential`；`--studio` 额外申请 Vox 的本地参考音色、
音效生成和中文提示词准备权限。授权变更需在 Console 队列空闲时进行。Vox 不使用 Shape 凭据；
Runtime 离线时，脚本编辑和已保存音频仍可使用。

## 开发与文档

| 目录 | 职责 |
| --- | --- |
| `crates/vox-core/src/domain` | 音频约定与兼容 Shape 的脚本语法 |
| `crates/vox-core/src/assembly.rs` | 有界的脚本编译、PCM 组装与校验 |
| `crates/vox-core/src/infer_runtime` | 官方 SDK、Vox 身份、本地合成与进度 |
| `crates/vox-core/src/project` | 独立项目、草稿、不可变素材与采用流程 |
| `crates/vox-core/src/handoff.rs` | Echo 编辑副本、显式返回与 PCM24 转换 |
| `apps/vox-cli` | 桌面应用、用户与自动化共用的 JSON 操作 |
| `apps/desktop` | Qt/QML 编辑、演绎配置、试听与审阅界面 |

参见[架构](docs/ARCHITECTURE.md)、[自动化接口](docs/AUTOMATION.md)、
[脚本语法](docs/SPEECH_SCRIPT.md)、[验证记录](docs/VALIDATION.md)、[路线图](ROADMAP.md)
和[图标来源](assets/branding/README.md)。

迁移复用了 Shape 的语音制作代码，保留其来源和许可；Shape 原有项目与配音界面不受影响。
目前可导入导出的文本和脚本，尚不自动转换整个 `.shape` 项目。分段重录、通过描述设计音色、
视频配音等形态列入后续规划。

## 许可证

Vox 采用 [MIT License](LICENSE)。迁移代码和依赖的来源及许可见
[第三方声明](THIRD_PARTY_NOTICES.md)与 [Shape 来源清单](docs/SHAPE_PROVENANCE.json)。
模型权重不随项目分发，其许可由相应提供方规定。
