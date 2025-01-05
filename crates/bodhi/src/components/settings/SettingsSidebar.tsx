'use client';

import { useState } from 'react';
import { AliasSelector } from '@/components/settings/AliasSelector';
import { SystemPrompt } from '@/components/settings/SystemPrompt';
import { StopWords } from '@/components/settings/StopWords';
import { TokenSlider } from '@/components/settings/TokenSlider';
import {
  SidebarContent,
  SidebarHeader,
  Sidebar,
  SidebarGroup,
} from '@/components/ui/sidebar';

export function SettingsSidebar() {
  const [isLoading, setIsLoading] = useState(false);

  return (
    <Sidebar side="right" variant="floating">
      <SidebarHeader>
        <h2 className="text-lg font-semibold">Settings</h2>
      </SidebarHeader>
      <SidebarContent>
        <SidebarGroup>
          <AliasSelector
            isLoadingCallback={setIsLoading}
          />
          <SystemPrompt
            isLoading={isLoading}
            initialEnabled={true}
          />
          <StopWords
            isLoading={isLoading}
            initialEnabled={true}
          />
          <TokenSlider
            isLoading={isLoading}
            initialEnabled={true}
          />
        </SidebarGroup>
      </SidebarContent>
    </Sidebar>
  );
}
