import { DOMAIN_COLOR_ORDER, DOMAIN_COLORS, type DomainColorValue } from "@/constants/colors"
import { cn } from "@/lib/utils"
import { useSettingsStore } from "@/stores"

interface DomainColorPickerProps {
  value: DomainColorValue
  onChange: (color: DomainColorValue) => void
}

export function DomainColorPicker({ value, onChange }: DomainColorPickerProps) {
  const theme = useSettingsStore((state) => state.theme)
  const isDark =
    theme === "dark" ||
    (theme === "system" && window.matchMedia("(prefers-color-scheme: dark)").matches)

  const handleColorClick = (colorKey: DomainColorValue) => {
    // 如果点击的是已选中的颜色，则取消选择（设置为 "none"）
    if (value === colorKey) {
      onChange("none")
    } else {
      onChange(colorKey)
    }
  }

  return (
    <div className="grid grid-cols-5 gap-2">
      {DOMAIN_COLOR_ORDER.map((colorKey) => {
        const color = DOMAIN_COLORS[colorKey]
        const hex = isDark ? color.darkHex : color.hex
        const isSelected = value === colorKey

        return (
          <button
            key={colorKey}
            type="button"
            onClick={() => handleColorClick(colorKey)}
            className={cn(
              "h-10 w-10 rounded-md border-2 transition-all",
              isSelected
                ? "border-primary ring-2 ring-primary ring-offset-2 ring-offset-background"
                : "border-transparent hover:border-muted-foreground/20"
            )}
            style={{ backgroundColor: hex }}
            title={color.name}
            aria-label={color.name}
          />
        )
      })}
    </div>
  )
}
