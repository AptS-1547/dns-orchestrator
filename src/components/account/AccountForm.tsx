import { Button } from "@/components/ui/button"
import {
  Dialog,
  DialogContent,
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
import { Eye, EyeOff, Loader2 } from "lucide-react"
import { useEffect, useState } from "react"
import { useTranslation } from "react-i18next"
import { ProviderIcon } from "./ProviderIcon"

interface AccountFormProps {
  open: boolean
  onOpenChange: (open: boolean) => void
}

export function AccountForm({ open, onOpenChange }: AccountFormProps) {
  const { t } = useTranslation()
  const { createAccount, isLoading, providers, fetchProviders } = useAccountStore()

  const [provider, setProvider] = useState<string>("")
  const [name, setName] = useState("")
  const [credentials, setCredentials] = useState<Record<string, string>>({})
  const [showPasswords, setShowPasswords] = useState<Record<string, boolean>>({})

  // 获取提供商列表
  useEffect(() => {
    if (providers.length === 0) {
      fetchProviders()
    }
  }, [providers.length, fetchProviders])

  // 默认选中第一个提供商
  useEffect(() => {
    if (providers.length > 0 && !provider) {
      setProvider(providers[0].id)
    }
  }, [providers, provider])

  const providerInfo = providers.find((p) => p.id === provider)

  const handleProviderChange = (value: string) => {
    setProvider(value)
    setCredentials({})
    setShowPasswords({})
  }

  const handleCredentialChange = (key: string, value: string) => {
    setCredentials((prev) => ({ ...prev, [key]: value }))
  }

  const togglePasswordVisibility = (key: string) => {
    setShowPasswords((prev) => ({ ...prev, [key]: !prev[key] }))
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()

    if (!providerInfo) return

    const result = await createAccount({
      name: name || `${providerInfo.name} 账号`,
      provider,
      credentials,
    })

    if (result) {
      // 重置表单
      setName("")
      setCredentials({})
      setShowPasswords({})
      onOpenChange(false)
    }
  }

  const isValid =
    providerInfo?.requiredFields.every((field) => credentials[field.key]?.trim()) ?? false

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>{t("account.addAccount")}</DialogTitle>
        </DialogHeader>

        <form onSubmit={handleSubmit} className="space-y-4">
          {/* 提供商选择 */}
          <div className="space-y-2">
            <Label>{t("account.provider")}</Label>
            <Select value={provider} onValueChange={handleProviderChange}>
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
              <Label htmlFor="name">{t("account.accountNameOptional")}</Label>
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
              <Label htmlFor={field.key}>{field.label}</Label>
              <div className="relative">
                <Input
                  id={field.key}
                  type={
                    field.type === "password" && !showPasswords[field.key] ? "password" : "text"
                  }
                  value={credentials[field.key] || ""}
                  onChange={(e) => handleCredentialChange(field.key, e.target.value)}
                  placeholder={field.placeholder}
                  className="pr-10"
                  required
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
              {field.helpText && <p className="text-muted-foreground text-xs">{field.helpText}</p>}
            </div>
          ))}

          <DialogFooter>
            <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>
              {t("common.cancel")}
            </Button>
            <Button type="submit" disabled={isLoading || !isValid}>
              {isLoading && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
              {t("common.add")}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
