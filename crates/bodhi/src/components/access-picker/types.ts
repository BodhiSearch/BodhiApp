/** A selectable resource (model or MCP) in the access picker. */
export interface AccessItem {
  id: string;
  label: string;
  /** Optional kind, surfaces a Local/API badge + type filter in the panel. */
  type?: 'local' | 'api';
  /** Optional secondary text (e.g. an MCP description). */
  meta?: string;
}

/** All current+future resources, or only the explicitly listed ids. */
export type AccessMode = 'all' | 'specific';
