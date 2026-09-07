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
