import { PROSE_CLASSES } from '@/app/docs/constants';
import { DocGroup } from '@/app/docs/utils';
import Link from 'next/link';
import { memo } from 'react';

interface DocsIndexProps {
  groups: DocGroup[];
  title?: string;
  description?: string;
}

const EmptyState = () => (
  <div className={PROSE_CLASSES.root}>
    <p>No documentation available.</p>
  </div>
);

export const DocsIndex = memo(
  ({ groups, title, description }: DocsIndexProps) => {
    if (!groups?.length) {
      return <EmptyState />;
    }

    return (
      <div className={PROSE_CLASSES.root}>
        {title && <h1 className={PROSE_CLASSES.heading.h1}>{title}</h1>}
        {description && <p className="lead">{description}</p>}

        {groups.map((group) => (
          <section key={group.key} className={PROSE_CLASSES.section}>
            <h2 className={PROSE_CLASSES.heading.h2}>{group.title}</h2>
            <div className={PROSE_CLASSES.grid}>
              {group.items.map((doc) => (
                <Link
                  key={doc.slug}
                  href={`/docs/${doc.slug}`}
                  className={PROSE_CLASSES.link}
                >
                  <h3 className={PROSE_CLASSES.heading.h3}>{doc.title}</h3>
                  {doc.description && (
                    <p className={PROSE_CLASSES.description}>
                      {doc.description}
                    </p>
                  )}
                </Link>
              ))}
            </div>
          </section>
        ))}
      </div>
    );
  }
);

DocsIndex.displayName = 'DocsIndex';
