import {test} from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import 'fake-indexeddb/auto';
globalThis.window=new EventTarget();
window.CraftFontsConfig={baseURL:'https://craft.test/nested/library/',assetBaseURL:'https://craft.test/nested/library/',databaseName:'portable-font-test'};
globalThis.document={fonts:{add(){}}};
globalThis.location={origin:'https://craft.test'};
globalThis.BroadcastChannel=class{postMessage(){}};
globalThis.FontFace=class{async load(){return this;}};
const catalog=JSON.parse(readFileSync(new URL('../catalog.json',import.meta.url)));
let requests=0,fail=true;
globalThis.fetch=async url=>{
 requests++;
 const u=new URL(url);
 if(u.pathname.endsWith('/catalog.json'))return new Response(JSON.stringify(catalog));
 const face=catalog.families.flatMap(f=>f.faces).find(f=>u.pathname.endsWith('/'+f.path));
 assert.ok(face);
 if(fail&&face===catalog.families[0].faces[0])return new Response('',{status:503});
 return new Response(readFileSync(new URL('../../'+face.path,import.meta.url)));
};
const lib=await import('../library.mjs');
await lib.ready;
test('defaults are opt-in; retry preserves disabled and removed faces',async()=>{
 assert.equal(requests,0);
 await lib.activateDefaults();assert.equal(lib.defaultStatus().errors.length,1);
 fail=false;await lib.activateDefaults();assert.equal((await lib.list()).length,7);
 const records=await lib.list();await lib.enable(records[0].id,false);await lib.remove(records[1].id);
 const before=requests;await lib.activateDefaults();assert.equal(requests,before);
 assert.equal((await lib.list()).find(r=>r.id===records[0].id).enabled,false);
});
test('byte bridge batches, acknowledges only delivered fonts and supports explicit retries',async()=>{
 const batch=window.CraftFonts.take();assert.equal(batch.length,2);
 assert.ok(batch[0].bytes instanceof Uint8Array);
 window.CraftFonts.ack('unknown',true);assert.equal(window.CraftFonts.status().loaded,0);
 window.CraftFonts.ack(batch[0].id,false,'Unsupported');
 assert.equal(window.CraftFonts.retry(batch[0].id),true);
 window.CraftFonts.ack(batch[1].id,true);assert.equal(window.CraftFonts.status().loaded,1);
 await lib.favorite(batch[1].id);
 const next=window.CraftFonts.take();assert.equal(next.some(r=>r.id===batch[1].id),false);
 assert.equal(window.CraftFonts.retry('unknown'),false);
});
test('portable asset paths reject traversal and other origins',async()=>{
 const {assetURL}=await import('../config.mjs');
 assert.equal(assetURL('fonts/example.ttf').href,'https://craft.test/nested/library/fonts/example.ttf');
 for(const path of ['../outside.ttf','https://evil.test/fonts/a.ttf','/other/a.ttf','fonts/a.ttf?q=1'])assert.throws(()=>assetURL(path));
});
test('oversized bundled response is rejected before consuming data',async()=>{
 const face=catalog.families[0].faces[0];
 globalThis.fetch=async()=>new Response('x',{headers:{'content-length':String(25*1024*1024)}});
 await assert.rejects(lib.installCatalog(face,catalog.families[0]),/24 MiB/);
});
