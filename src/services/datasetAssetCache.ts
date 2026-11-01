import { loadDatasetAsset, loadDescription } from './tauriApi';

const assetCache = new Map<string, Promise<string>>();
const descriptionCache = new Map<string, Promise<string>>();

export function loadCachedDatasetAsset(path: string, cacheKey = path): Promise<string> {
  const key = cacheKey.trim();
  if (!key) return Promise.resolve('');
  const existing = assetCache.get(key);
  if (existing) return existing;
  const request = loadDatasetAsset(path.trim()).catch((error) => {
    assetCache.delete(key);
    throw error;
  });
  assetCache.set(key, request);
  return request;
}

export function loadCachedDescription(path: string): Promise<string> {
  const key = path.trim();
  if (!key) return Promise.resolve('');
  const existing = descriptionCache.get(key);
  if (existing) return existing;
  const request = loadDescription(key).catch((error) => {
    descriptionCache.delete(key);
    throw error;
  });
  descriptionCache.set(key, request);
  return request;
}
