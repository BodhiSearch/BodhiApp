import { ShellIcon } from '@/components/shell';
import type { McpCapabilityCounts } from '@/hooks/mcps/useMcpClient';

import type { Feature } from './OverviewView';

interface NavItem {
  id: Feature;
  icon: string;
  label: string;
  count?: number;
}

export interface CapabilityNavProps {
  active: Feature;
  counts: McpCapabilityCounts;
  onSelect: (feature: Feature) => void;
}

export function CapabilityNav({ active, counts, onSelect }: CapabilityNavProps) {
  const items: NavItem[] = [
    { id: 'overview', icon: 'layout-dashboard', label: 'Overview' },
    { id: 'tools', icon: 'wrench', label: 'Tools', count: counts.tools },
    { id: 'prompts', icon: 'message-square-quote', label: 'Prompts', count: counts.prompts },
    { id: 'resources', icon: 'folder-open', label: 'Resources', count: counts.resources },
    { id: 'templates', icon: 'layout-template', label: 'Templates', count: counts.resourceTemplates },
  ];

  return (
    <nav className="pg-capnav" data-testid="mcp-playground-capnav" aria-label="Playground capabilities">
      <div className="pg-capnav-label">Explore</div>
      <ul>
        {items.map((it) => {
          const empty = it.count === 0;
          const on = active === it.id;
          return (
            <li key={it.id}>
              <button
                type="button"
                className={'pg-capnav-item' + (on ? ' on' : '') + (empty && it.id !== 'overview' ? ' muted' : '')}
                onClick={() => onSelect(it.id)}
                data-testid={`mcp-playground-capability-${it.id}`}
                data-test-active={on}
                aria-current={on ? 'page' : undefined}
              >
                <span className="pg-capnav-ico">
                  <ShellIcon name={it.icon} size={14} />
                </span>
                <span className="pg-capnav-label-text">{it.label}</span>
                {it.count !== undefined && (
                  <span className="pg-capnav-count" data-testid={`mcp-playground-capability-count-${it.id}`}>
                    {it.count}
                  </span>
                )}
              </button>
            </li>
          );
        })}
      </ul>
    </nav>
  );
}
