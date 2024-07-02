use std::{
    fs::{self, File},
    io::{self, Read, Write},
    path::Path,
};

use syn::{visit::Visit, ImplItemFn};

struct PyiImplVisitor<'ast> {
    pyi_file: &'ast mut File,
    class_name: String,
    is_empty: bool,
}

impl<'ast> PyiImplVisitor<'ast> {
    fn write_fn_item(&mut self, fn_item: &ImplItemFn) {
        let fn_name = fn_item.sig.ident.to_string();

        // From conversions and as_ref are not mapped into python
        if fn_name == "from" || fn_name == "as_ref" {
            return;
        }

        write!(self.pyi_file, "\tdef {}(", fn_item.sig.ident.to_string()).unwrap();
        let mut is_first = true;

        for fn_arg in fn_item.sig.inputs.iter() {
            match fn_arg {
                syn::FnArg::Receiver(_) => {
                    write!(self.pyi_file, "self").unwrap();
                    is_first = false;
                }
                syn::FnArg::Typed(t) => {
                    if !is_first {
                        write!(self.pyi_file, ", ").unwrap();
                    }
                    match t.pat.as_ref() {
                        syn::Pat::Ident(i) => {
                            write!(self.pyi_file, "{}", i.ident).unwrap();
                        }
                        _ => unimplemented!(),
                    };
                    write!(self.pyi_file, ": ").unwrap();
                    self.write_type(t.ty.as_ref());
                    is_first = false;
                }
            }
        }

        write!(self.pyi_file, "): ...\n").unwrap();

        self.is_empty = false;
    }

    fn write_type(&mut self, type_path: &syn::Type) {
        match type_path {
            syn::Type::Path(p) => {
                // If the TypePath has an ident it directly represents a type, otherwise
                // it is a generic
                if let Some(i) = p.path.get_ident() {
                    let type_name = match i.to_string().as_str() {
                        "u8" | "i8" | "u16" | "i16" | "u32" | "i32" | "u64" | "i64" => {
                            "int".to_string()
                        }
                        "String" => "str".to_string(),
                        _ => i.to_string(),
                    };
                    write!(self.pyi_file, "{}", type_name).unwrap();
                } else {
                    let type_name = p.path.segments[0].ident.to_string();
                    // write!(self.pyi_file, ": {:?}", p.path.segments[0]).unwrap();
                    match type_name.as_str() {
                        "Option" => match &p.path.segments[0].arguments {
                            syn::PathArguments::AngleBracketed(g) => match &g.args[0] {
                                syn::GenericArgument::Type(t) => self.write_type(t),
                                _ => unimplemented!(),
                            },
                            _ => unimplemented!(),
                        },
                        "Vec" => {
                            write!(self.pyi_file, "list[").unwrap();
                            match &p.path.segments[0].arguments {
                                syn::PathArguments::AngleBracketed(g) => match &g.args[0] {
                                    syn::GenericArgument::Type(t) => self.write_type(t),
                                    _ => unimplemented!(),
                                },
                                _ => unimplemented!(),
                            }
                            write!(self.pyi_file, "]").unwrap();
                        }
                        "Py" => write!(self.pyi_file, "Any").unwrap(),
                        _ => {
                            unimplemented!();
                        }
                    }
                }
            }
            syn::Type::Array(_) => todo!(),
            syn::Type::BareFn(_) => todo!(),
            syn::Type::Group(_) => todo!(),
            syn::Type::ImplTrait(_) => todo!(),
            syn::Type::Infer(_) => todo!(),
            syn::Type::Macro(_) => todo!(),
            syn::Type::Never(_) => todo!(),
            syn::Type::Paren(_) => todo!(),
            syn::Type::Ptr(_) => todo!(),
            syn::Type::Reference(r) => self.write_type(r.elem.as_ref()),
            syn::Type::Slice(_) => todo!(),
            syn::Type::TraitObject(_) => todo!(),
            syn::Type::Tuple(_) => todo!(),
            syn::Type::Verbatim(_) => todo!(),
            _ => todo!(),
        }
    }
}

impl<'ast> Visit<'ast> for PyiImplVisitor<'ast> {
    fn visit_item_impl(&mut self, i: &'ast syn::ItemImpl) {
        if let syn::Type::Path(impl_path) = i.self_ty.as_ref() {
            if let Some(path_ident) = impl_path.path.get_ident() {
                if path_ident.to_string() == self.class_name {
                    for fn_item in i.items.iter().filter_map(|i| match i {
                        syn::ImplItem::Fn(f) => Some(f),
                        _ => None,
                    }) {
                        self.write_fn_item(fn_item)
                    }
                }
            }
        }
    }
}

struct PyiStructVisitor<'ast> {
    pyi_file: &'ast mut fs::File,
    ast_file: &'ast syn::File,
}

impl<'ast> Visit<'ast> for PyiStructVisitor<'ast> {
    fn visit_item_struct(&mut self, node: &'ast syn::ItemStruct) {
        // Check if struct has attribute #[pyclass]
        if node
            .attrs
            .iter()
            .filter_map(|a| a.meta.path().get_ident())
            .find(|&i| i.to_string() == "pyclass")
            .is_some()
        {
            let class_name = node.ident.to_string();
            write!(self.pyi_file, "\n\nclass {}: \n", class_name).unwrap();
            let mut impl_visitor = PyiImplVisitor {
                pyi_file: self.pyi_file,
                class_name: node.ident.to_string(),
                is_empty: true,
            };
            impl_visitor.visit_file(self.ast_file);
            if impl_visitor.is_empty {
                write!(self.pyi_file, "\tpass\n").unwrap();
            }
        }
    }
}

fn is_rs_file(file: &Path) -> bool {
    if let Some(extension) = file.extension() {
        extension == "rs"
    } else {
        false
    }
}

fn visit_fs_file(file: &Path, pyi_file: &mut File) -> io::Result<()> {
    if is_rs_file(file) {
        let mut file = File::open(file)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        if let Ok(ast) = syn::parse_file(&content) {
            PyiStructVisitor {
                pyi_file,
                ast_file: &ast,
            }
            .visit_file(&ast);
        }
    }
    Ok(())
}

fn visit_fs_dir(dir: &Path, pyi_file: &mut File) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_fs_dir(&path, pyi_file)?;
            } else if path.is_file() {
                visit_fs_file(&path, pyi_file)?;
            }
        }
    }
    Ok(())
}

fn main() -> io::Result<()> {
    let cargo_dir = std::env::var("CARGO_MANIFEST_DIR").expect("Variable should exist");
    let cargo_dir_path = Path::new(&cargo_dir);
    let mut pyi_file = File::create(cargo_dir_path.join("dust_dds.pyi"))?;

    write!(pyi_file, "from typing import Any \n").unwrap();

    visit_fs_dir(cargo_dir_path, &mut pyi_file)?;

    Ok(())
}
