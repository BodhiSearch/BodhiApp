import * as React from 'react'
import { ThemeProvider as NextThemesProvider } from 'next-themes'
import { ThemeProviderProps } from 'next-themes/dist/types'
import { SidebarProvider } from '@/lib/hooks/use-sidebar'
import { TooltipProvider } from '@/components/ui/tooltip'
import { ChatHistoryProvider } from '@/lib/hooks/use-chat-history'
import { ChatSettingsProvider } from '@/lib/hooks/use-chat-settings'

export function Providers({ children, ...props }: ThemeProviderProps) {
  return (
    <NextThemesProvider {...props}>
      <ChatSettingsProvider>
        <ChatHistoryProvider>
          <SidebarProvider>
            <TooltipProvider>{children}</TooltipProvider>
          </SidebarProvider>
        </ChatHistoryProvider>
      </ChatSettingsProvider>
    </NextThemesProvider>
  )
}
