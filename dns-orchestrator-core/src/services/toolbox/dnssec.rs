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
    let (resolver, used_nameserver) = if let Some(ns) = nameserver {
        if ns.is_empty() {
            let system_dns = get_system_dns();
            let provider = TokioConnectionProvider::default();
            let resolver = TokioResolver::builder_with_config(ResolverConfig::default(), provider)
                .with_options(ResolverOpts::default())
                .build();
            (resolver, system_dns)
        } else {
            let ns_ip: IpAddr = ns.parse().map_err(|_| {
                CoreError::ValidationError(format!("Invalid DNS server address: {ns}"))
            })?;

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
                        // Extract fields
                        let type_covered = format!("{:?}", rrsig.type_covered());
                        let algorithm: u8 = rrsig.algorithm().into();
                        let labels = rrsig.num_labels();
                        let original_ttl = rrsig.original_ttl();

                        // Convert timestamps (SerialNumber wraps u32 Unix timestamp)
                        let expiration_ts = rrsig.sig_expiration().get();
                        let inception_ts = rrsig.sig_inception().get();

                        // Format timestamps to human-readable format
                        use chrono::{DateTime, Utc};
                        let expiration =
                            DateTime::<Utc>::from_timestamp(i64::from(expiration_ts), 0)
                                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
                                .unwrap_or_else(|| format!("Invalid ({})", expiration_ts));

                        let inception = DateTime::<Utc>::from_timestamp(i64::from(inception_ts), 0)
                            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
                            .unwrap_or_else(|| format!("Invalid ({})", inception_ts));

                        let key_tag = rrsig.key_tag();
                        let signer_name = rrsig.signer_name().to_string();

                        // Base64 encode signature
                        let signature_bytes = rrsig.sig();
                        use base64::{engine::general_purpose::STANDARD, Engine};
                        let signature_b64 = STANDARD.encode(signature_bytes);

                        rrsig_records.push(RrsigRecord {
                            type_covered,
                            algorithm,
                            algorithm_name: get_algorithm_name(algorithm),
                            labels,
                            original_ttl,
                            signature_expiration: expiration,
                            signature_inception: inception,
                            key_tag,
                            signer_name,
                            signature: signature_b64,
                        });
                    }
                    RData::DNSSEC(DNSSECRData::SIG(sig)) => {
                        // SIG is similar to RRSIG, process the same way
                        let type_covered = format!("{:?}", sig.type_covered());
                        let algorithm: u8 = sig.algorithm().into();
                        let labels = sig.num_labels();
                        let original_ttl = sig.original_ttl();

                        let expiration_ts = sig.sig_expiration().get();
                        let inception_ts = sig.sig_inception().get();

                        use chrono::{DateTime, Utc};
                        let expiration =
                            DateTime::<Utc>::from_timestamp(i64::from(expiration_ts), 0)
                                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
                                .unwrap_or_else(|| format!("Invalid ({})", expiration_ts));

                        let inception = DateTime::<Utc>::from_timestamp(i64::from(inception_ts), 0)
                            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
                            .unwrap_or_else(|| format!("Invalid ({})", inception_ts));

                        let key_tag = sig.key_tag();
                        let signer_name = sig.signer_name().to_string();

                        let signature_bytes = sig.sig();
                        use base64::{engine::general_purpose::STANDARD, Engine};
                        let signature_b64 = STANDARD.encode(signature_bytes);

                        rrsig_records.push(RrsigRecord {
                            type_covered,
                            algorithm,
                            algorithm_name: get_algorithm_name(algorithm),
                            labels,
                            original_ttl,
                            signature_expiration: expiration,
                            signature_inception: inception,
                            key_tag,
                            signer_name,
                            signature: signature_b64,
                        });
                    }
                    _ => {
                        log::warn!("Unexpected RData type in RRSIG query: {:?}", record.data());
                    }
                }
            }
        }
    }

    // 确定验证状态
    if dnssec_enabled {
        if !dnskey_records.is_empty() && !ds_records.is_empty() {
            validation_status = "secure".to_string();
        } else if !dnskey_records.is_empty() || !ds_records.is_empty() {
            validation_status = "indeterminate".to_string();
        } else {
            validation_status = "insecure".to_string();
        }
    } else {
        validation_status = "insecure".to_string();
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
