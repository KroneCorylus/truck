"""Dimension-only volume of the two concave rounds on the cylinder/cone stack."""
import math

def added_volume(radius, support_radius, slope):
    norm = math.hypot(1, slope)
    center = support_radius + radius * (norm - slope)
    contact = center - radius / norm
    height = radius - radius * slope / norm
    a, b = math.atan2(-slope, -1), -math.pi / 2
    def primitive(t):
        return radius * (center**2 * math.sin(t)
            + center * radius * (t + math.sin(2*t)/2)
            + radius**2 * (math.sin(t) - math.sin(t)**3/3))
    return -math.pi * (height * (support_radius**2 + support_radius*contact + contact**2)/3
        + primitive(b) - primitive(a))

original = math.pi * (3**2 * 5 + 3 * (2**2 + 2*1.5 + 1.5**2)/3 + 1**2 * 2)
additions = [added_volume(.3, 2, 1/6), added_volume(.3, 1, 0.)]
print('input:', original)
print('added:', additions)
print('filleted:', original + sum(additions))
