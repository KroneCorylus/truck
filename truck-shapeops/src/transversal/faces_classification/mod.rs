use super::loops_store::ShapesOpStatus;
use rustc_hash::FxHashMap as HashMap;
use truck_topology::*;

#[derive(Clone, Debug)]
pub struct FacesClassification<P, C, S> {
    shell: Shell<P, C, S>,
    status: HashMap<FaceID<S>, ShapesOpStatus>,
}

impl<P, C, S> Default for FacesClassification<P, C, S> {
    fn default() -> Self {
        Self {
            shell: Default::default(),
            status: HashMap::default(),
        }
    }
}

impl<P, C, S> FacesClassification<P, C, S> {
    pub fn push(&mut self, face: Face<P, C, S>, status: ShapesOpStatus) {
        self.status.insert(face.id(), status);
        self.shell.push(face);
    }

    fn faces_where(&self, keep: impl Fn(ShapesOpStatus) -> bool) -> Shell<P, C, S> {
        self.shell
            .iter()
            .filter(|face| keep(self.status[&face.id()]))
            .cloned()
            .collect()
    }

    /// The faces kept by AND, the faces kept by OR, and the faces still undecided.
    pub fn and_or_unknown(&self) -> [Shell<P, C, S>; 3] {
        use ShapesOpStatus::*;
        [
            self.faces_where(|status| matches!(status, And | Both)),
            self.faces_where(|status| matches!(status, Or | Both)),
            self.faces_where(|status| status == Unknown),
        ]
    }

    pub fn excludes_first_material(&self) -> bool {
        self.status
            .values()
            .any(|status| matches!(status, ShapesOpStatus::Or | ShapesOpStatus::Neither))
    }

    /// Gives each connected component of undecided faces the status of a decided face it
    /// shares an edge with. Overlaps of coincident faces decide nothing about their neighbours.
    pub fn integrate_by_component(&mut self) {
        let and_boundary = self
            .faces_where(|status| status == ShapesOpStatus::And)
            .extract_boundaries();
        let or_boundary = self
            .faces_where(|status| status == ShapesOpStatus::Or)
            .extract_boundaries();
        let unknown = self.faces_where(|status| status == ShapesOpStatus::Unknown);
        let components = unknown.connected_components();
        for comp in components {
            let boundary = comp.extract_boundaries();
            let Some(first) = boundary.iter().flatten().next() else {
                continue;
            };
            if and_boundary
                .iter()
                .flatten()
                .any(|edge| edge.id() == first.id())
            {
                comp.iter().for_each(|face| {
                    *self.status.get_mut(&face.id()).unwrap() = ShapesOpStatus::And;
                })
            } else if or_boundary
                .iter()
                .flatten()
                .any(|edge| edge.id() == first.id())
            {
                comp.iter().for_each(|face| {
                    *self.status.get_mut(&face.id()).unwrap() = ShapesOpStatus::Or;
                })
            }
        }
    }
}

#[cfg(test)]
mod tests;
