//! Wall time per stage of the set operations, accumulated for the benchmark in
//! `tests/bench_plate.rs`. Not part of the public API.

use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Wall time per stage of the set operations run since the last [`take`], and the number of
/// face pairs the bounding-box scan looked at and let through.
#[derive(Clone, Copy, Debug, Default)]
pub struct StageTimes {
    /// Triangulation of both shells.
    pub triangulation: Duration,
    /// The bounding-box scan over all face pairs, without the work on the surviving pairs.
    pub pairing: Duration,
    /// Mesh interference, coincidence and intersection curves of the surviving pairs, and their
    /// insertion into the loops stores.
    pub interference: Duration,
    /// Division of the faces along the loops.
    pub division: Duration,
    /// Ray casting of the pieces the cuts left undecided.
    pub classification: Duration,
    /// Fitting of the intersection curves of the result.
    pub fitting: Duration,
    /// Face pairs the scan looked at.
    pub pairs_scanned: usize,
    /// Face pairs whose bounding boxes overlap.
    pub pairs_overlapping: usize,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum Stage {
    Triangulation,
    /// The whole loops-store construction, interference included.
    LoopsStore,
    Interference,
    Division,
    Classification,
    Fitting,
}

static TIMES: Mutex<([Duration; 6], [usize; 2])> = Mutex::new(([Duration::ZERO; 6], [0; 2]));

/// The current time, or `None` where the platform has no clock.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn now() -> Option<Instant> { Some(Instant::now()) }
#[cfg(target_arch = "wasm32")]
pub(crate) fn now() -> Option<Instant> { None }

/// Adds the time since `start` to `stage`.
pub(crate) fn lap(stage: Stage, start: Option<Instant>) {
    if let Some(start) = start {
        TIMES.lock().unwrap().0[stage as usize] += start.elapsed();
    }
}

/// Adds to the counts of face pairs scanned and overlapping.
pub(crate) fn pairs(scanned: usize, overlapping: usize) {
    let mut times = TIMES.lock().unwrap();
    times.1[0] += scanned;
    times.1[1] += overlapping;
}

/// Returns the accumulated times and counts, and resets them.
pub fn take() -> StageTimes {
    let (t, [pairs_scanned, pairs_overlapping]) = std::mem::take(&mut *TIMES.lock().unwrap());
    StageTimes {
        triangulation: t[Stage::Triangulation as usize],
        pairing: t[Stage::LoopsStore as usize].saturating_sub(t[Stage::Interference as usize]),
        interference: t[Stage::Interference as usize],
        division: t[Stage::Division as usize],
        classification: t[Stage::Classification as usize],
        fitting: t[Stage::Fitting as usize],
        pairs_scanned,
        pairs_overlapping,
    }
}
