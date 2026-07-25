//! Route 核心逻辑库
//!
//! 提供配置文件解析、安全写入、备份回滚、线路管理等核心功能。
//! 本库不依赖 Tauri 框架，可独立编译和测试。

pub mod backup;
pub mod config_paths;
pub mod parsers;
pub mod routes;
pub mod writer;
