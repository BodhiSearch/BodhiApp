/* ═══════════════════════════════════════════════════════════════
   Bodhi Models — page root
   models/bodhi-models-app.jsx   (load LAST of the models modules)

   Owns page state (selection, tabs, stars, downloads, local-explore
   sort/columns/publisher presets) and assembles the shell with the
   sidebar, main list, and detail/downloads rail. One fixed mode per
   page, chosen by window.MODELS_MODE ('my-models' | 'local' | 'api').

   Module load order (set in each Bodhi Models*.html):
     models-base · models-downloads · models-filters · models-rows ·
     models-detail · models-main · bodhi-models-app   (+ models-local
     on the Local page only)
═══════════════════════════════════════════════════════════════ */
const { useState, useEffect } = React;
const { MY_MODELS, LOCAL_MODELS, API_PROVIDERS } = window.MODELS_DATA;

function ModelsApp() {
  const mode = window.MODELS_MODE || 'my-models';
  const cfg = MODE_CFG[mode];
  const [sel, setSel] = useState(null);
  const [tab, setTab] = useState('overview');
  const [apiConnectedOnly, setApiConnectedOnly] = useState(false);
  const initialMyIdx = React.useMemo(() => {
    if (mode !== 'my-models') return -1;
    const id = new URLSearchParams(window.location.search).get('select');
    return id ? MY_MODELS.findIndex((m) => m.id === id) : -1;
  }, []);
  const [starred, setStarred] = useState(() => new Set());
  const [showDownloads, setShowDownloads] = useState(false);
  const [dl, setDl] = useState(DOWNLOADS_INIT);
  const density = 'cozy';
  const showTags = true;
  const showScore = true;

  /* local explore: sort (maps to backend sort + sort_order), visible stat
     columns, and the publisher / staff-pick presets */
  const [sort, setSort] = useState({ key: 'trending', order: 'desc' });
  const [cols, setCols] = useState({ downloads: true, likes: true });
  const [orgFilters, setOrgFilters] = useState([]);
  const onSort = (k) => setSort((s) => s.key === k ? { key: k, order: s.order === 'desc' ? 'asc' : 'desc' } : { key: k, order: 'desc' });
  const onBrowse = (key) => setSort({ key, order: 'desc' });
  const onToggleCol = (k) => setCols((c) => ({ ...c, [k]: !c[k] }));
  const onPickOrg = (org) => { if (!org) return; setShowDownloads(false); setOrgFilters((prev) => prev.some((o) => o.toLowerCase() === org.toLowerCase()) ? prev : [...prev, org]); };
  const onRemoveOrg = (org) => setOrgFilters((prev) => prev.filter((o) => o !== org));
  const onClearOrgs = () => setOrgFilters([]);

  /* default selection on desktop */
  useEffect(() => {
    if (window.matchMedia('(max-width:767px)').matches) return;
    if (mode === 'local') setSel({ kind: 'local', item: LOCAL_MODELS[0], idx: 0 });
    else if (mode === 'api') setSel({ kind: 'api', item: API_PROVIDERS[0], idx: 0 });
    else {const i = initialMyIdx >= 0 ? initialMyIdx : 0;setSel({ kind: 'my', item: MY_MODELS[i], idx: i });}
  }, []);

  /* live progress — ticks active pulls while the panel is open */
  useEffect(() => {
    if (!showDownloads) return;
    const t = setInterval(() => {
      setDl((prev) => {
        if (!prev.active.length) return prev;
        const stillActive = [];const finished = [];
        prev.active.forEach((d) => {
          const np = d.pct + d.rate + Math.random() * d.rate * 0.5;
          const remMB = d.totalMB * (100 - np) / 100;
          const rateMB = parseFloat(d.speed) || 12;
          const secs = Math.max(0, Math.round(remMB / rateMB));
          const eta = secs > 90 ? Math.round(secs / 60) + ' min left' : secs + 's left';
          if (np >= 100) finished.push({ id: d.id, org: d.org, repo: d.repo, file: d.file, size: fmtSize(d.totalMB), when: 'Just now' });
          else stillActive.push({ ...d, pct: np, eta });
        });
        let queued = prev.queued, promoted = [];
        if (finished.length && queued.length) {
          const first = queued[0];queued = queued.slice(1);
          promoted.push({ id: first.id, org: first.org, repo: first.repo, file: first.file, pct: 0.4, totalMB: gbToMb(first.total), rate: 0.5, speed: '12.0 MB/s' });
        }
        return { ...prev, active: [...stillActive, ...promoted], queued, done: [...finished, ...prev.done] };
      });
    }, 900);
    return () => clearInterval(t);
  }, [showDownloads]);

  const dlDispatch = (a) => {
    if (a.t === 'remove') setDl((p) => ({ ...p, [a.kind]: p[a.kind].filter((x) => x.id !== a.id) }));
    else if (a.t === 'retry') setDl((p) => {
      const item = p.failed.find((x) => x.id === a.id);if (!item) return p;
      return { ...p, failed: p.failed.filter((x) => x.id !== a.id),
        active: [...p.active, { id: item.id, org: item.org, repo: item.repo, file: item.file, pct: 0.3, totalMB: gbToMb(item.size), rate: 0.5, speed: '12.0 MB/s' }] };
    });
  };

  const onSelect = (s) => {setSel(s);setTab('overview');setShowDownloads(false);};
  const openDownloads = () => {setSel(null);setShowDownloads((v) => !v);};
  const toggleStar = (key) => setStarred((prev) => {const n = new Set(prev);n.has(key) ? n.delete(key) : n.add(key);return n;});

  const railContent = showDownloads ?
    <DownloadsRail dl={dl} dispatch={dlDispatch} /> :
    sel ? <DetailBody sel={sel} tab={tab} setTab={setTab} starred={starred} toggleStar={toggleStar} onPickOrg={onPickOrg} /> : null;
  const railHead = showDownloads ?
    <DownloadsHead onClose={() => setShowDownloads(false)} /> :
    sel ? <DetailHeader sel={sel} onDeselect={() => setSel(null)} onPickOrg={onPickOrg} /> : undefined;

  return <>
    <AppShell
      section="models" subPage={cfg.subPage}
      resizeKey="models"
      breadcrumb={[{ label: 'Bodhi', href: 'Bodhi Chat.html' }, { label: 'Models', href: 'Bodhi Models.html' }, { label: cfg.label, current: true }]}
      sidebar={<ModelsSidebar mode={mode} orgFilters={orgFilters} onPickOrg={onPickOrg} onRemoveOrg={onRemoveOrg} onClearOrgs={onClearOrgs} sort={sort} onBrowse={onBrowse} apiConnectedOnly={apiConnectedOnly} onToggleApiConnected={() => setApiConnectedOnly((v) => !v)} />}
      contentClass="flush" mainScroll={false} railScroll={false}
      rail={railContent}
      railHeader={railHead}>
      
      <ModelsMain mode={mode} sel={sel} onSelect={onSelect} density={density} showTags={showTags} showScore={showScore}
        onShowDownloads={openDownloads} downloadsOpen={showDownloads} dlCount={dl.active.length + dl.queued.length}
        sort={sort} onSort={onSort} cols={cols} onToggleCol={onToggleCol}
        apiConnectedOnly={apiConnectedOnly} initialMyIdx={initialMyIdx}
        orgFilters={orgFilters} onPickOrg={onPickOrg} onRemoveOrg={onRemoveOrg} onClearOrgs={onClearOrgs} />
    </AppShell>
  </>;
}

ReactDOM.createRoot(document.getElementById('root')).render(<ModelsApp />);
