import React, { useEffect, useMemo, useState } from 'react';
import { listDatasetTables, saveDatasetTable, type DatasetTable } from '../services/tauriApi';

interface DatasetEditorProps {
  datasetPath: string;
  onExit: () => void;
}

type Selection = { row: number; column: string } | null;

const preferredOrder = [
  'config.csv',
  'colors.csv',
  'events.csv',
  'sponsors.csv',
  'quests.csv',
  'objects.csv',
  'costs.csv',
  'cost_rules.csv',
  'cost_rule_conditions.csv',
];

const tableDescription = (file: string) => {
  if (file === 'config.csv') return 'Labels, navigation names, calendar settings, and icon paths.';
  if (file === 'colors.csv') return 'Theme colors used by the main application.';
  if (file === 'events.csv') return 'Races, actions, activities, and their references.';
  if (file === 'sponsors.csv') return 'Sponsor tiers, interests, and proposal defaults.';
  if (file.includes('cost')) return 'Reusable costs and the rules or conditions that reference them.';
  return 'Raw dataset table. Changes are written back to this CSV.';
};

export const DatasetEditor: React.FC<DatasetEditorProps> = ({ datasetPath, onExit }) => {
  const [tables, setTables] = useState<DatasetTable[]>([]);
  const [selectedFile, setSelectedFile] = useState('events.csv');
  const [selection, setSelection] = useState<Selection>(null);
  const [hiddenColumns, setHiddenColumns] = useState<Record<string, string[]>>({});
  const [editing, setEditing] = useState(false);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState('');

  useEffect(() => {
    let cancelled = false;
    listDatasetTables(datasetPath)
      .then((loaded) => {
        if (cancelled) return;
        const sorted = [...loaded].sort((left, right) => {
          const leftIndex = preferredOrder.indexOf(left.file);
          const rightIndex = preferredOrder.indexOf(right.file);
          return (leftIndex < 0 ? 999 : leftIndex) - (rightIndex < 0 ? 999 : rightIndex)
            || left.file.localeCompare(right.file);
        });
        setTables(sorted);
        if (!sorted.some((table) => table.file === selectedFile)) setSelectedFile(sorted[0]?.file || '');
      })
      .catch((caught) => setError(String(caught)));
    return () => { cancelled = true; };
  }, [datasetPath, selectedFile]);

  const table = tables.find((entry) => entry.file === selectedFile) || tables[0];
  const visibleHeaders = table
    ? table.headers.filter((header) => !(hiddenColumns[table.file] || []).includes(header))
    : [];
  const selectedRow = table && selection ? table.rows[selection.row] : undefined;
  const selectedValue = selectedRow && selection ? selectedRow[selection.column] || '' : '';

  const referenceOptions = useMemo(() => {
    if (!table || !selection) return [];
    const column = selection.column.toLowerCase();
    if (!(column.endsWith('_id') || column.endsWith('_ids') || column.includes('type') || column === 'tags')) {
      return [];
    }
    const values = new Set<string>();
    if (column.includes('type') || column === 'tags') {
      table.rows.forEach((row) => (row[selection.column] || '').split(';').forEach((value) => {
        if (value.trim()) values.add(value.trim());
      }));
    }
    tables.forEach((candidate) => {
      const idHeader = candidate.headers.includes('id') ? 'id' : candidate.headers.includes('variable') ? 'variable' : '';
      if (idHeader && (column.endsWith('_id') || column.endsWith('_ids'))) {
        candidate.rows.forEach((row) => {
          if (row[idHeader]?.trim()) values.add(row[idHeader].trim());
        });
      }
    });
    return Array.from(values).sort();
  }, [selection, table, tables]);

  const updateSelectedValue = (value: string) => {
    if (!table || !selection) return;
    setTables((current) => current.map((candidate) => {
      if (candidate.file !== table.file) return candidate;
      return {
        ...candidate,
        rows: candidate.rows.map((row, index) =>
          index === selection.row ? { ...row, [selection.column]: value } : row),
      };
    }));
  };

  const saveTable = async () => {
    if (!table) return;
    setSaving(true);
    setError('');
    try {
      await saveDatasetTable(datasetPath, table.file, table.headers, table.rows);
    } catch (caught) {
      setError(String(caught));
    } finally {
      setSaving(false);
    }
  };

  const saveAllTables = async (): Promise<boolean> => {
    setSaving(true);
    setError('');
    try {
      await Promise.all(tables.map((entry) =>
        saveDatasetTable(datasetPath, entry.file, entry.headers, entry.rows)));
      return true;
    } catch (caught) {
      setError(String(caught));
      return false;
    } finally {
      setSaving(false);
    }
  };

  const addRow = () => {
    if (!table) return;
    const row = Object.fromEntries(table.headers.map((header) => [header, '']));
    setTables((current) => current.map((candidate) =>
      candidate.file === table.file ? { ...candidate, rows: [...candidate.rows, row] } : candidate));
    setSelection({ row: table.rows.length, column: table.headers[0] || '' });
  };

  const deleteSelectedRow = () => {
    if (!table || !selection) return;
    setTables((current) => current.map((candidate) =>
      candidate.file === table.file
        ? { ...candidate, rows: candidate.rows.filter((_, index) => index !== selection.row) }
        : candidate));
    setSelection(null);
  };

  const toggleColumn = (column: string) => {
    if (!table) return;
    setHiddenColumns((current) => {
      const hidden = new Set(current[table.file] || []);
      if (hidden.has(column)) hidden.delete(column);
      else if (hidden.size < table.headers.length - 1) hidden.add(column);
      return { ...current, [table.file]: Array.from(hidden) };
    });
  };

  return (
    <div style={styles.root}>
      <header style={styles.header}>
        <div>
          <strong>Dataset Editor</strong>
          <span style={styles.path}>{datasetPath}</span>
        </div>
        <div style={styles.headerActions}>
          <button onClick={() => setSelectedFile('colors.csv')} disabled={!tables.some((entry) => entry.file === 'colors.csv')}>
            Edit colors
          </button>
          <button onClick={() => setSelectedFile('config.csv')} disabled={!tables.some((entry) => entry.file === 'config.csv')}>
            Edit calendar / icons
          </button>
        <button
          style={styles.saveExit}
          onClick={async () => {
            if (await saveAllTables()) onExit();
          }}
          disabled={saving}
        >
          {saving ? 'Saving...' : 'Save and Exit'}
        </button>
        </div>
      </header>
      <div style={styles.body}>
        <aside style={styles.sidebar}>
          <h2>Tables</h2>
          {tables.map((entry) => (
            <button
              key={entry.file}
              style={entry.file === table?.file ? styles.selectedTable : styles.tableButton}
              onClick={() => { setSelectedFile(entry.file); setSelection(null); setEditing(false); }}
            >
              <strong>{entry.file}</strong>
              <small>{entry.rows.length} rows</small>
            </button>
          ))}
        </aside>
        {table ? (
          <main style={styles.workspace}>
            <section style={styles.editorPane}>
              <h1>{table.file}</h1>
              <p style={styles.muted}>{tableDescription(table.file)}</p>
              {selection && selectedRow ? (
                <>
                  <label style={styles.label}>
                    Column
                    <select
                      style={styles.input}
                      value={selection.column}
                      onChange={(event) => setSelection({ ...selection, column: event.target.value })}
                    >
                      {table.headers.map((header) => <option key={header}>{header}</option>)}
                    </select>
                  </label>
                  <label style={styles.label}>
                    Value
                    <textarea
                      style={styles.textarea}
                      value={selectedValue}
                      onChange={(event) => updateSelectedValue(event.target.value)}
                      autoFocus
                    />
                  </label>
                  {referenceOptions.length > 0 && (
                    <div>
                      <span style={styles.label}>Existing values</span>
                      <div style={styles.referenceList}>
                        {referenceOptions.slice(0, 80).map((option) => (
                          <button
                            key={option}
                            style={styles.referenceButton}
                            onClick={() => updateSelectedValue(
                              selection.column.toLowerCase().endsWith('_ids')
                                ? [selectedValue, option].filter(Boolean).join(';')
                                : option,
                            )}
                          >
                            {option}
                          </button>
                        ))}
                      </div>
                    </div>
                  )}
                  <div style={styles.actions}>
                    <button onClick={() => void saveTable()} disabled={saving}>Save table</button>
                    <button onClick={deleteSelectedRow} style={styles.danger}>Delete row</button>
                  </div>
                </>
              ) : (
                <p style={styles.muted}>Select a cell in the table to edit it.</p>
              )}
              <button onClick={addRow}>Add row</button>
              <h3>Visible columns</h3>
              <div style={styles.columnList}>
                {table.headers.map((header) => (
                  <label key={header}>
                    <input
                      type="checkbox"
                      checked={!hiddenColumns[table.file]?.includes(header)}
                      onChange={() => toggleColumn(header)}
                    />
                    {header}
                  </label>
                ))}
              </div>
            </section>
            <section style={styles.tablePane}>
              <div style={styles.tableHeader}>
                <strong>{table.file}</strong>
                <span>{table.rows.length} rows</span>
              </div>
              <div style={styles.tableScroll}>
                <table>
                  <thead>
                    <tr>
                      <th>#</th>
                      {visibleHeaders.map((header) => <th key={header}>{header}</th>)}
                    </tr>
                  </thead>
                  <tbody>
                    {table.rows.map((row, rowIndex) => (
                      <tr key={`${table.file}-${rowIndex}`}>
                        <td>{rowIndex + 1}</td>
                        {visibleHeaders.map((header) => (
                          <td key={header}>
                            <button
                              style={styles.cellButton}
                              onClick={() => { setSelection({ row: rowIndex, column: header }); setEditing(true); }}
                            >
                              {row[header]}
                            </button>
                          </td>
                        ))}
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </section>
          </main>
        ) : <p style={styles.muted}>No CSV tables were found.</p>}
      </div>
      {editing && table && selection && selectedRow && (
        <div style={styles.overlay} role="dialog" aria-modal="true">
          <div style={styles.dialog}>
            <div style={styles.dialogHeader}>
              <h2>Edit row {selection.row + 1} in {table.file}</h2>
              <button onClick={() => setEditing(false)}>Close</button>
            </div>
            <p style={styles.muted}>
              Every column is editable. Reference-like columns include quick selectors in the left pane.
            </p>
            <div style={styles.rowEditor}>
              {table.headers.map((header) => (
                <label key={header} style={styles.label}>
                  {header}
                  <textarea
                    style={styles.rowInput}
                    value={selectedRow[header] || ''}
                    onChange={(event) => {
                      setSelection({ row: selection.row, column: header });
                      updateSelectedValue(event.target.value);
                    }}
                  />
                </label>
              ))}
            </div>
            <div style={styles.actions}>
              <button onClick={() => void saveTable()}>Save table</button>
              <button onClick={() => setEditing(false)}>Done</button>
            </div>
          </div>
        </div>
      )}
      {error && <div style={styles.error}>{error}</div>}
    </div>
  );
};

const styles: Record<string, React.CSSProperties> = {
  root: { height: '100vh', display: 'flex', flexDirection: 'column', background: 'var(--app-background)', color: 'var(--primary-text)', fontFamily: 'sans-serif' },
  header: { minHeight: 64, display: 'flex', alignItems: 'center', justifyContent: 'space-between', padding: '0.75rem 1rem', borderBottom: '1px solid var(--surface-border)', background: 'var(--surface-background)' },
  headerActions: { display: 'flex', alignItems: 'center', gap: 8 },
  path: { display: 'block', color: 'var(--muted-text)', fontSize: '0.8rem', marginTop: 4 },
  saveExit: { background: 'var(--danger-action)', color: 'var(--white-text)', border: 0, borderRadius: 6, padding: '0.7rem 1rem', fontWeight: 700, cursor: 'pointer' },
  body: { flex: 1, minHeight: 0, display: 'flex' },
  sidebar: { width: 230, overflowY: 'auto', padding: '1rem', borderRight: '1px solid var(--surface-border)', background: 'var(--surface-background)' },
  tableButton: { display: 'grid', width: '100%', gap: 4, padding: '0.65rem', marginBottom: 5, textAlign: 'left', background: 'transparent', color: 'var(--primary-text)', border: '1px solid transparent', cursor: 'pointer' },
  selectedTable: { display: 'grid', width: '100%', gap: 4, padding: '0.65rem', marginBottom: 5, textAlign: 'left', background: 'var(--control-background)', color: 'var(--primary-text)', border: '1px solid var(--primary-accent)', cursor: 'pointer' },
  workspace: { flex: 1, minWidth: 0, minHeight: 0, display: 'grid', gridTemplateColumns: 'minmax(260px, 0.35fr) minmax(520px, 1fr)' },
  editorPane: { overflowY: 'auto', padding: '1rem', borderRight: '1px solid var(--surface-border)' },
  tablePane: { minWidth: 0, minHeight: 0, display: 'flex', flexDirection: 'column', padding: '1rem' },
  tableHeader: { display: 'flex', justifyContent: 'space-between', marginBottom: '0.75rem' },
  tableScroll: { overflow: 'auto', flex: 1, border: '1px solid var(--surface-border)' },
  label: { display: 'grid', gap: 5, margin: '0.75rem 0', fontWeight: 700 },
  input: { minHeight: 34, width: '100%' },
  textarea: { minHeight: 150, width: '100%', boxSizing: 'border-box', background: 'var(--light-surface)', color: 'var(--dark-text)' },
  muted: { color: 'var(--muted-text)' },
  actions: { display: 'flex', gap: 8, margin: '1rem 0' },
  danger: { color: 'var(--danger-text)' },
  columnList: { display: 'grid', gap: 4, maxHeight: 180, overflowY: 'auto' },
  referenceList: { display: 'flex', flexWrap: 'wrap', gap: 4, maxHeight: 130, overflowY: 'auto' },
  referenceButton: { padding: '0.25rem 0.4rem', cursor: 'pointer' },
  cellButton: { width: '100%', minWidth: 100, maxWidth: 360, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap', textAlign: 'left', border: 0, background: 'transparent', color: 'inherit', cursor: 'pointer', padding: '0.45rem' },
  overlay: { position: 'fixed', inset: 0, zIndex: 20, display: 'grid', placeItems: 'center', padding: '2rem', background: 'var(--modal-overlay)' },
  dialog: { width: 'min(1000px, 96vw)', height: 'min(760px, 92vh)', display: 'flex', flexDirection: 'column', padding: '1.25rem', background: 'var(--surface-background)', border: '1px solid var(--surface-border)', borderRadius: 10 },
  dialogHeader: { display: 'flex', justifyContent: 'space-between', alignItems: 'center' },
  rowEditor: { flex: 1, display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(260px, 1fr))', gap: '0.5rem', overflowY: 'auto' },
  rowInput: { minHeight: 70, width: '100%', boxSizing: 'border-box', resize: 'vertical', background: 'var(--light-surface)', color: 'var(--dark-text)', padding: '0.5rem' },
  error: { position: 'fixed', left: 250, right: 20, bottom: 20, padding: '0.75rem', color: 'var(--error-light-text)', background: 'var(--error-background)', border: '1px solid var(--error-border)' },
};
