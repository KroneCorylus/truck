# Continuing rolling-ball contacts on adjacent supports

ContactCircle::try_from_contacts initializes the existing offset-surface solve
from parameter guesses on both supports. Its guide supplies a center section
plane; the guide point need not lie on the supports. The signed radius chooses
the normal side. ApproxFilletSurface::approx_contact_circles fits those sections
using the existing adaptive contact and frame checks. The edge-based constructor
and fitter delegate to the same implementations. No new surface representation
or dependency is introduced.

The solve now converges to near2 before exposing its contacts. The previous near
threshold could leave the contact point observably off the radius constraint on
small geometry. The unchanged contact_precision regression fails on parent
67cf761f in an isolated checkout and passes after the fix.

The continuation regression independently derives the Q4 post/roof center:
C(t) = (2.5 + 2*cos(t), 2.5 + 2*sin(t),
        4 + 2*sqrt(1 - (cos(t) - 0.75)^2)), |t| <= acos(0.75).
It checks center positions, support-normal offsets, fitted section geometry,
radius and normals at scales 0.01, 1 and 100 after a rigid transform. Invalid
radius, nonfinite guesses and a singular guide return None.

This is contact and patch construction only. Q4's full solid remains rejected
until patch transitions and face-loop reconnection are implemented and validated.
No complete feature-parity verdict follows from these tests.

Validation: full release geometry/modeling/meshalgo/shapeops/STEP run: 857 passed,
one previously recorded PCurve derivative failure, five existing ignored tests
across 128 targets. The separately added contact_precision target also passes.
All-target release Clippy and action benchmark smoke pass. Existing tests are
unchanged. No performance claim is made.
