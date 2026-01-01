//! DNSSEC 验证模块

use std::net::IpAddr;
use std::time::Instant;

use hickory_resolver::{
    config::{NameServerConfigGroup, ResolverConfig, ResolverOpts},
    name_server::TokioConnectionProvider,
    proto::{
        dnssec::{rdata::DNSSECRData, PublicKey},
        rr::{record_data::RData, RecordType},
    },
    TokioResolver,
};

use crate::error::{CoreError, CoreResult};
use crate::types::{DnskeyRecord, DnssecResult, DsRecord, RrsigRecord};

/// Get algorithm name from algorithm number (RFC 8624)
fn get_algorithm_name(algorithm: u8) -> String {
    match algorithm {
        1 => "RSA/MD5 (deprecated)".to_string(),
        3 => "DSA/SHA-1 (deprecated)".to_string(),
        5 => "RSA/SHA-1".to_string(),
        6 => "DSA-NSEC3-SHA1 (deprecated)".to_string(),
        7 => "RSASHA1-NSEC3-SHA1".to_string(),
        8 => "RSA/SHA-256".to_string(),
        10 => "RSA/SHA-512".to_string(),
        12 => "GOST R 34.10-2001".to_string(),
        13 => "ECDSAP256SHA256".to_string(),
        14 => "ECDSAP384SHA384".to_string(),
        15 => "Ed25519".to_string(),
        16 => "Ed448".to_string(),
        _ => format!("Unknown ({})", algorithm),
    }
}

/// Get digest type name from digest type number (RFC 4034)
fn get_digest_type_name(digest_type: u8) -> String {
    match digest_type {
        1 => "SHA-1".to_string(),
        2 => "SHA-256".to_string(),
        3 => "GOST R 34.11-94".to_string(),
        4 => "SHA-384".to_string(),
        _ => format!("Unknown ({})", digest_type),
    }
}

/// 从 RRSIG/SIG 记录提取签名信息
fn extract_signature_record(
    type_covered: RecordType,
    algorithm: u8,
    labels: u8,
    original_ttl: u32,
    sig_expiration: u32,
    sig_inception: u32,
    key_tag: u16,
    signer_name: &str,
    signature_bytes: &[u8],
) -> RrsigRecord {
    use base64::{engine::general_purpose::STANDARD, Engine};
    use chrono::{DateTime, Utc};

    // 格式化时间戳
    let expiration = DateTime::<Utc>::from_timestamp(i64::from(sig_expiration), 0)
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
        .unwrap_or_else(|| format!("Invalid ({})", sig_expiration));

    let inception = DateTime::<Utc>::from_timestamp(i64::from(sig_inception), 0)
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
        .unwrap_or_else(|| format!("Invalid ({})", sig_inception));

    // Base64 编码签名
    let signature_b64 = STANDARD.encode(signature_bytes);

    RrsigRecord {
        type_covered: format!("{:?}", type_covered),
        algorithm,
        algorithm_name: get_algorithm_name(algorithm),
        labels,
        original_ttl,
        signature_expiration: expiration,
        signature_inception: inception,
        key_tag,
        signer_name: signer_name.to_string(),
        signature: signature_b64,
    }
}

/// DNSSEC 验证
pub async fn dnssec_check(domain: &str, nameserver: Option<&str>) -> CoreResult<DnssecResult> {
    let start_time = Instant::now();

    // Get system default DNS server addresses
    fn get_system_dns() -> String {
        let config = ResolverConfig::default();
        let servers: Vec<String> = config
            .name_servers()
            .iter()
            .map(|ns| ns.socket_addr.ip().to_string())
            .collect();
        if servers.is_empty() {
            "System Default".to_string()
        } else {
            servers.join(", ")
        }
    }

    // 根据 nameserver 参数决定使用自定义还是系统默认
    let effective_ns = nameserver.filter(|s| !s.is_empty());

    let (resolver, used_nameserver) = match effective_ns {
        Some(ns) => {
            let ns_ip: IpAddr = ns.parse().map_err(|_| {
                CoreError::ValidationError(format!("Invalid DNS server address: {ns}"))
            })?;

            let config = ResolverConfig::from_parts(
                None,
                vec![],
                NameServerConfigGroup::from_ips_clear(&[ns_ip], 53, true),
            );
            let provider = TokioConnectionProvider::default();
            let mut opts = ResolverOpts::default();
            opts.validate = true; // 启用 DNSSEC 验证
            let resolver = TokioResolver::builder_with_config(config, provider)
                .with_options(opts)
                .build();
            (resolver, ns.to_string())
        }
        None => {
            let system_dns = get_system_dns();
            let provider = TokioConnectionProvider::default();
            let mut opts = ResolverOpts::default();
            opts.validate = true; // 启用 DNSSEC 验证
            let resolver = TokioResolver::builder_with_config(ResolverConfig::default(), provider)
                .with_options(opts)
                .build();
            (resolver, system_dns)
        }
    };

    let mut dnskey_records = Vec::new();
    let mut ds_records = Vec::new();
    let mut rrsig_records = Vec::new();
    let mut dnssec_enabled = false;
    let mut validation_status = "indeterminate".to_string();

    // Query DNSKEY records
    if let Ok(response) = resolver.lookup(domain, RecordType::DNSKEY).await {
        dnssec_enabled = true;
        for record in response.record_iter() {
            // Try to parse DNSKEY from RData
            match record.data() {
                RData::DNSSEC(DNSSECRData::DNSKEY(dnskey)) => {
                    // Extract flags
                    let flags = dnskey.flags();

                    // Extract algorithm
                    let public_key = dnskey.public_key();
                    let algorithm = public_key.algorithm();
                    let algorithm_u8: u8 = algorithm.into();

                    // Extract public key bytes and encode as Base64
                    let public_key_bytes = public_key.public_bytes();
                    use base64::{engine::general_purpose::STANDARD, Engine};
                    let public_key_b64 = STANDARD.encode(public_key_bytes);

                    // Calculate key tag
                    let key_tag = match dnskey.calculate_key_tag() {
                        Ok(tag) => tag,
                        Err(e) => {
                            log::warn!("Failed to calculate key_tag: {}", e);
                            0
                        }
                    };

                    // Determine key type based on flags
                    let key_type = if dnskey.is_key_signing_key() {
                        "KSK".to_string()
                    } else if dnskey.zone_key() {
                        "ZSK".to_string()
                    } else {
                        format!("Unknown (flags={})", flags)
                    };

                    dnskey_records.push(DnskeyRecord {
                        flags,
                        protocol: 3,
                        algorithm: algorithm_u8,
                        algorithm_name: get_algorithm_name(algorithm_u8),
                        public_key: public_key_b64,
                        key_tag,
                        key_type,
                    });
                }
                _ => {
                    log::warn!("Unexpected RData type in DNSKEY query: {:?}", record.data());
                }
            }
        }
    }

    // Query DS records
    if let Ok(response) = resolver.lookup(domain, RecordType::DS).await {
        dnssec_enabled = true;
        for record in response.record_iter() {
            match record.data() {
                RData::DNSSEC(DNSSECRData::DS(ds)) => {
                    // Extract fields
                    let key_tag = ds.key_tag();
                    let algorithm: u8 = ds.algorithm().into();
                    let digest_type_enum = ds.digest_type();
                    let digest_type_u8: u8 = digest_type_enum.into();
                    let digest_bytes = ds.digest();

                    // Hex encode digest
                    let digest_hex = hex::encode(digest_bytes);

                    ds_records.push(DsRecord {
                        key_tag,
                        algorithm,
                        algorithm_name: get_algorithm_name(algorithm),
                        digest_type: digest_type_u8,
                        digest_type_name: get_digest_type_name(digest_type_u8),
                        digest: digest_hex,
                    });
                }
                _ => {
                    log::warn!("Unexpected RData type in DS query: {:?}", record.data());
                }
            }
        }
    }

    // Query RRSIG records
    if let Ok(response) = resolver.soa_lookup(domain).await {
        for record in response.as_lookup().record_iter() {
            if record.record_type() == RecordType::RRSIG {
                dnssec_enabled = true;

                match record.data() {
                    RData::DNSSEC(DNSSECRData::RRSIG(rrsig)) => {
                        let sig_record = extract_signature_record(
                            rrsig.type_covered(),
                            rrsig.algorithm().into(),
                            rrsig.num_labels(),
                            rrsig.original_ttl(),
                            rrsig.sig_expiration().get(),
                            rrsig.sig_inception().get(),
                            rrsig.key_tag(),
                            &rrsig.signer_name().to_string(),
                            rrsig.sig(),
                        );
                        rrsig_records.push(sig_record);
                    }
                    RData::DNSSEC(DNSSECRData::SIG(sig)) => {
                        let sig_record = extract_signature_record(
                            sig.type_covered(),
                            sig.algorithm().into(),
                            sig.num_labels(),
                            sig.original_ttl(),
                            sig.sig_expiration().get(),
                            sig.sig_inception().get(),
                            sig.key_tag(),
                            &sig.signer_name().to_string(),
                            sig.sig(),
                        );
                        rrsig_records.push(sig_record);
                    }
                    _ => {
                        log::warn!("Unexpected RData type in RRSIG query: {:?}", record.data());
                    }
                }
            }
        }
    }

    // 确定验证状态
    // 注意：由于启用了 ResolverOpts.validate = true，hickory-resolver 会自动验证 DNSSEC
    // 如果验证失败（bogus 签名），查询会返回 SERVFAIL 错误
    // 因此，能成功查询到 DNSSEC 记录说明验证通过或未启用 DNSSEC
    if dnssec_enabled {
        if !dnskey_records.is_empty() && !ds_records.is_empty() {
            // 有完整的 DNSSEC 记录，且查询成功（验证通过）
            validation_status = "secure".to_string();
            log::debug!(
                "DNSSEC validation for {}: Found DNSKEY ({}) and DS ({}) records, validation passed",
                domain,
                dnskey_records.len(),
                ds_records.len()
            );
        } else if !dnskey_records.is_empty() || !ds_records.is_empty() {
            // 只有部分 DNSSEC 记录
            validation_status = "indeterminate".to_string();
            log::debug!(
                "DNSSEC validation for {}: Partial DNSSEC records (DNSKEY: {}, DS: {})",
                domain,
                dnskey_records.len(),
                ds_records.len()
            );
        } else {
            validation_status = "insecure".to_string();
            log::debug!("DNSSEC validation for {}: No DNSSEC records found", domain);
        }
    } else {
        validation_status = "insecure".to_string();
        log::debug!("DNSSEC validation for {}: DNSSEC not enabled", domain);
    }

    let response_time_ms = start_time.elapsed().as_millis() as u64;

    Ok(DnssecResult {
        domain: domain.to_string(),
        dnssec_enabled,
        dnskey_records,
        ds_records,
        rrsig_records,
        validation_status,
        nameserver: used_nameserver,
        response_time_ms,
        error: None,
    })
}
