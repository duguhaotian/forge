# Forge

Forge 是一个面向 AI Code Agent 的计算资源控制 CLI。它统一管理本地物理机、VM 与远程主机，并把计算资源抽象成可发现、可探测、可分配、可执行、可扩展的 Provider 资源。

它的目标是：让 AI Agent 像调用函数一样使用计算资源。

## 文档

- `docs/design.md`：架构、数据模型、Provider 边界与扩展点
- `docs/usage.md`：安装、SSH 节点管理、导入格式与命令示例
- `README.md`：英文入口页

## 当前实现范围

当前 Rust 版本重点实现 SSH Provider 的节点资源管理能力：

- 初始化本地 Forge 状态
- 添加、导入、列出、查看和删除 SSH 节点
- 测试节点连通性
- 探测主机能力信息
- 通过 SSH 执行远端命令

`lease` 和 `workspace` 目前只保留命令入口，完整生命周期逻辑后续继续扩展。

## 安装

```bash
cargo build
cargo run -- --help
```

如果希望把当前仓库安装成本机 `forge` 命令：

```bash
cargo install --path .
forge --help
```

## 快速开始

```bash
forge init
forge provider list
forge node list ssh
```

添加一个 SSH 密钥节点：

```bash
forge node add ssh dev-key \
  --host 1.2.3.5 \
  --user ubuntu \
  --auth key-pair \
  --key ~/.ssh/id_ed25519
```

验证并探测节点：

```bash
forge node ping ssh dev-key
forge node inspect ssh dev-key
forge node exec ssh dev-key -- uname -a
```

## 命令概览

- `forge init`：创建 `~/.forge/` 和默认 Provider 文件
- `forge provider list`：查看可用 Provider
- `forge node add ssh`：添加单个 SSH 节点
- `forge node import ssh`：从 YAML 或 JSON 批量导入 SSH 节点
- `forge node list [ssh]`：列出已配置节点
- `forge node show ssh <name>`：以 YAML 输出节点详情，并脱敏敏感信息
- `forge node remove ssh <name>`：删除一个节点
- `forge node ping ssh <name>`：测试 SSH 连通性
- `forge node inspect ssh <name>`：采集主机名、系统、CPU、内存、磁盘和 uptime
- `forge node exec ssh <name> -- <command>`：执行远端命令并透传退出码

## 存储位置

Forge 的本地状态默认保存在 `~/.forge/` 下：

```text
~/.forge/
├── config.yaml
└── providers/
    └── ssh/
        └── nodes.yaml
```

SSH Provider 自己管理节点配置，后续其他 Provider 可以实现独立的配置来源和存储格式。

## SSH 认证

密码认证依赖系统命令 `sshpass`；密钥和证书认证依赖系统 `ssh`。

`password_ref` 支持以下写法：

- `env:NAME`
- `prompt`
- `plain:VALUE`
- 其他字符串会按字面量使用
