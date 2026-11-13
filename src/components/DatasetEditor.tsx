import { useEffect, useMemo, useState } from 'react';
import {
  getDatasetEditorCatalog,
  loadDatasetAsset,
  saveDatasetCar,
  saveDatasetChampionship,
  saveDatasetRace,
  type DatasetCarRequest,
  type DatasetChampionshipRequest,
  type DatasetEditorCar,
  type DatasetEditorCatalog,
  type DatasetRaceRequest,
} from '../services/tauriApi';

type FormState = {
  name: string;
  price: string;
  weight_kg: string;
  max_power_hp: string;
  trailer_id: string;
};

type CostValues = Record<string, string>;
type RaceForm = {
  name: string;
  day_of_year: string;
  entry_fee: string;
  reward_pool: string;
  tags: string;
  required_license_id: string;
  required_object_ids: string;
  quest_id: string;
  position_rewards: RewardRow[];
  description: string;
};
type ChampionshipForm = {
  name: string;
  join_fee: string;
  required_license_id: string;
  championship_rewards: RewardRow[];
  level: string;
  description: string;
};
type RewardRow = {
  position: string;
  amount: string;
};

const COST_RATES = [
  ['tires', 'Tires', 0.025],
  ['oil', 'Oil', 0.0075],
  ['engine', 'Engine rebuild', 0.12],
  ['gearbox', 'Gearbox rebuild', 0.08],
  ['brakes', 'Brakes', 0.03],
  ['cosmetic', 'Cosmetic repair', 0.01],
  ['bolt_on', 'Bolt-on repair', 0.06],
  ['factory', 'Factory rebuild', 0.2],
] as const;

const emptyForm: FormState = {
  name: '',
  price: '',
  weight_kg: '',
  max_power_hp: '',
  trailer_id: '',
};
const emptyRaceForm: RaceForm = {
  name: '', day_of_year: '', entry_fee: '', reward_pool: '', tags: 'race',
  required_license_id: '', required_object_ids: '', quest_id: '', position_rewards: [], description: '',
};
const emptyChampionshipForm: ChampionshipForm = {
  name: '', join_fee: '', required_license_id: '', championship_rewards: [],
  level: '1', description: '',
};
const PLACEHOLDER_PREVIEW = 'data:image/svg+xml,%3Csvg xmlns="http://www.w3.org/2000/svg" width="800" height="600" viewBox="0 0 800 600"%3E%3Crect width="800" height="600" fill="%23263238"/%3E%3Ctext x="400" y="310" text-anchor="middle" fill="%23ECEFF1" font-family="sans-serif" font-size="32"%3EPLACEHOLDER IMAGE%3C/text%3E%3C/svg%3E';

function parseRewards(value: string): RewardRow[] {
  return value.split(';').flatMap((entry) => {
    const [position, amount] = entry.split(':', 2);
    return position && amount ? [{ position, amount }] : [];
  });
}

function serializeRewards(rewards: RewardRow[]): string {
  return rewards
    .filter((reward) => reward.position.trim() || reward.amount.trim())
    .map((reward) => `${reward.position.trim()}:${reward.amount.trim()}`)
    .join(';');
}

function rewardsAreValid(rewards: RewardRow[]): boolean {
  const positions = rewards.map((reward) => Number(reward.position));
  return rewards.every((reward) =>
    Number.isInteger(Number(reward.position)) &&
    Number(reward.position) > 0 &&
    Number.isInteger(Number(reward.amount)) &&
    Number(reward.amount) >= 0,
  ) && new Set(positions).size === positions.length;
}

function costsFor(price: string): CostValues {
  const parsed = Number(price);
  return Object.fromEntries(
    COST_RATES.map(([key, _label, rate]) => [key, Number.isFinite(parsed) && parsed >= 0 ? String(Math.ceil(parsed * rate)) : '']),
  );
}

function imagePair(file: File | undefined): Promise<{ image_base64: string; thumbnail_base64: string }> {
  return new Promise((resolve, reject) => {
    const canvas = document.createElement('canvas');
    canvas.width = 800;
    canvas.height = 600;
    const context = canvas.getContext('2d');
    if (!context) {
      reject(new Error('Image processing is unavailable'));
      return;
    }
    const draw = (image: HTMLImageElement) => {
      const scale = Math.max(800 / image.naturalWidth, 600 / image.naturalHeight);
      const width = image.naturalWidth * scale;
      const height = image.naturalHeight * scale;
      context.fillStyle = '#263238';
      context.fillRect(0, 0, 800, 600);
      context.drawImage(image, (800 - width) / 2, (600 - height) / 2, width, height);
      const imageBase64 = canvas.toDataURL('image/jpeg', 0.92).split(',', 2)[1];
      const thumbnail = document.createElement('canvas');
      thumbnail.width = 400;
      thumbnail.height = 300;
      const thumbContext = thumbnail.getContext('2d');
      if (!thumbContext) {
        reject(new Error('Thumbnail processing is unavailable'));
        return;
      }
      thumbContext.drawImage(canvas, 0, 0, 400, 300);
      resolve({
        image_base64: imageBase64,
        thumbnail_base64: thumbnail.toDataURL('image/jpeg', 0.85).split(',', 2)[1],
      });
    };
    if (!file) {
      context.fillStyle = '#263238';
      context.fillRect(0, 0, 800, 600);
      context.fillStyle = '#ECEFF1';
      context.font = '32px sans-serif';
      context.fillText('CAR IMAGE', 290, 310);
      const imageBase64 = canvas.toDataURL('image/jpeg', 0.92).split(',', 2)[1];
      const thumbnail = document.createElement('canvas');
      thumbnail.width = 400;
      thumbnail.height = 300;
      thumbnail.getContext('2d')?.drawImage(canvas, 0, 0, 400, 300);
      resolve({
        image_base64: imageBase64,
        thumbnail_base64: thumbnail.toDataURL('image/jpeg', 0.85).split(',', 2)[1],
      });
      return;
    }
    const image = new Image();
    image.onload = () => draw(image);
    image.onerror = () => reject(new Error('The selected image could not be read'));
    image.src = URL.createObjectURL(file);
  });
}

export const DatasetEditor = ({ datasetPath }: { datasetPath: string }) => {
  const [catalog, setCatalog] = useState<DatasetEditorCatalog>({
    cars: [], trailers: [], races: [], championships: [],
  });
  const [selectedId, setSelectedId] = useState('');
  const [form, setForm] = useState<FormState>(emptyForm);
  const [imageFile, setImageFile] = useState<File>();
  const [imagePreview, setImagePreview] = useState('');
  const [costValues, setCostValues] = useState<CostValues>(costsFor(''));
  const [selectedRaceId, setSelectedRaceId] = useState('');
  const [selectedChampionshipId, setSelectedChampionshipId] = useState('');
  const [raceForm, setRaceForm] = useState<RaceForm>(emptyRaceForm);
  const [championshipForm, setChampionshipForm] = useState<ChampionshipForm>(emptyChampionshipForm);
  const [racingDialog, setRacingDialog] = useState<'race' | 'championship' | null>(null);
  const [racingImageFile, setRacingImageFile] = useState<File>();
  const [racingImagePreview, setRacingImagePreview] = useState('');
  const [organizerImageFile, setOrganizerImageFile] = useState<File>();
  const [organizerImagePreview, setOrganizerImagePreview] = useState('');
  const [dialogOpen, setDialogOpen] = useState(false);
  const [error, setError] = useState('');
  const [saving, setSaving] = useState(false);

  const selectedCar = useMemo(
    () => catalog.cars.find((car) => car.id === selectedId),
    [catalog.cars, selectedId],
  );
  const selectedRace = useMemo(
    () => catalog.races.find((race) => race.id === selectedRaceId),
    [catalog.races, selectedRaceId],
  );
  const selectedChampionship = useMemo(
    () => catalog.championships.find((championship) => championship.id === selectedChampionshipId),
    [catalog.championships, selectedChampionshipId],
  );
  const refresh = async () => {
    try {
      setCatalog(await getDatasetEditorCatalog(datasetPath));
      setError('');
    } catch (reason) {
      setError(String(reason));
    }
  };

  useEffect(() => {
    void refresh();
  }, [datasetPath]);

  useEffect(() => {
    let objectUrl: string | undefined;
    let cancelled = false;
    const loadPreview = async () => {
      if (imageFile) {
        objectUrl = URL.createObjectURL(imageFile);
        setImagePreview(objectUrl);
      } else if (dialogOpen && selectedCar?.image_path) {
        try {
          setImagePreview(await loadDatasetAsset(selectedCar.image_path));
        } catch {
          if (!cancelled) setImagePreview('');
        }
      } else if (dialogOpen) {
        setImagePreview(PLACEHOLDER_PREVIEW);
      } else {
        setImagePreview('');
      }
    };
    void loadPreview();
    return () => {
      cancelled = true;
      if (objectUrl) URL.revokeObjectURL(objectUrl);
    };
  }, [dialogOpen, imageFile, selectedCar]);

  useEffect(() => {
    let objectUrl: string | undefined;
    let cancelled = false;
    const selectedImagePath = racingDialog === 'race'
      ? selectedRace?.image_path
      : selectedChampionship?.image_path;
    const loadPreview = async () => {
      if (racingImageFile) {
        objectUrl = URL.createObjectURL(racingImageFile);
        setRacingImagePreview(objectUrl);
      } else if (racingDialog && selectedImagePath) {
        try {
          setRacingImagePreview(await loadDatasetAsset(selectedImagePath));
        } catch {
          if (!cancelled) setRacingImagePreview('');
        }
      } else if (racingDialog) {
        setRacingImagePreview(PLACEHOLDER_PREVIEW);
      } else {
        setRacingImagePreview('');
      }
    };
    void loadPreview();
    return () => {
      cancelled = true;
      if (objectUrl) URL.revokeObjectURL(objectUrl);
    };
  }, [racingDialog, racingImageFile, selectedRace, selectedChampionship]);

  useEffect(() => {
    let objectUrl: string | undefined;
    let cancelled = false;
    const selectedImagePath = racingDialog === 'race'
      ? selectedRace?.organizer_image_path
      : selectedChampionship?.organizer_image_path;
    const loadPreview = async () => {
      if (organizerImageFile) {
        objectUrl = URL.createObjectURL(organizerImageFile);
        setOrganizerImagePreview(objectUrl);
      } else if (racingDialog && selectedImagePath) {
        try {
          setOrganizerImagePreview(await loadDatasetAsset(selectedImagePath));
        } catch {
          if (!cancelled) setOrganizerImagePreview('');
        }
      } else if (racingDialog) {
        setOrganizerImagePreview(PLACEHOLDER_PREVIEW);
      } else {
        setOrganizerImagePreview('');
      }
    };
    void loadPreview();
    return () => {
      cancelled = true;
      if (objectUrl) URL.revokeObjectURL(objectUrl);
    };
  }, [racingDialog, organizerImageFile, selectedRace, selectedChampionship]);

  const openCreate = () => {
    setSelectedId('');
    setForm(emptyForm);
    setCostValues(costsFor(''));
    setImageFile(undefined);
    setImagePreview('');
    setDialogOpen(true);
    setError('');
  };

  const openEdit = () => {
    if (!selectedCar) {
      setError('Select a car to edit.');
      return;
    }
    setForm({
      name: selectedCar.name,
      price: String(selectedCar.price),
      weight_kg: String(selectedCar.weight_kg),
      max_power_hp: String(selectedCar.max_power_hp),
      trailer_id: selectedCar.trailer_id,
    });
    setCostValues(
      Object.fromEntries(
        COST_RATES.map(([key, _label, rate]) => [
          key,
          String(selectedCar.maintenance_costs[key] ?? Math.ceil(selectedCar.price * rate)),
        ]),
      ),
    );
    setImageFile(undefined);
    setDialogOpen(true);
    setError('');
  };

  const update = (key: keyof FormState, value: string) => {
    setForm((current) => ({ ...current, [key]: value }));
  };

  const updatePrice = (value: string) => {
    update('price', value);
    setCostValues(costsFor(value));
  };

  const openCreateRace = () => {
    setSelectedRaceId('');
    setRaceForm(emptyRaceForm);
    setRacingImageFile(undefined);
    setRacingImagePreview('');
    setOrganizerImageFile(undefined);
    setOrganizerImagePreview('');
    setRacingDialog('race');
    setError('');
  };
  const openEditRace = () => {
    if (!selectedRace) { setError('Select a race to edit.'); return; }
    setRaceForm({
      name: selectedRace.name,
      day_of_year: String(selectedRace.day_of_year),
      entry_fee: String(selectedRace.entry_fee),
      reward_pool: String(selectedRace.reward_pool),
      tags: selectedRace.tags,
      required_license_id: selectedRace.required_license_id,
      required_object_ids: selectedRace.required_object_ids,
      quest_id: selectedRace.quest_id,
      position_rewards: parseRewards(selectedRace.position_rewards),
      description: selectedRace.description_html,
    });
    setRacingImageFile(undefined);
    setOrganizerImageFile(undefined);
    setRacingDialog('race');
    setError('');
  };
  const openCreateChampionship = () => {
    setSelectedChampionshipId('');
    setChampionshipForm(emptyChampionshipForm);
    setRacingImageFile(undefined);
    setRacingImagePreview('');
    setOrganizerImageFile(undefined);
    setOrganizerImagePreview('');
    setRacingDialog('championship');
    setError('');
  };
  const openEditChampionship = () => {
    if (!selectedChampionship) { setError('Select a championship to edit.'); return; }
    setChampionshipForm({
      name: selectedChampionship.name,
      join_fee: String(selectedChampionship.join_fee),
      required_license_id: selectedChampionship.required_license_id,
      championship_rewards: parseRewards(selectedChampionship.championship_rewards),
      level: String(selectedChampionship.level),
      description: selectedChampionship.description_html,
    });
    setRacingImageFile(undefined);
    setOrganizerImageFile(undefined);
    setRacingDialog('championship');
    setError('');
  };

  const saveRaceOrChampionship = async () => {
    setSaving(true);
    try {
      const image = racingImageFile ? (await imagePair(racingImageFile)).image_base64 : undefined;
      const organizerImage = organizerImageFile
        ? (await imagePair(organizerImageFile)).image_base64
        : undefined;
      if (racingDialog === 'race') {
        if (!rewardsAreValid(raceForm.position_rewards)) {
          setError('Enter each race reward as a unique positive position and non-negative whole amount.');
          return;
        }
        const positionRewards = serializeRewards(raceForm.position_rewards);
        const request: DatasetRaceRequest = {
          dataset_path: datasetPath, edit_id: selectedRaceId || undefined,
          name: raceForm.name, day_of_year: Number(raceForm.day_of_year),
          entry_fee: Number(raceForm.entry_fee), reward_pool: Number(raceForm.reward_pool),
          tags: raceForm.tags, required_license_id: raceForm.required_license_id,
          required_object_ids: raceForm.required_object_ids, quest_id: raceForm.quest_id,
          position_rewards: positionRewards,
          organizer_image_base64: organizerImage,
          description: raceForm.description, image_base64: image,
        };
        if (!request.name.trim() || !Number.isInteger(request.day_of_year) || request.day_of_year < 0 ||
            !Number.isInteger(request.entry_fee) || request.entry_fee < 0 ||
            !Number.isInteger(request.reward_pool) || request.reward_pool < 0) {
          setError('Enter a race name, non-negative day, entry fee, and reward pool.');
          return;
        }
        await saveDatasetRace(request);
      } else if (racingDialog === 'championship') {
        if (!rewardsAreValid(championshipForm.championship_rewards)) {
          setError('Enter each championship reward as a unique positive position and non-negative whole amount.');
          return;
        }
        const championshipRewards = serializeRewards(championshipForm.championship_rewards);
        const request: DatasetChampionshipRequest = {
          dataset_path: datasetPath, edit_id: selectedChampionshipId || undefined,
          name: championshipForm.name, join_fee: Number(championshipForm.join_fee),
          required_license_id: championshipForm.required_license_id,
          championship_rewards: championshipRewards,
          level: Number(championshipForm.level), description: championshipForm.description,
          organizer_image_base64: organizerImage,
          image_base64: image,
        };
        if (!request.name.trim() || !Number.isInteger(request.join_fee) || request.join_fee < 0 ||
            !Number.isInteger(request.level) || request.level < 0) {
          setError('Enter a championship name, non-negative join fee, and level.');
          return;
        }
        await saveDatasetChampionship(request);
      }
      await refresh();
      setRacingDialog(null);
      setError('');
    } catch (reason) {
      setError(String(reason));
    } finally {
      setSaving(false);
    }
  };

  const save = async () => {
    const price = Number(form.price);
    const weight = Number(form.weight_kg);
    const power = Number(form.max_power_hp);
    if (!form.name.trim() || !Number.isInteger(price) || price < 0 ||
        !Number.isInteger(weight) || weight <= 0 ||
        !Number.isInteger(power) || power <= 0 || !form.trailer_id) {
      setError('Enter a name, non-negative whole price, positive whole weight and power, and a trailer.');
      return;
    }
    const maintenanceCosts = Object.fromEntries(
      COST_RATES.map(([key, label]) => {
        const amount = Number(costValues[key]);
        return [key, amount];
      }),
    );
    if (Object.values(maintenanceCosts).some((amount) => !Number.isInteger(amount) || amount < 0)) {
      setError('Enter a non-negative whole number for every maintenance cost.');
      return;
    }
    const summary = COST_RATES
      .map(([key, label]) => `${label}: ${maintenanceCosts[key].toLocaleString()}`)
      .join('\n');
    if (!window.confirm(`Calculated maintenance costs:\n\n${summary}\n\nSave this car?`)) return;
    setSaving(true);
    try {
      const pair = imageFile || !selectedId ? await imagePair(imageFile) : undefined;
      const request: DatasetCarRequest = {
        dataset_path: datasetPath,
        edit_id: selectedId || undefined,
        name: form.name,
        price,
        weight_kg: weight,
        max_power_hp: power,
        trailer_id: form.trailer_id,
        maintenance_costs: maintenanceCosts,
        image_base64: pair?.image_base64,
        thumbnail_base64: pair?.thumbnail_base64,
      };
      const saved = await saveDatasetCar(request);
      await refresh();
      setSelectedId(saved.id);
      setDialogOpen(false);
      setError('');
    } catch (reason) {
      setError(String(reason));
    } finally {
      setSaving(false);
    }
  };

  const saveChanges = async () => {
    if (dialogOpen) {
      await save();
    } else if (racingDialog) {
      await saveRaceOrChampionship();
    } else {
      setError('Open a car, race, or championship editor before saving changes.');
    }
  };

  return (
    <div className="dataset-editor-screen" data-testid="dataset-editor">
      <header className="dataset-editor-header">
        <h1>Dataset Editor</h1>
        <button
          type="button"
          onClick={() => void saveChanges()}
          disabled={saving || (!dialogOpen && !racingDialog)}
        >
          {saving ? 'Saving…' : 'Save changes'}
        </button>
      </header>
      {error && <p role="alert" className="dataset-editor-error">{error}</p>}
      <div className="dataset-editor-sections">
        <section className="dataset-editor-section dataset-editor-car-section">
          <h2>Cars</h2>
          <div className="dataset-editor-car-controls">
            <label className="dataset-editor-selector">
              Existing cars
              <select value={selectedId} onChange={(event) => setSelectedId(event.target.value)}>
                <option value="">Select a car</option>
                {catalog.cars.map((car) => <option key={car.id} value={car.id}>{car.name}</option>)}
              </select>
            </label>
            <div className="dataset-editor-section-actions">
              <button type="button" onClick={openCreate}>Add car</button>
              <button type="button" onClick={openEdit} disabled={!selectedCar}>Edit car</button>
            </div>
          </div>
        </section>
        <section className="dataset-editor-section dataset-editor-future-section">
          <h2>Races</h2>
          <label className="dataset-editor-selector">
            Existing races
            <select value={selectedRaceId} onChange={(event) => setSelectedRaceId(event.target.value)}>
              <option value="">Select a race</option>
              {catalog.races.map((race) => <option key={race.id} value={race.id}>{race.name}</option>)}
            </select>
          </label>
          <div className="dataset-editor-section-actions">
            <button type="button" onClick={openCreateRace}>Add race</button>
            <button type="button" onClick={openEditRace} disabled={!selectedRace}>Edit race</button>
          </div>
        </section>
        <section className="dataset-editor-section dataset-editor-future-section">
          <h2>Championships</h2>
          <label className="dataset-editor-selector">
            Existing championships
            <select value={selectedChampionshipId} onChange={(event) => setSelectedChampionshipId(event.target.value)}>
              <option value="">Select a championship</option>
              {catalog.championships.map((championship) => <option key={championship.id} value={championship.id}>{championship.name}</option>)}
            </select>
          </label>
          <div className="dataset-editor-section-actions">
            <button type="button" onClick={openCreateChampionship}>Add championship</button>
            <button type="button" onClick={openEditChampionship} disabled={!selectedChampionship}>Edit championship</button>
          </div>
        </section>
      </div>
      {dialogOpen && (
        <div className="dataset-editor-dialog-backdrop" role="presentation">
          <section className="dataset-editor-dialog" role="dialog" aria-modal="true" aria-labelledby="car-dialog-title">
            <h2 id="car-dialog-title">{selectedId ? 'Edit car' : 'Add car'}</h2>
            <label>Name<input value={form.name} onChange={(event) => update('name', event.target.value)} /></label>
            <label>Price<input type="number" min="0" step="1" value={form.price} onChange={(event) => updatePrice(event.target.value)} /></label>
            <label>Weight (kg)<input type="number" min="1" step="1" value={form.weight_kg} onChange={(event) => update('weight_kg', event.target.value)} /></label>
            <label>Power (hp)<input type="number" min="1" step="1" value={form.max_power_hp} onChange={(event) => update('max_power_hp', event.target.value)} /></label>
            <label>Required trailer
              <select value={form.trailer_id} onChange={(event) => update('trailer_id', event.target.value)}>
                <option value="">Select a trailer</option>
                {catalog.trailers.map((trailer) => <option key={trailer.id} value={trailer.id}>{trailer.name}</option>)}
              </select>
            </label>
            <label>Picture (normalized to 800×600)<input type="file" accept="image/*" onChange={(event) => setImageFile(event.target.files?.[0])} /></label>
            {imagePreview && <img className="dataset-editor-image-preview" src={imagePreview} alt="Car preview" />}
            <p>Without a picture, a placeholder JPEG is used.</p>
            <h3>Calculated maintenance costs</h3>
            <ul className="dataset-editor-cost-list">
              {COST_RATES.map(([key, label]) => (
                <li key={key}>
                  <label>{label}
                    <input
                      type="number"
                      min="0"
                      step="1"
                      value={costValues[key]}
                      onChange={(event) => setCostValues((current) => ({ ...current, [key]: event.target.value }))}
                    />
                  </label>
                </li>
              ))}
            </ul>
            <div className="dataset-editor-dialog-actions">
              <button type="button" onClick={() => setDialogOpen(false)} disabled={saving}>Cancel</button>
              <button type="button" onClick={() => void save()} disabled={saving}>{saving ? 'Saving…' : 'Save car'}</button>
            </div>
          </section>
        </div>
      )}
      {racingDialog && (
        <div className="dataset-editor-dialog-backdrop" role="presentation">
          <section className="dataset-editor-dialog" role="dialog" aria-modal="true" aria-labelledby="racing-dialog-title">
            <h2 id="racing-dialog-title">
              {racingDialog === 'race'
                ? (selectedRaceId ? 'Edit race' : 'Add race')
                : (selectedChampionshipId ? 'Edit championship' : 'Add championship')}
            </h2>
            {racingDialog === 'race' ? (
              <>
                <label>Name<input value={raceForm.name} onChange={(event) => setRaceForm({ ...raceForm, name: event.target.value })} /></label>
                <label>Day of year<input type="number" min="0" step="1" value={raceForm.day_of_year} onChange={(event) => setRaceForm({ ...raceForm, day_of_year: event.target.value })} /></label>
                <label>Entry fee<input type="number" min="0" step="1" value={raceForm.entry_fee} onChange={(event) => setRaceForm({ ...raceForm, entry_fee: event.target.value })} /></label>
                <label>Reward pool<input type="number" min="0" step="1" value={raceForm.reward_pool} onChange={(event) => setRaceForm({ ...raceForm, reward_pool: event.target.value })} /></label>
                <label>Tags<input value={raceForm.tags} onChange={(event) => setRaceForm({ ...raceForm, tags: event.target.value })} /></label>
                <label>Required licence ID<input value={raceForm.required_license_id} onChange={(event) => setRaceForm({ ...raceForm, required_license_id: event.target.value })} /></label>
                <label>Required vehicle/equipment IDs<input value={raceForm.required_object_ids} onChange={(event) => setRaceForm({ ...raceForm, required_object_ids: event.target.value })} /></label>
                <label>Championship ID<input value={raceForm.quest_id} onChange={(event) => setRaceForm({ ...raceForm, quest_id: event.target.value })} /></label>
                <fieldset className="dataset-editor-rewards">
                  <legend>Position rewards</legend>
                  {raceForm.position_rewards.map((reward, index) => (
                    <div className="dataset-editor-reward-row" key={`race-reward-${index}`}>
                      <input
                        type="number"
                        min="1"
                        step="1"
                        aria-label={`Race reward position ${index + 1}`}
                        placeholder="Position"
                        value={reward.position}
                        onChange={(event) => setRaceForm((current) => ({
                          ...current,
                          position_rewards: current.position_rewards.map((item, itemIndex) =>
                            itemIndex === index ? { ...item, position: event.target.value } : item),
                        }))}
                      />
                      <input
                        type="number"
                        min="0"
                        step="1"
                        aria-label={`Race reward amount ${index + 1}`}
                        placeholder="Amount"
                        value={reward.amount}
                        onChange={(event) => setRaceForm((current) => ({
                          ...current,
                          position_rewards: current.position_rewards.map((item, itemIndex) =>
                            itemIndex === index ? { ...item, amount: event.target.value } : item),
                        }))}
                      />
                      <button type="button" onClick={() => setRaceForm((current) => ({
                        ...current,
                        position_rewards: current.position_rewards.filter((_item, itemIndex) => itemIndex !== index),
                      }))}>Remove</button>
                    </div>
                  ))}
                  <button type="button" onClick={() => setRaceForm((current) => ({
                    ...current,
                    position_rewards: [...current.position_rewards, { position: String(current.position_rewards.length + 1), amount: '' }],
                  }))}>Add position reward</button>
                </fieldset>
                <label>Description<textarea value={raceForm.description} onChange={(event) => setRaceForm({ ...raceForm, description: event.target.value })} /></label>
              </>
            ) : (
              <>
                <label>Name<input value={championshipForm.name} onChange={(event) => setChampionshipForm({ ...championshipForm, name: event.target.value })} /></label>
                <label>Join fee<input type="number" min="0" step="1" value={championshipForm.join_fee} onChange={(event) => setChampionshipForm({ ...championshipForm, join_fee: event.target.value })} /></label>
                <label>Required licence ID<input value={championshipForm.required_license_id} onChange={(event) => setChampionshipForm({ ...championshipForm, required_license_id: event.target.value })} /></label>
                <fieldset className="dataset-editor-rewards">
                  <legend>Rewards</legend>
                  {championshipForm.championship_rewards.map((reward, index) => (
                    <div className="dataset-editor-reward-row" key={`championship-reward-${index}`}>
                      <input
                        type="number"
                        min="1"
                        step="1"
                        aria-label={`Championship reward position ${index + 1}`}
                        placeholder="Position"
                        value={reward.position}
                        onChange={(event) => setChampionshipForm((current) => ({
                          ...current,
                          championship_rewards: current.championship_rewards.map((item, itemIndex) =>
                            itemIndex === index ? { ...item, position: event.target.value } : item),
                        }))}
                      />
                      <input
                        type="number"
                        min="0"
                        step="1"
                        aria-label={`Championship reward amount ${index + 1}`}
                        placeholder="Amount"
                        value={reward.amount}
                        onChange={(event) => setChampionshipForm((current) => ({
                          ...current,
                          championship_rewards: current.championship_rewards.map((item, itemIndex) =>
                            itemIndex === index ? { ...item, amount: event.target.value } : item),
                        }))}
                      />
                      <button type="button" onClick={() => setChampionshipForm((current) => ({
                        ...current,
                        championship_rewards: current.championship_rewards.filter((_item, itemIndex) => itemIndex !== index),
                      }))}>Remove</button>
                    </div>
                  ))}
                  <button type="button" onClick={() => setChampionshipForm((current) => ({
                    ...current,
                    championship_rewards: [...current.championship_rewards, { position: String(current.championship_rewards.length + 1), amount: '' }],
                  }))}>Add reward</button>
                </fieldset>
                <label>Level<input type="number" min="0" step="1" value={championshipForm.level} onChange={(event) => setChampionshipForm({ ...championshipForm, level: event.target.value })} /></label>
                <label>Description<textarea value={championshipForm.description} onChange={(event) => setChampionshipForm({ ...championshipForm, description: event.target.value })} /></label>
              </>
            )}
            <label>Picture<input type="file" accept="image/*" onChange={(event) => setRacingImageFile(event.target.files?.[0])} /></label>
            {racingImagePreview && (
              <img
                className="dataset-editor-image-preview"
                src={racingImagePreview}
                alt={racingDialog === 'race' ? 'Race preview' : 'Championship preview'}
              />
            )}
            <label>Organiser picture<input type="file" accept="image/*" onChange={(event) => setOrganizerImageFile(event.target.files?.[0])} /></label>
            {organizerImagePreview && (
              <img
                className="dataset-editor-image-preview"
                src={organizerImagePreview}
                alt="Organiser preview"
              />
            )}
            <div className="dataset-editor-dialog-actions">
              <button type="button" onClick={() => setRacingDialog(null)} disabled={saving}>Cancel</button>
              <button type="button" onClick={() => void saveRaceOrChampionship()} disabled={saving}>{saving ? 'Saving…' : 'Save'}</button>
            </div>
          </section>
        </div>
      )}
    </div>
  );
};
