import { GeistSans } from 'geist/font/sans'
import { GeistMono } from 'geist/font/mono'
import { cn } from '@/lib/utils'
import { Toaster } from '@/components/ui/sonner'
import { Providers } from '@/components/providers'
import { SidebarDesktop } from '@/components/sidebar-desktop'

interface LayoutProps {
  children: React.ReactNode
}

export default function Layout({ children }: LayoutProps) {
  return (
    <div
      className={cn(
        'font-sans antialiased',
        GeistSans.variable,
        GeistMono.variable
      )}
    >
      <Toaster position="top-center" />
      <Providers
        attribute="class"
        defaultTheme="system"
        enableSystem
        disableTransitionOnChange
      >
        <div className="flex flex-col min-h-screen">
          <main className="flex flex-col flex-1 bg-muted/50">
            <div className="relative flex h-[calc(100vh_-_theme(spacing.16))] overflow-hidden">
              <SidebarDesktop />
              {children}
            </div>
          </main>
        </div>
      </Providers>
    </div>
  )
}