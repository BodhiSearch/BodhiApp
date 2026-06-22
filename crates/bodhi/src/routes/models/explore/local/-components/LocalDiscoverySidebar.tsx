import { useState } from 'react';

import type { ListModelsQuery, SortKey, Specialisation } from '@bodhiapp/reference-api-types';

import { ShellIcon } from '@/components/shell';

import '@/routes/models/-components/models.css';

/**
 * Faceted sidebar for Explore · Local Models. Controlled — selections drive the parent's
 * `ListModelsQuery`. Only facets the v1 API actually filters on are shown (no capability / size /
 * context / quant-bits / quant-method / curated — dropped in v1). See the batch plan.
 *
 * Phase 2a: Browse (sort), Specialisation, Task (pipeline_tag).
 * Phase 2b: Tag (advanced), Language, License (repeatable → repeated query keys).
 */

const SPECIALISATION_FACETS: { id: Specialisation; label: string }[] = [
  { id: 'coding', label: 'Coding' },
  { id: 'reasoning', label: 'Reasoning' },
  { id: 'vision', label: 'Vision' },
];

const TASK_FACETS: { id: string; label: string }[] = [
  { id: 'text-generation', label: 'Text Generation' },
  { id: 'image-text-to-text', label: 'Image-Text-to-Text' },
];

// Hardcoded enum sets (no taxonomy endpoint in v1). `tag` is AND-ed; language/license are OR-ed.
const TAG_FACETS = ['tool-use', 'conversational', 'thinking', 'moe', 'embedding'];
const LANGUAGE_FACETS = ['en', 'zh', 'es', 'fr', 'de', 'ja', 'ko'];
const LICENSE_FACETS: { id: string; label: string }[] = [
  { id: 'apache-2.0', label: 'Apache-2' },
  { id: 'mit', label: 'MIT' },
  { id: 'llama3.3', label: 'Llama' },
  { id: 'gemma', label: 'Gemma' },
  { id: 'deepseek', label: 'DeepSeek' },
];

/** Browse presets map to a sort key (Trending / New). */
const BROWSE: { id: SortKey; label: string; icon: string }[] = [
  { id: 'trending', label: 'Trending', icon: 'trending-up' },
  { id: 'created_at', label: 'New', icon: 'sparkles' },
];

function toggle<T>(list: T[] | undefined, value: T): T[] | undefined {
  const set = new Set(list ?? []);
  if (set.has(value)) set.delete(value);
  else set.add(value);
  const next = [...set];
  return next.length ? next : undefined;
}

export interface DiscoveryFacets {
  specialisation?: Specialisation[];
  pipeline_tag?: string;
  tag?: string[];
  language?: string[];
  license?: string[];
  author?: string[];
}

/** True when any facet (beyond the implicit Task default) is active. */
export function hasActiveFacets(f: DiscoveryFacets): boolean {
  return Boolean(
    f.specialisation?.length ||
      f.pipeline_tag ||
      f.tag?.length ||
      f.language?.length ||
      f.license?.length ||
      f.author?.length
  );
}

interface SidebarProps {
  facets: DiscoveryFacets;
  sort: SortKey;
  onFacetsChange: (next: DiscoveryFacets) => void;
  onBrowse: (sort: SortKey) => void;
  onClearAll: () => void;
}

export function LocalDiscoverySidebar({ facets, sort, onFacetsChange, onBrowse, onClearAll }: SidebarProps) {
  const spec = new Set(facets.specialisation ?? []);
  return (
    <div className="m-facets" data-testid="ld-facets">
      {hasActiveFacets(facets) && (
        <button type="button" className="ld-clear-all" onClick={onClearAll} data-testid="ld-clear-all">
          <ShellIcon name="x" size={11} /> Clear all filters
        </button>
      )}

      <FacetGroup icon="compass" title="Browse">
        <div className="m-facet-pills nowrap">
          {BROWSE.map((b) => (
            <button
              key={b.id}
              type="button"
              className={`m-facet-pill${sort === b.id ? ' active' : ''}`}
              aria-pressed={sort === b.id}
              onClick={() => onBrowse(b.id)}
              data-testid={`ld-browse-${b.id}`}
            >
              <ShellIcon name={b.icon} size={11} /> {b.label}
            </button>
          ))}
        </div>
      </FacetGroup>

      <FacetGroup icon="target" title="Specialisation">
        <div className="m-facet-pills">
          {SPECIALISATION_FACETS.map((f) => (
            <button
              key={f.id}
              type="button"
              className={`m-facet-pill${spec.has(f.id) ? ' active' : ''}`}
              aria-pressed={spec.has(f.id)}
              onClick={() => onFacetsChange({ ...facets, specialisation: toggle(facets.specialisation, f.id) })}
              data-testid={`ld-spec-${f.id}`}
            >
              {f.label}
            </button>
          ))}
        </div>
      </FacetGroup>

      <FacetGroup icon="list-checks" title="Task">
        <div className="m-facet-pills">
          {TASK_FACETS.map((f) => {
            // Single-select: text-generation is the API default (omit param); selecting a task pins it.
            const active = (facets.pipeline_tag ?? 'text-generation') === f.id;
            return (
              <button
                key={f.id}
                type="button"
                className={`m-facet-pill${active ? ' active' : ''}`}
                aria-pressed={active}
                onClick={() =>
                  onFacetsChange({
                    ...facets,
                    pipeline_tag: f.id === 'text-generation' ? undefined : f.id,
                  })
                }
                data-testid={`ld-task-${f.id}`}
              >
                {f.label}
              </button>
            );
          })}
        </div>
      </FacetGroup>

      <PublisherGroup
        authors={facets.author ?? []}
        onAdd={(name) => onFacetsChange({ ...facets, author: toggle(facets.author, name) })}
        onRemove={(name) =>
          onFacetsChange({ ...facets, author: (facets.author ?? []).filter((a) => a !== name) || undefined })
        }
      />

      <ChipFacetGroup
        icon="hash"
        title="Tag"
        note="advanced"
        facets={TAG_FACETS.map((t) => ({ id: t, label: t }))}
        selected={facets.tag}
        testIdPrefix="ld-tag"
        onToggle={(id) => onFacetsChange({ ...facets, tag: toggle(facets.tag, id) })}
        onClear={() => onFacetsChange({ ...facets, tag: undefined })}
      />

      <ChipFacetGroup
        icon="languages"
        title="Language"
        facets={LANGUAGE_FACETS.map((l) => ({ id: l, label: l }))}
        selected={facets.language}
        testIdPrefix="ld-lang"
        onToggle={(id) => onFacetsChange({ ...facets, language: toggle(facets.language, id) })}
        onClear={() => onFacetsChange({ ...facets, language: undefined })}
      />

      <ChipFacetGroup
        icon="scale"
        title="License"
        facets={LICENSE_FACETS}
        selected={facets.license}
        testIdPrefix="ld-license"
        onToggle={(id) => onFacetsChange({ ...facets, license: toggle(facets.license, id) })}
        onClear={() => onFacetsChange({ ...facets, license: undefined })}
      />
    </div>
  );
}

/** Free-text publisher filter → `author` (repeatable). No autocomplete (no /orgs endpoint in v1). */
function PublisherGroup({
  authors,
  onAdd,
  onRemove,
}: {
  authors: string[];
  onAdd: (name: string) => void;
  onRemove: (name: string) => void;
}) {
  const [text, setText] = useState('');
  const commit = () => {
    const v = text.trim();
    if (v && !authors.some((a) => a.toLowerCase() === v.toLowerCase())) onAdd(v);
    setText('');
  };
  return (
    <FacetGroup
      icon="building-2"
      title="Publisher"
      onClear={authors.length ? () => authors.forEach(onRemove) : undefined}
    >
      {authors.length > 0 && (
        <div className="ld-pub-tags">
          {authors.map((a) => (
            <span className="ld-pub-chip" key={a} data-testid={`ld-author-chip-${a}`}>
              {a}
              <button type="button" className="ld-pub-x" onClick={() => onRemove(a)} aria-label={`Remove ${a}`}>
                <ShellIcon name="x" size={10} />
              </button>
            </span>
          ))}
        </div>
      )}
      <input
        type="text"
        className="ld-pub-input"
        placeholder={authors.length ? 'Add another…' : 'org or author…'}
        value={text}
        data-testid="ld-author-input"
        onChange={(e) => setText(e.target.value)}
        onKeyDown={(e) => {
          if (e.key === 'Enter') commit();
        }}
      />
    </FacetGroup>
  );
}

/** A clearable multi-select chip group (Tag / Language / License). */
function ChipFacetGroup({
  icon,
  title,
  note,
  facets,
  selected,
  testIdPrefix,
  onToggle,
  onClear,
}: {
  icon: string;
  title: string;
  note?: string;
  facets: { id: string; label: string }[];
  selected: string[] | undefined;
  testIdPrefix: string;
  onToggle: (id: string) => void;
  onClear: () => void;
}) {
  const active = new Set(selected ?? []);
  return (
    <FacetGroup icon={icon} title={title} note={note} onClear={active.size > 0 ? onClear : undefined}>
      <div className="m-facet-pills">
        {facets.map((f) => (
          <button
            key={f.id}
            type="button"
            className={`m-facet-pill${active.has(f.id) ? ' active' : ''}`}
            aria-pressed={active.has(f.id)}
            onClick={() => onToggle(f.id)}
            data-testid={`${testIdPrefix}-${f.id}`}
          >
            {f.label}
          </button>
        ))}
      </div>
    </FacetGroup>
  );
}

/** Builds the API query params contributed by the facets (omitting defaults/empties). */
export function facetsToQuery(facets: DiscoveryFacets): Partial<ListModelsQuery> {
  return {
    ...(facets.specialisation?.length ? { specialisation: facets.specialisation } : {}),
    ...(facets.pipeline_tag ? { pipeline_tag: facets.pipeline_tag } : {}),
    ...(facets.tag?.length ? { tag: facets.tag } : {}),
    ...(facets.language?.length ? { language: facets.language } : {}),
    ...(facets.license?.length ? { license: facets.license } : {}),
    ...(facets.author?.length ? { author: facets.author } : {}),
  };
}

function FacetGroup({
  icon,
  title,
  note,
  onClear,
  children,
}: {
  icon: string;
  title: string;
  note?: string;
  onClear?: () => void;
  children: React.ReactNode;
}) {
  return (
    <div className="m-facet-group">
      <div className="m-facet-label">
        <ShellIcon name={icon} size={13} />
        <span>{title}</span>
        {note && <span className="m-facet-hint">({note})</span>}
        {onClear && (
          <button type="button" className="fg-clear" onClick={onClear} data-testid={`ld-clear-${title.toLowerCase()}`}>
            Clear
          </button>
        )}
      </div>
      {children}
    </div>
  );
}
