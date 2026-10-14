"""Volumes derived from the three input profiles and specified fillet radii."""
from math import pi, tan, atan, asin, sqrt, sin, cos

def polygon_corner(radius, angle):
    return radius**2 * (1/tan(angle/2) - (pi-angle)/2)

def circular_corner(support_radius, radius, inner):
    offset = support_radius + (radius if inner else -radius)
    angle = asin(radius/offset)
    center = sqrt(offset**2-radius**2)
    cx, end, original = (-center, -angle, -support_radius**2*angle) if inner else (center, angle, support_radius**2*angle)
    # Green's theorem replaces the original line/circle join by its tangent circular arc.
    arc = radius*cx*(sin(end)+1)-radius**2*cos(end)+radius**2*(end+pi/2)
    return (original-arc)/2

volumes = {
    'A2': 6000-10*(polygon_corner(2,pi/2)+polygon_corner(3,pi/2)),
    'B5': 15-5*(polygon_corner(.2,atan(.25))+polygon_corner(.5,pi/2)),
    'C4': 20*(48*pi-circular_corner(10,2,False)-circular_corner(2,1,True)),
}
if __name__ == '__main__':
    for case, volume in volumes.items():
        print(case, format(volume,'.17g'))
