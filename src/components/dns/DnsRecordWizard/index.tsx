import { useCallback, useState } from "react"
import { useTranslation } from "react-i18next"
import { Dialog, DialogContent, DialogHeader, DialogTitle } from "@/components/ui/dialog"
import { useDnsStore, useDomainStore } from "@/stores"
import type { CreateDnsRecordRequest, RecordData } from "@/types"
import { getMailPresetById, presetRecordToRecordData } from "./presets/mailPresets"
import { ConfirmStep } from "./steps/ConfirmStep"
import { MailPresetStep } from "./steps/MailPresetStep"
import { PurposeStep } from "./steps/PurposeStep"
import { SubdomainStep } from "./steps/SubdomainStep"
import { TargetStep } from "./steps/TargetStep"
import type { WizardPurpose, WizardState } from "./types"
import { getRecordTypeFromState, initialWizardState } from "./types"
import { WizardProgress } from "./WizardProgress"

interface DnsRecordWizardProps {
  accountId: string
  domainId: string
  onClose: () => void
  /** 当用户选择"手动配置邮件"时，回调打开高级表单 */
  onOpenAdvancedForm?: () => void
}

/** 各流程的步骤定义 */
const FLOW_STEPS: Record<WizardPurpose, string[]> = {
  website: ["dns.wizard.steps.subdomain", "dns.wizard.steps.target", "dns.wizard.steps.confirm"],
  mail: ["dns.wizard.steps.preset", "dns.wizard.steps.confirm"],
  subdomain: ["dns.wizard.steps.subdomain", "dns.wizard.steps.target", "dns.wizard.steps.confirm"],
  verification: [
    "dns.wizard.steps.subdomain",
    "dns.wizard.steps.target",
    "dns.wizard.steps.confirm",
  ],
}

/** 构建单条记录的 RecordData */
function buildRecordData(state: WizardState): RecordData | null {
  const recordType = getRecordTypeFromState(state)
  switch (recordType) {
    case "A":
      return { type: "A", content: { address: state.target } }
    case "AAAA":
      return { type: "AAAA", content: { address: state.target } }
    case "CNAME":
      return { type: "CNAME", content: { target: state.target } }
    case "TXT":
      return { type: "TXT", content: { text: state.target } }
    default:
      return null
  }
}

export function DnsRecordWizard({
  accountId,
  domainId,
  onClose,
  onOpenAdvancedForm,
}: DnsRecordWizardProps) {
  const { t } = useTranslation()
  const { createRecord, isLoading } = useDnsStore()

  // 获取当前域名
  const currentDomain = useDomainStore((state) => {
    const domains = state.getDomainsForAccount(accountId)
    return domains.find((d) => d.id === domainId) ?? null
  })
  const domainName = currentDomain?.name ?? ""

  // 向导状态
  const [state, setState] = useState<WizardState>(initialWizardState)
  const [step, setStep] = useState(0) // 0 = purpose selection, 1+ = flow steps

  // 更新状态
  const updateState = useCallback((updates: Partial<WizardState>) => {
    setState((prev) => ({ ...prev, ...updates }))
  }, [])

  // 获取当前流程的步骤
  const flowSteps = state.purpose ? FLOW_STEPS[state.purpose] : []
  const currentFlowStep = step - 1 // step 0 是选择目的，step 1 开始是流程步骤

  // 选择目的后进入流程
  const handleSelectPurpose = (purpose: WizardPurpose) => {
    updateState({ purpose })
    setStep(1)
  }

  // 下一步
  const handleNext = () => {
    setStep((prev) => prev + 1)
  }

  // 上一步
  const handleBack = () => {
    if (step === 1) {
      // 返回目的选择
      setState(initialWizardState)
      setStep(0)
    } else {
      setStep((prev) => prev - 1)
    }
  }

  // 构建请求并创建记录
  const handleSubmit = async () => {
    // 手动配置邮件 -> 打开高级表单
    if (state.purpose === "mail" && !state.mailPresetId) {
      onClose()
      onOpenAdvancedForm?.()
      return
    }

    // 邮件预设 -> 批量创建 MX 记录
    if (state.purpose === "mail" && state.mailPresetId) {
      await createMailPresetRecords()
      return
    }

    // 其他类型 -> 创建单条记录
    await createSingleRecord()
  }

  // 创建邮件预设记录
  const createMailPresetRecords = async () => {
    const preset = getMailPresetById(state.mailPresetId!)
    if (!preset) return

    for (const record of preset.records) {
      const data = presetRecordToRecordData(record, domainName)
      const request: CreateDnsRecordRequest = {
        domainId,
        name: record.name,
        ttl: record.ttl,
        data,
      }
      await createRecord(accountId, request)
    }
    onClose()
  }

  // 创建单条记录
  const createSingleRecord = async () => {
    const data = buildRecordData(state)
    if (!data) return

    const request: CreateDnsRecordRequest = {
      domainId,
      name: state.subdomain.trim() || "@",
      ttl: 600,
      data,
    }

    const result = await createRecord(accountId, request)
    if (result) {
      onClose()
    }
  }

  // 步骤组件 Props
  const stepProps = {
    state,
    onStateChange: updateState,
    onNext: handleNext,
    onBack: handleBack,
    domainName,
  }

  // 确认步骤的 Props
  const confirmProps = { ...stepProps, onSubmit: handleSubmit, isLoading }

  // 渲染标准流程步骤（subdomain -> target -> confirm）
  const renderStandardFlow = () => {
    if (currentFlowStep === 0) return <SubdomainStep {...stepProps} />
    if (currentFlowStep === 1) return <TargetStep {...stepProps} />
    return <ConfirmStep {...confirmProps} />
  }

  // 渲染邮件流程步骤（preset -> confirm）
  const renderMailFlow = () => {
    if (currentFlowStep === 0) return <MailPresetStep {...stepProps} />
    return <ConfirmStep {...confirmProps} />
  }

  // 渲染当前步骤
  const renderStep = () => {
    if (step === 0) {
      return <PurposeStep onSelect={handleSelectPurpose} />
    }

    if (state.purpose === "mail") {
      return renderMailFlow()
    }

    return renderStandardFlow()
  }

  return (
    <Dialog open onOpenChange={onClose}>
      <DialogContent className="sm:max-w-lg">
        <DialogHeader>
          <DialogTitle>{t("dns.wizard.title")}</DialogTitle>
        </DialogHeader>

        <div className="space-y-6 py-4">
          {/* 进度条（仅在选择目的后显示） */}
          {step > 0 && state.purpose && (
            <WizardProgress steps={flowSteps.map((key) => t(key))} currentStep={currentFlowStep} />
          )}

          {/* 步骤内容 */}
          {renderStep()}
        </div>
      </DialogContent>
    </Dialog>
  )
}
