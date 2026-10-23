import {discoveryURL} from './config.mjs';
import {MAX_FONT_BYTES, inspectFont, digest} from './font-metadata.mjs';
export const normalize = value => String(value || '').normalize('NFKD').replace(/\p{M}/gu,'').toLowerCase().replace(/[^\p{L}\p{N}]/gu,'');
export function openLicense(value) {
  return /\bOFL\b|SIL Open Font|Apache|\bMIT\b|\bCC-?0\b|Public Domain|CC-BY(?:-SA)?\b|Creative Commons Attribution|GNU General Public License|\bGPL\b|Bitstream Vera|GUST Font License|Ubuntu Font License/i.test(value) && !/non.?commercial|\bNC\b|no.?derivatives|\bND\b/i.test(value);
}
export function fontURL(value) {
  const url = new URL(value);
  if(url.protocol !== 'https:' || url.hostname !== 'fontlibrary.org' || url.port || url.username || url.password || !url.pathname.startsWith('/assets/fonts/') || !/\.(ttf|otf)$/i.test(url.pathname) || url.search || url.hash) throw Error('Invalid Font Library download address.');
  return url.href;
}
export function familyAvailable(family, installed, bundled) {
  const key=normalize(family.name);
  return bundled.some(f=>normalize(f.name)===key) || installed.some(r=>r.source?.family===family.id && r.source?.kind==='fontlibrary' || normalize(r.family)===key);
}
export function faceAvailable(face, installed) {return installed.find(r=>r.source?.url===face.url);}
const featured=['Cooper Hewitt','Fantasque Sans Mono','Avara','Jellee Typeface','Glacial Indifference','Xolonium','Futura Renner','Datalegreya'];
export function selectFamilies(families, {query='', category='', license='open', sort='featured', showAvailable=false, installed=[], bundled=[]}={}) {
  const names=new Set([...bundled.map(f=>normalize(f.name)),...installed.map(r=>normalize(r.family))].filter(Boolean));
  const importedIds=new Set(installed.filter(r=>r.source?.kind==='fontlibrary').map(r=>r.source.family));
  const known=new Map(families.map(f=>[f.id,names.has(normalize(f.name))||importedIds.has(f.id)]));
  const terms=query.trim().toLocaleLowerCase().split(/\s+/).filter(Boolean);
  return families.filter(f=>(!category || f.category===category) && (license!=='open' || openLicense(f.license)) && terms.every(t=>`${f.name} ${f.designer} ${f.category} ${f.languages} ${f.description}`.toLocaleLowerCase().includes(t)) && (showAvailable || !known.get(f.id))).sort((a,b)=>Number(known.get(a.id))-Number(known.get(b.id)) || (sort==='styles'?b.faces.length-a.faces.length:sort==='featured'&&!query.trim()?Number(featured.includes(b.name))-Number(featured.includes(a.name)):0) || a.name.localeCompare(b.name));
}
export function regularFace(family) {return family.faces.find(f=>/regular|book|normal/i.test(f.name)) || family.faces[0];}
let catalogPromise;
export function remoteCatalog() {
  return catalogPromise ||= fetch(discoveryURL).then(r=>{if(!r.ok)throw Error('The discovery catalog could not load. Check your connection and retry.');return r.json();}).catch(e=>{catalogPromise=null;throw e;});
}
// A bounded memory cache is shared by previews and imports. It never writes previews to IndexedDB.
const cache=new Map(), inflight=new Map();let retained=0;
export async function downloadFace(face, signal) {
  const url=fontURL(face.url);
  if(cache.has(url)){const data=cache.get(url);cache.delete(url);cache.set(url,data);return data;}
  if(inflight.has(url))return inflight.get(url);
  const promise=(async()=>{
    const timeout=AbortSignal.timeout(45000);
    const response=await fetch(url,{mode:'cors',redirect:'error',credentials:'omit',referrerPolicy:'no-referrer',signal:signal?AbortSignal.any([signal,timeout]):timeout});
    if(!response.ok)throw Error(`Font Library download failed (${response.status}). Try again.`);
    fontURL(response.url||url);
    if(Number(response.headers.get('content-length'))>MAX_FONT_BYTES)throw Error('This font exceeds the 24 MiB file limit.');
    const reader=response.body.getReader(),chunks=[];let size=0;
    try {while(true){const {done,value}=await reader.read();if(done)break;size+=value.byteLength;if(size>MAX_FONT_BYTES)throw Error('This font exceeds the 24 MiB file limit.');chunks.push(value);}}
    catch(e){await reader.cancel().catch(()=>{});throw e;}
    const bytes=new Uint8Array(size);let offset=0;for(const chunk of chunks){bytes.set(chunk,offset);offset+=chunk.byteLength;}
    const meta=inspectFont(bytes.buffer),id=await digest(bytes.buffer);
    const data={bytes:bytes.buffer,meta,id};
    while(retained+size>48*1024*1024 && cache.size){const [key,old]=cache.entries().next().value;cache.delete(key);retained-=old.bytes.byteLength;}
    cache.set(url,data);retained+=size;return data;
  })().finally(()=>inflight.delete(url));
  inflight.set(url,promise);return promise;
}
export function provenance(family, face, retrievedAt) {
  if(!openLicense(family.license))throw Error('This license needs review. Visit the source for licensing information.');
  return {kind:'fontlibrary',family:family.id,familyName:family.name,url:fontURL(face.url),sourceUrl:family.sourceUrl,downloadUrl:family.downloadUrl,license:family.license,licenseUrl:family.licenseUrl,designer:family.designer,catalogRetrievedAt:retrievedAt};
}
