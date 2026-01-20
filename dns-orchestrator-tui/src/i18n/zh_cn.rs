//! 中文翻译 (zh-CN)

use super::keys::*;

pub const TRANSLATIONS: Translations = Translations {
    // ========================================================================
    // 通用文本
    // ========================================================================
    common: CommonTexts {
        app_name: "DNS Orchestrator TUI",
        // 操作动词
        add: "添加",
        edit: "编辑",
        delete: "删除",
        cancel: "取消",
        save: "保存",
        confirm: "确认",
        close: "关闭",
        search: "搜索",
        query: "查询",
        check: "检查",
        lookup: "查询",
        quit: "退出",
        // 状态词
        loading: "加载中...",
        no_data: "暂无数据",
        result: "结果",
        error: "错误",
        // 是/否
        yes: "是",
        no: "否",
        // 导航词
        back: "返回",
        next: "下一步",
        prev: "上一步",
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
            navigate: "导航",
            switch_option: "切换选项",
            switch_panel: "切换面板",
            move_up_down: "上下移动",
            change_method: "切换方法",
            change_type: "切换类型",
        },
    },

    // ========================================================================
    // 导航栏
    // ========================================================================
    nav: NavTexts {
        home: "主页",
        accounts: "账号",
        domains: "域名",
        toolbox: "工具箱",
        settings: "设置",
    },

    // ========================================================================
    // 页面文本
    // ========================================================================
    home: HomeTexts {
        welcome: "欢迎使用 DNS Orchestrator",
        welcome_desc: "轻松管理你的 DNS 记录",
        quick_actions: "快捷操作",
        manage_domains: "管理 DNS 记录",
        use_tools: "DNS/WHOIS/SSL 工具",
        manage_accounts: "管理账号",
        configure_settings: "配置设置",
    },

    accounts: AccountsTexts {
        title: "账号",
        add_account: "添加账号",
        edit_account: "编辑账号",
        delete_account: "删除账号",
        no_accounts: "暂无账号",
        provider: "DNS 提供商",
        account_name: "账号名称",
        account_name_optional: "账号名称（可选）",
        delete_confirm: "确定要删除这个账号吗？",
    },

    domains: DomainsTexts {
        title: "域名",
        no_domains: "暂无域名",
        record_count: "条记录",
        status_active: "活跃",
        status_paused: "暂停",
        status_pending: "待验证",
        status_error: "异常",
    },

    dns_records: DnsRecordsTexts {
        title: "DNS 记录",
        add_record: "添加记录",
        edit_record: "编辑记录",
        delete_record: "删除记录",
        no_records: "暂无 DNS 记录",
        name: "名称",
        value: "值",
        ttl: "TTL",
        proxy: "代理",
    },

    toolbox: ToolboxTexts {
        title: "工具箱",
        tabs: ToolboxTabTexts {
            whois: "WHOIS",
            dns_lookup: "DNS 查询",
            ip_lookup: "IP 查询",
            ssl_check: "SSL 检查",
            http_headers: "HTTP 头",
            dns_propagation: "DNS 传播",
            dnssec_check: "DNSSEC",
        },
    },

    settings: SettingsTexts {
        title: "设置",
        theme: ThemeTexts {
            label: "主题",
            dark: "深色",
            light: "浅色",
            system: "跟随系统",
        },
        language: LanguageTexts {
            label: "语言",
            description: "选择你偏好的语言",
        },
        pagination: PaginationTexts {
            label: "分页模式",
            infinite_scroll: "无限滚动",
            traditional: "传统分页",
        },
    },

    // ========================================================================
    // 弹窗文本
    // ========================================================================
    modal: ModalTexts {
        add_account: AddAccountModalTexts {
            title: "新建账号",
            provider: "DNS 提供商",
            provider_hint: "(←→ 切换)",
            account_name: "账号名称（可选）",
            account_name_example: "示例：",
            main_account: "主账号",
            api_token: "API Token",
            api_token_hint: "请输入 Cloudflare API Token",
            accesskey_id: "AccessKey ID",
            accesskey_id_hint: "请输入 AccessKey ID",
            accesskey_secret: "AccessKey Secret",
            accesskey_secret_hint: "请输入 AccessKey Secret",
            secretid: "SecretID",
            secretid_hint: "请输入 SecretID",
            secretkey: "SecretKey",
            secretkey_hint: "请输入 SecretKey",
            access_key_id: "访问密钥",
            access_key_id_hint: "请输入访问密钥",
            secret_access_key: "秘密访问密钥",
            secret_access_key_hint: "请输入秘密访问密钥"
        },

        confirm_delete: ConfirmDeleteTexts {
            title: "确认删除",
            message: "确定要删除吗",
            confirm_button: "删除",
            cancel_button: "取消",
        },

        tools: ToolModalsTexts {
            titles: ToolModalTitles {
                dns_lookup: "DNS 查询",
                whois: "WHOIS 查询",
                ssl_check: "SSL 证书检查",
                ip_lookup: "IP 地理位置",
                http_header: "HTTP 头检查",
                dns_propagation: "DNS 传播检查",
                dnssec: "DNSSEC 验证",
            },

            labels: ToolModalLabels {
                domain: "域名：",
                record_type: "记录类型：",
                dns_server: "DNS 服务器：",
                url: "URL：",
                method: "请求方法：",
                ip_or_domain: "IP 或域名：",
            },

            placeholders: ToolModalPlaceholders {
                enter_domain: "输入域名（如 example.com）",
                enter_ip: "输入 IP（如 8.8.8.8）",
                enter_ip_or_domain: "输入 IP 或域名",
                enter_url: "输入 URL（如 https://example.com）",
            },

            status: ToolModalStatus {
                querying: "查询中...",
                checking: "检查中...",
                checking_propagation: "检查传播中...",
                checking_dnssec: "检查 DNSSEC...",
                looking_up: "查询中...",
            },

            result_label: "结果：",
        },
    },

    // ========================================================================
    // 其他组件
    // ========================================================================
    status_bar: StatusBarTexts {
        ready: "就绪",
        loading: "加载中...",
        error: "错误",
    },

    help: HelpTexts {
        title: "帮助",
        global_shortcuts: "全局快捷键",
        operation_shortcuts: "操作快捷键",
        close_hint: "按 Esc 关闭帮助",
        actions: HelpActionTexts {
            switch_panel: "切换面板",
            move_up_down: "上/下移动",
            confirm: "确认",
            back_cancel: "返回/取消",
            quit: "退出",
            add: "添加",
            edit: "编辑",
            delete: "删除",
        },
    },
};