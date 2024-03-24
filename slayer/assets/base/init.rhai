trinkets["arm"] = #{
    name: "Arm",

    on_damage: |ev| {
        info(`${this} ${ev}`);
    },
};
