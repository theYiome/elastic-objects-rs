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
    float drag;
    uint object_id;
    bool is_boundary;
};

KERNEL void main(uint node_count, GLOBAL struct Node *nodes, uint collisions_count, GLOBAL ulong *collisions, GLOBAL ulong *collisions_index, uint dt_div, GLOBAL float2 *result) {
    size_t index = get_global_id(0);

    float dt = 0.0;
    if (dt_div != 0)
        dt = 1.0 / dt_div;

    float v = 10000000.0f * dt;
    result[index] = (float2)(2.1f * v, 3.2f * v);
}