for(i = 0; i < ntime; ++i) {
    for(k = 0; k < Np; ++k) {
        V_kx = ...;
        V_ky = ...;

        j = i % n;
        V_reg[k][1][j] = V_kx / n_i;
        V_reg[k][2][j] = V_ky / n_i;

        V2_reg[k][1][j] = V_kx * V_kx / n_i;
        V2_reg[k][2][j] = V_ky * V_ky / n_i;
    }
    // obliczamy temperature dla k-tego wezla
    delta2_k_Vx = ...;
    delta2_k_Vy = ...;

    // dla k = 0, 1, ..., Np
    // n > 100
    T_k = (m/2)*(delta2_k_Vx + delta2_k_Vy)
}