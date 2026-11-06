//! Local copy of the reviewed scalar fenv boundary; no cross-crate source linking.
#![allow(unsafe_code)] // Confined to this foreign-library boundary.
#[derive(Clone, Copy)]
#[repr(C, align(4))]
struct Environment([u32; 8]);

#[link(name = "m")]
extern "C" {
    fn fegetenv(environment: *mut Environment) -> std::ffi::c_int;
}

pub(super) fn eligible() -> bool {
    let mut environment = Environment([0; 8]);
    // SAFETY: pinned x86_64 glibc fenv_t occupies 32 bytes with alignment 4.
    // fegetenv writes only that buffer, retaining no pointer. It preserves FP controls.
    if unsafe { fegetenv(&mut environment) } != 0 {
        return false;
    }
    controls_eligible(environment.0[0] as u16, environment.0[7])
}

fn controls_eligible(x87: u16, mxcsr: u32) -> bool {
    // RN, masked exceptions and extended x87 precision; RN + gradual SSE arithmetic.
    // Sticky status flags do not affect eligibility.
    x87 & 0x0f3f == 0x033f && mxcsr & 0xffc0 == 0x1f80
}

#[cfg(test)]
mod tests {
    use super::controls_eligible;
    #[test]
    fn rejects_every_incompatible_control_bit() {
        assert!(controls_eligible(0x037f, 0x1f80));
        assert!(controls_eligible(0x037f, 0x1fbf));
        for bit in [6, 13, 14, 15] {
            assert!(!controls_eligible(0x037f, 0x1f80 | (1 << bit)));
        }
        for bit in 7..13 {
            assert!(!controls_eligible(0x037f, 0x1f80 & !(1 << bit)));
        }
        for rc in [0x0400, 0x0800, 0x0c00] {
            assert!(!controls_eligible(0x037f | rc, 0x1f80));
        }
        for bit in 0..6 {
            assert!(!controls_eligible(0x037f & !(1 << bit), 0x1f80));
        }
        for precision in [0, 0x0100, 0x0200] {
            assert!(!controls_eligible(0x007f | precision, 0x1f80));
        }
    }
}

#[cfg(test)]
mod environment_tests {
    use super::*;
    #[link(name = "m")]
    extern "C" {
        fn fesetenv(environment: *const Environment) -> std::ffi::c_int;
    }
    fn read() -> Environment {
        let mut env = Environment([0; 8]);
        // SAFETY: same pinned ABI and buffer as the production reader.
        assert_eq!(unsafe { fegetenv(&mut env) }, 0);
        env
    }
    struct Restore(Environment);
    impl Drop for Restore {
        fn drop(&mut self) {
            // SAFETY: restores an environment saved on this thread.
            assert_eq!(unsafe { fesetenv(&self.0) }, 0);
        }
    }
    #[test]
    fn independent_thread_controls_are_read_on_every_call() {
        let mut controls = vec![(0x037fu32, 0x1f80u32, true)];
        for rc in 1..4 {
            controls.push((0x037f | (rc << 10), 0x1f80, false));
            controls.push((0x037f, 0x1f80 | (rc << 13), false));
        }
        for precision in [0, 0x100, 0x200] {
            controls.push((0x007f | precision, 0x1f80, false));
        }
        for denormal in [0x40, 0x8000, 0x8040] {
            controls.push((0x037f, 0x1f80 | denormal, false));
        }
        for bit in 0..6 {
            controls.push((0x037f & !(1 << bit), 0x1f80, false));
            controls.push((0x037f, 0x1f80 & !(1 << (bit + 7)), false));
        }
        for (x87, sse, expected) in controls {
            std::thread::spawn(move || {
                let old = read();
                let restore = Restore(old);
                let mut changed = old;
                changed.0[0] = (changed.0[0] & 0xffff0000) | x87;
                // Clear pending x87 flags and summary before unmasking; SSE flags
                // are cleared by assigning the complete MXCSR control word.
                changed.0[1] &= 0xffff0000;
                changed.0[7] = sse;
                // SAFETY: valid saved environment with independently selected controls.
                assert_eq!(unsafe { fesetenv(&changed) }, 0);
                // Only integer predicates and foreign environment reads in rejected states.
                let accepted = eligible();
                let observed = read();
                drop(restore);
                assert_eq!(accepted, expected);
                assert_eq!(observed.0[0], changed.0[0]);
                assert_eq!(observed.0[7], changed.0[7]);
                assert_eq!(eligible(), controls_eligible(old.0[0] as u16, old.0[7]));
            })
            .join()
            .unwrap();
        }
    }
}

#[cfg(test)]
pub(in crate::out) struct DownwardGuard(Environment);
#[cfg(test)]
impl DownwardGuard {
    pub(in crate::out) fn enter() -> Self {
        #[link(name = "m")]
        extern "C" {
            fn fesetenv(environment: *const Environment) -> std::ffi::c_int;
        }
        let mut old = Environment([0; 8]);
        // SAFETY: the pinned fenv_t ABI and a valid output buffer.
        assert_eq!(unsafe { fegetenv(&mut old) }, 0);
        let guard = Self(old);
        let mut changed = old;
        changed.0[0] = (changed.0[0] & !0xc00) | 0x400;
        changed.0[7] = (changed.0[7] & !0x6000) | 0x2000;
        // SAFETY: only changes the documented rounding controls on this thread.
        assert_eq!(unsafe { fesetenv(&changed) }, 0);
        guard
    }
}
#[cfg(test)]
impl Drop for DownwardGuard {
    fn drop(&mut self) {
        #[link(name = "m")]
        extern "C" {
            fn fesetenv(environment: *const Environment) -> std::ffi::c_int;
        }
        // SAFETY: restores the saved valid environment on the same test thread.
        assert_eq!(unsafe { fesetenv(&self.0) }, 0);
    }
}
#[cfg(test)]
pub(in crate::out) fn with_downward<T>(run: impl FnOnce() -> T) -> T {
    let _restore = DownwardGuard::enter();
    run()
}
