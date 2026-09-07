use super::*;
use crate::common::PartAttrs;
use ruststep::error::Error::UnknownEntity;
use std::fmt;

/// A face, edge or vertex that could not be converted and was left out of its shell.
#[derive(Debug)]
pub struct Skipped {
    /// the id of the STEP entity, 0 when it was written inline instead of numbered
    pub id: u64,
    /// its entity type: `VERTEX_POINT`, `EDGE_CURVE` or `FACE_SURFACE`
    pub entity: &'static str,
    pub reason: StepConvertingError,
}

impl fmt::Display for Skipped {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{} {}: {}", self.id, self.entity, self.reason)
    }
}

/// The result of a conversion, with what was left out of it.
pub type Converted<T> = Result<(T, Vec<Skipped>), StepConvertingError>;

fn ref_id<T>(place_holder: &PlaceHolder<T>) -> u64 {
    match place_holder {
        PlaceHolder::Ref(Name::Entity(idx)) => *idx,
        _ => 0,
    }
}

impl Table {
    /// Turns a failed lookup into a reason naming the unimplemented or unreadable entity behind it.
    fn reason(&self, e: ruststep::error::Error) -> StepConvertingError {
        if let UnknownEntity(idx) = e {
            if let Some(dummy) = self.dummy.get(&idx) {
                return format!("#{idx} is {}, which is not implemented", dummy.name).into();
            }
            if let Some((_, message)) = self.errors.iter().find(|(id, _)| *id == idx) {
                return format!("#{idx} could not be read: {message}").into();
            }
        }
        e.into()
    }

    fn place_holder_edge_any_to_index_and_edge_curve(
        &self,
        edge: &PlaceHolder<EdgeAnyHolder>,
    ) -> Option<(u64, EdgeCurveHolder)> {
        use PlaceHolder::Ref;
        let Ref(Name::Entity(ref idx)) = edge else {
            return None;
        };
        self.oriented_edge
            .get(idx)
            .and_then(|oriented_edge| {
                Some((
                    oriented_edge.edge_element_idx()?,
                    oriented_edge.edge_element_holder(self)?,
                ))
            })
            .or_else(|| {
                self.edge_curve
                    .get(idx)
                    .map(|edge_curve| (*idx, edge_curve.clone()))
            })
    }
    fn face_any_to_orientation_and_face(
        &self,
        face: Option<FaceAnyHolder>,
    ) -> Option<(bool, FaceSurfaceHolder)> {
        match face? {
            FaceAnyHolder::FaceSurface(face) => Some((true, face)),
            FaceAnyHolder::OrientedFace(oriented_face) => {
                let face_element = oriented_face.face_element_holder(self)?;
                Some((oriented_face.orientation, face_element))
            }
        }
    }

    /// The edge curves reachable from the loops of `shell`, with repetition.
    fn shell_edge_curves(&self, shell: &ShellHolder) -> Vec<(u64, EdgeCurveHolder)> {
        shell
            .cfs_faces_holder(self)
            .filter_map(move |face| self.face_any_to_orientation_and_face(face))
            .flat_map(move |(_, face)| face.bounds_holder(self))
            .filter_map(move |bound| bound?.bound_holder(self))
            .flat_map(move |bound| bound.edge_list)
            .filter_map(move |edge| self.place_holder_edge_any_to_index_and_edge_curve(&edge))
            .collect()
    }

    fn shell_vertices(
        &self,
        shell: &ShellHolder,
        skipped: &mut Vec<Skipped>,
    ) -> (Vec<Point3>, HashMap<u64, usize>) {
        let mut vertices = Vec::new();
        let mut vidx_map = HashMap::<u64, usize>::new();
        for (_, edge) in self.shell_edge_curves(shell) {
            for vertex in [edge.edge_start, edge.edge_end] {
                let idx = ref_id(&vertex);
                if vidx_map.contains_key(&idx) || skipped.iter().any(|s| s.id == idx) {
                    continue;
                }
                match EntityTable::<VertexPointHolder>::get_owned(self, idx) {
                    Ok(p) => {
                        vidx_map.insert(idx, vertices.len());
                        vertices.push(Point3::from(&p.vertex_geometry));
                    }
                    Err(e) => skipped.push(Skipped {
                        id: idx,
                        entity: "VERTEX_POINT",
                        reason: self.reason(e),
                    }),
                }
            }
        }
        (vertices, vidx_map)
    }

    fn compressed_edge(
        &self,
        edge: &EdgeCurveHolder,
        vidx_map: &HashMap<u64, usize>,
    ) -> Result<CompressedEdge<Curve3D>, StepConvertingError> {
        let vertex = |v: &PlaceHolder<VertexPointHolder>| {
            let idx = ref_id(v);
            vidx_map
                .get(&idx)
                .copied()
                .ok_or_else(|| format!("vertex #{idx} was skipped"))
        };
        let vertices = (vertex(&edge.edge_start)?, vertex(&edge.edge_end)?);
        let curve = self.edge_curve_owned(edge)?.parse_curve3d()?;
        Ok(CompressedEdge { vertices, curve })
    }

    /// The owned edge curve. A surface curve keeps only its 3D curve and, when it resolves, its
    /// master pcurve, so that an unsupported neighbouring surface does not take the edge with it.
    fn edge_curve_owned(&self, edge: &EdgeCurveHolder) -> Result<EdgeCurve, StepConvertingError> {
        use PreferredSurfaceCurveRepresentation::*;
        let mut edge = edge.clone();
        if let Some(surface_curve) = self.surface_curve.get(&ref_id(&edge.edge_geometry)) {
            let mut surface_curve = surface_curve.clone();
            let master = match surface_curve.master_representation {
                Curve3D => None,
                PcurveS1 => surface_curve.associated_geometry.first(),
                PcurveS2 => surface_curve.associated_geometry.get(1),
            };
            let master = master
                .filter(|geometry| (*geometry).clone().into_owned(self).is_ok())
                .cloned();
            surface_curve.master_representation = match master {
                Some(_) => PcurveS1,
                None => Curve3D,
            };
            surface_curve.associated_geometry = master.into_iter().collect();
            edge.edge_geometry =
                PlaceHolder::Owned(CurveAnyHolder::SurfaceCurve(Box::new(surface_curve)));
        }
        edge.into_owned(self).map_err(|e| self.reason(e))
    }

    fn shell_edges(
        &self,
        shell: &ShellHolder,
        vidx_map: &HashMap<u64, usize>,
        skipped: &mut Vec<Skipped>,
    ) -> (Vec<CompressedEdge<Curve3D>>, HashMap<u64, usize>) {
        let mut edges = Vec::new();
        let mut eidx_map = HashMap::<u64, usize>::new();
        for (idx, edge) in self.shell_edge_curves(shell) {
            if eidx_map.contains_key(&idx) || skipped.iter().any(|s| s.id == idx) {
                continue;
            }
            match self.compressed_edge(&edge, vidx_map) {
                Ok(edge) => {
                    eidx_map.insert(idx, edges.len());
                    edges.push(edge);
                }
                Err(reason) => skipped.push(Skipped {
                    id: idx,
                    entity: "EDGE_CURVE",
                    reason,
                }),
            }
        }
        (edges, eidx_map)
    }

    /// `None` for a `VERTEX_LOOP`, which bounds a degenerate face with no edges.
    fn face_bound_to_edges(
        &self,
        bound: FaceBoundHolder,
        eidx_map: &HashMap<u64, usize>,
    ) -> Result<Option<Vec<CompressedEdgeIndex>>, StepConvertingError> {
        let ori = bound.orientation;
        let Some(edge_loop) = bound.bound_holder(self) else {
            let idx = ref_id(&bound.bound);
            return match self.dummy.get(&idx) {
                Some(dummy) if dummy.name == "VERTEX_LOOP" => Ok(None),
                _ => Err(self.reason(UnknownEntity(idx))),
            };
        };
        let mut edges = Vec::new();
        for edge in edge_loop.edge_list {
            let idx = ref_id(&edge);
            let (edge_idx, orientation) = match self.oriented_edge.get(&idx) {
                Some(oriented_edge) => (
                    oriented_edge
                        .edge_element_idx()
                        .ok_or("an oriented edge does not reference its edge")?,
                    oriented_edge.orientation == ori,
                ),
                None => (idx, ori),
            };
            let Some(&index) = eidx_map.get(&edge_idx) else {
                return Err(match self.edge_curve.contains_key(&edge_idx) {
                    true => format!("edge #{edge_idx} was skipped").into(),
                    false => self.reason(UnknownEntity(edge_idx)),
                });
            };
            edges.push(CompressedEdgeIndex { index, orientation });
        }
        if !ori {
            edges.reverse();
        }
        Ok(Some(edges))
    }

    fn compressed_face(
        &self,
        id: u64,
        face: Option<FaceAnyHolder>,
        eidx_map: &HashMap<u64, usize>,
    ) -> Result<CompressedFace<Surface>, StepConvertingError> {
        let (orientation, face) = self
            .face_any_to_orientation_and_face(face)
            .ok_or_else(|| self.reason(UnknownEntity(id)))?;
        let step_surface: SurfaceAny = face
            .face_geometry
            .clone()
            .into_owned(self)
            .map_err(|e| self.reason(e))?;
        let mut surface = Surface::try_from(&step_surface)?;
        if !face.same_sense {
            surface.invert()
        }
        let mut boundaries = Vec::new();
        for (place_holder, bound) in face.bounds.iter().zip(face.bounds_holder(self)) {
            let bound = bound.ok_or_else(|| self.reason(UnknownEntity(ref_id(place_holder))))?;
            if let Some(edges) = self.face_bound_to_edges(bound, eidx_map)? {
                boundaries.push(edges);
            }
        }
        Ok(CompressedFace {
            surface,
            boundaries,
            orientation,
        })
    }

    fn shell_faces(
        &self,
        shell: &ShellHolder,
        eidx_map: &HashMap<u64, usize>,
        skipped: &mut Vec<Skipped>,
    ) -> Vec<CompressedFace<Surface>> {
        let mut faces = Vec::new();
        for (place_holder, face) in shell.cfs_faces.iter().zip(shell.cfs_faces_holder(self)) {
            let id = ref_id(place_holder);
            match self.compressed_face(id, face, eidx_map) {
                Ok(face) => faces.push(face),
                Err(reason) => skipped.push(Skipped {
                    id,
                    entity: "FACE_SURFACE",
                    reason,
                }),
            }
        }
        faces
    }

    /// Constructs `CompressedShell` of `truck` from `Shell` in STEP file, with the faces, edges
    /// and vertices that could not be converted. A face is left out with any edge of its loops.
    /// # Example
    /// ```
    /// use truck_stepio::r#in::{*, step_geometry::*};
    /// // read file
    /// let step_string = include_str!(concat!(
    ///     env!("CARGO_MANIFEST_DIR"),
    ///     "/../resources/step/occt-cube.step",
    /// ));
    /// // parse into Rust structs
    /// let table = Table::from_step(&step_string).unwrap();
    /// // take one shell (this is only one shell)
    /// let step_shell = table.shell.values().next().unwrap();
    /// // convert STEP shell to `CompressedShell`
    /// let (cshell, skipped) = table.to_compressed_shell(step_shell).unwrap();
    /// // The cube has 6 faces!
    /// assert_eq!(cshell.faces.len(), 6);
    /// assert!(skipped.is_empty());
    /// ```
    pub fn to_compressed_shell(
        &self,
        shell: &impl StepShell,
    ) -> Converted<CompressedShell<Point3, Curve3D, Surface>> {
        shell.to_compressed_shell(self)
    }

    /// Constructs `CompressedShell`s of `truck` from `ShellBasedSurfaceModel` in STEP file
    pub fn to_compressed_shells(
        &self,
        shells: &ShellBasedSurfaceModelHolder,
    ) -> Converted<Vec<CompressedShell<Point3, Curve3D, Surface>>> {
        let mut res = Vec::new();
        let mut skipped = Vec::new();
        for place_holder in &shells.sbsm_boundary {
            let PlaceHolder::Ref(Name::Entity(idx)) = place_holder else {
                return Err("failed to reference an element of `sbsm_boundary`".into());
            };
            let (shell, more) = if let Some(shell) = self.shell.get(idx) {
                self.to_compressed_shell(shell)?
            } else if let Some(oriented_shell) = self.oriented_shell.get(idx) {
                self.to_compressed_shell(oriented_shell)?
            } else {
                return Err("failed to reference an element of `sbsm_boundary`".into());
            };
            res.push(shell);
            skipped.extend(more);
        }
        Ok((res, skipped))
    }

    /// Constructs `CompressedSolid` of `truck` from `ManifoldSolidBrep` in STEP file
    /// # Example
    /// ```
    /// use truck_stepio::r#in::{*, step_geometry::*};
    /// truck_topology::prelude!(Point3, Curve3D, Surface);
    /// // read file
    /// let step_string = include_str!(concat!(
    ///     env!("CARGO_MANIFEST_DIR"),
    ///     "/../resources/step/occt-cube.step",
    /// ));
    /// // parse into Rust structs
    /// let table = Table::from_step(&step_string).unwrap();
    /// // take the solid
    /// let step_solid = table.manifold_solid_brep.values().next().unwrap();
    /// // convert STEP shell to `CompressedSolid`
    /// let (csolid, skipped) = table.to_compressed_solid(step_solid).unwrap();
    /// assert!(skipped.is_empty());
    /// // Convert to truck `Solid`
    /// let solid = Solid::extract(csolid).unwrap();
    /// // The cube has 6 faces!
    /// assert_eq!(solid.boundaries()[0].len(), 6);
    /// ```
    pub fn to_compressed_solid(
        &self,
        solid: &ManifoldSolidBrepHolder,
    ) -> Converted<CompressedSolid<Point3, Curve3D, Surface>> {
        let PlaceHolder::Ref(Name::Entity(outer_idx)) = &solid.outer else {
            return Err("failed to reference `solid.outer`".into());
        };
        let (outer_shell, mut skipped) = if let Some(step_shell) = self.shell.get(outer_idx) {
            self.to_compressed_shell(step_shell)
        } else if let Some(step_shell) = self.oriented_shell.get(outer_idx) {
            self.to_compressed_shell(step_shell)
        } else {
            Err("failed to reference `solid.outer`".into())
        }?;
        let mut boundaries = vec![outer_shell];
        for shell in &solid.voids {
            let PlaceHolder::Ref(Name::Entity(outer_idx)) = shell else {
                return Err("failed to reference an element of `solid.voids`".into());
            };
            let Some(oriented_shell) = self.oriented_shell.get(outer_idx) else {
                return Err("failed to reference an element of `solid.voids`".into());
            };
            let (shell, more) = self.to_compressed_shell(oriented_shell)?;
            boundaries.push(shell);
            skipped.extend(more);
        }
        Ok((CompressedSolid { boundaries }, skipped))
    }
}

#[derive(Clone, Debug, PartialEq, derive_more::From)]
pub enum NodeMatrix {
    Identity,
    Transform(Box<ItemDefinedTransformation>),
}

#[derive(Clone, Debug, PartialEq, derive_more::From)]
pub enum ProductShape {
    Shells(Vec<CompressedShell<Point3, Curve3D, Surface>>),
    Solid(CompressedSolid<Point3, Curve3D, Surface>),
    Matrix(Matrix4),
}

pub type ProductEntity = NodeEntity<Vec<ProductShape>, PartAttrs>;
pub type AssembleEntity = EdgeEntity<NodeMatrix, PartAttrs>;
pub type StepAssembly = Assembly<Vec<ProductShape>, PartAttrs, NodeMatrix, PartAttrs>;

impl TryFrom<&NodeMatrix> for Matrix3 {
    type Error = StepConvertingError;
    fn try_from(value: &NodeMatrix) -> Result<Self, Self::Error> {
        match value {
            NodeMatrix::Identity => Ok(Self::identity()),
            NodeMatrix::Transform(trans) => (&**trans).try_into(),
        }
    }
}

impl TryFrom<&NodeMatrix> for Matrix4 {
    type Error = StepConvertingError;
    fn try_from(value: &NodeMatrix) -> Result<Self, Self::Error> {
        match value {
            NodeMatrix::Identity => Ok(Self::identity()),
            NodeMatrix::Transform(trans) => (&**trans).try_into(),
        }
    }
}

impl ProductShape {
    pub fn try_from_index(idx: u64, table: &Table) -> Converted<Self> {
        if let Some(step_solid) = table.manifold_solid_brep.get(&idx) {
            let (solid, skipped) = table.to_compressed_solid(step_solid)?;
            Ok((solid.into(), skipped))
        } else if let Some(step_shells) = table.shell_based_surface_model.get(&idx) {
            let (shells, skipped) = table.to_compressed_shells(step_shells)?;
            Ok((shells.into(), skipped))
        } else if table.axis2_placement_3d.contains_key(&idx) {
            let axis = EntityTable::<Axis2Placement3dHolder>::get_owned(table, idx)?;
            Ok((Matrix4::from(&axis).into(), Vec::new()))
        } else {
            Err("Unknown Shape".into())
        }
    }
}

impl Table {
    fn product_node_entity(
        &self,
        pds_idx: u64,
        pd: &ProductDefinitionHolder,
        skipped: &mut Vec<Skipped>,
    ) -> Result<ProductEntity, StepConvertingError> {
        let PlaceHolder::Ref(Name::Entity(pdf_idx)) = &pd.formation else {
            return Err("failed to reference `product_definition.formation`".into());
        };
        let Some(pdf) = self.product_definition_formation.get(pdf_idx) else {
            return Err("failed to reference `prouct_definition_formation`".into());
        };
        let PlaceHolder::Ref(Name::Entity(p_idx)) = &pdf.of_product else {
            return Err("failed to reference `product_definition_formation.of_product`".into());
        };
        let Some(product) = self.product.get(p_idx) else {
            return Err("failed to reference `product`".into());
        };
        let attrs = PartAttrs {
            id: product.id.clone(),
            name: product.name.clone(),
            description: product.description.clone(),
        };

        let Some(sdr) = self.shape_definition_representation.values().find(|sdr| {
            let &PlaceHolder::Ref(Name::Entity(idx)) = &sdr.definition else {
                return false;
            };
            pds_idx == idx
        }) else {
            return Err("failed to find `shape_definition_representation` corresp. to `product_definition_shape`".into());
        };
        let PlaceHolder::Ref(Name::Entity(sr_idx)) = &sdr.used_representation else {
            return Err(
                "failed to reference `shape_definition_representation.used_representation`".into(),
            );
        };
        let Some(sr) = self.shape_representation.get(sr_idx) else {
            return Err("failed to reference `shape_representation`".into());
        };
        let mut shape = Vec::new();
        for place_holder in &sr.items {
            let &PlaceHolder::Ref(Name::Entity(item_idx)) = place_holder else {
                return Err(
                    "failed to reference an element of `shape_representation.items`".into(),
                );
            };
            let (item, more) = ProductShape::try_from_index(item_idx, self)?;
            shape.push(item);
            skipped.extend(more);
        }

        Ok(NodeEntity { shape, attrs })
    }

    fn assy_node_entity(
        &self,
        pds_idx: u64,
        next_assy: &NextAssemblyUsageOccurrenceHolder,
    ) -> Result<(AssembleEntity, (u64, u64)), StepConvertingError> {
        let &PlaceHolder::Ref(Name::Entity(parent_idx)) = &next_assy.relating_product_definition
        else {
            return Err("failed to reference the parent node".into());
        };
        let &PlaceHolder::Ref(Name::Entity(child_idx)) = &next_assy.related_product_definition
        else {
            return Err("failed to reference the child node".into());
        };

        let attrs = PartAttrs {
            id: next_assy.id.clone(),
            name: next_assy.name.clone(),
            description: next_assy.description.clone(),
        };

        let Some(cdsr) = self
            .context_dependent_shape_representation
            .values()
            .find(|cdsr| {
                let &PlaceHolder::Ref(Name::Entity(idx)) = &cdsr.represented_product_relation
                else {
                    return false;
                };
                pds_idx == idx
            })
        else {
            return Err("".into());
        };

        let PlaceHolder::Ref(Name::Entity(srrwt_idx)) = &cdsr.representation_relation else {
            return Err("failed to reference `context_dependent_shape_representation.representation_relation`".into());
        };

        let Some(srrwt) = self
            .shape_representation_relationship_with_transformation
            .get(srrwt_idx)
        else {
            return Err("failed to reference `shape_representation_relationship`".into());
        };
        let idtf = srrwt.transformation_operator.clone().into_owned(self)?;

        let entity = AssembleEntity {
            matrix: NodeMatrix::Transform(idtf.into()),
            attrs,
        };

        Ok((entity, (parent_idx, child_idx)))
    }

    /// Constructs the assembly of the file, with the faces, edges and vertices of every product
    /// that could not be converted.
    pub fn step_assy(&self) -> Converted<StepAssembly> {
        let mut skipped = Vec::new();
        let mut product_entities = Vec::<ProductEntity>::new();
        let mut indices_map = HashMap::<u64, usize>::new();
        let mut assy_nodes = Vec::<(AssembleEntity, (u64, u64))>::new();
        for (&pds_idx, pds) in &self.product_definition_shape {
            let &PlaceHolder::Ref(Name::Entity(idx)) = &pds.definition else {
                return Err("failed to reference `product_definition_shape.definition`".into());
            };
            if let Some(pd) = self.product_definition.get(&idx) {
                product_entities.push(self.product_node_entity(pds_idx, pd, &mut skipped)?);
                indices_map.insert(idx, product_entities.len() - 1);
            } else if let Some(next_assy) = self.next_assembly_usage_occurrence.get(&idx) {
                assy_nodes.push(self.assy_node_entity(pds_idx, next_assy)?);
            }
        }

        let adjacency = assy_nodes
            .into_iter()
            .map(|(entity, (from, to))| {
                let from = *indices_map.get(&from)?;
                let to = *indices_map.get(&to)?;
                Some((from, to, entity))
            })
            .collect::<Option<Vec<_>>>()
            .ok_or::<StepConvertingError>("failed to reference `product_definiion_shape`".into())?;

        let assy = StepAssembly::try_from_adjacency(product_entities, adjacency)
            .ok_or("maybe the graph has a cycle.")?;
        Ok((assy, skipped))
    }
}

pub trait StepShell {
    fn to_compressed_shell(
        &self,
        table: &Table,
    ) -> Converted<CompressedShell<Point3, Curve3D, Surface>>;
}

impl StepShell for ShellHolder {
    fn to_compressed_shell(
        &self,
        table: &Table,
    ) -> Converted<CompressedShell<Point3, Curve3D, Surface>> {
        let mut skipped = Vec::new();
        let (vertices, vidx_map) = table.shell_vertices(self, &mut skipped);
        let (edges, eidx_map) = table.shell_edges(self, &vidx_map, &mut skipped);
        let faces = table.shell_faces(self, &eidx_map, &mut skipped);
        Ok((
            CompressedShell {
                vertices,
                edges,
                faces,
            },
            skipped,
        ))
    }
}

impl StepShell for OrientedShellHolder {
    fn to_compressed_shell(
        &self,
        table: &Table,
    ) -> Converted<CompressedShell<Point3, Curve3D, Surface>> {
        let PlaceHolder::Ref(Name::Entity(idx)) = &self.shell_element else {
            return Err("failed to reference shell".into());
        };
        let Some(shell) = table.shell.get(idx) else {
            return Err("failed to reference shell".into());
        };
        let (mut res, skipped) = shell.to_compressed_shell(table)?;
        if !self.orientation {
            for face in &mut res.faces {
                face.orientation = !face.orientation;
            }
        }
        Ok((res, skipped))
    }
}

impl StepShell for ShellAnyHolder {
    fn to_compressed_shell(
        &self,
        table: &Table,
    ) -> Converted<CompressedShell<Point3, Curve3D, Surface>> {
        match self {
            ShellAnyHolder::OrientedShell(shell) => shell.to_compressed_shell(table),
            ShellAnyHolder::Shell(shell) => shell.to_compressed_shell(table),
        }
    }
}
