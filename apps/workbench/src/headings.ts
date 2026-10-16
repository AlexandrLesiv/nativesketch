export function headingSlug(text: string, counts: Map<string, number>): string {
  const slug =
    text
      .toLocaleLowerCase()
      .replace(/[^\p{L}\p{N}_\- ]/gu, '')
      .trim()
      .replace(/\s+/g, '-') || 'section';
  const count = counts.get(slug) ?? 0;
  counts.set(slug, count + 1);
  return count ? `${slug}-${count}` : slug;
}
