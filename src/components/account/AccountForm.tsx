import { Eye, EyeOff, Loader2 } from "lucide-react"
import { useEffect, useState } from "react"
import { useTranslation } from "react-i18next"
import { Button } from "@/components/ui/button"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import { useAccountStore } from "@/stores"
import type { Account, ProviderCredentials } from "@/types"
import { ProviderIcon } from "./ProviderIcon"

/**
 * 构建结构化的 Provider 凭证对象（v1.7.0 类型安全重构）
 * @param provider Provider 类型
 * @param credentialsMap 凭证键值对（使用驼峰命名，与 ProviderMetadata 一致）
 * @returns 结构化的 ProviderCredentials 对象
 */
function buildProviderCredentials(
  provider: string,
  credentialsMap: Record<string, string>
): ProviderCredentials {
  switch (provider) {
    case "cloudflare":
      return {
        provider: "cloudflare",
        credentials: {
          api_token: credentialsMap.apiToken,
        },
      }
    case "aliyun":
      return {
        provider: "aliyun",
        credentials: {
          access_key_id: credentialsMap.accessKeyId,
          access_key_secret: credentialsMap.accessKeySecret,
        },
      }
    case "dnspod":
      return {
        provider: "dnspod",
        credentials: {
          secret_id: credentialsMap.secretId,
          secret_key: credentialsMap.secretKey,
        },
      }
    case "huaweicloud":
      return {
        provider: "huaweicloud",
        credentials: {
          access_key_id: credentialsMap.accessKeyId,
          secret_access_key: credentialsMap.secretAccessKey,
        },
      }
    default:
      throw new Error(`Unknown provider: ${provider}`)
  }
}

interface AccountFormProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  account?: Account // 编辑模式时传入
}

export function AccountForm({ open, onOpenChange, account }: AccountFormProps) {
  const { t } = useTranslation()
  const {
    createAccount,
    updateAccount,
    isLoading,
    isUpdating,
    providers,
    fetchProviders,
    fieldErrors,
    clearFieldErrors,
  } = useAccountStore()

  const isEditing = !!account

  const [provider, setProvider] = useState<string>("")
  const [name, setName] = useState("")
  const [credentials, setCredentials] = useState<Record<string, string>>({})
  const [showPasswords, setShowPasswords] = useState<Record<string, boolean>>({})

  // 获取提供商列表 + 默认选中第一个
  useEffect(() => {
    if (providers.length === 0) {
      fetchProviders()
    } else if (!(provider || isEditing)) {
      setProvider(providers[0].id)
    }
  }, [providers, provider, fetchProviders, isEditing])

  // 编辑模式：预填充表单
  useEffect(() => {
    if (open && account) {
      setProvider(account.provider)
      setName(account.name)
      setCredentials({}) // 凭证不回显
      setShowPasswords({})
    } else if (open && !account) {
      // 创建模式：重置表单
      setName("")
      setCredentials({})
      setShowPasswords({})
      if (providers.length > 0) {
        setProvider(providers[0].id)
      }
    }
  }, [open, account, providers])

  const providerInfo = providers.find((p) => p.id === provider)

  const handleProviderChange = (value: string) => {
    setProvider(value)
    setCredentials({})
    setShowPasswords({})
  }

  const handleCredentialChange = (key: string, value: string) => {
    setCredentials((prev) => ({ ...prev, [key]: value }))
    // 用户输入时清除该字段的错误
    if (fieldErrors[key]) {
      clearFieldErrors()
    }
  }

  const togglePasswordVisibility = (key: string) => {
    setShowPasswords((prev) => ({ ...prev, [key]: !prev[key] }))
  }

  /**
   * 处理创建账户
   */
  const handleCreate = async () => {
    if (!providerInfo) return

    const providerCredentials = buildProviderCredentials(provider, credentials)
    const result = await createAccount({
      name: name || `${providerInfo.name} 账号`,
      provider,
      credentials: providerCredentials,
    })

    if (result) {
      setName("")
      setCredentials({})
      setShowPasswords({})
      onOpenChange(false)
    }
  }

  /**
   * 处理更新账户
   */
  const handleUpdate = async () => {
    if (!account) return

    const hasCredentials = Object.values(credentials).some((v) => v.trim())
    const result = await updateAccount({
      id: account.id,
      name: name || undefined,
      credentials: hasCredentials ? buildProviderCredentials(provider, credentials) : undefined,
    })

    if (result) {
      onOpenChange(false)
    }
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()

    if (isEditing) {
      await handleUpdate()
    } else {
      await handleCreate()
    }
  }

  // 创建模式：所有必填字段都要填写
  // 编辑模式：至少修改了名称或凭证中的任意一个
  const isValidForCreate =
    providerInfo?.requiredFields.every((field) => credentials[field.key]?.trim()) ?? false
  const isValidForEdit = name !== account?.name || Object.values(credentials).some((v) => v.trim())
  const isValid = isEditing ? isValidForEdit : isValidForCreate

  const handleOpenChange = (isOpen: boolean) => {
    if (!isOpen) {
      clearFieldErrors()
    }
    onOpenChange(isOpen)
  }

  const isSubmitting = isLoading || isUpdating

  return (
    <Dialog open={open} onOpenChange={handleOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>
            {isEditing ? t("account.editAccount") : t("account.addAccount")}
          </DialogTitle>
          {isEditing && <DialogDescription>{t("account.editAccountDesc")}</DialogDescription>}
        </DialogHeader>

        <form onSubmit={handleSubmit} className="space-y-4">
          {/* 提供商选择 */}
          <div className="space-y-2">
            <Label>{t("account.provider")}</Label>
            <Select value={provider} onValueChange={handleProviderChange} disabled={isEditing}>
              <SelectTrigger>
                <SelectValue
                  placeholder={
                    providers.length === 0 ? t("common.loading") : t("account.selectProvider")
                  }
                />
              </SelectTrigger>
              <SelectContent>
                {providers.map((p) => (
                  <SelectItem key={p.id} value={p.id}>
                    <div className="flex items-center gap-2">
                      <ProviderIcon provider={p.id} className="h-4 w-4" />
                      <span>{p.name}</span>
                    </div>
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            {providerInfo && (
              <p className="text-muted-foreground text-xs">{providerInfo.description}</p>
            )}
          </div>

          {/* 账号名称 */}
          {providerInfo && (
            <div className="space-y-2">
              <Label htmlFor="name">
                {isEditing ? t("account.accountName") : t("account.accountNameOptional")}
              </Label>
              <Input
                id="name"
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder={t("account.accountNamePlaceholder", { provider: providerInfo.name })}
              />
            </div>
          )}

          {/* 凭证字段 */}
          {providerInfo?.requiredFields.map((field) => (
            <div key={field.key} className="space-y-2">
              <Label htmlFor={field.key}>
                {field.label}
                {isEditing && (
                  <span className="ml-1 font-normal text-muted-foreground">
                    ({t("account.leaveEmptyToKeep")})
                  </span>
                )}
              </Label>
              <div className="relative">
                <Input
                  id={field.key}
                  type={
                    field.type === "password" && !showPasswords[field.key] ? "password" : "text"
                  }
                  value={credentials[field.key] || ""}
                  onChange={(e) => handleCredentialChange(field.key, e.target.value)}
                  placeholder={isEditing ? t("account.enterNewValue") : field.placeholder}
                  className={`pr-10 ${fieldErrors[field.key] ? "border-destructive" : ""}`}
                  required={!isEditing}
                />
                {field.type === "password" && (
                  <Button
                    type="button"
                    variant="ghost"
                    size="icon"
                    className="absolute top-0 right-0 h-full px-3"
                    onClick={() => togglePasswordVisibility(field.key)}
                  >
                    {showPasswords[field.key] ? (
                      <EyeOff className="h-4 w-4" />
                    ) : (
                      <Eye className="h-4 w-4" />
                    )}
                  </Button>
                )}
              </div>
              {fieldErrors[field.key] && (
                <p className="text-destructive text-xs">{fieldErrors[field.key]}</p>
              )}
              {field.helpText && !fieldErrors[field.key] && (
                <p className="text-muted-foreground text-xs">{field.helpText}</p>
              )}
            </div>
          ))}

          <DialogFooter>
            <Button type="button" variant="outline" onClick={() => handleOpenChange(false)}>
              {t("common.cancel")}
            </Button>
            <Button type="submit" disabled={isSubmitting || !isValid}>
              {isSubmitting && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
              {isEditing ? t("common.save") : t("common.add")}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
