use clap::ArgMatches;
use std::path::{PathBuf, Path};
use std::collections::HashMap;
use nitro::{Name, Model, Texture, Palette, Animation};
use errors::Result;
use util::cur::Cur;

type TextureId = usize;
type PaletteId = usize;

#[derive(Default)]
pub struct Database {
    pub file_paths: Vec<PathBuf>,

    pub models: Vec<Model>,
    pub textures: Vec<Texture>,
    pub palettes: Vec<Palette>,
    pub animations: Vec<Animation>,

    pub textures_by_name: HashMap<Name, TextureId>,
    pub palettes_by_name: HashMap<Name, PaletteId>,
}

impl Database {
    pub fn build(file_paths: Vec<PathBuf>) -> Result<Database> {
        use std::default::Default;

        let mut db: Database = Default::default();
        db.file_paths = file_paths;

        for path in &db.file_paths {
            let buf = read_file(&path)?;
            let cur = Cur::new(&buf);

            use nitro::container::read_container;
            match read_container(cur) {
                Ok(cont) => {
                    db.models.extend(cont.models.into_iter());
                    db.textures.extend(cont.textures.into_iter());
                    db.palettes.extend(cont.palettes.into_iter());
                    db.animations.extend(cont.animations.into_iter());
                }
                Err(e) => {
                    error!("error in file {}: {}", path.to_string_lossy(), e);
                }
            }
        }

        db.build_by_name_maps();

        Ok(db)
    }

    /// Fill out `textures_by_name` and `palettes_by_name`.
    fn build_by_name_maps(&mut self) {
        use std::collections::hash_map::Entry::*;

        let mut name_clash = false;
        for (id, texture) in self.textures.iter().enumerate() {
            match self.textures_by_name.entry(texture.name) {
                Vacant(ve) => { ve.insert(id); },
                Occupied(_) => {
                    warn!("multiple textures have the name {}", texture.name);
                    name_clash = true;
                }
            }
        }

        for (id, palette) in self.palettes.iter().enumerate() {
            match self.palettes_by_name.entry(palette.name) {
                Vacant(ve) => { ve.insert(id); },
                Occupied(_) => {
                    warn!("multiple palettes have the name {}", palette.name);
                    name_clash = true;
                }
            }
        }

        if name_clash {
            warn!("since there were name clashes, some textures might be wrong");
        }
    }

    pub fn from_arg_matches(matches: &ArgMatches) -> Result<Database> {
        let file_paths: Vec<PathBuf> =
            matches
            .values_of_os("INPUT").unwrap()
            .map(|os_str| PathBuf::from(os_str))
            .collect();
        Database::build(file_paths)
    }
}

fn read_file(path: &Path) -> Result<Vec<u8>> {
    use std::{fs::File, io::Read};
    let mut f = File::open(&path)?;
    let mut b: Vec<u8> = vec![];
    f.read_to_end(&mut b)?;
    Ok(b)
}