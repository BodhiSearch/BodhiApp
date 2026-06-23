import { useState } from 'react';

import { ShellIcon } from '@/components/shell';
import { Input } from '@/components/ui/input';
import { useGetAppInfo } from '@/hooks/info';
import { toast } from '@/hooks/use-toast';
import { copyToClipboard, getClipboardUnavailableMessage } from '@/lib/clipboard';

/** Invite-link header action — multi-tenant deployments only (gated on AppInfo.deployment). */
export function InviteLinkAction() {
  const { data: appInfo } = useGetAppInfo();
  const [open, setOpen] = useState(false);
  const [copied, setCopied] = useState(false);

  if (appInfo?.deployment !== 'multi_tenant') {
    return null;
  }

  const inviteUrl = `${appInfo.url}/ui/login/?invite=${appInfo.client_id}`;

  const handleCopy = async () => {
    try {
      await copyToClipboard(inviteUrl);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch {
      setCopied(false);
      toast({
        title: 'Copy Failed',
        description: getClipboardUnavailableMessage() ?? 'Failed to copy invite URL',
        variant: 'destructive',
      });
    }
  };

  return (
    <div style={{ position: 'relative' }}>
      <button className="l-iconbtn" title="Invite users" onClick={() => setOpen((o) => !o)} data-testid="invite-toggle">
        <ShellIcon name="user-plus" size={15} />
      </button>
      {open && (
        <div
          style={{
            position: 'absolute',
            top: 'calc(100% + 6px)',
            right: 0,
            zIndex: 50,
            background: 'hsl(var(--popover))',
            border: '1px solid hsl(var(--border))',
            borderRadius: 10,
            padding: 12,
            boxShadow: 'var(--shadow-md)',
          }}
        >
          <div className="mu-invite-pop">
            <div className="mu-invite-pop-label">Invite link</div>
            <div className="mu-invite-pop-row">
              <Input readOnly value={inviteUrl} data-testid="invite-url-input" className="text-sm" />
              <button
                className="l-iconbtn"
                onClick={handleCopy}
                title="Copy invite link"
                data-testid="invite-copy-button"
              >
                <ShellIcon name={copied ? 'check' : 'copy'} size={14} />
              </button>
            </div>
            <div className="mu-invite-hint">Share this link to invite users to your organization.</div>
          </div>
        </div>
      )}
    </div>
  );
}
