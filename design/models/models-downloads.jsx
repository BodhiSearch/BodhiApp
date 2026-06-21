/* ═══════════════════════════════════════════════════════════════
   Bodhi Models — Downloads rail
   models/models-downloads.jsx   (load after models-base.jsx)

   Right-rail panel listing in-flight + finished model pulls. The
   progress bar (DownloadProgress) is the SAME visual primitive used on
   the setup wizard's Local Models step — lotus-pink fill on a light
   track, recoloured per-theme. Reused here verbatim.

   Exports: fmtSize, gbToMb, DOWNLOADS_INIT, DownloadProgress, DlRow,
            DownloadsHead, DownloadsRail
═══════════════════════════════════════════════════════════════ */
const fmtSize = (mb) => mb >= 1024 ? (mb / 1024).toFixed(mb / 1024 >= 10 ? 1 : 2) + ' GB' : mb.toFixed(1) + ' MB';
const gbToMb = (s) => parseFloat(s) * (s.includes('GB') ? 1024 : 1);

const DOWNLOADS_INIT = {
  active: [
    { id: 'd1', org: 'Qwen', repo: 'Qwen3-Coder-32B', file: 'Qwen3-Coder-32B-Q4_K_M.gguf', pct: 42.3, totalMB: 18944, rate: 0.55, speed: '24.6 MB/s' },
    { id: 'd2', org: 'BAAI', repo: 'bge-m3', file: 'bge-m3-Q4_K_M.gguf', pct: 30.3, totalMB: 417.5, rate: 4.1, speed: '17.8 MB/s' }
  ],
  queued: [
    { id: 'q1', org: 'meta-llama', repo: 'Llama-3.3-70B', file: 'Llama-3.3-70B-Instruct.Q4_K_M.gguf', total: '35.0 GB', waitOn: 'Qwen3-Coder-32B' }
  ],
  done: [
    { id: 'c1', org: 'Mungert', repo: 'SmolLM2-135M-Instruct-GGUF', file: 'SmolLM2-135M-Instruct-bf16_q8_0.gguf', size: '138 MB', when: 'Today, 16:49' },
    { id: 'c2', org: 'microsoft', repo: 'Phi-4', file: 'Phi-4-Q4_K_M.gguf', size: '5.1 GB', when: 'Yesterday' },
    { id: 'c3', org: 'google', repo: 'gemma-2-9b-it', file: 'gemma-2-9b-it-Q4_K_M.gguf', size: '5.8 GB', when: '2 days ago' }
  ],
  failed: [
    { id: 'f1', org: 'deepseek-ai', repo: 'DeepSeek-V3', file: 'DeepSeek-V3-Q2_K.gguf', size: '35.0 GB', reason: 'Not enough disk space' }
  ]
};

function DownloadProgress({ pct, loadedMB, totalMB }) {
  const p = Math.min(100, pct);
  return (
    <div className="dl-prog">
      <div className="dl-prog-head">
        <span className="dl-prog-pct">{p.toFixed(0)}%</span>
        <span className="dl-prog-bytes">{fmtSize(loadedMB)} <span className="dl-prog-sep">/</span> {fmtSize(totalMB)}</span>
      </div>
      <div className="dl-prog-track"><div className="dl-prog-fill" style={{ width: p + '%' }} /></div>
    </div>);
}

function DlRow({ d, kind, onCancel, onRetry, onClear }) {
  return (
    <div className={'dl-card dl-' + kind}>
      <div className="dl-card-top">
        <div className={'dl-icon dl-icon-' + kind}>
          <Ic name={kind === 'done' ? 'check' : kind === 'failed' ? 'alert-triangle' : kind === 'queued' ? 'clock' : 'hard-drive'} size={14} />
        </div>
        <div className="dl-meta">
          <div className="dl-name"><span className="dl-org">{d.org}/</span>{d.repo}</div>
          <div className="dl-file">{d.file}</div>
        </div>
        {kind === 'active' && <button className="dl-act" title="Cancel download" onClick={onCancel}><Ic name="x" size={13} /></button>}
        {kind === 'queued' && <button className="dl-act" title="Remove from queue" onClick={onCancel}><Ic name="x" size={13} /></button>}
        {kind === 'failed' && <button className="dl-act" title="Retry" onClick={onRetry}><Ic name="rotate-cw" size={13} /></button>}
        {kind === 'done' && <button className="dl-act" title="Remove from list" onClick={onClear}><Ic name="x" size={13} /></button>}
      </div>

      {kind === 'active' &&
        <>
          <DownloadProgress pct={d.pct} loadedMB={d.totalMB * d.pct / 100} totalMB={d.totalMB} />
          <div className="dl-substat"><Ic name="arrow-down" size={10} />{d.speed}<span className="dl-substat-dot">·</span>{d.eta || 'calculating…'}</div>
        </>}
      {kind === 'queued' &&
        <div className="dl-line"><span className="dl-line-l"><Ic name="clock" size={10} /> Waiting for {d.waitOn}</span><span className="dl-line-r">{d.total}</span></div>}
      {kind === 'done' &&
        <div className="dl-line"><span className="dl-line-l dl-ok"><Ic name="check-circle" size={10} /> Completed · {d.when}</span><span className="dl-line-r">{d.size}</span></div>}
      {kind === 'failed' &&
        <div className="dl-line"><span className="dl-line-l dl-err"><Ic name="alert-triangle" size={10} /> {d.reason}</span><span className="dl-line-r">{d.size}</span></div>}
    </div>);
}

function DownloadsHead({ onClose }) {
  return (
    <div className="panel-head-rail">
      <div className="dl-head-icon"><Ic name="arrow-down-to-line" size={14} /></div>
      <div className="panel-head-title">Downloads</div>
      <button className="panel-close" onClick={onClose}><Ic name="x" size={15} /></button>
    </div>);
}

function DownloadsRail({ dl, dispatch }) {
  const empty = !dl.active.length && !dl.queued.length && !dl.done.length && !dl.failed.length;
  if (empty) return (
    <div className="dl-empty">
      <div className="dl-empty-icon"><Ic name="inbox" size={22} /></div>
      <div className="dl-empty-title">No downloads</div>
      <div className="dl-empty-sub">Download a model from the list and it will show up here with live progress.</div>
    </div>);

  const sect = (key, label, items, kind) => items.length > 0 &&
    <div className="dl-sect" key={key}>
      <div className="dl-sect-head"><span className="dl-sect-lbl">{label}</span><span className="dl-sect-count">{items.length}</span></div>
      <div className="dl-list">{items.map((d) =>
        <DlRow key={d.id} d={d} kind={kind}
          onCancel={() => dispatch({ t: 'remove', kind, id: d.id })}
          onClear={() => dispatch({ t: 'remove', kind, id: d.id })}
          onRetry={() => dispatch({ t: 'retry', id: d.id })} />)}</div>
    </div>;

  return (
    <div className="dl-rail">
      {sect('active', 'Downloading', dl.active, 'active')}
      {sect('queued', 'Queued', dl.queued, 'queued')}
      {sect('failed', 'Failed', dl.failed, 'failed')}
      {sect('done', 'Completed', dl.done, 'done')}
    </div>);
}

Object.assign(window, {
  fmtSize, gbToMb, DOWNLOADS_INIT, DownloadProgress, DlRow, DownloadsHead, DownloadsRail,
});
