layout(std140) struct Camera {
  mat4 view_proj;
};

layout(std140) struct Player {
  vec3 pos, speed;
};

layout(std140) struct Light {
  vec3 pos;
  vec3 color;
  float intensity;
};
