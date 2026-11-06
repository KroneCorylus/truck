# Supported ellipse writer candidate

`ellipse-reviewed-glibc` is default-off. It is an artifact build option, not
runtime certification. Only a final reviewed x86_64 GNU/Linux artifact selected
by the external supported-artifact launcher may use it. Other targets decline.
No build/installation/launcher/default changes are approved by this patch.

Writer guarantee: bounded serialized curve/support agreement, under reviewed
scalar glibc 2^-48 on [-8,8], faithful arithmetic, mm/radian placements, exact
built-in Point3 / modeling Curve / modeling Surface formatters, and the existing
combined <=5e-8 ledger. Every actual descriptor/endpoint/ledger computation runs
inside the eligible construction path. Feature-off bypasses those computations.
Custom types (including wrappers) decline metadata optimization and keep their
ordinary formatting behavior; returning metadata alone does not admit a type.

StepShell owns pcurves/trim/indices and immutably borrows the concrete source.
StepLength never changes. Ineligible construction declines before allocation.
Failed post-construction eligibility marks the object unpublishable; it does not
rewrite counts. Formatting checks current-thread controls, renders to a private
String using built-ins, checks controls again, then copies bytes to the caller.
It does not require thread affinity: eligible new threads are valid, incompatible
controls reject. No new user-input panic: app uses fmt::write and propagates Err
before fs::write, for both single-solid and assembly export. Arbitrary callers of
Display must use fallible formatting if they require rejection without to_string
panic. Partial output at a larger containing formatter must always be discarded
on Err; a shell buffer is not a transactional entire-document writer.

Arbitrary sink callbacks run only after the shell's numerical work. Their control
changes are caught at the next certified shell. No numerical work is performed on
already serialized shell bytes. Built-in geometry has no interior-mutable custom
callbacks; exact TypeId checks exclude those types before metadata extraction.
Allocation/foreign/signal hooks still require the final artifact's continuous
control-preservation audit: entry/exit reads alone cannot detect transient
change-and-restore inside such a hook. The candidate does not claim they can.

Guarded OCCT-reader guarantee is separate downstream release QA: reuse the exact
post-transfer binding guard and exclusively owned fresh caches from
ellipse-guard-validation-20261105. Recheck after synchronous trusted QA before
publishing results. No OCCT runtime dependency is added to PiezaCAD; arbitrary
readers, default healing and arbitrary callbacks are not certified.

Breaking: StepModel From<&CompressedShell>, From<&CompressedSolid>, and
FromIterator<&CompressedShell> require P: Copy + DisplayByStep + 'static,
C: 'static and S: 'static in addition to existing bounds. Add those bounds to
generic callers; use owned concrete geometry rather than borrowed formatter
wrappers. Shell/solid borrow lifetimes themselves are unchanged. Coordinator must
copy this Breaking line into CHANGELOG.md under Unreleased when applying.

Inspected app production uses: kernel/mod.rs step_file and kernel/assembly.rs
assembly_step; both prepare owned modeling solids and keep them alive through
StepDesign/StepDisplay. Both now buffer via fmt::write before filesystem write.
Tests/healing.rs concrete modeling usages need no type migration. Filesystem I/O
atomicity across an export set is unchanged and outside this numerical contract.
