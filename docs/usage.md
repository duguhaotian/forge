# Forge 使用文档

## 先决条件

- Rust 工具链
- `ssh`：用于密钥、证书和大部分 SSH 连接场景
- `sshpass`：仅当使用密码认证时需要

## 安装与构建

```bash
cargo build
cargo run -- --help
```

如果你已经安装到本机，也可以直接运行 `forge`。

## 初始化

首次使用先初始化本地目录：

```bash
forge init
```

初始化后会生成：

```text
~/.forge/
├── config.yaml
└── providers/
    └── ssh/
        └── nodes.yaml
```

## 查看 Provider

```bash
forge provider list
```

当前可见的 Provider 是 `ssh`。

## 节点管理

### 添加 SSH 节点

密码认证：

```bash
forge node add ssh dev-password \
  --host 1.2.3.4 \
  --user ubuntu \
  --auth password \
  --password-ref env:FORGE_DEV_PASSWORD
```

也可以直接给明文密码：

```bash
forge node add ssh dev-password \
  --host 1.2.3.4 \
  --user ubuntu \
  --auth password \
  --password 'secret'
```

密钥认证：

```bash
forge node add ssh dev-key \
  --host 1.2.3.5 \
  --user ubuntu \
  --auth key-pair \
  --key ~/.ssh/id_ed25519
```

证书认证：

```bash
forge node add ssh dev-cert \
  --host 1.2.3.6 \
  --user ubuntu \
  --auth certificate \
  --key ~/.ssh/id_ed25519 \
  --cert ~/.ssh/id_ed25519-cert.pub
```

如果节点名已存在，默认会失败；加 `--replace` 才会覆盖。

### 列出节点

```bash
forge node list ssh
```

也可以直接写：

```bash
forge node list
```

输出包含节点名、Provider、主机、用户、端口和标签。

### 查看节点

```bash
forge node show ssh dev-key
```

这会输出单个节点的 YAML；明文 `password` 会被脱敏。

### 删除节点

```bash
forge node remove ssh dev-key
```

### 测试连通性

```bash
forge node ping ssh dev-key
```

### 探测节点信息

```bash
forge node inspect ssh dev-key
```

会尝试采集主机名、系统、架构、CPU、内存、磁盘和 uptime。

### 执行远端命令

```bash
forge node exec ssh dev-key -- uname -a
```

`--` 后面的内容会按参数原样转交给远端 shell。

## 批量导入

支持 YAML 和 JSON 文件：

```bash
forge node import ssh ./nodes.yaml --dry-run
forge node import ssh ./nodes.yaml
forge node import ssh ./nodes.yaml --replace
```

### YAML 示例

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

### JSON 示例

```json
{
  "nodes": [
    {
      "name": "dev-key",
      "host": "1.2.3.5",
      "port": 22,
      "user": "ubuntu",
      "auth": {
        "type": "key_pair",
        "key_path": "~/.ssh/id_ed25519"
      }
    }
  ]
}
```

## 认证值约定

`password_ref` 支持以下写法：

- `env:NAME`
- `prompt`
- `plain:VALUE`
- 其他字符串直接按字面量使用

如果使用密码认证，请确保系统里安装了 `sshpass`。

## 配置位置

- 节点库：`~/.forge/providers/ssh/nodes.yaml`
- 全局配置：`~/.forge/config.yaml`

## 当前限制

- `lease` 和 `workspace` 目前只是命令入口，占位未实现
- 一些 `SshAuth` 数据模式已预留，但 CLI 尚未完全提供对应创建选项

如果你想把 Forge 接到真实工作流里，下一步通常是先把 SSH 节点整理成规范的导入文件，再用 `inspect` 和 `exec` 验证连通性。
