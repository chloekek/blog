#version 450 core

/// See [`TrivialBlockPipeline::render`].
layout(location = 1) uniform float atlas_len;
layout(location = 2) uniform mat4 mvp_matrix;

/// See [`TrivialBlockFace`].
layout(location = 0) in uint face_xy;
layout(location = 1) in uint face_zf;
layout(location = 2) in uint face_u;
layout(location = 3) in uint face_v;

/// Normalized U and V coordinates in the texture atlas.
out vec2 fragment_uv;

/// For each of the six faces of the cube,
/// this specifies the coordinate of each corner
/// relative to the center of the face.
const vec3 corner_positions[6 * 4] = {
    // East face.
    vec3(+0.5, +0.5, +0.5),
    vec3(+0.5, -0.5, +0.5),
    vec3(+0.5, -0.5, -0.5),
    vec3(+0.5, +0.5, -0.5),
    // North face.
    vec3(-0.5, +0.5, +0.5),
    vec3(+0.5, +0.5, +0.5),
    vec3(+0.5, +0.5, -0.5),
    vec3(-0.5, +0.5, -0.5),
    // West face.
    vec3(-0.5, +0.5, +0.5),
    vec3(-0.5, +0.5, -0.5),
    vec3(-0.5, -0.5, -0.5),
    vec3(-0.5, -0.5, +0.5),
    // South face.
    vec3(-0.5, -0.5, +0.5),
    vec3(-0.5, -0.5, -0.5),
    vec3(+0.5, -0.5, -0.5),
    vec3(+0.5, -0.5, +0.5),
    // Top face.
    vec3(-0.5, +0.5, +0.5),
    vec3(-0.5, -0.5, +0.5),
    vec3(+0.5, -0.5, +0.5),
    vec3(+0.5, +0.5, +0.5),
    // Bottom face.
    vec3(-0.5, +0.5, -0.5),
    vec3(+0.5, +0.5, -0.5),
    vec3(+0.5, -0.5, -0.5),
    vec3(-0.5, -0.5, -0.5),
};

/// This specifies the offset to be applied
/// to the U and V coordinates for each corner.
const vec2 uv_per_corner[4] = {
    vec2(0.0, 0.0),
    vec2(0.0, 1.0),
    vec2(1.0, 1.0),
    vec2(1.0, 0.0),
};

void main()
{
    // Some attributes are packed into four bits
    // because they only ever range from 0 through 15.
    // We unpack them into integer variables here.
    int  face_x = int(face_xy >> 4);
    int  face_y = int(face_xy & 0xFu);
    int  face_z = int(face_zf >> 4);
    uint face_f = face_zf & 0xFu;

    // The corner position depends on the face position
    // and which of the four corners we are processing.
    vec3 center = vec3(face_x, face_y, face_z);
    vec3 corner = corner_positions[4 * face_f + gl_VertexID];
    gl_Position = mvp_matrix * vec4(center + corner, 1.0);

    // The texture coordinates also depend on which corner we are processing.
    // Furthermore, they need to be normalized into the interval [0, 1]
    // so we divide them by the number of textures in the texture atlas.
    vec2 face_uv = vec2(face_u, face_v);
    fragment_uv = (face_uv + uv_per_corner[gl_VertexID]) / texture_atlas_size;
}
