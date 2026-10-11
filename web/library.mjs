import {databaseName, catalogURL, assetURL, autoActivate} from './config.mjs';
import {inspectFont, digest, MAX_FONT_BYTES} from './font-metadata.mjs';
const DB = databaseName, VERSION = 2, BACKUP_VERSION = 1, MAX_ACTIVE_BYTES = 256 * 1024 * 1024;
const events = new EventTarget();
let database, catalogPromise, active = [], booted = false;
const inflight = new Map();
const pending = new Map(), acknowledged = new Set(), browserFaces = new Map();
const channel = typeof BroadcastChannel === 'function' ? new BroadcastChannel(DB) : null;
function emit(detail) { events.dispatchEvent(new CustomEvent('change', {detail})); }
function request(r) { return new Promise((resolve,reject)=>{r.onsuccess=()=>resolve(r.result);r.onerror=()=>reject(r.error);}); }
async function db() {
  if (!database) database = new Promise((resolve,reject)=>{
    const r = indexedDB.open(DB, VERSION);
    r.onupgradeneeded=()=>{
      if(!r.result.objectStoreNames.contains('fonts'))r.result.createObjectStore('fonts',{keyPath:'id'});
      if(!r.result.objectStoreNames.contains('preferences'))r.result.createObjectStore('preferences',{keyPath:'id'});
    };
    r.onsuccess=()=>{r.result.onversionchange=()=>{r.result.close();database=null;};resolve(r.result);};
    r.onerror=()=>{database=null;reject(Error('Browser storage is unavailable. Allow site data and try again.'));};
    r.onblocked=()=>{database=null;reject(Error('Close other Craft app tabs to upgrade the font library.'));};
  });
  return database;
}
async function transaction(mode, operation, store='fonts') {
  const database = await db();
  return new Promise((resolve,reject)=>{
    const tx = database.transaction(store,mode); let value;
    tx.oncomplete=()=>resolve(value);tx.onabort=()=>reject(tx.error||Error('Font storage transaction failed.'));tx.onerror=()=>{};
    const r = operation(tx.objectStore(store));r.onsuccess=()=>{value=r.result;};
  });
}
export const list = () => transaction('readonly',s=>s.getAll());
export function catalog() {
  if (!catalogPromise) catalogPromise = fetch(catalogURL).then(r=>{if(!r.ok)throw Error('Font catalog could not be loaded.');return r.json();}).catch(e=>{catalogPromise=null;throw e;});
  return catalogPromise;
}
async function preview(record) {
  if (browserFaces.has(record.id)) return;
  const font = new FontFace('Craft-'+record.id, record.bytes, {weight:String(record.weight),style:record.italic?'italic':'normal'});
  await font.load();document.fonts.add(font);browserFaces.set(record.id,font);
}
async function refresh(external=false) {
  const records = await list();
  const installed = records.filter(r=>r.enabled);
  const removed = active.some(r=>!installed.some(n=>n.id===r.id));
  active = installed;
  for (const r of installed) {
    if (!acknowledged.has(r.id) && !inflight.has(r.id)) pending.set(r.id,r);
  }
  for (const map of [pending, inflight]) for (const id of map.keys()) if (!installed.some(r=>r.id===id)) map.delete(id);
  emit({reloadRequired:removed,external});
  window.dispatchEvent(new Event('craft:font-change'));
}
channel && (channel.onmessage=()=>refresh(true).catch(e=>emit({error:e.message})));
async function changed() { await refresh();channel?.postMessage('changed'); }
export async function install(bytes, source={kind:'upload'}) {
  const meta = inspectFont(bytes), id = await digest(bytes);
  if (meta.noSubsetting || ['restricted','bitmap-only','preview-print'].includes(meta.embedding)) throw Error('This font does not permit editable document embedding and subsetting needed across the apps. Choose a font with editable or installable embedding permission.');
  const records = await list(), old = records.find(r=>r.id===id);
  if (old) { if(!old.enabled) await enable(id,true);return old; }
  if (records.filter(r=>r.enabled).reduce((n,r)=>n+r.bytes.byteLength,0)+bytes.byteLength > MAX_ACTIVE_BYTES) throw Error('Active fonts exceed 256 MiB. Deactivate unused fonts before adding more.');
  // Browser decoding validates the outline before accepting persistent data.
  const record = {...meta,id,bytes,source,enabled:true,favorite:false,addedAt:new Date().toISOString()};
  await preview(record);
  await insertRecord(record);await changed();return record;
}

async function readFontResponse(response) {
  if (Number(response.headers.get('content-length')) > MAX_FONT_BYTES) throw Error('Font exceeds 24 MiB.');
  const reader=response.body.getReader(), chunks=[];let size=0;
  try {while(true){const {done,value}=await reader.read();if(done)break;size+=value.byteLength;if(size>MAX_FONT_BYTES)throw Error('Font exceeds 24 MiB.');chunks.push(value);}}
  catch(e){await reader.cancel().catch(()=>{});throw e;}
  const bytes=new Uint8Array(size);let offset=0;for(const chunk of chunks){bytes.set(chunk,offset);offset+=chunk.byteLength;}return bytes.buffer;
}
async function insertRecord(record, onlyIfMissing=false) {
  const database=await db();
  await new Promise((resolve,reject)=>{
    const tx=database.transaction(['fonts','preferences'],'readwrite');let failure;
    tx.oncomplete=resolve;tx.onabort=()=>reject(failure||tx.error||Error('Font transaction failed.'));
    const fonts=tx.objectStore('fonts'), prefs=tx.objectStore('preferences');
    const all=fonts.getAll();all.onsuccess=()=>{
      if(onlyIfMissing && all.result.some(r=>r.id===record.id))return;
      const pref=prefs.get(record.id);pref.onsuccess=()=>{
        if(onlyIfMissing && pref.result?.removed)return;
        const total=all.result.filter(r=>r.enabled&&r.id!==record.id).reduce((sum,r)=>sum+r.bytes.byteLength,0)+(record.enabled?record.bytes.byteLength:0);
        if(total>MAX_ACTIVE_BYTES){failure=Error('Active fonts exceed 256 MiB.');tx.abort();return;}
        fonts.put(record);prefs.delete(record.id);
      };
    };
  });
}

export async function installCatalog(face, family) {
  const url = assetURL(face.path);
  const r = await fetch(url, {signal:AbortSignal.timeout(60000)});
  if (!r.ok) throw Error('Font download failed. Try again when connected.');
  const bytes = await readFontResponse(r);
  if(bytes.byteLength > MAX_FONT_BYTES || await digest(bytes)!==face.sha256) throw Error('Font checksum mismatch. No font was installed.');
  return install(bytes,{kind:'catalog',family:family.id,license:family.license,licensePath:family.licensePath,url:face.url});
}
export async function enable(id, enabled) {
  const records=await list(), r=records.find(r=>r.id===id);if(!r)throw Error('Font no longer exists.');
  if(enabled && records.filter(x=>x.enabled&&x.id!==id).reduce((n,x)=>n+x.bytes.byteLength,0)+r.bytes.byteLength>MAX_ACTIVE_BYTES)throw Error('Active fonts exceed 256 MiB.');
  await insertRecord({...r,enabled});await changed();
}
export async function favorite(id) {const r=(await list()).find(r=>r.id===id);if(r){await transaction('readwrite',s=>s.put({...r,favorite:!r.favorite}));await changed();}}
export async function remove(id) {const database=await db();await new Promise((resolve,reject)=>{const tx=database.transaction(['fonts','preferences'],'readwrite');tx.oncomplete=resolve;tx.onabort=()=>reject(tx.error);tx.objectStore('fonts').delete(id);tx.objectStore('preferences').put({id,removed:true});});await changed();}
export async function storage() {return {estimate:await navigator.storage?.estimate?.(), persistent:await navigator.storage?.persisted?.()};}
export async function persist() {return navigator.storage?.persist?.() || false;}
export async function previewFont(record) {await preview(record);return 'Craft-'+record.id;}
export async function backup() {
  const records = await list();
  if(records.reduce((n,r)=>n+r.bytes.byteLength,0)>256*1024*1024)throw Error('Library exceeds the backup size limit. Remove unused fonts before backing up.');
  const fonts = records.map(r=>{let binary='';for(const b of new Uint8Array(r.bytes))binary+=String.fromCharCode(b);return {...r,bytes:btoa(binary)};});
  return new Blob([JSON.stringify({format:'craft-font-library',version:BACKUP_VERSION,fonts})],{type:'application/json'});
}

// Source fields are descriptive provenance only. Never trust a backup URL for an automatic download.
function restoredSource(source) {
  if(!source || !['fontlibrary','catalog','upload','restored'].includes(source.kind))return {kind:'restored'};
  const safe={kind:source.kind};
  for(const key of ['family','familyName','url','sourceUrl','downloadUrl','license','licenseUrl','licensePath','designer','catalogRetrievedAt']){
    if(typeof source[key]==='string')safe[key]=source[key].slice(0,4096);
  }
  return safe;
}
export async function restore(file) {
  if(file.size>360*1024*1024)throw Error('Backup exceeds 360 MiB.');
  const data=JSON.parse(await file.text());
  if(!['craft-font-library','starcraft-font-library'].includes(data.format)||data.version!==BACKUP_VERSION||!Array.isArray(data.fonts)||data.fonts.length>1000)throw Error('Unsupported font backup.');
  // Validate all files before one atomic transaction. Never trust backup metadata or URLs.
  const records=await list(), merged=new Map(records.map(r=>[r.id,r]));
  for(const r of data.fonts){
    if(typeof r.bytes!=='string'||r.bytes.length>MAX_FONT_BYTES*1.4)throw Error('Invalid backup font.');
    const bytes=Uint8Array.from(atob(r.bytes),c=>c.charCodeAt(0)).buffer,meta=inspectFont(bytes),id=await digest(bytes);
    if(id!==r.id||meta.noSubsetting||['restricted','bitmap-only','preview-print'].includes(meta.embedding))throw Error('Invalid or restricted font in backup.');
    const record={...meta,id,bytes,source:restoredSource(r.source),enabled:!!r.enabled,favorite:!!r.favorite,addedAt:new Date().toISOString()};
    await preview(record);if(!merged.has(id))merged.set(id,record);
  }
  if([...merged.values()].filter(r=>r.enabled).reduce((n,r)=>n+r.bytes.byteLength,0)>MAX_ACTIVE_BYTES)throw Error('Backup would exceed the 256 MiB active-font limit.');
  const database=await db();await new Promise((resolve,reject)=>{
    const tx=database.transaction(['fonts','preferences'],'readwrite');let failure;
    tx.oncomplete=resolve;tx.onabort=()=>reject(failure||tx.error);
    const fonts=tx.objectStore('fonts'), prefs=tx.objectStore('preferences');
    const current=fonts.getAll();current.onsuccess=()=>{
      const final=new Map([...merged.values()].map(r=>[r.id,r]));
      // Concurrent live records win over backup records too.
      for(const r of current.result)final.set(r.id,r);
      if([...final.values()].filter(r=>r.enabled).reduce((sum,r)=>sum+r.bytes.byteLength,0)>MAX_ACTIVE_BYTES){failure=Error('Backup would exceed the 256 MiB active-font limit.');tx.abort();return;}
      for(const r of final.values()){fonts.put(r);prefs.delete(r.id);}
    };
  });await changed();
}
export const onChange = fn => events.addEventListener('change', e=>fn(e.detail));
let defaultRun;
const defaultState={loading:false,completed:0,total:0,errors:[]};
export const defaultStatus=()=>({...defaultState,errors:[...defaultState.errors]});
// Install the bundled collection progressively. Two downloads run at a time and
// native engines continue consuming their existing two-face batches while loading.
export function activateDefaults() {
  if(defaultRun)return defaultRun;
  defaultRun=(async()=>{
    await ready;
    const data=await catalog(), faces=data.families.flatMap(family=>family.faces.map(face=>({face,family})));
    const existing=new Set((await list()).map(r=>r.id));
    const removed=new Set((await transaction('readonly',s=>s.getAll(),'preferences')).filter(r=>r.removed).map(r=>r.id));
    Object.assign(defaultState,{loading:true,completed:0,total:faces.length,errors:[]});
    let next=0;
    async function worker(){
      while(next<faces.length){
        const {face,family}=faces[next++];
        try{
          if(!existing.has(face.sha256)&&!removed.has(face.sha256)){
            const url=assetURL(face.path);
            const response=await fetch(url,{signal:AbortSignal.timeout(60000)});
            if(!response.ok)throw Error('Download failed');
            const bytes=await readFontResponse(response),meta=inspectFont(bytes),id=await digest(bytes);
            if(bytes.byteLength>MAX_FONT_BYTES||id!==face.sha256)throw Error('Font checksum mismatch');
            if(meta.noSubsetting||['restricted','bitmap-only','preview-print'].includes(meta.embedding))throw Error('Embedding restricted');
            const record={...meta,id,bytes,source:{kind:'catalog',family:family.id,license:family.license,licensePath:family.licensePath,url:face.url},enabled:true,favorite:false,addedAt:new Date().toISOString()};
            await preview(record);
            await insertRecord(record, true);
            await refresh();
          }
        }catch(e){defaultState.errors.push(`${family.name}: ${e.message}`);}
        defaultState.completed++;emit({defaults:defaultStatus()});
      }
    }
    await Promise.all([worker(),worker()]);
    await changed();
    return defaultStatus();
  })().catch(e=>{defaultState.errors.push(e.message);}).finally(()=>{
    defaultState.loading=false;defaultRun=null;emit({defaults:defaultStatus()});
  });
  return defaultRun;
}
// Versioned adapter API. The native app consumes bytes, not CSS font names.
window.CraftFonts = {
  version:1,
  take(){const batch=[...pending.values()].slice(0,2);for(const r of batch){pending.delete(r.id);inflight.set(r.id,r);}if(pending.size)setTimeout(()=>window.dispatchEvent(new Event('craft:font-change')),50);return batch.map(r=>({id:r.id,family:r.family,style:r.style,bytes:new Uint8Array(r.bytes)}));},
  ack(id,ok,message){if(!inflight.has(id))return;inflight.delete(id);if(ok)acknowledged.add(id);emit({id,ok,message});},
  retry(id){const r=active.find(r=>r.id===id);if(!r)return false;inflight.delete(id);acknowledged.delete(id);pending.set(id,r);window.dispatchEvent(new Event('craft:font-change'));return true;},
  status(){return {ready:booted,active:active.length,loaded:acknowledged.size,defaults:defaultStatus()};}
};
export const ready=refresh().then(()=>{booted=true;}).catch(e=>{emit({error:e.message});throw e;});
ready.catch(()=>{});
// Start without requiring a visit to the font manager. Explicitly disabled or
// removed faces stay that way; failed downloads are retried on the next visit.
export const defaultsReady=autoActivate ? activateDefaults() : ready;
