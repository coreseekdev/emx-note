# CapsaEngine 重构设计文档

## 概述

将当前分散式的命令重构为基于 `CapsaEngine` 的面向对象设计。

## 当前架构问题

### 分散式模式
- 每个命令都重复 `resolve_capsa()` → 文件操作
- 配置通过 `ResolveContext` 传递，但没有统一的操作入口
- 命令代码中有大量重复逻辑

### 重复代码示例
```rust
// note.rs, daily.rs, tag.rs 都有类似模式
let capsa_ref = super::resolve::resolve_capsa(ctx, caps)?;
let note_dir = capsa_ref.path.join("note");
fs::create_dir_all(&note_dir)?;
// ... 重复的逻辑
```

## 新架构设计

### 核心结构

```
CapsaEngine (核心)
  ├─ 默认方法：Note 操作
  ├─ tags() → Tags (Tag 集合)
  │   └─ get(name) → Tag (单个 Tag)
  └─ task_file() → TaskFile (任务文件)
```

### 类型定义

#### 1. CapsaEngine
```rust
pub struct CapsaEngine {
    inner: CapsaRef,
}

impl CapsaEngine {
    // 构造
    pub fn new(ref: CapsaRef) -> Self;

    // === Note 操作（默认、核心）===
    pub fn create_permanent_note(
        &self,
        title: Option<&str>,
        source: Option<&str>,
        content: &str,
    ) -> io::Result<PathBuf>;

    pub fn create_daily_note(
        &self,
        title: Option<&str>,
        content: &str,
    ) -> io::Result<PathBuf>;

    pub fn resolve_note(
        &self,
        note_ref: &str,
        force: bool,
    ) -> io::Result<Vec<PathBuf>>;

    // === 获取 Tags 集合 ===
    pub fn tags(&self) -> Tags;

    // === 获取 Task 文件 ===
    pub fn task_file(&self) -> TaskFile;
}

// Deref 到 CapsaRef
impl Deref for CapsaEngine {
    type Target = CapsaRef;
}
```

#### 2. Tags (Tag 集合)
```rust
pub struct Tags<'a> {
    capsa: &'a CapsaRef,
}

impl<'a> Tags<'a> {
    /// 获取特定 tag 对象
    pub fn get(&self, tag: &str) -> Tag<'_>;

    /// 列出所有 tags
    pub fn list(&self) -> io::Result<Vec<String>>;
}
```

#### 3. Tag (单个 Tag)
```rust
pub struct Tag<'a> {
    capsa: &'a CapsaRef,
    name: String,
}

impl<'a> Tag<'a> {
    /// Tag 文件路径
    pub fn file(&self) -> PathBuf;

    /// 添加笔记到这个 tag
    pub fn add_note(&self, note_path: &PathBuf) -> io::Result<()>;

    /// 从这个 tag 移除笔记
    pub fn remove_note(&self, note_relative: &str) -> io::Result<()>;

    /// 列出这个 tag 下的所有笔记
    pub fn list_notes(&self) -> io::Result<Vec<PathBuf>>;

    /// 删除整个 tag 文件
    pub fn delete(self) -> io::Result<()>;
}
```

#### 4. TaskFile (任务文件)
```rust
pub struct TaskFile<'a> {
    capsa: &'a CapsaRef,
}

impl<'a> TaskFile<'a> {
    /// Task 文件路径
    pub fn file(&self) -> PathBuf;

    /// 加载所有任务
    pub fn load(&self) -> io::Result<Vec<Task>>;

    /// 保存任务
    pub fn save(&self, tasks: &[Task]) -> io::Result<()>;

    /// 添加新任务
    pub fn add(&self, summary: &str) -> io::Result<Task>;

    /// 列出所有任务
    pub fn list(&self) -> io::Result<Vec<Task>>;

    /// 查找任务
    pub fn find(&self, note_ref: &str) -> io::Result<Vec<Task>>;
}
```

## 使用示例

### Note 操作
```rust
let capsa = CapsaEngine::new(resolve_capsa(ctx, caps)?);
let note = capsa.create_permanent_note(
    Some("My Note"),
    None,
    "Content here"
)?;
```

### Tag 操作
```rust
let capsa = CapsaEngine::new(resolve_capsa(ctx, caps)?);

// 添加到多个 tags
capsa.tags().get("work").add_note(&note)?;
capsa.tags().get("important").add_note(&note)?;

// 列出 tag 下的笔记
let notes = capsa.tags().get("work").list_notes()?;
```

### Task 操作
```rust
let capsa = CapsaEngine::new(resolve_capsa(ctx, caps)?);

// 添加任务
capsa.task_file().add("Review note")?;

// 列出任务
for task in capsa.task_file().list()? {
    println!("{}", task.summary);
}
```

## 重构映射

### 命令 → 新方法映射

| 命令 | 当前实现 | 新实现 |
|------|---------|--------|
| `note` | `note.rs` | `CapsaEngine::create_permanent_note()` |
| `daily` | `daily.rs` | `CapsaEngine::create_daily_note()` |
| `tag add` | `tag.rs::add_tags()` | `Tag::add_note()` |
| `tag remove` | `tag.rs::remove_tags()` | `Tag::remove_note()` |
| `task add` | `task/add.rs` | `TaskFile::add()` |
| `task list` | `task/list.rs` | `TaskFile::list()` |
| `task show` | `task/show.rs` | `TaskFile::load()` + 过滤 |
| `task take` | `task/take.rs` | `TaskFile` + 修改操作 |
| `task release` | `task/release.rs` | `TaskFile` + 修改操作 |
| `task comment` | `task/comment.rs` | `TaskFile` + 修改操作 |

## 文件组织

### 新增文件
- `src/engine.rs` - CapsaEngine 及相关类型

### 修改文件
- `src/lib.rs` - 导出 `pub use engine::*`
- `src/cmd/note.rs` - 重构使用 `CapsaEngine`
- `src/cmd/daily.rs` - 重构使用 `CapsaEngine`
- `src/cmd/tag.rs` - 重构使用 `Tag`
- `src/cmd/task/*.rs` - 重构使用 `TaskFile`

### 保留文件
- `src/resolve.rs` - `ResolveContext` 和 `CapsaRef` 保持不变
- `src/note_resolver.rs` - Note 解析逻辑保持不变
- `src/edit.rs` - `EditOp` 保持不变

## 测试覆盖

### 需要检查的 E2E 测试
- `tests/note.txtar` - note 命令
- `tests/daily.txtar` - daily 命令
- `tests/tag*.txtar` - tag 命令
- `tests/task*.txtar` - task 命令

### 测试要求
- 重构前后行为必须一致
- 所有 E2E 测试必须通过
- 如果有未覆盖的行为，先添加测试

## 实施步骤

1. ✅ 创建设计文档
2. ⏳ 检查并完善 E2E 测试覆盖
3. ⏳ 实现 `CapsaEngine` 基础结构
4. ⏳ 实现 Note 操作方法
5. ⏳ 重构 `note.rs` 命令
6. ⏳ 重构 `daily.rs` 命令
7. ⏳ 实现 `Tags` 和 `Tag`
8. ⏳ 重构 `tag.rs` 命令
9. ⏳ 实现 `TaskFile`
10. ⏳ 重构 `task/*.rs` 命令
11. ⏳ 运行所有测试
12. ⏳ 清理未使用的代码

## 注意事项

### Testspec Heredoc 行为
- 当前 heredoc 会在最后追加 `\n`
- 编写测试时需要考虑这个行为
- 参考现有测试的写法

### 向后兼容
- `ResolveContext` 和 `CapsaRef` 保持公开
- 命令接口不变
- 只改变内部实现

### 性能
- `CapsaEngine` 只是轻量包装
- `Tags`/`Tag`/`TaskFile` 只是引用包装
- 零成本抽象

## 设计原则

1. **Note 是核心** - `CapsaEngine` 默认提供 Note 操作
2. **按需构造** - Tag/Task 通过方法按需获取
3. **语义清晰** - `capsa.tags().get("work")` 比传递参数更清晰
4. **状态复用** - `Tag`/`TaskFile` 对象可以多次使用
5. **零成本** - 所有类型只是引用的包装
