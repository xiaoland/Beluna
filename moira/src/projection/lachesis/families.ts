const CORTEX_ORGAN_FAMILIES = [
  'cortex.primary',
  'cortex.sense-helper',
  'cortex.goal-forest-helper',
  'cortex.acts-helper',
] as const

const AI_TRANSPORT_FAMILIES = ['ai-gateway.request'] as const
const AI_CHAT_FAMILIES = ['ai-gateway.chat.turn', 'ai-gateway.chat.thread'] as const

export function isCortexOrganFamily(family: string | null): boolean {
  return !!family && (CORTEX_ORGAN_FAMILIES as readonly string[]).includes(family)
}

export function isAiTransportFamily(family: string | null): boolean {
  return !!family && (AI_TRANSPORT_FAMILIES as readonly string[]).includes(family)
}

export function isAiChatFamily(family: string | null): boolean {
  return !!family && (AI_CHAT_FAMILIES as readonly string[]).includes(family)
}

export function isAiFamily(family: string | null): boolean {
  return isAiTransportFamily(family) || isAiChatFamily(family)
}
