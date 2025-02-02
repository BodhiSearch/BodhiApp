'use client'

import { CodeBlock } from '@/components/ui/codeblock'
import * as hooks from '@/hooks/use-copy-to-clipboard'
import { fireEvent, render, screen } from '@testing-library/react'
import fs from 'fs'
import path from 'path'
import prettier from 'prettier'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { writeFileSync } from 'fs'

// Mock the copy-to-clipboard hook with a default implementation
vi.mock('@/hooks/use-copy-to-clipboard', () => ({
  useCopyToClipboard: () => ({
    isCopied: false,
    copyToClipboard: vi.fn()
  })
}));

vi.mock('@/components/ThemeProvider', () => ({
  useTheme: () => ({
    theme: 'dark'
  })
}));

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

async function writeFailedTestOutput(filename: string, content: string) {
  const outputPath = path.join(TEST_FILES_DIR, `${filename}.actual.html`)
  await fs.promises.writeFile(outputPath, content)
  console.log(`Wrote failed test output to: ${outputPath}`)
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

      try {
        expect(formattedActual).toBe(formattedExpected)
      } catch (error) {
        writeFailedTestOutput(file, formattedActual)
        throw error // Re-throw the error to fail the test
      }
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

  it('renders code block with correct language and content', () => {
    render(<CodeBlock language={mockLanguage} value={mockCode} />)

    expect(screen.getByText(mockLanguage)).toBeInTheDocument()
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
}) 