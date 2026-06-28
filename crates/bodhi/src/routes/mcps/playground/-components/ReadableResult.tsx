import { type ReactNode, Fragment, useState } from 'react';

import { ShellIcon } from '@/components/shell';
import type { McpPromptMessage } from '@/hooks/mcps/useMcpClient';

import type { Block, ResourceReadable, ToolReadable } from './playgroundTypes';

// ── helpers ─────────────────────────────────────────────────────

function prettyKey(k: string): string {
  if (!k) return '';
  const spaced = k.replace(/[_-]+/g, ' ').replace(/([a-z])([A-Z])/g, '$1 $2');
  return spaced.charAt(0).toUpperCase() + spaced.slice(1);
}

// ── inline markdown ─────────────────────────────────────────────

function inlineMd(text: string): ReactNode[] {
  const nodes: ReactNode[] = [];
  const re = /(\*\*[^*]+\*\*|`[^`]+`|\[[^\]]+\]\([^)]+\))/g;
  let last = 0;
  let m: RegExpExecArray | null;
  let i = 0;
  while ((m = re.exec(text))) {
    if (m.index > last) nodes.push(<Fragment key={i++}>{text.slice(last, m.index)}</Fragment>);
    const tok = m[0];
    if (tok.startsWith('**')) nodes.push(<strong key={i++}>{tok.slice(2, -2)}</strong>);
    else if (tok.startsWith('`'))
      nodes.push(
        <code key={i++} className="md-code">
          {tok.slice(1, -1)}
        </code>
      );
    else {
      const mm = tok.match(/^\[([^\]]+)\]\(([^)]+)\)$/);
      if (mm) {
        nodes.push(
          <a key={i++} className="md-link" href={mm[2]} target="_blank" rel="noopener noreferrer">
            {mm[1]}
          </a>
        );
      } else {
        nodes.push(<Fragment key={i++}>{tok}</Fragment>);
      }
    }
    last = m.index + tok.length;
  }
  if (last < text.length) nodes.push(<Fragment key={i++}>{text.slice(last)}</Fragment>);
  return nodes;
}

export function Markdownish({ text }: { text: string }) {
  const lines = String(text).split('\n');
  const out: ReactNode[] = [];
  let bullets: ReactNode[] | null = null;
  let i = 0;
  const flush = () => {
    if (bullets) {
      out.push(
        <ul className="md-ul" key={'ul' + out.length}>
          {bullets}
        </ul>
      );
      bullets = null;
    }
  };
  const parseRow = (r: string): string[] =>
    r
      .replace(/^\s*\|/, '')
      .replace(/\|\s*$/, '')
      .split('|')
      .map((c) => c.trim());

  while (i < lines.length) {
    const t = lines[i].trimEnd();
    if (/^```/.test(t.trim())) {
      flush();
      const buf: string[] = [];
      i++;
      while (i < lines.length && !/^```/.test(lines[i].trim())) {
        buf.push(lines[i]);
        i++;
      }
      i++;
      out.push(
        <pre className="md-pre" key={'cd' + out.length}>
          <code>{buf.join('\n')}</code>
        </pre>
      );
      continue;
    }
    if (t.includes('|') && i + 1 < lines.length && /-/.test(lines[i + 1]) && /^[\s:|-]+$/.test(lines[i + 1].trim())) {
      flush();
      const headers = parseRow(t);
      i += 2;
      const rows: string[][] = [];
      while (i < lines.length && lines[i].includes('|') && lines[i].trim() !== '') {
        rows.push(parseRow(lines[i]));
        i++;
      }
      out.push(
        <div className="md-table-wrap" key={'tb' + out.length}>
          <table className="md-table">
            <thead>
              <tr>
                {headers.map((h, hi) => (
                  <th key={hi}>{inlineMd(h)}</th>
                ))}
              </tr>
            </thead>
            <tbody>
              {rows.map((r, ri) => (
                <tr key={ri}>
                  {r.map((c, ci) => (
                    <td key={ci}>{inlineMd(c)}</td>
                  ))}
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      );
      continue;
    }
    if (/^#{1,3}\s/.test(t)) {
      flush();
      const lvl = (t.match(/^#+/) ?? ['#'])[0].length;
      out.push(
        <div className={'md-h md-h' + lvl} key={i}>
          {inlineMd(t.replace(/^#+\s/, ''))}
        </div>
      );
    } else if (/^>\s?/.test(t)) {
      flush();
      out.push(
        <blockquote className="md-quote" key={i}>
          {inlineMd(t.replace(/^>\s?/, ''))}
        </blockquote>
      );
    } else if (/^[-*]\s/.test(t)) {
      (bullets = bullets || []).push(<li key={i}>{inlineMd(t.replace(/^[-*]\s/, ''))}</li>);
    } else if (t === '') {
      flush();
    } else {
      flush();
      out.push(
        <p className="md-p" key={i}>
          {inlineMd(t)}
        </p>
      );
    }
    i++;
  }
  flush();
  return <div className="md">{out}</div>;
}

// ── recursive data view ─────────────────────────────────────────

function DataTable({ rows }: { rows: Array<Record<string, unknown>> }) {
  const cols: string[] = [];
  for (const r of rows) {
    for (const k of Object.keys(r)) if (!cols.includes(k)) cols.push(k);
  }
  return (
    <div className="pg-table-wrap">
      <table className="pg-table">
        <thead>
          <tr>
            {cols.map((c) => (
              <th key={c}>{prettyKey(c)}</th>
            ))}
          </tr>
        </thead>
        <tbody>
          {rows.map((r, i) => (
            <tr key={i}>
              {cols.map((c) => (
                <td key={c}>{r[c] == null ? <span className="dv-null">—</span> : <DataView value={r[c]} compact />}</td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

export function DataView({ value, compact }: { value: unknown; compact?: boolean }): JSX.Element {
  if (value === null || value === undefined) return <span className="dv-null">—</span>;
  if (typeof value === 'string')
    return compact ? <span className="dv-str">{value}</span> : <span className="dv-str dv-block">{value}</span>;
  if (typeof value === 'number') return <span className="dv-num">{String(value)}</span>;
  if (typeof value === 'boolean')
    return <span className={'dv-bool ' + (value ? 'on' : 'off')}>{value ? 'Yes' : 'No'}</span>;
  if (Array.isArray(value)) {
    if (value.length === 0) return <span className="dv-null">— empty —</span>;
    const allObj = value.every((v) => v && typeof v === 'object' && !Array.isArray(v));
    if (allObj && !compact) return <DataTable rows={value as Array<Record<string, unknown>>} />;
    return (
      <ul className="dv-list">
        {value.map((v, i) => (
          <li key={i}>
            <DataView value={v} compact={compact} />
          </li>
        ))}
      </ul>
    );
  }
  if (typeof value === 'object') {
    const entries = Object.entries(value as Record<string, unknown>);
    if (compact)
      return (
        <span className="dv-inline">
          {entries.map(([k, v]) => (
            <span key={k} className="dv-chip">
              {prettyKey(k)}: <DataView value={v} compact />
            </span>
          ))}
        </span>
      );
    return (
      <dl className="dv-obj">
        {entries.map(([k, v]) => (
          <div className="dv-row" key={k}>
            <dt className="dv-key">{prettyKey(k)}</dt>
            <dd className="dv-val">
              <DataView value={v} />
            </dd>
          </div>
        ))}
      </dl>
    );
  }
  return <span>{String(value)}</span>;
}

// ── prompt messages → chat bubbles ─────────────────────────────

function messageBody(content: unknown): string {
  if (typeof content === 'string') return content;
  if (Array.isArray(content)) {
    return content
      .map((c) =>
        c && typeof c === 'object' && 'text' in c && typeof (c as { text?: unknown }).text === 'string'
          ? (c as { text: string }).text
          : JSON.stringify(c, null, 2)
      )
      .join('\n\n');
  }
  if (content && typeof content === 'object' && 'text' in content) {
    const t = (content as { text?: unknown }).text;
    if (typeof t === 'string') return t;
  }
  return JSON.stringify(content, null, 2);
}

export function MessagesView({ messages }: { messages: McpPromptMessage[] }) {
  if (!messages.length) {
    return <div className="pg-noresult">The prompt returned no messages.</div>;
  }
  return (
    <div className="pg-messages">
      {messages.map((m, i) => (
        <div className={'pg-msg pg-msg-' + (m.role || 'user')} key={i} data-testid={`mcp-playground-prompt-msg-${i}`}>
          <div className="pg-msg-role">{m.role}</div>
          <div className="pg-msg-body">
            <Markdownish text={messageBody(m.content)} />
          </div>
        </div>
      ))}
    </div>
  );
}

// ── content block renderers ────────────────────────────────────

function TextBlock({ block }: { block: Extract<Block, { type: 'text' }> }) {
  const fmt = block.format || 'markdown';
  if (fmt === 'pre')
    return (
      <pre className="pg-pre">
        <code>{block.text}</code>
      </pre>
    );
  if (fmt === 'plain') return <div className="pg-plain">{block.text}</div>;
  return <Markdownish text={block.text} />;
}

function ImageBlock({ block }: { block: Extract<Block, { type: 'image' }> }) {
  const w = block.w || 240;
  const h = block.h || 150;
  return (
    <figure className="pg-img">
      <div
        className="pg-img-tile"
        style={{ aspectRatio: `${w} / ${h}` }}
        role="img"
        aria-label={block.alt || block.name}
      >
        <ShellIcon name="image" size={22} />
        <span className="pg-img-dims">
          {w}×{h}
        </span>
      </div>
      <figcaption className="pg-img-cap">
        <span className="mono">{block.name || 'image'}</span>
        <span className="pg-img-mime">{block.mimeType || 'image/png'}</span>
      </figcaption>
    </figure>
  );
}

export interface ResourceLinkProps {
  block: Extract<Block, { type: 'resource_link' }>;
  onOpen?: (uri: string, name?: string) => void;
}

export function ResourceLinkBlock({ block, onOpen }: ResourceLinkProps) {
  const handleClick = () => onOpen?.(block.uri, block.name);
  return (
    <button
      type="button"
      className="pg-rlink"
      onClick={handleClick}
      data-testid={`mcp-playground-resource-link-${block.uri}`}
    >
      <span className="pg-rlink-ico">
        <ShellIcon name="file-text" size={15} />
      </span>
      <span className="pg-rlink-body">
        <span className="pg-rlink-name">{block.name || block.uri}</span>
        {block.description && <span className="pg-rlink-desc">{block.description}</span>}
        <span className="pg-rlink-uri mono">{block.uri}</span>
      </span>
      <span className="pg-rlink-go">
        <ShellIcon name="arrow-up-right" size={14} />
      </span>
    </button>
  );
}

function ResourceBlock({ block }: { block: Extract<Block, { type: 'resource' }> }) {
  const r = block.resource;
  const isText = /text|markdown|json/.test(r.mimeType || '');
  const body = r.text ?? r.blob ?? '';
  return (
    <div className="pg-embed">
      <div className="pg-embed-head">
        <ShellIcon name="paperclip" size={12} />
        <span className="mono">{r.uri}</span>
        {r.mimeType && <span className="pg-mime">{r.mimeType}</span>}
      </div>
      {isText ? <Markdownish text={body} /> : <DataView value={body} />}
    </div>
  );
}

function ContentBlock({ block, onOpen }: { block: Block; onOpen?: (uri: string, name?: string) => void }) {
  if (block.type === 'image') return <ImageBlock block={block} />;
  if (block.type === 'resource_link') return <ResourceLinkBlock block={block} onOpen={onOpen} />;
  if (block.type === 'resource') return <ResourceBlock block={block} />;
  return <TextBlock block={block} />;
}

// ── structured content (collapsible) ───────────────────────────

function StructuredView({ data }: { data: unknown }) {
  if (data && typeof data === 'object' && !Array.isArray(data)) {
    const keys = Object.keys(data as object);
    const v = (data as Record<string, unknown>)[keys[0]];
    if (
      keys.length === 1 &&
      Array.isArray(v) &&
      v.length > 0 &&
      v.every((x) => x && typeof x === 'object' && !Array.isArray(x))
    ) {
      return (
        <div className="pg-structured">
          <div className="pg-structured-cap">{prettyKey(keys[0])}</div>
          <DataTable rows={v as Array<Record<string, unknown>>} />
        </div>
      );
    }
  }
  if (Array.isArray(data) && data.length > 0 && data.every((x) => x && typeof x === 'object' && !Array.isArray(x))) {
    return <DataTable rows={data as Array<Record<string, unknown>>} />;
  }
  return <DataView value={data} />;
}

function StructDetails({ data, defaultOpen }: { data: unknown; defaultOpen?: boolean }) {
  const [open, setOpen] = useState(defaultOpen !== false);
  return (
    <div className={'pg-struct-wrap' + (open ? ' open' : '')}>
      <button type="button" className="pg-struct-sum" onClick={() => setOpen((o) => !o)}>
        <ShellIcon name={open ? 'chevron-down' : 'chevron-right'} size={12} />
        <ShellIcon name="table-2" size={12} />
        <span>Structured data</span>
      </button>
      {open && (
        <div className="pg-struct-body">
          <StructuredView data={data} />
        </div>
      )}
    </div>
  );
}

// ── readable surfaces ──────────────────────────────────────────

export function ToolResultView({
  model,
  onOpenResource,
}: {
  model: ToolReadable;
  onOpenResource?: (uri: string, name?: string) => void;
}) {
  const blocks = model.content || [];
  const links = blocks.filter((b): b is Extract<Block, { type: 'resource_link' }> => b.type === 'resource_link');
  const nonLinks = blocks.filter((b) => b.type !== 'resource_link');
  return (
    <div className="pg-toolresult">
      {nonLinks.map((b, i) => (
        <ContentBlock key={i} block={b} onOpen={onOpenResource} />
      ))}
      {links.length > 0 && (
        <div className="pg-rlinks">
          {nonLinks.length > 0 && <div className="pg-rlinks-cap">Linked resources</div>}
          {links.map((b, i) => (
            <ResourceLinkBlock key={i} block={b} onOpen={onOpenResource} />
          ))}
        </div>
      )}
      {model.structuredContent !== undefined && <StructDetails data={model.structuredContent} defaultOpen={true} />}
      {blocks.length === 0 && model.structuredContent === undefined && (
        <div className="pg-noresult">The tool returned nothing.</div>
      )}
    </div>
  );
}

export function ResourceContentsView({ data }: { data: ResourceReadable }) {
  if (!data.contents.length) return <div className="pg-noresult">No contents returned.</div>;
  return (
    <div className="pg-resource-out">
      <div className="pg-resource-meta">
        <span className="pg-uri mono">{data.uri}</span>
        {data.mimeType && <span className="pg-mime">{data.mimeType}</span>}
      </div>
      {data.contents.map((c, i) => {
        const mime = c.mimeType || data.mimeType || '';
        const isText = /text|markdown|json/.test(mime);
        if (c.text != null && isText) return <Markdownish key={i} text={c.text} />;
        if (c.text != null) return <DataView key={i} value={c.text} />;
        if (c.blob != null)
          return (
            <div key={i} className="pg-blob">
              <ShellIcon name="paperclip" size={12} /> Binary contents ({mime || 'application/octet-stream'},{' '}
              {c.blob.length} chars base64)
            </div>
          );
        return <DataView key={i} value={c} />;
      })}
    </div>
  );
}
