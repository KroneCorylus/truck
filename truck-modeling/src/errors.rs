use thiserror::Error;

/// Modeling errors
#[derive(Debug, PartialEq, Eq, Error)]
pub enum Error {
    /// wrapper of topological error
    #[error(transparent)]
    FromTopology(#[from] truck_topology::errors::Error),
    /// tried to attach a plane to a wire that was not on one plane.
    /// cf. [`builder::try_attach_plane`](../builder/fn.try_attach_plane.html)
    #[error("cannot attach a plane to a wire that is not on one plane.")]
    WireNotInOnePlane,
    /// tried to create homotopy for two wires with different numbers of edges.
    /// cf. [`builder::try_wire_homotopy`](../builder/fn.try_wire_homotopy.html)
    #[error("The wires must contain the same number of edges to create a homotopy.")]
    NotSameNumberOfEdges,
    /// a loft needs at least two sections.
    /// cf. [`builder::try_loft_shell`](../builder/fn.try_loft_shell.html)
    #[error("a loft needs at least two sections.")]
    TooFewLoftSections,
    /// the sections of a loft must have the same number of edges.
    /// cf. [`builder::try_loft_shell`](../builder/fn.try_loft_shell.html)
    #[error("sections {0} and {1} of the loft have different numbers of edges.")]
    LoftSectionsMismatch(usize, usize),
    /// two sections of a loft are the same wire, or lie on each other.
    /// cf. [`builder::try_loft_shell`](../builder/fn.try_loft_shell.html)
    #[error("sections {0} and {1} of the loft coincide; closed lofts and repeated sections are not supported.")]
    LoftSectionsCoincide(usize, usize),
    /// a curve that cannot be lofted: an intersection curve, which has no exact NURBS form, or a
    /// NURBS curve whose end weights differ.
    #[error("the curve has no NURBS form usable for lofting: an intersection curve, or unequal weights at its ends.")]
    NoNurbsForm,
    /// an edge of a sweep path that is neither a line nor a circular arc.
    /// cf. [`builder::sweep_along_wire`](../builder/fn.sweep_along_wire.html)
    #[error("edge {0} of the path is neither a line nor a circular arc.")]
    PathEdgeNotSweepable(usize),
    /// a sweep path with a gap or a corner at a vertex.
    /// cf. [`builder::sweep_along_wire`](../builder/fn.sweep_along_wire.html)
    #[error("the path is not continuous and tangent continuous at vertex {0}.")]
    PathNotSmooth(usize),
    /// an arc of a sweep path whose axis the profile reaches, so the sweep would fold over.
    /// cf. [`builder::sweep_along_wire`](../builder/fn.sweep_along_wire.html)
    #[error("the profile reaches the axis of arc {0} of the path.")]
    PathTooTight(usize),
    /// a closed sweep path around which the profile does not return to where it started.
    /// cf. [`builder::sweep_along_wire`](../builder/fn.sweep_along_wire.html)
    #[error("the profile does not return to its start around the closed path; only planar closed paths are supported.")]
    ClosedPathNotPlanar,
}

#[test]
fn print_messages() {
    use std::io::Write;
    writeln!(
        &mut std::io::stderr(),
        "****** test of the expressions of error messages ******\n"
    )
    .unwrap();
    writeln!(
        &mut std::io::stderr(),
        "{}\n",
        Error::FromTopology(truck_topology::errors::Error::SameVertex)
    )
    .unwrap();
    writeln!(&mut std::io::stderr(), "{}\n", Error::WireNotInOnePlane).unwrap();
    writeln!(
        &mut std::io::stderr(),
        "*******************************************************"
    )
    .unwrap();
}
