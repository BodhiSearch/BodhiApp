import { DocsIndex } from '@/app/docs/DocsIndex';
import { getDocsForPath } from '@/app/docs/utils';

export default function DocsPage() {
  const sortedGroups = getDocsForPath(null);

  return (
    <DocsIndex
      groups={sortedGroups}
      title="Documentation"
      description="Welcome to our documentation. Choose a topic below to get started."
    />
  );
}
