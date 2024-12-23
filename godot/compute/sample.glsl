#[compute]
#version 450

// Invocations in the (x, y, z) dimension
layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

// A binding to the buffer we create in our script
layout(set = 0, binding = 0, std430) restrict buffer SphereBuffer {
    float vertices[];
} body;

layout(set = 0, binding = 1, std430) restrict buffer Params {
    uint vert_count;
    float seed;
} params;

#include "noise.glsl"

vec3 update(vec3 vert) {
    float lvl = 1.;

    float freq = 1.;
    float impact = 1. / 30.;

    float freq_falloff = 1.5;
    float impact_falloff = 0.6;

    for (int i = 0; i < 10; i++) {
        lvl += snoise(vert * freq) * impact;

        freq *= freq_falloff;
        impact *= impact_falloff;
    }

    lvl += -pow((ridge_noise(vert * 1.4) + 1.) / 2. * 0.02, 2);

    return vert * lvl;
}

// The code we want to execute in each invocation
void main() {
    const uint index = gl_GlobalInvocationID.x;

    if (index >= params.vert_count) {
        return;
    }

    vec4 vert;
    vert.x = body.vertices[index * 3];
    vert.y = body.vertices[index * 3 + 1];
    vert.z = body.vertices[index * 3 + 2];

    vert.xyz = update(vert.xyz);

    body.vertices[index * 3] = vert.x;
    body.vertices[index * 3 + 1] = vert.y;
    body.vertices[index * 3 + 2] = vert.z;
}
