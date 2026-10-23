"""Refresh searchable Font Library metadata. No font binaries are bundled.
Uses only fontlibrary.org public catalogue/detail/CSS pages, at <= 12 concurrent requests.
Cached responses make interrupted refreshes resumable. Review the diff before publishing.
"""
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path
from urllib.request import urlopen, Request
from urllib.parse import urljoin, urlparse, quote
from urllib.error import HTTPError
from datetime import datetime, timezone
from email.utils import parsedate_to_datetime
import argparse, hashlib, html, json, re, time
ROOT = Path(__file__).resolve().parents[2]
CACHE = ROOT / '.cache/fontlibrary'
CACHE.mkdir(parents=True, exist_ok=True)
BASE = 'https://fontlibrary.org'
def fetch(path):
    url = quote(urljoin(BASE, path),safe=':/?=&%')
    assert urlparse(url).hostname == 'fontlibrary.org'
    cache = CACHE / (hashlib.sha256(url.encode()).hexdigest()+'.txt')
    if cache.exists(): return cache.read_text()
    for attempt in range(3):
        try:
            with urlopen(Request(url, headers={'User-Agent':'CraftFontCatalog/1.0'}), timeout=35) as r:
                value=r.read(8*1024*1024).decode('utf-8')
            cache.write_text(value)
            return value
        except HTTPError as e:
            if e.code==404:
                cache.write_text('');return ''
            if attempt==2: raise
            time.sleep(attempt+1)
        except Exception:
            if attempt == 2: raise
            time.sleep(attempt+1)
def text(value): return html.unescape(re.sub(r'<[^>]*>', '', value)).strip()
def field(doc, name):
    found = re.search(r'<dt>\s*'+re.escape(name)+r'\s*</dt>\s*<dd>(.*?)</dd>', doc, re.S|re.I)
    return found.group(1) if found else ''
def catalogue(page):
    doc=fetch('/en/catalogue?order=name&page='+str(page))
    rows=[]
    for block in doc.split('<div class="fonts-specimen">')[1:]:
        match=re.search(r'class="family-name heading"><a href="[^"]*/font/([^"]+)">([^<]+)</a>',block)
        if not match: continue
        slug,name=match.groups()
        designer=re.search(r'class="byline">(.*?)</span>',block,re.S)
        rows.append({'id':slug,'name':text(name),'designer':text(designer.group(1)) if designer else ''})
    return rows

def family(row):
    doc=fetch('/en/font/'+row['id'])
    css=fetch('/en/face/'+row['id'])
    license_markup=field(doc,'License')
    license_url=re.search(r'href="([^"]+)"',license_markup)
    download=re.search(r'href="([^"]+\.zip)"',doc)
    faces=[]
    for face in re.findall(r'@font-face\s*\{([^}]+)\}',css,re.S):
        name=re.search(r"font-family:\s*['\"]([^'\"]+)",face)
        source=re.search(r"url\(['\"]?([^)'\"]+\.(?:ttf|otf))['\"]?\)",face,re.I)
        if not name or not source: continue
        url=quote(urljoin(BASE,source.group(1)),safe=':/%')
        if urlparse(url).hostname!='fontlibrary.org' or not urlparse(url).path.startswith('/assets/fonts/'): continue
        if any(f['url']==url for f in faces): continue
        faces.append({'name':name.group(1),'url':url})
    row.update(category=text(field(doc,'Category')),license=text(license_markup),licenseUrl=html.unescape(license_url.group(1)) if license_url else '',languages=text(field(doc,'Full Language Support')),description=' '.join(text(field(doc,'Description')).split()[:20]),sourceUrl=BASE+'/en/font/'+row['id'],downloadUrl=urljoin(BASE,download.group(1)) if download else '',faces=faces)
    return row

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--refresh',action='store_true',help='Discard cached metadata before a fresh refresh')
    args=parser.parse_args()
    if args.refresh:
        for old in CACHE.glob('*.txt'): old.unlink()
    rows=[]
    # Paginate until the first empty page. Requests in small batches avoid a hardcoded count.
    with ThreadPoolExecutor(max_workers=12) as pool:
        page=1
        while True:
            batch=list(pool.map(catalogue,range(page,page+6)))
            for group in batch: rows.extend(group)
            print('Catalog:',len(rows),'families',flush=True)
            if any(not group for group in batch): break
            page+=6
            if page>300: raise RuntimeError('Pagination exceeded expected bounds')
        rows=list({r['id']:r for r in rows}.values())
        results=[];failures=[]
        jobs={pool.submit(family,row):row for row in rows}
        for future in as_completed(jobs):
            row=jobs[future]
            try: results.append(future.result())
            except Exception as e:
                failures.append({'id':row['id'],'error':str(e)});print('Unavailable:',row['id'],str(e),flush=True)
            if (len(results)+len(failures))%100==0: print('Details:',len(results),'failures:',len(failures),flush=True)
    if failures:
        (CACHE/'failures.json').write_text(json.dumps(failures,indent=2))
        raise RuntimeError(f'{len(failures)} families failed. Rerun to resume. Existing catalog unchanged.')
    if len(results)<1000: raise RuntimeError('Unexpected catalog shrinkage; review before publishing')
    # Prefer the source server's clock for a portable, auditable retrieval date.
    with urlopen(Request(BASE+'/en/catalogue',method='HEAD'),timeout=35) as response:
        retrieved=parsedate_to_datetime(response.headers['Date']).isoformat()
    if sum(bool(r['license']) for r in results)<900 or sum(len(r['faces']) for r in results)<1000:
        raise RuntimeError('Source structure or licensing metadata changed; review before publishing')
    result={'version':1,'source':'https://fontlibrary.org','retrievedAt':retrieved,'families':sorted(results,key=lambda r:r['name'].casefold())}
    target=ROOT/'web/fontlibrary-catalog.json'
    target.write_text(json.dumps(result,ensure_ascii=False,separators=(',',':'))+'\n')
    print('Saved',len(results),'families,',sum(len(r['faces']) for r in results),'faces',flush=True)
if __name__=='__main__': main()
