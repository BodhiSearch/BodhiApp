import { DocsIndex } from '@/components/docs/DocsIndex';
import { getDocsForSlug } from '@/components/docs/utils';

export default function DocsPage() {
  const sortedGroups = getDocsForSlug(null);

  return (
    <DocsIndex
      groups={sortedGroups}
      title="Documentation"
      description="Welcome to our documentation. Choose a topic below to get started."
    />
  );
}
