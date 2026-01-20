//! DNS 记录页面状态

use crate::model::domain::{DnsRecord, RecordData};

/// DNS 记录页面状态
#[derive(Debug, Default)]
pub struct DnsRecordsState {
    /// DNS 记录列表
    pub records: Vec<DnsRecord>,
    /// 当前选中的索引
    pub selected: usize,
    /// 是否正在加载
    pub loading: bool,
    /// 错误信息
    pub error: Option<String>,
    /// 当前域名 ID
    pub domain_id: String,
    /// 当前账号 ID
    pub account_id: String,
}

impl DnsRecordsState {
    /// 创建新的 DNS 记录状态
    pub fn new() -> Self {
        Self::default()
    }

    /// 选择上一项
    pub fn select_previous(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    /// 选择下一项
    pub fn select_next(&mut self) {
        if !self.records.is_empty() && self.selected < self.records.len() - 1 {
            self.selected += 1;
        }
    }

    /// 选择第一项
    pub fn select_first(&mut self) {
        self.selected = 0;
    }

    /// 选择最后一项
    pub fn select_last(&mut self) {
        if !self.records.is_empty() {
            self.selected = self.records.len() - 1;
        }
    }

    /// 获取当前选中的记录
    pub fn selected_record(&self) -> Option<&DnsRecord> {
        self.records.get(self.selected)
    }

    /// 设置记录列表
    pub fn set_records(&mut self, records: Vec<DnsRecord>) {
        self.records = records;
        self.selected = 0;
        self.loading = false;
        self.error = None;
    }

    /// 设置当前域名信息
    pub fn set_domain(&mut self, account_id: String, domain_id: String) {
        self.account_id = account_id;
        self.domain_id = domain_id;
        self.records.clear();
        self.selected = 0;
    }

    /// 添加模拟数据（开发测试用）
    pub fn load_mock_data(&mut self) {
        self.records = vec![
            DnsRecord {
                id: "rec_1".to_string(),
                domain_id: self.domain_id.clone(),
                name: "@".to_string(),
                data: RecordData::A {
                    address: "1.2.3.4".to_string(),
                },
                ttl: 600,
                proxied: Some(false),
                created_at: None,
                updated_at: None,
            },
            DnsRecord {
                id: "rec_2".to_string(),
                domain_id: self.domain_id.clone(),
                name: "www".to_string(),
                data: RecordData::Cname {
                    target: "example.com".to_string(),
                },
                ttl: 600,
                proxied: Some(true),
                created_at: None,
                updated_at: None,
            },
            DnsRecord {
                id: "rec_3".to_string(),
                domain_id: self.domain_id.clone(),
                name: "@".to_string(),
                data: RecordData::Mx {
                    priority: 10,
                    exchange: "mail.example.com".to_string(),
                },
                ttl: 3600,
                proxied: None,
                created_at: None,
                updated_at: None,
            },
            DnsRecord {
                id: "rec_4".to_string(),
                domain_id: self.domain_id.clone(),
                name: "@".to_string(),
                data: RecordData::Txt {
                    text: "v=spf1 include:_spf.google.com ~all".to_string(),
                },
                ttl: 3600,
                proxied: None,
                created_at: None,
                updated_at: None,
            },
        ];
        self.loading = false;
    }
}
