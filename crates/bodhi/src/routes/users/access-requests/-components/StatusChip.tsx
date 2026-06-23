import { ShellIcon } from '@/components/shell';
import { STATUS_ICON } from '@/routes/users/access-requests/-components/utils';

export function StatusChip({ status }: { status: string }) {
  return (
    <span className={`ua-status ${status}`} data-testid={`request-status-${status}`}>
      <ShellIcon name={STATUS_ICON[status] ?? 'circle'} size={11} />
      {status.charAt(0).toUpperCase() + status.slice(1)}
    </span>
  );
}
