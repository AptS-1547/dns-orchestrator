//! 页面状态模块
//!
//! 定义各个页面的状态数据结构

mod accounts;
mod dns_records;
mod domains;
pub mod favorites;
mod modal;
mod settings;
mod toolbox;

pub use accounts::AccountsState;
pub use dns_records::DnsRecordsState;
pub use domains::DomainsState;
pub use favorites::FavoritesState;
pub use modal::{
    Modal, ModalState, QueryToolType, get_all_dns_servers, get_all_providers, get_all_record_types,
    get_credential_fields,
};
pub use settings::{PaginationMode, SettingItem, SettingsState, Theme};
pub use toolbox::{ToolboxState, ToolboxTab};
