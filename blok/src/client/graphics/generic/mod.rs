pub use self::fragment_shader::*;
pub use self::instance::*;
pub use self::model::*;

use anyhow::Result;
use std::borrow::Borrow;

mod fragment_shader;
mod instance;
mod model;

pub struct Pipeline
{
}

impl Pipeline
{
    #[doc = crate::doc_safety_opengl!()]
    pub unsafe fn render<I, M, N>(models: I) -> Result<()>
        where I: IntoIterator<Item=(M, N)>
            , M: Borrow<Model>
            , N: Borrow<InstanceSet>
    {
        todo!()
    }
}
