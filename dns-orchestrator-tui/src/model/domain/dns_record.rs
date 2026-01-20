//! DNS 记录数据模型

/// DNS 记录类型（用于查询过滤）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DnsRecordType {
    A,
    Aaaa,
    Cname,
    Mx,
    Txt,
    Ns,
    Srv,
    Caa,
}

impl DnsRecordType {
    /// 获取记录类型名称（大写形式，用于显示）
    pub fn as_str(&self) -> &'static str {
        match self {
            DnsRecordType::A => "A",
            DnsRecordType::Aaaa => "AAAA",
            DnsRecordType::Cname => "CNAME",
            DnsRecordType::Mx => "MX",
            DnsRecordType::Txt => "TXT",
            DnsRecordType::Ns => "NS",
            DnsRecordType::Srv => "SRV",
            DnsRecordType::Caa => "CAA",
        }
    }
}

/// DNS 记录数据 - 类型安全的多态表示
///
/// 不同类型的 DNS 记录有不同的字段结构，
/// 使用枚举来确保类型安全，避免无效的字段组合。
#[derive(Debug, Clone, PartialEq)]
pub enum RecordData {
    /// A 记录：IPv4 地址
    A { address: String },

    /// AAAA 记录：IPv6 地址
    Aaaa { address: String },

    /// CNAME 记录：别名
    Cname { target: String },

    /// MX 记录：邮件交换
    Mx { priority: u16, exchange: String },

    /// TXT 记录：文本
    Txt { text: String },

    /// NS 记录：域名服务器
    Ns { nameserver: String },

    /// SRV 记录：服务定位
    Srv {
        priority: u16,
        weight: u16,
        port: u16,
        target: String,
    },

    /// CAA 记录：证书颁发机构授权
    Caa {
        flags: u8,
        tag: String,
        value: String,
    },
}

impl RecordData {
    /// 获取记录类型
    pub fn record_type(&self) -> DnsRecordType {
        match self {
            Self::A { .. } => DnsRecordType::A,
            Self::Aaaa { .. } => DnsRecordType::Aaaa,
            Self::Cname { .. } => DnsRecordType::Cname,
            Self::Mx { .. } => DnsRecordType::Mx,
            Self::Txt { .. } => DnsRecordType::Txt,
            Self::Ns { .. } => DnsRecordType::Ns,
            Self::Srv { .. } => DnsRecordType::Srv,
            Self::Caa { .. } => DnsRecordType::Caa,
        }
    }

    /// 获取显示用的主要值（用于列表显示）
    pub fn display_value(&self) -> String {
        match self {
            Self::A { address } | Self::Aaaa { address } => address.clone(),
            Self::Cname { target } => target.clone(),
            Self::Mx { exchange, .. } => exchange.clone(),
            Self::Txt { text } => text.clone(),
            Self::Ns { nameserver } => nameserver.clone(),
            Self::Srv { target, .. } => target.clone(),
            Self::Caa { value, .. } => value.clone(),
        }
    }
}

/// DNS 记录
#[derive(Debug, Clone)]
pub struct DnsRecord {
    pub id: String,
    pub domain_id: String,
    pub name: String,
    pub ttl: u32,
    /// 类型安全的记录数据
    pub data: RecordData,
    /// Cloudflare 专用：是否启用 CDN 代理
    pub proxied: Option<bool>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}