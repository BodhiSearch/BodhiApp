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
      <div className="text-xs font-medium text-muted-foreground mb-2">{label}</div>
      <div className="max-h-64 overflow-y-auto rounded-md border divide-y">
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
              className={`flex w-full flex-col items-start gap-0.5 px-3 py-2 text-left text-sm transition-colors ${
                added ? 'opacity-50 cursor-not-allowed' : 'hover:bg-muted/50'
              }`}
            >
              <span className="flex w-full items-center gap-2">
                <span className="font-mono text-xs">{entry.key}</span>
                <span className="text-[10px] uppercase tracking-wide text-muted-foreground">{entry.type}</span>
                {added && <span className="ml-auto text-[10px] text-muted-foreground">added</span>}
              </span>
              <span className="text-xs text-muted-foreground">{entry.desc}</span>
            </button>
          );
        })}
      </div>
    </div>
  );
}
