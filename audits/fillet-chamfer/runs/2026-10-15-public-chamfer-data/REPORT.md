# Public chamfer fixture native execution

279 unchanged dedicated native chamfer cases: 239 passed, 22 known failures,
18 missing-data skips. No unexpected failure, crash or timeout. These are native
harness outcomes, not Truck results or shape-equivalence claims.

32 of the 36 required filenames were downloaded from the public dataset
https://github.com/a-betenev/opencascade-dataset at pinned revision
 d78a7a9e6de57699261fa03641ec1b5bafac39a1. All 535523 bytes were checked against
Git blob hashes and sizes; SHA256 and exact source URLs are in data-provenance.json.
Raw fixture bytes are inside artifacts.zip alongside all native logs.
The four absent files still skip 18 cases: CCH_001_ahev.rle, CCH_001_ahew.rle,
CCH_001_chde.rle and CFI_pro10129.rle. The default OCCT data directory was preserved;
the external directory was explicitly added and hashes verified after execution.

Archive: 871 byte-verified members.
SHA256 aacae63b5f92afb70e5fd38e29b4cb56ba9c15f67075f9bcfac635c01f6db0e6.
Command: python3 audits/fillet-chamfer/run_occt.py --occt ../investigations/sequential-holes-2026-10-10/occt --library-path target/occt-sdk/usr/lib64 --data-path target/occt-public-chamfer-data --include-missing --case 'chamfer/*' --workers 4 --timeout 120 --output target/native-chamfer-public-data

Next: physically capture and replay the 239 passing native entries. Known failures
are retained separately; absent files remain missing. Static missing-in-checkout
labels remain accurate because external fixtures were not copied into OCCT/data.
