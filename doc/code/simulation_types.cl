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