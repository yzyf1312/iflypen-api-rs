# TODO

### 错误处理优化

- [ ] 引入 `thiserror` 定义自定义错误类型
  - 在 `src/error.rs` 创建 `IflyrecError` 枚举
  - 替换所有 `Box<dyn Error>` 返回类型
  - 添加 API 错误码结构化处理逻辑

- [ ] 重构 API 响应错误处理逻辑
  - 在 `src/api.rs` 添加错误码匹配处理
  - 将通用错误码转换为自定义错误类型
  - 在 HTTP 请求方法中统一错误映射
  

### 内存与性能优化
- [ ] 实现流式文件上传
  - 在 `src/api.rs` 修改 `upload_audio_file` 方法
  - 使用 `tokio::fs::File` 和 `ReaderStream`
  - 替换 `fs::read` 的内存加载方式
  - 移除 `final_data.extend_from_slice` 的内存拼接

- [ ] 优化音频时长计算
  - 在 `src/audio.rs` 创建新模块
  - 使用 `hound` 库解析 WAV 文件头信息
  - 添加对其他音频格式的检测支持

### 代码结构优化
- [ ] 提取 HTTP 请求构建逻辑
  - 在 `src/api/client.rs` 添加 `build_request` 方法
  - 统一处理 Headers 和超时配置
  - 替换所有 HTTP 请求中的重复代码

- [ ] 拆分模块结构
  - 创建 `src/api/mod.rs`
  - 将 `TranscriptionOptions` 移到 `src/api/options.rs`
  - 将 `AudioMetadata` 移到 `src/api/metadata.rs`

### 安全性增强
- [ ] 使用 SecretString 存储敏感信息
  - 修改 `IflyrecClient` 的 `session_id` 字段类型
  - 在 `Cargo.toml` 添加 `secrecy` 依赖
  - 更新相关方法的参数处理逻辑

### 重试策略优化
- [ ] 实现可配置的重试策略
  - 在 `src/retry.rs` 定义配置结构体
  - 修改 `initiate_transcription_task` 的重试逻辑
  - 允许外部传入最大重试次数和延迟配置

### 日志与可观测性
- [ ] 添加 tracing 日志记录
  - 在 `Cargo.toml` 添加 `tracing` 依赖
  - 在关键方法添加 info!/error! 日志记录
  - 在 `src/lib.rs` 初始化日志框架

### 测试覆盖
- [ ] 编写单元测试
  - 在 `src/api/metadata.rs` 添加测试模块
  - 测试 `calculate_wav_duration` 的边界条件
  - 测试 `AudioMetadata` 的序列化正确性

- [ ] 创建集成测试
  - 在 `tests/integration.rs` 添加端到端测试
  - 使用 `wiremock` 模拟 HTTP 服务
  - 验证 API 请求的正确性

## 建议

- **文件结构**：将核心功能拆分为模块存放在 `src/api/` 子目录，主模块声明在 `src/lib.rs`
- **配置文件**：将重试策略等配置参数放在 `src/config.rs` 并通过 `IflyrecClient::builder()` 配置
- **测试文件**：单元测试与代码同目录，集成测试放在项目根目录的 `tests/` 目录