// CUDA
#ifdef __CUDACC__
    #define GLOBAL
    #define KERNEL extern "C" __global__
// OpenCL
#else
    #define GLOBAL __global
    #define KERNEL __kernel
#endif

struct Node {
    float2 position;
    float2 velocity;
    float2 last_acceleration;
    float2 current_acceleration;
    float mass;
};

KERNEL void mainkernel(uint num, GLOBAL struct Node *nodes, uint iterations, uint dt_div) {
    size_t index = get_global_id(0);

    float dt = 0.0;
    if (dt_div != 0)
        dt = 1.0 / dt_div;

    for (uint i = 0; i < iterations; ++i) {
        // nodes[index].position.x += 0.0f * dt;
        nodes[index].position.y += -9.0f * dt;
        // // start_integrate_velocity_verlet
        // {
        //     nodes[index].position += (nodes[index].velocity * dt) + (0.5f * nodes[index].current_acceleration * dt * dt);
        //     nodes[index].last_acceleration = nodes[index].current_acceleration;
        //     nodes[index].current_acceleration = (0.0f, 0.0f);
        // }
        // barrier(CLK_LOCAL_MEM_FENCE | CLK_GLOBAL_MEM_FENCE);

        // wall_repulsion_force_y
        // {
        //     float v0 = 200.0;
        //     float dx = 0.05;
        //     float2 dir = (float2)(nodes[index].position.x, -1.05f) - nodes[index].position;
        //     float l = length(dir);
        //     float mi = nodes[index].mass;

        //     float c = pown(dx / l, 13);
        //     float2 v = normalize(dir) * 3.0f * (v0 / dx) * c;

        //     nodes[index].current_acceleration -= v / mi;
        // }
        // barrier(CLK_LOCAL_MEM_FENCE | CLK_GLOBAL_MEM_FENCE);

        // // end_integrate_velocity_verlet
        // {
        //     nodes[index].velocity += 0.5f * (nodes[index].last_acceleration + nodes[index].current_acceleration) * dt;
        // }
        barrier(CLK_LOCAL_MEM_FENCE | CLK_GLOBAL_MEM_FENCE);
    }
}

KERNEL void add(uint num, GLOBAL uint *a, GLOBAL uint *b, GLOBAL uint *result) {
    for (uint i = 0; i < num; i++) {
        result[i] = a[i] + b[i];
    }
}