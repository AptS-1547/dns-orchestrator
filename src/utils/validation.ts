import { NETWORK } from "@/constants"

interface PortValidationResult {
  valid: boolean
  portNum?: number
  isInvalid?: boolean
  isOutOfRange?: boolean
}

/**
 * 验证端口号
 * @param port 端口号字符串
 * @returns 验证结果对象，包含 valid 标志和解析后的端口号
 */
export function validatePort(port: string): PortValidationResult {
  const portNum = Number.parseInt(port, 10)

  if (!port || Number.isNaN(portNum)) {
    return { valid: false, isInvalid: true }
  }

  if (portNum < NETWORK.MIN_PORT || portNum > NETWORK.MAX_PORT) {
    return { valid: false, isOutOfRange: true }
  }

  return { valid: true, portNum }
}
