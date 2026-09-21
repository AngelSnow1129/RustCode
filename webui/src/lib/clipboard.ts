// Clipboard writes that also work OUTSIDE a secure context.
//
// `navigator.clipboard` only exists in a SECURE CONTEXT (https, or http on
// localhost / 127.0.0.1). `rustcode webui` defaults to binding 0.0.0.0 and the
// page is routinely opened over a plain-HTTP LAN address, where that API is
// `undefined` (or throws NotAllowedError) — every copy button silently did
// nothing there. `document.execCommand('copy')` still works on those origins,
// so: modern API first, legacy path second, and a boolean the caller can turn
// into real feedback instead of a no-op.

export async function copyText(text: string): Promise<boolean> {
  if (!text) return false;
  try {
    const clipboard = typeof navigator === 'undefined' ? undefined : navigator.clipboard;
    if (clipboard && typeof clipboard.writeText === 'function') {
      await clipboard.writeText(text);
      return true;
    }
  } catch {
    // Permission denied / blocked by policy -> try the legacy path below.
  }
  return legacyCopyText(text);
}

/** Synchronous fallback for non-secure origins (and older Safari). */
function legacyCopyText(text: string): boolean {
  if (typeof document === 'undefined') return false;
  const area = document.createElement('textarea');
  area.value = text;
  area.setAttribute('readonly', '');
  // Keep it focusable and selectable but invisible; `display:none` would make
  // the selection empty and `execCommand('copy')` a no-op.
  area.style.position = 'fixed';
  area.style.top = '0';
  area.style.left = '0';
  area.style.width = '1px';
  area.style.height = '1px';
  area.style.opacity = '0';
  area.style.pointerEvents = 'none';
  document.body.appendChild(area);
  try {
    area.select();
    area.setSelectionRange(0, text.length);
    return document.execCommand('copy');
  } catch {
    return false;
  } finally {
    area.remove();
  }
}
