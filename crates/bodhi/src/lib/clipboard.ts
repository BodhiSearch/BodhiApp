export function getClipboardUnavailableMessage(): string | null {
  if (typeof window === 'undefined') return null;
  const { hostname, port, protocol } = window.location;
  if (hostname === '0.0.0.0') {
    const portSuffix = port ? `:${port}` : '';
    return `Clipboard is disabled on ${protocol}//0.0.0.0${portSuffix}/. Open the app at ${protocol}//localhost${portSuffix}/ instead.`;
  }
  return null;
}

export async function copyToClipboard(text: string): Promise<void> {
  if (typeof navigator !== 'undefined' && navigator.clipboard?.writeText) {
    try {
      await navigator.clipboard.writeText(text);
      return;
    } catch {
      // fall through to legacy path (e.g. permission denied in iframe)
    }
  }

  if (typeof document === 'undefined') {
    throw new Error('Clipboard not available');
  }

  const textarea = document.createElement('textarea');
  textarea.value = text;
  textarea.setAttribute('readonly', '');
  textarea.style.position = 'fixed';
  textarea.style.top = '0';
  textarea.style.left = '0';
  textarea.style.opacity = '0';
  textarea.style.pointerEvents = 'none';
  document.body.appendChild(textarea);

  const selection = document.getSelection();
  const previousRange = selection && selection.rangeCount > 0 ? selection.getRangeAt(0) : null;

  textarea.focus();
  textarea.select();
  textarea.setSelectionRange(0, text.length);

  let succeeded = false;
  try {
    succeeded = document.execCommand('copy');
  } finally {
    document.body.removeChild(textarea);
    if (previousRange && selection) {
      selection.removeAllRanges();
      selection.addRange(previousRange);
    }
  }

  if (!succeeded) {
    throw new Error('Clipboard copy command was rejected');
  }
}
