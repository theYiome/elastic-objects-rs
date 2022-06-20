__kernel void main(
    read_only const float collision_dx,
    read_only const float collision_v0,
    read_only const ulong node_count,
    read_only const __global struct Node * const nodes, 
    read_only const __global ulong * const collisions_index, 
    read_only const __global ulong * const collisions, 
    read_only const __global ulong * const connections_index,
    read_only const __global struct Connection * const connections,
    write_only __global float2 *result
) {

    size_t i = get_global_id(0);

    if (i < node_count) {
        float2 acceleration = (float2)(0.0f, 0.0f);

        // Collisions
        acceleration -= collision_force(i, nodes, collisions_index, collisions);
        // Connections
        acceleration += connection_force(i, nodes, connections_index, connections);
        // Wall
        acceleration -= wall_force(i, nodes);
        // acceleration from nodes interactions
        acceleration /= nodes[i].mass;
        // drag
        acceleration -= drag_force(i);
        // gravity
        acceleration.y -= gravity_force(i);

        result[i] = acceleration;
    }

}