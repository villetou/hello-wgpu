// shader.vert
#version 450

layout(location=0) in vec3 a_position;
layout(location=5) in mat4 model_matrix;
layout(location=9) in uint frame;

layout(location=0) out vec2 v_tex_coords;

layout(set=1, binding=0)
uniform Uniforms {
    mat4 u_view_proj;
    vec4 sprite_coordinates[24];
};

void main() {
    vec2 tex_coords = vec2(0, 0);

    // Sort with Z
    mat4 y_to_inverse_z = mat4(1.0);
    y_to_inverse_z[2][1] = -0.01;

    switch(gl_VertexIndex) {
        case 0:
            tex_coords = vec2(sprite_coordinates[frame].x, sprite_coordinates[frame].w);
            break;
        case 1:
            tex_coords = vec2(sprite_coordinates[frame].z, sprite_coordinates[frame].y);
            break;
        case 2:
            tex_coords = vec2(sprite_coordinates[frame].x, sprite_coordinates[frame].y);
            break;
        case 3:
            tex_coords = vec2(sprite_coordinates[frame].z, sprite_coordinates[frame].w);
            break;
    }

    v_tex_coords = tex_coords;
    gl_Position = u_view_proj * y_to_inverse_z * model_matrix * vec4(a_position, 1.0);
}
