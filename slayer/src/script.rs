use bevy::{
    asset::{AssetLoader, AsyncReadExt},
    prelude::*,
};
use rhai::{Engine, AST};
use thiserror::Error;

#[derive(Asset, TypePath, Debug)]
pub struct Script {
    pub ast: AST,
}

#[derive(Debug, Error)]
pub enum ScriptLoaderError {
    #[error("Failed to load script: {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to parse script: {0}")]
    Parse(#[from] rhai::ParseError),
}

#[derive(Default)]
pub struct ScriptLoader {}

impl AssetLoader for ScriptLoader {
    type Asset = Script;
    type Settings = ();
    type Error = ScriptLoaderError;

    fn extensions(&self) -> &[&str] {
        &["rhai"]
    }

    fn load<'a>(
        &'a self,
        reader: &'a mut bevy::asset::io::Reader,
        _settings: &'a Self::Settings,
        _load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let engine = Engine::new_raw();

            let mut script = String::new();
            reader.read_to_string(&mut script).await?;

            let ast = match engine.compile(script) {
                Ok(ast) => ast,
                Err(err) => {
                    error!("Failed to compile script: {}", err);
                    AST::default()
                }
            };

            Ok(Script { ast })
        })
    }
}
