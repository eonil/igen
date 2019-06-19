
use rustc::hir;
use rustc::hir::def_id::DefId;
use rustdoc::clean;
use super::Module;

impl Module {
    pub fn scan(ccrate: &clean::Crate) -> Module {
        let id_item_map = collect_defs(ccrate);
        /*
        match scan(ccrate) {
            Ok(ref kmod) => kmod,
            None => {
                panic!("Crate module is missing.");
            },
        }
        //println!("{:#?}", krate);
        */
    }
}

fn collect_defs(ccrate: &clean::Crate) -> HashMap<DefId,&Item> {
    if let Some(ref cmod) = ccrate.module {
        return collect_defs(cmod);
    }
    return HashMap::new();
}
fn collect_defs(citem: &clean::Item) -> HashMap<DefId,&Item> {
       
}

/*
fn scan(ccrate: &clean::Crate) -> Option<Module> {
    let mut kmod = Module::new();
    println!();
    let cmod = &ccrate.module?;
    match cmod.inner {
        StructItem(ref cstruct) => {
        },
        EnumItem(ref cenum) => {
        },
        ModuleItem(ref cmod) => {
        },
        _,
    }
}

fn scan(citem: &clean::Item) -> Item {
}
*/
