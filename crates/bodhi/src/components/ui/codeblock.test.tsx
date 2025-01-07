'use client'

import { render, screen, fireEvent } from '@testing-library/react'
import { describe, expect, it, vi, beforeEach } from 'vitest'
import { CodeBlock } from './codeblock'
import * as hooks from '@/hooks/use-copy-to-clipboard'
import fs from 'fs'
import path from 'path'
import prettier from 'prettier'

// Mock the copy-to-clipboard hook with a default implementation
vi.mock('@/hooks/use-copy-to-clipboard', () => ({
  useCopyToClipboard: () => ({
    isCopied: false,
    copyToClipboard: vi.fn()
  })
}))

const TEST_FILES_DIR = path.join(__dirname, '__tests__')

// Helper function to read test files
function readTestFile(filename: string): string {
  return fs.readFileSync(path.join(TEST_FILES_DIR, filename), 'utf-8').trim()
}

// Helper function to format HTML using prettier
async function formatHTML(html: string): Promise<string> {
  return prettier.format(html, {
    parser: 'html',
    semi: true,
    singleQuote: true,
    tabWidth: 2,
    trailingComma: 'es5',
    endOfLine: 'lf',
    htmlWhitespaceSensitivity: 'ignore',
    printWidth: 80,
  })
}

describe('CodeBlock language rendering', () => {
  const testFiles = [
    { lang: 'javascript', file: 'input.js' },
    { lang: 'python', file: 'input.py' },
    { lang: 'java', file: 'Main.java' },
    { lang: 'typescript', file: 'input.ts' },
    { lang: 'go', file: 'input.go' },
    { lang: 'ruby', file: 'input.rb' },
  ]

  beforeEach(() => {
    vi.clearAllMocks()
  })

  testFiles.forEach(({ lang, file }) => {
    it(`renders ${lang} code correctly`, async () => {
      vi.spyOn(hooks, 'useCopyToClipboard').mockImplementation(() => ({
        isCopied: false,
        copyToClipboard: vi.fn()
      }))

      const input = readTestFile(file)
      const expected = readTestFile(`${file}.html`)

      const { container } = render(<CodeBlock language={lang} value={input} />)
      const syntaxHighlighter = container.querySelector('.syntax-highlighter')
      const actual = syntaxHighlighter?.innerHTML || ''

      // Format both expected and actual HTML through the same prettier config
      const formattedExpected = await formatHTML(expected)
      const formattedActual = await formatHTML(actual)

      // Log formatted HTML for easier debugging
      if (formattedExpected !== formattedActual) {
        console.log('Expected:\n', formattedExpected)
        console.log('\nActual:\n', formattedActual)
      }

      expect(formattedActual).toBe(formattedExpected)
    })
  })
})

describe('CodeBlock', () => {
  const mockCode = 'const hello = "world";'
  const mockLanguage = 'javascript'

  beforeEach(() => {
    // Reset all mocks before each test
    vi.clearAllMocks()

    // Mock the copy-to-clipboard hook implementation
    vi.spyOn(hooks, 'useCopyToClipboard').mockImplementation(() => ({
      isCopied: false,
      copyToClipboard: vi.fn()
    }))
  })

  it.skip('renders code block with correct language and content', () => {
    render(<CodeBlock language={mockLanguage} value={mockCode} />)

    expect(screen.getByText(mockLanguage)).toBeInTheDocument()
    expect(screen.getByText(mockCode)).toBeInTheDocument()
  })

  it('shows copy button and handles copy action', async () => {
    const mockCopyToClipboard = vi.fn()
    vi.spyOn(hooks, 'useCopyToClipboard').mockImplementation(() => ({
      isCopied: false,
      copyToClipboard: mockCopyToClipboard
    }))

    render(<CodeBlock language={mockLanguage} value={mockCode} />)

    const copyButton = screen.getByRole('button', { name: /copy code/i })
    fireEvent.click(copyButton)

    expect(mockCopyToClipboard).toHaveBeenCalledWith(mockCode)
  })

  it('shows check icon after copying', () => {
    vi.spyOn(hooks, 'useCopyToClipboard').mockImplementation(() => ({
      isCopied: true,
      copyToClipboard: vi.fn()
    }))

    render(<CodeBlock language={mockLanguage} value={mockCode} />)

    expect(screen.getByTestId('check-icon')).toBeInTheDocument()
  })

  it('handles download functionality', () => {
    // Mock window.prompt
    const mockPrompt = vi.fn().mockReturnValue('test.js')
    window.prompt = mockPrompt

    // Mock URL.createObjectURL and URL.revokeObjectURL
    const mockCreateObjectURL = vi.fn()
    const mockRevokeObjectURL = vi.fn()
    global.URL.createObjectURL = mockCreateObjectURL
    global.URL.revokeObjectURL = mockRevokeObjectURL

    render(<CodeBlock language={mockLanguage} value={mockCode} />)

    const downloadButton = screen.getByRole('button', { name: /download/i })
    fireEvent.click(downloadButton)

    expect(mockPrompt).toHaveBeenCalled()
    expect(mockCreateObjectURL).toHaveBeenCalled()
    expect(mockRevokeObjectURL).toHaveBeenCalled()
  })

  it('cancels download when prompt is cancelled', () => {
    // Mock window.prompt to return null (simulating cancel)
    window.prompt = vi.fn().mockReturnValue(null)

    render(<CodeBlock language={mockLanguage} value={mockCode} />)

    const downloadButton = screen.getByRole('button', { name: /download/i })
    fireEvent.click(downloadButton)

    // Verify that no download was initiated
    expect(document.querySelector('a')).not.toBeInTheDocument()
  })
}) 