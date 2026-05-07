# Forge 是一个面向 AI Code Agent 的计算资源控制 CLI。

它统一管理本地物理机、VM 与远程主机，并以 workspace、lease 与 capability 为核心抽象，为 Agent 提供稳定、可编程的计算环境。

Forge 不关心“部署 workload”，而是专注于：

* 发现计算资源
* 分配与回收资源
* 创建 Agent Workspace
* 执行编译 / 测试 / 验证任务
* 通过 Provider 插件扩展不同基础设施

Forge 采用 SSH-first 与 provider-based 架构，初期支持：

* 本地物理机
* libvirt/qemu VM
* SSH Host

后续可扩展：

* AWS / GCP / Azure
* Bare Metal
* GPU Fleet
* Kubernetes / Nomad
* Snapshot / Image / Sandbox

Forge 的目标是：

“让 AI Agent 像调用函数一样使用计算资源。”

更详细的设计说明见 `docs/design.md`，使用说明见 `docs/usage.md`。

## 当前实现范围

当前版本提供 Rust 实现的 `forge` CLI 框架，并优先实现 SSH Provider 的节点资源管理能力。`lease`、`workspace` 与其他 Provider 暂时保留命令入口，后续继续扩展。

## 安装与构建

```bash
cargo build
cargo run -- --help
```

密码认证依赖系统命令 `sshpass`，证书与密钥认证依赖系统 `ssh`。使用 `password_ref: prompt` 时会从标准输入读取密码。

## 初始化

```bash
forge init
```

默认会创建：

```text
~/.forge/
├── config.yaml
└── providers/
    └── ssh/
        └── nodes.yaml
```

## SSH 节点管理

添加密码认证节点：

```bash
forge node add ssh dev-password \
  --host 1.2.3.4 \
  --user ubuntu \
  --auth password \
  --password-ref env:FORGE_DEV_PASSWORD
```

也允许明文密码：

```bash
forge node add ssh dev-password \
  --host 1.2.3.4 \
  --user ubuntu \
  --auth password \
  --password 'secret'
```

添加普通 SSH 密钥节点：

```bash
forge node add ssh dev-key \
  --host 1.2.3.5 \
  --user ubuntu \
  --auth key-pair \
  --key ~/.ssh/id_ed25519
```

添加 OpenSSH certificate 节点：

```bash
forge node add ssh dev-cert \
  --host 1.2.3.6 \
  --user ubuntu \
  --auth certificate \
  --key ~/.ssh/id_ed25519 \
  --cert ~/.ssh/id_ed25519-cert.pub
```

常用操作：

```bash
forge node list ssh
forge node show ssh dev-key
forge node ping ssh dev-key
forge node inspect ssh dev-key
forge node exec ssh dev-key -- uname -a
forge node remove ssh dev-key
```

## 批量导入 SSH 节点

支持 YAML 与 JSON：

```bash
forge node import ssh ./nodes.yaml --dry-run
forge node import ssh ./nodes.yaml
forge node import ssh ./nodes.yaml --replace
```

YAML 示例：

```yaml
nodes:
  - name: dev-password
    host: 1.2.3.4
    port: 22
    user: ubuntu
    auth:
      type: password
      password_ref: env:FORGE_DEV_PASSWORD
    labels:
      env: dev

  - name: dev-key
    host: 1.2.3.5
    user: ubuntu
    auth:
      type: key_pair
      key_path: ~/.ssh/id_ed25519

  - name: dev-cert
    host: 1.2.3.6
    user: ubuntu
    auth:
      type: certificate
      key_path: ~/.ssh/id_ed25519
      cert_path: ~/.ssh/id_ed25519-cert.pub
```

SSH Provider 的节点配置由 Provider 自己管理，后续其他 Provider 可以实现独立的配置来源和存储格式。
