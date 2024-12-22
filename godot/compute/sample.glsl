#[compute]
#version 450

// Invocations in the (x, y, z) dimension
layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in;

// A binding to the buffer we create in our script
layout(set = 0, binding = 0, std430) buffer SphereBuffer {
    // uint total_verts;
    vec3 vertices[];
} body_buffer;

// The code we want to execute in each invocation
void main() {
    const uint index = gl_GlobalInvocationID.x;

    // if (index >= body_buffer.total_verts) {
    //     return;
    // }

    vec3 vert = body_buffer.vertices[index];

    body_buffer.vertices[index] = vert * 1.;
}
