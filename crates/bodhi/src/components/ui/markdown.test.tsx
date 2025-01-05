'use client'

import { render } from '@testing-library/react'
import { describe, expect, it, beforeEach } from 'vitest'
import { MemoizedReactMarkdown } from './markdown'
import fs from 'fs'
import path from 'path'
import prettier from 'prettier'
import remarkGfm from 'remark-gfm'

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

describe('MemoizedReactMarkdown', () => {
  const testFiles = [
    { file: 'basic.md' },
    { file: 'lists.md' },
    { file: 'tables.md' },
    { file: 'code.md' },
  ]

  beforeEach(() => {
    // Reset any mocks if needed
  })

  testFiles.forEach(({ file }) => {
    it(`renders ${file} markdown correctly`, async () => {
      const input = readTestFile(file)
      const expected = readTestFile(`${file}.html`)

      const { container } = render(
        <MemoizedReactMarkdown 
          className="markdown-body"
          remarkPlugins={[remarkGfm]}
        >
          {input}
        </MemoizedReactMarkdown>
      )

      const markdownContent = container.firstChild?.innerHTML || ''

      // Format both expected and actual HTML through the same prettier config
      const formattedExpected = await formatHTML(expected)
      const formattedActual = await formatHTML(markdownContent)

      // Log formatted HTML for easier debugging
      if (formattedExpected !== formattedActual) {
        console.log('Expected:\n', formattedExpected)
        console.log('\nActual:\n', formattedActual)
      }

      expect(formattedActual).toBe(formattedExpected)
    })
  })

  it('memoizes correctly with same props', () => {
    const content = '# Hello World'
    const className = 'markdown-body'

    const { rerender, container } = render(
      <MemoizedReactMarkdown className={className}>
        {content}
      </MemoizedReactMarkdown>
    )

    const firstRender = container.innerHTML

    // Rerender with same props
    rerender(
      <MemoizedReactMarkdown className={className}>
        {content}
      </MemoizedReactMarkdown>
    )

    const secondRender = container.innerHTML

    expect(firstRender).toBe(secondRender)
  })

  it('re-renders when content changes', () => {
    const className = 'markdown-body'
    const initialContent = '# Hello World'
    const newContent = '# Hello Universe'

    const { rerender, container } = render(
      <MemoizedReactMarkdown className={className}>
        {initialContent}
      </MemoizedReactMarkdown>
    )

    const firstRender = container.innerHTML

    // Rerender with different content
    rerender(
      <MemoizedReactMarkdown className={className}>
        {newContent}
      </MemoizedReactMarkdown>
    )

    const secondRender = container.innerHTML

    expect(firstRender).not.toBe(secondRender)
  })
}) 