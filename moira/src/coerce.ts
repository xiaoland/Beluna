import type { RawEvent } from './projection/lachesis/models'

export type JsonRecord = Record<string, unknown>

export function toArray(value: unknown): unknown[] {
  return Array.isArray(value) ? value : []
}

export function toRecord(value: unknown): JsonRecord {
  return value && typeof value === 'object' && !Array.isArray(value) ? (value as JsonRecord) : {}
}

export function read(record: JsonRecord, keys: string[]): unknown {
  for (const key of keys) {
    if (key in record) {
      return record[key]
    }
  }

  return undefined
}

export function readArray(record: JsonRecord, keys: string[]): unknown[] {
  const value = read(record, keys)
  return Array.isArray(value) ? value : []
}

export function readNumber(record: JsonRecord, keys: string[]): number | null {
  const value = read(record, keys)

  if (typeof value === 'number' && Number.isFinite(value)) {
    return value
  }

  if (typeof value === 'string') {
    const parsed = Number(value)
    return Number.isFinite(parsed) ? parsed : null
  }

  return null
}

export function readString(record: JsonRecord, keys: string[]): string | null {
  return stringify(read(record, keys))
}

export function stringify(value: unknown): string | null {
  if (typeof value === 'string') {
    return value
  }

  if (typeof value === 'number' || typeof value === 'boolean') {
    return String(value)
  }

  return null
}

export function parseMaybeJson(value: unknown): unknown {
  if (typeof value !== 'string') {
    return value
  }

  const trimmed = value.trim()
  if (!trimmed) {
    return null
  }

  try {
    return JSON.parse(trimmed)
  } catch {
    return value
  }
}

export function compareDateDesc(left: string | null, right: string | null): number {
  const leftMs = left ? Date.parse(left) : 0
  const rightMs = right ? Date.parse(right) : 0
  return rightMs - leftMs
}

export function cryptoRandomId(): string {
  if (typeof crypto !== 'undefined' && typeof crypto.randomUUID === 'function') {
    return crypto.randomUUID()
  }

  return `raw-${Math.random().toString(16).slice(2)}`
}

export function matchesKeywords(event: RawEvent, keywords: string[]): boolean {
  const haystack = [
    event.subsystem,
    event.family,
    event.target,
    event.messageText,
  ]
    .filter(Boolean)
    .join(' ')
    .toLowerCase()

  return keywords.every((keyword) => haystack.includes(keyword.toLowerCase()))
}
