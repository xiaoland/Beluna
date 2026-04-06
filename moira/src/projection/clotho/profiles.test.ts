import { describe, expect, it } from 'vitest'

import {
  createEmptyEditableProfileDocument,
  parseEditableProfileDocument,
  serializeEditableProfileDocument,
} from './profiles'

describe('clotho profile projection', () => {
  it('parses wrapper profile documents into structured editor state', () => {
    const editable = parseEditableProfileDocument(`{
      core_config: {
        logging: {
          dir: './logs',
        },
      },
      environment: {
        env_files: [
          { path: './local.env' },
          { path: './optional.env', required: false },
        ],
        inline: {
          OPENAI_API_KEY: 'openai',
          BAILIAN_API_KEY: 'bailian',
        },
      },
    }`)

    expect(editable.coreConfig).toContain("dir: './logs'")
    expect(editable.envFiles).toEqual([
      { path: './local.env', required: true },
      { path: './optional.env', required: false },
    ])
    expect(editable.inlineEnvironment).toEqual([
      { key: 'OPENAI_API_KEY', value: 'openai' },
      { key: 'BAILIAN_API_KEY', value: 'bailian' },
    ])
  })

  it('treats legacy plain config documents as core_config-only drafts', () => {
    const editable = parseEditableProfileDocument(`{
      logging: {
        dir: './logs',
      },
    }`)

    expect(editable.coreConfig).toContain("dir: './logs'")
    expect(editable.envFiles).toEqual([])
    expect(editable.inlineEnvironment).toEqual([])
  })

  it('serializes structured editor state back into a wrapper profile document', () => {
    const contents = serializeEditableProfileDocument({
      coreConfig: "{ logging: { dir: './logs' } }\n",
      envFiles: [
        { path: './local.env', required: true },
        { path: '   ', required: false },
      ],
      inlineEnvironment: [
        { key: 'OPENAI_API_KEY', value: 'openai' },
        { key: '', value: '' },
      ],
    })

    expect(contents).toContain('core_config:')
    expect(contents).toContain("path: './local.env'")
    expect(contents).toContain("OPENAI_API_KEY: 'openai'")
    expect(contents).not.toContain("path: '   '")
  })

  it('provides a wrapper-ready empty draft', () => {
    expect(createEmptyEditableProfileDocument()).toEqual({
      coreConfig: '{\n  \n}\n',
      envFiles: [],
      inlineEnvironment: [],
    })
  })
})
