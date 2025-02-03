import { getAllDocPaths } from '@/app/docs/utils'
import matter from 'gray-matter'
import fs from 'fs'
import path from 'path'
import Link from 'next/link'

function getDocDetails(filePath: string) {
  try {
    const fullPath = path.join(process.cwd(), 'src/docs', `${filePath}.md`)
    const fileContents = fs.readFileSync(fullPath, 'utf8')
    const { data } = matter(fileContents)
    return {
      title: data.title || filePath.split('/').pop()?.replace(/-/g, ' ').replace(/\b\w/g, c => c.toUpperCase()),
      description: data.description || '',
      slug: filePath
    }
  } catch (e) {
    console.error(`Error reading doc details for ${filePath}:`, e)
    return {
      title: filePath.split('/').pop()?.replace(/-/g, ' ').replace(/\b\w/g, c => c.toUpperCase()),
      description: '',
      slug: filePath
    }
  }
}

interface DocGroup {
  title: string
  items: {
    title: string
    description: string
    slug: string
  }[]
}

export default function DocsPage() {
  const paths = getAllDocPaths()
  const groups: { [key: string]: DocGroup } = {}

  paths.forEach(path => {
    const parts = path.split('/')
    const groupName = parts.length > 1 ? parts[0] : 'Getting Started'
    const details = getDocDetails(path)

    if (!groups[groupName]) {
      groups[groupName] = {
        title: groupName.replace(/-/g, ' ').replace(/\b\w/g, c => c.toUpperCase()),
        items: []
      }
    }

    groups[groupName].items.push(details)
  })

  return (
    <div className="max-w-none prose prose-slate dark:prose-invert">
      <h1>Documentation</h1>
      <p className="lead">
        Welcome to our documentation. Choose a topic below to get started.
      </p>

      {Object.entries(groups).map(([key, group]) => (
        <section key={key} className="mb-12">
          <h2 className="text-2xl font-bold mb-4">{group.title}</h2>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {group.items.map((doc) => (
              <Link
                key={doc.slug}
                href={`/docs/${doc.slug}`}
                className="block p-4 border rounded-lg hover:border-blue-500 transition-colors no-underline"
              >
                <h3 className="text-lg font-semibold mb-1 mt-0">{doc.title}</h3>
                {doc.description && (
                  <p className="text-sm text-gray-600 dark:text-gray-400 m-0">
                    {doc.description}
                  </p>
                )}
              </Link>
            ))}
          </div>
        </section>
      ))}
    </div>
  )
} 