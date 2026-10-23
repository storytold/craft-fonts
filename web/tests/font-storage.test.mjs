import {test} from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {pathToFileURL} from 'node:url';
await import('fake-indexeddb/auto');
globalThis.window=new EventTarget();
globalThis.document={fonts:{add(){}}};
globalThis.location={origin:'https://craft.test',href:'https://craft.test/photocraft/',pathname:'/photocraft/'};
window.CraftFontsConfig={baseURL:'https://craft.test/fonts/',assetBaseURL:'https://craft.test/'};
globalThis.BroadcastChannel=class{postMessage(){}close(){}};
// Raster decoding is covered separately by font metadata QA/native tests. This tests storage only.
globalThis.FontFace=class{async load(){return this;}};
Object.defineProperty(globalThis,'navigator',{value:{storage:{estimate:async()=>({usage:1,quota:1000}),persisted:async()=>false,persist:async()=>true}},configurable:true});
const library=await import('../library.mjs');
await library.ready;
const catalog=JSON.parse(readFileSync(new URL('../catalog.json',import.meta.url)));
const b=readFileSync(new URL('../../'+catalog.families[0].faces[0].path,import.meta.url));
const bytes=b.buffer.slice(b.byteOffset,b.byteOffset+b.byteLength);
test('shared storage survives module restart, deduplicates and preserves activation/favorites',async()=>{
 const first=await library.install(bytes),second=await library.install(bytes);
 assert.equal(first.id,second.id);assert.equal((await library.list()).length,1);
 await library.favorite(first.id);
 await library.enable(first.id,false);
 const next=await import('../library.mjs?second-app');await next.ready;
 const [record]=await next.list();assert.equal(record.id,first.id);assert.equal(record.favorite,true);assert.equal(record.enabled,false);
 await next.enable(first.id,true);const queue=window.CraftFonts.take();assert.equal(queue.length,1);assert.deepEqual(queue[0].bytes,new Uint8Array(bytes));
 window.CraftFonts.ack(first.id,true,'loaded');assert.equal(window.CraftFonts.status().loaded,1);
});
test('backup restores bytes and invalid restore is atomic',async()=>{
 const original=await library.list(),backup=await library.backup();
 const payload=JSON.parse(await backup.text());payload.fonts.push({...payload.fonts[0],id:'bad'});
 await assert.rejects(library.restore(new Blob([JSON.stringify(payload)])),/Invalid/);
 assert.deepEqual((await library.list()).map(r=>r.id),original.map(r=>r.id));
 await library.remove(original[0].id);assert.equal((await library.list()).length,0);
 await library.restore(backup);assert.deepEqual(new Uint8Array((await library.list())[0].bytes),new Uint8Array(bytes));
});
test('malformed input and restricted embedding never reach storage',async()=>{
 const before=(await library.list()).length;
 await assert.rejects(library.install(new ArrayBuffer(12)));
 const restricted=bytes.slice(0),v=new DataView(restricted);
 for(let i=0;i<v.getUint16(4);i++){const p=12+i*16;const tag=String.fromCharCode(...new Uint8Array(restricted,p,4));if(tag==='OS/2')v.setUint16(v.getUint32(p+8)+8,2);}
 await assert.rejects(library.install(restricted),/embedding/);assert.equal((await library.list()).length,before);
 await assert.rejects(library.restore(new Blob(['{"format":"wrong"}'])),/Unsupported/);
});

 test('Font Library source and licensing survive backup and restore',async()=>{
  const all=await library.list();for(const r of all)await library.remove(r.id);
  const source={kind:'fontlibrary',family:'test-family',url:'https://fontlibrary.org/assets/fonts/test/family.ttf',sourceUrl:'https://fontlibrary.org/en/font/test-family',license:'OFL',licenseUrl:'https://openfontlicense.org',designer:'Test Designer',catalogRetrievedAt:'2026-10-11'};
  const record=await library.install(bytes,source);const file=await library.backup();await library.remove(record.id);await library.restore(file);
  assert.deepEqual((await library.list())[0].source,source);
 });
