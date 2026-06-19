import { ShellIcon } from '@/components/shell';

/** The vertical "↓ on error" pill shown between consecutive step cards. */
export function StepConnector({ testId }: { testId: string }) {
  return (
    <div className="rf-connector" data-testid={testId}>
      <div className="rf-connector-line" />
      <div className="rf-connector-badge">
        <ShellIcon name="arrow-down" size={9} /> on error
      </div>
      <div className="rf-connector-line" />
    </div>
  );
}
