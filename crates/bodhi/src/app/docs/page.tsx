import { DocsIndex } from '@/app/docs/DocsIndex';
import { getDocsForPath } from '@/app/docs/utils';

export default function DocsPage() {
  const sortedGroups = getDocsForPath(null);

  return <DocsIndex groups={sortedGroups} />;
}
