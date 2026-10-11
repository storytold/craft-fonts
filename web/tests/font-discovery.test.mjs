import {test} from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
globalThis.window={CraftFontsConfig:{baseURL:'https://craft.test/fonts/',assetBaseURL:'https://craft.test/'}};
const {familyAvailable,faceAvailable,fontURL,normalize,openLicense,selectFamilies,regularFace,downloadFace,provenance}=await import('../discovery.mjs');
const families=[{id:'cooper',name:'Cooper Hewitt',designer:'Village',category:'Sans-serif',languages:'Latin',license:'OFL (SIL Open Font License)',faces:[{name:'Bold',url:'https://fontlibrary.org/assets/fonts/cooper/bold.otf'},{name:'Book',url:'https://fontlibrary.org/assets/fonts/cooper/book.otf'}]},{id:'liberation',name:'Liberation Serif',category:'Serif',license:'SIL Open Font License',faces:[]},{id:'restricted',name:'Demo',category:'Display',license:'CC-BY-NC',faces:[]}];
test('discovery hides available families, searches metadata, and sorts available last',()=>{
 const bundled=[{name:'Cooper Hewitt'}];assert.equal(familyAvailable(families[0],[],bundled),true);
 assert.deepEqual(selectFamilies(families,{bundled}).map(f=>f.id),['liberation']);
 assert.deepEqual(selectFamilies(families,{bundled,showAvailable:true}).map(f=>f.id),['liberation','cooper']);
 assert.deepEqual(selectFamilies(families,{query:'village latin'}).map(f=>f.id),['cooper']);
 assert.equal(selectFamilies(families,{category:'Serif'}).length,1);
 assert.equal(selectFamilies(families,{license:'all'}).length,3);
 assert.equal(familyAvailable(families[0],[{family:'Other',source:{kind:'fontlibrary',family:'cooper'}}],[]),true);
 assert.equal(familyAvailable(families[0],[{family:'Cooper-Hewitt'}],[]),true);
 assert.equal(regularFace(families[0]).name,'Book');
});
test('download allowlist and license filter reject unsupported sources',()=>{
 for(const url of ['http://fontlibrary.org/assets/fonts/a.ttf','https://evil.test/a.ttf','https://fontlibrary.org.evil.test/assets/fonts/a.ttf','https://fontlibrary.org/assets/fonts/a.woff2','https://a:secret@fontlibrary.org/assets/fonts/a.ttf','https://fontlibrary.org/assets/fonts/a.ttf?q=1','https://fontlibrary.org:444/assets/fonts/a.ttf'])assert.throws(()=>fontURL(url));
 assert.equal(openLicense('CC-BY-NC'),false);assert.equal(openLicense('Unknown'),false);assert.equal(openLicense('Apache 2.0'),true);
 assert.throws(()=>provenance(families[2],families[0].faces[0],'2026'),/license/);
 const p=provenance(families[0],families[0].faces[0],'2026');assert.equal(p.license,families[0].license);assert.equal(p.catalogRetrievedAt,'2026');
 assert.equal(faceAvailable(families[0].faces[0],[{source:p}]).source.kind,'fontlibrary');
});
test('downloads are bounded, parsed, shared between preview/import, and retry after failure',async()=>{
 const data=JSON.parse(readFileSync(new URL('../catalog.json',import.meta.url)));
 const bytes=readFileSync(new URL('../../'+data.families[0].faces[0].path,import.meta.url));
 let calls=0;globalThis.fetch=async()=>{calls++;return new Response(bytes);};
 const a=await downloadFace(families[0].faces[0]);const b=await downloadFace(families[0].faces[0]);assert.equal(calls,1);assert.equal(a.id,b.id);assert.equal(a.id.length,64);assert.equal(a.bytes.byteLength,bytes.byteLength);
 globalThis.fetch=async()=>new Response('bad',{headers:{'content-length':String(25*1024*1024)}});
 await assert.rejects(downloadFace(families[0].faces[1]),/24 MiB/);
 globalThis.fetch=async()=>new Response('not a font');await assert.rejects(downloadFace(families[0].faces[1]));
 globalThis.fetch=async()=>new Response(bytes);assert.equal((await downloadFace(families[0].faces[1])).id,a.id);
});
test('featured and style-count sorts preserve the available-font ordering',()=>{
 const input=[...families,{...families[0],id:'avara',name:'Avara',faces:[{}, {}, {}]}];
 assert.equal(selectFamilies(input)[0].name,'Avara');
 assert.equal(selectFamilies(input,{sort:'styles'})[0].name,'Avara');
 assert.equal(selectFamilies(input,{sort:'featured',bundled:[{name:'Avara'}],showAvailable:true}).at(-1).name,'Avara');
});

test('duplicate detection preserves non-Latin family identities',()=>{assert.notEqual(normalize('明朝体'),normalize('黑体'));assert.equal(normalize('Cooper-Hewitt'),normalize('Cooper Hewitt'));});
test('snapshot contains validated metadata and bounded descriptions',async()=>{
 const {existsSync}=await import('node:fs');const path=new URL('../fontlibrary-catalog.json',import.meta.url);
 assert.ok(existsSync(path),'Discovery snapshot must exist');
 const data=JSON.parse(readFileSync(path));assert.equal(data.version,1);assert.ok(data.families.length>=1000);
 const ids=new Set();
 for(const f of data.families){assert.ok(!ids.has(f.id));ids.add(f.id);assert.ok(f.name);assert.ok(f.sourceUrl.startsWith('https://fontlibrary.org/en/font/'));assert.ok((f.description||'').split(/\s+/).length<=20);for(const face of f.faces)assert.equal(fontURL(face.url),face.url);}
});
