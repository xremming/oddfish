use std::{borrow::Borrow, cell::RefCell, rc::Rc};

use bevy::{prelude::*, utils::HashMap};
use rhai::{CallFnOptions, Dynamic, Engine, FnPtr, Map, Scope};
use slayer::script;

#[derive(Component)]
struct Trinket {
    script: Handle<script::Script>,
}

fn startup(mut commands: Commands, assets: Res<AssetServer>) {
    let script = assets.load::<script::Script>("base/init.rhai");
    commands.spawn(Trinket { script });
}

#[derive(Resource, Default, Clone, Debug)]
struct Trinkets {
    data: HashMap<String, Map>,
}

impl Trinkets {
    fn set(&mut self, key: &str, value: Map) {
        self.data.entry(key.to_string()).insert(value);
    }
}

fn update(trinkets: Query<&Trinket>, script_assets: Res<Assets<script::Script>>) {
    for trinket in trinkets.iter() {
        let trinket = script_assets.get(&trinket.script);

        if let Some(trinket) = trinket {
            info!("trinket = {:?}", trinket);

            let mut engine = Engine::new_raw();
            engine
                .register_fn("info", |s: &str| info!("Script info: {}", s))
                .register_type::<Trinkets>()
                .register_indexer_set(Trinkets::set);

            let mut scope = Scope::new();
            scope.push("trinkets", Trinkets::default());

            engine.run_ast_with_scope(&mut scope, &trinket.ast).unwrap();

            let trinkets = scope.get_value::<Trinkets>("trinkets").unwrap();
            info!("trinkets = {:?}", trinkets);

            let arm = &trinkets.data["arm"];

            let on_damage_fn = &arm["on_damage"].clone().try_cast::<FnPtr>().unwrap();
            info!("on_damage_fn = {:?}", on_damage_fn);

            let mut this = Dynamic::TRUE;
            engine
                .call_fn_with_options::<()>(
                    CallFnOptions::new()
                        .bind_this_ptr(&mut this)
                        .rewind_scope(true),
                    &mut scope,
                    &trinket.ast,
                    on_damage_fn.fn_name(),
                    (123,),
                )
                .unwrap();
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_asset::<script::Script>()
        .init_asset_loader::<script::ScriptLoader>()
        .add_systems(Startup, startup)
        .add_systems(Update, update)
        .run();
}
