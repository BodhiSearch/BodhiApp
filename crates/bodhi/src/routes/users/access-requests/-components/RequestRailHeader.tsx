import { UserAccessRequest } from '@bodhiapp/ts-client';

import { ShellIcon } from '@/components/shell';
import { avatarColor, initials } from '@/routes/users/access-requests/-components/utils';

export function RequestRailHeader({ req, onClose }: { req: UserAccessRequest; onClose: () => void }) {
  return (
    <div className="dp-head">
      <div className="dp-head-icon" style={{ background: avatarColor(req.username) }}>
        {initials(req.username)}
      </div>
      <div className="dp-head-body">
        <div className="dp-head-title">{req.username}</div>
        <div className="dp-head-sub">User access request</div>
      </div>
      <button className="dp-close" onClick={onClose} title="Close" data-testid="request-detail-close">
        <ShellIcon name="x" size={15} />
      </button>
    </div>
  );
}
