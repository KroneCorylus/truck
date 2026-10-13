import math
import numpy as np


def volume(a, b, shift, height, radius, count=1024, order=32):
    phi = 2 * math.pi * np.arange(count) / count
    e = np.stack((a*np.cos(phi), b*np.sin(phi), np.zeros(count)), axis=1)
    ep = np.stack((-a*np.sin(phi), b*np.cos(phi), np.zeros(count)), axis=1)
    epp = np.stack((-a*np.cos(phi), -b*np.sin(phi), np.zeros(count)), axis=1)
    vector = np.array([shift, 0., height])
    axis = np.array([0., 0., 1.])
    w = np.cross(ep, vector)
    wp = np.cross(epp, vector)
    length = np.linalg.norm(w, axis=1)
    n = w/length[:,None]
    np_ = (wp - n*np.sum(n*wp,axis=1)[:,None])/length[:,None]
    v = (height-radius+radius*n[:,2])/height
    vp = radius*np_[:,2]/height
    center = e+v[:,None]*vector-radius*n
    cp = ep+vp[:,None]*vector-radius*np_
    horizontal = np.sqrt(1-n[:,2]**2)
    m = (n-n[:,2,None]*axis)/horizontal[:,None]
    mp = (np_-np_[:,2,None]*axis)/horizontal[:,None] + m*(n[:,2]*np_[:,2]/horizontal**2)[:,None]
    alpha = np.arccos(n[:,2])
    nodes, weights = np.polynomial.legendre.leggauss(order)
    theta = alpha[:,None]*(nodes+1)/2
    sine, cosine = np.sin(theta)[:,:,None], np.cos(theta)[:,:,None]
    patch = center[:,None,:]+radius*(cosine*axis+sine*m[:,None,:])
    dp = cp[:,None,:]+radius*sine*mp[:,None,:]
    dt = radius*(-sine*axis+cosine*m[:,None,:])
    step = 2*math.pi/count
    patch_moment = np.sum(np.sum(patch*np.cross(dt,dp),axis=2)*weights*alpha[:,None]/2)*step
    cap_area = np.sum(center[:,0]*cp[:,1]-center[:,1]*cp[:,0])*step/2
    side_moment = np.sum((1-v)*np.sum(e*w,axis=1))*step
    removed = (height*(math.pi*a*b-cap_area)+side_moment-patch_moment)/3
    return math.pi*a*b*height-removed, removed


if __name__ == '__main__':
    for case, args in [('circle-check',(50.,50.,0.,100.,10.)),('J6',(100.,50.,20.,100.,10.)),('J8',(50.,50.,100.,100.,10.))]:
        print(case)
        for count,order in [(256,16),(512,32),(1024,64)]:
            print(count,order,volume(*args,count,order))
    r=10.;R=50.
    exact=2*math.pi*(R-r*(10-3*math.pi)/(12-3*math.pi))*r*r*(1-math.pi/4)
    print('circle-check independent exact removed',exact)
