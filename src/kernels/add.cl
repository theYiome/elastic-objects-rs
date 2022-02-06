// CUDA
#ifdef __CUDACC__
    #define GLOBAL
    #define KERNEL extern "C" __global__
// OpenCL
#else
    #define GLOBAL __global
    #define KERNEL __kernel
#endif

struct Vec2 {
    float x;
    float y;
};

float len(struct Vec2 v) {
    return sqrt(v.x * v.x + v.y * v.y);
};

struct Vec2 norm(struct Vec2 v) {
    float l = len(v);
    v.x = v.x / l;
    v.y = v.y / l;
    return v;
};

struct Node {
    struct Vec2 position;
    struct Vec2 velocity;
    struct Vec2 last_acceleration;
    struct Vec2 current_acceleration;
    float mass;
};

KERNEL void mainkernel(uint num, GLOBAL struct Node *nodes, uint iterations, uint dt_div) {
    size_t index = get_global_id(0);

    float dt = 0.0;
    if (dt_div != 0)
        dt = 1.0 / dt_div;

    for (uint i = 0; i < iterations; ++i) {
        // start_integrate_velocity_verlet
        {
            nodes[index].position.x += (nodes[index].velocity.x) * dt + (0.5f * nodes[index].current_acceleration.x * dt * dt);
            nodes[index].position.y += (nodes[index].velocity.y) * dt + (0.5f * nodes[index].current_acceleration.y * dt * dt);
            nodes[index].last_acceleration.x = nodes[index].current_acceleration.x;
            nodes[index].last_acceleration.y = nodes[index].current_acceleration.y;
            nodes[index].current_acceleration.x = 0.0f;
            nodes[index].current_acceleration.y = 0.0f;
        }
        barrier(CLK_LOCAL_MEM_FENCE | CLK_GLOBAL_MEM_FENCE);

        // gravity_force
        {
            nodes[index].current_acceleration.y += -9.81f;
        }
        barrier(CLK_LOCAL_MEM_FENCE | CLK_GLOBAL_MEM_FENCE);

        // wall_repulsion_force_y
        {
            float v0 = 200.0f;
            float dx = 0.05f;

            struct Vec2 dir;
            dir.x = nodes[index].position.x - nodes[index].position.x;
            dir.y = -1.05f - nodes[index].position.y;
            
            float l = len(dir);
            float mi = nodes[index].mass;
            float c = pown(dx / l, 13);
            float k = 3.0f * (v0 / dx) * c / mi;

            struct Vec2 v = norm(dir);
            v.x *= k;
            v.y *= k;

            nodes[index].current_acceleration.x -= v.x;
            nodes[index].current_acceleration.y -= v.y;
        }
        barrier(CLK_LOCAL_MEM_FENCE | CLK_GLOBAL_MEM_FENCE);

        // end_integrate_velocity_verlet
        {
            nodes[index].velocity.x += 0.5f * (nodes[index].last_acceleration.x + nodes[index].current_acceleration.x) * dt;
            nodes[index].velocity.y += 0.5f * (nodes[index].last_acceleration.y + nodes[index].current_acceleration.y) * dt;
        }
        barrier(CLK_LOCAL_MEM_FENCE | CLK_GLOBAL_MEM_FENCE);
    }
}

KERNEL void add(uint num, GLOBAL uint *a, GLOBAL uint *b, GLOBAL uint *result) {
    for (uint i = 0; i < num; i++) {
        result[i] = a[i] + b[i];
    }
}