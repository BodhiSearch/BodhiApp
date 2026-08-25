import { API_FORMAT_PRESETS } from '@/schemas/apiModel';

/** Subset of the form values the pure test/fetch gating logic needs. */
export interface ApiModelGatingInput {
  api_format?: string;
  base_url?: string;
  models?: string[];
  llm_liberty_envelope?: string;
}

/** True when creating an LLM Liberty OAuth alias, where the pasted envelope replaces base_url/api_key/extras. */
export function isLlmLibertyCreate(apiFormat: string | undefined, isEditMode: boolean): boolean {
  return apiFormat === 'llm_liberty_oauth' && !isEditMode;
}

export function hasEnvelope(envelope: string | undefined): boolean {
  return Boolean(envelope?.trim());
}

export function computeCanTest(values: ApiModelGatingInput, isEditMode: boolean): boolean {
  if (isLlmLibertyCreate(values.api_format, isEditMode)) {
    return hasEnvelope(values.llm_liberty_envelope) && Boolean(values.models?.[0]);
  }
  return Boolean(values.base_url);
}

export function computeCanFetch(values: ApiModelGatingInput, isEditMode: boolean): boolean {
  if (isLlmLibertyCreate(values.api_format, isEditMode)) {
    return hasEnvelope(values.llm_liberty_envelope);
  }
  return Boolean(values.base_url);
}

/** Returns null when the generic (non-liberty) path should compute the disabled reason instead. */
export function llmLibertyTestDisabledReason(values: ApiModelGatingInput, isEditMode: boolean): string | null {
  if (!isLlmLibertyCreate(values.api_format, isEditMode)) return null;
  if (!hasEnvelope(values.llm_liberty_envelope)) {
    return 'Paste the LLM Liberty envelope to test connection';
  }
  if (!values.models?.[0]) {
    return 'You need to add at least one model to test connection';
  }
  return '';
}

/** Returns null when the generic (non-liberty) path should compute the disabled reason instead. */
export function llmLibertyFetchDisabledReason(values: ApiModelGatingInput, isEditMode: boolean): string | null {
  if (!isLlmLibertyCreate(values.api_format, isEditMode)) return null;
  return hasEnvelope(values.llm_liberty_envelope) ? '' : 'Paste the LLM Liberty envelope to fetch models';
}

/** Whether the Extras section should show, i.e. the selected preset declares default headers/body. */
export function computeShowExtras(apiFormat: string | undefined): boolean {
  if (!apiFormat) return false;
  const preset = API_FORMAT_PRESETS[apiFormat as keyof typeof API_FORMAT_PRESETS];
  if (!preset) return false;
  return 'defaultHeaders' in preset || 'defaultBody' in preset;
}

/** Serialized default headers/body for a preset ('' when the preset declares no defaults). */
export function presetExtras(apiFormat: string): { headers: string; body: string } {
  const preset = API_FORMAT_PRESETS[apiFormat as keyof typeof API_FORMAT_PRESETS];
  const headers =
    preset && 'defaultHeaders' in preset && preset.defaultHeaders ? JSON.stringify(preset.defaultHeaders, null, 2) : '';
  const body =
    preset && 'defaultBody' in preset && preset.defaultBody ? JSON.stringify(preset.defaultBody, null, 2) : '';
  return { headers, body };
}

/** Base URL the preset seeds on format change ('' when the preset has none). */
export function presetBaseUrl(apiFormat: string): string {
  const preset = API_FORMAT_PRESETS[apiFormat as keyof typeof API_FORMAT_PRESETS];
  return preset?.baseUrl ?? '';
}
