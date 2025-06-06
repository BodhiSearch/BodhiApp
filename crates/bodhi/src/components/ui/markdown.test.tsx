

import { MemoizedReactMarkdown } from '@/components/ui/markdown'
import { render } from '@testing-library/react'
import fs from 'fs'
import { parse as parseHTML } from 'node-html-parser'
import path from 'path'
import prettier from 'prettier'
import remarkGfm from 'remark-gfm'
import { beforeEach, describe, expect, it } from 'vitest'

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

// Add this helper function at the top with other helpers
async function writeFailedTestOutput(filename: string, content: string) {
  const outputPath = path.join(TEST_FILES_DIR, `${filename}.actual.html`)
  await fs.promises.writeFile(outputPath, content)
  console.log(`Wrote failed test output to: ${outputPath}`)
}

it.skip('MemoizedReactMarkdown', () => {
  const testFiles = [
    { file: 'basic.md' },
    { file: 'lists.md' },
    { file: 'tables.md' },
    { file: 'code.md' },
    { file: 'tic-tac-toe.non-stream.txt' },
  ]

  beforeEach(() => {
    // Reset any mocks if needed
  })

  testFiles.forEach(({ file }) => {
    it(`renders ${file} markdown correctly in block mode`, async () => {
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

      const markdownContent = (container.firstChild as HTMLElement)?.innerHTML || ''

      // Format both expected and actual HTML through the same prettier config
      const formattedExpected = await formatHTML(expected)
      const formattedActual = await formatHTML(markdownContent)

      try {
        expect(formattedActual).toBe(formattedExpected)
      } catch (error) {
        await writeFailedTestOutput(file, formattedActual)
        throw error
      }
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

it.skip('MemoizedReactMarkdown Streaming', () => {
  const testFiles = [
    { file: 'basic.md' },
    { file: 'lists.md' },
    { file: 'tables.md' },
    { file: 'code.md' },
    { file: 'tic-tac-toe.non-stream.txt' },
  ]

  testFiles.forEach(({ file }) => {
    it(`renders ${file} markdown correctly in streaming mode`, async () => {
      const input = readTestFile(file)
      const words = input.match(/\S+|\s+/g) || []
      let accumulatedContent = ''

      for (const word of words) {
        accumulatedContent += word

        const { container } = render(
          <MemoizedReactMarkdown
            className="markdown-body"
            remarkPlugins={[remarkGfm]}
          >
            {accumulatedContent.trim()}
          </MemoizedReactMarkdown>
        )

        const markdownContent = (container.firstChild as HTMLElement)?.innerHTML || ''

        // Parse HTML with strict options
        const root = parseHTML(markdownContent, {
          comment: false,
          blockTextElements: {
            script: true,
            noscript: true,
            style: true,
            pre: true,
          },
          lowerCaseTagName: true,
          parseNoneClosedTags: false,  // Don't auto-close tags
          voidTag: {
            tags: ['area', 'base', 'br', 'col', 'embed', 'hr', 'img', 'input', 'link', 'meta', 'param', 'source', 'track', 'wbr']
          },
        })

        // Log the HTML content and structure for debugging
        if (!root || !root.firstChild) {
          console.log('Current accumulated content:', accumulatedContent)
          console.log('Generated HTML:', markdownContent)
          throw new Error('Invalid HTML structure - no root or first child')
        }

        try {
          // Check that all opening tags have matching closing tags
          const elements = root.querySelectorAll('*')
          elements.forEach(el => {
            const tagName = el.tagName.toLowerCase()
            const validTags = /^(h[1-6]|p|strong|em|code|pre|ul|ol|li|table|thead|tbody|tr|th|td|blockquote|a|br|hr)$/

            if (!validTags.test(tagName)) {
              console.log('Invalid tag found:', tagName)
              console.log('Current HTML:', markdownContent)
              console.log('Current accumulated content:', accumulatedContent)
              throw new Error(`Invalid HTML tag: ${tagName}`)
            }
          })

        } catch (error) {
          console.log('Error in word:', word)
          console.log('Accumulated content:', accumulatedContent)
          console.log('Current HTML state:', markdownContent)
          throw error
        }
      }
    })
  })

  it('handles empty lines in streaming content', () => {
    const { container } = render(
      <MemoizedReactMarkdown
        className="markdown-body"
        remarkPlugins={[remarkGfm]}
      >
        {'\n'}
      </MemoizedReactMarkdown>
    )

    expect(container.firstChild).toBeTruthy()
  })
}) 