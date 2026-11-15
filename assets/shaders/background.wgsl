struct Parameters {
    screen: vec4f,
    motion: vec4f,
    detail: vec4f,
    color0: vec4f,
    color1: vec4f,
    color2: vec4f,
    finish: vec4f,
};
@group(0) @binding(0) var<uniform> u: Parameters;
struct Vertex { @builtin(position) position: vec4f, @location(0) uv: vec2f };
@vertex fn vs_main(@builtin(vertex_index) index: u32) -> Vertex {
    let p = vec2f(f32((index << 1u) & 2u), f32(index & 2u));
    var out: Vertex;
    out.position = vec4f(p * vec2f(2.0, -2.0) + vec2f(-1.0, 1.0), 0.0, 1.0);
    out.uv = p;
    return out;
}
fn hash(p: vec2f) -> f32 { return fract(sin(dot(p, vec2f(127.1, 311.7)) + u.detail.z) * 43758.5453); }
fn noise(p: vec2f) -> f32 {
    let i = floor(p); let f = fract(p); let s = f * f * (3.0 - 2.0 * f);
    return mix(mix(hash(i), hash(i + vec2f(1.0, 0.0)), s.x), mix(hash(i + vec2f(0.0, 1.0)), hash(i + vec2f(1.0, 1.0)), s.x), s.y);
}
fn fbm(p0: vec2f) -> f32 {
    var p = p0; var amplitude = 0.5; var value = 0.0;
    for (var i = 0; i < 6; i++) {
        if f32(i) < clamp(u.detail.x * 0.6 + 1.0, 2.0, 6.0) { value += noise(p) * amplitude; }
        p = p * 2.03 + vec2f(17.4, 8.2); amplitude *= 0.5;
    }
    return value;
}
fn palette(a: f32, b: f32) -> vec3f { return mix(mix(u.color0.rgb, u.color1.rgb, clamp(a, 0.0, 1.0)), u.color2.rgb, clamp(b, 0.0, 1.0)); }
@fragment fn fs_main(in: Vertex) -> @location(0) vec4f {
    let uv = in.uv;
    let aspect = u.screen.x / max(u.screen.y, 1.0);
    let t = u.screen.z * u.motion.x;
    let angle = u.detail.y;
    let point = (uv - 0.5) * vec2f(aspect, 1.0);
    let rotated = vec2f(cos(angle) * point.x - sin(angle) * point.y, sin(angle) * point.x + cos(angle) * point.y);
    let p = rotated * u.motion.y;
    let soft = max(u.motion.w, 0.04);
    let kind = i32(u.screen.w);
    var c = u.color0.rgb;
    if kind == 0 {
        let bend = fbm(vec2f(p.x * 1.8 + t * 0.15, t * 0.08)) - 0.5;
        let ribbon = exp(-abs(p.y + bend * 0.55 + sin(p.x * 2.0 + t) * 0.15) / (0.025 + soft * 0.22));
        let curtain = 0.6 + 0.4 * sin(p.x * 8.0 + fbm(p + t * 0.1) * 6.0);
        c = palette(ribbon * curtain, ribbon * (0.5 + 0.5 * sin(p.x * 1.4 - t * 0.3)) * 0.8);
    } else if kind == 1 {
        let n = fbm(p * 3.5 + vec2f(t * 0.08, -t * 0.04));
        c = palette(smoothstep(0.15, 0.4 + soft * 0.6, n), smoothstep(0.4, 0.9, fbm(p * 2.0 - t * 0.04)) * 0.7);
        let cell = floor(uv * (70.0 + u.detail.x * 14.0));
        let star = pow(max(0.0, 1.0 - length(fract(uv * (70.0 + u.detail.x * 14.0)) - 0.5) * 3.0), 12.0);
        c += vec3f(star * step(0.97, hash(cell)) * (0.6 + 0.4 * sin(t + hash(cell) * 20.0)));
    } else if kind == 2 {
        let a = sin(p.x * (4.0 + u.detail.x * 0.5) + t + sin(p.y * 3.0 - t * 0.5));
        let b = sin(p.y * (3.0 + u.detail.x * 0.5) - t * 0.6 + p.x * 1.5);
        c = palette(smoothstep(-soft, soft + 0.6, a + p.y), smoothstep(0.2, 1.3, b + a * 0.4) * 0.65);
    } else if kind == 3 {
        let warp = sin(p.x * 2.4 + t * 0.5) * 0.8 + fbm(p + t * 0.12) * 0.8;
        let silk = pow(0.5 + 0.5 * sin(p.y * (8.0 + u.detail.x * 2.0) + warp * 5.0), 1.2 + (1.0 - soft) * 3.0);
        c = palette(silk * 0.9, smoothstep(0.55, 1.0, silk) * 0.7);
    } else if kind == 4 || kind == 8 {
        c = mix(u.color0.rgb, u.color1.rgb, uv.y * 0.15);
        for (var i = 0; i < 64; i++) {
            if f32(i) < 8.0 + u.detail.x * 5.0 {
                let fi = f32(i);
                let origin = vec2f(hash(vec2f(fi, 1.0)), hash(vec2f(fi, 4.0)));
                let position = vec2f(origin.x + sin(t * 0.2 + fi) * 0.08, fract(origin.y - t * (0.025 + origin.x * 0.025)));
                let distance = length((uv - position) * vec2f(aspect, 1.0));
                let radius = select(0.008 + soft * 0.018, (0.03 + origin.x * 0.08) * (0.5 + soft), kind == 8) / u.motion.y;
                let glow = exp(-distance * distance / max(radius * radius, 0.00001));
                c += mix(u.color1.rgb, u.color2.rgb, origin.x) * glow * select(0.75, 0.3, kind == 8);
            }
        }
    } else if kind == 5 {
        let radius = length(p + vec2f(sin(t * 0.2), cos(t * 0.15)) * 0.12);
        let ring = 0.5 + 0.5 * sin(radius * (12.0 + u.detail.x * 2.5) - t * 1.4 + fbm(p * 2.0) * soft * 2.0);
        c = palette(pow(ring, 1.5), smoothstep(0.7, 1.0, ring) * (0.5 + 0.5 * sin(radius * 3.0 + t * 0.1)));
    } else if kind == 6 {
        let warp = vec2f(fbm(p * 2.0 + vec2f(t * 0.1, 1.0)), fbm(p * 2.0 - vec2f(2.0, t * 0.1)));
        let flow = fbm(p * 3.0 + warp * (2.0 + soft * 3.0) + t * 0.05);
        c = palette(smoothstep(0.15, 0.8, flow), smoothstep(0.4, 0.9, warp.x + flow * 0.25));
    } else if kind == 7 {
        let grid = p * (5.0 + u.detail.x) + vec2f(t * 0.12, t * 0.08);
        let cell = abs(fract(grid) - 0.5);
        let lines = 1.0 - smoothstep(0.01, 0.02 + soft * 0.12, min(cell.x, cell.y));
        let pulse = 0.55 + 0.45 * sin(length(p) * 5.0 - t);
        c = palette(lines * pulse, lines * (0.5 + 0.5 * sin(p.x + p.y + t * 0.3)) * 0.7);
    } else if kind == 9 {
        let q = p * (1.2 + u.detail.x * 0.3);
        let v1 = sin(q.x * 1.5 + t * 0.25) + cos(q.y * 1.8 - t * 0.2);
        let v2 = sin(q.y * 1.8 + t * 0.18 + v1 * 0.7) + cos(q.x * 1.6 - t * 0.15);
        let warp = vec2f(
            sin(q.x * 1.2 + v2 * 0.7 + t * 0.12),
            cos(q.y * 1.2 + v1 * 0.7 - t * 0.14)
        );
        let flow1 = 0.5 + 0.5 * sin(length(q + warp) * 2.2 - t * 0.3);
        let flow2 = 0.5 + 0.5 * cos(warp.x * 2.4 + warp.y * 1.8 + t * 0.22);
        let blend_a = smoothstep(0.08, 0.92, mix(flow1, flow2, 0.5 + 0.5 * sin(t * 0.15)));
        let blend_b = smoothstep(0.18, 0.82, 0.5 + 0.5 * sin(q.x * 1.4 - q.y * 1.2 + warp.x * 1.8));
        c = palette(blend_a, blend_b * (0.6 + soft * 0.4));
    } else if kind == 10 {
        let ray_origin = vec2f(point.x, point.y + 0.7);
        let ray_dir = normalize(ray_origin);
        let ray_angle = atan2(ray_dir.y, ray_dir.x);
        let ray_dist = length(ray_origin);
        let beams = 0.5 + 0.5 * sin(ray_angle * (14.0 + u.detail.x * 3.0) + t * 0.6 + sin(ray_angle * 5.0 - t * 0.3));
        let dust = fbm(vec2f(ray_angle * 3.0, ray_dist * 2.0 - t * 0.15));
        let intensity = pow(beams, 1.8) * exp(-ray_dist * (0.7 + (1.0 - soft) * 0.8)) * (0.7 + 0.3 * dust);
        c = palette(intensity * 0.85, smoothstep(0.4, 0.9, intensity) * 0.75);
    } else if kind == 11 {
        c = mix(u.color0.rgb, u.color1.rgb, 0.12 + 0.15 * sin(p.y * 1.2 + t * 0.05));
        let nebula_dust = fbm(p * 2.2 + vec2f(t * 0.03, -t * 0.02));
        c += u.color1.rgb * smoothstep(0.35, 0.75, nebula_dust) * 0.45;
        for (var i = 0; i < 48; i++) {
            let fi = f32(i);
            let h1 = hash(vec2f(fi * 13.1 + 1.7, fi * 37.7 + 5.3));
            let h2 = hash(vec2f(fi * 59.3 + 9.1, fi * 17.9 + 3.7));
            let star_pos = vec2f(fract(h1 + t * 0.006 * (0.4 + h2)), fract(h2 - t * 0.004 * (0.4 + h1)));
            let d = length((uv - star_pos) * vec2f(aspect, 1.0));
            let twinkle = 0.4 + 0.6 * sin(t * (1.2 + h1 * 2.5) + h2 * 30.0);
            let size = (0.002 + h1 * 0.0035) * (0.6 + soft * 0.5);
            let brightness = exp(-d * d / max(size * size * 0.45, 0.000001)) * twinkle * step(0.12, h2);
            c += mix(u.color1.rgb, u.color2.rgb, h1) * brightness * 0.85;
        }
        for (var j = 0; j < 8; j++) {
            let fj = f32(j);
            let h3 = hash(vec2f(fj * 71.3 + 4.1, fj * 93.1 + 8.7));
            let h4 = hash(vec2f(fj * 29.7 + 2.3, fj * 83.5 + 6.9));
            let flare_pos = vec2f(fract(h3 + sin(t * 0.03 + fj) * 0.05), fract(h4 + cos(t * 0.025 + fj) * 0.05));
            let df = length((uv - flare_pos) * vec2f(aspect, 1.0));
            let flare_glow = exp(-df * (12.0 + (1.0 - soft) * 10.0)) * (0.6 + 0.4 * sin(t * 0.8 + fj * 4.0));
            c += u.color2.rgb * flare_glow * 0.6;
        }
    } else if kind == 12 {
        let r = length(point) * (3.5 + u.detail.x * 0.8);
        let pulse_wave = sin(r - t * 1.2 + fbm(rotated * 3.0) * soft * 2.0);
        let core = exp(-length(point) * (2.8 - soft * 1.2));
        let ring_wave = smoothstep(0.0, 0.8, pulse_wave) * exp(-r * 0.28);
        c = palette(core * 0.9 + ring_wave * 0.65, (core + ring_wave) * (0.5 + 0.5 * sin(t * 0.5)));
    } else {
        c = palette(0.5, 0.5);
    }
    c *= u.motion.z;
    c *= 1.0 - clamp(length((uv - 0.5) * 1.3), 0.0, 1.0) * u.detail.w;
    c += (hash(uv * u.screen.xy + floor(t * 12.0)) - 0.5) * u.finish.x;
    let r = min(u.finish.y, min(u.screen.x, u.screen.y) * 0.5);
    let q = abs((uv - 0.5) * u.screen.xy) - u.screen.xy * 0.5 + r;
    let distance = length(max(q, vec2f(0.0))) + min(max(q.x, q.y), 0.0) - r;
    let alpha = (1.0 - smoothstep(-0.7, 0.7, distance)) * u.finish.z;
    return vec4f(clamp(c, vec3f(0.0), vec3f(1.0)) * alpha, alpha);
}
