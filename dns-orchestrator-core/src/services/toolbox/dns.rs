//! DNS 查询模块

use std::net::IpAddr;

use futures::future::join_all;
use hickory_resolver::{
    config::{NameServerConfigGroup, ResolverConfig, ResolverOpts},
    name_server::TokioConnectionProvider,
    TokioResolver,
};

use crate::error::{CoreError, CoreResult};
use crate::types::{DnsLookupRecord, DnsLookupResult};

/// DNS 查询
pub async fn dns_lookup(
    domain: &str,
    record_type: &str,
    nameserver: Option<&str>,
) -> CoreResult<DnsLookupResult> {
    // 获取系统默认 DNS 服务器地址
    fn get_system_dns() -> String {
        let config = ResolverConfig::default();
        let servers: Vec<String> = config
            .name_servers()
            .iter()
            .map(|ns| ns.socket_addr.ip().to_string())
            .collect();
        if servers.is_empty() {
            "系统默认".to_string()
        } else {
            servers.join(", ")
        }
    }

    // 根据 nameserver 参数决定使用自定义还是系统默认
    let (resolver, used_nameserver) = if let Some(ns) = nameserver {
        if ns.is_empty() {
            let system_dns = get_system_dns();
            let provider = TokioConnectionProvider::default();
            let resolver = TokioResolver::builder_with_config(ResolverConfig::default(), provider)
                .with_options(ResolverOpts::default())
                .build();
            (resolver, system_dns)
        } else {
            let ns_ip: IpAddr = ns
                .parse()
                .map_err(|_| CoreError::ValidationError(format!("无效的 DNS 服务器地址: {ns}")))?;

            let config = ResolverConfig::from_parts(
                None,
                vec![],
                NameServerConfigGroup::from_ips_clear(&[ns_ip], 53, true),
            );
            let provider = TokioConnectionProvider::default();
            let resolver = TokioResolver::builder_with_config(config, provider)
                .with_options(ResolverOpts::default())
                .build();
            (resolver, ns.to_string())
        }
    } else {
        let system_dns = get_system_dns();
        let provider = TokioConnectionProvider::default();
        let resolver = TokioResolver::builder_with_config(ResolverConfig::default(), provider)
            .with_options(ResolverOpts::default())
            .build();
        (resolver, system_dns)
    };

    let mut records: Vec<DnsLookupRecord> = Vec::new();
    let record_type_upper = record_type.to_uppercase();

    match record_type_upper.as_str() {
        "A" => lookup_a(&resolver, domain, &mut records).await,
        "AAAA" => lookup_aaaa(&resolver, domain, &mut records).await,
        "MX" => lookup_mx(&resolver, domain, &mut records).await,
        "TXT" => lookup_txt(&resolver, domain, &mut records).await,
        "NS" => lookup_ns(&resolver, domain, &mut records).await,
        "CNAME" => lookup_cname(&resolver, domain, &mut records).await,
        "SOA" => lookup_soa(&resolver, domain, &mut records).await,
        "SRV" => lookup_srv(&resolver, domain, &mut records).await,
        "CAA" => lookup_caa(&resolver, domain, &mut records).await,
        "PTR" => lookup_ptr(&resolver, domain, &mut records).await,
        "ALL" => {
            let types = vec![
                "A", "AAAA", "CNAME", "MX", "TXT", "NS", "SOA", "SRV", "CAA", "PTR",
            ];
            let ns = nameserver.map(String::from);
            let futures: Vec<_> = types
                .into_iter()
                .map(|t| {
                    let ns = ns.clone();
                    let domain = domain.to_string();
                    async move { dns_lookup(&domain, t, ns.as_deref()).await }
                })
                .collect();

            let results = join_all(futures).await;
            for result in results.into_iter().flatten() {
                records.extend(result.records);
            }
        }
        _ => {
            return Err(CoreError::ValidationError(format!(
                "不支持的记录类型: {record_type}"
            )));
        }
    }

    Ok(DnsLookupResult {
        nameserver: used_nameserver,
        records,
    })
}

async fn lookup_a(resolver: &TokioResolver, domain: &str, records: &mut Vec<DnsLookupRecord>) {
    if let Ok(response) = resolver.ipv4_lookup(domain).await {
        for ip in response.iter() {
            records.push(DnsLookupRecord {
                record_type: "A".to_string(),
                name: domain.to_string(),
                value: ip.to_string(),
                ttl: response
                    .as_lookup()
                    .record_iter()
                    .next()
                    .map_or(0, hickory_resolver::proto::rr::Record::ttl),
                priority: None,
            });
        }
    }
}

async fn lookup_aaaa(resolver: &TokioResolver, domain: &str, records: &mut Vec<DnsLookupRecord>) {
    if let Ok(response) = resolver.ipv6_lookup(domain).await {
        for ip in response.iter() {
            records.push(DnsLookupRecord {
                record_type: "AAAA".to_string(),
                name: domain.to_string(),
                value: ip.to_string(),
                ttl: response
                    .as_lookup()
                    .record_iter()
                    .next()
                    .map_or(0, hickory_resolver::proto::rr::Record::ttl),
                priority: None,
            });
        }
    }
}

async fn lookup_mx(resolver: &TokioResolver, domain: &str, records: &mut Vec<DnsLookupRecord>) {
    if let Ok(response) = resolver.mx_lookup(domain).await {
        for mx in response.iter() {
            records.push(DnsLookupRecord {
                record_type: "MX".to_string(),
                name: domain.to_string(),
                value: mx.exchange().to_string().trim_end_matches('.').to_string(),
                ttl: response
                    .as_lookup()
                    .record_iter()
                    .next()
                    .map_or(0, hickory_resolver::proto::rr::Record::ttl),
                priority: Some(mx.preference()),
            });
        }
    }
}

async fn lookup_txt(resolver: &TokioResolver, domain: &str, records: &mut Vec<DnsLookupRecord>) {
    if let Ok(response) = resolver.txt_lookup(domain).await {
        for txt in response.iter() {
            let txt_data: String = txt
                .iter()
                .map(|data| String::from_utf8_lossy(data).to_string())
                .collect::<String>();
            records.push(DnsLookupRecord {
                record_type: "TXT".to_string(),
                name: domain.to_string(),
                value: txt_data,
                ttl: response
                    .as_lookup()
                    .record_iter()
                    .next()
                    .map_or(0, hickory_resolver::proto::rr::Record::ttl),
                priority: None,
            });
        }
    }
}

async fn lookup_ns(resolver: &TokioResolver, domain: &str, records: &mut Vec<DnsLookupRecord>) {
    if let Ok(response) = resolver.ns_lookup(domain).await {
        for ns in response.iter() {
            records.push(DnsLookupRecord {
                record_type: "NS".to_string(),
                name: domain.to_string(),
                value: ns.to_string().trim_end_matches('.').to_string(),
                ttl: response
                    .as_lookup()
                    .record_iter()
                    .next()
                    .map_or(0, hickory_resolver::proto::rr::Record::ttl),
                priority: None,
            });
        }
    }
}

async fn lookup_cname(resolver: &TokioResolver, domain: &str, records: &mut Vec<DnsLookupRecord>) {
    if let Ok(response) = resolver
        .lookup(domain, hickory_resolver::proto::rr::RecordType::CNAME)
        .await
    {
        for record in response.record_iter() {
            if let Some(cname) = record.data().as_cname() {
                records.push(DnsLookupRecord {
                    record_type: "CNAME".to_string(),
                    name: domain.to_string(),
                    value: cname.0.to_string().trim_end_matches('.').to_string(),
                    ttl: record.ttl(),
                    priority: None,
                });
            }
        }
    }
}

async fn lookup_soa(resolver: &TokioResolver, domain: &str, records: &mut Vec<DnsLookupRecord>) {
    if let Ok(response) = resolver.soa_lookup(domain).await {
        if let Some(soa) = response.iter().next() {
            let value = format!(
                "{} {} {} {} {} {} {}",
                soa.mname().to_string().trim_end_matches('.'),
                soa.rname().to_string().trim_end_matches('.'),
                soa.serial(),
                soa.refresh(),
                soa.retry(),
                soa.expire(),
                soa.minimum()
            );
            records.push(DnsLookupRecord {
                record_type: "SOA".to_string(),
                name: domain.to_string(),
                value,
                ttl: response
                    .as_lookup()
                    .record_iter()
                    .next()
                    .map_or(0, hickory_resolver::proto::rr::Record::ttl),
                priority: None,
            });
        }
    }
}

async fn lookup_srv(resolver: &TokioResolver, domain: &str, records: &mut Vec<DnsLookupRecord>) {
    if let Ok(response) = resolver.srv_lookup(domain).await {
        for srv in response.iter() {
            let value = format!(
                "{} {} {}",
                srv.weight(),
                srv.port(),
                srv.target().to_string().trim_end_matches('.')
            );
            records.push(DnsLookupRecord {
                record_type: "SRV".to_string(),
                name: domain.to_string(),
                value,
                ttl: response
                    .as_lookup()
                    .record_iter()
                    .next()
                    .map_or(0, hickory_resolver::proto::rr::Record::ttl),
                priority: Some(srv.priority()),
            });
        }
    }
}

async fn lookup_caa(resolver: &TokioResolver, domain: &str, records: &mut Vec<DnsLookupRecord>) {
    if let Ok(response) = resolver
        .lookup(domain, hickory_resolver::proto::rr::RecordType::CAA)
        .await
    {
        for record in response.record_iter() {
            if let Some(caa) = record.data().as_caa() {
                let value = format!(
                    "{} {} \"{}\"",
                    if caa.issuer_critical() { 128 } else { 0 },
                    caa.tag().as_str(),
                    String::from_utf8_lossy(caa.raw_value())
                );
                records.push(DnsLookupRecord {
                    record_type: "CAA".to_string(),
                    name: domain.to_string(),
                    value,
                    ttl: record.ttl(),
                    priority: None,
                });
            }
        }
    }
}

async fn lookup_ptr(resolver: &TokioResolver, domain: &str, records: &mut Vec<DnsLookupRecord>) {
    if let Ok(response) = resolver
        .lookup(domain, hickory_resolver::proto::rr::RecordType::PTR)
        .await
    {
        for record in response.record_iter() {
            if let Some(ptr) = record.data().as_ptr() {
                records.push(DnsLookupRecord {
                    record_type: "PTR".to_string(),
                    name: domain.to_string(),
                    value: ptr.0.to_string().trim_end_matches('.').to_string(),
                    ttl: record.ttl(),
                    priority: None,
                });
            }
        }
    }
}
