# EMX-Note 路径使用规则

本文档详细说明了 emx-note 中笔记（Note）路径的解析、命名和组织规则。

## 目录结构

```
~/.emx-notes/                    # Capsa 根目录
├── @shared/                    # 全局共享命名空间（无 agent 时）
│   ├── .                      # 默认 capsa
│   ├── work/                  # 工作 capsa
│   └── #research.md           # 标签文件
├── agent1/                    # Agent 私有命名空间
│   ├── .                      # agent1 的默认 capsa
│   ├── projects/              # agent1 的 projects capsa
│   └── #tasks.md              # 标签文件
└── agent2/
    ├── .
    └── personal/

每个 capsa 内部:
├── note/                      # 永久笔记
│   ├── timestamp-title.md    # 带时间戳的笔记
│   ├── title.md              # 不带时间戳的笔记
│   └── {hash}/               # 带 source 的笔记
│       ├── timestamp-title.md
│       └── .source           # source 跟踪文件
├── #daily/                   # 每日笔记
│   ├── YYYYMMDD/             # 日期目录
│   │   ├── HHmmSS-title.md   # 带标题的每日笔记
│   │   └── HHmmSS.md         # 不带标题的每日笔记
│   └── #daily.md             # 每日笔记索引
├── #tag1.md                  # 标签文件
├── #tag2.md
└── TASK.md                   # 任务文件
```

## Note 路径解析规则

### 解析优先级（从高到低）

#### Rule 0: 日期/前缀格式 `YYYYMMDD/prefix`

直接在指定日期的每日笔记目录中查找。

**格式**: `YYYYMMDD/prefix`

**示例**:
```bash
# 查找 2024-01-15 的每日笔记中，以 "meet" 开头的笔记
20240115/meet

# 查找 2024-01-15 的每日笔记中，时间戳为 143000 的笔记
20240115/143000
```

**解析逻辑**:
1. 验证日期部分是否为有效的 8 位数字（YYYYMMDD）
2. 在 `#daily/YYYYMMDD/` 目录中查找前缀匹配的文件
3. 支持时间前缀和标题前缀混合匹配

---

#### Rule 1: 完整时间戳 `YYYYMMDDHHmmSS`

**格式**: 14 位连续数字

**示例**:
```bash
# 完整时间戳
20240115143022

# 解析为: 日期 2024-01-15, 时间 14:30:22
# 查找路径: #daily/20240115/143022*.md
```

**解析逻辑**:
1. 验证是否为 14 位数字
2. 验证日期部分（前 8 位）有效性（1900-2100年，月份1-12，日期1-31）
3. 验证时间部分（后 6 位）有效性（小时00-23，分钟00-59，秒00-59）
4. 首先在 `#daily/YYYYMMDD/` 目录查找
5. 如果未找到，也在 `note/` 目录查找（支持带时间戳的永久笔记）

---

#### Rule 2: 时间前缀 `HHmmSS`、`HHmm`、`HH`

**格式**: 6 位或更少的时间数字前缀

**示例**:
```bash
# 完整时间（秒）
143022          # → #daily/20240115/143022*.md

# 分钟精度
1430            # → #daily/20240115/1430*.md

# 小时精度
14              # → #daily/20240115/14*.md
```

**解析逻辑**:
1. 提取开头的数字字符（最多 6 位）
2. 使用当前日期
3. 在 `#daily/{当前日期}/` 目录中查找匹配时间前缀的文件
4. 支持时间+标题混合格式（如 `143000-meeting`）

---

#### Rule 3: 标题 slug

**格式**: URL 友好的标题字符串

**示例**:
```bash
# 标题 slug
meeting-notes    # → 先查找 #daily/{today}/meeting-notes*.md
                  #    再查找 note/meeting-notes*.md
                  #    最后搜索 #*.md 索引文件

# 自动 slugify
"Hello World!"   # → hello-world
"Test@Note#123"   # → test-note-123
```

**解析逻辑**:
1. **3a.** 在 `#daily/{当前日期}/` 目录中前缀匹配
2. **3b.** 在 `note/` 目录中前缀匹配
3. **3c.** 搜索根目录的 `#*.md` 索引文件（标签文件）

**Slugify 规则**:
- 转换为小写
- 字母数字保留
- 特殊字符转换为 `-`
- 连续 `-` 压缩为单个 `-`
- 去除首尾 `-`

---

## 文件命名约定

### 永久笔记 (Permanent Notes)

**位置**: `note/`

**命名格式**:
```
{title}.{ext}           # 不带时间戳
{timestamp}.{ext}       # 纯时间戳
{timestamp}-{title}.{ext}  # 时间戳+标题
```

**示例**:
```
note/project-ideas.md                      # 标题
note/20240115143022.md                      # 纯时间戳
note/20240115143022-research-notes.md       # 时间戳+标题
```

### 每日笔记 (Daily Notes)

**位置**: `#daily/YYYYMMDD/`

**命名格式**:
```
HHmmSS.{ext}            # 不带标题
HHmmSS-{title}.{ext}    # 带标题
```

**示例**:
```
#daily/20240115/143000.md                  # 纯时间
#daily/20240115/143000-meeting.md          # 带标题
#daily/20240115/1430-quick-note.md         # 分钟精度
```

### 带 Source 的笔记

**位置**: `note/{hash}/`

**结构**:
```
note/{hash}/
├── {timestamp}-{title}.md
└── .source                # 包含原始 source 字符串
```

**Hash 生成**:
- 使用 SHA256 算法
- 取前 12 位作为 hash
- Source 可以是 URL、文件路径等任意字符串

**示例**:
```bash
# 创建笔记时指定 source
emx-note note --source "https://example.com/article" --title "Article Summary"

# 生成的路径
note/a1b2c3d4e5f6/20240115143022-article-summary.md
note/a1b2c3d4e5f6/.source  # 内容: https://example.com/article
```

---

## 索引文件 (Index Files)

### 标签文件

**位置**: Capsa 根目录

**命名**: `#{tag-name}.md`

**格式**:
```markdown
# {tag-name}

## YYYY-MM-DD

- [Note Title](relative/path/to/note.md)
- [Another Note](note/another-note.md)

## YYYY-MM-DD

- [Today Note](#daily/20240115/143000-todo.md)
```

**规则**:
- 每个标签文件记录一个主题的笔记
- 按日期分组
- 使用 Markdown 链接格式
- 自动去重（同一笔记只添加一次）

### 每日笔记索引

**位置**: `note/#daily.md`

**格式**:
```markdown
# Daily Notes

- [Meeting Notes](#daily/20240115/143000-meeting.md)
- [Quick Thought](#daily/20240115/143145.md)
```

---

## 支持的文件格式

**扩展名** (优先级顺序):
1. `.md` - Markdown
2. `.mx` - emx-note 扩展
3. `.emx` - emx-note 扩展

**默认扩展名**: `.md`

---

## 环境变量

### 测试支持

#### `EMX_TASK_TIMESTAMP`

覆盖当前时间戳（用于测试和可重现的脚本）

**格式**: `YYYY-MM-DD HH:MM`

**示例**:
```bash
# 设置固定时间
export EMX_TASK_TIMESTAMP="2024-01-15 14:30"

# 创建笔记将使用此时间
emx-note daily --title "Test Note"
# → 创建 #daily/20240115/143000-test-note.md
```

#### `EMX_AGENT_NAME`

设置 agent 名称（用于多 agent 协作）

**示例**:
```bash
export EMX_AGENT_NAME="researcher"
# → 使用 researcher/ 命名空间
```

#### `EMX_TASKFILE`

覆盖任务文件名

**默认**: `TASK.md`

**示例**:
```bash
export EMX_TASKFILE="TASKS.md"
# → 使用 TASKS.md 而非 TASK.md
```

---

## 路径解析示例

### 示例 1: 查找每日笔记

```bash
# 当前日期: 2024-01-15

# 查找今天的时间戳笔记
143000
# → #daily/20240115/143000.md

# 查找今天的特定笔记
meeting
# → #daily/20240115/*meeting*.md (前缀匹配)

# 完整时间戳
20240115143000
# → #daily/20240115/143000.md
```

### 示例 2: 查找永久笔记

```bash
# 按标题查找
project-ideas
# → note/project-ideas.md

# 查找带时间戳的永久笔记
20240115143000
# → note/20240115143000.md
# → 或 #daily/20240115/143000.md
```

### 示例 3: 指定日期查找

```bash
# 在特定日期的每日笔记中查找
20240114/meeting
# → #daily/20240114/*meeting*.md

20240114/1430
# → #daily/20240114/1430*.md
```

### 示例 4: 通过标签查找

```bash
# 标签文件索引了跨目录的笔记
# #research.md 包含:
# - [Paper Review](note/20240115-paper-review.md)
# - [Idea](#daily/20240115/143000-idea.md)

# 查找时自动搜索索引文件
research-paper
# → 在 #research.md 中查找
# → 返回 note/20240115-paper-review.md
```

---

## 模糊匹配规则

### 前缀匹配 (Prefix Matching)

**规则**:
- 输入 `meet` 可以匹配 `meeting.md`、`meet-up.md`
- 不区分大小写（先转换为 slug）
- 支持部分匹配

**时间+标题混合**:
- 输入 `1430-meet` 可以匹配 `143000-meeting.md`
- 先匹配时间戳前缀，再匹配标题前缀

### 歧义处理 (Ambiguity)

当找到多个匹配文件时:
1. 列出所有候选文件
2. 显示相对路径
3. 提示使用 `--force` 操作所有匹配项

**示例**:
```bash
$ emx-note tag add research 1430
Error: Ambiguous note reference '1430'
Found 3 matching notes:
  1. #daily/20240115/143000-idea.md
  2. #daily/20240115/143145.md
  3. #daily/20240114/143022-discussion.md

Use --force add to all matching notes.
```

---

## 特殊字符处理

### 路径分隔符

**规则**:
- 输入中的反斜杠 `\` 自动转换为正斜杠 `/`
- 跨平台兼容（Windows、Linux、macOS）

**示例**:
```bash
# Windows 风格输入
note\subdir\file
# → 规范化为: note/subdir/file
```

### 空格处理

**规则**:
- 自动去除首尾空格
- 标题中的空格转换为 `-`

**示例**:
```bash
# 输入
"  Meeting Notes  "

# Slugify 后
"meeting-notes"
```

---

## 实际使用场景

### 场景 1: 快速创建今日笔记

```bash
# 不带标题
emx-note daily
# → #daily/20240115/143145.md

# 带标题
emx-note daily --title "Standup"
# → #daily/20240115/143145-standup.md
```

### 场景 2: 创建带来源的笔记

```bash
# 从 URL
emx-note note --source "https://blog.example.com/post" --title "Blog Summary"
# → note/a1b2c3d4e5f6/20240115143022-blog-summary.md

# .source 文件包含原始 URL
```

### 场景 3: 查找并编辑笔记

```bash
# 查找并编辑
emx-note edit meeting
# → 查找 meeting 前缀的笔记
# → 打开编辑器

# 查找特定日期的笔记
emx-note edit 20240115/1430
# → 在 2024-01-15 的每日笔记中查找 1430 前缀
```

### 场景 4: 标签管理

```bash
# 添加到标签
emx-note tag add research 20240115/143000-paper

# 列出标签内容
emx-note tag list research
# → 显示 #research.md 中所有笔记

# 移除标签
emx-note tag remove research 20240115/143000-paper
```

---

## 最佳实践

### 1. 使用描述性标题

```bash
# 好的做法
emx-note daily --title "Design Review - API Authentication"

# 避免
emx-note daily --title "Notes"
```

### 2. 合理使用时间戳

```bash
# 精确时间（用于多条笔记）
143000    # 14:30:00
143030    # 14:30:30

# 粗略时间（用于快速笔记）
143       # 14:30xx 的任意笔记
```

### 3. Source 跟踪

```bash
# 记录来源便于追溯
emx-note note \
  --source "https://arxiv.org/abs/2401.12345" \
  --title "Paper Summary: LLM Architecture"
```

### 4. 标签组织

```bash
# 按主题组织
emx-note tag add research    # 研究笔记
emx-note tag add ideas       # 想法收集
emx-note tag add todo        # 待办事项
```

---

## 故障排除

### Note 未找到

```bash
$ emx-note edit meeting
Error: Note 'meeting' not found
```

**可能原因**:
1. 笔记不存在
2. 前缀不匹配
3. 日期范围错误

**解决方法**:
1. 使用 `emx-note list` 查看所有笔记
2. 使用更精确的前缀
3. 使用完整时间戳或日期/前缀格式

### 歧义引用

```bash
$ emx-note tag add research 14
Error: Ambiguous note reference '14'
Found 2 matching notes:
  1. #daily/20240115/140000-coffee.md
  2. #daily/20240115/143000-meeting.md

Use --force add to all matching notes.
```

**解决方法**:
```bash
# 使用更精确的前缀
emx-note tag add research 1400

# 或使用 --force
emx-note tag add --force research 14
```

---

## 版本历史

- **v1.0** - 初始版本，支持基本的时间戳和标题解析
- **v1.1** - 添加 source 跟踪和 hash 目录组织
- **v1.2** - 添加日期/前缀格式（Rule 0）
- **v1.3** - 改进 slugify 和前缀匹配算法

---

## 相关文档

- [CAPSA_ORGANIZATION.md](./CAPSA_ORGANIZATION.md) - Capsa 组织和命名空间
- [TASK_SYSTEM.md](./TASK_SYSTEM.md) - 任务系统使用指南
- [CLI_REFERENCE.md](./CLI_REFERENCE.md) - 命令行参考
