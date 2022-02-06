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

struct Node {
    struct Vec2 position;
    struct Vec2 velocity;
    struct Vec2 last_acceleration;
    struct Vec2 current_acceleration;
    float mass;
};

KERNEL void mainkernel(uint num, GLOBAL struct Node *nodes, uint iterations) {
    size_t index = get_global_id(0);

    for (uint i = 0; i < iterations; ++i) {
        for (uint i = 0; i < num; ++i) {
            nodes[i].position.y -= 0.001;
        }
        barrier(CLK_LOCAL_MEM_FENCE | CLK_GLOBAL_MEM_FENCE);
    }
}

KERNEL void add(uint num, GLOBAL uint *a, GLOBAL uint *b, GLOBAL uint *result) {
    for (uint i = 0; i < num; i++) {
        result[i] = a[i] + b[i];
    }
}