import fs from 'fs'
import path from 'path'
import matter from 'gray-matter'
import '@/app/docs/prism-theme.css'
import { Menu } from "lucide-react"
import { Button } from "@/components/ui/button"
import { Sheet, SheetContent, SheetTrigger } from "@/components/ui/sheet"
import { Navigation } from "@/app/docs/Navigation"
import { getAllDocPaths } from "@/app/docs/utils"
import type { NavItem } from "@/app/docs/types"

function getDocTitle(filePath: string): string {
  try {
    const fullPath = path.join(process.cwd(), 'src/docs', `${filePath}.md`)
    const fileContents = fs.readFileSync(fullPath, 'utf8')
    const { data } = matter(fileContents)
    return data.title || getDefaultTitle(filePath)
  } catch (e) {
    console.error(`Error reading doc title for ${filePath}:`, e);
    return getDefaultTitle(filePath)
  }
}

function getDefaultTitle(filePath: string): string {
  return filePath.split('/').pop()?.replace(/-/g, ' ').replace(/\b\w/g, c => c.toUpperCase()) || 'Untitled'
}

function buildNavigation(): NavItem[] {
  const paths = getAllDocPaths()
  const nav: NavItem[] = []

  // Sort paths to ensure consistent ordering and group by folders
  paths.sort((a, b) => {
    const aParts = a.split('/')
    const bParts = b.split('/')

    // Sort by folder first
    if (aParts[0] !== bParts[0]) {
      return aParts[0].localeCompare(bParts[0])
    }

    // Then by depth (folders before files)
    if (aParts.length !== bParts.length) {
      return bParts.length - aParts.length
    }

    // Finally by name
    return a.localeCompare(b)
  }).forEach(path => {
    const parts = path.split('/')
    const title = getDocTitle(path)

    if (parts.length === 1) {
      nav.push({ title, slug: path })
    } else {
      // Handle nested paths
      let currentLevel = nav
      for (let i = 0; i < parts.length - 1; i++) {
        const parentSlug = parts.slice(0, i + 1).join('/')
        let parent = currentLevel.find(item => item.slug === parentSlug)
        if (!parent) {
          parent = {
            title: parts[i].replace(/-/g, ' ').replace(/\b\w/g, c => c.toUpperCase()),
            slug: parentSlug,
            children: []
          }
          currentLevel.push(parent)
        }
        parent.children = parent.children || []
        currentLevel = parent.children
      }
      currentLevel.push({
        title,
        slug: path
      })
    }
  })

  return nav;
}

export default function DocsLayout({
  children,
}: {
  children: React.ReactNode
}) {
  const navigation = buildNavigation()

  return (
    <div className="flex min-h-screen">
      {/* Desktop Sidebar */}
      <aside className="hidden lg:block w-80 shrink-0 border-r bg-background" aria-label="Documentation navigation">
        <div className="h-16 border-b px-6 flex items-center">
          <h1 className="text-lg font-semibold">Documentation</h1>
        </div>
        <Navigation items={navigation} />
      </aside>

      {/* Mobile Sidebar */}
      <Sheet>
        <SheetTrigger asChild>
          <Button
            variant="ghost"
            size="icon"
            className="lg:hidden fixed left-4 top-4 z-[50]"
            aria-label="Open documentation navigation"
          >
            <Menu className="h-4 w-4" />
          </Button>
        </SheetTrigger>
        <SheetContent side="left" className="w-80 p-0">
          <div className="h-16 border-b px-6 flex items-center">
            <h2 className="text-lg font-semibold">Documentation</h2>
          </div>
          <Navigation items={navigation} />
        </SheetContent>
      </Sheet>

      {/* Main content */}
      <main className="flex-1 min-w-0" role="main">
        <div className="h-16 border-b px-6 flex items-center lg:hidden">
          <h2 className="text-lg font-semibold ml-12">Documentation</h2>
        </div>
        <div className="container py-8 px-6">
          {children}
        </div>
      </main>
    </div>
  )
} 