fn init() {
    this.id = "arm";
    this.name = "Arm";

    this.before = ["leg"];
    this.after = [];
}

fn on_damage(event) {
    if event.source == SourceTrinket && event.source.id != this.id {
        world.get_entity(event.source.entity_id)?.damage(1);
    }
}
