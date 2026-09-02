// Settings store: theme (light/dark/system) + language (zh/en), persisted to
// localStorage and exposed via a Preact context. `t()` does message lookup +
// {placeholder} interpolation against the i18n catalog.

import { createContext, ComponentChildren } from 'preact';
import { useContext, useEffect, useState } from 'preact/hooks';
import { Lang, MsgKey, I18nParams, resolveI18n, readStoredLang, LANG_STORAGE_KEY } from './i18n';

export type Theme = 'light' | 'dark' | 'system';

/** Discrete app-zoom steps. Named rather than free-numeric so the control is
 *  a segmented picker like the others, and so a stored value is always sane. */
export type FontScale = 'small' | 'normal' | 'large' | 'xlarge';

const FONT_SCALE_FACTORS: Record<FontScale, number> = {
  // 1 is the design's own scale — the stylesheet's ladder already puts body
  // copy on the browser-default 16px — so this is a pure user preference and
  // no longer compensation for sizes that were too small to begin with.
  small: 0.875,
  normal: 1, // keep in sync with the `--app-font-scale` fallback in theme.css
  large: 1.125,
  xlarge: 1.25,
};

/** Which settings dialog to open from the sidebar settings menu. */
export type SettingsSection = 'theme' | 'language' | 'model' | 'remote' | 'notifications';

interface SettingsCtx {
  theme: Theme;
  setTheme: (t: Theme) => void;
  fontScale: FontScale;
  setFontScale: (s: FontScale) => void;
  lang: Lang;
  setLang: (l: Lang) => void;
  t: (key: MsgKey, params?: I18nParams) => string;
}

const Ctx = createContext<SettingsCtx | null>(null);

const THEME_KEY = 'rustcode.theme';
const FONT_SCALE_KEY = 'rustcode.fontScale';

function readTheme(): Theme {
  try {
    const v = localStorage.getItem(THEME_KEY);
    if (v === 'light' || v === 'dark' || v === 'system') return v;
  } catch {
    /* ignore */
  }
  // Default to the warm-ivory light theme (warm-ivory look) when the user
  // hasn't picked one; they can still switch to dark/system in settings.
  return 'light';
}

// Language persistence/read is shared with non-React modules via
// readStoredLang()/LANG_STORAGE_KEY in i18n.ts; the React context adds
// re-rendering on language switch on top of the same lookup.


function readFontScale(): FontScale {
  try {
    const v = localStorage.getItem(FONT_SCALE_KEY);
    if (v && v in FONT_SCALE_FACTORS) return v as FontScale;
  } catch {
    /* ignore */
  }
  return 'normal';
}

export function SettingsProvider({ children }: { children: ComponentChildren }) {
  const [theme, setThemeState] = useState<Theme>(readTheme);
  const [lang, setLangState] = useState<Lang>(readStoredLang);
  const [fontScale, setFontScaleState] = useState<FontScale>(readFontScale);

  // Apply theme to <html data-theme>; theme.css keys light/dark off this.
  useEffect(() => {
    document.documentElement.setAttribute('data-theme', theme);
    try {
      localStorage.setItem(THEME_KEY, theme);
    } catch {
      /* ignore */
    }
  }, [theme]);

  useEffect(() => {
    document.documentElement.setAttribute('lang', lang === 'zh' ? 'zh-CN' : 'en');
    try {
      localStorage.setItem(LANG_STORAGE_KEY, lang);
    } catch {
      /* ignore */
    }
  }, [lang]);

  // Drive `--app-font-scale`, which theme.css multiplies into the root
  // font-size. Every `font-size` in the stylesheet is a rem, so this scales
  // all text at once — on a wide display the 12-14px defaults are otherwise
  // unreadably small.
  useEffect(() => {
    document.documentElement.style.setProperty(
      '--app-font-scale',
      String(FONT_SCALE_FACTORS[fontScale]),
    );
    try {
      localStorage.setItem(FONT_SCALE_KEY, fontScale);
    } catch {
      /* ignore */
    }
  }, [fontScale]);

  /**
   * Translate a key for the current language. Lookup/interpolation live in
   * i18n.ts (resolveI18n) so non-React modules share the exact same behavior;
   * this closure additionally re-renders consumers when `lang` changes.
   */
  function t(key: MsgKey, params?: I18nParams): string {
    return resolveI18n(lang, key, params);
  }

  return (
    <Ctx.Provider
      value={{
        theme,
        setTheme: setThemeState,
        fontScale,
        setFontScale: setFontScaleState,
        lang,
        setLang: setLangState,
        t,
      }}
    >
      {children}
    </Ctx.Provider>
  );
}

export function useSettings(): SettingsCtx {
  const c = useContext(Ctx);
  if (!c) throw new Error('useSettings must be used within <SettingsProvider>');
  return c;
}

/** Convenience hook when a component only needs the translator. */
export function useT(): SettingsCtx['t'] {
  return useSettings().t;
}
