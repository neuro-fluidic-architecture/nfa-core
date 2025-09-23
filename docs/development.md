# NFA Core 开发指南

本文档提供 NFA Core 项目的开发环境设置、代码结构和开发流程指南。

## 开发环境设置

###  prerequisites

- Rust 1.70+ (使用 [rustup](https://rustup.rs/))
- Go 1.21+
- Protocol Buffers 编译器 (protoc)
- Docker (可选，用于容器化部署)

### 快速开始

1. 克隆项目：
   ```bash
   git clone https://github.com/neuro-fluidic-architecture/nfa-core.git
   cd nfa-core