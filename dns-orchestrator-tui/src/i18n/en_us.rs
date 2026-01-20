//! 英文翻译 (en-US)

use super::keys::*;

pub const TRANSLATIONS: Translations = Translations {
    // ========================================================================
    // 通用文本
    // ========================================================================
    common: CommonTexts {
        app_name: "DNS Orchestrator TUI",
        // 操作动词
        add: "Add",
        edit: "Edit",
        delete: "Delete",
        cancel: "Cancel",
        save: "Save",
        confirm: "Confirm",
        close: "Close",
        search: "Search",
        query: "Query",
        check: "Check",
        lookup: "Lookup",
        quit: "Quit",
        // 状态词
        loading: "Loading...",
        no_data: "No data",
        result: "Result",
        error: "Error",
        // 是/否
        yes: "Yes",
        no: "No",
        // 导航词
        back: "Back",
        next: "Next",
        prev: "Prev",
    },

    // ========================================================================
    // 键盘提示
    // ========================================================================
    hints: HintTexts {
        keys: KeyNames {
            enter: "Enter",
            esc: "Esc",
            tab: "Tab",
            arrows_lr: "←→",
            arrows_ud: "↑↓",
            tab_arrows: "Tab/↑↓",
        },
        actions: ActionTexts {
            navigate: "Navigate",
            switch_option: "Switch",
            switch_panel: "Switch panel",
            move_up_down: "Move",
            change_method: "Change method",
            change_type: "Change type",
        },
    },

    // ========================================================================
    // 导航栏
    // ========================================================================
    nav: NavTexts {
        home: "Home",
        accounts: "Accounts",
        domains: "Domains",
        toolbox: "Toolbox",
        settings: "Settings",
    },

    // ========================================================================
    // 页面文本
    // ========================================================================
    home: HomeTexts {
        welcome: "Welcome to DNS Orchestrator",
        welcome_desc: "Manage your DNS records with ease",
        quick_actions: "Quick Actions",
        manage_domains: "Manage DNS Records",
        use_tools: "DNS/WHOIS/SSL Tools",
        manage_accounts: "Manage Accounts",
        configure_settings: "Configure Settings",
    },

    accounts: AccountsTexts {
        title: "Accounts",
        add_account: "Add Account",
        edit_account: "Edit Account",
        delete_account: "Delete Account",
        no_accounts: "No accounts",
        provider: "DNS Provider",
        account_name: "Account Name",
        account_name_optional: "Account Name (optional)",
        delete_confirm: "Are you sure to delete this account?",
    },

    domains: DomainsTexts {
        title: "Domains",
        no_domains: "No domains",
        record_count: "records",
        status_active: "Active",
        status_paused: "Paused",
        status_pending: "Pending",
        status_error: "Error",
    },

    dns_records: DnsRecordsTexts {
        title: "DNS Records",
        add_record: "Add Record",
        edit_record: "Edit Record",
        delete_record: "Delete Record",
        no_records: "No DNS records",
        name: "Name",
        value: "Value",
        ttl: "TTL",
        proxy: "Proxy",
    },

    toolbox: ToolboxTexts {
        title: "Toolbox",
        tabs: ToolboxTabTexts {
            whois: "WHOIS",
            dns_lookup: "DNS Lookup",
            ip_lookup: "IP Lookup",
            ssl_check: "SSL Check",
            http_headers: "HTTP Headers",
            dns_propagation: "DNS Propagation",
            dnssec_check: "DNSSEC Check",
        },
    },

    settings: SettingsTexts {
        title: "Settings",
        theme: ThemeTexts {
            label: "Theme",
            dark: "Dark",
            light: "Light",
            system: "System",
        },
        language: LanguageTexts {
            label: "Language",
            description: "Choose your preferred language",
        },
        pagination: PaginationTexts {
            label: "Pagination Mode",
            infinite_scroll: "Infinite Scroll",
            traditional: "Traditional",
        },
    },

    // ========================================================================
    // 弹窗文本
    // ========================================================================
    modal: ModalTexts {
        add_account: AddAccountModalTexts {
            title: "New Account",
            provider: "DNS Provider",
            provider_hint: "(←→ to Switch)",
            account_name: "Account Name (Optional)",
            account_name_example: "Example:",
            main_account: "Main Account",
            api_token: "API Token",
            api_token_hint: "Enter the Cloudflare API Token",
            accesskey_id: "AccessKey ID",
            accesskey_id_hint: "Enter the AccessKey ID",
            accesskey_secret: "AccessKey Secret",
            accesskey_secret_hint: "Enter the AccessKey Secret",
            secretid: "SecretID",
            secretid_hint: "Enter the SecretId",
            secretkey: "SecretKey",
            secretkey_hint: "Enter the SecretKey",
            access_key_id: "Access Key ID",
            access_key_id_hint: "Enter the Access Key ID",
            secret_access_key: "Secret Access Key",
            secret_access_key_hint: "Enter the Secret Access Key"
        },

        confirm_delete: ConfirmDeleteTexts {
            title: "Confirm Deletion",
            message: "Are you sure you want to delete",
            confirm_button: "Delete",
            cancel_button: "Cancel",
        },

        tools: ToolModalsTexts {
            titles: ToolModalTitles {
                dns_lookup: "DNS Lookup",
                whois: "WHOIS Lookup",
                ssl_check: "SSL Certificate Check",
                ip_lookup: "IP Lookup",
                http_header: "HTTP Header Check",
                dns_propagation: "DNS Propagation Check",
                dnssec: "DNSSEC Validation",
            },

            labels: ToolModalLabels {
                domain: "Domain:",
                record_type: "Record Type:",
                dns_server: "DNS Server:",
                url: "URL:",
                method: "Method:",
                ip_or_domain: "IP or Domain:",
            },

            placeholders: ToolModalPlaceholders {
                enter_domain: "Enter domain (e.g., example.com)",
                enter_ip: "Enter IP (e.g., 8.8.8.8)",
                enter_ip_or_domain: "Enter IP or domain",
                enter_url: "Enter URL (e.g., https://example.com)",
            },

            status: ToolModalStatus {
                querying: "Querying...",
                checking: "Checking...",
                checking_propagation: "Checking propagation...",
                checking_dnssec: "Checking DNSSEC...",
                looking_up: "Looking up...",
            },

            result_label: "Result:",
        },
    },

    // ========================================================================
    // 其他组件
    // ========================================================================
    status_bar: StatusBarTexts {
        ready: "Ready",
        loading: "Loading...",
        error: "Error",
    },

    help: HelpTexts {
        title: "Help",
        global_shortcuts: "Global shortcuts",
        operation_shortcuts: "Operation shortcuts",
        close_hint: "Press Esc to close the help",
        actions: HelpActionTexts {
            switch_panel: "Switch panel",
            move_up_down: "Move Up/Down",
            confirm: "Confirm",
            back_cancel: "Back/Cancel",
            quit: "Quit",
            add: "Add",
            edit: "Edit",
            delete: "Delete",
        },
    },
};