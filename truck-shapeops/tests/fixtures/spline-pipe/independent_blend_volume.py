"""Independent ideal-tube volume; the envelope derivation is in README.md."""

import math


def simpson(function, a, b, divisions):
    return (b - a) / (3 * divisions) * sum(
        (1 if i in (0, divisions) else 2 if i % 2 == 0 else 4)
        * function(a + (b - a) * i / divisions)
        for i in range(divisions + 1)
    )


def centerline(t):
    x = 300 * t - 300 * t * t + 200 * t * t * t
    y = 300 * t * t - 200 * t * t * t
    dx, dy = 300 - 600 * t + 600 * t * t, 600 * t - 600 * t * t
    ddx, ddy = -600 + 1200 * t, 600 - 1200 * t
    speed = math.hypot(dx, dy)
    return x, y, dx / speed, dy / speed, speed, (dx * ddy - dy * ddx) / speed**3


def rounded_volume(divisions, blend):
    radius = 50.0
    offset = radius - blend

    def removed(theta):
        c, s = math.cos(theta), math.sin(theta)
        lo, hi = 0.0, 1.0
        for _ in range(60):
            t = (lo + hi) / 2
            x, _, tx, ty, speed, kappa = centerline(t)
            if x - offset * ty * c < blend:
                lo = t
            else:
                hi = t
        t = (lo + hi) / 2
        _, _, tx, ty, speed, kappa = centerline(t)
        dt = -offset * ty * s / (speed * (1 - offset * kappa * c) * tx)
        ky = speed * (1 - offset * kappa * c) * ty * dt - offset * tx * s
        kz = offset * c
        nx, ny, nz = -ty * c, tx * c, s
        ny_der = -kappa * speed * ty * c * dt - tx * s
        nz_der = c
        q = math.hypot(ny, nz)
        uy, uz = ny / q, nz / q
        q_der = (ny * ny_der + nz * nz_der) / q
        uy_der = ny_der / q - ny * q_der / q**2
        uz_der = nz_der / q - nz * q_der / q**2
        k = -ky * uz + kz * uy
        omega = -uy_der * uz + uz_der * uy
        assert k > 0 and omega > 0
        beta = math.acos(-nx)
        i0 = math.sin(beta) - beta / 2 - math.sin(2 * beta) / 4
        i1 = 1 / 6 - math.cos(beta)**2 / 2 + math.cos(beta)**3 / 3

        def strip(t):
            x, _, _, ty, speed, kappa = centerline(t)
            x -= radius * ty * c
            return x * (-ty * c) * radius * speed * (1 - radius * kappa * c)

        old_wall = simpson(strip, 0, t, divisions // 4)
        return old_wall + blend**2 * (k * i0 + blend * omega * i1)

    old = math.pi * radius**2 * simpson(lambda t: centerline(t)[4], 0, 1, divisions * 2)
    return old - simpson(removed, 0, 2 * math.pi, divisions)


if __name__ == "__main__":
    for blend in (5.0, 10.0):
        for divisions in (128, 256, 512):
            print(blend, divisions, rounded_volume(divisions, blend))
