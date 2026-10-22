import React, { useEffect, useState } from 'react';
import type { GameState, ResourceForecast as Forecast } from '../types/game';
import { getLabel } from '../types/game';
import { getResourceForecast } from '../services/tauriApi';
import { formatCalendarDay } from '../utils/calendar';

export const ResourceForecast: React.FC<{ state: GameState }> = ({ state }) => {
  const [forecast, setForecast] = useState<Forecast | null>(null);
  const [error, setError] = useState('');
  useEffect(() => {
    let cancelled = false;
    setForecast(null);
    setError('');
    getResourceForecast().then((result) => {
      if (!cancelled) setForecast(result);
    }).catch((reason) => {
      if (!cancelled) setError(String(reason));
    });
    return () => { cancelled = true; };
  }, [state]);
  const text = (key: string, fallback: string) => getLabel(state.catalog, key, fallback);
  const date = (day: number) => formatCalendarDay(day, state.days_per_year, state.catalog.labels.values);
  const value = (id: string, amount: number) =>
    `${id === state.catalog.resource_roles?.currency ? text('currency_symbol', '') : ''}${amount.toLocaleString(undefined, { maximumFractionDigits: 2 })}`;
  return (
    <section style={{ padding: '1.25rem', border: '1px solid var(--surface-border)', borderRadius: 12, background: 'var(--surface-background)' }}>
      <h2>{text('forecast_title', '30-day resource forecast')}</h2>
      <p>{text('forecast_assumptions', 'Projection of existing commitments, with no new actions or random incidents. Income depends on meeting obligations; this is not a guarantee. Discretionary spending and future result rewards are excluded.')}</p>
      {error && <p role="alert" style={{ color: 'var(--danger-text)' }}>{text('forecast_error', 'Unable to calculate forecast')}: {error}</p>}
      {!forecast && !error && <p role="status">{text('planning_loading', 'Calculating current rules...')}</p>}
      {forecast && (
        <>
          <p>{date(forecast.start_day)} — {date(forecast.through_day)}</p>
          {forecast.terminal_day !== null && (
            <p role="alert" style={{ color: 'var(--danger-text)' }}>
              {text('forecast_terminal', 'Under these assumptions, the character reaches a terminal state on')} {date(forecast.terminal_day)}.
            </p>
          )}
          {forecast.resources.length === 0 ? <p>{text('forecast_no_resources', 'This dataset has no configured currency or recovery resource.')}</p> : (
            <div style={{ overflowX: 'auto' }}>
              <table style={{ width: '100%', textAlign: 'left', borderCollapse: 'collapse' }}>
                <thead><tr>
                  {[
                    text('planning_resource', 'Resource'), text('forecast_current', 'Current'),
                    text('forecast_end', 'Projected end'), text('forecast_minimum', 'Lowest daily balance'),
                  ].map((heading) => <th key={heading} style={{ padding: 8 }}>{heading}</th>)}
                </tr></thead>
                <tbody>{forecast.resources.map((resource) => (
                  <tr key={resource.id}>
                    <th style={{ padding: 8 }}>{resource.name}</th>
                    <td style={{ padding: 8 }}>{value(resource.id, resource.current)}</td>
                    <td style={{ padding: 8 }}>{value(resource.id, resource.projected)}</td>
                    <td style={{ padding: 8, color: resource.minimum < 0 ? 'var(--danger-text)' : undefined }}>
                      {value(resource.id, resource.minimum)}
                      {resource.first_negative_day !== null && ` (${date(resource.first_negative_day)})`}
                    </td>
                  </tr>
                ))}</tbody>
              </table>
            </div>
          )}
          <details>
            <summary>{text('forecast_daily', 'Daily balances, changes and consequences')}</summary>
            <div style={{ overflowX: 'auto', maxHeight: '26rem' }}>
              <table style={{ width: '100%', textAlign: 'left' }}>
                <thead><tr><th>{text('planning_date', 'Date')}</th>
                  {forecast.resources.map((resource) => <th key={resource.id}>{resource.name}</th>)}
                  <th>{text('planning_notes', 'Details')}</th>
                </tr></thead>
                <tbody>{forecast.days.map((day) => (
                  <tr key={day.day}>
                    <td>{date(day.day)}</td>
                    {forecast.resources.map((resource) => <td key={resource.id}>
                      {value(resource.id, day.values[resource.id])}
                      {' '}({day.changes[resource.id] >= 0 ? '+' : ''}{day.changes[resource.id].toLocaleString(undefined, { maximumFractionDigits: 2 })})
                    </td>)}
                    <td>{day.notes.join(' · ')}</td>
                  </tr>
                ))}</tbody>
              </table>
            </div>
          </details>
        </>
      )}
    </section>
  );
};
