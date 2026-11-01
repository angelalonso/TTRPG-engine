import React, { useEffect, useState } from 'react';
import type { GameState, ResourceForecast as Forecast } from '../types/game';
import { getLabel } from '../types/game';
import { getResourceForecast } from '../services/tauriApi';

const HINT_FORECAST_DAYS = 30;
const LOW_RESOURCE_THRESHOLD = 5;

export const ResourceForecast: React.FC<{ state: GameState }> = ({ state }) => {
  const [forecast, setForecast] = useState<Forecast | null>(null);
  const [error, setError] = useState('');

  useEffect(() => {
    let cancelled = false;
    setForecast(null);
    setError('');
    getResourceForecast(HINT_FORECAST_DAYS).then((result) => {
      if (!cancelled) setForecast(result);
    }).catch((reason) => {
      if (!cancelled) setError(String(reason));
    });
    return () => { cancelled = true; };
  }, [state]);

  const text = (key: string, fallback: string) => getLabel(state.catalog, key, fallback);
  const currencyId = state.catalog.resource_roles?.currency;
  const recoveryId = state.catalog.resource_roles?.recovery;
  const lowResources = forecast?.resources.filter((resource) =>
    (resource.id === currencyId || resource.id === recoveryId) &&
    resource.minimum <= LOW_RESOURCE_THRESHOLD,
  ) ?? [];
  const resourceName = (id: string) => {
    if (id === currencyId) return text('budget_name', 'Budget');
    if (id === recoveryId) return text('stamina_name', 'Stamina');
    return id;
  };

  return (
    <section style={{ padding: '1.25rem', border: '1px solid var(--surface-border)', borderRadius: 12, background: 'var(--surface-background)' }}>
      <h2>Hints</h2>
      {error && <p role="alert" style={{ color: 'var(--danger-text)' }}>{text('forecast_error', 'Unable to calculate forecast')}: {error}</p>}
      {!forecast && !error && <p role="status">{text('planning_loading', 'Calculating current rules...')}</p>}
      {forecast && lowResources.length === 0 && <p>{text('hints_none', 'No hints for the next 30 days.')}</p>}
      {lowResources.map((resource) => (
        <p key={resource.id} role="alert" style={{ color: 'var(--danger-text)' }}>
          {text('hints_low_resource', 'You will run low of')} {resourceName(resource.id)} {text('hints_low_resource_suffix', 'in the next 30 days; you should double-check that.')}
        </p>
      ))}
    </section>
  );
};
