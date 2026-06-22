import type { CatalogEntry } from './paramCatalogs';

interface ParamCatalogProps {
  label: string;
  catalog: CatalogEntry[];
  /** Keys already present in the target textarea (rendered as disabled "added" entries). */
  addedKeys: Set<string>;
  onAdd: (entry: CatalogEntry) => void;
  /** testid prefix so context-flags and request-params catalogs don't collide. */
  testIdPrefix: string;
}

/** A click-to-add list of flags/params; clicking an entry appends it to the paired textarea. */
export function ParamCatalog({ label, catalog, addedKeys, onAdd, testIdPrefix }: ParamCatalogProps) {
  return (
    <div data-testid={`${testIdPrefix}-catalog`}>
      <div className="lf-split-label">{label}</div>
      <div className="lf-flag-list">
        {catalog.map((entry) => {
          const added = addedKeys.has(entry.key);
          return (
            <button
              type="button"
              key={entry.key}
              disabled={added}
              onClick={() => onAdd(entry)}
              data-testid={`${testIdPrefix}-add-${entry.key}`}
              title={added ? 'Already added' : `Add ${entry.key}`}
              className={`lf-flag-item${added ? ' lf-flag-added' : ''}`}
            >
              <div className="lf-flag-row">
                <span className="lf-flag-name">{entry.key}</span>
                <span className="lf-flag-type">{entry.type}</span>
                {added && <span className="lf-flag-added-badge">added</span>}
              </div>
              <div className="lf-flag-desc">{entry.desc}</div>
            </button>
          );
        })}
      </div>
    </div>
  );
}
