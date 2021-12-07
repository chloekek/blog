#version 450 core

// This specialization constant is overridden during pipeline construction.
// GLSL requires a default value and I picked 1 arbitrarily.
layout(constant_id = 0) const uint BONES = 1;

layout(location = 0) uniform mat4 mvp_matrix;
layout(location = 1) uniform mat4 bone_matrices[BONES];

layout(location = 0) in vec3 vertex_position;
layout(location = 1) in vec2 vertex_texcoord;
layout(location = 2) in uint vertex_bone;

layout(location = 0) out vec2 fragment_uv;

void main()
{
    gl_Position =
        mvp_matrix *
        bone_matrices[vertex_bone] *
        vec4(vertex_position, 1.0);

    // Pass through texture coordinate.
    fragment_uv = vertex_texcoord;
}
