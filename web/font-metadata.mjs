// Bounded SFNT metadata reader. WOFF/collections need conversion before Rust engines can use them.
export const MAX_FONT_BYTES = 24 * 1024 * 1024;
export function inspectFont(buffer) {
  if (!(buffer instanceof ArrayBuffer) || buffer.byteLength < 12 || buffer.byteLength > MAX_FONT_BYTES) throw Error('Font must be a TTF or OTF file smaller than 24 MiB.');
  const v = new DataView(buffer), u16 = o => v.getUint16(o), u32 = o => v.getUint32(o);
  const tag = o => String.fromCharCode(...new Uint8Array(buffer, o, 4));
  if (u32(0) !== 0x00010000 && tag(0) !== 'OTTO') throw Error('Use an uncompressed TTF or OTF font. WOFF, WOFF2 and collections are not supported.');
  const count = u16(4), tables = new Map();
  if (!count || count > 256 || 12 + count * 16 > buffer.byteLength) throw Error('Invalid font table directory.');
  for (let i = 0; i < count; i++) {
    const p = 12 + i * 16, start = u32(p + 8), size = u32(p + 12);
    if (start + size > buffer.byteLength) throw Error('Invalid font table bounds.');
    tables.set(tag(p), {start, size});
  }
  for (const t of ['name','head','maxp','cmap']) if (!tables.has(t)) throw Error(`Missing ${t} font table.`);
  const nt = tables.get('name');
  if (nt.size < 6) throw Error('Invalid font names.');
  const n = u16(nt.start + 2), offset = u16(nt.start + 4), names = new Map();
  if (n > 4096 || 6 + n * 12 > nt.size) throw Error('Invalid font names.');
  for (let i = 0; i < n; i++) {
    const p = nt.start + 6 + i * 12, platform = u16(p), language = u16(p + 4), id = u16(p + 6), len = u16(p + 8), start = offset + u16(p + 10);
    if (start + len > nt.size || len > 4096) continue;
    if (![0,3].includes(platform)) continue;
    const val = new TextDecoder('utf-16be').decode(new Uint8Array(buffer, nt.start + start, len)).replace(/[\x00-\x1f\x7f]/g, '').trim();
    if (val && (!names.has(id) || language === 0x409)) names.set(id, val);
  }
  const family = names.get(16) || names.get(1), style = names.get(17) || names.get(2) || 'Regular';
  if (!family || family.length > 160) throw Error('Font has no usable family name.');
  const os = tables.get('OS/2'), fsType = os?.size >= 10 ? u16(os.start + 8) : 0;
  const weight = os?.size >= 8 ? u16(os.start + 4) : 400;
  const head = tables.get('head');
  if (head.size < 54) throw Error('Invalid font header.');
  const italic = !!(u16(head.start + 44) & 2);
  const axes = [], fv = tables.get('fvar');
  if (fv && fv.size >= 16) {
    const offset = u16(fv.start + 4), count = u16(fv.start + 8), size = u16(fv.start + 10);
    if (count > 64 || size < 20 || offset + count * size > fv.size) throw Error('Invalid variable font axes.');
    for (let i = 0; i < count; i++) {
      const p = fv.start + offset + i * size;
      axes.push({tag:tag(p), min:v.getInt32(p+4)/65536, default:v.getInt32(p+8)/65536, max:v.getInt32(p+12)/65536});
    }
  }
  return {copyright:names.get(0)||'',version:names.get(5)||'',designer:names.get(9)||'',family, style, weight, italic, axes, fsType, postscriptName:names.get(6)||'', license:names.get(13)||'', licenseUrl:names.get(14)||'', embedding:fsType & 2 ? 'restricted' : fsType & 0x200 ? 'bitmap-only' : fsType & 4 ? 'preview-print' : 'editable', noSubsetting:!!(fsType & 0x100)};
}
export async function digest(bytes) {
  return [...new Uint8Array(await crypto.subtle.digest('SHA-256', bytes))].map(v=>v.toString(16).padStart(2,'0')).join('');
}
