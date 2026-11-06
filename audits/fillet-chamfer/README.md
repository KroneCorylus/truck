# Fillet and chamfer audit catalog

`build_catalog.py` generates `catalog.json` and the standalone `catalog.html`
viewer. Both are local build products and are ignored by Git.

```sh
python3 audits/fillet-chamfer/build_catalog.py --occt /path/to/OCCT --app ../PiezaCAD
python3 -B -m unittest discover -s audits/fillet-chamfer -p 'test_*.py'
```

The generator validates original source and artifact hashes before indexing runs.
The schema-2 catalog retains every case and execution outcome, including failed,
incomplete and unverified results. It links to the full execution manifests for
physical tasks, artifact inventories, source hashes and detailed measurements.
Newly generated manifest references also carry their SHA-256 digest.

The HTML embeds only this compact index for offline use. Keep it alongside the
linked run manifests and archives when sharing an audit. Construction success
and sampled agreement do not establish full geometric parity.

Keep the generator, viewer template, tests and required source fixtures in Git.
Do not force-add generated catalogs: preserving their repeated embedded evidence
in every revision makes both the files and repository history unnecessarily large.
Historical evidence manifests and archives are not modified by compaction.
