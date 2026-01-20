//! 弹窗/对话框状态

use crate::i18n::t;
use crate::model::domain::ProviderType;

/// DNS 记录类型（用于 DNS Lookup 工具）
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DnsRecordTypeOption {
    All,
    A,
    Aaaa,
    Cname,
    Mx,
    Txt,
    Ns,
    Srv,
    Caa,
}

impl DnsRecordTypeOption {
    pub fn name(&self) -> &'static str {
        match self {
            Self::All => "ALL",
            Self::A => "A",
            Self::Aaaa => "AAAA",
            Self::Cname => "CNAME",
            Self::Mx => "MX",
            Self::Txt => "TXT",
            Self::Ns => "NS",
            Self::Srv => "SRV",
            Self::Caa => "CAA",
        }
    }
}

/// 获取所有 DNS 记录类型选项
pub fn get_all_record_types() -> Vec<DnsRecordTypeOption> {
    vec![
        DnsRecordTypeOption::All,
        DnsRecordTypeOption::A,
        DnsRecordTypeOption::Aaaa,
        DnsRecordTypeOption::Cname,
        DnsRecordTypeOption::Mx,
        DnsRecordTypeOption::Txt,
        DnsRecordTypeOption::Ns,
        DnsRecordTypeOption::Srv,
        DnsRecordTypeOption::Caa,
    ]
}

/// DNS 服务器选项
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DnsServerOption {
    SystemDefault,
    Google,
    Cloudflare,
    Quad9,
}

impl DnsServerOption {
    pub fn name(&self) -> &'static str {
        match self {
            Self::SystemDefault => "System Default",
            Self::Google => "Google DNS (8.8.8.8)",
            Self::Cloudflare => "Cloudflare DNS (1.1.1.1)",
            Self::Quad9 => "Quad9 DNS (9.9.9.9)",
        }
    }

    pub fn address(&self) -> Option<&'static str> {
        match self {
            Self::SystemDefault => None,
            Self::Google => Some("8.8.8.8"),
            Self::Cloudflare => Some("1.1.1.1"),
            Self::Quad9 => Some("9.9.9.9"),
        }
    }
}

/// 获取所有 DNS 服务器选项
pub fn get_all_dns_servers() -> Vec<DnsServerOption> {
    vec![
        DnsServerOption::SystemDefault,
        DnsServerOption::Google,
        DnsServerOption::Cloudflare,
        DnsServerOption::Quad9,
    ]
}

/// 凭证字段定义
#[derive(Debug, Clone)]
pub struct CredentialField {
    /// 字段键名（用于存储）
    pub key: &'static str,
    /// 显示标签
    pub label: &'static str,
    /// 占位符文本
    pub placeholder: &'static str,
    /// 是否为密码字段
    pub is_secret: bool,
}

/// 获取服务商的凭证字段定义
pub fn get_credential_fields(provider: &ProviderType) -> Vec<CredentialField> {
    let texts = t();
    match provider {
        ProviderType::Cloudflare => vec![CredentialField {
            key: "apiToken",
            label: texts.modal.add_account.api_token,
            placeholder: texts.modal.add_account.api_token_hint,
            is_secret: true,
        }],
        ProviderType::Aliyun => vec![
            CredentialField {
                key: "accessKeyId",
                label: texts.modal.add_account.accesskey_id,
                placeholder: texts.modal.add_account.accesskey_id_hint,
                is_secret: false,
            },
            CredentialField {
                key: "accessKeySecret",
                label: texts.modal.add_account.accesskey_secret,
                placeholder: texts.modal.add_account.accesskey_secret_hint,
                is_secret: true,
            },
        ],
        ProviderType::Dnspod => vec![
            CredentialField {
                key: "secretId",
                label: texts.modal.add_account.secretid,
                placeholder: texts.modal.add_account.secretid_hint,
                is_secret: false,
            },
            CredentialField {
                key: "secretKey",
                label: texts.modal.add_account.secretkey,
                placeholder: texts.modal.add_account.secretkey_hint,
                is_secret: true,
            },
        ],
        ProviderType::Huaweicloud => vec![
            CredentialField {
                key: "accessKeyId",
                label: texts.modal.add_account.access_key_id,
                placeholder: texts.modal.add_account.access_key_id_hint,
                is_secret: false,
            },
            CredentialField {
                key: "secretAccessKey",
                label: texts.modal.add_account.secret_access_key,
                placeholder: texts.modal.add_account.secret_access_key_hint,
                is_secret: true,
            },
        ],
    }
}

/// 获取所有可用的服务商列表
pub fn get_all_providers() -> Vec<ProviderType> {
    vec![
        ProviderType::Cloudflare,
        ProviderType::Aliyun,
        ProviderType::Dnspod,
        ProviderType::Huaweicloud,
    ]
}

/// 弹窗类型
#[derive(Debug, Clone)]
pub enum Modal {
    /// 添加账号
    AddAccount {
        /// 选中的服务商索引
        provider_index: usize,
        /// 账号名称
        name: String,
        /// 凭证值（与字段定义对应）
        credential_values: Vec<String>,
        /// 当前焦点：0=服务商, 1=名称, 2+=凭证字段
        focus: usize,
        /// 密码是否可见
        show_secrets: bool,
        /// 错误信息
        error: Option<String>,
    },
    /// 编辑账号
    EditAccount {
        account_id: String,
        name: String,
        focus: usize,
    },
    /// 确认删除
    ConfirmDelete {
        /// 删除类型描述
        item_type: String,
        /// 项目名称
        item_name: String,
        /// 项目 ID
        item_id: String,
        /// 焦点：0=取消, 1=确认
        focus: usize,
    },
    /// 添加 DNS 记录
    AddDnsRecord {
        /// 记录名称
        name: String,
        /// 记录类型索引
        record_type_index: usize,
        /// 记录值
        value: String,
        /// TTL
        ttl: String,
        /// 当前焦点
        focus: usize,
    },
    /// 编辑 DNS 记录
    EditDnsRecord {
        record_id: String,
        name: String,
        value: String,
        ttl: String,
        focus: usize,
    },
    /// DNS 查询工具
    DnsLookup {
        /// 域名输入
        domain: String,
        /// DNS 记录类型索引
        record_type_index: usize,
        /// DNS 服务器索引
        dns_server_index: usize,
        /// 焦点：0=域名, 1=记录类型, 2=DNS服务器
        focus: usize,
        /// 查询结果
        result: Option<String>,
        /// 是否正在查询
        loading: bool,
    },
    /// WHOIS 查询工具
    WhoisLookup {
        /// 域名输入
        domain: String,
        /// 查询结果
        result: Option<String>,
        /// 是否正在查询
        loading: bool,
    },
    /// SSL 证书检查工具
    SslCheck {
        /// 域名输入
        domain: String,
        /// 查询结果
        result: Option<String>,
        /// 是否正在查询
        loading: bool,
    },
    /// IP 查询工具
    IpLookup {
        /// IP 或域名输入
        input: String,
        /// 查询结果
        result: Option<String>,
        /// 是否正在查询
        loading: bool,
    },
    /// HTTP 头检查工具
    HttpHeaderCheck {
        /// URL 输入
        url: String,
        /// HTTP 方法索引 (GET, HEAD, POST, etc.)
        method_index: usize,
        /// 焦点：0=URL, 1=方法
        focus: usize,
        /// 查询结果
        result: Option<String>,
        /// 是否正在查询
        loading: bool,
    },
    /// DNS 传播检查工具
    DnsPropagation {
        /// 域名输入
        domain: String,
        /// DNS 记录类型索引
        record_type_index: usize,
        /// 焦点：0=域名, 1=记录类型
        focus: usize,
        /// 查询结果
        result: Option<String>,
        /// 是否正在查询
        loading: bool,
    },
    /// DNSSEC 验证工具
    DnssecCheck {
        /// 域名输入
        domain: String,
        /// 查询结果
        result: Option<String>,
        /// 是否正在查询
        loading: bool,
    },
    /// 帮助信息
    Help,
    /// 错误提示
    Error {
        title: String,
        message: String,
    },
}

impl Modal {
    /// 获取添加账号弹窗的总字段数（服务商 + 名称 + 凭证字段数）
    pub fn add_account_field_count(provider_index: usize) -> usize {
        let providers = get_all_providers();
        let provider = &providers[provider_index];
        let credential_count = get_credential_fields(provider).len();
        2 + credential_count // 服务商 + 名称 + 凭证字段
    }
}

/// 弹窗状态
#[derive(Debug, Default)]
pub struct ModalState {
    /// 当前活动的弹窗
    pub active: Option<Modal>,
}

impl ModalState {
    /// 创建新的弹窗状态
    pub fn new() -> Self {
        Self::default()
    }

    /// 显示弹窗
    pub fn show(&mut self, modal: Modal) {
        self.active = Some(modal);
    }

    /// 关闭弹窗
    pub fn close(&mut self) {
        self.active = None;
    }

    /// 是否有活动弹窗
    pub fn is_open(&self) -> bool {
        self.active.is_some()
    }

    /// 显示添加账号弹窗
    pub fn show_add_account(&mut self) {
        let providers = get_all_providers();
        let credential_count = get_credential_fields(&providers[0]).len();

        self.active = Some(Modal::AddAccount {
            provider_index: 0,
            name: String::new(),
            credential_values: vec![String::new(); credential_count],
            focus: 0,
            show_secrets: false,
            error: None,
        });
    }

    /// 显示确认删除弹窗
    pub fn show_confirm_delete(&mut self, item_type: &str, item_name: &str, item_id: &str) {
        self.active = Some(Modal::ConfirmDelete {
            item_type: item_type.to_string(),
            item_name: item_name.to_string(),
            item_id: item_id.to_string(),
            focus: 0,
        });
    }

    /// 显示添加 DNS 记录弹窗
    pub fn show_add_dns_record(&mut self) {
        self.active = Some(Modal::AddDnsRecord {
            name: String::new(),
            record_type_index: 0,
            value: String::new(),
            ttl: "600".to_string(),
            focus: 0,
        });
    }

    /// 显示错误弹窗
    pub fn show_error(&mut self, title: &str, message: &str) {
        self.active = Some(Modal::Error {
            title: title.to_string(),
            message: message.to_string(),
        });
    }

    /// 显示帮助弹窗
    pub fn show_help(&mut self) {
        self.active = Some(Modal::Help);
    }

    /// 显示 DNS 查询工具弹窗
    pub fn show_dns_lookup(&mut self) {
        self.active = Some(Modal::DnsLookup {
            domain: String::new(),
            record_type_index: 0,
            dns_server_index: 0,
            focus: 0,
            result: None,
            loading: false,
        });
    }

    /// 显示 WHOIS 查询工具弹窗
    pub fn show_whois_lookup(&mut self) {
        self.active = Some(Modal::WhoisLookup {
            domain: String::new(),
            result: None,
            loading: false,
        });
    }

    /// 显示 SSL 证书检查工具弹窗
    pub fn show_ssl_check(&mut self) {
        self.active = Some(Modal::SslCheck {
            domain: String::new(),
            result: None,
            loading: false,
        });
    }

    /// 显示 IP 查询工具弹窗
    pub fn show_ip_lookup(&mut self) {
        self.active = Some(Modal::IpLookup {
            input: String::new(),
            result: None,
            loading: false,
        });
    }

    /// 显示 HTTP 头检查工具弹窗
    pub fn show_http_header_check(&mut self) {
        self.active = Some(Modal::HttpHeaderCheck {
            url: String::new(),
            method_index: 0,
            focus: 0,
            result: None,
            loading: false,
        });
    }

    /// 显示 DNS 传播检查工具弹窗
    pub fn show_dns_propagation(&mut self) {
        self.active = Some(Modal::DnsPropagation {
            domain: String::new(),
            record_type_index: 0,
            focus: 0,
            result: None,
            loading: false,
        });
    }

    /// 显示 DNSSEC 验证工具弹窗
    pub fn show_dnssec_check(&mut self) {
        self.active = Some(Modal::DnssecCheck {
            domain: String::new(),
            result: None,
            loading: false,
        });
    }
}
