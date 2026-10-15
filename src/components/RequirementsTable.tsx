import React, { useMemo, useState } from 'react';
import { openExternalUrl } from '../services/tauriApi';

export type RequirementModType = 'Track' | 'Car' | 'Other';

export interface RequirementMod {
  name: string;
  type: RequirementModType;
  url: string;
}

const URL_PATTERN = /(https?:\/\/[^\s<]+)/i;

function sectionType(name: string): RequirementModType {
  const normalized = name.toLowerCase();
  if (normalized.includes('track')) return 'Track';
  if (
    normalized.includes('car')
    || normalized.includes('formula')
    || normalized.includes('caterham')
    || normalized.includes('tvr')
    || normalized.includes('ginetta')
    || normalized.includes('btcc')
  ) return 'Car';
  return 'Other';
}

export function parseRequirementMods(text: string): RequirementMod[] {
  const rows: RequirementMod[] = [];
  let currentType: RequirementModType = 'Other';

  for (const sourceLine of text.split(/\r?\n/)) {
    const line = sourceLine.trim();
    if (!line || line.startsWith('If you want') || line.startsWith('On top of that')) continue;

    const section = line.replace(/^-+\s*/, '').replace(/:\s*$/, '').trim();
    const urlMatch = line.match(URL_PATTERN);
    const url = urlMatch?.[1].replace(/[),.;!?]+$/, '') ?? '';
    const withoutUrl = url ? line.replace(urlMatch?.[0] ?? '', '').trim() : line;
    const isSection = line.startsWith('- ') && (!url || withoutUrl.replace(/^-+\s*/, '').trim().length === 0);

    if (isSection) {
      currentType = sectionType(section);
      continue;
    }

    const parts = withoutUrl.split(/\t+|\s{2,}/).map((part) => part.trim()).filter(Boolean);
    const name = (parts[0] || withoutUrl)
      .replace(/^-+\s*/, '')
      .replace(/["“”]/g, '')
      .trim();
    if (!name || name.startsWith('If you')) continue;

    rows.push({ name, type: currentType, url });
  }

  return rows;
}

interface RequirementsTableProps {
  text: string;
}

export const RequirementsTable: React.FC<RequirementsTableProps> = ({ text }) => {
  const [query, setQuery] = useState('');
  const rows = useMemo(() => parseRequirementMods(text), [text]);
  const filteredRows = useMemo(() => {
    const normalizedQuery = query.trim().toLowerCase();
    if (!normalizedQuery) return rows;
    return rows.filter((row) =>
      `${row.name} ${row.type} ${row.url}`.toLowerCase().includes(normalizedQuery),
    );
  }, [query, rows]);

  return (
    <div style={styles.wrapper}>
      <label style={styles.searchLabel} htmlFor="gtr2-requirements-search">
        Search mods
      </label>
      <input
        id="gtr2-requirements-search"
        type="search"
        value={query}
        onChange={(event) => setQuery(event.target.value)}
        placeholder="Search by name, type, or URL"
        style={styles.search}
      />
      <div style={styles.tableWrapper}>
        <table style={styles.table}>
          <thead>
            <tr>
              <th style={styles.heading}>Name</th>
              <th style={styles.heading}>Type</th>
              <th style={styles.heading}>Download URL</th>
            </tr>
          </thead>
          <tbody>
            {filteredRows.map((row) => (
              <tr key={`${row.type}-${row.name}-${row.url}`}>
                <td style={styles.cell}>{row.name}</td>
                <td style={styles.cell}>{row.type}</td>
                <td style={styles.cell}>
                  {row.url ? (
                    <a
                      href={row.url}
                      target="_blank"
                      rel="noreferrer"
                      onClick={(event) => {
                        event.preventDefault();
                        event.stopPropagation();
                        void openExternalUrl(row.url);
                      }}
                      style={styles.link}
                    >
                      {row.url}
                    </a>
                  ) : (
                    <span style={styles.missing}>Not provided</span>
                  )}
                </td>
              </tr>
            ))}
            {filteredRows.length === 0 && (
              <tr>
                <td style={styles.empty} colSpan={3}>No matching mods.</td>
              </tr>
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
};

const styles: Record<string, React.CSSProperties> = {
  wrapper: { display: 'grid', gap: '0.5rem' },
  searchLabel: { fontWeight: 700 },
  search: {
    width: '100%',
    boxSizing: 'border-box',
    padding: '0.7rem 0.8rem',
    border: '1px solid #64748b',
    borderRadius: 6,
    background: '#0f172a',
    color: '#fff',
    font: 'inherit',
  },
  tableWrapper: { maxHeight: '55vh', overflow: 'auto', border: '1px solid #475569' },
  table: { width: '100%', borderCollapse: 'collapse', textAlign: 'left' },
  heading: {
    position: 'sticky',
    top: 0,
    padding: '0.7rem',
    background: '#1e293b',
    borderBottom: '1px solid #64748b',
    fontWeight: 700,
  },
  cell: { padding: '0.65rem 0.7rem', borderBottom: '1px solid #334155', verticalAlign: 'top' },
  empty: { padding: '1rem', textAlign: 'center', color: '#cbd5e1' },
  link: { color: '#93c5fd', textDecoration: 'underline', overflowWrap: 'anywhere' },
  missing: { color: '#cbd5e1', fontStyle: 'italic' },
};
