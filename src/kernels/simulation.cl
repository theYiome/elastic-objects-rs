// CUDA
#ifdef __CUDACC__
    #define GLOBAL
    #define KERNEL extern "C" __global__
// OpenCL
#else
    #define GLOBAL __global
    #define KERNEL __kernel
#endif


struct Connection {
    ulong j;
    float dx;
    float v0;
};

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

const float COLLISION_V0 = 10.0f;
const float COLLISION_DX = 0.015f;

const float WALL_V0 = 200.f;
const float WALL_DX = 0.05f;

KERNEL void main(
    const uint node_count,
    const GLOBAL struct Node * const nodes, 
    const GLOBAL ulong * const collisions_index, 
    const GLOBAL ulong * const collisions, 
    const GLOBAL ulong * const connections_index,
    const GLOBAL struct Connection * const connections,
    GLOBAL float2 *result
) {

    size_t i = get_global_id(0);

    if (i < node_count) {


        float2 acceleration = (float2)(0.0f, 0.0f);

        // Collisions
        {
            ulong collisions_index_start = 0;
            if (i > 0)
                collisions_index_start = collisions_index[i - 1];

            const ulong collisions_index_end = collisions_index[i];


            for (ulong c_i = collisions_index_start; c_i < collisions_index_end; c_i++) {
                const ulong j = collisions[c_i];

                float2 dir = nodes[j].position - nodes[i].position;
                float l = length(dir);
                float c = pown(COLLISION_DX / l, 13);
                acceleration -= (normalize(dir) * 3.0f * (COLLISION_V0 / COLLISION_DX) * c);
            }
        }


        // Connections
        {
            ulong connections_index_start = 0;
            if (i > 0)
                connections_index_start = connections_index[i - 1];
            
            const ulong connections_index_end = connections_index[i];

            for (ulong c_i = connections_index_start; c_i < connections_index_end; c_i++) {
                const ulong j = connections[c_i].j;
                const float dx = connections[c_i].dx;
                const float v0 = connections[c_i].v0;

                float2 dir = nodes[j].position - nodes[i].position;
                float l = length(dir);
                float c = pown(dx / l, 7) - pown(dx / l, 13);
                acceleration += (normalize(dir) * 3.0f * (v0 / dx) * c);
            }
        }

        // Wall
        {
            float2 dir = (float2)(0.0f, -1.f - nodes[i].position.y);
            float l = length(dir);
            float c = pown(WALL_DX / l, 13);
            acceleration -= (normalize(dir) * 3.0f * (WALL_V0 / WALL_DX) * c);
        }

        // Sum up acceleration
        result[i] += acceleration / nodes[i].mass;

        // Drag
        result[i] -= nodes[i].velocity * length(nodes[i].velocity) * nodes[i].drag;

        // Gravity
        result[i].y -= 9.81f;
    }

}