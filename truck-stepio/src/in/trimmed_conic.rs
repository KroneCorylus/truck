use super::*;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum ParameterTrim {
    #[serde(rename = "PARAMETER_VALUE")]
    ParameterValue(f64),
}

impl<'de> Deserialize<'de> for ParameterTrim {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let values = BTreeMap::<String, f64>::deserialize(deserializer)?;
        if values.len() == 1 {
            if let Some(value) = values.get("PARAMETER_VALUE") {
                return Ok(Self::ParameterValue(*value));
            }
        }
        Err(serde::de::Error::custom(
            "TRIMMED_CURVE requires PARAMETER_VALUE selectors",
        ))
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum TrimmingPreference {
    Parameter,
    Cartesian,
    Unspecified,
}

/// Parameter-trimmed conics. Restricting the basis to a conic prevents recursive
/// curve references from entering the generated holder resolver.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(field = trimmed_curve)]
#[holder(generate_deserialize)]
pub struct TrimmedConic {
    pub label: String,
    #[holder(use_place_holder)]
    pub basis_curve: Conic,
    pub trim_1: Vec<ParameterTrim>,
    pub trim_2: Vec<ParameterTrim>,
    pub sense_agreement: bool,
    pub master_representation: TrimmingPreference,
}

impl TrimmedConic {
    fn range(&self) -> Result<(f64, f64), StepConvertingError> {
        if !matches!(self.master_representation, TrimmingPreference::Parameter) {
            return Err("TRIMMED_CURVE requires PARAMETER master representation".into());
        }
        let ([ParameterTrim::ParameterValue(a)], [ParameterTrim::ParameterValue(b)]) =
            (self.trim_1.as_slice(), self.trim_2.as_slice())
        else {
            return Err("TRIMMED_CURVE requires singleton PARAMETER_VALUE trims".into());
        };
        let (a, mut b) = if self.sense_agreement {
            (*a, *b)
        } else {
            (*b, *a)
        };
        if !a.is_finite() || !b.is_finite() {
            return Err("TRIMMED_CURVE requires finite parameters".into());
        }
        if !matches!(self.basis_curve, Conic::Circle(_) | Conic::Ellipse(_)) {
            return Err("TRIMMED_CURVE supports only circle and ellipse bases".into());
        }
        let valid_axes = match &self.basis_curve {
            Conic::Circle(c) => c.radius.is_finite() && c.radius > 0.,
            Conic::Ellipse(c) => {
                c.semi_axis_1.is_finite()
                    && c.semi_axis_2.is_finite()
                    && c.semi_axis_1 > 0.
                    && c.semi_axis_2 > 0.
            }
            _ => false,
        };
        if !valid_axes {
            return Err("TRIMMED_CURVE requires finite positive conic axes".into());
        }
        // Conic parameters are radians, independent of axis lengths. Preserve
        // explicit turns; unwrap only descending seam crossings.
        let tau = 2. * PI;
        let whole_turn = b == a + tau || a == b + tau;
        if a == b || (!whole_turn && (b - a).abs() > tau) {
            return Err("TRIMMED_CURVE requires a nonzero span of at most one turn".into());
        }
        if whole_turn {
            b = a + tau;
        } else if b < a {
            b += tau;
        }
        if b <= a || !b.is_finite() || phase_error((a, b)) > TOLERANCE * 0.01 {
            return Err("TRIMMED_CURVE has an invalid periodic interval".into());
        }
        Ok((a, b))
    }
}

// Includes rounded period shifts, the binary64 period error and Processor's
// lo + hi - t reflection. Keep this arithmetic below 1% of point tolerance.
fn phase_error((a, b): (f64, f64)) -> f64 { 16. * f64::EPSILON * (a.abs() + b.abs() + 2. * PI) }

fn finite_placement<const N: usize>(matrix: &[[f64; N]; N], range: (f64, f64)) -> bool {
    matrix.iter().flatten().all(|x| x.is_finite())
        && (0..N - 1).all(|i| {
            let axes = matrix[0][i].abs() + matrix[1][i].abs();
            (matrix[N - 1][i].abs() + axes).is_finite()
                && axes * phase_error(range) <= TOLERANCE * 0.01 / (N as f64)
        })
}

impl TryFrom<&TrimmedConic> for Curve3D {
    type Error = StepConvertingError;
    fn try_from(value: &TrimmedConic) -> Result<Self, Self::Error> {
        let range = value.range()?;
        let Conic3D::Ellipse(curve) = Conic3D::try_from(&value.basis_curve)? else {
            return Err("unsupported TRIMMED_CURVE basis".into());
        };
        let matrix: &[[f64; 4]; 4] = curve.transform().as_ref();
        let finite_inverse = curve.transform().invert().is_some_and(|inverse| {
            let entries: &[[f64; 4]; 4] = inverse.as_ref();
            entries.iter().flatten().all(|x| x.is_finite())
        });
        if !finite_placement(matrix, range) || !finite_inverse {
            return Err("TRIMMED_CURVE requires a finite nonsingular placement".into());
        }
        let mut curve = curve.map_ref(|c| truck::TrimmedCurve::new(*c.curve(), range));
        if !value.sense_agreement {
            curve.invert();
        }
        Ok(Self::Conic(Conic3D::Ellipse(curve)))
    }
}

impl TryFrom<&TrimmedConic> for Curve2D {
    type Error = StepConvertingError;
    fn try_from(value: &TrimmedConic) -> Result<Self, Self::Error> {
        let range = value.range()?;
        let Conic2D::Ellipse(curve) = Conic2D::try_from(&value.basis_curve)? else {
            return Err("unsupported TRIMMED_CURVE basis".into());
        };
        let matrix: &[[f64; 3]; 3] = curve.transform().as_ref();
        let finite_inverse = curve.transform().invert().is_some_and(|inverse| {
            let entries: &[[f64; 3]; 3] = inverse.as_ref();
            entries.iter().flatten().all(|x| x.is_finite())
        });
        if !finite_placement(matrix, range) || !finite_inverse {
            return Err("TRIMMED_CURVE requires a finite nonsingular placement".into());
        }
        let mut curve = curve.map_ref(|c| truck::TrimmedCurve::new(*c.curve(), range));
        if !value.sense_agreement {
            curve.invert();
        }
        Ok(Self::Conic(Conic2D::Ellipse(curve)))
    }
}
