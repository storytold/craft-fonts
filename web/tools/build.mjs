#!/usr/bin/env node
// Dependency-free static packaging from the repository's existing manifest.
import {readFile, writeFile, mkdir, copyFile, cp} from 'node:fs/promises';
import {resolve, dirname, relative, sep} from 'node:path';
import {fileURLToPath} from 'node:url';
import {createHash} from 'node:crypto';
const root=fileURLToPath(new URL('../',import.meta.url));
const output=resolve(process.argv[2] || resolve(root,'../.cache/web-fonts'));
const groups=new Map();
function safe(path){const full=resolve(root,'..',path);if(relative(resolve(root,'..'),full).startsWith('..'+sep))throw Error(`Path escapes checkout: ${path}`);return full;}
for(const [i,line] of (await readFile(resolve(root,'../fonts/manifest.txt'),'utf8')).split('\n').entries()) {
 if(!line.trim()||line.trim().startsWith('#'))continue;
 const fields=line.split(' | ').map(s=>s.trim());
 if(fields.length!==8||fields.some(s=>!s))throw Error(`Invalid manifest line ${i+1}`);
 const [name,style,path,scripts,license,licensePath,sha256,url]=fields;
 const bytes=await readFile(safe(path));
 if(bytes.length>24*1024*1024||createHash('sha256').update(bytes).digest('hex')!==sha256)throw Error(`Integrity/size mismatch: ${path}`);
 for(const item of [path,licensePath]){await mkdir(dirname(resolve(output,item)),{recursive:true});await copyFile(safe(item),resolve(output,item));}
 if(!groups.has(name))groups.set(name,{id:dirname(path).split('/').pop(),name,category:'',scripts:scripts.split(','),license,licensePath,faces:[]});
 groups.get(name).faces.push({path,sha256,url,name:style,style,size:bytes.length});
}
await mkdir(output,{recursive:true});
await writeFile(resolve(output,'catalog.json'),JSON.stringify({schemaVersion:1,families:[...groups.values()]},null,2)+'\n');
await cp(resolve(root,'example.html'),resolve(output,'index.html'));
for(const file of ['library.mjs','config.mjs','manager.mjs','discovery.mjs','font-metadata.mjs','library.css','fontlibrary-catalog.json'])await copyFile(resolve(root,file),resolve(output,file));
for(const file of ['LICENSE-MIT','LICENSE-APACHE','ATTRIBUTION.md'])await copyFile(resolve(root,'..',file),resolve(output,file));
console.log(`Packaged ${groups.size} families to ${output}`);
