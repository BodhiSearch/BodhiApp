import type { Model } from '@bodhiapp/reference-api-types';

import { ShellIcon } from '@/components/shell';

/** Compact count: 1234 → 1.2k, 1_200_000 → 1.2M. */
export function compact(n: number | null | undefined): string {
  if (n == null) return '—';
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1).replace(/\.0$/, '')}M`;
  if (n >= 1_000) return `${(n / 1_000).toFixed(1).replace(/\.0$/, '')}k`;
  return String(n);
}

/** The REPOSITORY cell: org/repo, verified + multimodal badges, and the tag/quant/license chips. */
export function LocalRepoCell({ model }: { model: Model }) {
  const tags = model.tags ?? model.specialisation ?? [];
  const isMultimodal = model.pipeline_tag === 'image-text-to-text';
  return (
    <div className="ld-body">
      <div className="ld-name">
        <span className="ld-org">{model.namespace}</span>
        <span className="ld-sep">/</span>
        <span className="ld-repo">{model.repo}</span>
        {model.owner_verified && (
          <span className="ld-verified" title="Verified publisher">
            <ShellIcon name="badge-check" size={13} />
          </span>
        )}
        {isMultimodal && (
          <span className="ld-modality" title="Image-Text-to-Text (multimodal)">
            <ShellIcon name="image" size={10} />
            multimodal
          </span>
        )}
      </div>
      <div className="ld-tags">
        {tags.slice(0, 4).map((t) => (
          <span className="ld-tag" key={t}>
            {t}
          </span>
        ))}
        {model.quant_count != null && (
          <span className="ld-meta-chip" title={`${model.quant_count} quantizations`}>
            {model.quant_count} quants
          </span>
        )}
        {model.license && <span className="ld-meta-chip">{model.license}</span>}
      </div>
    </div>
  );
}
