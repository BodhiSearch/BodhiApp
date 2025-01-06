'use client';

import { useState } from 'react';
import { AliasSelector } from '@/components/settings/AliasSelector';
import { SystemPrompt } from '@/components/settings/SystemPrompt';
import { StopWords } from '@/components/settings/StopWords';
import { TokenSlider } from '@/components/settings/TokenSlider';
import { useModels } from '@/hooks/useQuery';
import {
  SidebarContent,
  SidebarHeader,
  Sidebar,
  SidebarGroup,
} from '@/components/ui/sidebar';

export function SettingsSidebar() {
  const { data: modelsResponse, isLoading } = useModels(1, 100, 'alias', 'asc');
  const models = modelsResponse?.data || [];

  return (
    <Sidebar side="right" variant="floating">
      <SidebarHeader>
        <h2>Settings</h2>
      </SidebarHeader>
      <SidebarContent>
        <SidebarGroup>
          <AliasSelector
            models={models}
            isLoading={isLoading}
          />
          <SystemPrompt
            isLoading={isLoading}
          />
          <StopWords
            isLoading={isLoading}
          />
          <TokenSlider
            isLoading={isLoading}
          />
        </SidebarGroup>
      </SidebarContent>
    </Sidebar>
  );
}
