// Account / login state for the settings menu (the gear's "Account" entry).
// Self-contained: reads the webui token from the URL and calls /auth/*
// directly (decoupled from api.ts). Labels live in the shared i18n catalog;
// neutral/open builds hide the whole entry point (managed_available=false).

import { useEffect, useRef, useState } from 'preact/hooks';
import { useSettings } from '../settings';
import type { MsgKey } from '../i18n';

const TOKEN = new URLSearchParams(location.search).get('token') ?? '';
function authHeaders(): Record<string, string> {
  return TOKEN ? { Authorization: 'Bearer ' + TOKEN } : {};
}

export interface UserInfo {
  username: string;
  name?: string | null;
  email?: string | null;
  avatar_url?: string | null;
}

// Login/logout state + actions, consumed by the Sidebar settings menu.
export function useAuth() {
  const { t } = useSettings();
  // Keep the same `labels` shape consumers expect, but source every string
  // from the shared catalog so zh/en parity is type-checked with MsgKey.
  const label = (key: MsgKey) => t(key);
  const labels = {
    signIn: label('login.signIn'),
    signingIn: label('login.signingIn'),
    signOut: label('login.signOut'),
    hint: label('login.hint'),
    expired: label('login.expired'),
  };

  const [loggedIn, setLoggedIn] = useState(false);
  // Credentials exist but the token is dead (expired + unrefreshable). The
  // server now probes real usability, so the sidebar can stop claiming
  // "logged in" when chat would actually reject the token.
  const [expired, setExpired] = useState(false);
  // Whether this build ships a managed sign-in service at all
  // (`/auth/status` -> managed_available). Neutral / open builds report false:
  // the sidebar then hides the whole account area, since sign-in can only fail.
  const [managedAvailable, setManagedAvailable] = useState(false);
  const [user, setUser] = useState<UserInfo | null>(null);
  const [busy, setBusy] = useState(false);
  const busyRef = useRef(false);
  const managedRef = useRef(false);
  const pollTimer = useRef<number | null>(null);
  const loginGeneration = useRef(0);

  async function refresh(shouldApply: () => boolean = () => true) {
    try {
      const r = await fetch('/auth/status', { headers: authHeaders() });
      const s = await r.json();
      const managed = s.managed_available === true;
      managedRef.current = managed;
      if (!shouldApply()) return;
      setManagedAvailable(managed);
      setLoggedIn(!!s.logged_in);
      setExpired(!!s.expired);
      setUser(s.user ?? null);
    } catch {
      /* ignore */
    }
  }

  useEffect(() => {
    let active = true;
    const refreshWhileMounted = () => {
      // In a neutral build no managed service exists, so status can never
      // flip to logged-in -- poll once on mount (to learn the flag) and skip
      // the 2s churn afterwards.
      if (active && managedRef.current) void refresh(() => active);
    };
    void refresh(() => active);
    const interval = window.setInterval(refreshWhileMounted, 2_000);
    const onVisibility = () => {
      if (document.visibilityState === 'visible') refreshWhileMounted();
    };
    document.addEventListener('visibilitychange', onVisibility);
    return () => {
      active = false;
      window.clearInterval(interval);
      document.removeEventListener('visibilitychange', onVisibility);
      loginGeneration.current += 1;
      if (pollTimer.current !== null) clearTimeout(pollTimer.current);
    };
  }, []);

  async function startLogin() {
    // Fail closed: a neutral build has no sign-in service. The entry points
    // are hidden, but never drive the user into a 500 from a stale UI.
    if (busyRef.current || !managedRef.current) return;
    busyRef.current = true;
    const generation = ++loginGeneration.current;
    setBusy(true);
    try {
      const r = await fetch('/auth/login/start', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json', ...authHeaders() },
        body: JSON.stringify({ open_browser: false }),
      });
      if (!r.ok) throw new Error(`Login start failed: ${r.status}`);
      const start = await r.json();
      if (start?.url) window.open(start.url, '_blank', 'noopener');
      const id = start?.login_id;
      if (!id) {
        busyRef.current = false;
        setBusy(false);
        return;
      }
      const deadline = Date.now() + Math.max(1, start.expires_in_seconds ?? 600) * 1000;
      const schedule = (delayMs: number) => {
        if (loginGeneration.current !== generation) return;
        pollTimer.current = window.setTimeout(() => void poll(), Math.max(100, delayMs));
      };
      const poll = async () => {
        if (loginGeneration.current !== generation) return;
        if (Date.now() >= deadline) {
          await fetch(`/auth/login/${encodeURIComponent(id)}`, {
            method: 'DELETE',
            headers: authHeaders(),
          }).catch(() => undefined);
          stopPolling();
          return;
        }
        try {
          const response = await fetch(`/auth/login/${encodeURIComponent(id)}/poll`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json', ...authHeaders() },
          });
          const result = await response.json().catch(() => ({}));
          if (!response.ok) {
            if (result.retryable === true && Date.now() < deadline) {
              schedule(2000);
            } else {
              stopPolling();
            }
            return;
          }
          if (result.status === 'pending') {
            schedule(result.retry_after_ms ?? 2000);
            return;
          }
          if (result.status === 'authorized') {
            await refresh();
            stopPolling();
            return;
          }
          // expired / cancelled / failed are terminal login states.
          stopPolling();
        } catch {
          if (Date.now() < deadline) schedule(2000); else stopPolling();
        }
      };
      await poll();
    } catch {
      busyRef.current = false;
      setBusy(false);
    }
  }

  function stopPolling() {
    busyRef.current = false;
    loginGeneration.current += 1;
    if (pollTimer.current !== null) {
      clearTimeout(pollTimer.current);
      pollTimer.current = null;
    }
    setBusy(false);
  }

  async function doLogout() {
    stopPolling();
    try {
      await fetch('/auth/logout', { method: 'POST', headers: authHeaders() });
    } catch {
      /* ignore */
    }
    setLoggedIn(false);
    setExpired(false);
    setUser(null);
  }

  return { loggedIn, expired, managedAvailable, user, busy, labels, startLogin, doLogout };
}
