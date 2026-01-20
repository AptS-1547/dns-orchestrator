//! 业务领域模型
//!
//! 这些数据结构与 UI 完全无关，后期可以直接对接 dns-orchestrator-core

mod account;
mod dns_record;
mod domain;

pub use account::{Account, AccountStatus, ProviderType};
pub use dns_record::{DnsRecord, DnsRecordType, RecordData};
pub use domain::{Domain, DomainStatus};