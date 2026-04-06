import JSON5 from 'json5'

import type {
  ProfileDocumentPayload,
  ProfileDocumentSummaryPayload,
} from '@/bridge/contracts/clotho'
import type {
  EditableProfileDocument,
  EditableProfileEnvironmentFile,
  EditableProfileInlineEnvironment,
  ProfileDocument,
  ProfileDocumentSummary,
} from './models'

export function normalizeProfileDocumentSummary(
  payload: ProfileDocumentSummaryPayload,
): ProfileDocumentSummary {
  return {
    profileId: normalizeString(payload.profileId),
    profilePath: normalizeString(payload.profilePath),
  }
}

export function normalizeProfileDocument(payload: ProfileDocumentPayload): ProfileDocument {
  return {
    profileId: normalizeString(payload.profileId),
    profilePath: normalizeString(payload.profilePath),
    contents: typeof payload.contents === 'string' ? payload.contents : '',
  }
}

export function compareProfileDocumentSummary(
  left: ProfileDocumentSummary,
  right: ProfileDocumentSummary,
): number {
  return left.profileId.localeCompare(right.profileId)
}

export function createEmptyEditableProfileDocument(): EditableProfileDocument {
  return {
    coreConfig: '{\n  \n}\n',
    envFiles: [],
    inlineEnvironment: [],
  }
}

export function parseEditableProfileDocument(contents: string): EditableProfileDocument {
  const parsed = parseProfileObject(contents)

  if ('core_config' in parsed) {
    const extraKeys = Object.keys(parsed).filter((key) => key !== 'core_config' && key !== 'environment')
    if (extraKeys.length > 0) {
      throw new Error(
        `Wrapper profile documents only support \`core_config\` and \`environment\`; found extra field(s): ${extraKeys.join(', ')}`,
      )
    }

    const coreConfig = expectObject(parsed.core_config, '`core_config`')
    const environment = parsed.environment === undefined ? {} : expectObject(parsed.environment, '`environment`')

    return {
      coreConfig: renderJsonc(coreConfig),
      envFiles: parseEnvironmentFiles(environment.env_files),
      inlineEnvironment: parseInlineEnvironment(environment.inline),
    }
  }

  return {
    coreConfig: renderJsonc(parsed),
    envFiles: [],
    inlineEnvironment: [],
  }
}

export function serializeEditableProfileDocument(document: EditableProfileDocument): string {
  const coreConfig = parseCoreConfig(document.coreConfig)
  const envFiles = serializeEnvironmentFiles(document.envFiles)
  const inlineEnvironment = serializeInlineEnvironment(document.inlineEnvironment)

  return `${JSON5.stringify(
    {
      core_config: coreConfig,
      environment: {
        env_files: envFiles,
        inline: inlineEnvironment,
      },
    },
    null,
    2,
  )}\n`
}

function normalizeString(value: string | null | undefined): string {
  return typeof value === 'string' ? value.trim() : ''
}

function parseProfileObject(contents: string): Record<string, unknown> {
  let parsed: unknown
  try {
    parsed = JSON5.parse(contents)
  } catch (error) {
    throw new Error(`Profile document must be valid JSONC: ${errorMessage(error)}`)
  }

  return expectObject(parsed, 'profile document')
}

function parseCoreConfig(contents: string): Record<string, unknown> {
  const trimmed = contents.trim()
  if (!trimmed) {
    throw new Error('`core_config` must not be empty')
  }

  let parsed: unknown
  try {
    parsed = JSON5.parse(contents)
  } catch (error) {
    throw new Error(`\`core_config\` must be valid JSONC: ${errorMessage(error)}`)
  }

  return expectObject(parsed, '`core_config`')
}

function parseEnvironmentFiles(value: unknown): EditableProfileEnvironmentFile[] {
  if (value === undefined) {
    return []
  }
  if (!Array.isArray(value)) {
    throw new Error('`environment.env_files` must be an array')
  }

  return value.map((entry, index) => {
    const object = expectObject(entry, `\`environment.env_files[${index}]\``)
    const path = object.path
    if (typeof path !== 'string') {
      throw new Error(`\`environment.env_files[${index}].path\` must be a string`)
    }

    return {
      path,
      required: typeof object.required === 'boolean' ? object.required : true,
    }
  })
}

function parseInlineEnvironment(value: unknown): EditableProfileInlineEnvironment[] {
  if (value === undefined) {
    return []
  }

  const object = expectObject(value, '`environment.inline`')
  return Object.entries(object).map(([key, entryValue]) => {
    if (typeof entryValue !== 'string') {
      throw new Error(`\`environment.inline.${key}\` must be a string`)
    }

    return {
      key,
      value: entryValue,
    }
  })
}

function serializeEnvironmentFiles(
  entries: EditableProfileEnvironmentFile[],
): Array<{ path: string; required: boolean }> {
  return entries.flatMap((entry, index) => {
    const path = entry.path.trim()
    if (!path) {
      return []
    }

    return [
      {
        path,
        required: entry.required,
      },
    ]
  })
}

function serializeInlineEnvironment(
  entries: EditableProfileInlineEnvironment[],
): Record<string, string> {
  const inline: Record<string, string> = {}

  for (const [index, entry] of entries.entries()) {
    const key = entry.key.trim()
    if (!key) {
      if (entry.value.length === 0) {
        continue
      }

      throw new Error(`Inline environment row ${index + 1} is missing a variable name`)
    }
    if (key in inline) {
      throw new Error(`Inline environment variable \`${key}\` is duplicated`)
    }

    inline[key] = entry.value
  }

  return inline
}

function renderJsonc(value: Record<string, unknown>): string {
  return `${JSON5.stringify(value, null, 2)}\n`
}

function expectObject(value: unknown, label: string): Record<string, unknown> {
  if (value === null || typeof value !== 'object' || Array.isArray(value)) {
    throw new Error(`${label} must be an object`)
  }

  return value as Record<string, unknown>
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error)
}
