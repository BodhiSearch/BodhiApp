/* Setup · Step 3 — Local Models (recommendation cards) */

const CHAT_MODELS = [
  { name: 'Qwen3.5 35B-A3B', tag: 'Best Overall', rec: true,  desc: 'All-round flagship for conversation, writing, and Q&A.',  size: '22GB',   params: '35B · 3B active', ctx: '262K', quant: 'Q4_K_M', quality: 5, speed: 4 },
  { name: 'Qwen3.5 27B',     tag: 'Best Dense',                desc: 'Dense model with strong, consistent quality.',           size: '16.7GB', params: '27B',             ctx: '262K', quant: 'Q4_K_M', quality: 5, speed: 4 },
  { name: 'Phi-4 Reasoning', tag: 'Best Reasoning',           desc: 'Tuned for step-by-step reasoning and analysis.',         size: '15.6GB', params: '14B',             ctx: '16K',  quant: 'Q8_0',   quality: 5, speed: 4 },
  { name: 'Qwen3.5 9B',      tag: 'Best Value',               desc: 'Best quality-per-gigabyte for everyday chat.',           size: '9.5GB',  params: '9B',              ctx: '262K', quant: 'Q8_0',   quality: 4, speed: 5 },
  { name: 'GLM-4.7 Flash',   tag: 'Multimodal',               desc: 'Fast multimodal model with a long context window.',      size: '18.1GB', params: '30B · 3.6B active', ctx: '200K', quant: 'Q4_K_M', quality: 4, speed: 4 },
  { name: 'Gemma 3 27B',     tag: 'Google Multimodal',        desc: 'Google’s capable multimodal open model.',                size: '16.5GB', params: '27B',             ctx: '128K', quant: 'Q4_K_M', quality: 4, speed: 4 },
];

const EMBED_MODELS = [
  { name: 'Qwen3 Embedding 8B', tag: 'Top Choice', rec: true, desc: 'Top-ranked embeddings for RAG and semantic search.',   size: '8.05GB', params: '8B',   ctx: '8K',  quant: 'Q8_0',   quality: 5, speed: 4 },
  { name: 'BGE-M3',             tag: 'Best Multilingual',      desc: 'Excellent multilingual retrieval across 100+ languages.', size: '438MB', params: '567M', ctx: '8K',  quant: 'Q4_K_M', quality: 5, speed: 4 },
  { name: 'Nomic Embed v1.5',   tag: 'Most Efficient',         desc: 'Tiny, fast, and great for local RAG.',                  size: '274MB',  params: '137M', ctx: '8K',  quant: 'Q8_0',   quality: 4, speed: 5 },
  { name: 'BGE Large EN v1.5',  tag: 'Strong English',         desc: 'Strong English-only retrieval at a small size.',        size: '208MB',  params: '335M', ctx: '512', quant: 'Q4_K_M', quality: 4, speed: 4 },
];

function Stars({ value }) {
  return (
    <div className="su-stars" aria-label={`${value} of 5`}>
      {[1, 2, 3, 4, 5].map((i) => (
        <svg key={i} className={i <= value ? 'is-on' : ''} viewBox="0 0 24 24" aria-hidden="true">
          <path d="M12 2.2l2.95 5.98 6.6.96-4.78 4.66 1.13 6.57L12 17.98 6.1 20.37l1.13-6.57L2.45 9.14l6.6-.96z" />
        </svg>
      ))}
    </div>
  );
}

function RateRow({ label, value }) {
  return (
    <div className="su-rate">
      <span className="su-rate-label">{label}</span>
      <Stars value={value} />
    </div>
  );
}

function ModelCard({ m }) {
  const [state, setState] = React.useState('idle'); // idle · queued
  const queued = state === 'queued';
  return (
    <article className={`su-model-card${m.rec ? ' is-rec' : ''}`}>
      <div className="su-model-top">
        <h3 className="su-model-name">
          {m.name}
          <a href="#" aria-label="Model details" onClick={(e) => e.preventDefault()}><Icon name="external-link" size={13} /></a>
        </h3>
        <span className={`su-tag${m.rec ? ' is-rec' : ''}`}>{m.tag}</span>
      </div>
      <p className="su-model-desc">{m.desc}</p>

      <div className="su-specs">
        <span className="su-spec"><span>Size</span><b>{m.size}</b></span>
        <span className="su-spec"><span>Params</span><b>{m.params}</b></span>
        <span className="su-spec"><span>Context</span><b>{m.ctx}</b></span>
        <span className="su-spec"><span>Quant</span><b>{m.quant}</b></span>
      </div>

      <div className="su-meters">
        <RateRow label="Quality" value={m.quality} />
        <RateRow label="Speed" value={m.speed} />
      </div>

      <button className={`su-btn su-dl is-block ${queued ? 'su-btn-secondary' : 'su-btn-primary'}`}
              onClick={() => setState(queued ? 'idle' : 'queued')}>
        {queued
          ? (<><Icon name="check" size={16} strokeWidth={3} /> Queued</>)
          : (<><Icon name="download" size={16} /> Download</>)}
      </button>
    </article>
  );
}

function ModelSection({ title, sub, models }) {
  return (
    <section className="su-card" style={{ background: 'transparent', border: 'none', boxShadow: 'none', overflow: 'visible' }}>
      <div className="su-section-head">
        <h2 className="su-section-title">{title}</h2>
        <p className="su-section-sub">{sub}</p>
      </div>
      <div className="su-model-grid">
        {models.map((m) => <ModelCard key={m.name} m={m} />)}
      </div>
    </section>
  );
}

function Step3() {
  return (
    <SetupShell current={2}>
      <div className="su-rise">
        <ModelSection
          title="Chat Models"
          sub="For conversations, content generation, summarization, and Q&A."
          models={CHAT_MODELS} />

        <div style={{ height: 36 }} />

        <ModelSection
          title="Embedding Models"
          sub="For RAG, semantic search, and document retrieval."
          models={EMBED_MODELS} />

        <div className="su-info" style={{ marginTop: 32 }}>
          <p>Downloads continue in the background while you finish setup. You can always add or remove models later from the Models page.</p>
        </div>

        <div className="su-nav">
          <a className="su-btn su-btn-ghost" href="setup-2-login.html">
            <Icon name="arrow-left" size={17} /> Back
          </a>
          <span className="su-nav-spacer" />
          <a className="su-btn su-btn-primary" href="setup-4-api-models.html">
            Continue <Icon name="arrow-right" size={17} />
          </a>
        </div>
      </div>
    </SetupShell>
  );
}

ReactDOM.createRoot(document.getElementById('root')).render(<Step3 />);
