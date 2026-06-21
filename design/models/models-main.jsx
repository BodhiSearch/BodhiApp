/* ═══════════════════════════════════════════════════════════════
   Bodhi Models — main column (toolbar + list)
   models/models-main.jsx   (load after models-rows.jsx + models-filters.jsx)

   The search toolbar, result/sort bar, list headers, and the scrolling
   list itself — switching between MyRow / LocalRow / ApiRow by mode.
   Sort + column controls come from bodhi-list.jsx.

   Exports: ModelsMain
═══════════════════════════════════════════════════════════════ */
const { useState: useMainState } = React;
const { MY_MODELS: MAIN_MY, LOCAL_MODELS: MAIN_LOCAL, API_PROVIDERS: MAIN_API } = window.MODELS_DATA;

function ModelsMain({ mode, sel, onSelect, density, showTags, showScore, onShowDownloads, downloadsOpen, dlCount,
  sort, onSort, cols, onToggleCol, orgFilters, onPickOrg, onRemoveOrg, onClearOrgs }) {
  const { openRail } = useShell();
  useListKeyNav({ rootSelector: '.model-list', rowSelector: '.m-row, .my-card' });
  const [q, setQ] = useMainState('');
  const pick = (kind, item, idx) => {onSelect({ kind, item, idx });openRail();};

  const listClass = ['model-list',
  mode === 'my-models' ? 'my-mode' : '',
  density === 'compact' ? 'compact' : '',
  !showTags ? 'hide-tags' : '',
  !showScore ? 'hide-scorelbl' : ''].
  filter(Boolean).join(' ');

  /* local: apply publisher preset, then backend-style sort */
  let localRows = MAIN_LOCAL;
  if (mode === 'local') {
    if (orgFilters && orgFilters.length) localRows = localRows.filter((m) => orgFilters.some((o) => o.toLowerCase() === m.org.toLowerCase()));
    const val = (m) => sort.key === 'downloads' ? m.dlNum : sort.key === 'likes' ? m.likeNum : sort.key === 'created' ? new Date(m.created).getTime() : m.trending;
    localRows = [...localRows].sort((a, b) => sort.order === 'asc' ? val(a) - val(b) : val(b) - val(a));
  }
  const sortLabel = { trending: 'Trending', created: 'Newest', downloads: 'Downloads', likes: 'Likes' }[sort.key];

  return (
    <div className="models-main">
      <div className="toolbar" style={{ padding: "12px 16px" }}>
        <ShellSearch value={q} onChange={setQ} placeholder={PLACEHOLDER[mode]} kbd="⌘K" />
        <button className={'l-iconbtn' + (downloadsOpen ? ' on' : '')} title="Downloads" onClick={onShowDownloads}>
          <Ic name="arrow-down-to-line" size={15} />
          {dlCount > 0 && <span className="dl-badge">{dlCount}</span>}
        </button>
      </div>

      {mode === 'local' &&
      <div className="result-bar">
          <span className="result-count">Showing {localRows.length}</span>
          <span className="result-sort">sorted by <strong>{sortLabel}</strong> · {sort.order === 'asc' ? 'ascending' : 'descending'}</span>
        </div>
      }

      {mode === 'local' &&
      <div className="list-head list-head-local">
          <div className="lh-num lh-label">#</div>
          <div className="lh-model lh-label" style={{ paddingLeft: 4 }}>Repository</div>
          <div className="lh-stats">
            {cols.downloads && <SortHeaderCell label="Downloads" k="downloads" sortKey={sort.key} sortOrder={sort.order} onSort={onSort} />}
            {cols.likes && <SortHeaderCell label="Likes" k="likes" sortKey={sort.key} sortOrder={sort.order} onSort={onSort} />}
          </div>
          <div className="lh-cols"><ColumnsMenu cols={cols} onToggle={onToggleCol} compact /></div>
        </div>
      }
      {mode === 'api' &&
      <div className="list-head">
          <div className="lh-num lh-label">#</div>
          <div className="lh-model lh-label" style={{ paddingLeft: 56 }}>Provider</div>
          <div className="lh-score" style={{ minWidth: 68 }}>Models</div>
          <div className="lh-action" />
        </div>
      }

      <div className={listClass}>
        {mode === 'my-models' && MAIN_MY.map((item, i) =>
        <MyRow key={item.id} item={item} active={sel && sel.kind === 'my' && sel.idx === i} onClick={() => pick('my', item, i)} />)}
        {mode === 'local' && <>
          {localRows.map((m, i) =>
          <LocalRow key={m.org + m.repo} m={m} idx={i + 1} cols={cols} sortKey={sort.key}
            active={sel && sel.kind === 'local' && sel.item.org === m.org && sel.item.repo === m.repo}
            onClick={() => pick('local', m, i)} onPickOrg={onPickOrg} />)}
          {localRows.length === 0 &&
            <div className="list-empty"><Ic name="search-x" size={22} /><div>No repositories match these filters.</div></div>}
          {localRows.length > 0 && <button className="load-more"><Ic name="chevrons-down" size={14} /> Load more</button>}
        </>}
        {mode === 'api' && <>
          {MAIN_API.map((p, i) =>
          <ApiRow key={p.slug} p={p} active={sel && sel.kind === 'api' && sel.idx === i} onClick={() => pick('api', p, i)} />)}
          <button className="load-more"><Ic name="chevrons-down" size={14} /> Load more</button>
        </>}
      </div>
    </div>);

}

Object.assign(window, { ModelsMain });
