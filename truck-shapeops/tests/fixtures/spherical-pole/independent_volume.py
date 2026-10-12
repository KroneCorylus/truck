"""Volume of a sphere quadrant with one meridian rim rounded; see README.md."""
import math


def volumes(divisions, radius=150.0, fillet=10.0):
    height = radius * math.sqrt(3) / 2
    original = math.pi / 4 * (
        radius**2 * height - height**3 / 3 + 2 * radius**3 / 3
    )
    center = math.sqrt(radius**2 - 2 * radius * fillet)

    def area(rho):
        return (
            height * math.sqrt(rho**2 - height**2)
            + rho**2 * math.asin(height / rho)
            + math.pi * rho**2 / 2
        ) / 2

    limit = math.acos(-fillet / (radius - fillet))
    step = limit / divisions

    def strip(i):
        angle = step * i
        y = fillet * (1 - math.cos(angle))
        old = math.sqrt(radius**2 - y**2)
        new = center + fillet * math.sin(angle)
        weight = 1 if i in (0, divisions) else 2 if i % 2 == 0 else 4
        return weight * (area(old) - area(new)) * fillet * math.sin(angle)

    removed = sum(strip(i) for i in range(divisions + 1)) * step / 3
    return original, original - removed


if __name__ == "__main__":
    for divisions in (256, 512, 1024, 2048, 4096):
        print(divisions, *volumes(divisions))
