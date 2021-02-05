use m::{ff, fi, Fx};
use na::{ComplexField, RealField};

pub type LerpFunction = fn(progress: Fx) -> Fx;

fn lerp_constant(_progress: Fx) -> Fx {
    return fi(1);
}

fn lerp_linear(progress: Fx) -> Fx {
    return progress;
}

// x ^ 2

fn lerp_quad_in(progress: Fx) -> Fx {
    return progress * progress;
}

fn lerp_quad_out(progress: Fx) -> Fx {
    let progress = fi(1) - progress;
    return fi(1) - progress * progress;
}

fn lerp_quad_inout(progress: Fx) -> Fx {
    if progress < ff(0.5) {
        return fi(2) * progress * progress;
    } else {
        let progress = fi(1) - progress;
        return fi(1) - fi(2) * progress * progress;
    }
}

// x ^ 3

fn lerp_cubic_in(progress: Fx) -> Fx {
    return progress * progress * progress;
}

fn lerp_cubic_out(progress: Fx) -> Fx {
    let progress = fi(1) - progress;
    return fi(1) - progress * progress * progress;
}

fn lerp_cubic_inout(progress: Fx) -> Fx {
    if progress < ff(0.5) {
        return fi(4) * progress * progress * progress;
    } else {
        let progress = fi(1) - progress;
        return fi(1) - fi(4) * progress * progress * progress;
    }
}

// x ^ 4

fn lerp_quart_in(progress: Fx) -> Fx {
    return progress * progress * progress * progress;
}

fn lerp_quart_out(progress: Fx) -> Fx {
    let progress = fi(1) - progress;
    return fi(1) - progress * progress * progress * progress;
}

fn lerp_quart_inout(progress: Fx) -> Fx {
    if progress < ff(0.5) {
        return fi(8) * progress * progress * progress * progress;
    } else {
        let progress = fi(1) - progress;
        return fi(1) - fi(8) * progress * progress * progress * progress;
    }
}

// sin(x)

fn lerp_sin_in(progress: Fx) -> Fx {
    return fi(1) - Fx::cos(progress * Fx::frac_pi_2());
}

fn lerp_sin_out(progress: Fx) -> Fx {
    return Fx::sin(progress * Fx::frac_pi_2());
}

fn lerp_sin_inout(progress: Fx) -> Fx {
    return ff(0.5) - ff(0.5) * Fx::cos(Fx::pi() * progress);
}

// 2 ^ 10x

fn lerp_expo_in(progress: Fx) -> Fx {
    if progress == fi(0) {
        return fi(0);
    } else {
        return Fx::powf(fi(2), fi(10) * progress - fi(10));
    }
}

fn lerp_expo_out(progress: Fx) -> Fx {
    if progress == fi(1) {
        return fi(1);
    } else {
        let progress = fi(1) - progress;
        return fi(1) - Fx::powf(fi(2), fi(10) * progress - fi(10));
    }
}

fn lerp_expo_inout(progress: Fx) -> Fx {
    if progress < ff(0.5) {
        if progress == fi(0) {
            return fi(0);
        } else {
            let progress = fi(2) * progress;
            return ff(0.5) * Fx::powf(fi(2), fi(10) * progress - fi(10));
        }
    } else {
        if progress == fi(1) {
            return fi(1);
        } else {
            let progress = fi(2) - fi(2) * progress;
            return fi(1) - ff(0.5) * Fx::powf(fi(2), fi(10) * progress - fi(10));
        }
    }
}
