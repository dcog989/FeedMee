export function thumbnailCacheKey(imageUrl: string | null, url: string, size: number): string {
  return `${imageUrl || url}_${size}`;
}
