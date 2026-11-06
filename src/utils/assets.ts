export function datasetAssetPath(path: string): string {
  const trimmed = path.trim().replaceAll('\\', '/');
  if (!trimmed) return './';
  return trimmed.startsWith('./') ? trimmed : `./${trimmed.replace(/^\/+/, '')}`;
}

export function datasetImagePath(id: string, configuredPath?: string): string {
  const configured = configuredPath?.trim();
  return configured || `./img/${id}.jpeg`;
}

export function datasetThumbnailPath(path: string): string {
  const normalized = path.trim().replaceAll('\\', '/');
  if (!normalized || normalized.startsWith('data:')) return datasetAssetPath(path);

  const separator = normalized.lastIndexOf('/');
  const directory = separator >= 0 ? normalized.slice(0, separator + 1) : '';
  const filename = separator >= 0 ? normalized.slice(separator + 1) : normalized;
  const match = filename.match(/^(.+)\.(jpe?g|png|webp)$/i);
  if (!match || filename.toLowerCase().startsWith('thumb_')) {
    return datasetAssetPath(path);
  }

  return datasetAssetPath(`${directory}thumb_${match[1]}.jpeg`);
}
