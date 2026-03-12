//! 内容面板更新逻辑
//!
//! 处理内容面板中的各种操作消息

use crate::message::ContentMessage;
use crate::model::{App, Page};
use dns_orchestrator_core::types::DomainMetadataUpdate;

/// 处理内容面板消息
pub fn update(app: &mut App, msg: ContentMessage) {
    match msg {
        // ========== 列表导航 ==========
        ContentMessage::SelectPrevious => {
            handle_select_previous(app);
        }
        ContentMessage::SelectNext => {
            handle_select_next(app);
        }
        ContentMessage::SelectFirst => {
            handle_select_first(app);
        }
        ContentMessage::SelectLast => {
            handle_select_last(app);
        }
        ContentMessage::Confirm => {
            handle_confirm(app);
        }

        // ========== CRUD 操作 ==========
        ContentMessage::Add => {
            handle_add(app);
        }
        ContentMessage::Edit => {
            handle_edit(app);
        }
        ContentMessage::Delete => {
            handle_delete(app);
        }

        // ========== 导入导出 ==========
        ContentMessage::Import => {
            handle_import(app);
        }
        ContentMessage::Export => {
            handle_export(app);
        }

        // ========== 工具箱专用 ==========
        ContentMessage::SwitchTab => {
            handle_switch_tab(app);
        }
        ContentMessage::Execute => {
            handle_execute(app);
        }

        // ========== 设置页面专用 ==========
        ContentMessage::TogglePrev => {
            handle_toggle_prev(app);
        }
        ContentMessage::ToggleNext => {
            handle_toggle_next(app);
        }

        // ========== 收藏操作 ==========
        ContentMessage::ToggleFavorite => {
            handle_toggle_favorite(app);
        }
    }
}

// ========== 列表导航处理 ==========

fn handle_select_previous(app: &mut App) {
    match &app.current_page {
        Page::Accounts => {
            app.accounts.select_previous();
        }
        Page::Domains => {
            app.domains.select_previous();
        }
        Page::Favorites => {
            app.favorites.select_previous();
        }
        Page::DnsRecords { .. } => {
            app.dns_records.select_previous();
        }
        Page::Toolbox => {
            app.toolbox.prev_tab();
        }
        Page::Settings => {
            app.settings.select_previous();
        }
        Page::Home => {}
    }
}

fn handle_select_next(app: &mut App) {
    match &app.current_page {
        Page::Accounts => {
            app.accounts.select_next();
        }
        Page::Domains => {
            app.domains.select_next();
        }
        Page::Favorites => {
            app.favorites.select_next();
        }
        Page::DnsRecords { .. } => {
            app.dns_records.select_next();
        }
        Page::Toolbox => {
            app.toolbox.next_tab();
        }
        Page::Settings => {
            app.settings.select_next();
        }
        Page::Home => {}
    }
}

fn handle_select_first(app: &mut App) {
    match &app.current_page {
        Page::Accounts => {
            app.accounts.select_first();
        }
        Page::Domains => {
            app.domains.select_first();
        }
        Page::Favorites => {
            app.favorites.select_first();
        }
        Page::DnsRecords { .. } => {
            app.dns_records.select_first();
        }
        _ => {}
    }
}

fn handle_select_last(app: &mut App) {
    match &app.current_page {
        Page::Accounts => {
            app.accounts.select_last();
        }
        Page::Domains => {
            app.domains.select_last();
        }
        Page::Favorites => {
            app.favorites.select_last();
        }
        Page::DnsRecords { .. } => {
            app.dns_records.select_last();
        }
        _ => {}
    }
}

fn handle_confirm(app: &mut App) {
    match &app.current_page {
        Page::Accounts => {
            // 账号页面暂不支持进入详情
            if let Some(account) = app.accounts.selected_account() {
                app.set_status(format!("Selected: {}", account.name));
            }
        }
        Page::Domains => {
            // 进入 DNS 记录页面
            if let Some(domain) = app.domains.selected_domain() {
                let account_id = domain.account_id.clone();
                let domain_id = domain.id.clone();

                // 设置 DNS 记录页面的域名信息
                app.dns_records
                    .set_domain(account_id.clone(), domain_id.clone());
                app.dns_records.load_mock_data();

                // 切换页面
                app.current_page = Page::DnsRecords {
                    account_id,
                    domain_id,
                };
                app.clear_status(); // 切换页面时清除状态消息
            }
        }
        Page::Favorites => {
            // 进入 DNS 记录页面（同 Domains）
            if let Some(domain) = app.favorites.selected_domain() {
                let account_id = domain.account_id.clone();
                let domain_id = domain.id.clone();

                app.dns_records
                    .set_domain(account_id.clone(), domain_id.clone());
                app.dns_records.load_mock_data();

                app.current_page = Page::DnsRecords {
                    account_id,
                    domain_id,
                };
                app.clear_status();
            }
        }
        Page::DnsRecords { .. } => {
            // 编辑选中的 DNS 记录
            if let Some(record) = app.dns_records.selected_record() {
                app.set_status(format!("Edit record: {}", record.name));
                // TODO: 打开编辑弹窗
            }
        }
        _ => {}
    }
}

// ========== CRUD 操作处理 ==========

fn handle_add(app: &mut App) {
    match &app.current_page {
        Page::Accounts => {
            app.modal.show_add_account();
            app.set_status("Adding new account...");
        }
        Page::DnsRecords { .. } => {
            app.modal.show_add_dns_record();
            app.set_status("Adding new DNS record...");
        }
        _ => {
            app.set_status("Add not supported on this page");
        }
    }
}

fn handle_edit(app: &mut App) {
    match &app.current_page {
        Page::Accounts => {
            if let Some(account) = app.accounts.selected_account() {
                app.set_status(format!("Editing account: {}", account.name));
                // TODO: 打开编辑弹窗
            } else {
                app.set_status("No account selected");
            }
        }
        Page::DnsRecords { .. } => {
            if let Some(record) = app.dns_records.selected_record() {
                app.set_status(format!("Editing record: {}", record.name));
                // TODO: 打开编辑弹窗
            } else {
                app.set_status("No record selected");
            }
        }
        _ => {
            app.set_status("Edit not supported on this page");
        }
    }
}

fn handle_delete(app: &mut App) {
    match &app.current_page {
        Page::Accounts => {
            if let Some(account) = app.accounts.selected_account() {
                app.modal
                    .show_confirm_delete("account", &account.name, &account.id);
            } else {
                app.set_status("No account selected");
            }
        }
        Page::DnsRecords { .. } => {
            if let Some(record) = app.dns_records.selected_record() {
                app.modal
                    .show_confirm_delete("DNS record", &record.name, &record.id);
            } else {
                app.set_status("No record selected");
            }
        }
        _ => {
            app.set_status("Delete not supported on this page");
        }
    }
}

// ========== 导入导出处理 ==========

fn handle_import(app: &mut App) {
    match &app.current_page {
        Page::DnsRecords { .. } => {
            app.set_status("Import DNS records (not implemented)");
        }
        _ => {
            app.set_status("Import not supported on this page");
        }
    }
}

fn handle_export(app: &mut App) {
    match &app.current_page {
        Page::DnsRecords { .. } => {
            app.set_status("Export DNS records (not implemented)");
        }
        _ => {
            app.set_status("Export not supported on this page");
        }
    }
}

// ========== 工具箱处理 ==========

fn handle_switch_tab(app: &mut App) {
    if matches!(app.current_page, Page::Toolbox) {
        app.toolbox.next_tab();
        app.set_status(format!("Tool: {}", app.toolbox.current_tab.name()));
    }
}

fn handle_execute(app: &mut App) {
    use crate::model::state::ToolboxTab;

    if matches!(app.current_page, Page::Toolbox) {
        // 根据当前选中的工具打开对应弹窗
        match app.toolbox.current_tab {
            ToolboxTab::DnsLookup => {
                app.modal.show_dns_lookup();
            }
            ToolboxTab::Whois => {
                app.modal.show_whois_lookup();
            }
            ToolboxTab::SslCheck => {
                app.modal.show_ssl_check();
            }
            ToolboxTab::IpLookup => {
                app.modal.show_ip_lookup();
            }
            ToolboxTab::HttpHeaderCheck => {
                app.modal.show_http_header_check();
            }
            ToolboxTab::DnsPropagation => {
                app.modal.show_dns_propagation();
            }
            ToolboxTab::DnssecCheck => {
                app.modal.show_dnssec_check();
            }
        }
    }
}

// ========== 设置页面处理 ==========

fn handle_toggle_prev(app: &mut App) {
    if matches!(app.current_page, Page::Settings) {
        app.settings.toggle_prev();
        // 同步主题到 view 层（定义索引值 0=Dark, 1=Light）
        if app.settings.current_item() == Some(crate::model::state::SettingItem::Theme) {
            let theme_index = match app.settings.theme {
                crate::model::state::Theme::Dark => 0,
                crate::model::state::Theme::Light => 1,
            };
            crate::view::theme::set_theme_index(theme_index);
        }
    }
}

fn handle_toggle_next(app: &mut App) {
    if matches!(app.current_page, Page::Settings) {
        app.settings.toggle_next();
        // 同步主题到 view 层（定义索引值 0=Dark, 1=Light）
        if app.settings.current_item() == Some(crate::model::state::SettingItem::Theme) {
            let theme_index = match app.settings.theme {
                crate::model::state::Theme::Dark => 0,
                crate::model::state::Theme::Light => 1,
            };
            crate::view::theme::set_theme_index(theme_index);
        }
    }
}

// ========== 收藏操作处理 ==========

fn handle_toggle_favorite(app: &mut App) {
    match &app.current_page {
        Page::Domains => {
            if let Some(domain) = app.domains.domains.get_mut(app.domains.selected) {
                // 调用后端 toggle，用返回值同步本地状态
                let result = app.rt_handle.block_on(
                    app.core_service
                        .domain_metadata()
                        .toggle_favorite(&domain.account_id, &domain.id),
                );
                match result {
                    Ok(is_fav) => {
                        domain.is_favorite = is_fav;
                        let name = domain.name.clone();
                        if is_fav {
                            app.set_status(format!("★ {name}"));
                        } else {
                            app.set_status(format!("☆ {name}"));
                        }
                    }
                    Err(e) => {
                        app.set_status(format!("Error: {e}"));
                    }
                }
            }
        }
        Page::Favorites => {
            if let Some(fav_domain) = app.favorites.selected_domain() {
                let account_id = fav_domain.account_id.clone();
                let domain_id = fav_domain.id.clone();
                let name = fav_domain.name.clone();

                // 显式 unfavorite（而非 toggle），避免状态不一致时语义错误
                let update = DomainMetadataUpdate {
                    is_favorite: Some(false),
                    tags: None,
                    color: None,
                    note: None,
                };
                let result = app.rt_handle.block_on(
                    app.core_service
                        .domain_metadata()
                        .update_metadata(&account_id, &domain_id, update),
                );
                match result {
                    Ok(()) => {
                        // 同步本地状态
                        if let Some(domain) = app
                            .domains
                            .domains
                            .iter_mut()
                            .find(|d| d.id == domain_id && d.account_id == account_id)
                        {
                            domain.is_favorite = false;
                        }
                        app.favorites.rebuild(&app.domains.domains);
                        app.set_status(format!("☆ {name}"));
                    }
                    Err(e) => {
                        app.set_status(format!("Error: {e}"));
                    }
                }
            }
        }
        _ => {}
    }
}
