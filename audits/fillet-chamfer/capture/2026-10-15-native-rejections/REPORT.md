# Full capture with native rejection replay

Clean revision 5412b62723e49cf50b8133daae26fdae5cefdf7b: 157 passing native entries, 162 captures.
106 valid constructions and 97 limited comparison passes remain exactly unchanged.
The one former incomplete capture, bugs/moddata_1/bug119, now records a native
error return and an immutable Truck OutsideNeighbour rejection. It is one of
54 operation-failed records: the 53 prior construction failures plus this native
rejection case. Two closed-shell imports remain unsupported. Zero incomplete,
unhandled, panic or timeout records. No new construction or parity claim.

All 106 successful result/comparison records are exactly unchanged. Archive:
2245 byte-verified files, SHA256 df40561a33bc3dc3a3de61401311f64a11e65b644e1fb350e2f2d5d279079b8c.
Source/executable hashes checked before and after. Catalog verification, 19 audit
and eight capture tests pass. Rust, native plugin and geometry sources unchanged;
no new Rust build or kernel test run was needed for the Python-only change.
