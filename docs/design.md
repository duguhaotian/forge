# Forge 设计文档

## 目标

Forge 是一个面向 AI Code Agent 的计算资源控制 CLI。它把“可用计算资源”抽象成可发现、可租用、可执行、可扩展的 Provider 资源，而不是把重点放在传统意义上的 workload 部署。

当前仓库的实现重点是 SSH Provider：通过本地 `forge` CLI 管理 SSH 节点配置，并把节点能力暴露给后续的 workspace / lease 流程。

## 总体架构

仓库按四层组织：

- `src/cli/`：命令行入口与子命令分发
- `src/core/`：跨 Provider 共享的数据模型与 trait
- `src/providers/ssh/`：SSH Provider 的配置、存储、客户端与探测实现
- `src/output/`：表格输出等展示逻辑

### 运行路径

1. `main.rs` 进入 `cli::run()`
2. `clap` 解析命令与参数
3. CLI 层把请求路由到对应 Provider
4. Provider 从本地存储读取节点配置
5. `SshClient` 组装系统 `ssh` / `sshpass` 命令并执行

## 核心数据模型

### `NodeSummary`

用于列表展示的节点摘要，包含：

- `name`
- `provider`
- `host`
- `user`
- `port`
- `labels`

### `NodeInspection`

用于 `inspect` 的节点能力信息，当前字段包括：

- `hostname`
- `os`
- `arch`
- `cpu_cores`
- `memory_total_mb`
- `disk_total_mb`
- `uptime`

### SSH 节点模型

`SshNode` 是当前实际持久化的数据结构，包含：

- 节点基础信息：`name`、`host`、`port`、`user`
- 认证信息：`auth`
- 标签：`labels`

`SshAuth` 目前在数据层支持多种模式：

- `password`
- `key_pair`
- `certificate`
- `agent`
- `keyboard_interactive`
- `gssapi`
- `hostbased`
- `none`
- `openssh_config`

其中 CLI 当前主要暴露的是 `password`、`key_pair`、`certificate` 三种创建方式；其余模式已在模型中预留，但尚未完全接入命令行创建流程。

## 存储设计

SSH Provider 使用用户目录下的固定路径存储节点配置：

- 配置根目录：`~/.forge/`
- 全局配置：`~/.forge/config.yaml`
- SSH 节点库：`~/.forge/providers/ssh/nodes.yaml`

`init` 命令会在缺失时创建这些文件。

节点库使用 YAML 持久化，也支持从 JSON / YAML 文件导入。

## 命令设计

### `forge init`

初始化本地目录结构与最小配置。

### `forge provider list`

当前返回已实现 Provider 名称。现在可见的是 `ssh`。

### `forge node`

节点相关命令都挂在 `node` 下，并按 Provider 再分一层：

- `node add ssh`
- `node import ssh`
- `node list [ssh]`
- `node show ssh`
- `node remove ssh`
- `node ping ssh`
- `node inspect ssh`
- `node exec ssh`

### 占位命令

`lease` 和 `workspace` 目前只保留入口，后续用于资源租用和工作区生命周期管理。

## SSH Provider 行为

### 添加与导入

- `add`：新增单个节点，重复名称默认失败，可通过 `--replace` 覆盖
- `import`：批量导入节点文件，支持 `--dry-run` 和 `--replace`
- 导入时会先校验节点名称唯一性与必填字段

### 列表与详情

- `list`：打印表格视图
- `show`：输出单个节点的 YAML
- `show` 会对直接保存的明文 `password` 做脱敏，避免把敏感信息原样打印出来

### 可达性与探测

- `ping`：执行远端 `true` 验证连通性
- `inspect`：执行一段 shell 脚本采集主机信息

### 命令执行

- `exec`：把本地参数拼接后交给远端 shell 执行
- 返回码直接透传给本地进程，便于脚本化调用

## 认证与外部依赖

### Password

Password 模式依赖系统 `sshpass`，并通过 `sshpass -e` 注入密码。

`password_ref` 支持的值语义如下：

- `env:NAME`：从环境变量读取
- `prompt`：从标准输入读取
- `plain:VALUE`：把 `VALUE` 当作明文
- 其他值：按字面量处理

### Key / Certificate

- `key_pair`：依赖系统 `ssh`
- `certificate`：在 `ssh` 基础上附加 `CertificateFile`

### 未完成的模式

`agent`、`keyboard_interactive`、`gssapi`、`hostbased`、`none`、`openssh_config` 已建模，但目前只有部分路径真正接入了执行逻辑。

## 扩展点

后续新增 Provider 时，通常需要补齐四类能力：

1. Provider 配置与节点存储
2. `Provider` trait 的实现
3. CLI 子命令绑定
4. 输出与验证逻辑

这种结构可以让 SSH、VM、裸机或云 Provider 共享统一的命令入口，同时保留各自独立的生命周期与存储格式。
