# ims_mml（Rust / MML）

基于 Rust 的 MML 文本反序列化/序列化库，包含：

- `mml_def`：运行时解析、类型定义、Trait 与示例
- `mml_def_derive`：`proc-macro` 派生宏（`MmlMessage` / `MmlBranch` / `MmlValueEnum`）

## 目录结构

```text
mylib/
├─ Cargo.toml                # workspace
├─ mml_def/                  # 运行时 crate
└─ mml_def_derive/           # 派生宏 crate
```

## 核心能力

- MML 行解析与组装：`OP OBJECT:K=V,...;`
- 参数名大小写不敏感匹配（内部统一为大写键）
- `derive` 方式定义命令结构体
- 支持 `#[mml(rename = "...")]` 与 `#[mml(skip)]`
- 支持“分支参数组”建模（如 `DID=ESL/V5ST/...`）：
  - 用 `enum + #[derive(MmlBranch)]` 表示分支
- 支持枚举值解析：
  - 用 `#[derive(MmlValueEnum)]` 将枚举映射为 MML 字面量

## 快速开始

```rust
use mml_def::{MmlBranch, MmlMessage, MmlValueEnum};

#[derive(Debug, Clone, MmlValueEnum)]
enum Regtp {
    Single,
    #[mml(rename = "UNKNOW")]
    Unknown,
}

#[derive(Debug, Clone, MmlBranch)]
enum AsbrDid {
    Esl { eid: String, tid: u32 },
    V5st { v5iid: u32, l3addr: u32 },
}

#[derive(Debug, Clone, MmlMessage)]
#[mml(op = "ADD", object = "ASBR")]
struct AddAsbr {
    pui: String,
    pri: String,
    regtp: Regtp,
    did: AsbrDid,
    #[mml(skip)]
    parsed_at: Option<String>,
}
```

## 常用命令

```powershell
cargo check --workspace
cargo test -p mml_def
cargo test -p mml_def_derive
cargo run -p mml_def --example asbr_serde
```

## 示例运行相关环境变量

部分示例依赖本地文件或数据库，使用环境变量避免硬编码：

- `mml_def/examples/save_db.rs`
  - `MML_INPUT_FILE`（必填）
  - `MML_DB_PASSWORD`（必填）
  - `MML_DB_HOST`（可选，默认 `localhost`）
  - `MML_DB_NAME`（可选，默认 `imsdb`）
  - `MML_DB_USER`（可选，默认 `postgres`）
- `mml_def/examples/eid.rs`
  - `MML_LOG_DIR`（可选，默认 `logfile`）
  - `MML_EIDS_FILE`（可选，默认 `${MML_LOG_DIR}/eids.txt`）
- `mml_def/examples/sipusers.rs`
  - `MML_SIPUSERS_FILE`（可选，默认 `logfile/agcf/AGCF65_20240506.txt`）
- `mml_def/examples/t001.rs`
  - `MML_T001_LOG`（可选，默认 `logfile/lst_ifcs.txt`）
