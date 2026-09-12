import React, { useState } from 'react';
import { postMessage } from '../vscode';
import { useChatContext } from '../state/ChatProvider';
import { MsgKey, useT } from '../i18n';

interface QuickAction {
  id: string;
  labelKey: MsgKey;
  icon: string;
}

const quickActions: QuickAction[] = [
  { id: 'intro', labelKey: 'welcome.quick.intro', icon: '[*]' },
  { id: 'projectOverview', labelKey: 'welcome.quick.projectOverview', icon: '[~]' },
  { id: 'improvements', labelKey: 'welcome.quick.improvements', icon: '[?]' },
  { id: 'devPlan', labelKey: 'welcome.quick.devPlan', icon: '[>]' },
  { id: 'configuration', labelKey: 'welcome.quick.configuration', icon: '[#]' },
  { id: 'tips', labelKey: 'welcome.quick.tips', icon: '[+]' },
];

export function WelcomeScreen() {
  const { state } = useChatContext();
  const t = useT();
  const [providerName, setProviderName] = useState('openai');
  const [providerType, setProviderType] = useState('openai');
  const [model, setModel] = useState('gpt-4o');
  // Start blank: the open build is vendor-neutral -- no provider endpoint is
  // pre-filled. Managed builds also leave the field to the user/sync flow.
  const [baseUrl, setBaseUrl] = useState('');
  const [apiKey, setApiKey] = useState('');

  function handleAction(action: string) {
    postMessage({ type: 'quickAction', action });
  }

  function submitProvider(e: React.FormEvent) {
    e.preventDefault();
    postMessage({
      type: 'providerCreate',
      provider: {
        name: providerName,
        type: providerType,
        model,
        base_url: baseUrl || undefined,
        api_key: apiKey || undefined,
        set_default: true,
      },
    });
    // Reset all form fields
    setProviderName('openai');
    setProviderType('openai');
    setModel('gpt-4o');
    setBaseUrl('');
    setApiKey('');
  }

  const needsSetup = state.setupRequired || state.providers.length === 0;
  // Local identity: no managed sign-in service. The bring-your-own-key
  // provider form is always the primary (and expanded) setup path.
  const manualFormExpanded = true;

  return (
    <div className="welcome-screen">
      <div className="welcome-content">
        <h1 className="welcome-title">RustCode</h1>
        <p className="welcome-subtitle">
          {needsSetup ? t('welcome.subtitle.setup') : t('welcome.subtitle.ready')}
        </p>

        {needsSetup && (
          <section className="setup-card">
            <div className="setup-step">
              <div className="setup-copy">
                <div className="setup-title">{t('setup.models')}</div>
                <div className="setup-subtitle">
                  {state.providers.length > 0
                    ? t('setup.providersConfigured', { count: state.providers.length })
                    : t('setup.addProviderHint')}
                </div>
              </div>
            </div>

            {manualFormExpanded && (
              <form className="provider-form" onSubmit={submitProvider}>
                <input value={providerName} onChange={(e) => setProviderName(e.target.value)} placeholder={t('setup.providerName')} />
                <input value={providerType} onChange={(e) => setProviderType(e.target.value)} placeholder={t('setup.providerType')} />
                <input value={model} onChange={(e) => setModel(e.target.value)} placeholder={t('setup.model')} />
                <input value={baseUrl} onChange={(e) => setBaseUrl(e.target.value)} placeholder={t('setup.baseUrl')} />
                <input value={apiKey} onChange={(e) => setApiKey(e.target.value)} placeholder={t('setup.apiKey')} type="password" />
                <button className="setup-primary setup-wide" type="submit">{t('setup.saveProvider')}</button>
              </form>
            )}

            {state.setupStatus && <div className="setup-status">{state.setupStatus}</div>}
            {state.setupError && <div className="setup-error">{state.setupError}</div>}
          </section>
        )}

        <div className="quick-actions">
          {quickActions.map((a) => (
            <button
              key={a.id}
              className="quick-action-card"
              onClick={() => handleAction(a.id)}
            >
              <span className="quick-action-icon">{a.icon}</span>
              <span className="quick-action-label">{t(a.labelKey)}</span>
            </button>
          ))}
        </div>
      </div>
    </div>
  );
}
