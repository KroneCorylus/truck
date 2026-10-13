import numpy as np
from numpy.polynomial.legendre import leggauss

R, r, f = 200., 50., 10.
angle = 8 * np.pi / 9
height = r * np.sin(angle)

def blend(theta, w):
    rho = R + (r - f) * np.cos(theta)
    x = np.sqrt(rho * rho - f * f)
    zero = np.zeros_like(theta + w)
    center = np.stack((x + zero, f + zero, (r - f) * np.sin(theta) + zero), axis=-1)
    normal = np.stack((np.cos(theta) * x / rho + zero,
                       np.cos(theta) * f / rho + zero, np.sin(theta) + zero), axis=-1)
    dot = -normal[..., 1]
    other = (np.array([0., -1., 0.]) - dot[..., None] * normal) / np.sqrt(1 - dot * dot)[..., None]
    beta = w * np.arccos(dot)
    return center + f * (np.cos(beta)[..., None] * normal + np.sin(beta)[..., None] * other)

for count in [16, 32, 64, 128]:
    roots, weights = leggauss(count)
    w = (roots + 1) / 2
    low, high = np.full_like(w, np.pi / 2), np.full_like(w, np.pi)
    for _ in range(60):
        middle = (low + high) / 2
        above = blend(middle, w)[..., 2] > height
        low = np.where(above, middle, low)
        high = np.where(above, high, middle)
    end = (low + high) / 2
    theta = (roots[:, None] + 1) * end[None, :] / 2
    p = blend(theta, w[None, :])
    dt = blend(theta + 1.e-20j, w[None, :]).imag / 1.e-20
    dw = blend(theta, w[None, :] + 1.e-20j).imag / 1.e-20
    integrand = p[..., 1] * np.cross(dt, dw)[..., 1]
    added_flux = np.sum(integrand * weights[:, None] * weights[None, :] * end[None, :] / 4)
    t = (roots + 1) * angle / 2
    rho = R + r * np.cos(t)
    alpha = np.arcsin(f / (R + (r-f) * np.cos(t)))
    removed_flux = np.sum(weights * r * rho * rho * np.cos(t) * (alpha/2 - np.sin(2*alpha)/4)) * angle/2
    s = np.sin(angle)
    original = np.pi / 4 * (r*R*R*s + R*r*r*(angle+np.sin(2*angle)/2) + r**3*(s-s**3/3))
    print(count, 'original', original, 'removed_flux', removed_flux, 'blend_flux', added_flux,
          'volume', original-removed_flux+added_flux)
