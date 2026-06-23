import { RoleOption } from '@/routes/users/access-requests/-components/utils';

export function RoleSelect({
  value,
  roles,
  onChange,
  className = 'ua-role-select',
  testId,
}: {
  value: string;
  roles: RoleOption[];
  onChange: (role: string) => void;
  className?: string;
  testId?: string;
}) {
  return (
    <select
      className={className}
      value={value}
      data-testid={testId}
      onClick={(e) => e.stopPropagation()}
      onChange={(e) => {
        e.stopPropagation();
        onChange(e.target.value);
      }}
    >
      {roles.map((r) => (
        <option key={r.value} value={r.value}>
          {r.label}
        </option>
      ))}
    </select>
  );
}
