use super::oumae_kumiko::Run;
use super::uji_bashi::Cry;

#[derive(Clone)]
pub enum Data {
    OumaeKumiko(Run),
    UjiBashi(Cry),
}

impl Data {
    pub fn oumae_kumiko(self) -> Result<Run, ()> {
        if let Self::OumaeKumiko(run) = self {
            return Ok(run);
        }
        Err(())
    }

    pub fn uji_bashi(self) -> Result<Cry, ()> {
        if let Self::UjiBashi(cry) = self {
            return Ok(cry);
        }
        Err(())
    }
}
