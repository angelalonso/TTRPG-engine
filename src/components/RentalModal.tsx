import React, { useState } from 'react';

export interface RentalCarOption {
  id: string;
  name: string;
  price: number;
  available: boolean;
  reason?: string;
}

interface RentalModalProps {
  eventName: string;
  currency: string;
  cars: RentalCarOption[];
  onRent: (carId: string) => Promise<void>;
  onClose: () => void;
}

export const RentalModal: React.FC<RentalModalProps> = ({
  eventName,
  currency,
  cars,
  onRent,
  onClose,
}) => {
  const [selectedCarId, setSelectedCarId] = useState(
    cars.find((car) => car.available)?.id || cars[0]?.id || '',
  );
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState('');
  const selectedCar = cars.find((car) => car.id === selectedCarId);

  const submit = async () => {
    if (!selectedCar?.available || submitting) return;
    setSubmitting(true);
    setError('');
    try {
      await onRent(selectedCar.id);
    } catch (reason) {
      setError(String(reason));
      setSubmitting(false);
    }
  };

  return (
    <div style={styles.overlay} onClick={onClose}>
      <div style={styles.modal} onClick={(event) => event.stopPropagation()}>
        <header style={styles.header}>
          <h2 style={styles.title}>Rent a car for {eventName}</h2>
          <button style={styles.close} onClick={onClose} aria-label="Close">×</button>
        </header>
        <div style={styles.content}>
          <p>Select a car from the dataset. Rental costs are one twenty-fifth of the listed purchase price.</p>
          {cars.length === 0 ? (
            <p style={styles.error}>No vehicle entries are configured in this dataset.</p>
          ) : (
            <label style={styles.field}>
              Car
              <select
                style={styles.select}
                value={selectedCarId}
                onChange={(event) => setSelectedCarId(event.target.value)}
                disabled={submitting}
              >
                {cars.map((car) => (
                  <option key={car.id} value={car.id} disabled={!car.available}>
                    {car.name} — {currency}{(car.price / 25).toLocaleString()} rental
                    {!car.available ? ` (${car.reason || 'not eligible'})` : ''}
                  </option>
                ))}
              </select>
            </label>
          )}
          {selectedCar && (
            <p style={selectedCar.available ? styles.muted : styles.error}>
              {selectedCar.available
                ? `${selectedCar.name} rental: ${currency}${(selectedCar.price / 25).toLocaleString()}`
                : selectedCar.reason || 'This car is not currently eligible.'}
            </p>
          )}
          {error && <p role="alert" style={styles.error}>{error}</p>}
        </div>
        <footer style={styles.footer}>
          <button type="button" style={styles.pillButton} onClick={onClose} disabled={submitting}>Cancel</button>
          <button
            type="button"
            style={styles.primaryButton}
            onClick={() => void submit()}
            disabled={!selectedCar?.available || submitting}
          >
            {submitting ? 'Renting…' : 'Rent car'}
          </button>
        </footer>
      </div>
    </div>
  );
};

const styles: Record<string, React.CSSProperties> = {
  overlay: {
    position: 'fixed', inset: 0, zIndex: 3000, display: 'flex',
    alignItems: 'center', justifyContent: 'center', padding: '1rem',
    background: 'var(--modal-overlay)', backdropFilter: 'blur(4px)',
  },
  modal: {
    width: 'min(620px, 94vw)', background: 'var(--surface-background)',
    border: '1px solid var(--control-border)', borderRadius: '10px',
    boxShadow: '0 24px 50px var(--modal-overlay-dark)',
  },
  header: {
    display: 'flex', alignItems: 'center', justifyContent: 'space-between',
    padding: '1rem 1.25rem', borderBottom: '1px solid var(--surface-border)',
  },
  title: { margin: 0, fontSize: '1.2rem' },
  close: {
    border: 0, background: 'transparent', color: 'var(--subtle-text)',
    fontSize: '1.8rem', lineHeight: 1, cursor: 'pointer',
  },
  content: { display: 'grid', gap: '0.75rem', padding: '1.25rem' },
  field: { display: 'grid', gap: '0.4rem', fontWeight: 700 },
  select: {
    width: '100%', padding: '0.65rem 0.75rem', border: '1px solid var(--control-border)',
    borderRadius: '6px', background: 'var(--control-background)', color: 'var(--primary-text)',
  },
  muted: { margin: 0, color: 'var(--muted-text)' },
  error: { margin: 0, color: 'var(--danger-text)' },
  footer: {
    display: 'flex', justifyContent: 'flex-end', gap: '0.75rem',
    padding: '1rem 1.25rem', borderTop: '1px solid var(--surface-border)',
  },
  pillButton: {
    border: '1px solid var(--control-border)', borderRadius: '999px',
    background: 'var(--surface-background)', color: 'var(--secondary-text)',
    padding: '0.55rem 1rem', cursor: 'pointer',
  },
  primaryButton: {
    border: '1px solid var(--primary-accent-border)', borderRadius: '999px',
    background: 'var(--primary-accent)', color: 'var(--white-text)',
    padding: '0.55rem 1rem', cursor: 'pointer', fontWeight: 700,
  },
};

export default RentalModal;
