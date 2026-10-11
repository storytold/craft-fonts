#!/usr/bin/env python3
"""Acquire the pinned OFL font catalog and verify every file against Git blob IDs.

Requires Python 3.10+ and fonttools. Network is used only for missing assets.
python acquire_fonts.py --output assets/fonts --catalog catalog.json
Use --verify-only to check an existing cache without any network access.
"""
import argparse
import concurrent.futures
import hashlib
import json
import pathlib
import time
import urllib.parse
import urllib.request
from fontTools.ttLib import TTFont
from fontTools.unicodedata import script as unicode_script

ROOT = pathlib.Path(__file__).resolve().parent


def digest(data):
    return hashlib.sha256(data).hexdigest()


def git_digest(data):
    return hashlib.sha1(b"blob " + str(len(data)).encode() + b"\0" + data).hexdigest()


def acquire(entry, family_id, manifest, output, verify_only):
    destination = output / family_id / pathlib.PurePosixPath(entry["path"]).name
    url = "https://raw.githubusercontent.com/google/fonts/" + manifest["commit"] + "/" + urllib.parse.quote(entry["path"])
    if destination.exists():
        data = destination.read_bytes()
        if git_digest(data) != entry["gitBlobSha"] or (entry.get("sha256") and digest(data) != entry["sha256"]):
            raise ValueError(f"Integrity mismatch: {destination}. Remove the modified file explicitly before retrying.")
    else:
        if verify_only:
            raise FileNotFoundError(destination)
        for attempt in range(5):
            try:
                with urllib.request.urlopen(url, timeout=120) as response:
                    data = response.read()
                break
            except Exception:
                if attempt == 4:
                    raise
                time.sleep(2 ** attempt)
        if len(data) != entry["size"] or git_digest(data) != entry["gitBlobSha"] or (entry.get("sha256") and digest(data) != entry["sha256"]):
            raise ValueError(f"Downloaded file does not match pinned Git blob: {entry['path']}")
        destination.parent.mkdir(parents=True, exist_ok=True)
        temporary = destination.with_suffix(destination.suffix + ".part")
        temporary.write_bytes(data)
        temporary.replace(destination)
    return destination, {"path": "fonts/" + family_id + "/" + destination.name, "url": url, "sourceUrl": url,
                         "sha256": digest(data), "gitBlobSha": entry["gitBlobSha"], "size": len(data)}


def inspect_font(path, asset):
    with TTFont(path, lazy=False) as font:
        names = font["name"]
        axes = [{"tag": axis.axisTag, "min": axis.minValue, "default": axis.defaultValue,
                 "max": axis.maxValue, "name": names.getDebugName(axis.axisNameID)}
                for axis in font["fvar"].axes] if "fvar" in font else []
        cmap = font.getBestCmap() or {}
        scripts = sorted({unicode_script(cp) for cp in cmap} - {"Zyyy", "Zinh", "Zzzz"})
        os2 = font.get("OS/2")
        fs_type = os2.fsType if os2 else 0
        if fs_type != 0:
            raise ValueError(f"Bundled font has nonzero embedding restrictions: {path}")
        italic = bool((os2.fsSelection & 1) if os2 else font["head"].macStyle & 2)
        weight = int(os2.usWeightClass) if os2 else 400
        weight_axis = next((a for a in axes if a["tag"] == "wght"), None)
        return {**asset, "format": "opentype" if path.suffix == ".otf" else "truetype",
                "familyName": names.getBestFamilyName(), "subfamilyName": names.getBestSubFamilyName(),
                "postscriptName": names.getDebugName(6), "style": "italic" if italic else "normal",
                "weight": weight, "weightRange": [int(weight_axis["min"]), int(weight_axis["max"])] if weight_axis else [weight, weight],
                "axes": axes, "scripts": scripts, "glyphCount": font["maxp"].numGlyphs,
                "characterCount": len(cmap), "embeddingFsType": fs_type,
                "subsetAllowed": not bool(fs_type & 0x0100)}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", type=pathlib.Path, default=ROOT.parent / "collections/catalog.lock.json")
    parser.add_argument("--output", type=pathlib.Path, default=ROOT.parent.parent / ".cache/web-fonts/fonts")
    parser.add_argument("--catalog", type=pathlib.Path, default=ROOT.parent.parent / ".cache/web-fonts/catalog.json")
    parser.add_argument("--verify-only", action="store_true")
    parser.add_argument("--workers", type=int, default=12)
    args = parser.parse_args()
    manifest = json.loads(args.manifest.read_text())
    jobs = [(family["id"], file) for family in manifest["families"] for file in [family["license"], *family["files"]]]
    results = {}
    with concurrent.futures.ThreadPoolExecutor(max_workers=args.workers) as executor:
        futures = {executor.submit(acquire, file, fid, manifest, args.output, args.verify_only): (fid, file["path"]) for fid, file in jobs}
        for index, future in enumerate(concurrent.futures.as_completed(futures), 1):
            results[futures[future]] = future.result()
            if index % 25 == 0:
                print(f"Verified {index}/{len(jobs)} assets", flush=True)
    families = []
    for family in manifest["families"]:
        license_path, license_asset = results[(family["id"], family["license"]["path"])]
        license_text = license_path.read_text(encoding="utf-8-sig")
        if "SIL OPEN FONT LICENSE" not in license_text.upper() or "1.1" not in license_text:
            raise ValueError(f"Expected SIL OFL 1.1 license: {license_path}")
        faces = [inspect_font(*results[(family["id"], face["path"])]) for face in family["files"]]
        # Most variable families use the typographic family name. Preserve all
        # original face names so consumers can alias them without changing bytes.
        name = faces[0]["familyName"]
        families.append({"id": family["id"], "name": name, "category": family["category"],
                         "scripts": sorted({script for face in faces for script in face["scripts"]}),
                         "license": "OFL-1.1", "licensePath": license_asset["path"],
                         "licenseMetadata": {"spdx": "OFL-1.1", **license_asset}, "faces": faces})
    catalog = {"schemaVersion": 1, "source": {"repository": manifest["repository"], "commit": manifest["commit"]},
               "assetBasePath": "/", "families": families}
    args.catalog.parent.mkdir(parents=True, exist_ok=True)
    args.catalog.write_text(json.dumps(catalog, ensure_ascii=False, indent=2) + "\n")
    print(f"Catalog written: {len(families)} families, {sum(len(f['faces']) for f in families)} faces, "
          f"{sum(face['size'] for f in families for face in f['faces']) / 1e6:.1f} MB")


if __name__ == "__main__":
    main()
