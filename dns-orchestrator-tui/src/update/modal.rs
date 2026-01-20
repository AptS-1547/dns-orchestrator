//! 弹窗更新逻辑

use crate::message::ModalMessage;
use crate::model::state::{get_all_providers, get_credential_fields, Modal};
use crate::model::App;

/// 处理弹窗消息
pub fn update(app: &mut App, msg: ModalMessage) {
    let Some(ref mut modal) = app.modal.active else {
        return;
    };

    match modal {
        Modal::AddAccount { .. } => handle_add_account(app, msg),
        Modal::ConfirmDelete { .. } => handle_confirm_delete(app, msg),
        Modal::DnsLookup { .. } => handle_dns_lookup(app, msg),
        Modal::WhoisLookup { .. } => handle_whois_lookup(app, msg),
        Modal::SslCheck { .. } => handle_ssl_check(app, msg),
        Modal::IpLookup { .. } => handle_ip_lookup(app, msg),
        Modal::HttpHeaderCheck { .. } => handle_http_header_check(app, msg),
        Modal::DnsPropagation { .. } => handle_dns_propagation(app, msg),
        Modal::DnssecCheck { .. } => handle_dnssec_check(app, msg),
        Modal::Error { .. } | Modal::Help => handle_simple_modal(app, msg),
        _ => {}
    }
}

/// 处理添加账号弹窗
fn handle_add_account(app: &mut App, msg: ModalMessage) {
    let Some(Modal::AddAccount {
        ref mut provider_index,
        ref mut name,
        ref mut credential_values,
        ref mut focus,
        ref mut show_secrets,
        ref mut error,
    }) = app.modal.active
    else {
        return;
    };

    let providers = get_all_providers();
    let total_fields = Modal::add_account_field_count(*provider_index);

    match msg {
        ModalMessage::Close => {
            app.modal.close();
            app.clear_status();
        }

        ModalMessage::NextField => {
            *focus = (*focus + 1) % total_fields;
        }

        ModalMessage::PrevField => {
            if *focus == 0 {
                *focus = total_fields - 1;
            } else {
                *focus -= 1;
            }
        }

        ModalMessage::PrevProvider => {
            if *focus == 0 {
                // 只有在服务商字段时才切换
                if *provider_index == 0 {
                    *provider_index = providers.len() - 1;
                } else {
                    *provider_index -= 1;
                }
                // 重置凭证值（因为不同服务商有不同字段）
                let new_credential_count = get_credential_fields(&providers[*provider_index]).len();
                *credential_values = vec![String::new(); new_credential_count];
            }
        }

        ModalMessage::NextProvider => {
            if *focus == 0 {
                *provider_index = (*provider_index + 1) % providers.len();
                // 重置凭证值
                let new_credential_count = get_credential_fields(&providers[*provider_index]).len();
                *credential_values = vec![String::new(); new_credential_count];
            }
        }

        ModalMessage::Confirm => {
            // 验证必填字段
            let provider = &providers[*provider_index];
            let fields = get_credential_fields(provider);

            // 检查凭证字段是否填写
            let has_empty_credentials = credential_values
                .iter()
                .enumerate()
                .any(|(i, v)| v.is_empty() && i < fields.len());

            if has_empty_credentials {
                *error = Some("Please fill in all the credential fields".to_string());
                return;
            }

            // TODO: 调用 CoreService 添加账号
            let account_name = if name.is_empty() {
                format!("{} Account", provider.display_name())
            } else {
                name.clone()
            };

            app.set_status(format!("Adding account...: {}...", account_name));

            // 暂时关闭弹窗并显示成功消息
            app.modal.close();
            app.set_status(format!("Success...! \"{}\" ", account_name));
        }

        ModalMessage::Input(ch) => {
            // 根据焦点输入字符
            if *focus == 1 {
                // 账号名称
                name.push(ch);
            } else if *focus >= 2 {
                // 凭证字段
                let field_index = *focus - 2;
                if let Some(value) = credential_values.get_mut(field_index) {
                    value.push(ch);
                }
            }
            // 清除错误
            *error = None;
        }

        ModalMessage::Backspace => {
            if *focus == 1 {
                name.pop();
            } else if *focus >= 2 {
                let field_index = *focus - 2;
                if let Some(value) = credential_values.get_mut(field_index) {
                    value.pop();
                }
            }
        }

        ModalMessage::Delete => {
            // Delete 键暂时不处理（需要光标位置支持）
        }

        ModalMessage::ToggleSecrets => {
            *show_secrets = !*show_secrets;
        }

        ModalMessage::ToggleDeleteFocus => {
            // 不适用于此弹窗
        }
    }
}

/// 处理确认删除弹窗
fn handle_confirm_delete(app: &mut App, msg: ModalMessage) {
    let Some(Modal::ConfirmDelete {
        ref item_type,
        ref item_name,
        ref item_id,
        ref mut focus,
    }) = app.modal.active
    else {
        return;
    };

    match msg {
        ModalMessage::Close => {
            app.modal.close();
            app.clear_status();
        }

        ModalMessage::ToggleDeleteFocus | ModalMessage::NextField | ModalMessage::PrevField => {
            *focus = if *focus == 0 { 1 } else { 0 };
        }

        ModalMessage::PrevProvider | ModalMessage::NextProvider => {
            // 左右键也可以切换焦点
            *focus = if *focus == 0 { 1 } else { 0 };
        }

        ModalMessage::Confirm => {
            if *focus == 1 {
                // 确认删除
                let item_type = item_type.clone();
                let item_name = item_name.clone();
                let _item_id = item_id.clone();

                app.modal.close();
                // TODO: 调用 CoreService 删除
                app.set_status(format!("Deleted{}: \"{}\"", item_type, item_name));
            } else {
                // 取消
                app.modal.close();
                app.clear_status();
            }
        }

        _ => {}
    }
}

/// 处理简单弹窗（帮助、错误）
fn handle_simple_modal(app: &mut App, msg: ModalMessage) {
    match msg {
        ModalMessage::Close | ModalMessage::Confirm => {
            app.modal.close();
        }
        _ => {}
    }
}

/// 处理 DNS 查询工具弹窗
fn handle_dns_lookup(app: &mut App, msg: ModalMessage) {
    use crate::model::state::{get_all_dns_servers, get_all_record_types};

    let Some(Modal::DnsLookup {
        ref mut domain,
        ref mut record_type_index,
        ref mut dns_server_index,
        ref mut focus,
        ref mut result,
        ref mut loading,
    }) = app.modal.active
    else {
        return;
    };

    let record_types = get_all_record_types();
    let dns_servers = get_all_dns_servers();

    match msg {
        ModalMessage::Close => {
            app.modal.close();
            app.clear_status();
        }

        ModalMessage::NextField => {
            *focus = (*focus + 1) % 3; // 0=域名, 1=记录类型, 2=DNS服务器
        }

        ModalMessage::PrevField => {
            if *focus == 0 {
                *focus = 2;
            } else {
                *focus -= 1;
            }
        }

        ModalMessage::PrevProvider => {
            // 用于切换记录类型或DNS服务器
            match *focus {
                1 => {
                    // 记录类型
                    if *record_type_index == 0 {
                        *record_type_index = record_types.len() - 1;
                    } else {
                        *record_type_index -= 1;
                    }
                }
                2 => {
                    // DNS服务器
                    if *dns_server_index == 0 {
                        *dns_server_index = dns_servers.len() - 1;
                    } else {
                        *dns_server_index -= 1;
                    }
                }
                _ => {}
            }
        }

        ModalMessage::NextProvider => {
            match *focus {
                1 => {
                    *record_type_index = (*record_type_index + 1) % record_types.len();
                }
                2 => {
                    *dns_server_index = (*dns_server_index + 1) % dns_servers.len();
                }
                _ => {}
            }
        }

        ModalMessage::Confirm => {
            if domain.is_empty() {
                app.set_status("Please enter a domain name");
                return;
            }

            *loading = true;
            let record_type = record_types[*record_type_index].name();
            let dns_server = dns_servers[*dns_server_index].name();

            // TODO: 实际执行 DNS 查询
            *result = Some(format!(
                "DNS Lookup for {} (Type: {}, Server: {})\nResult: (To be implemented)",
                domain, record_type, dns_server
            ));
            *loading = false;

            let domain_clone = domain.clone();
            app.set_status(format!("DNS query completed: {}", domain_clone));
        }

        ModalMessage::Input(ch) => {
            if *focus == 0 {
                domain.push(ch);
            }
        }

        ModalMessage::Backspace => {
            if *focus == 0 {
                domain.pop();
            }
        }

        _ => {}
    }
}

/// 处理 WHOIS 查询工具弹窗
fn handle_whois_lookup(app: &mut App, msg: ModalMessage) {
    let Some(Modal::WhoisLookup {
        ref mut domain,
        ref mut result,
        ref mut loading,
    }) = app.modal.active
    else {
        return;
    };

    match msg {
        ModalMessage::Close => {
            app.modal.close();
            app.clear_status();
        }

        ModalMessage::Confirm => {
            if domain.is_empty() {
                app.set_status("Please enter a domain name");
                return;
            }

            *loading = true;
            // TODO: 实际执行 WHOIS 查询
            *result = Some(format!(
                "WHOIS Lookup for {}\nResult: (To be implemented)",
                domain
            ));
            *loading = false;

            let domain_clone = domain.clone();
            app.set_status(format!("WHOIS query completed: {}", domain_clone));
        }

        ModalMessage::Input(ch) => {
            domain.push(ch);
        }

        ModalMessage::Backspace => {
            domain.pop();
        }

        _ => {}
    }
}

/// 处理 SSL 证书检查工具弹窗
fn handle_ssl_check(app: &mut App, msg: ModalMessage) {
    let Some(Modal::SslCheck {
        ref mut domain,
        ref mut result,
        ref mut loading,
    }) = app.modal.active
    else {
        return;
    };

    match msg {
        ModalMessage::Close => {
            app.modal.close();
            app.clear_status();
        }

        ModalMessage::Confirm => {
            if domain.is_empty() {
                app.set_status("Please enter a domain name");
                return;
            }

            *loading = true;
            // TODO: 实际执行 SSL 证书检查
            *result = Some(format!(
                "SSL Certificate Check for {}\nResult: (To be implemented)",
                domain
            ));
            *loading = false;

            let domain_clone = domain.clone();
            app.set_status(format!("SSL check completed: {}", domain_clone));
        }

        ModalMessage::Input(ch) => {
            domain.push(ch);
        }

        ModalMessage::Backspace => {
            domain.pop();
        }

        _ => {}
    }
}

/// 处理 IP 查询工具弹窗
fn handle_ip_lookup(app: &mut App, msg: ModalMessage) {
    let Some(Modal::IpLookup {
        ref mut input,
        ref mut result,
        ref mut loading,
    }) = app.modal.active
    else {
        return;
    };

    match msg {
        ModalMessage::Close => {
            app.modal.close();
            app.clear_status();
        }

        ModalMessage::Confirm => {
            if input.is_empty() {
                app.set_status("Please enter an IP address or domain");
                return;
            }

            *loading = true;
            // TODO: 实际执行 IP 查询
            *result = Some(format!(
                "IP Lookup for {}\nResult: (To be implemented)",
                input
            ));
            *loading = false;

            let input_clone = input.clone();
            app.set_status(format!("IP lookup completed: {}", input_clone));
        }

        ModalMessage::Input(ch) => {
            input.push(ch);
        }

        ModalMessage::Backspace => {
            input.pop();
        }

        _ => {}
    }
}

/// 处理 HTTP 头检查工具弹窗
fn handle_http_header_check(app: &mut App, msg: ModalMessage) {
    let Some(Modal::HttpHeaderCheck {
        ref mut url,
        ref mut method_index,
        ref mut focus,
        ref mut result,
        ref mut loading,
    }) = app.modal.active
    else {
        return;
    };

    let methods = ["GET", "HEAD", "POST", "PUT", "DELETE", "OPTIONS"];

    match msg {
        ModalMessage::Close => {
            app.modal.close();
            app.clear_status();
        }

        ModalMessage::NextField => {
            *focus = (*focus + 1) % 2; // 0=URL, 1=方法
        }

        ModalMessage::PrevField => {
            if *focus == 0 {
                *focus = 1;
            } else {
                *focus -= 1;
            }
        }

        ModalMessage::PrevProvider => {
            if *focus == 1 {
                if *method_index == 0 {
                    *method_index = methods.len() - 1;
                } else {
                    *method_index -= 1;
                }
            }
        }

        ModalMessage::NextProvider => {
            if *focus == 1 {
                *method_index = (*method_index + 1) % methods.len();
            }
        }

        ModalMessage::Confirm => {
            if url.is_empty() {
                app.set_status("Please enter a URL");
                return;
            }

            *loading = true;
            let method = methods[*method_index];

            // TODO: 实际执行 HTTP 头检查
            *result = Some(format!(
                "HTTP Header Check for {} (Method: {})\nResult: (To be implemented)",
                url, method
            ));
            *loading = false;

            let url_clone = url.clone();
            app.set_status(format!("HTTP header check completed: {}", url_clone));
        }

        ModalMessage::Input(ch) => {
            if *focus == 0 {
                url.push(ch);
            }
        }

        ModalMessage::Backspace => {
            if *focus == 0 {
                url.pop();
            }
        }

        _ => {}
    }
}

/// 处理 DNS 传播检查工具弹窗
fn handle_dns_propagation(app: &mut App, msg: ModalMessage) {
    use crate::model::state::get_all_record_types;

    let Some(Modal::DnsPropagation {
        ref mut domain,
        ref mut record_type_index,
        ref mut focus,
        ref mut result,
        ref mut loading,
    }) = app.modal.active
    else {
        return;
    };

    let record_types = get_all_record_types();

    match msg {
        ModalMessage::Close => {
            app.modal.close();
            app.clear_status();
        }

        ModalMessage::NextField => {
            *focus = (*focus + 1) % 2; // 0=域名, 1=记录类型
        }

        ModalMessage::PrevField => {
            if *focus == 0 {
                *focus = 1;
            } else {
                *focus -= 1;
            }
        }

        ModalMessage::PrevProvider => {
            if *focus == 1 {
                if *record_type_index == 0 {
                    *record_type_index = record_types.len() - 1;
                } else {
                    *record_type_index -= 1;
                }
            }
        }

        ModalMessage::NextProvider => {
            if *focus == 1 {
                *record_type_index = (*record_type_index + 1) % record_types.len();
            }
        }

        ModalMessage::Confirm => {
            if domain.is_empty() {
                app.set_status("Please enter a domain name");
                return;
            }

            *loading = true;
            let record_type = record_types[*record_type_index].name();

            // TODO: 实际执行 DNS 传播检查
            *result = Some(format!(
                "DNS Propagation Check for {} (Type: {})\nResult: (To be implemented)",
                domain, record_type
            ));
            *loading = false;

            let domain_clone = domain.clone();
            app.set_status(format!("DNS propagation check completed: {}", domain_clone));
        }

        ModalMessage::Input(ch) => {
            if *focus == 0 {
                domain.push(ch);
            }
        }

        ModalMessage::Backspace => {
            if *focus == 0 {
                domain.pop();
            }
        }

        _ => {}
    }
}

/// 处理 DNSSEC 验证工具弹窗
fn handle_dnssec_check(app: &mut App, msg: ModalMessage) {
    let Some(Modal::DnssecCheck {
        ref mut domain,
        ref mut result,
        ref mut loading,
    }) = app.modal.active
    else {
        return;
    };

    match msg {
        ModalMessage::Close => {
            app.modal.close();
            app.clear_status();
        }

        ModalMessage::Confirm => {
            if domain.is_empty() {
                app.set_status("Please enter a domain name");
                return;
            }

            *loading = true;
            // TODO: 实际执行 DNSSEC 验证
            *result = Some(format!(
                "DNSSEC Check for {}\nResult: (To be implemented)",
                domain
            ));
            *loading = false;

            let domain_clone = domain.clone();
            app.set_status(format!("DNSSEC check completed: {}", domain_clone));
        }

        ModalMessage::Input(ch) => {
            domain.push(ch);
        }

        ModalMessage::Backspace => {
            domain.pop();
        }

        _ => {}
    }
}
