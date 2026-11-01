export function datasetAssetPath(path: string): string {
  const trimmed = path.trim().replaceAll('\\', '/');
  if (!trimmed) return './';
  return trimmed.startsWith('./') ? trimmed : `./${trimmed.replace(/^\/+/, '')}`;
}
