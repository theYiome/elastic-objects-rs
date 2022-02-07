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
    float2 position;
    float2 velocity;
    float2 last_acceleration;
    float2 current_acceleration;
    float mass;
    float damping;
};

KERNEL void mainkernel(uint node_count, GLOBAL struct Node *nodes, uint connections_count, GLOBAL uint2 *connections_keys, GLOBAL float2 *connections_vals, uint iterations, uint dt_div) {
    size_t index = get_global_id(0);

    float dt = 0.0;
    if (dt_div != 0)
        dt = 1.0 / dt_div;

    for (uint i = 0; i < iterations; ++i) {
        // start_integrate_velocity_verlet
        {
            nodes[index].position += (nodes[index].velocity) * dt + (0.5f * nodes[index].current_acceleration * dt * dt);
            nodes[index].last_acceleration = nodes[index].current_acceleration;
            nodes[index].current_acceleration *= 0.0f;
        }
        barrier(CLK_LOCAL_MEM_FENCE | CLK_GLOBAL_MEM_FENCE);

        // gravity_force
        {
            nodes[index].current_acceleration.y += -9.81f;
        }
        // barrier(CLK_LOCAL_MEM_FENCE | CLK_GLOBAL_MEM_FENCE);

        // wall_repulsion_force_y
        {
            float v0 = 200.0f;
            float dx = 0.05f;

            float2 dir = (0.0, -1.0f - nodes[index].position.y);

            float l = fast_length(dir);
            float mi = nodes[index].mass;
            float c = pown(dx / l, 13);
            float k = 3.0f * (v0 / dx) * c / mi;
            nodes[index].current_acceleration -= fast_normalize(dir) * k;
        }
        barrier(CLK_LOCAL_MEM_FENCE | CLK_GLOBAL_MEM_FENCE);


        for (uint c_i = index; c_i < connections_count; c_i += node_count) {
            uint i = connections_keys[c_i].x;
            uint j = connections_keys[c_i].y;
            float dx = connections_vals[c_i].x;
            float v0 = connections_vals[c_i].y;


            float2 dir = nodes[j].position - nodes[i].position;
            
            float l = fast_length(dir);

            float mi = nodes[i].mass;
            float mj = nodes[j].mass;
            float c = pown(dx / l, 7) - pown(dx / l, 13);
            float k = 3.0f * (v0 / dx) * c;
            float2 v = fast_normalize(dir) * k;

            // printf("i: %u, j %u, dx: %f, v0: %f, k: %f, mi: %f, mj: %f\n", i, j, dx, v0, k, mi, mj);

            barrier(CLK_LOCAL_MEM_FENCE | CLK_GLOBAL_MEM_FENCE);
            nodes[i].current_acceleration += v / mi;
            barrier(CLK_LOCAL_MEM_FENCE | CLK_GLOBAL_MEM_FENCE);
            nodes[j].current_acceleration -= v / mj;
        }
        barrier(CLK_LOCAL_MEM_FENCE | CLK_GLOBAL_MEM_FENCE);

        // repulsion_force_simple
        // {
        //     float v0 = 500.0f;
        //     float dx = 0.1f;

        //     for (uint i = 0; i < node_count; ++i) {
        //         if (index != i) {
        //             float2 dir = nodes[i].position - nodes[index].position;
                    
        //             float l = fast_length(dir);
        //             float mi = nodes[index].mass;
        //             float c = pown(dx / l, 13);
        //             float k = 3.0f * (v0 / dx) * c / mi;
        //             nodes[index].current_acceleration -= fast_normalize(dir) * k;
        //         }
        //     }
        // }

        // attraction_force_simple
        // {
        //     float v0 = 500.0f;
        //     float dx = 0.1f;

        //     for (uint i = 0; i < node_count; ++i) {
        //         if (index != i) {
        //             float2 dir = nodes[i].position - nodes[index].position;
                    
        //             float l = fast_length(dir);
        //             float mi = nodes[index].mass;
        //             float c = pown(dx / l, 7);
        //             float k = 3.0f * (v0 / dx) * c / mi;
        //             nodes[index].current_acceleration += fast_normalize(dir) * k;
        //         }
        //     }
        // }
        // barrier(CLK_LOCAL_MEM_FENCE | CLK_GLOBAL_MEM_FENCE);

        // end_integrate_velocity_verlet
        {
            nodes[index].velocity += 0.5f * (nodes[index].last_acceleration + nodes[index].current_acceleration) * dt;
        }
        barrier(CLK_LOCAL_MEM_FENCE | CLK_GLOBAL_MEM_FENCE);
    }
}

KERNEL void add(uint node_count, GLOBAL uint *a, GLOBAL uint *b, GLOBAL uint *result) {
    for (uint i = 0; i < node_count; i++) {
        result[i] = a[i] + b[i];
    }
}