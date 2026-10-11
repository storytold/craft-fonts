import {assetURL, supportURL} from './config.mjs';
import * as lib from './library.mjs';
import * as discovery from './discovery.mjs';
let dialog, grid, status, search, previewText, category, available, license, sort, count, more, remoteNote, tabBar, storageNote, retry;
let bundled, remote, installed=[], scope='discover', page=1, revision=0, reloadRequired=false, busy=false;
function styleLabel(family,face){const compact=family.name.replace(/[^a-z0-9]/gi,'');const name=face.name||'Regular';const style=name.toLowerCase().startsWith(compact.toLowerCase())?name.slice(compact.length):name;return (style||'Regular').replace(/([a-z])([A-Z])/g,'$1 $2').replace(/[_-]/g,' ').trim();}
const PAGE_SIZE=24, sampleText='The quick brown fox • 0123456789';
const previews=new Map();let observer, previewQueue=[], previewJobs=0;
function el(tag,text,cls){const n=document.createElement(tag);if(text)n.textContent=text;if(cls)n.className=cls;return n;}
function button(text,fn,cls){const b=el('button',text,cls);b.type='button';b.onclick=fn;return b;}
function link(text,href){const a=el('a',text);try{const u=new URL(href,location.origin);if(!['https:','http:'].includes(u.protocol))return el('span',text);a.href=u.href;a.target='_blank';a.rel='noopener noreferrer';}catch{return el('span',text);}return a;}
function label(text,input){const n=el('label',text);n.append(input);return n;}
function select(name,options){const s=el('select');s.setAttribute('aria-label',name);for(const [value,text] of options){const o=el('option',text);o.value=value;s.append(o);}return s;}
function errorMessage(e){return e.name==='QuotaExceededError'?'Browser storage is full. Remove unused fonts from My fonts, then retry.':e.message;}
function announce(message,error=false){status.textContent=message;status.classList.toggle('is-error',error);}
function download(blob,name){const a=el('a');a.href=URL.createObjectURL(blob);a.download=name;a.click();setTimeout(()=>URL.revokeObjectURL(a.href),5000);}
async function action(fn,success='Font library updated.'){
 if(busy)return;busy=true;dialog.classList.add('is-working');
 try{await fn();installed=await lib.list();render();announce(success+(reloadRequired?' Save your work and reload open editors to finish removing fonts.':''));}
 catch(e){installed=await lib.list().catch(()=>installed);render();announce(errorMessage(e),true);}
 finally{busy=false;dialog.classList.remove('is-working');}
}
function updatePreviewText(){dialog.querySelectorAll('[data-font-sample]').forEach(n=>n.textContent=previewText.value||sampleText);}
function paintPreview(sample,font){sample.style.fontFamily=`"${font}"`;sample.dataset.fontSample='';sample.textContent=previewText.value||sampleText;sample.classList.add('is-loaded');}
async function remotePreview(face,sample,note,token){
 const url=face.url;
 try{
  let item=previews.get(url);
  if(!item){
   const data=await discovery.downloadFace(face);
   const font=new FontFace('Discover-'+data.id,data.bytes);
   await font.load();document.fonts.add(font);item={name:font.family,font};previews.set(url,item);
   // Bound decoded preview faces as well as their original byte cache.
   while(previews.size>48){const [key,old]=previews.entries().next().value;document.fonts.delete(old.font);previews.delete(key);}
  }
  if(token!==revision || !sample.isConnected)return;
  paintPreview(sample,item.name);note.textContent='Preview only · not imported';
 }catch(e){if(token!==revision || !sample.isConnected)return;sample.textContent='Preview unavailable';note.replaceChildren(button('Retry preview',()=>queuePreview(face,sample,note,token),'font-text-button'));note.append(el('span', ' '+e.message));}
}
function queuePreview(face,sample,note,token){previewQueue.push({face,sample,note,token});queueMicrotask(pumpPreviews);}
function pumpPreviews(){while(previewJobs<2&&previewQueue.length){const job=previewQueue.shift();if(job.token!==revision||!job.sample.isConnected)continue;previewJobs++;remotePreview(job.face,job.sample,job.note,job.token).finally(()=>{previewJobs--;pumpPreviews();});}}
function attachPreview(face,sample,note){const token=revision;sample.textContent='Loading font preview…';note.textContent='Preview from Font Library';if(observer){sample._previewJob={face,sample,note,token};observer.observe(sample);}else queuePreview(face,sample,note,token);}
function localPreview(record,sample,note){if(!record){sample.textContent='Not downloaded';return;}const token=revision;lib.previewFont(record).then(name=>{if(token===revision&&sample.isConnected){paintPreview(sample,name);note.textContent=record.enabled?'Available in supported app text tools':'Imported · inactive';}}).catch(()=>{sample.textContent='Preview unavailable';});}
function refreshFilters(){
 category.replaceChildren();const categories=scope==='discover'?[...new Set((remote?.families||[]).map(f=>f.category).filter(Boolean))].sort():scope==='bundled'?[...new Set(bundled.families.map(f=>f.category))].sort():[];
 for(const value of ['',...categories]){const o=el('option',value||'All categories');o.value=value;category.append(o);}
 if(scope==='mine'){category.replaceChildren();for(const [value,text] of [['','All styles'],['active','Active'],['inactive','Inactive'],['favorites','Favorites']]){const o=el('option',text);o.value=value;category.append(o);}}
 category.hidden=false;available.parentElement.hidden=scope!=='discover';license.hidden=scope!=='discover';sort.hidden=scope!=='discover';remoteNote.hidden=scope!=='discover';
 for(const b of tabBar.children){b.setAttribute('aria-pressed',String(b.dataset.scope===scope));}
 search.placeholder=scope==='discover'?'Search families, designers, or languages':'Search your font families and styles';
}
function changeScope(value){scope=value;page=1;search.value='';refreshFilters();render();}
function familyCard(family){
 const isRemote=scope==='discover';const card=el('article',null,'font-card');
 const title=el('h3',family.name);const heading=el('div',null,'font-card-heading');heading.append(title);
 const isAvailable=isRemote&&discovery.familyAvailable(family,installed,bundled.families);
 if(isAvailable)heading.append(el('span','In your library','font-badge'));
 const meta=el('p',`${family.category||'Font family'} · ${family.faces.length} ${family.faces.length===1?'style':'styles'}`,'font-meta');
 const sample=el('p',null,'font-sample');sample.setAttribute('aria-label',`${family.name} preview`);
 const previewNote=el('p',null,'font-preview-note');
 const choice=select(`Style for ${family.name}`,family.faces.map((face,i)=>[i,isRemote?styleLabel(family,face):face.subfamilyName||face.style||'Regular']));
 if(isRemote){choice.value=family.faces.indexOf(discovery.regularFace(family));}
 const actions=el('div',null,'font-actions');
 const importButton=button(isRemote?'Import style':'Activate',()=>{
  const face=family.faces[Number(choice.value)];
  return action(async()=>{announce(`Importing ${family.name}…`);if(isRemote)await importFace(family,face);else await lib.installCatalog(face,family);},`${family.name} is available in your shared library.`);
 },'font-primary');
 const preview=()=>{
  const face=family.faces[Number(choice.value)];
  const current=isRemote?discovery.faceAvailable(face,installed):installed.find(r=>r.id===face.sha256);
  importButton.textContent=current?.enabled?'Available':current?'Activate':isRemote?'Import style':'Activate';importButton.disabled=!!current?.enabled || (isRemote&&!discovery.openLicense(family.license));
  if(current)localPreview(current,sample,previewNote);
  else if(isRemote)attachPreview(face,sample,previewNote);
  else {sample.textContent='Bundled with *Craft';previewNote.textContent='Activate this style to preview it.';}
 };
 // Separate specimen node per selection prevents an older download painting a newer style.
 // Preserve style choice without rebuilding cards or losing focus.
 choice.onchange=()=>{
  const face=family.faces[Number(choice.value)],current=isRemote?discovery.faceAvailable(face,installed):installed.find(r=>r.id===face.sha256);
  const fresh=el('p',null,'font-sample');fresh.setAttribute('aria-label',`${family.name} preview`);card.querySelector('.font-sample').replaceWith(fresh);
  importButton.textContent=current?.enabled?'Available':current?'Activate':isRemote?'Import style':'Activate';importButton.disabled=!!current?.enabled||(isRemote&&!discovery.openLicense(family.license));
  if(current)localPreview(current,fresh,previewNote);else if(isRemote)attachPreview(face,fresh,previewNote);else fresh.textContent='Activate this style to preview it.';
 };
 actions.append(choice,importButton);
 card.append(heading,meta,sample,previewNote,actions);
 if(isRemote){
  const footer=el('div',null,'font-card-footer');footer.append(el('span',family.license||'License not listed'),button('Details',()=>showDetails(family),'font-text-button'));card.append(footer);
  if(!family.faces.length){importButton.disabled=true;previewNote.textContent='No supported TTF / OTF files';sample.textContent='See source for available formats';}else preview();
 }else{const footer=el('div',null,'font-card-footer');footer.append(el('span','Included fonts'),link('License',assetURL(family.licensePath)));card.append(footer);preview();}
 return card;
}
async function importFace(family,face){
 const current=discovery.faceAvailable(face,installed);
 if(current){if(!current.enabled)await lib.enable(current.id,true);return current;}
 const source=discovery.provenance(family,face,remote.retrievedAt);
 const data=await discovery.downloadFace(face);return lib.install(data.bytes,source);
}
function showDetails(family){
 const detail=el('dialog',null,'craft-font-detail');detail.setAttribute('aria-label',`${family.name} details`);
 const header=el('header');header.append(el('div','Font Library · family details','font-eyebrow'),button('Close',()=>detail.close(),'font-close'));
 detail.append(header,el('h2',family.name),el('p',family.designer||'Designer not listed','font-meta'),el('p',family.description||'No description provided.','font-description'));
 const info=el('dl');for(const [name,value] of [['Category',family.category],['License',family.license||'Not listed'],['Languages',family.languages||'Not listed'],['Styles',String(family.faces.length)]]){info.append(el('dt',name),el('dd',value));}detail.append(info);
 const links=el('div',null,'font-actions');links.append(link('View source & full license',family.sourceUrl));if(family.licenseUrl)links.append(link('License terms',family.licenseUrl));if(family.downloadUrl)links.append(link('Original font package',family.downloadUrl));detail.append(links);
 detail.append(el('p','Fonts are downloaded from Font Library and saved only in this browser. License and designer details are retained with the import. Keep the original package for its full license and notices.','font-meta'));
 const progress=el('p','', 'font-status');progress.setAttribute('role','status');
 let cancel, cancelled=false;
 const add=button(`Import all ${family.faces.length} styles`,async()=>{
  if(busy)return;busy=true;add.disabled=true;stop.disabled=false;cancel=new AbortController();let done=0,attempted=0;const failures=[];cancelled=false;
  try {
   for(const face of family.faces){
    if(cancel.signal.aborted){cancelled=true;break;}
    progress.textContent=`Importing ${++attempted} of ${family.faces.length}: ${styleLabel(family,face)}`;
    try{await importFace(family,face);done++;}catch(e){failures.push(`${styleLabel(family,face)}: ${errorMessage(e)}`);}
   }
   installed=await lib.list();render();
   progress.textContent=`${done} of ${family.faces.length} styles ready.${cancelled?' Import stopped. Imported styles were kept.':''}${failures.length?' Some styles could not be imported: '+failures.join(' · '):cancelled?'':' Available in your shared library.'}`;
   add.textContent=cancelled?'Resume import':failures.length?'Retry missing styles':'Import complete';add.disabled=!failures.length&&!cancelled;
  }catch(e){progress.textContent=e.message;add.disabled=false;}
  finally{busy=false;stop.disabled=true;}
 });
 const stop=button('Stop import',()=>{cancel?.abort();progress.textContent='Stopping after the current font…';},'font-text-button');stop.disabled=true;
 add.className='font-primary';add.disabled=!family.faces.length||!discovery.openLicense(family.license);
 detail.append(progress,add,stop);
 if(!discovery.openLicense(family.license))detail.append(el('p','This license needs review. Automatic import is unavailable. See the source for license details.','font-meta'));
 detail.addEventListener('close',()=>{cancel?.abort();detail.remove();});document.body.append(detail);detail.showModal();
}
function myCard(record){
 const card=el('article',null,'font-card');const head=el('div',null,'font-card-heading');head.append(el('h3',record.family),el('span',record.enabled?'Active':'Inactive','font-badge'));
 const sample=el('p',null,'font-sample'),note=el('p',null,'font-preview-note');localPreview(record,sample,note);
 const actions=el('div',null,'font-actions');actions.append(button(record.enabled?'Deactivate':'Activate',()=>action(()=>lib.enable(record.id,!record.enabled))),button(record.favorite?'★ Favorite':'☆ Favorite',()=>action(()=>lib.favorite(record.id))),button('Remove',()=>{if(confirm(`Remove ${record.family} ${record.style} from this browser? Saved documents may need it again.`))action(()=>lib.remove(record.id));}));
 card.append(head,el('p',`${record.style} · ${(record.bytes.byteLength/1024).toFixed(0)} KB · ${record.source?.kind==='fontlibrary'?'Font Library':record.source?.kind==='catalog'?'Bundled':'Local file'}`,'font-meta'),sample,note,actions);
 if(record.source?.sourceUrl)card.append(link('Source & license',record.source.sourceUrl));return card;
}
function render(){
 if(!bundled)return;revision++;previewQueue=[];observer?.disconnect();grid.replaceChildren();
 let rows=[];
 if(scope==='discover')rows=discovery.selectFamilies(remote?.families||[],{query:search.value,category:category.value,license:license.value,sort:sort.value,showAvailable:available.checked,installed,bundled:bundled.families});
 else if(scope==='bundled')rows=bundled.families.filter(f=>(!category.value||f.category===category.value)&&`${f.name} ${(f.scripts||[]).join(' ')}`.toLowerCase().includes(search.value.toLowerCase()));
 else rows=installed.filter(r=>(!category.value||category.value==='active'&&r.enabled||category.value==='inactive'&&!r.enabled||category.value==='favorites'&&r.favorite)&&`${r.family} ${r.style}`.toLowerCase().includes(search.value.toLowerCase())).sort((a,b)=>a.family.localeCompare(b.family)||a.weight-b.weight);
 const end=Math.min(page*PAGE_SIZE,rows.length);count.textContent=`${rows.length.toLocaleString()} ${scope==='mine'?'styles':'families'}${scope==='discover'&&!available.checked?' to discover':''} · showing ${end}`;
 for(const row of rows.slice(0,end))grid.append(scope==='mine'?myCard(row):familyCard(row));
 if(!rows.length){const empty=el('div',null,'font-empty');empty.append(el('span','Aa','font-empty-icon'),el('h3',scope==='discover'&&!remote?'Catalog unavailable':'No fonts found'),el('p','Try a different name or category. Available fonts are hidden in Discover unless you show them.'));
 empty.append(button('Reset filters',()=>{search.value='';category.value='';license.value='open';available.checked=false;page=1;render();}));
 if(scope==='discover'&&!remote)empty.append(button('Retry catalog',async()=>{try{remote=await discovery.remoteCatalog();refreshFilters();render();}catch(e){announce(errorMessage(e),true);}}));grid.append(empty);}
 more.hidden=end>=rows.length;more.textContent=`Show ${Math.min(PAGE_SIZE,rows.length-end)} more`;
 const state=lib.defaultStatus();retry.hidden=!state.errors.length;
 storageNote.textContent=`${installed.length} stored styles · ${(installed.reduce((sum,r)=>sum+r.bytes.byteLength,0)/1024/1024).toFixed(0)} MB${state.loading?` · preparing bundled fonts ${state.completed}/${state.total}`:''}${reloadRequired?' · save work and reload editors after removing fonts':''}`;
}
function setup(){
 dialog=el('dialog',null,'craft-font-dialog');dialog.setAttribute('aria-labelledby','craft-font-title');
 const header=el('header',null,'font-header');const titles=el('div');titles.append(el('p','CRAFT / TYPE COLLECTION','font-eyebrow'));const title=el('h2','Find your next typeface');title.id='craft-font-title';titles.append(title,el('p','Discover fonts. Try your words. Make them yours.','font-intro'));header.append(titles,button('Close',()=>dialog.close(),'font-close'));
 tabBar=el('nav',null,'font-tabs');tabBar.setAttribute('aria-label','Font collections');for(const [value,text] of [['discover','Discover'],['mine','My fonts'],['bundled','Included fonts']]){const b=button(text,()=>changeScope(value));b.dataset.scope=value;tabBar.append(b);}
 const controls=el('div',null,'font-controls');search=el('input');search.type='search';search.setAttribute('aria-label','Search fonts');
 category=select('Category',[['','All categories']]);license=select('License filter',[['open','Open licenses'],['all','All licenses']]);
 sort=select('Sort fonts',[['featured','Featured first'],['name','Name A–Z'],['styles','Most styles']]);
 const first=el('div',null,'font-search-row');first.append(search,category,license,sort);
 previewText=el('input');previewText.type='text';previewText.maxLength=160;previewText.placeholder=sampleText;previewText.setAttribute('aria-label','Preview text');previewText.oninput=updatePreviewText;
 const size=el('input');size.type='range';size.min=18;size.max=64;size.value=32;size.setAttribute('aria-label','Preview font size');size.oninput=()=>dialog.style.setProperty('--sample-size',size.value+'px');
 available=el('input');available.type='checkbox';const availLabel=label('Show available fonts',available);availLabel.className='font-toggle';
 const second=el('div',null,'font-preview-controls');second.append(label('Preview text',previewText),label('Size',size),availLabel);controls.append(first,second);
 for(const n of [search,category,license,sort,available])n.oninput=()=>{page=1;render();};
 remoteNote=el('p','Search the Font Library catalog. Previews download font files from fontlibrary.org. Preview text stays on your device.','font-remote-note');
 const resultBar=el('div',null,'font-result-bar');count=el('p');count.setAttribute('role','status');resultBar.append(count,remoteNote);
 status=el('p','','font-status');status.setAttribute('role','status');status.setAttribute('aria-live','polite');
 grid=el('section',null,'font-grid');grid.setAttribute('aria-label','Font results');more=button('Show more',()=>{page++;render();},'font-load-more');
 const body=el('div',null,'font-body');body.append(controls,resultBar,status,grid,more);
 const footer=el('footer',null,'font-footer');const upload=el('input');upload.type='file';upload.accept='.ttf,.otf';upload.multiple=true;upload.hidden=true;
 upload.onchange=()=>action(async()=>{const failures=[];for(const file of upload.files){try{if(file.size>24*1024*1024)throw Error('File exceeds 24 MiB.');await lib.install(await file.arrayBuffer());}catch(e){failures.push(`${file.name}: ${e.message}`);}}upload.value='';if(failures.length)throw Error(failures.join(' · '));},'Your fonts are ready in the shared library.');
 const restore=el('input');restore.type='file';restore.accept='.json';restore.hidden=true;restore.onchange=()=>action(async()=>{if(restore.files[0])await lib.restore(restore.files[0]);restore.value='';},'Backup restored.');
 const utilities=el('details',null,'font-utilities');utilities.append(el('summary','Storage & backups'));const utilityActions=el('div',null,'font-actions');utilityActions.append(button('Back up fonts',()=>action(async()=>download(await lib.backup(),'craft-font-library.json'),'Backup downloaded.')),button('Restore backup',()=>restore.click()),button('Keep on this device',()=>action(async()=>{if(!await lib.persist())throw Error('Your browser did not grant persistent storage. Keep a backup of your fonts.');},'Persistent storage enabled. Keep a backup too.')));
 retry=button('Retry bundled downloads',()=>action(()=>lib.activateDefaults()));utilityActions.append(retry);utilities.append(utilityActions,el('p','Imports stay in this browser and are shared by the Craft apps on this site. Back up before clearing site data. Deactivation and removal require saving work and reloading open editors.','font-meta'),link('App support & limitations',supportURL));
 storageNote=el('p','','font-meta');footer.append(button('+ Upload fonts',()=>upload.click(),'font-upload'),storageNote,utilities,upload,restore);
 dialog.append(header,tabBar,body,footer);document.body.append(dialog);
 if(typeof IntersectionObserver==='function')observer=new IntersectionObserver(entries=>{for(const entry of entries){if(entry.isIntersecting&&entry.target._previewJob){const job=entry.target._previewJob;observer.unobserve(entry.target);queuePreview(job.face,job.sample,job.note,job.token);}}},{root:body,rootMargin:'100px'});
 dialog.addEventListener('close',()=>{revision++;previewQueue=[];observer?.disconnect();});
 let refreshTimer;
 lib.onChange(d=>{
  if(d.reloadRequired)reloadRequired=true;
  if(d.error)announce(d.error,true);
  if(d.ok===false)announce(`This app could not use a font: ${d.message||'unsupported font'}`,true);
  if(d.defaults?.loading){storageNote.textContent=`Preparing bundled fonts: ${d.defaults.completed}/${d.defaults.total}`;return;}
  if(d.external||d.defaults||d.reloadRequired){clearTimeout(refreshTimer);refreshTimer=setTimeout(async()=>{installed=await lib.list();if(dialog.open)render();},150);}
 });
}
export async function openFonts(){
 if(!dialog)setup();if(dialog.open)return;dialog.showModal();announce('Loading font collections…');
 try{
  await lib.ready;installed=await lib.list();let error='';
  try{bundled=await lib.catalog();}catch(e){bundled||={families:[]};error=e.message;}
  try{remote=await discovery.remoteCatalog();}catch(e){error=e.message;}
  if(!remote&&installed.length)scope='mine';
  refreshFilters();render();
  if(error)announce(error+' Your stored fonts are still available under My fonts.',true);
  else announce(`Font Library catalog · ${remote.families.length.toLocaleString()} families · updated ${new Date(remote.retrievedAt).toLocaleDateString()}.`);
  search.focus();
 }catch(e){announce(errorMessage(e),true);}
}
window.addEventListener('craft:fonts',()=>openFonts());
document.querySelector('[data-open-fonts]')?.addEventListener('click',openFonts);
if(document.querySelector('canvas')){const b=button('Aa',openFonts,'craft-font-open');b.title='Find and manage shared fonts';b.setAttribute('aria-label',b.title);document.body.append(b);}
