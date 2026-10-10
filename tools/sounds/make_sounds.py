#!/usr/bin/env python3
"""Placeholder sound effects for Gahturiyu Sim, synthesised from scratch.

Every sound is built here from noise, sines, simple filters and envelopes;
no recordings or third-party samples are used. The script is seeded, so
running it again writes exactly the same files.

    python3 tools/sounds/make_sounds.py
        # writes assets/sounds/*.ogg and manifest.json (what the game loads),
        # and listening previews in tools/sounds/preview/ (wav/*.wav, listen.html)

Needs numpy, scipy and soundfile (libsndfile >= 1.0.29 for Ogg Vorbis).
"""
import json
import os
import zlib

import numpy as np
import soundfile as sf
from scipy import signal

SR = 44100
SEED = 20261010
HERE = os.path.dirname(os.path.abspath(__file__))
# The game's files go to the repo's assets/sounds; listening previews stay here.
OUT = os.path.normpath(os.path.join(HERE, "..", "..", "assets", "sounds"))
PREVIEW = os.path.join(HERE, "preview")

# Peak level per group (dBFS), with a few per-sound trims below.
GROUP_PEAK = {"interface": -18.0, "screens": -14.0, "world": -10.0, "loop": -20.0}
TRIM = {"ui_hover": -6.0, "ui_release": -2.0, "talk_open": -3.0}


# ---------------------------------------------------------------- building blocks

def rng_for(name):
    return np.random.default_rng([SEED, zlib.crc32(name.encode())])


def n_of(sec):
    return int(round(sec * SR))


def tvec(n):
    return np.arange(n) / SR


def place(buf, x, at):
    """Add x into buf starting at time `at` (s), clipped to buf's length."""
    i = n_of(at)
    if i >= len(buf):
        return buf
    m = min(len(x), len(buf) - i)
    buf[i:i + m] += x[:m]
    return buf


def decay(n, tau, attack=0.0005):
    """Exponential decay with a short linear attack (no click at onset)."""
    t = tvec(n)
    e = np.exp(-t / tau)
    a = n_of(attack)
    if a > 1:
        e[:a] *= np.linspace(0, 1, a)
    return e


def bell(n, peak_at=0.5, sharp=2.0):
    """A smooth hump from 0 to 1 and back, peaking at fraction peak_at."""
    x = np.linspace(0, 1, n)
    up = np.clip(x / peak_at, 0, 1)
    down = np.clip((1 - x) / (1 - peak_at), 0, 1)
    e = np.where(x < peak_at, np.sin(up * np.pi / 2), np.sin(down * np.pi / 2))
    return e ** sharp


def bandpass(x, lo, hi, order=2):
    sos = signal.butter(order, [lo, hi], btype="band", fs=SR, output="sos")
    return signal.sosfilt(sos, x)


def lowpass(x, fc, order=2):
    sos = signal.butter(order, fc, btype="low", fs=SR, output="sos")
    return signal.sosfilt(sos, x)


def highpass(x, fc, order=2):
    sos = signal.butter(order, fc, btype="high", fs=SR, output="sos")
    return signal.sosfilt(sos, x)


def resonate(x, freqs, q=12.0, gains=None):
    """Run x through a bank of parallel resonant peaks (a crude wooden body)."""
    out = np.zeros_like(x)
    gains = gains or [1.0] * len(freqs)
    for f, g in zip(freqs, gains):
        b, a = signal.iirpeak(f, q, fs=SR)
        out += g * signal.lfilter(b, a, x)
    return out


def svf_bandpass(x, fc, q):
    """Bandpass with a cutoff that moves per sample (TPT state-variable filter)."""
    fc = np.broadcast_to(np.asarray(fc, float), x.shape)
    g = np.tan(np.pi * np.clip(fc, 20, SR * 0.45) / SR)
    k = 1.0 / q
    out = np.empty_like(x)
    ic1 = ic2 = 0.0
    for i in range(len(x)):
        gi = g[i]
        a1 = 1.0 / (1.0 + gi * (gi + k))
        a2 = gi * a1
        a3 = gi * a2
        v3 = x[i] - ic2
        v1 = a1 * ic1 + a2 * v3
        v2 = ic2 + a2 * ic1 + a3 * v3
        ic1 = 2 * v1 - ic1
        ic2 = 2 * v2 - ic2
        out[i] = v1
    return out


def sweep(n, f0, f1, curve="exp"):
    x = np.linspace(0, 1, n)
    if curve == "exp":
        return f0 * (f1 / f0) ** x
    return f0 + (f1 - f0) * x


def tone(n, freq, phase=0.0):
    """Sine with a constant or per-sample frequency, starting at zero."""
    f = np.broadcast_to(np.asarray(freq, float), (n,))
    ph = 2 * np.pi * np.cumsum(f) / SR
    ph -= ph[0]
    return np.sin(ph + phase)


def modes(n, freqs, taus, amps, attack=0.0003):
    """A struck object: a few decaying sine partials."""
    out = np.zeros(n)
    for f, tau, a in zip(freqs, taus, amps):
        out += a * tone(n, f) * decay(n, tau, attack)
    return out


def grains(n, rate, rng, env=None):
    """Sparse random impulses (stick-slip, crackle, grit). rate = per second."""
    p = rate / SR
    if env is not None:
        p = p * env
    hits = rng.random(n) < p
    amp = rng.uniform(0.3, 1.0, n) * rng.choice([-1, 1], n)
    if env is not None:
        amp = amp * np.sqrt(env)   # the envelope thins the grains and softens them
    return hits * amp


# ---------------------------------------------------------------- interface

def ui_hover(rng, take):
    n = n_of(0.035)
    x = bandpass(rng.standard_normal(n), 2500, 5000) * decay(n, 0.003)
    x += 0.4 * modes(n, [2300], [0.006], [1.0])
    return lowpass(x, 7000)


def ui_press(rng, take):
    n = n_of(0.075)
    d = 1 + rng.uniform(-0.06, 0.06)
    click = bandpass(rng.standard_normal(n), 1500, 4000) * decay(n, 0.0035)
    body = modes(n, [680 * d, 1550 * d], [0.014, 0.008], [0.9, 0.35])
    brass = modes(n, [2650 * d, 3870 * d], [0.018, 0.010], [0.18, 0.08])
    return lowpass(0.8 * click + body + brass, 6500)


def ui_release(rng, take):
    n = n_of(0.055)
    d = 1.22 + rng.uniform(-0.03, 0.03)
    click = bandpass(rng.standard_normal(n), 2000, 5000) * decay(n, 0.0025)
    body = modes(n, [680 * d, 1550 * d], [0.009, 0.005], [0.6, 0.25])
    brass = modes(n, [2650 * d], [0.010], [0.12])
    return lowpass(0.7 * click + body + brass, 7000)


def ui_open(rng, take):
    n = n_of(0.2)
    noise = rng.standard_normal(n)
    x = svf_bandpass(noise, sweep(n, 600, 2600), 1.6)
    x *= bell(n, 0.5, 2.4)
    x += 0.25 * svf_bandpass(rng.standard_normal(n), sweep(n, 2500, 5000), 2.0) * bell(n, 0.6, 2.5)
    return lowpass(x, 6000)


def ui_close(rng, take):
    n = n_of(0.15)
    x = svf_bandpass(rng.standard_normal(n), sweep(n, 2200, 650), 1.6)
    x *= bell(n, 0.35, 1.6)
    return lowpass(x, 5500)


def thud(n, f0, f1, tau, rng, noise_lp=900, noise_amt=0.4, noise_tau=0.012):
    body = tone(n, sweep(n, f0, f1)) * decay(n, tau, 0.001)
    puff = lowpass(rng.standard_normal(n), noise_lp) * decay(n, noise_tau, 0.0008)
    return body + noise_amt * puff


def ui_cant(rng, take):
    n = n_of(0.18)
    out = np.zeros(n)
    m = n_of(0.09)
    place(out, thud(m, 170, 120, 0.022, rng), 0.0)
    place(out, 0.75 * thud(m, 150, 105, 0.024, rng), 0.085)
    return lowpass(out, 1200)


def ui_select(rng, take):
    n = n_of(0.15)
    f = 1900
    x = modes(n, [f, f * 2.32, f * 4.07, f * 0.5],
              [0.055, 0.032, 0.016, 0.03], [1.0, 0.45, 0.18, 0.15], attack=0.0012)
    x += 0.15 * bandpass(rng.standard_normal(n), 3000, 7000) * decay(n, 0.002)
    return lowpass(x, 8000)


def ui_order(rng, take):
    n = n_of(0.12)
    d = 1 + rng.uniform(-0.07, 0.07)
    x = thud(n, 190 * d, 95 * d, 0.03, rng, noise_lp=450, noise_amt=0.6, noise_tau=0.02)
    return lowpass(x, 1500)


# ---------------------------------------------------------------- screens

def talk_open(rng, take):
    n = n_of(0.3)
    x = svf_bandpass(rng.standard_normal(n), sweep(n, 700, 1300), 0.9)
    x *= bell(n, 0.4, 1.5)
    rustle = bandpass(grains(n, 700, rng, bell(n, 0.5, 1.0)), 1500, 4500)
    return lowpass(x + 0.6 * rustle, 5000)


def coin_hit(rng, n, base, amp=1.0, ring=1.0):
    r = rng.uniform(0.97, 1.03, 4)
    f = base * np.array([1.0, 1.58, 2.24, 2.92]) * r
    x = modes(n, f, ring * rng.uniform(0.6, 1.2) * np.array([0.11, 0.07, 0.045, 0.03]),
              [1.0, 0.55, 0.35, 0.2])
    x += 0.5 * bandpass(rng.standard_normal(n), 3000, 9000) * decay(n, 0.0015)
    return amp * x


def wood_tap(rng, n, amp=1.0):
    x = modes(n, [220, 510], [0.014, 0.008], [0.6, 0.3], attack=0.0008)
    x += 0.5 * lowpass(rng.standard_normal(n), 1500) * decay(n, 0.006)
    return amp * x


def coins(rng, take):
    n = n_of(0.4)
    out = np.zeros(n)
    m = n_of(0.2)
    t = 0.0
    count = rng.integers(5, 8)
    place(out, wood_tap(rng, m, 0.9), 0.0)
    for i in range(count):
        amp = 0.95 * (0.78 ** i) * rng.uniform(0.7, 1.0)
        place(out, coin_hit(rng, m, rng.uniform(2700, 4300), amp), t)
        if i < 2:
            place(out, wood_tap(rng, m, 0.4 * amp), t)
        t += rng.uniform(0.025, 0.06) * (0.92 ** i)
    return lowpass(out, 9000)


def coin_single(rng, take):
    n = n_of(0.15)
    out = np.zeros(n)
    m = n_of(0.15)
    place(out, wood_tap(rng, m, 0.6), 0.0)
    place(out, coin_hit(rng, m, 3500, 1.0, ring=0.45), 0.0)
    place(out, coin_hit(rng, m, 3500, 0.3, ring=0.35), 0.045)
    return lowpass(out, 9000)


def pack_open(rng, take):
    n = n_of(0.3)
    env = bell(n, 0.35, 1.2) * (tvec(n) < 0.22)
    creak = resonate(grains(n, 180, rng, env), [380, 860, 1500], q=8, gains=[1.0, 0.6, 0.3])
    rub = bandpass(rng.standard_normal(n), 300, 1800) * env * 0.15
    out = creak + rub
    m = n_of(0.08)
    buckle = modes(m, [2480, 3910, 5200], [0.025, 0.015, 0.008], [1.0, 0.5, 0.2])
    place(out, 0.35 * buckle / max(1e-9, np.abs(buckle).max()) * np.abs(out).max(), 0.215)
    return lowpass(out, 7000)


def craft_open(rng, take):
    n = n_of(0.4)
    out = np.zeros(n)
    m = n_of(0.2)
    times = [0.0, 0.07, 0.15, 0.24]
    bases = [620, 1350, 900, 1800]
    amps = [1.0, 0.6, 0.75, 0.35]
    for t, f, a in zip(times, bases, amps):
        f *= rng.uniform(0.95, 1.05)
        hit = modes(m, [f, f * 2.76, f * 5.4], [0.09, 0.05, 0.02], [1.0, 0.5, 0.2])
        place(out, a * 0.6 * hit, t)
        place(out, a * wood_tap(rng, m, 0.8), t)
    return lowpass(out, 7500)


def creak(n, rng, rate0, rate1, res, q=10, jitter=0.25):
    """Stick-slip friction: an uneven train of tiny pulses ringing a wooden body."""
    rate = sweep(n, rate0, rate1)
    rate = rate * (1 + jitter * lowpass(rng.standard_normal(n), 8) * 3)
    ph = np.cumsum(np.clip(rate, 5, None)) / SR
    pulses = np.diff(np.floor(ph), prepend=0.0)
    pulses *= rng.uniform(0.5, 1.0, n)
    return resonate(pulses, res, q=q, gains=[1.0, 0.7, 0.4][:len(res)])


def lid_open(rng, take):
    n = n_of(0.6)
    out = np.zeros(n)
    m = n_of(0.08)
    place(out, 0.5 * wood_tap(rng, m), 0.0)
    c = creak(n_of(0.52), rng, 35, 90, [420, 950, 1700], q=14)
    c *= bell(len(c), 0.45, 1.0)
    c /= max(1e-9, np.abs(c).max())
    place(out, 0.9 * c, 0.05)
    return lowpass(highpass(out, 120), 6000)


def lid_close(rng, take):
    n = n_of(0.25)
    out = thud(n, 120, 75, 0.045, rng, noise_lp=900, noise_amt=0.7, noise_tau=0.02)
    out += 0.6 * modes(n, [340, 760], [0.03, 0.015], [1.0, 0.4], attack=0.001)
    m = n_of(0.1)
    place(out, 0.25 * wood_tap(rng, m), 0.045)
    return lowpass(out, 3000)


def take_item(rng, take):
    n = n_of(0.15)
    env = bell(n, rng.uniform(0.3, 0.45), 2.2)
    x = svf_bandpass(rng.standard_normal(n), sweep(n, 900, 1800) * rng.uniform(0.9, 1.1), 1.2) * env
    x += 0.8 * bandpass(grains(n, 900, rng, env), 1200, 4000)
    return lowpass(x, 6000)


def page_turn(rng, take):
    n = n_of(0.35)
    env = bell(n, 0.6, 1.2)
    x = svf_bandpass(rng.standard_normal(n), sweep(n, 1800, 3500), 1.0) * env
    crinkle = bandpass(grains(n, 400, rng, env), 2000, 7000)
    x += 1.2 * crinkle
    m = n_of(0.12)
    flap = lowpass(rng.standard_normal(m), 700) * decay(m, 0.025, 0.004)
    place(x, 0.6 * flap / np.abs(flap).max() * np.abs(x).max(), 0.22)
    return lowpass(highpass(x, 200), 7500)


# ---------------------------------------------------------------- world

def step_dirt(rng, take):
    n = n_of(0.12)
    out = np.zeros(n)
    for at, a in [(0.0, 1.0), (rng.uniform(0.025, 0.04), 0.6)]:
        m = n_of(0.09)
        th = lowpass(rng.standard_normal(m), 450) * decay(m, 0.018, 0.002)
        grit = bandpass(grains(m, 3000, rng, decay(m, 0.022)), 1000, 4000)
        place(out, a * (th + 0.35 * grit), at)
    return highpass(lowpass(out, 5000), 60)


def step_stone(rng, take):
    n = n_of(0.1)
    out = np.zeros(n)
    for at, a in [(0.0, 1.0), (rng.uniform(0.018, 0.03), 0.55)]:
        m = n_of(0.07)
        click = bandpass(rng.standard_normal(m), 1500, 5000) * decay(m, 0.006)
        body = modes(m, [150 * rng.uniform(0.9, 1.1), 420], [0.015, 0.008], [0.6, 0.3], attack=0.0008)
        scuff = bandpass(rng.standard_normal(m), 2500, 7000) * bell(m, 0.3, 2) * 0.12
        place(out, a * (click + body + scuff), at)
    return lowpass(out, 8000)


def swing(rng, take):
    n = n_of(0.25)
    peak = rng.uniform(0.4, 0.5)
    fpk = rng.uniform(1300, 1800)
    x = np.linspace(0, 1, n)
    fc = 450 + (fpk - 450) * np.exp(-((x - peak) / 0.22) ** 2)
    out = svf_bandpass(rng.standard_normal(n), fc, 2.2) * bell(n, peak, 2.0)
    return lowpass(out, 6000)


def hit_flesh(rng, take):
    n = n_of(0.15)
    d = rng.uniform(0.9, 1.1)
    body = tone(n, sweep(n, 95 * d, 55 * d)) * decay(n, 0.045, 0.001)
    mass = lowpass(rng.standard_normal(n), 600) * decay(n, 0.025, 0.0008)
    slap = bandpass(rng.standard_normal(n), 1000, 3000) * decay(n, 0.004)
    return lowpass(body + 0.8 * mass + 0.25 * slap, 3500)


def hit_block(rng, take):
    n = n_of(0.15)
    d = rng.uniform(0.92, 1.08)
    wood = modes(n, [450 * d, 1120 * d], [0.025, 0.012], [1.0, 0.45], attack=0.0005)
    metal = modes(n, [1830 * d, 2960 * d, 4410 * d], [0.06, 0.035, 0.02], [0.35, 0.2, 0.1])
    click = bandpass(rng.standard_normal(n), 1500, 6000) * decay(n, 0.003)
    thump = lowpass(rng.standard_normal(n), 500) * decay(n, 0.015)
    return lowpass(wood + metal + 0.6 * click + 0.5 * thump, 8000)


def bow_release(rng, take):
    n = n_of(0.3)
    # Karplus-Strong plucked string, low and damped, pitch sagging a little.
    f0 = 98.0
    period = int(SR / f0)
    buf = lowpass(rng.standard_normal(period), 3000)
    out = np.zeros(n)
    damp = 0.992
    hist = np.concatenate([buf, np.zeros(n)])
    for i in range(period, period + n):
        hist[i] = damp * 0.5 * (hist[i - period] + hist[i - period - 1 if i - period - 1 >= 0 else 0])
    out = hist[period:period + n] * decay(n, 0.09, 0.001)
    thwip = svf_bandpass(rng.standard_normal(n), sweep(n, 2500, 900), 2.5) * decay(n, 0.03, 0.002)
    knock = modes(n, [260, 610], [0.02, 0.01], [0.5, 0.2], attack=0.0005)
    return lowpass(highpass(out + 0.35 * thwip + knock, 60), 6000)


def cast(rng, take):
    n = n_of(0.6)
    x = np.linspace(0, 1, n)
    env = np.clip(x / 0.45, 0, 1) ** 1.5 * np.clip((1 - x) / 0.3, 0, 1) ** 1.2
    out = np.zeros(n)
    base = sweep(n, 420, 640)
    for ratio, amp in [(1.0, 1.0), (1.5, 0.5), (2.0, 0.35), (3.01, 0.15)]:
        for det in (-0.006, 0.0, 0.007):
            vib = 1 + 0.004 * np.sin(2 * np.pi * rng.uniform(4.5, 6.5) * tvec(n))
            out += amp / 3 * tone(n, base * ratio * (1 + det) * vib, rng.uniform(0, 2 * np.pi))
    # light FM sparkle on top
    mod = np.sin(2 * np.pi * np.cumsum(base * 3.5) / SR)
    out += 0.12 * np.sin(2 * np.pi * np.cumsum(base * 4.0) / SR + 1.5 * mod)
    air = svf_bandpass(rng.standard_normal(n), sweep(n, 3000, 7000), 1.5) * 0.35
    shimmer = 0.75 + 0.25 * np.sin(2 * np.pi * 11 * tvec(n))
    return lowpass((out + air) * env * shimmer, 9000)


def door_open(rng, take):
    n = n_of(0.7)
    out = np.zeros(n)
    m = n_of(0.06)
    latch = modes(m, [2200, 3400, 5100], [0.012, 0.008, 0.005], [1.0, 0.5, 0.2])
    latch += 0.5 * bandpass(rng.standard_normal(m), 2000, 7000) * decay(m, 0.002)
    place(out, 0.55 * latch, 0.0)
    place(out, 0.4 * latch, 0.045)
    place(out, 0.3 * wood_tap(rng, n_of(0.08)), 0.05)
    c = creak(n_of(0.58), rng, 22, 60, [230, 560, 1150], q=16, jitter=0.35)
    c *= bell(len(c), 0.4, 0.9)
    c /= max(1e-9, np.abs(c).max())
    place(out, 0.8 * c, 0.11)
    return lowpass(highpass(out, 90), 7000)


def knock_down(rng, take):
    n = n_of(0.4)
    out = np.zeros(n)
    body = tone(n, sweep(n, 72, 42)) * decay(n, 0.09, 0.002)
    mass = lowpass(rng.standard_normal(n), 300) * decay(n, 0.06, 0.002)
    out += body + 0.9 * mass
    m = n_of(0.15)
    for at, a in [(0.075, 0.45), (0.14, 0.3)]:
        sub = lowpass(rng.standard_normal(m), 500) * decay(m, 0.025, 0.001)
        place(out, a * sub / np.abs(sub).max(), at)
    gear = bandpass(grains(n, 600, rng, decay(n, 0.08)), 2000, 5000)
    out += 0.25 * gear
    return lowpass(highpass(out, 35), 4000)


def fire_loop(rng, take):
    length = 4.0
    xf = 0.6
    n = n_of(length + xf)
    t = tvec(n)
    # low roar: slow-breathing brown-ish noise
    b = lowpass(rng.standard_normal(n), 1.5)
    breath = np.clip(0.8 + 0.2 * b / b.std(), 0.4, 1.2)
    roar = lowpass(np.cumsum(rng.standard_normal(n)) * 0.02, 350)
    roar = highpass(roar, 50) * breath
    hiss = bandpass(rng.standard_normal(n), 2500, 6000) * 0.02 * breath
    # crackles: rare short bright snaps, a few lower pops
    crack = grains(n, 14, rng) * rng.pareto(2.5, n)
    crack = highpass(resonate(crack, [1800, 3200, 5200], q=4), 900)
    pops = grains(n, 3, rng) * rng.uniform(0.5, 1.0, n)
    pops = resonate(pops, [380, 700], q=6)
    sig = roar / np.abs(roar).std() * 0.06 + hiss + 0.5 * crack / np.abs(crack).max() + 0.25 * pops / np.abs(pops).max()
    sig = lowpass(sig, 8000)
    # seamless loop: blend the tail past 4 s into the first xf seconds (equal power)
    L, X = n_of(length), n_of(xf)
    a = np.linspace(0, np.pi / 2, X)
    loop = sig[:L].copy()
    loop[:X] = sig[:X] * np.sin(a) + sig[L:L + X] * np.cos(a)
    return loop


# ---------------------------------------------------------------- catalogue

S = []  # (name, group, function, takes, description, recipe)


def add(name, group, fn, takes, desc, recipe):
    S.append((name, group, fn, takes, desc, recipe))


add("ui_hover", "interface", ui_hover, 1, "Softest tick as the mouse passes over a button.",
    "3 ms band-passed noise click plus a 2.3 kHz ping, very quiet.")
add("ui_press", "interface", ui_press, 3, "Button pressed: a soft woody/brass click.",
    "Noise click plus damped 680/1550 Hz wood modes and faint inharmonic brass partials, detuned per take.")
add("ui_release", "interface", ui_release, 1, "Button released: a lighter, higher click.",
    "The press recipe pitched up ~22%, shorter and quieter.")
add("ui_open", "interface", ui_open, 1, "Panel opening: a short soft paper/cloth swish.",
    "Noise through a band-pass sweeping 600 to 2600 Hz under a smooth hump.")
add("ui_close", "interface", ui_close, 1, "Panel closing: a shorter downward swish.",
    "Noise through a band-pass sweeping 2200 down to 650 Hz, front-weighted.")
add("ui_cant", "interface", ui_cant, 1, "Action not possible: two quick dull low thuds.",
    "Two pitch-dropping low sines (170 to 120 Hz) with low-passed noise puffs, 85 ms apart, low-passed at 1.2 kHz.")
add("ui_select", "interface", ui_select, 1, "Squad member selected: a small bright brass tink.",
    "Bell-like inharmonic partials on 1.9 kHz (x1, 2.32, 4.07) with a soft attack.")
add("ui_order", "interface", ui_order, 3, "Move order given: a soft low thup.",
    "Sine dropping 190 to 95 Hz plus a low-passed noise puff, detuned per take.")

add("talk_open", "screens", talk_open, 1, "Conversation opens: a very soft breath/rustle.",
    "Broad noise band sweeping 700 to 1300 Hz plus sparse rustle grains.")
add("coins", "screens", coins, 3, "Trade screen opens (trade_open): coins clinking onto wood.",
    "5-7 coin strikes (4 inharmonic modes each, random pitch) settling faster and quieter, over wooden taps.")
add("coin_single", "screens", coin_single, 1, "One coin set down.",
    "One coin strike on a wood tap with a small bounce.")
add("pack_open", "screens", pack_open, 1, "Pack opens: leather creak then a buckle clink.",
    "Friction grains through 380/860/1500 Hz resonators, then a small metal clink.")
add("craft_open", "screens", craft_open, 1, "Crafting opens: tools set on a bench.",
    "Four metal strikes at different pitches, each with a wooden thump.")
add("lid_open", "screens", lid_open, 1, "Chest lid creaking open.",
    "Stick-slip pulse train (35 to 90 per second) ringing wooden resonators at 420/950/1700 Hz.")
add("lid_close", "screens", lid_close, 1, "Wooden lid thump shut.",
    "Low sine thud (120 to 75 Hz), wood modes and a small rattle.")
add("take_item", "screens", take_item, 3, "Picking something up: a soft cloth/leather shuffle.",
    "Short swept noise band plus rustle grains, timing varied per take.")
add("page_turn", "screens", page_turn, 1, "A page turning (journal, spell book).",
    "Rising noise band with crinkle grains and a soft low flap at the end.")

add("step_dirt", "world", step_dirt, 3, "Footstep on earth.",
    "Heel and toe: low-passed noise thuds with gritty grains on top.")
add("step_stone", "world", step_stone, 3, "Footstep on stone.",
    "Heel and toe: sharp band-passed clicks with a small low body and scuff.")
add("swing", "world", swing, 3, "A weapon swishing through the air.",
    "Noise through a band-pass that rises to ~1.5 kHz and falls, under a bell envelope.")
add("hit_flesh", "world", hit_flesh, 3, "A blow landing: a muffled thud.",
    "Sine dropping 95 to 55 Hz, low-passed noise mass and a faint slap.")
add("hit_block", "world", hit_block, 3, "A blow blocked: a wood/metal knock.",
    "Wood modes (450/1120 Hz) plus quieter inharmonic metal partials and a click.")
add("bow_release", "world", bow_release, 1, "A bowstring twang.",
    "Karplus-Strong plucked string at 98 Hz, heavily damped, plus a falling swish and a wood knock.")
add("cast", "world", cast, 1, "A spell starting: a soft rising shimmer.",
    "Detuned sine partials gliding 420 to 640 Hz with vibrato, light FM and airy rising noise, swelling in.")
add("door_open", "world", door_open, 1, "A wooden door: latch, then a creak.",
    "Two metal latch clicks, then a slow stick-slip creak through 230/560/1150 Hz resonators.")
add("knock_down", "world", knock_down, 1, "Someone falling: a heavy body thud.",
    "Sine dropping 72 to 42 Hz with low noise mass, two smaller follow-up impacts and faint gear rattle.")
add("fire_loop", "loop", fire_loop, 1, "Quiet crackling fire, seamless 4 s loop.",
    "Breathing brown-noise roar, faint hiss, random crackles and pops; tail cross-faded into the head.")

GROUP_TITLE = {"interface": "Interface", "screens": "Screens", "world": "World", "loop": "World"}


# ---------------------------------------------------------------- finishing

def finish(x, peak_db, is_loop):
    x = np.asarray(x, float)
    if not is_loop:
        x = highpass(x, 25)          # remove any DC / sub rumble
        fi, fo = n_of(0.001), n_of(0.008)
        x[:fi] *= 0.5 - 0.5 * np.cos(np.linspace(0, np.pi, fi))
        x[-fo:] *= 0.5 + 0.5 * np.cos(np.linspace(0, np.pi, fo))
        x[0] = x[-1] = 0.0
    else:
        x = x - x.mean()
    target = 10 ** (peak_db / 20)
    x = x / np.abs(x).max() * target
    return x


def db(v):
    return 20 * np.log10(max(v, 1e-12))


def main():
    os.makedirs(os.path.join(PREVIEW, "wav"), exist_ok=True)
    os.makedirs(OUT, exist_ok=True)
    manifest = []
    for name, group, fn, takes, desc, recipe in S:
        for k in range(1, takes + 1):
            fname = f"{name}_{k}" if takes > 1 else name
            rng = rng_for(fname)
            raw = fn(rng, k)
            peak_db = GROUP_PEAK[group] + TRIM.get(name, 0.0)
            y = finish(raw, peak_db, group == "loop")
            pcm = np.clip(np.round(y * 32767), -32768, 32767).astype(np.int16)
            sf.write(os.path.join(PREVIEW, "wav", fname + ".wav"), pcm, SR, subtype="PCM_16")
            sf.write(os.path.join(OUT, fname + ".ogg"), y.astype(np.float32), SR,
                     format="OGG", subtype="VORBIS", compression_level=0.3)
            f = pcm.astype(float) / 32768
            manifest.append({
                "name": fname,
                "file": fname + ".ogg",
                "seconds": round(len(pcm) / SR, 4),
                "peak_db": round(db(np.abs(f).max()), 2),
                "rms_db": round(db(np.sqrt(np.mean(f ** 2))), 2),
                "group": GROUP_TITLE[group].lower(),
                "description": desc,
                "recipe": recipe,
            })
    with open(os.path.join(OUT, "manifest.json"), "w") as fh:
        json.dump(manifest, fh, indent=2)
    write_html(manifest)
    print(f"wrote {len(manifest)} sounds")


def write_html(manifest):
    import html
    groups = ["interface", "screens", "world"]
    parts = []
    for g in groups:
        rows = []
        for m in manifest:
            if m["group"] != g:
                continue
            rows.append(
                f'<div class="row"><div class="info"><div class="name">{html.escape(m["name"])}</div>'
                f'<div class="desc">{html.escape(m["description"])}</div>'
                f'<div class="meta">{m["seconds"]:.3f} s &middot; peak {m["peak_db"]:.1f} dBFS'
                f'{" &middot; loops" if m["name"] == "fire_loop" else ""}</div></div>'
                f'<audio controls preload="none"{" loop" if m["name"] == "fire_loop" else ""} '
                f'src="wav/{html.escape(m["name"])}.wav"></audio></div>')
        parts.append(f'<section><h2>{g.capitalize()}</h2>{"".join(rows)}</section>')
    page = f"""<!doctype html>
<html lang="en"><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Gahturiyu Sounds</title>
<style>
:root {{ --bg:#1c1510; --panel:#2a2018; --text:#e8dcc4; --brass:#c9a35a; --dim:#a8987c; }}
* {{ box-sizing:border-box; }}
body {{ margin:0; background:var(--bg); color:var(--text);
  font-family:"Alegreya", Georgia, serif; font-size:17px; line-height:1.4; }}
main {{ max-width:860px; margin:0 auto; padding:28px 16px 60px; }}
h1 {{ color:var(--brass); font-weight:normal; letter-spacing:.06em; margin:0 0 4px; font-size:30px; }}
p.lede {{ color:var(--dim); margin:0 0 24px; }}
h2 {{ color:var(--brass); font-weight:normal; letter-spacing:.08em; text-transform:uppercase;
  font-size:16px; border-bottom:1px solid #5a4630; padding-bottom:6px; margin:28px 0 10px; }}
.row {{ background:var(--panel); border:1px solid #3d2f22; border-radius:4px; padding:10px 14px;
  margin:8px 0; display:flex; gap:14px; align-items:center; flex-wrap:wrap; }}
.info {{ flex:1 1 300px; min-width:0; }}
.name {{ color:var(--brass); font-variant:small-caps; letter-spacing:.04em; font-size:18px; }}
.desc {{ font-size:15px; }}
.meta {{ font-size:13px; color:var(--dim); }}
audio {{ flex:0 1 300px; width:300px; max-width:100%; height:36px; }}
</style></head><body><main>
<h1>Gahturiyu Sim &mdash; placeholder sounds</h1>
<p class="lede">Synthesised by us in <code>make_sounds.py</code>; no recordings, no third-party licence.
The game uses the .ogg files; these previews play the .wav copies.</p>
{"".join(parts)}
</main></body></html>
"""
    with open(os.path.join(PREVIEW, "listen.html"), "w") as fh:
        fh.write(page)


if __name__ == "__main__":
    main()
